//! Oyuna özel performans profili: oyun başlayınca uygulanır, son oyun kapanınca geri alınır.
//!
//! Geri alınacak her şey (önceki güç planı, bildirim durumu, kapatılan uygulamalar) önce bir durum
//! dosyasına (`perf-state.json`) yazılır; uygulama çökse bile bir sonraki açılışta geri alınır.
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use super::{memory, power, processes, tweaks};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PerfProfile {
    pub enabled: bool,
    pub high_priority: bool,
    pub power_plan: bool,
    pub clean_ram: bool,
    pub close_apps: bool,
    pub mute_notifications: bool,
}

impl PerfProfile {
    /// "Performans modunu aç" denince önerilen tüm iyileştirmeler.
    pub fn recommended() -> Self {
        Self {
            enabled: true,
            high_priority: true,
            power_plan: true,
            clean_ram: true,
            close_apps: false,
            mute_notifications: true,
        }
    }
}

pub fn load_profile(conn: &Connection, game_id: i64) -> rusqlite::Result<PerfProfile> {
    let json: Option<String> = conn
        .query_row(
            "SELECT profile FROM perf_profiles WHERE game_id = ?1",
            [game_id],
            |r| r.get(0),
        )
        .optional()?;
    Ok(json
        .and_then(|j| serde_json::from_str(&j).ok())
        .unwrap_or_default())
}

pub fn save_profile(conn: &Connection, game_id: i64, p: &PerfProfile) -> rusqlite::Result<()> {
    let json = serde_json::to_string(p).unwrap_or_else(|_| "{}".into());
    conn.execute(
        "INSERT INTO perf_profiles (game_id, profile) VALUES (?1, ?2)
         ON CONFLICT(game_id) DO UPDATE SET profile = excluded.profile",
        params![game_id, json],
    )?;
    Ok(())
}

/// Geri alınacak sistem durumu (diske yazılır).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RevertState {
    pub previous_power_plan: Option<String>,
    pub notifications_were_on: Option<bool>,
    /// Kapatılan uygulamaların exe yolları (isteğe bağlı yeniden başlatma için).
    pub closed_apps: Vec<String>,
}

impl RevertState {
    fn is_empty(&self) -> bool {
        self.previous_power_plan.is_none()
            && self.notifications_were_on.is_none()
            && self.closed_apps.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoostReport {
    pub power_plan: Option<String>,
    pub ram_freed_bytes: u64,
    pub apps_closed: Vec<String>,
    pub notifications_muted: bool,
}

/// Global performans tercihleri (ayarlardan).
#[derive(Debug, Clone, Default)]
pub struct BoostOptions {
    pub close_list: Vec<String>,
    pub restart_closed_apps: bool,
}

pub struct BoostManager {
    state_file: PathBuf,
    /// Profil etkin olan ve şu an çalışan oyunlar.
    active_games: HashSet<i64>,
    /// Önceliği yükseltilmiş süreçler (başlatıcı sonradan yeni süreç açarsa onlara da uygulanır).
    boosted_pids: HashSet<u32>,
    state: RevertState,
}

impl BoostManager {
    pub fn new(state_file: PathBuf) -> Self {
        Self {
            state_file,
            active_games: HashSet::new(),
            boosted_pids: HashSet::new(),
            state: RevertState::default(),
        }
    }

    fn persist(&self) {
        if self.state.is_empty() {
            let _ = std::fs::remove_file(&self.state_file);
        } else if let Ok(json) = serde_json::to_vec_pretty(&self.state) {
            let _ = std::fs::write(&self.state_file, json);
        }
    }

    /// Önceki oturumdan (çökme) kalan değişiklikleri geri alır.
    pub fn recover(&mut self, opts: &BoostOptions) -> bool {
        let Ok(bytes) = std::fs::read(&self.state_file) else {
            return false;
        };
        if let Ok(state) = serde_json::from_slice::<RevertState>(&bytes) {
            self.state = state;
            tracing::warn!("önceki oturumdan kalan performans değişiklikleri geri alınıyor");
            self.revert(opts);
        }
        let _ = std::fs::remove_file(&self.state_file);
        true
    }

    pub fn is_active(&self) -> bool {
        !self.active_games.is_empty()
    }

    /// Bir oyun başladığında profili uygular. İlk etkin oyunda sistem değişiklikleri yapılır.
    pub fn on_game_started(
        &mut self,
        game_id: i64,
        profile: &PerfProfile,
        pids: &[u32],
        opts: &BoostOptions,
    ) -> BoostReport {
        let mut report = BoostReport::default();
        if !profile.enabled {
            return report;
        }
        let first = self.active_games.is_empty();
        self.active_games.insert(game_id);
        if profile.high_priority {
            self.boost_new_pids(pids);
        }
        if !first {
            return report; // sistem geneli ayarlar zaten uygulandı
        }
        if profile.power_plan {
            let schemes = power::list();
            if let Some(target) = power::pick_game_scheme(&schemes) {
                let current = power::active_guid();
                if current.as_deref() != Some(target.guid.as_str())
                    && power::set_active(&target.guid)
                {
                    self.state.previous_power_plan = current;
                    report.power_plan = Some(target.name);
                }
            }
        }
        if profile.mute_notifications {
            if let Some(t) = tweaks::find("notifications") {
                if tweaks::current(t) == Some(true) && tweaks::apply_direct(t, false).is_ok() {
                    self.state.notifications_were_on = Some(true);
                    report.notifications_muted = true;
                }
            }
        }
        if profile.close_apps && !opts.close_list.is_empty() {
            let paths = processes::exe_paths(&opts.close_list);
            let r = processes::close_apps(&opts.close_list, false);
            report.apps_closed = r.closed.clone();
            self.state.closed_apps = paths
                .into_iter()
                .filter(|p| {
                    r.closed
                        .iter()
                        .any(|n| p.to_lowercase().ends_with(&n.to_lowercase()))
                })
                .collect();
        }
        self.persist();
        if profile.clean_ram {
            report.ram_freed_bytes = memory::trim_working_sets().freed_bytes;
        }
        tracing::info!(?report, game_id, "performans profili uygulandı");
        report
    }

    /// Çalışan oyunun yeni süreçlerine (başlatıcının açtığı asıl oyun) de öncelik verir.
    pub fn boost_new_pids(&mut self, pids: &[u32]) {
        let fresh: Vec<u32> = pids
            .iter()
            .copied()
            .filter(|p| !self.boosted_pids.contains(p))
            .collect();
        if !fresh.is_empty() {
            processes::boost_priority(&fresh);
            self.boosted_pids.extend(fresh);
        }
    }

    /// Oyun kapandı; etkin oyun kalmadıysa her şeyi geri alır.
    pub fn on_game_ended(&mut self, game_id: i64, opts: &BoostOptions) {
        if self.active_games.remove(&game_id) && self.active_games.is_empty() {
            self.revert(opts);
        }
    }

    fn revert(&mut self, opts: &BoostOptions) {
        if let Some(prev) = self.state.previous_power_plan.take() {
            if !power::set_active(&prev) {
                tracing::warn!("önceki güç planına dönülemedi");
            }
        }
        if self.state.notifications_were_on.take() == Some(true) {
            if let Some(t) = tweaks::find("notifications") {
                let _ = tweaks::apply_direct(t, true);
            }
        }
        let closed = std::mem::take(&mut self.state.closed_apps);
        if opts.restart_closed_apps {
            for exe in closed {
                if Path::new(&exe).is_file() {
                    let _ = std::process::Command::new(&exe).spawn();
                }
            }
        }
        self.boosted_pids.clear();
        self.persist();
        tracing::info!("performans profili geri alındı");
    }
}

/// Takipçinin bulduğu süreçlerden oyun → pid listesi.
pub type RunningGames = HashMap<i64, Vec<u32>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn profile_roundtrip_and_default() {
        let conn = db::open_in_memory().unwrap();
        conn.execute(
            "INSERT INTO games (platform, platform_id, title, added_at) VALUES ('steam', '1', 'X', 0)",
            [],
        )
        .unwrap();
        let id = conn.last_insert_rowid();
        assert_eq!(load_profile(&conn, id).unwrap(), PerfProfile::default());
        save_profile(&conn, id, &PerfProfile::recommended()).unwrap();
        assert_eq!(load_profile(&conn, id).unwrap(), PerfProfile::recommended());
    }

    #[test]
    fn disabled_profile_changes_nothing_and_writes_no_state() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("perf-state.json");
        let mut m = BoostManager::new(file.clone());
        let r = m.on_game_started(1, &PerfProfile::default(), &[], &BoostOptions::default());
        assert!(r.power_plan.is_none() && !r.notifications_muted);
        assert!(!m.is_active());
        assert!(!file.exists());
    }

    #[test]
    fn recover_reads_and_removes_stale_state_file() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("perf-state.json");
        // Boş durum (geri alınacak bir şey yok) → yine de dosya temizlenir.
        std::fs::write(&file, b"{}").unwrap();
        let mut m = BoostManager::new(file.clone());
        assert!(m.recover(&BoostOptions::default()));
        assert!(!file.exists());
        assert!(
            !m.recover(&BoostOptions::default()),
            "dosya yoksa yapılacak bir şey yok"
        );
    }

    #[cfg(windows)]
    #[test]
    #[ignore = "gerçek güç planını ve bildirim ayarını değiştirir, sonra geri alır"]
    fn real_boost_applies_and_reverts() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("perf-state.json");
        let before_plan = power::active_guid();
        let notif = tweaks::find("notifications").unwrap();
        let before_notif = tweaks::current(notif);
        let mut m = BoostManager::new(file.clone());
        let profile = PerfProfile {
            enabled: true,
            power_plan: true,
            mute_notifications: true,
            ..Default::default()
        };
        let r = m.on_game_started(1, &profile, &[], &BoostOptions::default());
        println!("uygulandı: {r:?}; durum dosyası: {}", file.exists());
        if r.power_plan.is_some() {
            assert_ne!(power::active_guid(), before_plan);
        }
        m.on_game_ended(1, &BoostOptions::default());
        assert_eq!(power::active_guid(), before_plan, "güç planı geri alınmalı");
        assert_eq!(
            tweaks::current(notif),
            before_notif,
            "bildirim ayarı geri alınmalı"
        );
        assert!(!file.exists());
    }
}

#[cfg(all(test, windows))]
mod real_system {
    /// Elle (uçtan uca test hazırlığı): gerçek veritabanında "Sahin Oyunu" test oyununa profil yazar.
    #[test]
    #[ignore = "gerçek veritabanına test profili yazar"]
    fn set_profile_for_test_game() {
        let db = std::path::PathBuf::from(std::env::var("LOCALAPPDATA").unwrap())
            .join("com.oyunarsivim.app/library.sqlite");
        let conn = crate::db::open(&db).unwrap();
        let id: i64 = conn
            .query_row(
                "SELECT id FROM games WHERE platform = 'manual' AND title = 'Sahin Oyunu'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let p = super::PerfProfile {
            enabled: true,
            power_plan: true,
            high_priority: true,
            ..Default::default()
        };
        super::save_profile(&conn, id, &p).unwrap();
        println!("profil yazıldı: oyun {id}");
    }
}
