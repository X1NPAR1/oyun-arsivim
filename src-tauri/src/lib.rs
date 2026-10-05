mod artwork;
mod commands;
mod db;
mod detectors;
mod elevate;
mod game_actions;
mod icon;
mod library;
mod logging;
mod perf;
mod playtime;
mod settings;
mod startup;
mod tracker;
mod tray;
mod window_style;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use serde::Serialize;
use tauri::{Emitter, Manager, RunEvent, WebviewWindowBuilder};
use window_style::Backdrop;

use commands::SharedState;

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

const SPLASH: &str = "splash";

/// Açılış bir kez başarıyla bitince true; tekrar dene düğmesi ikinci kez başlatmasın.
struct StartupFlag {
    running: AtomicBool,
    done: AtomicBool,
    /// Splash penceresi ilk karesini boyadı ve durumu okudu.
    splash_ready: AtomicBool,
}

/// Splash hazır olmasa bile ana pencere en geç bu süre sonra açılır (takılı kalmasın).
const SPLASH_WAIT_MAX: std::time::Duration = std::time::Duration::from_secs(4);

fn open_splash(app: &tauri::AppHandle) {
    if app.get_webview_window(SPLASH).is_some() {
        return;
    }
    let Some(cfg) = app
        .config()
        .app
        .windows
        .iter()
        .find(|w| w.label == SPLASH)
        .cloned()
    else {
        return;
    };
    match WebviewWindowBuilder::from_config(app, &cfg).and_then(|b| b.build()) {
        Ok(w) => {
            // Win10'da OS gölgesi yuvarlak köşelerin arkasında dikdörtgen çizer.
            let _ = w.set_shadow(false);
        }
        Err(err) => tracing::warn!(%err, "açılış penceresi açılamadı"),
    }
}

/// Açılış adımlarını arka planda çalıştırır. Başarıda tepsi, takipçi ve ana pencere başlar;
/// hata olursa açılış penceresine bildirilir (Tekrar dene / Günlüğü aç).
fn begin_startup(app: tauri::AppHandle, hidden: bool) {
    let flag = app.state::<StartupFlag>();
    if flag.done.load(Ordering::SeqCst) || flag.running.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(move || {
        let flag = app.state::<StartupFlag>();
        let data = match app.path().app_local_data_dir() {
            Ok(d) => d,
            Err(err) => {
                let _ = app.emit(
                    startup::EVENT_ERROR,
                    startup::ErrorEvent {
                        message: err.to_string(),
                    },
                );
                flag.running.store(false, Ordering::SeqCst);
                return;
            }
        };
        match startup::run(&app, &data) {
            Ok(state) => {
                app.manage(state.clone());
                flag.done.store(true, Ordering::SeqCst);
                if let Err(err) = tray::build(&app) {
                    tracing::warn!(%err, "sistem tepsisi oluşturulamadı");
                }
                // Oyun süresi takibi pencereden bağımsız çalışır (tepsideyken de).
                tracker::spawn(app.clone(), state.clone(), data.join("perf-state.json"));
                if !hidden {
                    // Splash görünmeden ana pencere açılırsa açılış ekranı yalnız bir an yanıp söner.
                    let started = std::time::Instant::now();
                    while !flag.splash_ready.load(Ordering::SeqCst)
                        && started.elapsed() < SPLASH_WAIT_MAX
                    {
                        std::thread::sleep(std::time::Duration::from_millis(40));
                    }
                    startup::set_status(&app, |s| s.done = true);
                    tray::show_main(&app);
                }
                // Sonraki açılışlarda eksik görseller arka planda tamamlanır.
                commands::fetch_artwork(&app, &state);
            }
            Err(message) => {
                tracing::error!(%message, "açılış başarısız");
                startup::set_status(&app, |s| s.error = Some(message.clone()));
                let _ = app.emit(startup::EVENT_ERROR, startup::ErrorEvent { message });
            }
        }
        flag.running.store(false, Ordering::SeqCst);
    });
}

/// Ana pencere ilk karesini boyadı: açılış penceresi solarak kapanabilir.
#[tauri::command]
fn main_ready(app: tauri::AppHandle) {
    let _ = app.emit(startup::EVENT_DONE, ());
}

/// Splash açıldığında son durumu okur (olaylardan önce gönderilenler kaçmasın) ve hazır olduğunu bildirir.
#[tauri::command]
fn splash_ready(app: tauri::AppHandle) -> startup::StartupStatus {
    app.state::<StartupFlag>()
        .splash_ready
        .store(true, Ordering::SeqCst);
    app.state::<startup::StatusState>()
        .0
        .lock()
        .map(|s| s.clone())
        .unwrap_or_default()
}

#[tauri::command]
fn startup_retry(app: tauri::AppHandle) {
    tracing::info!("açılış yeniden deneniyor");
    begin_startup(app, false);
}

fn close_to_tray_enabled(app: &tauri::AppHandle) -> bool {
    app.try_state::<SharedState>()
        .and_then(|s| s.db.lock().ok().map(|c| settings::load(&c).close_to_tray))
        .unwrap_or(true)
}

/// `--admin-task <id> <on|off>` ile başlatıldıysa görevi yapıp çıkış kodunu döndürür.
pub fn handle_admin_task() -> Option<i32> {
    let args: Vec<String> = std::env::args().collect();
    let i = args.iter().position(|a| a == "--admin-task")?;
    Some(perf::tweaks::run_admin_task(&args[i + 1..]))
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
            app.manage(perf::monitor::PerfMonitor::new());
            app.manage(BackdropState(Mutex::new(Backdrop::None)));
            app.manage(tray::QuitFlag(AtomicBool::new(false)));
            app.manage(StartupFlag {
                running: AtomicBool::new(false),
                done: AtomicBool::new(false),
                splash_ready: AtomicBool::new(false),
            });
            app.manage(startup::StatusState(Mutex::new(
                startup::StartupStatus::default(),
            )));
            let _ = data;
            // Windows açılışında `--hidden` ile başlatıldıysa açılış penceresi de ana pencere de açılmaz.
            let hidden = tray::started_hidden();
            if !hidden {
                open_splash(app.handle());
            }
            begin_startup(app.handle().clone(), hidden);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            window_backdrop,
            open_log_dir,
            main_ready,
            splash_ready,
            startup_retry,
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
            perf::commands::perf_snapshot,
            perf::commands::perf_tweaks,
            perf::commands::perf_set_tweak,
            perf::commands::perf_restore_defaults,
            perf::commands::perf_clean_ram,
            perf::commands::perf_purge_standby,
            perf::commands::power_overview,
            perf::commands::power_set,
            perf::commands::perf_profile_get,
            perf::commands::perf_profile_set,
            perf::commands::perf_profiled_games,
            perf::commands::perf_profile_recommended,
            perf::commands::perf_close_suggestions,
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
