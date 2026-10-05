//! Pencere arka plan efekti. Windows 11'de Mica, diğerlerinde arayüzün CSS yedeği.
//!
//! Windows 10'da Acrylic de mümkün, ancak pencere sürüklenip yeniden boyutlandırılırken bilinen
//! ciddi takılmalara yol açtığı için bilerek kullanılmıyor (bkz. MEMORY.md).
use serde::Serialize;
use tauri::window::{Effect, EffectsBuilder};
use tauri::WebviewWindow;

/// Windows 11'in ilk derlemesi; Mica bu sürümden itibaren desteklenir.
const FIRST_WIN11_BUILD: u32 = 22_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Backdrop {
    Mica,
    None,
}

pub fn supports_mica(build: u32) -> bool {
    build >= FIRST_WIN11_BUILD
}

#[cfg(windows)]
fn windows_build() -> Option<u32> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    let key = winreg::RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
        .ok()?;
    let build: String = key.get_value("CurrentBuildNumber").ok()?;
    build.trim().parse().ok()
}

#[cfg(not(windows))]
fn windows_build() -> Option<u32> {
    None
}

/// Efekti uygular ve hangi arka planın etkin olduğunu döndürür. Hata uygulamayı durdurmaz:
/// efekt uygulanamazsa CSS yedeği devreye girer.
pub fn apply(window: &WebviewWindow) -> Backdrop {
    let build = windows_build().unwrap_or(0);
    if !supports_mica(build) {
        // Win10 köşeleri yuvarlatmaz; OS gölgesi kabuğun yuvarlak köşelerinin arkasında dikdörtgen
        // bir çerçeve çizerdi. Gölgeyi kapatıp köşeleri arayüz çizer.
        if let Err(err) = window.set_shadow(false) {
            eprintln!("Pencere gölgesi kapatılamadı: {err}");
        }
        return Backdrop::None;
    }
    match window.set_effects(EffectsBuilder::new().effect(Effect::Mica).build()) {
        Ok(()) => Backdrop::Mica,
        Err(err) => {
            eprintln!("Mica uygulanamadı, CSS yedeği kullanılacak: {err}");
            Backdrop::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mica_only_on_windows_11() {
        assert!(!supports_mica(19_045)); // Windows 10 22H2
        assert!(supports_mica(22_000));
        assert!(supports_mica(26_100));
        assert!(!supports_mica(0));
    }

    #[test]
    fn backdrop_serializes_lowercase() {
        assert_eq!(serde_json::to_string(&Backdrop::Mica).unwrap(), "\"mica\"");
        assert_eq!(serde_json::to_string(&Backdrop::None).unwrap(), "\"none\"");
    }

    #[cfg(windows)]
    #[test]
    fn reads_real_windows_build() {
        assert!(windows_build().unwrap_or(0) > 10_000);
    }
}
