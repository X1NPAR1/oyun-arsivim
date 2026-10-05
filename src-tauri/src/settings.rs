//! Tipli uygulama ayarları. Veritabanında `settings` tablosunda anahtar başına JSON değer olarak
//! saklanır; bilinmeyen/bozuk değerler varsayılana düşer (eski sürüm verisi uygulamayı bozmaz).
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Windows açılışında arka planda (tepside) başlat.
    pub launch_at_startup: bool,
    /// Pencere kapatılınca uygulamadan çıkma, sistem tepsisinde çalışmaya devam et.
    pub close_to_tray: bool,
    /// Animasyon seviyesi: "system" | "full" | "reduced" | "off".
    pub motion: String,
    /// Kullanılan bellek eşiği aşınca otomatik RAM temizliği.
    pub auto_ram_clean: bool,
    /// Otomatik temizlik eşiği (kullanılan bellek yüzdesi).
    pub auto_ram_threshold: u8,
    /// Performans profilinde "uygulamaları kapat" açıkken kapatılacak exe adları.
    pub close_list: Vec<String>,
    /// Oyun bitince kapatılan uygulamaları yeniden başlat.
    pub restart_closed_apps: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            launch_at_startup: false,
            close_to_tray: true,
            motion: "system".to_owned(),
            auto_ram_clean: false,
            auto_ram_threshold: 85,
            close_list: Vec::new(),
            restart_closed_apps: true,
        }
    }
}

const KEY: &str = "app";

pub fn load(conn: &Connection) -> Settings {
    let raw: Option<String> = conn
        .query_row("SELECT value FROM settings WHERE key = ?1", [KEY], |r| {
            r.get(0)
        })
        .optional()
        .unwrap_or_else(|err| {
            tracing::warn!(%err, "ayarlar okunamadı, varsayılanlar kullanılıyor");
            None
        });
    let mut s: Settings = raw
        .and_then(|r| serde_json::from_str(&r).ok())
        .unwrap_or_default();
    if !matches!(s.motion.as_str(), "system" | "full" | "reduced" | "off") {
        s.motion = "system".to_owned();
    }
    s.auto_ram_threshold = s.auto_ram_threshold.clamp(50, 98);
    s
}

pub fn save(conn: &Connection, s: &Settings) -> rusqlite::Result<()> {
    let json = serde_json::to_string(s).unwrap_or_else(|_| "{}".to_owned());
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![KEY, json],
    )?;
    Ok(())
}

/// Kısmi güncelleme: arayüz yalnız değişen alanları gönderir.
pub fn merge(current: &Settings, patch: &serde_json::Value) -> Settings {
    let mut value = serde_json::to_value(current).unwrap_or_default();
    if let (Some(base), Some(p)) = (value.as_object_mut(), patch.as_object()) {
        for (k, v) in p {
            if base.contains_key(k) {
                base.insert(k.clone(), v.clone());
            }
        }
    }
    serde_json::from_value(value).unwrap_or_else(|_| current.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn defaults_then_persist_roundtrip() {
        let conn = db::open_in_memory().unwrap();
        assert_eq!(load(&conn), Settings::default());
        let s = Settings {
            launch_at_startup: true,
            close_to_tray: false,
            motion: "off".into(),
            close_list: vec!["Spotify.exe".into()],
            ..Settings::default()
        };
        save(&conn, &s).unwrap();
        assert_eq!(load(&conn), s);
    }

    #[test]
    fn corrupt_or_unknown_values_fall_back() {
        let conn = db::open_in_memory().unwrap();
        conn.execute("INSERT INTO settings VALUES ('app', '{bozuk')", [])
            .unwrap();
        assert_eq!(load(&conn), Settings::default());
        conn.execute(
            "UPDATE settings SET value = '{\"motion\":\"uçuk\",\"closeToTray\":false,\"eski\":1}'",
            [],
        )
        .unwrap();
        let s = load(&conn);
        assert_eq!(s.motion, "system");
        assert!(!s.close_to_tray);
    }

    #[test]
    fn merge_applies_only_known_keys_with_right_types() {
        let base = Settings::default();
        let m = merge(
            &base,
            &serde_json::json!({"launchAtStartup": true, "hacker": 1}),
        );
        assert!(m.launch_at_startup);
        // Yanlış tip: değişiklik reddedilir, mevcut ayar korunur.
        let m2 = merge(&base, &serde_json::json!({"closeToTray": "evet"}));
        assert_eq!(m2, base);
    }
}
