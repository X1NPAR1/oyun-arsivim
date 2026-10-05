//! Açılış adımları ve gerçek ilerleme olayları (splash penceresi için).
//!
//! İlerleme SAHTE değildir: her adım bittiğinde ağırlığı kadar ilerler; görsel indirme adımında oyun
//! başına ilerler. "Az kaldı…" yalnız gerçek ilerleme %80'i geçince gösterilir.
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::artwork::{ArtworkStore, HttpFetcher};
use crate::commands::{now, AppState, SharedState, EVENT_LIBRARY_CHANGED};
use crate::detectors::{self, steam_playtime, Platform};
use crate::perf::boost::{BoostManager, BoostOptions};
use crate::{db, library, playtime, settings};

pub const EVENT_PROGRESS: &str = "startup://progress";
pub const EVENT_ERROR: &str = "startup://error";
pub const EVENT_DONE: &str = "startup://done";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub percent: f64,
    pub message: String,
}

/// Açılışın son durumu: splash penceresi olayları dinlemeye başlamadan önce gönderilenleri kaçırmasın.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupStatus {
    pub percent: f64,
    pub message: String,
    pub error: Option<String>,
    pub done: bool,
}

pub struct StatusState(pub Mutex<StartupStatus>);

pub fn set_status(app: &AppHandle, f: impl FnOnce(&mut StartupStatus)) {
    if let Some(s) = app.try_state::<StatusState>() {
        if let Ok(mut st) = s.0.lock() {
            f(&mut st);
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEvent {
    pub message: String,
}

/// Adım ağırlıkları (toplam 100). Görsel adımı yalnız ilk açılışta uzun sürer.
const W_DB: f64 = 10.0;
const W_SETTINGS: f64 = 5.0;
const W_STEAM: f64 = 25.0;
const W_EPIC: f64 = 15.0;
const W_PLAYTIME: f64 = 10.0;
const W_ARTWORK: f64 = 30.0;
const W_PERF: f64 = 5.0;

/// Yüzdeye göre gösterilecek genel durum metni; adım metni yoksa veya sona yaklaşıldıysa kullanılır.
pub fn phase_message(percent: f64, step: &str) -> String {
    if percent >= 100.0 {
        "Hazır!".to_owned()
    } else if percent >= 95.0 {
        "Bitmek üzere...".to_owned()
    } else if percent >= 80.0 {
        "Az kaldı...".to_owned()
    } else {
        step.to_owned()
    }
}

struct Reporter<'a> {
    app: &'a AppHandle,
    done: f64,
}

impl Reporter<'_> {
    fn emit(&self, step: &str) {
        let percent = self.done.clamp(0.0, 100.0);
        let message = phase_message(percent, step);
        set_status(self.app, |s| {
            s.percent = percent;
            s.message.clone_from(&message);
            s.error = None;
        });
        let _ = self
            .app
            .emit(EVENT_PROGRESS, ProgressEvent { percent, message });
    }

    /// Adım başlarken metni göster, bitince ağırlığı ekle.
    fn step<T>(&mut self, message: &str, weight: f64, f: impl FnOnce() -> T) -> T {
        self.emit(message);
        let out = f();
        self.done += weight;
        self.emit(message);
        out
    }
}

/// Bütün açılış adımlarını çalıştırır; başarıda paylaşılan durumu döndürür.
pub fn run(app: &AppHandle, data_dir: &Path) -> Result<SharedState, String> {
    let mut r = Reporter { app, done: 0.0 };
    // Yalnız debug derlemesinde: hata ekranını uçtan uca test etmek için.
    #[cfg(debug_assertions)]
    if std::env::var_os("OYUN_ARSIVIM_TEST_STARTUP_ERROR").is_some() {
        r.step("Veritabanı hazırlanıyor...", W_DB, || {
            std::thread::sleep(std::time::Duration::from_millis(600))
        });
        return Err("Test hatası: veritabanı dosyası kilitli (yalnız debug)".to_owned());
    }

    let conn = r
        .step("Veritabanı hazırlanıyor...", W_DB, || {
            db::open(&data_dir.join("library.sqlite"))
        })
        .map_err(|e| format!("Veritabanı açılamadı: {e}"))?;
    let first_launch = library::list(&conn).map(|g| g.is_empty()).unwrap_or(true);
    let prefs = r.step("Ayarlar yükleniyor...", W_SETTINGS, || {
        settings::load(&conn)
    });

    let mut detected = Vec::new();
    let mut scanned: Vec<Platform> = Vec::new();
    for detector in detectors::system_detectors() {
        let (message, weight) = match detector.platform() {
            Platform::Steam => ("Steam kütüphanesi taranıyor...", W_STEAM),
            Platform::Epic => ("Epic Games oyunları bulunuyor...", W_EPIC),
            Platform::Manual => ("Oyunlar taranıyor...", 0.0),
        };
        match r.step(message, weight, || detector.detect()) {
            Ok(mut games) => {
                scanned.push(detector.platform());
                detected.append(&mut games);
            }
            Err(err) => {
                tracing::warn!(platform = ?detector.platform(), %err, "açılış taraması başarısız")
            }
        }
    }

    let mut conn = conn;
    r.step("Oyun süreleri alınıyor...", W_PLAYTIME, || {
        if let Err(err) = library::sync(&mut conn, &detected, &scanned, now()) {
            tracing::warn!(%err, "kütüphane senkronlanamadı");
        }
        if let Some(root) = detectors::steam::steam_root_from_registry() {
            let _ = playtime::apply_steam_import(&conn, &steam_playtime::read_all(&root));
        }
    });

    let artwork = ArtworkStore::new(data_dir.join("artwork"));
    artwork.cleanup_legacy();
    // İlk açılışta görseller şimdi indirilir (ayrıntılı ilerleme); sonraki açılışlarda arka planda.
    let games = library::list(&conn).unwrap_or_default();
    if first_launch && !games.is_empty() {
        let fetcher = HttpFetcher::new();
        let per_game = W_ARTWORK / f64::from(u32::try_from(games.len()).unwrap_or(1));
        for g in &games {
            r.emit("Kapak görselleri hazırlanıyor...");
            artwork.ensure(g, &fetcher);
            r.done += per_game;
        }
    } else {
        r.step("Kapak görselleri hazırlanıyor...", W_ARTWORK, || ());
    }

    r.step("Performans ayarları kontrol ediliyor...", W_PERF, || {
        // Önceki oturum çöktüyse oyun profilinin değişikliklerini geri al.
        let opts = BoostOptions {
            close_list: prefs.close_list.clone(),
            restart_closed_apps: prefs.restart_closed_apps,
        };
        BoostManager::new(data_dir.join("perf-state.json")).recover(&opts);
    });

    r.done = 100.0;
    r.emit("Hazır!");
    let _ = app.emit(EVENT_LIBRARY_CHANGED, ());
    Ok(Arc::new(AppState {
        db: Mutex::new(conn),
        artwork,
        scanning: AtomicBool::new(false),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weights_sum_to_hundred() {
        let total = W_DB + W_SETTINGS + W_STEAM + W_EPIC + W_PLAYTIME + W_ARTWORK + W_PERF;
        assert!((total - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn phase_messages_follow_real_progress() {
        assert_eq!(
            phase_message(10.0, "Veritabanı hazırlanıyor..."),
            "Veritabanı hazırlanıyor..."
        );
        assert_eq!(
            phase_message(79.9, "Kapak görselleri hazırlanıyor..."),
            "Kapak görselleri hazırlanıyor..."
        );
        assert_eq!(phase_message(80.0, "x"), "Az kaldı...");
        assert_eq!(phase_message(96.0, "x"), "Bitmek üzere...");
        assert_eq!(phase_message(100.0, "x"), "Hazır!");
    }
}
