//! Kayıt defteri tabanlı platformlar: GOG Galaxy, EA app, Ubisoft Connect, Battle.net, Rockstar.
//! Yöntemler ve sınırlamalar: docs/PLATFORMS.md. Hepsinde ortak kural: kurulum klasörü diskte yoksa
//! oyun kurulu sayılmaz (kaldırılmış ama kaydı kalmış oyunlar).
use std::path::{Path, PathBuf};

use super::registry::RegReader;
use super::{DetectedGame, DetectorError, GameDetector, Platform};

const WOW: &str = r"HKLM\SOFTWARE\WOW6432Node";
const UNINSTALL: &[&str] = &[
    r"HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
    r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
];

fn clean_dir(raw: &str) -> Option<String> {
    let d = raw
        .trim()
        .trim_matches('"')
        .trim_end_matches(['\\', '/'])
        .to_owned();
    (!d.is_empty() && Path::new(&d).is_dir()).then_some(d)
}

fn game(platform: Platform, id: String, title: String, dir: String) -> DetectedGame {
    DetectedGame {
        launch_args: None,
        platform,
        platform_id: id,
        title,
        install_dir: dir,
        executable: None,
        launch_uri: String::new(),
        size_bytes: None,
        last_played: None,
    }
}

// ───────────── GOG Galaxy ─────────────

pub struct GogDetector {
    reg: Box<dyn RegReader>,
}

impl GogDetector {
    pub fn new(reg: Box<dyn RegReader>) -> Self {
        Self { reg }
    }
}

impl GameDetector for GogDetector {
    fn platform(&self) -> Platform {
        Platform::Gog
    }

    /// `HKLM\SOFTWARE\WOW6432Node\GOG.com\Games\<id>` → gameName, path, exe, launchParam, workingDir.
    /// Galaxy'nin dış "başlat" adresi yok (yalnız oyun sayfasını açar); oyun exe'siyle başlatılır.
    fn detect(&self) -> Result<Vec<DetectedGame>, DetectorError> {
        let base = format!(r"{WOW}\GOG.com\Games");
        let mut out = Vec::new();
        for id in self.reg.subkeys(&base) {
            let key = format!(r"{base}\{id}");
            let (Some(title), Some(dir)) = (
                self.reg.value(&key, "gameName"),
                self.reg.value(&key, "path").and_then(|p| clean_dir(&p)),
            ) else {
                continue;
            };
            let mut g = game(Platform::Gog, id, title, dir);
            g.executable = self
                .reg
                .value(&key, "exe")
                .filter(|e| Path::new(e).is_file());
            g.launch_args = self
                .reg
                .value(&key, "launchParam")
                .filter(|a| !a.trim().is_empty());
            out.push(g);
        }
        Ok(out)
    }
}

// ───────────── EA app ─────────────

pub struct EaDetector {
    reg: Box<dyn RegReader>,
}

impl EaDetector {
    pub fn new(reg: Box<dyn RegReader>) -> Self {
        Self { reg }
    }
}

/// EA'nın kendi bileşenleri (oyun değil).
const EA_NON_GAMES: &[&str] = &[
    "ea core",
    "ea desktop",
    "eadm",
    "origin",
    "ea app",
    "ea installer",
];

/// `__Installer\installerdata.xml` içindeki ilk `<contentID>` (EA app başlatma adresi için).
pub fn ea_content_id(xml: &str) -> Option<String> {
    let start = xml.find("<contentID>")? + "<contentID>".len();
    let end = xml[start..].find("</contentID>")? + start;
    let id = xml[start..end].trim();
    (!id.is_empty()).then(|| id.to_owned())
}

impl GameDetector for EaDetector {
    fn platform(&self) -> Platform {
        Platform::Ea
    }

    /// `HKLM\SOFTWARE\WOW6432Node\{Electronic Arts, EA Games}\<oyun>` → "Install Dir".
    fn detect(&self) -> Result<Vec<DetectedGame>, DetectorError> {
        let mut out: Vec<DetectedGame> = Vec::new();
        for vendor in ["Electronic Arts", "EA Games"] {
            let base = format!(r"{WOW}\{vendor}");
            for name in self.reg.subkeys(&base) {
                if EA_NON_GAMES.contains(&name.to_lowercase().as_str()) {
                    continue;
                }
                let key = format!(r"{base}\{name}");
                let Some(dir) = self
                    .reg
                    .value(&key, "Install Dir")
                    .and_then(|p| clean_dir(&p))
                else {
                    continue;
                };
                if out.iter().any(|g| g.install_dir.eq_ignore_ascii_case(&dir)) {
                    continue;
                }
                let mut g = game(Platform::Ea, name.clone(), name, dir.clone());
                let xml = std::fs::read_to_string(
                    PathBuf::from(&dir).join(r"__Installer\installerdata.xml"),
                )
                .unwrap_or_default();
                if let Some(id) = ea_content_id(&xml) {
                    g.launch_uri = format!("origin2://game/launch?offerIds={id}&autoDownload=1");
                    g.platform_id = id;
                }
                out.push(g);
            }
        }
        Ok(out)
    }
}

// ───────────── Ubisoft Connect ─────────────

pub struct UbisoftDetector {
    reg: Box<dyn RegReader>,
}

impl UbisoftDetector {
    pub fn new(reg: Box<dyn RegReader>) -> Self {
        Self { reg }
    }
}

impl GameDetector for UbisoftDetector {
    fn platform(&self) -> Platform {
        Platform::Ubisoft
    }

    /// `...\Ubisoft\Launcher\Installs\<id>` → InstallDir; ad: Uninstall\"Uplay Install <id>".
    fn detect(&self) -> Result<Vec<DetectedGame>, DetectorError> {
        let base = format!(r"{WOW}\Ubisoft\Launcher\Installs");
        let mut out = Vec::new();
        for id in self.reg.subkeys(&base) {
            let Some(dir) = self
                .reg
                .value(&format!(r"{base}\{id}"), "InstallDir")
                .and_then(|p| clean_dir(&p))
            else {
                continue;
            };
            let title = UNINSTALL
                .iter()
                .find_map(|u| {
                    self.reg
                        .value(&format!(r"{u}\Uplay Install {id}"), "DisplayName")
                })
                .unwrap_or_else(|| {
                    Path::new(&dir)
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default()
                });
            let mut g = game(Platform::Ubisoft, id.clone(), title, dir);
            g.launch_uri = format!("uplay://launch/{id}/0");
            out.push(g);
        }
        Ok(out)
    }
}

// ───────────── Battle.net ─────────────

pub struct BattlenetDetector {
    reg: Box<dyn RegReader>,
}

impl BattlenetDetector {
    pub fn new(reg: Box<dyn RegReader>) -> Self {
        Self { reg }
    }
}

/// Battle.net ürün kodları (başlatma adresi `battlenet://<kod>`).
pub fn battlenet_code(title: &str) -> Option<&'static str> {
    let t = title.to_lowercase();
    [
        ("world of warcraft", "WoW"),
        ("overwatch", "Pro"),
        ("diablo iv", "Fen"),
        ("diablo iii", "D3"),
        ("diablo ii", "OSI"),
        ("diablo immortal", "ANBS"),
        ("hearthstone", "WTCG"),
        ("starcraft ii", "S2"),
        ("starcraft", "S1"),
        ("heroes of the storm", "Hero"),
        ("warcraft iii", "W3"),
        ("call of duty", "AUKS"),
        ("crash bandicoot", "WLBY"),
    ]
    .iter()
    .find(|(name, _)| t.contains(name))
    .map(|(_, code)| *code)
}

impl GameDetector for BattlenetDetector {
    fn platform(&self) -> Platform {
        Platform::Battlenet
    }

    /// Uninstall kayıtları: Publisher "Blizzard Entertainment" (Battle.net istemcisinin kendisi hariç).
    fn detect(&self) -> Result<Vec<DetectedGame>, DetectorError> {
        let mut out: Vec<DetectedGame> = Vec::new();
        for u in UNINSTALL {
            for key_name in self.reg.subkeys(u) {
                let key = format!(r"{u}\{key_name}");
                let publisher = self.reg.value(&key, "Publisher").unwrap_or_default();
                let Some(title) = self.reg.value(&key, "DisplayName") else {
                    continue;
                };
                if !publisher.to_lowercase().contains("blizzard")
                    || title.eq_ignore_ascii_case("Battle.net")
                {
                    continue;
                }
                let Some(dir) = self
                    .reg
                    .value(&key, "InstallLocation")
                    .and_then(|p| clean_dir(&p))
                else {
                    continue;
                };
                if out.iter().any(|g| g.install_dir.eq_ignore_ascii_case(&dir)) {
                    continue;
                }
                let mut g = game(Platform::Battlenet, key_name.clone(), title.clone(), dir);
                g.launch_uri = battlenet_code(&title)
                    .map_or_else(|| "battlenet://".to_owned(), |c| format!("battlenet://{c}"));
                out.push(g);
            }
        }
        Ok(out)
    }
}

// ───────────── Rockstar Games Launcher ─────────────

pub struct RockstarDetector {
    reg: Box<dyn RegReader>,
}

impl RockstarDetector {
    pub fn new(reg: Box<dyn RegReader>) -> Self {
        Self { reg }
    }
}

const ROCKSTAR_NON_GAMES: &[&str] = &[
    "launcher",
    "rockstar games social club",
    "rockstar games sdk",
    "steam",
];

/// Oyun klasöründeki başlatıcı exe: önce `Play*.exe`, yoksa kökteki en büyük exe.
pub fn pick_main_exe(dir: &Path) -> Option<PathBuf> {
    let exes: Vec<(PathBuf, u64)> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|x| x.to_str())
                .is_some_and(|x| x.eq_ignore_ascii_case("exe"))
        })
        .filter(|p| {
            let n = p
                .file_name()
                .map(|n| n.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            !(n.starts_with("unins")
                || n.contains("setup")
                || n.contains("crash")
                || n.contains("redist"))
        })
        .map(|p| {
            let size = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
            (p, size)
        })
        .collect();
    exes.iter()
        .find(|(p, _)| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().to_lowercase().starts_with("play"))
        })
        .or_else(|| exes.iter().max_by_key(|(_, s)| *s))
        .map(|(p, _)| p.clone())
}

impl GameDetector for RockstarDetector {
    fn platform(&self) -> Platform {
        Platform::Rockstar
    }

    /// `HKLM\SOFTWARE\WOW6432Node\Rockstar Games\<oyun>` → InstallFolder (Steam/Epic kopyaları
    /// kendi dedektörlerinde; burada kayıt boşsa atlanır ve tekilleştirme çakışmayı çözer).
    fn detect(&self) -> Result<Vec<DetectedGame>, DetectorError> {
        let base = format!(r"{WOW}\Rockstar Games");
        let mut out = Vec::new();
        for name in self.reg.subkeys(&base) {
            if ROCKSTAR_NON_GAMES.contains(&name.to_lowercase().as_str()) {
                continue;
            }
            let Some(dir) = self
                .reg
                .value(&format!(r"{base}\{name}"), "InstallFolder")
                .and_then(|p| clean_dir(&p))
            else {
                continue;
            };
            let mut g = game(Platform::Rockstar, name.clone(), name, dir.clone());
            g.executable = pick_main_exe(Path::new(&dir)).map(|p| p.to_string_lossy().into_owned());
            out.push(g);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detectors::registry::FakeRegistry;

    fn tmpdir(name: &str) -> (tempfile::TempDir, String) {
        let t = tempfile::tempdir().unwrap();
        let d = t.path().join(name);
        std::fs::create_dir_all(&d).unwrap();
        let s = d.to_string_lossy().into_owned();
        (t, s)
    }

    #[test]
    fn gog_reads_registry_and_skips_missing_dirs() {
        let (_t, dir) = tmpdir("Şahin'in GOG Oyunu");
        let exe = format!(r"{dir}\game.exe");
        std::fs::write(&exe, b"MZ").unwrap();
        let reg = FakeRegistry::default()
            .with(
                r"HKLM\SOFTWARE\WOW6432Node\GOG.com\Games\1207658924",
                &[
                    ("gameName", "The Witcher"),
                    ("path", &dir),
                    ("exe", &exe),
                    ("launchParam", "-skip"),
                ],
            )
            .with(
                r"HKLM\SOFTWARE\WOW6432Node\GOG.com\Games\999",
                &[("gameName", "Silinmiş"), ("path", r"Z:\yok")],
            );
        let games = GogDetector::new(Box::new(reg)).detect().unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].title, "The Witcher");
        assert_eq!(games[0].executable.as_deref(), Some(exe.as_str()));
        assert_eq!(games[0].launch_args.as_deref(), Some("-skip"));
        assert!(games[0].launch_uri.is_empty(), "GOG exe ile başlatılır");
    }

    #[test]
    fn ea_filters_launcher_entries_and_reads_content_id() {
        let (_t, dir) = tmpdir("Battlefield 1");
        std::fs::create_dir_all(format!(r"{dir}\__Installer")).unwrap();
        std::fs::write(
            format!(r"{dir}\__Installer\installerdata.xml"),
            "<x><contentIDs><contentID>1026023</contentID></contentIDs></x>",
        )
        .unwrap();
        let reg = FakeRegistry::default()
            .with(
                r"HKLM\SOFTWARE\WOW6432Node\Electronic Arts\EA Desktop",
                &[("Install Dir", &dir)],
            )
            .with(
                r"HKLM\SOFTWARE\WOW6432Node\Electronic Arts\Battlefield 1",
                &[("Install Dir", &format!(r"{dir}\"))],
            )
            .with(
                r"HKLM\SOFTWARE\WOW6432Node\EA Games\Battlefield 1",
                &[("Install Dir", &dir)],
            );
        let games = EaDetector::new(Box::new(reg)).detect().unwrap();
        assert_eq!(
            games.len(),
            1,
            "launcher kaydı elenir, aynı klasör iki kez sayılmaz"
        );
        assert_eq!(
            games[0].launch_uri,
            "origin2://game/launch?offerIds=1026023&autoDownload=1"
        );
        assert_eq!(ea_content_id("<a/>"), None);
    }

    #[test]
    fn ubisoft_uses_uninstall_name_and_uplay_uri() {
        let (_t, dir) = tmpdir("Rainbow Six Siege");
        let reg = FakeRegistry::default()
            .with(r"HKLM\SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs\635", &[("InstallDir", &dir)])
            .with(r"HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Uplay Install 635", &[("DisplayName", "Tom Clancy's Rainbow Six Siege")]);
        let g = &UbisoftDetector::new(Box::new(reg)).detect().unwrap()[0];
        assert_eq!(g.title, "Tom Clancy's Rainbow Six Siege");
        assert_eq!(g.launch_uri, "uplay://launch/635/0");
    }

    #[test]
    fn battlenet_finds_blizzard_games_but_not_the_client() {
        let (_t, dir) = tmpdir("Overwatch");
        let u = r"HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall";
        let reg = FakeRegistry::default()
            .with(
                &format!(r"{u}\Overwatch"),
                &[
                    ("DisplayName", "Overwatch"),
                    ("Publisher", "Blizzard Entertainment"),
                    ("InstallLocation", &dir),
                ],
            )
            .with(
                &format!(r"{u}\Battle.net"),
                &[
                    ("DisplayName", "Battle.net"),
                    ("Publisher", "Blizzard Entertainment"),
                    ("InstallLocation", &dir),
                ],
            )
            .with(
                &format!(r"{u}\Other"),
                &[
                    ("DisplayName", "Başka"),
                    ("Publisher", "Acme"),
                    ("InstallLocation", &dir),
                ],
            );
        let games = BattlenetDetector::new(Box::new(reg)).detect().unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].launch_uri, "battlenet://Pro");
        assert_eq!(battlenet_code("Bilinmeyen"), None);
    }

    #[test]
    fn rockstar_skips_launcher_and_picks_play_exe() {
        let (_t, dir) = tmpdir("Grand Theft Auto V");
        std::fs::write(format!(r"{dir}\GTA5.exe"), vec![0u8; 2000]).unwrap();
        std::fs::write(format!(r"{dir}\PlayGTAV.exe"), b"MZ").unwrap();
        std::fs::write(format!(r"{dir}\unins000.exe"), vec![0u8; 5000]).unwrap();
        let reg = FakeRegistry::default()
            .with(
                r"HKLM\SOFTWARE\WOW6432Node\Rockstar Games\Grand Theft Auto V",
                &[("InstallFolder", &dir)],
            )
            .with(
                r"HKLM\SOFTWARE\WOW6432Node\Rockstar Games\Launcher",
                &[("InstallFolder", &dir)],
            )
            .with(
                r"HKLM\SOFTWARE\WOW6432Node\Rockstar Games\Red Dead Redemption 2",
                &[],
            );
        let games = RockstarDetector::new(Box::new(reg)).detect().unwrap();
        assert_eq!(games.len(), 1);
        assert!(games[0]
            .executable
            .as_deref()
            .unwrap()
            .ends_with("PlayGTAV.exe"));
    }
}
