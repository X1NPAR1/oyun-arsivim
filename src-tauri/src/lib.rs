use serde::Serialize;

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

#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo::current()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![app_info])
        .run(tauri::generate_context!())
        .unwrap_or_else(|err| {
            // Burada henüz pencere yok; stderr'e yazıp süreçten çıkmak tek dürüst seçenek.
            eprintln!("Oyun Arşivim başlatılamadı: {err}");
            std::process::exit(1);
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
