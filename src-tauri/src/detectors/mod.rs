//! Oyun platformu dedektörleri. Her platform `GameDetector` uygular; kök yollar enjekte edilebilir
//! olduğundan testler sahte klasörlerle çalışır ve kullanıcının gerçek oyunlarına dokunmaz.
pub mod epic;
pub mod steam;
pub mod steam_playtime;
pub mod vdf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Steam,
    Epic,
    Manual,
}

impl Platform {
    pub fn as_str(self) -> &'static str {
        match self {
            Platform::Steam => "steam",
            Platform::Epic => "epic",
            Platform::Manual => "manual",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "steam" => Some(Platform::Steam),
            "epic" => Some(Platform::Epic),
            "manual" => Some(Platform::Manual),
            _ => None,
        }
    }
}

/// Bir dedektörün bulduğu kurulu oyun.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedGame {
    pub platform: Platform,
    /// Platform içindeki kalıcı kimlik (Steam appid, Epic AppName).
    pub platform_id: String,
    pub title: String,
    pub install_dir: String,
    pub executable: Option<String>,
    pub launch_uri: String,
    pub size_bytes: Option<u64>,
    /// Platformun bildirdiği son oynama zamanı (Unix saniye).
    pub last_played: Option<i64>,
}

#[derive(Debug, thiserror::Error)]
pub enum DetectorError {
    #[error("dosya okunamadı: {0}")]
    Io(#[from] std::io::Error),
}

pub trait GameDetector: Send + Sync {
    fn platform(&self) -> Platform;
    /// Platform kurulu değilse boş liste döner (hata değil). Tekil bozuk girdiler atlanır ve loglanır.
    fn detect(&self) -> Result<Vec<DetectedGame>, DetectorError>;
}

pub fn system_detectors() -> Vec<Box<dyn GameDetector>> {
    vec![
        Box::new(steam::SteamDetector::system_default()),
        Box::new(epic::EpicDetector::system_default()),
    ]
}

#[cfg(test)]
mod real_system {
    use super::*;

    /// Gerçek sistem testi (elle): `cargo test real_system -- --ignored --nocapture`.
    #[test]
    #[ignore = "bu makinedeki gerçek kütüphaneyi okur"]
    fn scan_this_machine() {
        for d in system_detectors() {
            let games = d.detect().unwrap();
            println!("{:?}: {} oyun", d.platform(), games.len());
            for g in &games {
                println!("  - {} [{}] {}", g.title, g.platform_id, g.install_dir);
            }
        }
    }
}
