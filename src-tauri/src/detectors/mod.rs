//! Oyun platformu dedektörleri. Her platform `GameDetector` uygular; kök yollar enjekte edilebilir
//! olduğundan testler sahte klasörlerle çalışır ve kullanıcının gerçek oyunlarına dokunmaz.
pub mod epic;
pub mod registry;
pub mod regplatforms;
pub mod riot;
pub mod steam;
pub mod steam_playtime;
pub mod vdf;
pub mod xbox;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Steam,
    Epic,
    Gog,
    Ea,
    Ubisoft,
    Battlenet,
    Rockstar,
    Riot,
    Xbox,
    Manual,
}

impl Platform {
    pub fn as_str(self) -> &'static str {
        match self {
            Platform::Steam => "steam",
            Platform::Epic => "epic",
            Platform::Gog => "gog",
            Platform::Ea => "ea",
            Platform::Ubisoft => "ubisoft",
            Platform::Battlenet => "battlenet",
            Platform::Rockstar => "rockstar",
            Platform::Riot => "riot",
            Platform::Xbox => "xbox",
            Platform::Manual => "manual",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "steam" => Some(Platform::Steam),
            "epic" => Some(Platform::Epic),
            "gog" => Some(Platform::Gog),
            "ea" => Some(Platform::Ea),
            "ubisoft" => Some(Platform::Ubisoft),
            "battlenet" => Some(Platform::Battlenet),
            "rockstar" => Some(Platform::Rockstar),
            "riot" => Some(Platform::Riot),
            "xbox" => Some(Platform::Xbox),
            "manual" => Some(Platform::Manual),
            _ => None,
        }
    }

    /// Aynı oyun birden fazla kaynaktan bulunursa hangisinin tutulacağı (küçük = öncelikli).
    /// Mağazanın kendi istemcisi (Steam/Epic) süre ve görsel verisi sağladığı için önce gelir.
    pub fn dedupe_rank(self) -> u8 {
        match self {
            Platform::Steam => 0,
            Platform::Epic => 1,
            Platform::Gog => 2,
            Platform::Ubisoft => 3,
            Platform::Battlenet => 4,
            Platform::Riot => 5,
            Platform::Rockstar => 6,
            Platform::Ea => 7,
            Platform::Xbox => 8,
            Platform::Manual => 9,
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
    /// `executable` ile birlikte verilecek komut satırı argümanları (ör. Riot istemcisi).
    pub launch_args: Option<String>,
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
        Box::new(regplatforms::GogDetector::new(registry::system())),
        Box::new(regplatforms::EaDetector::new(registry::system())),
        Box::new(regplatforms::UbisoftDetector::new(registry::system())),
        Box::new(regplatforms::BattlenetDetector::new(registry::system())),
        Box::new(regplatforms::RockstarDetector::new(registry::system())),
        Box::new(riot::RiotDetector::system_default()),
        Box::new(xbox::XboxDetector),
    ]
}

/// Aynı oyun birden fazla platformdan bulunursa (aynı kurulum klasörü) tek kayıt tutulur:
/// öncelik `Platform::dedupe_rank`. Ör. EA app kaydı Steam klasörünü gösteren Battlefield 1.
///
/// Not: bir platformun tamamı tekilleştirmede elenirse o platform yine "tarandı" sayılır; böylece
/// önceki sürümlerde başka platformdan kaydedilmiş kopya doğru şekilde "kaldırıldı" olur.
pub fn dedupe(mut games: Vec<DetectedGame>) -> Vec<DetectedGame> {
    let norm = |p: &str| p.replace('/', "\\").trim_end_matches('\\').to_lowercase();
    games.sort_by_key(|g| g.platform.dedupe_rank());
    let mut seen = std::collections::HashSet::new();
    games.retain(|g| seen.insert(norm(&g.install_dir)));
    games
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

#[cfg(test)]
mod dedupe_tests {
    use super::*;

    fn g(platform: Platform, id: &str, dir: &str) -> DetectedGame {
        DetectedGame {
            platform,
            platform_id: id.into(),
            title: id.into(),
            install_dir: dir.into(),
            executable: None,
            launch_uri: String::new(),
            launch_args: None,
            size_bytes: None,
            last_played: None,
        }
    }

    #[test]
    fn same_install_dir_keeps_higher_priority_platform() {
        // Gerçek sistem vakası: EA app'in Battlefield 1 kaydı Steam klasörünü gösteriyor.
        let out = dedupe(vec![
            g(
                Platform::Ea,
                "bf1-ea",
                r"D:\SteamLibrary\steamapps\common\Battlefield 1\",
            ),
            g(
                Platform::Steam,
                "1238840",
                r"d:/steamlibrary/steamapps/common/battlefield 1",
            ),
            g(
                Platform::Riot,
                "valorant.live",
                r"F:\Riot Games\VALORANT\live",
            ),
        ]);
        assert_eq!(out.len(), 2);
        assert!(out.iter().any(|x| x.platform_id == "1238840"));
        assert!(!out.iter().any(|x| x.platform_id == "bf1-ea"));
    }
}
