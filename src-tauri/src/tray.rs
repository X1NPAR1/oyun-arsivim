//! Sistem tepsisi ve ana pencere yaşam döngüsü.
//!
//! Düşük RAM için pencere "gizlenmez", tamamen kapatılır: WebView2 süreçleri sonlanır ve yalnız
//! küçük Rust süreci tepside kalır. Tepsiden açılınca pencere yapılandırmadan yeniden oluşturulur.
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WebviewWindowBuilder};

use crate::commands::{self, SharedState};
use crate::window_style;

pub const MAIN: &str = "main";
/// Başlangıçta pencere açmadan tepside başlamak için komut satırı bayrağı.
pub const HIDDEN_ARG: &str = "--hidden";

/// "Çıkış" seçildiğinde true olur; aksi hâlde son pencerenin kapanması uygulamayı sonlandırmaz.
pub struct QuitFlag(pub AtomicBool);

pub fn started_hidden() -> bool {
    std::env::args().any(|a| a == HIDDEN_ARG)
}

/// Ana pencereyi gösterir; tepside kapatılmışsa yeniden oluşturur.
pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(MAIN) {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }
    let Some(cfg) = app
        .config()
        .app
        .windows
        .iter()
        .find(|w| w.label == MAIN)
        .cloned()
    else {
        tracing::error!("ana pencere yapılandırması yok");
        return;
    };
    match WebviewWindowBuilder::from_config(app, &cfg).and_then(|b| b.build()) {
        // Pencere `visible:false` başlar; arayüz ilk karesini boyayınca kendini gösterir.
        Ok(w) => {
            let backdrop = window_style::apply(&w);
            if let Some(state) = app.try_state::<crate::BackdropState>() {
                if let Ok(mut b) = state.0.lock() {
                    *b = backdrop;
                }
            }
        }
        Err(err) => tracing::error!(%err, "ana pencere oluşturulamadı"),
    }
}

pub fn quit(app: &AppHandle) {
    if let Some(flag) = app.try_state::<QuitFlag>() {
        flag.0.store(true, Ordering::SeqCst);
    }
    app.exit(0);
}

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Oyun Arşivim'i aç", true, None::<&str>)?;
    let scan = MenuItem::with_id(app, "scan", "Kütüphaneyi tara", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Çıkış", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&open, &scan, &separator, &quit_item])?;

    let mut builder = TrayIconBuilder::with_id("main-tray")
        .tooltip("Oyun Arşivim")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_main(app),
            "scan" => {
                let app = app.clone();
                std::thread::spawn(move || {
                    let state = app.state::<SharedState>().inner().clone();
                    if let Err(err) = commands::run_scan(&app, &state) {
                        tracing::warn!(%err, "tepsiden tarama başarısız");
                    }
                });
            }
            "quit" => quit(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }
    builder.build(app)?;
    Ok(())
}
