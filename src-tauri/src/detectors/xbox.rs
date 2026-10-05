//! Xbox / Microsoft Store (GDK) oyunları.
//!
//! Windows paket API'si (`PackageManager.FindPackagesByUserSecurityId`, yönetici gerektirmez) ile
//! kullanıcının paketleri listelenir; kurulum klasöründe `MicrosoftGame.config` olanlar oyundur.
//! Ad: `ShellVisuals DefaultDisplayName`; uygulama kimliği: `AppxManifest.xml` → `<Application Id>`.
//! Başlatma: `shell:AppsFolder\<PackageFamilyName>!<AppId>`.
//! Sınırlama: `WindowsApps` klasörü korumalı olduğundan exe ikonu/boyutu alınamayabilir.
use std::path::Path;

use super::{DetectedGame, DetectorError, GameDetector, Platform};

/// XML'de `<etiket ... ad="değer"` biçimindeki ilk özniteliği okur (tam XML ayrıştırıcı gerekmez).
pub fn xml_attr(xml: &str, tag: &str, attr: &str) -> Option<String> {
    let start = xml.find(&format!("<{tag}"))?;
    let rest = &xml[start..];
    let end = rest.find('>').unwrap_or(rest.len());
    let el = &rest[..end];
    let key = format!("{attr}=\"");
    let a = el.find(&key)? + key.len();
    let b = el[a..].find('"')? + a;
    let v = el[a..b].trim();
    (!v.is_empty()).then(|| v.to_owned())
}

/// Paket klasöründen oyun bilgisi üretir; oyun değilse `None`.
pub fn game_from_package(
    dir: &Path,
    family_name: &str,
    fallback_name: &str,
) -> Option<DetectedGame> {
    let config = std::fs::read_to_string(dir.join("MicrosoftGame.config")).ok()?;
    let manifest = std::fs::read_to_string(dir.join("AppxManifest.xml")).unwrap_or_default();
    let title = xml_attr(&config, "ShellVisuals", "DefaultDisplayName")
        .filter(|t| !t.starts_with("ms-resource:"))
        .unwrap_or_else(|| fallback_name.to_owned());
    let app_id = xml_attr(&manifest, "Application", "Id").unwrap_or_else(|| "Game".to_owned());
    Some(DetectedGame {
        platform: Platform::Xbox,
        platform_id: family_name.to_owned(),
        title,
        install_dir: dir.to_string_lossy().into_owned(),
        executable: None,
        launch_uri: format!(r"shell:AppsFolder\{family_name}!{app_id}"),
        launch_args: None,
        size_bytes: None,
        last_played: None,
    })
}

pub struct XboxDetector;

impl GameDetector for XboxDetector {
    fn platform(&self) -> Platform {
        Platform::Xbox
    }

    #[cfg(windows)]
    fn detect(&self) -> Result<Vec<DetectedGame>, DetectorError> {
        use windows::core::HSTRING;
        use windows::Management::Deployment::PackageManager;

        let to_err =
            |e: windows::core::Error| DetectorError::Io(std::io::Error::other(e.to_string()));
        let pm = PackageManager::new().map_err(to_err)?;
        // Boş SID = geçerli kullanıcı (yönetici gerektirmez).
        let packages = pm
            .FindPackagesByUserSecurityId(&HSTRING::new())
            .map_err(to_err)?;
        let mut out = Vec::new();
        for pkg in packages {
            if pkg.IsFramework().unwrap_or(true) || pkg.IsResourcePackage().unwrap_or(true) {
                continue;
            }
            let Ok(path) = pkg.InstalledPath() else {
                continue;
            };
            let dir = std::path::PathBuf::from(path.to_string_lossy());
            let Ok(family) = pkg.Id().and_then(|id| id.FamilyName()) else {
                continue;
            };
            let fallback = pkg
                .DisplayName()
                .map(|n| n.to_string_lossy())
                .unwrap_or_default();
            if let Some(g) = game_from_package(&dir, &family.to_string_lossy(), &fallback) {
                out.push(g);
            }
        }
        Ok(out)
    }

    #[cfg(not(windows))]
    fn detect(&self) -> Result<Vec<DetectedGame>, DetectorError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_game_config_and_manifest() {
        let t = tempfile::tempdir().unwrap();
        std::fs::write(
            t.path().join("MicrosoftGame.config"),
            r#"<Game configVersion="1"><Identity Name="Microsoft.MinecraftUWP"/><ShellVisuals DefaultDisplayName="Minecraft for Windows" PublisherDisplayName="Microsoft Studios"/></Game>"#,
        )
        .unwrap();
        std::fs::write(t.path().join("AppxManifest.xml"), r#"<Package><Applications><Application Id="Game" Executable="Minecraft.Windows.exe"></Application></Applications></Package>"#).unwrap();
        let g = game_from_package(t.path(), "Microsoft.MinecraftUWP_8wekyb3d8bbwe", "x").unwrap();
        assert_eq!(g.title, "Minecraft for Windows");
        assert_eq!(
            g.launch_uri,
            r"shell:AppsFolder\Microsoft.MinecraftUWP_8wekyb3d8bbwe!Game"
        );
    }

    #[test]
    fn non_game_packages_and_resource_names_are_handled() {
        let t = tempfile::tempdir().unwrap();
        assert!(
            game_from_package(t.path(), "Uygulama_x", "Hesap Makinesi").is_none(),
            "MicrosoftGame.config yoksa oyun değil"
        );
        std::fs::write(
            t.path().join("MicrosoftGame.config"),
            r#"<ShellVisuals DefaultDisplayName="ms-resource:AppName"/>"#,
        )
        .unwrap();
        assert_eq!(
            game_from_package(t.path(), "P_x", "Roblox").unwrap().title,
            "Roblox"
        );
    }

    #[cfg(windows)]
    #[test]
    fn enumerates_real_packages_without_admin() {
        // Hata vermeden çalışmalı; bu makinede Minecraft/Roblox bulunur (yoksa boş liste de geçerli).
        let games = XboxDetector.detect().unwrap();
        assert!(games
            .iter()
            .all(|g| g.launch_uri.starts_with(r"shell:AppsFolder\")));
    }
}
