//! __NAME__ dedektörü. Tespit yöntemi ve sınırlamalar: docs/PLATFORMS.md#__ID__
use std::path::{Path, PathBuf};

use super::{DetectedGame, DetectorError, GameDetector, Platform};

pub struct __NAME__Detector {
    /// Testlerde sahte klasör enjekte edilir; üretimde `system_default()` kullanılır.
    root: Option<PathBuf>,
}

impl __NAME__Detector {
    pub fn new(root: Option<PathBuf>) -> Self {
        Self { root }
    }

    pub fn system_default() -> Self {
        // TODO: kayıt defteri / bilinen konumdan kök yolu bul.
        Self::new(None)
    }

    fn parse_entry(&self, _path: &Path) -> Result<Option<DetectedGame>, DetectorError> {
        // TODO: manifest/kayıt girdisini ayrıştır; araç/redistributable ise Ok(None).
        Ok(None)
    }
}

impl GameDetector for __NAME__Detector {
    fn platform(&self) -> Platform {
        Platform::__NAME__
    }

    fn detect(&self) -> Result<Vec<DetectedGame>, DetectorError> {
        let Some(root) = &self.root else {
            return Ok(Vec::new()); // platform kurulu değil: hata değil
        };
        let mut games = Vec::new();
        for entry in std::fs::read_dir(root)? {
            let path = entry?.path();
            match self.parse_entry(&path) {
                Ok(Some(game)) => games.push(game),
                Ok(None) => {}
                Err(err) => tracing::warn!(?path, %err, "__ID__ girdisi atlandı"),
            }
        }
        Ok(games)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/__ID__")
    }

    #[test]
    fn missing_root_yields_empty_list() {
        assert!(__NAME__Detector::new(None).detect().unwrap().is_empty());
    }

    #[test]
    fn finds_installed_games() {
        let games = __NAME__Detector::new(Some(fixture())).detect().unwrap();
        assert!(!games.is_empty());
    }
}
