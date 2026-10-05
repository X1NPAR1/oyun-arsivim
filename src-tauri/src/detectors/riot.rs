//! Riot Games (VALORANT, League of Legends, Legends of Runeterra).
//!
//! `%ProgramData%\Riot Games\Metadata\<ürün>.<yama hattı>\<ürün>.<hat>.product_settings.yaml` →
//! `product_install_full_path`. İstemci yolu `RiotClientInstalls.json` → `rc_default`.
//! Başlatma: `RiotClientServices.exe --launch-product=<ürün> --launch-patchline=<hat>`.
use std::path::{Path, PathBuf};

use super::{DetectedGame, DetectorError, GameDetector, Platform};

pub struct RiotDetector {
    root: Option<PathBuf>,
}

impl RiotDetector {
    pub fn new(root: Option<PathBuf>) -> Self {
        Self { root }
    }

    pub fn system_default() -> Self {
        Self::new(
            std::env::var_os("ProgramData")
                .map(|p| PathBuf::from(p).join("Riot Games"))
                .filter(|p| p.is_dir()),
        )
    }
}

fn title_of(product: &str, patchline: &str) -> Option<String> {
    let base = match product {
        "valorant" => "VALORANT",
        "league_of_legends" => "League of Legends",
        "bacon" => "Legends of Runeterra",
        "lion" => "2XKO",
        _ => return None, // istemci, TFT (LoL içinde) ve bilinmeyenler
    };
    Some(if patchline == "live" {
        base.to_owned()
    } else {
        format!("{base} ({})", patchline.to_uppercase())
    })
}

/// YAML'dan tek bir anahtarın tırnaklı değerini okur (tam YAML ayrıştırıcıya gerek yok).
pub fn yaml_value(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let rest = l.trim_start().strip_prefix(key)?.strip_prefix(':')?;
        let v = rest.trim().trim_matches('"').to_owned();
        (!v.is_empty()).then_some(v)
    })
}

/// `RiotClientInstalls.json` → `rc_default` (istemci exe yolu).
pub fn client_path(json: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    v["rc_default"]
        .as_str()
        .or_else(|| v["rc_live"].as_str())
        .map(|s| s.replace('/', "\\"))
}

impl GameDetector for RiotDetector {
    fn platform(&self) -> Platform {
        Platform::Riot
    }

    fn detect(&self) -> Result<Vec<DetectedGame>, DetectorError> {
        let Some(root) = &self.root else {
            return Ok(Vec::new());
        };
        let client = std::fs::read_to_string(root.join("RiotClientInstalls.json"))
            .ok()
            .and_then(|j| client_path(&j))
            .filter(|p| Path::new(p).is_file());
        let Ok(entries) = std::fs::read_dir(root.join("Metadata")) else {
            return Ok(Vec::new());
        };
        let mut out = Vec::new();
        for dir in entries.flatten().map(|e| e.path()).filter(|p| p.is_dir()) {
            let name = dir
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let Some((product, patchline)) = name.split_once('.') else {
                continue;
            };
            if patchline.contains('.') {
                continue; // ör. league_of_legends.live.game_patch
            }
            let Some(title) = title_of(product, patchline) else {
                continue;
            };
            let settings = dir.join(format!("{name}.product_settings.yaml"));
            let Some(install) = std::fs::read_to_string(&settings)
                .ok()
                .and_then(|t| yaml_value(&t, "product_install_full_path"))
                .map(|p| p.replace('/', "\\"))
                .filter(|p| Path::new(p).is_dir())
            else {
                continue;
            };
            out.push(DetectedGame {
                platform: Platform::Riot,
                platform_id: name.clone(),
                title,
                install_dir: install,
                executable: client.clone(),
                launch_uri: String::new(),
                launch_args: Some(format!(
                    "--launch-product={product} --launch-patchline={patchline}"
                )),
                size_bytes: None,
                last_played: None,
            });
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_riot() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("Riot Games");
        let games = t.path().join("Oyunlar");
        let client = games.join("Riot Client").join("RiotClientServices.exe");
        std::fs::create_dir_all(client.parent().unwrap()).unwrap();
        std::fs::write(&client, b"MZ").unwrap();
        let fwd = |p: &Path| p.to_string_lossy().replace('\\', "/");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("RiotClientInstalls.json"),
            format!(r#"{{"rc_default":"{}"}}"#, fwd(&client)),
        )
        .unwrap();
        let mk = |name: &str, install: Option<&Path>| {
            let d = root.join("Metadata").join(name);
            std::fs::create_dir_all(&d).unwrap();
            let line = install
                .map(|i| format!("product_install_full_path: \"{}\"\n", fwd(i)))
                .unwrap_or_default();
            std::fs::write(
                d.join(format!("{name}.product_settings.yaml")),
                format!("auto_patching: false\n{line}"),
            )
            .unwrap();
        };
        let val = games.join("VALORANT").join("live");
        std::fs::create_dir_all(&val).unwrap();
        mk("valorant.live", Some(&val));
        mk("league_of_legends.live", Some(&games.join("LoL silinmiş")));
        mk("Riot Client", None);
        mk("teamfighttactics.live", None);
        std::fs::create_dir_all(
            root.join("Metadata")
                .join("league_of_legends.live.game_patch"),
        )
        .unwrap();
        (t, root, client)
    }

    #[test]
    fn finds_installed_riot_games_with_client_launch_args() {
        let (_t, root, client) = fake_riot();
        let games = RiotDetector::new(Some(root)).detect().unwrap();
        assert_eq!(games.len(), 1, "silinmiş LoL, istemci ve TFT elenir");
        let v = &games[0];
        assert_eq!(v.title, "VALORANT");
        assert_eq!(
            v.executable.as_deref(),
            Some(client.to_string_lossy().as_ref())
        );
        assert_eq!(
            v.launch_args.as_deref(),
            Some("--launch-product=valorant --launch-patchline=live")
        );
    }

    #[test]
    fn helpers() {
        assert_eq!(
            yaml_value(
                "a: 1\n  product_install_full_path: \"F:/X\"",
                "product_install_full_path"
            )
            .as_deref(),
            Some("F:/X")
        );
        assert_eq!(
            title_of("league_of_legends", "pbe").as_deref(),
            Some("League of Legends (PBE)")
        );
        assert!(RiotDetector::new(None).detect().unwrap().is_empty());
    }
}
