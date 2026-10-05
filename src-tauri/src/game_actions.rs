//! Oyun bağlam menüsü eylemleri: kaldırma/yükleme (platformun kendi akışıyla), masaüstü kısayolu,
//! klasör açma. Kural: uygulama oyun dosyalarını kendisi SİLMEZ; kaldırmayı platform, kullanıcı
//! onayıyla yapar.
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::detectors::Platform;
use crate::library::Game;

/// Arayüzün kullanıcıya ne olacağını anlatabilmesi için eylemin nasıl gerçekleştiği.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum PlatformAction {
    /// Bu adres açılır (platform kendi onay penceresini gösterir).
    OpenUri(String),
    /// Platform bu eylemi dışarıdan desteklemiyor; açıklama ve açılacak sayfa.
    Guided { uri: String, hint: String },
}

const EPIC_LIBRARY: &str = "com.epicgames.launcher://store/library";
const WINDOWS_APPS: &str = "ms-settings:appsfeatures";

pub fn uninstall_action(game: &Game) -> PlatformAction {
    match game.platform {
        Platform::Steam => {
            PlatformAction::OpenUri(format!("steam://uninstall/{}", game.platform_id))
        }
        // Epic Launcher dışarıdan kaldırma adresi sunmuyor; kütüphane sayfası açılır.
        Platform::Epic => PlatformAction::Guided {
            uri: EPIC_LIBRARY.to_owned(),
            hint: format!(
                "Epic Games kütüphanesinde \"{}\" oyununun yanındaki ••• menüsünden Kaldır'ı seç.",
                game.title
            ),
        },
        Platform::Manual => PlatformAction::Guided {
            uri: WINDOWS_APPS.to_owned(),
            hint: format!(
                "Windows \"Yüklü uygulamalar\" listesinde \"{}\" oyununu bulup kaldırabilirsin.",
                game.title
            ),
        },
    }
}

pub fn install_action(game: &Game) -> Option<PlatformAction> {
    match game.platform {
        Platform::Steam => Some(PlatformAction::OpenUri(format!(
            "steam://install/{}",
            game.platform_id
        ))),
        Platform::Epic if game.launch_uri.contains("action=launch") => Some(
            PlatformAction::OpenUri(game.launch_uri.replace("action=launch", "action=install")),
        ),
        Platform::Epic => Some(PlatformAction::OpenUri(EPIC_LIBRARY.to_owned())),
        Platform::Manual => None,
    }
}

/// Windows dosya adında geçersiz karakterleri temizler; boş kalırsa yedek ad verir.
pub fn sanitize_file_name(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => ' ',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let trimmed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = trimmed.trim_end_matches(['.', ' ']).to_owned();
    // Windows'un ayrılmış adları (CON, NUL, COM1…) dosya adı olamaz.
    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "LPT1", "LPT2",
    ];
    if trimmed.is_empty() || reserved.contains(&trimmed.to_uppercase().as_str()) {
        "Oyun".to_owned()
    } else {
        trimmed.chars().take(120).collect()
    }
}

/// İnternet kısayolu (.url) içeriği. Windows bu dosyaları UTF-8 BOM'suz ANSI okur; Türkçe
/// karakterler için `[InternetShortcut.W]` bölümü UTF-7 yerine doğrudan URL'i kullanırız (URL ASCII).
pub fn url_shortcut_content(uri: &str, icon: Option<&Path>) -> String {
    let mut s = format!("[InternetShortcut]\r\nURL={uri}\r\n");
    if let Some(icon) = icon {
        s.push_str(&format!("IconFile={}\r\nIconIndex=0\r\n", icon.display()));
    }
    s
}

#[derive(Debug, thiserror::Error)]
pub enum ShortcutError {
    #[error("Masaüstü klasörü bulunamadı")]
    NoDesktop,
    #[error("Kısayol yazılamadı: {0}")]
    Io(#[from] std::io::Error),
    #[error("Kısayol oluşturulamadı: {0}")]
    Win(String),
    #[error("Bu oyunun başlatma bilgisi yok")]
    NoTarget,
}

/// Kısayol için `.ico`: oyunun ikonundan veya kapağının ortasından kare kırpılarak üretilir.
pub fn build_icon(source: &Path, target: &Path) -> Option<PathBuf> {
    let img = image::open(source).ok()?;
    let side = img.width().min(img.height());
    let x = (img.width() - side) / 2;
    let y = (img.height() - side) / 2;
    let square = img.crop_imm(x, y, side, side).resize_exact(
        256,
        256,
        image::imageops::FilterType::Lanczos3,
    );
    square
        .save_with_format(target, image::ImageFormat::Ico)
        .ok()?;
    Some(target.to_path_buf())
}

/// Kısayolu `dir` klasörüne yazar, oluşan dosyanın yolunu döndürür.
pub fn create_shortcut_in(
    dir: &Path,
    game: &Game,
    icon: Option<&Path>,
) -> Result<PathBuf, ShortcutError> {
    let name = sanitize_file_name(&game.title);
    match game.platform {
        Platform::Manual => {
            let exe = game.executable.as_deref().ok_or(ShortcutError::NoTarget)?;
            let path = dir.join(format!("{name}.lnk"));
            platform::write_lnk(&path, Path::new(exe), Path::new(&game.install_dir))?;
            Ok(path)
        }
        Platform::Steam | Platform::Epic => {
            if game.launch_uri.is_empty() {
                return Err(ShortcutError::NoTarget);
            }
            let path = dir.join(format!("{name}.url"));
            std::fs::write(&path, url_shortcut_content(&game.launch_uri, icon))?;
            Ok(path)
        }
    }
}

pub fn desktop_dir() -> Result<PathBuf, ShortcutError> {
    platform::desktop_dir()
}

#[cfg(windows)]
mod platform {
    use super::ShortcutError;
    use std::os::windows::ffi::OsStrExt;
    use std::path::{Path, PathBuf};
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, IPersistFile,
        CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::UI::Shell::{
        FOLDERID_Desktop, IShellLinkW, SHGetKnownFolderPath, ShellLink, KF_FLAG_DEFAULT,
    };

    fn wide(p: &Path) -> Vec<u16> {
        p.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    /// Masaüstü OneDrive'a veya başka bir sürücüye yönlendirilmiş olabilir; Known Folder API bunu çözer.
    pub fn desktop_dir() -> Result<PathBuf, ShortcutError> {
        // SAFETY: dönen PWSTR CoTaskMemFree ile serbest bırakılır.
        unsafe {
            let p = SHGetKnownFolderPath(&FOLDERID_Desktop, KF_FLAG_DEFAULT, None)
                .map_err(|_| ShortcutError::NoDesktop)?;
            let s = p.to_string().map_err(|_| ShortcutError::NoDesktop);
            CoTaskMemFree(Some(p.0 as *const _));
            Ok(PathBuf::from(s?))
        }
    }

    pub fn write_lnk(path: &Path, exe: &Path, workdir: &Path) -> Result<(), ShortcutError> {
        // SAFETY: COM bu iş parçacığında başlatılıp kapatılır; tüm arabellekler çağrı boyunca yaşar.
        unsafe {
            let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let result = (|| {
                let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
                    .map_err(|e| ShortcutError::Win(e.to_string()))?;
                let exe_w = wide(exe);
                let dir_w = wide(workdir);
                link.SetPath(PCWSTR(exe_w.as_ptr()))
                    .map_err(|e| ShortcutError::Win(e.to_string()))?;
                link.SetWorkingDirectory(PCWSTR(dir_w.as_ptr()))
                    .map_err(|e| ShortcutError::Win(e.to_string()))?;
                link.SetIconLocation(PCWSTR(exe_w.as_ptr()), 0)
                    .map_err(|e| ShortcutError::Win(e.to_string()))?;
                let file: IPersistFile =
                    link.cast().map_err(|e| ShortcutError::Win(e.to_string()))?;
                let path_w = wide(path);
                file.Save(PCWSTR(path_w.as_ptr()), true)
                    .map_err(|e| ShortcutError::Win(e.to_string()))
            })();
            if init.is_ok() {
                CoUninitialize();
            }
            result
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use super::ShortcutError;
    use std::path::{Path, PathBuf};
    pub fn desktop_dir() -> Result<PathBuf, ShortcutError> {
        Err(ShortcutError::NoDesktop)
    }
    pub fn write_lnk(_: &Path, _: &Path, _: &Path) -> Result<(), ShortcutError> {
        Err(ShortcutError::Win("yalnız Windows".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(platform: Platform, id: &str, title: &str) -> Game {
        Game {
            id: 7,
            platform,
            platform_id: id.into(),
            title: title.into(),
            install_dir: String::new(),
            executable: None,
            launch_uri: match platform {
                Platform::Steam => format!("steam://rungameid/{id}"),
                Platform::Epic => {
                    format!("com.epicgames.launcher://apps/ns%3Ai%3A{id}?action=launch&silent=true")
                }
                Platform::Manual => String::new(),
            },
            size_bytes: None,
            last_played: None,
            installed: true,
            added_at: 0,
            removed_at: None,
        }
    }

    #[test]
    fn platform_actions() {
        let cs = game(Platform::Steam, "730", "CS2");
        assert_eq!(
            uninstall_action(&cs),
            PlatformAction::OpenUri("steam://uninstall/730".into())
        );
        assert_eq!(
            install_action(&cs),
            Some(PlatformAction::OpenUri("steam://install/730".into()))
        );
        let dl = game(Platform::Epic, "abc", "Dying Light");
        assert!(matches!(
            uninstall_action(&dl),
            PlatformAction::Guided { .. }
        ));
        assert_eq!(
            install_action(&dl),
            Some(PlatformAction::OpenUri(
                "com.epicgames.launcher://apps/ns%3Ai%3Aabc?action=install&silent=true".into()
            ))
        );
        assert_eq!(install_action(&game(Platform::Manual, "x", "X")), None);
    }

    #[test]
    fn file_names_are_windows_safe() {
        assert_eq!(
            sanitize_file_name("Counter-Strike: Global Offensive"),
            "Counter-Strike Global Offensive"
        );
        assert_eq!(sanitize_file_name("Şahin'in Oyunu?"), "Şahin'in Oyunu");
        assert_eq!(sanitize_file_name("nul"), "Oyun");
        assert_eq!(sanitize_file_name("..."), "Oyun");
        assert_eq!(sanitize_file_name("Battlefield™ 1"), "Battlefield™ 1");
    }

    #[test]
    fn writes_url_shortcut_with_icon() {
        let tmp = tempfile::tempdir().unwrap();
        let ico = tmp.path().join("cs.ico");
        let path = create_shortcut_in(
            tmp.path(),
            &game(Platform::Steam, "730", "CS: 2"),
            Some(&ico),
        )
        .unwrap();
        assert_eq!(path.file_name().unwrap(), "CS 2.url");
        let text = std::fs::read_to_string(path).unwrap();
        assert!(text.contains("URL=steam://rungameid/730\r\n"));
        assert!(text.contains("IconFile="));
    }

    #[test]
    fn builds_ico_from_cover() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("cover.png");
        image::RgbaImage::from_pixel(60, 90, image::Rgba([200, 10, 10, 255]))
            .save(&src)
            .unwrap();
        let ico = build_icon(&src, &tmp.path().join("x.ico")).unwrap();
        let back = image::open(ico).unwrap();
        assert_eq!((back.width(), back.height()), (256, 256));
    }

    #[cfg(windows)]
    #[test]
    fn writes_real_lnk_for_manual_game_and_finds_desktop() {
        let tmp = tempfile::tempdir().unwrap();
        let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".into());
        let mut g = game(Platform::Manual, "x", "Şahin'in Oyunu");
        g.executable = Some(format!("{windir}\\notepad.exe"));
        g.install_dir = windir;
        let path = create_shortcut_in(tmp.path(), &g, None).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        // .lnk imzası: 0x4C ve Shell Link CLSID başlangıcı.
        assert_eq!(&bytes[..4], &[0x4C, 0, 0, 0]);
        assert!(desktop_dir().unwrap().is_dir());
    }
}
