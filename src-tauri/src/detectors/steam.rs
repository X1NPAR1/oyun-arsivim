//! Steam dedektörü. Yöntem: docs/PLATFORMS.md#steam
//!
//! Steam kök klasörü → `steamapps/libraryfolders.vdf` (tüm kütüphaneler, farklı diskler dahil) →
//! her kütüphanede `steamapps/appmanifest_<appid>.acf`. Araçlar, çalışma zamanları ve
//! redistributable paketler oyun değildir ve elenir.
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::vdf::{self, Vdf};
use super::{DetectedGame, DetectorError, GameDetector, Platform};

/// Oyun olmayan bilinen Steam uygulamaları (redist, Proton, Linux runtime, SteamVR, SDK'lar).
const NON_GAME_APPIDS: &[&str] = &[
    "228980",  // Steamworks Common Redistributables
    "250820",  // SteamVR
    "1070560", // Steam Linux Runtime 1.0 (scout)
    "1391110", // Steam Linux Runtime 2.0 (soldier)
    "1628350", // Steam Linux Runtime 3.0 (sniper)
    "1493710", // Proton Experimental
    "2180100", // Proton Hotfix
    "961940",  // Proton 3.16
    "1054830", // Proton 5.0
    "1113280", // Proton 5.13
    "1420170", // Proton 6.3
    "1580130", // Proton 7.0
    "2348590", // Proton 8.0
    "2805730", // Proton 9.0
    "1007",    // Steamworks SDK Redist
    "1826330", // Steamworks SDK Redist
    "243750",  // Source SDK Base 2013 Multiplayer
    "243730",  // Source SDK Base 2013 Singleplayer
    "431960",  // Wallpaper Engine (yazılım)
];

const NON_GAME_NAME_PATTERNS: &[&str] = &[
    "steamworks common redistributables",
    "steam linux runtime",
    "proton ",
    "proton-",
    "dedicated server",
    "redistributable",
    "steamvr",
];

/// `StateFlags` içinde "tam kurulu" biti.
const STATE_FULLY_INSTALLED: u32 = 4;

pub struct SteamDetector {
    root: Option<PathBuf>,
}

impl SteamDetector {
    pub fn new(root: Option<PathBuf>) -> Self {
        Self { root }
    }

    pub fn system_default() -> Self {
        Self::new(steam_root_from_registry())
    }

    fn library_paths(&self, root: &Path) -> Vec<PathBuf> {
        let mut paths = vec![root.to_path_buf()];
        let file = root.join("steamapps").join("libraryfolders.vdf");
        match std::fs::read_to_string(&file).map(|t| vdf::parse(&t)) {
            Ok(Ok(doc)) => {
                if let Some(folders) = doc.get("libraryfolders") {
                    for (_, entry) in folders.entries() {
                        // Yeni biçim: { "path" "..." }, eski biçim: "1" "D:\\SteamLibrary".
                        let path = match entry {
                            Vdf::Obj(_) => entry.str("path").map(str::to_owned),
                            Vdf::Str(s) => Some(s.clone()),
                        };
                        if let Some(p) = path {
                            paths.push(PathBuf::from(p));
                        }
                    }
                }
            }
            Ok(Err(err)) => tracing::warn!(?file, %err, "libraryfolders.vdf bozuk"),
            Err(err) => tracing::debug!(?file, %err, "libraryfolders.vdf okunamadı"),
        }
        // Aynı klasör farklı yazımlarla tekrar edebilir (f:/steam ve F:\steam).
        let mut seen = HashSet::new();
        paths.retain(|p| seen.insert(normalize(p)));
        paths
    }
}

fn normalize(p: &Path) -> String {
    p.to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase()
}

#[cfg(windows)]
pub fn steam_root_from_registry() -> Option<PathBuf> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Software\Valve\Steam")
        .and_then(|k| k.get_value::<String, _>("SteamPath"));
    let path = hkcu.or_else(|_| {
        RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(r"SOFTWARE\WOW6432Node\Valve\Steam")
            .and_then(|k| k.get_value::<String, _>("InstallPath"))
    });
    path.ok()
        .map(|p| PathBuf::from(p.replace('/', "\\")))
        .filter(|p| p.is_dir())
}

#[cfg(not(windows))]
pub fn steam_root_from_registry() -> Option<PathBuf> {
    None
}

fn is_non_game(appid: &str, name: &str) -> bool {
    let lower = name.to_lowercase();
    NON_GAME_APPIDS.contains(&appid) || NON_GAME_NAME_PATTERNS.iter().any(|p| lower.contains(p))
}

/// Tek bir `appmanifest_*.acf` dosyasını ayrıştırır. Oyun değilse veya tam kurulu değilse `None`.
pub fn parse_manifest(text: &str, library: &Path) -> Option<DetectedGame> {
    let doc = vdf::parse(text).ok()?;
    let app = doc.get("AppState")?;
    let appid = app.str("appid")?.trim().to_owned();
    let name = app.str("name")?.trim().to_owned();
    if appid.is_empty() || name.is_empty() || is_non_game(&appid, &name) {
        return None;
    }
    let flags: u32 = app
        .str("StateFlags")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    if flags & STATE_FULLY_INSTALLED == 0 {
        return None;
    }
    let install_dir = library
        .join("steamapps")
        .join("common")
        .join(app.str("installdir").unwrap_or(&name));
    Some(DetectedGame {
        platform: Platform::Steam,
        launch_uri: format!("steam://rungameid/{appid}"),
        platform_id: appid,
        title: name,
        install_dir: install_dir.to_string_lossy().into_owned(),
        executable: None,
        size_bytes: app.str("SizeOnDisk").and_then(|s| s.parse().ok()),
        last_played: app
            .str("LastPlayed")
            .and_then(|s| s.parse().ok())
            .filter(|t: &i64| *t > 0),
    })
}

impl GameDetector for SteamDetector {
    fn platform(&self) -> Platform {
        Platform::Steam
    }

    fn detect(&self) -> Result<Vec<DetectedGame>, DetectorError> {
        let Some(root) = &self.root else {
            return Ok(Vec::new());
        };
        let mut games = Vec::new();
        let mut seen = HashSet::new();
        for library in self.library_paths(root) {
            let steamapps = library.join("steamapps");
            let Ok(entries) = std::fs::read_dir(&steamapps) else {
                tracing::debug!(?steamapps, "kütüphane klasörü erişilemedi");
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let is_manifest = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("appmanifest_") && n.ends_with(".acf"));
                if !is_manifest {
                    continue;
                }
                match std::fs::read_to_string(&path) {
                    Ok(text) => {
                        if let Some(game) = parse_manifest(&text, &library) {
                            // Manifest kalmış ama oyun klasörü silinmişse kurulu sayılmaz.
                            if !Path::new(&game.install_dir).is_dir() {
                                tracing::info!(title = %game.title, "Steam oyun klasörü yok");
                                continue;
                            }
                            if seen.insert(game.platform_id.clone()) {
                                games.push(game);
                            }
                        }
                    }
                    Err(err) => tracing::warn!(?path, %err, "appmanifest okunamadı"),
                }
            }
        }
        Ok(games)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn acf(appid: &str, name: &str, flags: u32) -> String {
        format!(
            "\"AppState\"\n{{\n\t\"appid\"\t\t\"{appid}\"\n\t\"name\"\t\t\"{name}\"\n\t\"StateFlags\"\t\t\"{flags}\"\n\t\"installdir\"\t\t\"{name}\"\n\t\"LastPlayed\"\t\t\"1790880012\"\n\t\"SizeOnDisk\"\t\t\"1024\"\n}}\n"
        )
    }

    /// Gerçek yapıyı taklit eden sahte Steam: kök + ikinci kütüphane (Türkçe/özel karakterli yol).
    fn fake_steam() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Steam");
        let lib2 = tmp.path().join("Şahin'in Kütüphanesi");
        fs::create_dir_all(root.join("steamapps")).unwrap();
        fs::create_dir_all(lib2.join("steamapps")).unwrap();
        let lib2_escaped = lib2.to_string_lossy().replace('\\', "\\\\");
        fs::write(
            root.join("steamapps/libraryfolders.vdf"),
            format!(
                "\"libraryfolders\"\n{{\n\t\"0\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t}}\n\t\"1\"\n\t{{\n\t\t\"path\"\t\t\"{lib2_escaped}\"\n\t}}\n}}",
                root.to_string_lossy().replace('\\', "\\\\")
            ),
        )
        .unwrap();
        fs::write(
            root.join("steamapps/appmanifest_730.acf"),
            acf("730", "Counter-Strike 2", 4),
        )
        .unwrap();
        fs::write(
            root.join("steamapps/appmanifest_228980.acf"),
            acf("228980", "Steamworks Common Redistributables", 4),
        )
        .unwrap();
        fs::write(
            lib2.join("steamapps/appmanifest_304930.acf"),
            acf("304930", "Unturned", 4),
        )
        .unwrap();
        fs::write(
            lib2.join("steamapps/appmanifest_999.acf"),
            acf("999", "Yarım İndirme", 1026 & !4),
        )
        .unwrap();
        fs::write(
            lib2.join("steamapps/appmanifest_1.acf"),
            "\"AppState\" { \"appid\" ",
        )
        .unwrap();
        // Kurulu oyunların klasörleri; 555'in manifesti var ama klasörü yok (silinmiş).
        fs::create_dir_all(root.join("steamapps/common/Counter-Strike 2")).unwrap();
        fs::create_dir_all(lib2.join("steamapps/common/Unturned")).unwrap();
        fs::write(
            lib2.join("steamapps/appmanifest_555.acf"),
            acf("555", "Silinmiş Oyun", 4),
        )
        .unwrap();
        (tmp, root)
    }

    #[test]
    fn finds_games_across_libraries_and_filters_non_games() {
        let (_tmp, root) = fake_steam();
        let mut games = SteamDetector::new(Some(root)).detect().unwrap();
        games.sort_by(|a, b| a.title.cmp(&b.title));
        let titles: Vec<_> = games.iter().map(|g| g.title.as_str()).collect();
        assert_eq!(titles, ["Counter-Strike 2", "Unturned"]);
        let unturned = &games[1];
        assert_eq!(unturned.launch_uri, "steam://rungameid/304930");
        assert!(unturned.install_dir.contains("Şahin'in Kütüphanesi"));
        assert_eq!(unturned.last_played, Some(1_790_880_012));
        assert_eq!(unturned.size_bytes, Some(1024));
    }

    #[test]
    fn missing_root_is_empty_not_error() {
        assert!(SteamDetector::new(None).detect().unwrap().is_empty());
        let tmp = tempfile::tempdir().unwrap();
        assert!(SteamDetector::new(Some(tmp.path().join("yok")))
            .detect()
            .unwrap()
            .is_empty());
    }

    #[test]
    fn manifest_parsing_rules() {
        let lib = Path::new("D:\\SteamLibrary");
        assert!(parse_manifest(&acf("4000", "Garry's Mod", 4), lib).is_some());
        assert!(parse_manifest(&acf("1", "Proton 9.0", 4), lib).is_none());
        assert!(parse_manifest(&acf("2", "X Dedicated Server", 4), lib).is_none());
        assert!(parse_manifest(&acf("3", "Güncelleniyor", 6), lib).is_some()); // 4 biti açık
        assert!(parse_manifest("bozuk", lib).is_none());
    }

    #[test]
    fn duplicate_library_paths_are_collapsed() {
        assert_eq!(
            normalize(Path::new("f:/steam/")),
            normalize(Path::new("F:\\Steam"))
        );
    }
}
