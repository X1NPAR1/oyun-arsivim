mod artwork;
mod commands;
mod db;
mod detectors;
mod game_actions;
mod icon;
mod library;
mod logging;
mod playtime;
mod settings;
mod tracker;
mod tray;
mod window_style;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{Manager, RunEvent};
use window_style::Backdrop;

use commands::{AppState, SharedState};

/// Arayüzün "Hakkında" ve tanılama ekranları için temel uygulama kimliği.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
}

impl AppInfo {
    pub fn current() -> Self {
        Self {
            name: "Oyun Arşivim".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
        }
    }
}

/// Günlük bekçisi uygulama boyunca yaşamalı (düşerse son kayıtlar kaybolur).
struct LogGuard(#[allow(dead_code)] Option<tracing_appender::non_blocking::WorkerGuard>);

/// Günlük klasörü ("Günlük klasörünü aç" için).
pub(crate) struct LogDir(pub(crate) std::path::PathBuf);

#[tauri::command]
fn open_log_dir(dir: tauri::State<'_, LogDir>) -> Result<(), String> {
    tauri_plugin_opener::open_path(&dir.0, None::<&str>).map_err(|e| e.to_string())
}

/// Etkin pencere arka planı; pencere her (yeniden) oluşturulduğunda güncellenir.
pub(crate) struct BackdropState(pub(crate) Mutex<Backdrop>);

#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo::current()
}

#[tauri::command]
fn window_backdrop(state: tauri::State<'_, BackdropState>) -> Backdrop {
    state.0.lock().map(|b| *b).unwrap_or(Backdrop::None)
}

fn init_state(app: &tauri::App) -> Result<SharedState, Box<dyn std::error::Error>> {
    let data = app.path().app_local_data_dir()?;
    let conn = db::open(&data.join("library.sqlite"))?;
    Ok(Arc::new(AppState {
        db: Mutex::new(conn),
        artwork: artwork::ArtworkStore::new(data.join("artwork")),
        scanning: AtomicBool::new(false),
    }))
}

fn close_to_tray_enabled(app: &tauri::AppHandle) -> bool {
    app.try_state::<SharedState>()
        .and_then(|s| s.db.lock().ok().map(|c| settings::load(&c).close_to_tray))
        .unwrap_or(true)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        // Tek örnek: uygulama zaten (tepside) çalışıyorsa ikinci başlatma pencereyi öne getirir.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![tray::HIDDEN_ARG]),
        ))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data = app.path().app_local_data_dir()?;
            let logs = logging::log_dir(&data);
            app.manage(LogGuard(logging::init(&logs)));
            app.manage(LogDir(logs));
            app.manage(BackdropState(Mutex::new(Backdrop::None)));
            app.manage(tray::QuitFlag(AtomicBool::new(false)));
            let state = init_state(app)?;
            state.artwork.cleanup_legacy();
            app.manage(state.clone());
            tray::build(app.handle())?;
            // Oyun süresi takibi pencereden bağımsız çalışır (tepsideyken de).
            tracker::spawn(app.handle().clone(), state.clone());

            // Windows açılışında `--hidden` ile başlatıldıysa pencere açılmaz, yalnız tepside çalışır.
            if !tray::started_hidden() {
                tray::show_main(app.handle());
            }

            // İlk tarama arka planda: arayüz önce veritabanındaki listeyle hemen açılır.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                if let Err(err) = commands::run_scan(&handle, &state) {
                    tracing::warn!(%err, "açılış taraması başarısız");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            window_backdrop,
            open_log_dir,
            commands::list_games,
            commands::scan_library,
            commands::add_manual_game,
            commands::remove_manual_game,
            commands::rename_game,
            commands::launch_game,
            commands::get_settings,
            commands::update_settings,
            commands::uninstall_game,
            commands::install_game,
            commands::create_desktop_shortcut,
            commands::open_game_folder,
            commands::playtime_hourly,
            commands::playtime_sessions,
        ])
        .build(tauri::generate_context!());

    let app = match app {
        Ok(app) => app,
        Err(err) => {
            // Burada henüz pencere yok; stderr'e yazıp süreçten çıkmak tek dürüst seçenek.
            eprintln!("Oyun Arşivim başlatılamadı: {err}");
            std::process::exit(1);
        }
    };

    app.run(|handle, event| {
        // Son pencere kapanınca (code: None) tepside kal; "Çıkış" ise app.exit(0) ile gelir.
        if let RunEvent::ExitRequested {
            code: None, api, ..
        } = event
        {
            let quitting = handle
                .try_state::<tray::QuitFlag>()
                .is_some_and(|f| f.0.load(Ordering::SeqCst));
            if !quitting && close_to_tray_enabled(handle) {
                api.prevent_exit();
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_info_uses_crate_version() {
        let info = AppInfo::current();
        assert_eq!(info.name, "Oyun Arşivim");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn app_info_serializes_for_ipc() {
        let json = serde_json::to_value(AppInfo::current()).unwrap();
        assert!(json.get("name").is_some());
        assert!(json.get("version").is_some());
    }
}
