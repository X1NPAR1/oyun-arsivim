//! Arayüze açılan IPC komutları. Hatalar kullanıcıya gösterilecek Türkçe metinlere dönüştürülür.
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::artwork::{Artwork, ArtworkStore, HttpFetcher};
use crate::detectors::{self, Platform};
use crate::game_actions::{self, PlatformAction};
use crate::library::{self, Game, SyncReport};
use crate::playtime::{self, HourRow, PlaytimeSummary, SessionRow};
use crate::settings::{self, Settings};

pub const EVENT_LIBRARY_CHANGED: &str = "library://changed";

pub struct AppState {
    pub db: Mutex<Connection>,
    pub artwork: ArtworkStore,
    pub scanning: AtomicBool,
}

pub type SharedState = Arc<AppState>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameView {
    #[serde(flatten)]
    pub game: Game,
    pub art: Artwork,
    pub playtime: PlaytimeSummary,
}

/// Komut hataları arayüze düz metin olarak gider (toast'ta gösterilir).
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("{0}")]
    Library(#[from] library::LibraryError),
    #[error("Veritabanı meşgul, lütfen tekrar deneyin")]
    Lock,
    #[error("Oyun başlatılamadı: {0}")]
    Launch(String),
    #[error("{0}")]
    Action(String),
}

impl Serialize for CommandError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

type CmdResult<T> = Result<T, CommandError>;

/// Komut satırı argümanlarını böler; çift tırnak içindeki boşluklar korunur.
pub fn split_args(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    for c in s.chars() {
        match c {
            '"' => quoted = !quoted,
            c if c.is_whitespace() && !quoted => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

fn views(state: &AppState) -> CmdResult<Vec<GameView>> {
    let conn = state.db.lock().map_err(|_| CommandError::Lock)?;
    let games = library::list(&conn)?;
    let mut times = playtime::summary_all(&conn, now()).map_err(library::LibraryError::from)?;
    drop(conn);
    Ok(games
        .into_iter()
        .map(|game| GameView {
            art: state.artwork.cached(&game),
            playtime: times.remove(&game.id).unwrap_or_default(),
            game,
        })
        .collect())
}

/// Tüm dedektörleri çalıştırır, veritabanını senkronlar, ardından eksik görselleri indirir.
/// Görseller indirildikçe arayüze `library://changed` gönderilir.
pub fn run_scan(app: &AppHandle, state: &SharedState) -> CmdResult<SyncReport> {
    if state.scanning.swap(true, Ordering::SeqCst) {
        return Ok(SyncReport::default()); // zaten taranıyor
    }
    let result: CmdResult<SyncReport> = (|| {
        let mut detected = Vec::new();
        let mut scanned: Vec<Platform> = Vec::new();
        for detector in detectors::system_detectors() {
            match detector.detect() {
                Ok(mut games) => {
                    scanned.push(detector.platform());
                    detected.append(&mut games);
                }
                Err(err) => {
                    tracing::warn!(platform = ?detector.platform(), %err, "tarama başarısız")
                }
            }
        }
        let report = {
            let mut conn = state.db.lock().map_err(|_| CommandError::Lock)?;
            let detected = detectors::dedupe(detected);
            let report = library::sync(&mut conn, &detected, &scanned, now())?;
            // Steam oynama sürelerini (yerel localconfig.vdf) her taramada tazele.
            if let Some(root) = detectors::steam::steam_root_from_registry() {
                let data = detectors::steam_playtime::read_all(&root);
                playtime::apply_steam_import(&conn, &data).map_err(library::LibraryError::from)?;
            }
            report
        };
        tracing::info!(?report, "kütüphane senkronlandı");
        let _ = app.emit(EVENT_LIBRARY_CHANGED, ());
        Ok(report)
    })();
    state.scanning.store(false, Ordering::SeqCst);
    let report = result?;
    fetch_artwork(app, state);
    Ok(report)
}

pub fn fetch_artwork(app: &AppHandle, state: &SharedState) {
    let games = match state.db.lock() {
        Ok(conn) => library::list(&conn).unwrap_or_default(),
        Err(_) => return,
    };
    let fetcher = HttpFetcher::new();
    let mut pending = 0usize;
    for game in &games {
        if state.artwork.ensure(game, &fetcher) {
            pending += 1;
            // Arayüzü her görselde değil, birkaç oyunda bir yenile.
            if pending >= 4 {
                let _ = app.emit(EVENT_LIBRARY_CHANGED, ());
                pending = 0;
            }
        }
    }
    if pending > 0 {
        let _ = app.emit(EVENT_LIBRARY_CHANGED, ());
    }
}

#[tauri::command]
pub fn list_games(state: State<'_, SharedState>) -> CmdResult<Vec<GameView>> {
    views(&state)
}

#[tauri::command]
pub async fn scan_library(app: AppHandle, state: State<'_, SharedState>) -> CmdResult<SyncReport> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || run_scan(&app, &state))
        .await
        .map_err(|e| CommandError::Launch(e.to_string()))?
}

#[tauri::command]
pub async fn add_manual_game(
    app: AppHandle,
    state: State<'_, SharedState>,
    exe_path: String,
    title: Option<String>,
) -> CmdResult<GameView> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let game = {
            let conn = state.db.lock().map_err(|_| CommandError::Lock)?;
            library::add_manual(&conn, &PathBuf::from(&exe_path), title.as_deref(), now())?
        };
        state.artwork.ensure(&game, &HttpFetcher::new());
        let _ = app.emit(EVENT_LIBRARY_CHANGED, ());
        Ok(GameView {
            art: state.artwork.cached(&game),
            playtime: PlaytimeSummary::default(),
            game,
        })
    })
    .await
    .map_err(|e| CommandError::Launch(e.to_string()))?
}

#[tauri::command]
pub fn remove_manual_game(app: AppHandle, state: State<'_, SharedState>, id: i64) -> CmdResult<()> {
    let conn = state.db.lock().map_err(|_| CommandError::Lock)?;
    let game = library::get(&conn, id)?;
    library::remove_manual(&conn, id)?;
    drop(conn);
    state.artwork.remove_for(&game);
    let _ = app.emit(EVENT_LIBRARY_CHANGED, ());
    Ok(())
}

#[tauri::command]
pub fn rename_game(
    app: AppHandle,
    state: State<'_, SharedState>,
    id: i64,
    title: String,
) -> CmdResult<()> {
    let conn = state.db.lock().map_err(|_| CommandError::Lock)?;
    library::rename(&conn, id, &title)?;
    let _ = app.emit(EVENT_LIBRARY_CHANGED, ());
    Ok(())
}

#[tauri::command]
pub fn launch_game(state: State<'_, SharedState>, id: i64) -> CmdResult<()> {
    let game = {
        let conn = state.db.lock().map_err(|_| CommandError::Lock)?;
        library::get(&conn, id)?
    };
    if !game.installed {
        return Err(CommandError::Launch(
            "oyun bu bilgisayarda kurulu değil".into(),
        ));
    }
    // Platform adresi varsa (steam://, uplay://, shell:AppsFolder…) o açılır; yoksa exe çalıştırılır.
    if !game.launch_uri.is_empty() {
        tauri_plugin_opener::open_url(&game.launch_uri, None::<&str>)
            .map_err(|e| CommandError::Launch(e.to_string()))?;
        return Ok(());
    }
    let exe = game.executable.ok_or_else(|| {
        CommandError::Launch(
            "bu oyunun başlatma bilgisi yok; platformun kendi uygulamasından başlat".into(),
        )
    })?;
    let mut cmd = std::process::Command::new(&exe);
    if let Some(args) = game.launch_args.as_deref() {
        cmd.args(split_args(args));
    }
    if !game.install_dir.is_empty() && std::path::Path::new(&game.install_dir).is_dir() {
        cmd.current_dir(&game.install_dir);
    }
    cmd.spawn()
        .map_err(|e| CommandError::Launch(e.to_string()))?;
    Ok(())
}

// ───────────── Ayarlar ─────────────

#[tauri::command]
pub fn get_settings(state: State<'_, SharedState>) -> CmdResult<Settings> {
    let conn = state.db.lock().map_err(|_| CommandError::Lock)?;
    Ok(settings::load(&conn))
}

/// Kısmi ayar güncellemesi; yan etkiler (Windows açılışına kayıt) burada uygulanır.
#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, SharedState>,
    patch: serde_json::Value,
) -> CmdResult<Settings> {
    let conn = state.db.lock().map_err(|_| CommandError::Lock)?;
    let current = settings::load(&conn);
    let mut next = settings::merge(&current, &patch);
    if next.launch_at_startup != current.launch_at_startup {
        use tauri_plugin_autostart::ManagerExt;
        let manager = app.autolaunch();
        let applied = if next.launch_at_startup {
            manager.enable()
        } else {
            manager.disable()
        };
        if let Err(err) = applied {
            // Kayıt başarısızsa ayarı da değiştirme: arayüz gerçek durumu göstersin.
            next.launch_at_startup = current.launch_at_startup;
            settings::save(&conn, &next).map_err(library::LibraryError::from)?;
            return Err(CommandError::Action(format!(
                "Windows açılışına eklenemedi: {err}"
            )));
        }
    }
    settings::save(&conn, &next).map_err(library::LibraryError::from)?;
    Ok(next)
}

// ───────────── Oyun eylemleri ─────────────

fn game_by_id(state: &AppState, id: i64) -> CmdResult<Game> {
    let conn = state.db.lock().map_err(|_| CommandError::Lock)?;
    Ok(library::get(&conn, id)?)
}

fn perform(action: PlatformAction) -> CmdResult<PlatformAction> {
    let uri = match &action {
        PlatformAction::OpenUri(u) => u,
        PlatformAction::Guided { uri, .. } => uri,
    };
    tauri_plugin_opener::open_url(uri, None::<&str>)
        .map_err(|e| CommandError::Action(format!("Platform açılamadı: {e}")))?;
    Ok(action)
}

/// Oyunu platformun kendi kaldırma akışıyla kaldırır (dosyalar uygulama tarafından silinmez).
#[tauri::command]
pub fn uninstall_game(state: State<'_, SharedState>, id: i64) -> CmdResult<PlatformAction> {
    let game = game_by_id(&state, id)?;
    perform(game_actions::uninstall_action(&game))
}

#[tauri::command]
pub fn install_game(state: State<'_, SharedState>, id: i64) -> CmdResult<PlatformAction> {
    let game = game_by_id(&state, id)?;
    let action = game_actions::install_action(&game).ok_or_else(|| {
        CommandError::Action(
            "Elle eklenen oyunlar buradan yüklenemez; oyunu yeniden kurup kütüphaneye ekle.".into(),
        )
    })?;
    perform(action)
}

#[tauri::command]
pub fn create_desktop_shortcut(state: State<'_, SharedState>, id: i64) -> CmdResult<String> {
    let game = game_by_id(&state, id)?;
    let art = state.artwork.cached(&game);
    // Kısayol ikonu: exe ikonu → kapak → hero (kare kırpılmış .ico).
    let icon_target = state.artwork.shortcut_icon_path(&game);
    let icon = [art.icon, art.cover, art.hero]
        .into_iter()
        .flatten()
        .find_map(|src| game_actions::build_icon(std::path::Path::new(&src), &icon_target));
    let desktop = game_actions::desktop_dir().map_err(|e| CommandError::Action(e.to_string()))?;
    let path = game_actions::create_shortcut_in(&desktop, &game, icon.as_deref())
        .map_err(|e| CommandError::Action(e.to_string()))?;
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn open_game_folder(state: State<'_, SharedState>, id: i64) -> CmdResult<()> {
    let game = game_by_id(&state, id)?;
    let dir = std::path::Path::new(&game.install_dir);
    if game.install_dir.is_empty() || !dir.is_dir() {
        return Err(CommandError::Action("Oyun klasörü bulunamadı".into()));
    }
    tauri_plugin_opener::open_path(&game.install_dir, None::<&str>)
        .map_err(|e| CommandError::Action(e.to_string()))
}

// ───────────── İstatistik ─────────────

/// Saatlik (UTC saat numarası) aktif/AFK süreleri; yerel gün/saat dağılımı arayüzde hesaplanır.
#[tauri::command]
pub fn playtime_hourly(
    state: State<'_, SharedState>,
    from_hour: i64,
    to_hour: i64,
    game_id: Option<i64>,
) -> CmdResult<Vec<HourRow>> {
    let conn = state.db.lock().map_err(|_| CommandError::Lock)?;
    Ok(playtime::hourly_rows(&conn, from_hour, to_hour, game_id)
        .map_err(library::LibraryError::from)?)
}

#[tauri::command]
pub fn playtime_sessions(
    state: State<'_, SharedState>,
    from: i64,
    to: i64,
    game_id: Option<i64>,
) -> CmdResult<Vec<SessionRow>> {
    let conn = state.db.lock().map_err(|_| CommandError::Lock)?;
    Ok(
        playtime::sessions_between(&conn, from, to, game_id)
            .map_err(library::LibraryError::from)?,
    )
}
