//! Epic Games dedektörü. Yöntem: docs/PLATFORMS.md#epic-games
//!
//! `%ProgramData%\Epic\EpicGamesLauncher\Data\Manifests\*.item` (JSON). DLC'ler, yarım kurulumlar ve
//! oyun kategorisinde olmayan uygulamalar (ör. Unreal Engine) elenir.
use std::path::PathBuf;

use serde::Deserialize;

use super::{DetectedGame, DetectorError, GameDetector, Platform};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct EpicManifest {
    display_name: Option<String>,
    app_name: Option<String>,
    #[serde(default)]
    main_game_app_name: Option<String>,
    install_location: Option<String>,
    launch_executable: Option<String>,
    catalog_namespace: Option<String>,
    catalog_item_id: Option<String>,
    install_size: Option<u64>,
    #[serde(default, rename = "bIsIncompleteInstall")]
    incomplete: bool,
    #[serde(default)]
    app_categories: Vec<String>,
}

pub struct EpicDetector {
    manifests_dir: Option<PathBuf>,
}

impl EpicDetector {
    pub fn new(manifests_dir: Option<PathBuf>) -> Self {
        Self { manifests_dir }
    }

    pub fn manifests_dir(&self) -> Option<&std::path::Path> {
        self.manifests_dir.as_deref()
    }

    pub fn system_default() -> Self {
        let dir = std::env::var_os("ProgramData")
            .map(PathBuf::from)
            .map(|p| p.join(r"Epic\EpicGamesLauncher\Data\Manifests"))
            .filter(|p| p.is_dir());
        Self::new(dir)
    }
}

pub fn parse_manifest(text: &str) -> Option<DetectedGame> {
    let m: EpicManifest = serde_json::from_str(text).ok()?;
    let app_name = m.app_name.filter(|s| !s.is_empty())?;
    let title = m.display_name.filter(|s| !s.trim().is_empty())?;
    let install_dir = m.install_location.filter(|s| !s.is_empty())?;
    if m.incomplete {
        return None;
    }
    // DLC: ana oyun adı dolu ve kendisinden farklı.
    if m.main_game_app_name
        .as_deref()
        .is_some_and(|main| !main.is_empty() && main != app_name)
    {
        return None;
    }
    if !m.app_categories.is_empty() && !m.app_categories.iter().any(|c| c == "games") {
        return None;
    }
    let launch_uri = match (&m.catalog_namespace, &m.catalog_item_id) {
        (Some(ns), Some(item)) => format!(
            "com.epicgames.launcher://apps/{ns}%3A{item}%3A{app_name}?action=launch&silent=true"
        ),
        _ => format!("com.epicgames.launcher://apps/{app_name}?action=launch&silent=true"),
    };
    let executable = m.launch_executable.filter(|e| !e.is_empty()).map(|e| {
        PathBuf::from(&install_dir)
            .join(e)
            .to_string_lossy()
            .into_owned()
    });
    Some(DetectedGame {
        launch_args: None,
        platform: Platform::Epic,
        platform_id: app_name,
        title: title.trim().to_owned(),
        install_dir,
        executable,
        launch_uri,
        size_bytes: m.install_size,
        last_played: None,
    })
}

impl GameDetector for EpicDetector {
    fn platform(&self) -> Platform {
        Platform::Epic
    }

    fn detect(&self) -> Result<Vec<DetectedGame>, DetectorError> {
        let Some(dir) = &self.manifests_dir else {
            return Ok(Vec::new());
        };
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Ok(Vec::new());
        };
        let mut games = Vec::new();
        for path in entries.flatten().map(|e| e.path()) {
            if path.extension().and_then(|e| e.to_str()) != Some("item") {
                continue;
            }
            match std::fs::read_to_string(&path) {
                Ok(text) => match parse_manifest(&text) {
                    // Manifest kalmış ama klasör silinmiş olabilir (gerçek sistemde GTA V vakası).
                    Some(game) if !std::path::Path::new(&game.install_dir).is_dir() => {
                        tracing::info!(title = %game.title, "Epic oyun klasörü yok, kurulu sayılmadı");
                    }
                    Some(game) => games.push(game),
                    None => tracing::debug!(?path, "Epic manifesti oyun değil veya bozuk"),
                },
                Err(err) => tracing::warn!(?path, %err, "Epic manifesti okunamadı"),
            }
        }
        Ok(games)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item_at(dir: &str, name: &str, app: &str, cats: &str) -> String {
        let dir = dir.replace('\\', "\\\\");
        format!(
            r#"{{"DisplayName":"{name}","AppName":"{app}","InstallLocation":"{dir}","CatalogNamespace":"ns1","CatalogItemId":"item1","AppCategories":[{cats}]}}"#
        )
    }

    fn item(name: &str, app: &str, main: &str, cats: &str, incomplete: bool) -> String {
        format!(
            r#"{{"DisplayName":"{name}","AppName":"{app}","MainGameAppName":"{main}","InstallLocation":"F:\\Oyunlar\\{name}","LaunchExecutable":"Bin/{name}.exe","CatalogNamespace":"ns1","CatalogItemId":"item1","InstallSize":42,"bIsIncompleteInstall":{incomplete},"AppCategories":[{cats}]}}"#
        )
    }

    #[test]
    fn parses_game_manifest() {
        let g = parse_manifest(&item(
            "Dying Light",
            "abc",
            "",
            r#""games","applications""#,
            false,
        ))
        .unwrap();
        assert_eq!(g.title, "Dying Light");
        assert_eq!(g.platform_id, "abc");
        assert_eq!(
            g.launch_uri,
            "com.epicgames.launcher://apps/ns1%3Aitem1%3Aabc?action=launch&silent=true"
        );
        assert!(g.executable.unwrap().ends_with("Dying Light.exe"));
        assert_eq!(g.size_bytes, Some(42));
    }

    #[test]
    fn filters_dlc_incomplete_and_non_games() {
        assert!(parse_manifest(&item("DLC", "dlc", "abc", r#""addons""#, false)).is_none());
        assert!(parse_manifest(&item("Yarım", "y", "", r#""games""#, true)).is_none());
        assert!(parse_manifest(&item("Unreal Engine", "ue", "", r#""engines""#, false)).is_none());
        assert!(parse_manifest("{ bozuk").is_none());
        // Ana oyun adı kendisine eşitse DLC değildir.
        assert!(parse_manifest(&item("Oyun", "o", "o", r#""games""#, false)).is_some());
    }

    #[test]
    fn scans_directory_and_skips_other_files() {
        let tmp = tempfile::tempdir().unwrap();
        let installed = tmp.path().join("Şahin'in Fortnite'ı");
        std::fs::create_dir_all(&installed).unwrap();
        let fortnite = item_at(&installed.to_string_lossy(), "Fortnite", "f", r#""games""#);
        std::fs::write(tmp.path().join("a.item"), fortnite).unwrap();
        // Manifesti kalmış ama klasörü silinmiş oyun kurulu sayılmaz.
        let gone = item_at(
            &tmp.path().join("silinmis").to_string_lossy(),
            "GTA V",
            "g",
            r#""games""#,
        );
        std::fs::write(tmp.path().join("d.item"), gone).unwrap();
        std::fs::write(tmp.path().join("b.item"), "bozuk").unwrap();
        std::fs::write(
            tmp.path().join("c.txt"),
            item("X", "x", "", r#""games""#, false),
        )
        .unwrap();
        let games = EpicDetector::new(Some(tmp.path().to_path_buf()))
            .detect()
            .unwrap();
        assert_eq!(games.len(), 1);
        assert!(EpicDetector::new(None).detect().unwrap().is_empty());
    }
}
