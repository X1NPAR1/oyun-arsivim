//! Oyun süresi takipçisi: çalışan süreçleri kütüphanedeki oyunlarla eşler, oturum ve saatlik
//! (aktif / AFK) süreleri yazar. Uygulama tepsideyken de çalışır.
//!
//! - Eşleme: Steam/Epic için süreç exe'si oyunun kurulum klasörü altında mı (en uzun önek kazanır);
//!   elle eklenenlerde yalnız tam exe yolu (sistem klasöründeki bir exe her süreci oyun saymasın).
//! - AFK: klavye/fare VE gamepad (XInput) girdisi `IDLE_AFTER` süresince yoksa süre AFK sayılır.
//! - Uyku/askıya alma: iki ölçüm arası `MAX_GAP`'ten uzunsa aradaki süre hiç sayılmaz.
//! - Kesinti toleransı: süreç `GRACE_TICKS` ölçüm boyunca görünmezse oturum kapanır (yeniden başlatan
//!   başlatıcılar oturumu bölmesin).
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use rusqlite::Connection;
use tauri::{AppHandle, Emitter};

use crate::commands::{now, SharedState, EVENT_LIBRARY_CHANGED, EVENT_RUNNING_CHANGED};
use crate::detectors::{steam, steam_playtime, Platform};
use crate::library::{self, Game};
use crate::perf::boost::{self, BoostManager, BoostOptions, RunningGames};
use crate::perf::memory;
use crate::playtime;
use crate::settings;

pub const IDLE_AFTER_SECS: u64 = 180;
const MAX_GAP_SECS: i64 = 45;
const GRACE_TICKS: u32 = 2;
const POLL_ACTIVE: Duration = Duration::from_secs(5);
const POLL_IDLE: Duration = Duration::from_secs(10);
const MATCHER_REFRESH_SECS: i64 = 60;
/// Steam localconfig.vdf'yi oyun kapandıktan sonra yazar; biraz bekleyip yeniden okuruz.
const STEAM_REIMPORT_DELAY_SECS: i64 = 15;

fn norm(p: &str) -> String {
    p.replace('/', "\\").to_lowercase()
}

#[derive(Debug, Default)]
pub struct GameMatcher {
    /// (normalize klasör + "\\", oyun id) — uzundan kısaya sıralı.
    dirs: Vec<(String, i64)>,
    exes: HashMap<String, i64>,
}

impl GameMatcher {
    pub fn new(games: &[Game]) -> Self {
        let mut dirs = Vec::new();
        let mut exes = HashMap::new();
        for g in games.iter().filter(|g| g.installed) {
            match g.platform {
                Platform::Manual => {
                    if let Some(exe) = &g.executable {
                        exes.insert(norm(exe), g.id);
                    }
                }
                _ => {
                    let mut dir = norm(g.install_dir.trim_end_matches(['\\', '/']));
                    // "C:" gibi kök klasörler her şeyi eşlerdi; güvenlik için reddedilir.
                    if dir.len() > 3 {
                        dir.push('\\');
                        dirs.push((dir, g.id));
                    }
                }
            }
        }
        dirs.sort_by_key(|d| std::cmp::Reverse(d.0.len()));
        Self { dirs, exes }
    }

    pub fn match_exe(&self, exe: &str) -> Option<i64> {
        let p = norm(exe);
        if let Some(id) = self.exes.get(&p) {
            return Some(*id);
        }
        self.dirs
            .iter()
            .find(|(d, _)| p.starts_with(d))
            .map(|(_, id)| *id)
    }
}

struct Active {
    session_id: i64,
    missing: u32,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct TickOutcome {
    pub started: Vec<i64>,
    pub ended: Vec<i64>,
}

/// Saf durum makinesi: zaman ve girdiler dışarıdan verilir (testte simüle edilir).
#[derive(Default)]
pub struct TrackerCore {
    active: HashMap<i64, Active>,
    last_tick: Option<i64>,
}

impl TrackerCore {
    pub fn is_tracking(&self) -> bool {
        !self.active.is_empty()
    }

    pub fn tick(
        &mut self,
        conn: &Connection,
        now: i64,
        running: &HashSet<i64>,
        idle: bool,
    ) -> rusqlite::Result<TickOutcome> {
        let mut out = TickOutcome::default();
        let prev = self.last_tick.replace(now);
        // Uyku/askıya alma veya ilk ölçüm: aradaki süre sayılmaz.
        let credited_from = prev.filter(|p| now > *p && now - *p <= MAX_GAP_SECS);

        for &game in running {
            match self.active.get_mut(&game) {
                Some(a) => {
                    a.missing = 0;
                    if let Some(from) = credited_from {
                        let secs = now - from;
                        playtime::add_interval(conn, game, from, now, idle)?;
                        let (act, idl) = if idle { (0, secs) } else { (secs, 0) };
                        playtime::extend_session(conn, a.session_id, now, act, idl)?;
                    }
                }
                None => {
                    let session_id = playtime::open_session(conn, game, now)?;
                    self.active.insert(
                        game,
                        Active {
                            session_id,
                            missing: 0,
                        },
                    );
                    out.started.push(game);
                }
            }
        }

        let gone: Vec<i64> = self
            .active
            .iter_mut()
            .filter(|(id, _)| !running.contains(id))
            .filter_map(|(id, a)| {
                a.missing += 1;
                (a.missing >= GRACE_TICKS).then_some(*id)
            })
            .collect();
        for game in gone {
            if let Some(a) = self.active.remove(&game) {
                playtime::close_session(conn, a.session_id)?;
                out.ended.push(game);
            }
        }
        Ok(out)
    }
}

// ───────────── Sistem girdisi ─────────────

#[cfg(windows)]
mod input {
    use windows::Win32::System::SystemInformation::GetTickCount;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
    use windows::Win32::UI::Input::XboxController::{XInputGetState, XINPUT_STATE};

    /// Klavye/fare için son girdiden bu yana geçen süre.
    pub fn idle_seconds() -> u64 {
        let mut info = LASTINPUTINFO {
            cbSize: u32::try_from(std::mem::size_of::<LASTINPUTINFO>()).unwrap_or(8),
            dwTime: 0,
        };
        // SAFETY: info doğru boyutla başlatıldı.
        if !unsafe { GetLastInputInfo(&mut info) }.as_bool() {
            return 0;
        }
        // SAFETY: parametresiz sistem çağrısı. Her ikisi de 49 günde taşan u32; wrapping_sub doğru farkı verir.
        let now = unsafe { GetTickCount() };
        u64::from(now.wrapping_sub(info.dwTime)) / 1000
    }

    /// Gamepad'lerin paket numaraları; değişmişse kol kullanılıyordur.
    pub fn gamepad_packets() -> [u32; 4] {
        let mut out = [0u32; 4];
        for (i, slot) in out.iter_mut().enumerate() {
            let mut state = XINPUT_STATE::default();
            // SAFETY: state geçerli bir çıkış arabelleği; bağlı olmayan kol hata kodu döndürür.
            let rc = unsafe { XInputGetState(u32::try_from(i).unwrap_or(0), &mut state) };
            if rc == 0 {
                *slot = state.dwPacketNumber;
            }
        }
        out
    }
}

#[cfg(not(windows))]
mod input {
    pub fn idle_seconds() -> u64 {
        0
    }
    pub fn gamepad_packets() -> [u32; 4] {
        [0; 4]
    }
}

/// Klavye/fare + gamepad birlikte değerlendirilerek AFK durumu.
struct InputMonitor {
    last_packets: [u32; 4],
    last_pad_activity: i64,
}

impl InputMonitor {
    fn new(now: i64) -> Self {
        Self {
            last_packets: input::gamepad_packets(),
            last_pad_activity: now,
        }
    }

    fn is_idle(&mut self, now: i64) -> bool {
        let packets = input::gamepad_packets();
        if packets != self.last_packets {
            self.last_packets = packets;
            self.last_pad_activity = now;
        }
        let pad_idle = u64::try_from(now - self.last_pad_activity).unwrap_or(0);
        input::idle_seconds().min(pad_idle) >= IDLE_AFTER_SECS
    }
}

fn running_games(sys: &mut sysinfo::System, matcher: &GameMatcher) -> RunningGames {
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, UpdateKind};
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
    );
    let mut out: RunningGames = HashMap::new();
    for p in sys.processes().values() {
        if let Some(id) = p
            .exe()
            .and_then(|exe| matcher.match_exe(&exe.to_string_lossy()))
        {
            out.entry(id).or_default().push(p.pid().as_u32());
        }
    }
    out
}

/// Otomatik RAM temizliği en sık bu aralıkta yapılır.
const AUTO_CLEAN_MIN_INTERVAL_SECS: i64 = 300;

fn reimport_steam(conn: &Connection) {
    if let Some(root) = steam::steam_root_from_registry() {
        let data = steam_playtime::read_all(&root);
        if let Err(err) = playtime::apply_steam_import(conn, &data) {
            tracing::warn!(%err, "Steam süreleri yazılamadı");
        }
    }
}

/// Eşleyici hiç kurulmadıysa veya süresi dolduysa yenilenir. (Önceden `i64::MIN` başlangıç değeri
/// çıkarmada taşıyordu: release'te eşleyici hiç kurulmuyor, debug'da iş parçacığı çöküyordu.)
fn matcher_due(last: Option<i64>, now: i64) -> bool {
    last.is_none_or(|l| now.saturating_sub(l) >= MATCHER_REFRESH_SECS)
}

/// Takipçiyi arka plan iş parçacığında başlatır. `perf_state`: performans geri alma durum dosyası.
pub fn spawn(app: AppHandle, state: SharedState, perf_state: std::path::PathBuf) {
    std::thread::Builder::new()
        .name("oyun-takipci".into())
        .spawn(move || {
            if let Ok(conn) = state.db.lock() {
                let _ = playtime::close_stale_sessions(&conn);
                reimport_steam(&conn);
            }
            let mut core = TrackerCore::default();
            let mut sys = sysinfo::System::new();
            let mut input = InputMonitor::new(now());
            let mut matcher = GameMatcher::default();
            let mut matcher_at: Option<i64> = None;
            let mut steam_reimport_at: Option<i64> = None;
            let mut boost = BoostManager::new(perf_state);
            let mut opts = BoostOptions::default();
            let mut prefs = settings::Settings::default();
            let mut last_auto_clean = i64::MIN / 2;
            let mut ram_sys = sysinfo::System::new();
            if let Ok(conn) = state.db.lock() {
                prefs = settings::load(&conn);
                opts = BoostOptions { close_list: prefs.close_list.clone(), restart_closed_apps: prefs.restart_closed_apps };
            }
            // Önceki oturum çöktüyse performans değişikliklerini geri al.
            boost.recover(&opts);
            loop {
                let t = now();
                if matcher_due(matcher_at, t) {
                    if let Ok(conn) = state.db.lock() {
                        let games = library::list(&conn).unwrap_or_default();
                        matcher = GameMatcher::new(&games);
                        if let Ok(mut r) = state.running.lock() {
                            r.set_titles(games.iter().map(|g| (g.id, g.title.clone())).collect());
                        }
                        prefs = settings::load(&conn);
                        opts = BoostOptions { close_list: prefs.close_list.clone(), restart_closed_apps: prefs.restart_closed_apps };
                    }
                    matcher_at = Some(t);
                    // Otomatik RAM temizliği (eşik + en fazla 5 dakikada bir).
                    if prefs.auto_ram_clean && t - last_auto_clean >= AUTO_CLEAN_MIN_INTERVAL_SECS {
                        ram_sys.refresh_memory_specifics(sysinfo::MemoryRefreshKind::nothing().with_ram());
                        if memory::should_auto_clean(ram_sys.total_memory(), ram_sys.available_memory(), prefs.auto_ram_threshold) {
                            let r = memory::trim_working_sets();
                            tracing::info!(freed_mb = r.freed_bytes / 1_048_576, "otomatik RAM temizliği");
                            last_auto_clean = t;
                        }
                    }
                }
                let running_map = running_games(&mut sys, &matcher);
                let running: HashSet<i64> = running_map.keys().copied().collect();
                let changed = state.running.lock().ok().and_then(|mut r| r.update(t, &running_map).then(|| r.snapshot(t)));
                if let Some(snapshot) = changed {
                    let _ = app.emit(EVENT_RUNNING_CHANGED, snapshot);
                }
                let idle = input.is_idle(t);
                let outcome = match state.db.lock() {
                    Ok(conn) => {
                        let r = core.tick(&conn, t, &running, idle);
                        if steam_reimport_at.is_some_and(|due| t >= due) {
                            reimport_steam(&conn);
                            steam_reimport_at = None;
                            let _ = app.emit(EVENT_LIBRARY_CHANGED, ());
                        }
                        r
                    }
                    Err(_) => Ok(TickOutcome::default()),
                };
                match outcome {
                    Ok(o) if !o.started.is_empty() || !o.ended.is_empty() => {
                        tracing::info!(started = ?o.started, ended = ?o.ended, "oyun oturumu değişti");
                        if !o.ended.is_empty() {
                            steam_reimport_at = Some(t + STEAM_REIMPORT_DELAY_SECS);
                        }
                        for game in &o.started {
                            let profile = state
                                .db
                                .lock()
                                .ok()
                                .and_then(|c| boost::load_profile(&c, *game).ok())
                                .unwrap_or_default();
                            let pids = running_map.get(game).cloned().unwrap_or_default();
                            boost.on_game_started(*game, &profile, &pids, &opts);
                        }
                        for game in &o.ended {
                            boost.on_game_ended(*game, &opts);
                        }
                        let _ = app.emit(EVENT_LIBRARY_CHANGED, ());
                    }
                    Ok(_) => {}
                    Err(err) => tracing::warn!(%err, "takip verisi yazılamadı"),
                }
                // Başlatıcı üzerinden açılan oyunların sonradan doğan süreçlerine de öncelik ver.
                if boost.is_active() {
                    let pids: Vec<u32> = running_map.values().flatten().copied().collect();
                    boost.boost_new_pids(&pids);
                }
                std::thread::sleep(if core.is_tracking() { POLL_ACTIVE } else { POLL_IDLE });
            }
        })
        .map(|_| ())
        .unwrap_or_else(|err| tracing::error!(%err, "takipçi başlatılamadı"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::detectors::DetectedGame;

    fn game(platform: Platform, id: i64, dir: &str, exe: Option<&str>) -> Game {
        Game {
            id,
            platform,
            platform_id: id.to_string(),
            title: format!("G{id}"),
            install_dir: dir.into(),
            executable: exe.map(Into::into),
            launch_uri: String::new(),
            launch_args: None,
            size_bytes: None,
            last_played: None,
            installed: true,
            added_at: 0,
            removed_at: None,
        }
    }

    #[test]
    fn matcher_prefers_longest_dir_and_exact_manual_exe() {
        let m = GameMatcher::new(&[
            game(
                Platform::Steam,
                1,
                "D:\\SteamLibrary\\steamapps\\common\\GarrysMod",
                None,
            ),
            game(
                Platform::Steam,
                2,
                "D:\\SteamLibrary\\steamapps\\common\\GarrysMod\\sub",
                None,
            ),
            game(Platform::Epic, 3, "F:/GTAV/", None),
            game(
                Platform::Manual,
                4,
                "C:\\Windows\\System32",
                Some("C:\\Windows\\System32\\notepad.exe"),
            ),
            game(Platform::Steam, 5, "C:\\", None),
        ]);
        assert_eq!(
            m.match_exe("d:\\steamlibrary\\steamapps\\common\\garrysmod\\hl2.exe"),
            Some(1)
        );
        assert_eq!(
            m.match_exe("D:\\SteamLibrary\\steamapps\\common\\GarrysMod\\sub\\x.exe"),
            Some(2)
        );
        assert_eq!(m.match_exe("F:\\GTAV\\GTA5.exe"), Some(3));
        assert_eq!(m.match_exe("C:\\Windows\\System32\\NOTEPAD.EXE"), Some(4));
        assert_eq!(
            m.match_exe("C:\\Windows\\System32\\svchost.exe"),
            None,
            "manuel oyunun klasörü eşlenmez"
        );
        assert_eq!(
            m.match_exe("D:\\SteamLibrary\\steamapps\\common\\GarrysModX\\a.exe"),
            None
        );
        assert_eq!(m.match_exe("C:\\x\\y.exe"), None, "kök klasör reddedilir");
    }

    fn setup() -> (Connection, i64) {
        let mut conn = db::open_in_memory().unwrap();
        let g = DetectedGame {
            launch_args: None,
            platform: Platform::Steam,
            platform_id: "730".into(),
            title: "CS2".into(),
            install_dir: "D:\\cs".into(),
            executable: None,
            launch_uri: String::new(),
            size_bytes: None,
            last_played: None,
        };
        library::sync(&mut conn, &[g], &[Platform::Steam], 1).unwrap();
        let id = library::list(&conn).unwrap()[0].id;
        (conn, id)
    }

    #[test]
    fn tracks_active_and_idle_time_and_closes_after_grace() {
        let (conn, id) = setup();
        let mut core = TrackerCore::default();
        let on: HashSet<i64> = [id].into();
        let off = HashSet::new();
        let t0 = 10_000 * 3600;

        assert_eq!(core.tick(&conn, t0, &on, false).unwrap().started, vec![id]);
        core.tick(&conn, t0 + 5, &on, false).unwrap(); // +5 aktif
        core.tick(&conn, t0 + 10, &on, true).unwrap(); // +5 AFK
        core.tick(&conn, t0 + 600, &on, false).unwrap(); // uyku: 590 sn sayılmaz
        core.tick(&conn, t0 + 605, &on, false).unwrap(); // +5 aktif
                                                         // Kısa kesinti: 1 ölçüm görünmez → oturum sürer.
        assert!(core
            .tick(&conn, t0 + 610, &off, false)
            .unwrap()
            .ended
            .is_empty());
        core.tick(&conn, t0 + 615, &on, false).unwrap(); // geri geldi, +5
        core.tick(&conn, t0 + 620, &off, false).unwrap();
        let out = core.tick(&conn, t0 + 625, &off, false).unwrap();
        assert_eq!(out.ended, vec![id]);
        assert!(!core.is_tracking());

        let rows = playtime::hourly_rows(&conn, 0, i64::MAX, Some(id)).unwrap();
        let active: i64 = rows.iter().map(|r| r.active_seconds).sum();
        let idle: i64 = rows.iter().map(|r| r.idle_seconds).sum();
        assert_eq!((active, idle), (15, 5));
        // Toplam 20 sn < 30 sn: oturum çok kısa sayılıp silinir, saatlik süre korunur.
        assert!(playtime::sessions_between(&conn, 0, i64::MAX, None)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn long_session_is_kept() {
        let (conn, id) = setup();
        let mut core = TrackerCore::default();
        let on: HashSet<i64> = [id].into();
        let t0 = 5_000_000;
        for i in 0..=12 {
            core.tick(&conn, t0 + i * 5, &on, false).unwrap();
        }
        let off = HashSet::new();
        core.tick(&conn, t0 + 65, &off, false).unwrap();
        core.tick(&conn, t0 + 70, &off, false).unwrap();
        let s = playtime::sessions_between(&conn, 0, i64::MAX, Some(id)).unwrap();
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].active_seconds, 60);
        assert_eq!(s[0].started_at, t0);
    }

    #[test]
    fn matcher_refresh_schedule_never_overflows() {
        assert!(matcher_due(None, 0));
        assert!(matcher_due(None, i64::MIN));
        assert!(!matcher_due(Some(100), 159));
        assert!(matcher_due(Some(100), 160));
        assert!(matcher_due(Some(i64::MIN), i64::MAX));
    }

    #[cfg(windows)]
    #[test]
    fn real_input_apis_respond() {
        // Gerçek API'ler hata vermeden çalışmalı (değerleri ortama bağlı).
        let _ = input::idle_seconds();
        let _ = input::gamepad_packets();
    }
}

#[cfg(test)]
mod real_system {
    use super::*;

    /// Elle: test oyunu çalışırken `cargo test tracker::real_system -- --ignored --nocapture`.
    #[test]
    #[ignore = "gerçek veritabanı ve süreçleri okur"]
    fn diagnose_matching() {
        let db = std::path::PathBuf::from(std::env::var("LOCALAPPDATA").unwrap())
            .join("com.oyunarsivim.app/library.sqlite");
        let conn = crate::db::open(&db).unwrap();
        let games = library::list(&conn).unwrap();
        for g in games.iter().filter(|g| g.platform == Platform::Manual) {
            println!(
                "manuel: {:?} kurulu={} exe={:?}",
                g.title, g.installed, g.executable
            );
        }
        let matcher = GameMatcher::new(&games);
        let mut sys = sysinfo::System::new();
        println!("eşleşen: {:?}", running_games(&mut sys, &matcher));
        use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, UpdateKind};
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_exe(UpdateKind::Always),
        );
        for p in sys.processes().values() {
            if p.name().to_string_lossy().to_lowercase().contains("sahin") {
                println!("süreç: {:?} exe={:?}", p.name(), p.exe());
            }
        }
    }
}
