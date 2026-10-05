//! Windows oyun/performans ayarları. Her ayar: geçerli durumu okunur, açılır/kapatılır, orijinal değeri
//! ilk değişiklikte saklanır ("Varsayılanlara dön").
//!
//! Güvenlik: yalnız aşağıdaki sabit listedeki kayıt defteri değerleri yazılır. Yönetici gerektirenler
//! (HKLM) uygulamanın kendisi `--admin-task <id> <on|off>` ile UAC onayıyla çalıştırılarak yazılır;
//! yardımcı görev de yalnız bu listeyi kabul eder.
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hive {
    CurrentUser,
    LocalMachine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegValue {
    Dword(u32),
    Text(&'static str),
}

/// Bir ayarın kayıt defterindeki tek bir değeri (açık/kapalı karşılıklarıyla).
#[derive(Debug, Clone, Copy)]
pub struct RegEntry {
    pub hive: Hive,
    pub path: &'static str,
    pub name: &'static str,
    pub on: RegValue,
    pub off: RegValue,
}

#[derive(Debug, Clone, Copy)]
pub enum Mechanism {
    Registry(&'static [RegEntry]),
    /// SystemParametersInfo ile anında uygulanır (Windows değeri kendisi kalıcılaştırır).
    MouseAcceleration,
    WindowAnimations,
}

#[derive(Debug, Clone, Copy)]
pub struct Tweak {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    /// Oyun performansı için önerilen durum.
    pub recommended: bool,
    pub requires_admin: bool,
    pub requires_restart: bool,
    pub mechanism: Mechanism,
}

const HKCU: Hive = Hive::CurrentUser;
const HKLM: Hive = Hive::LocalMachine;
const SYSTEM_PROFILE: &str =
    r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile";
const GAMES_TASK: &str =
    r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games";

pub const TWEAKS: &[Tweak] = &[
    Tweak {
        id: "game_mode",
        title: "Windows Oyun Modu",
        description: "Oyun açıkken Windows arka plan işlerini (güncelleme, sürücü kurulumu) erteler ve oyuna öncelik verir.",
        category: "Windows oyun ayarları",
        recommended: true,
        requires_admin: false,
        requires_restart: false,
        mechanism: Mechanism::Registry(&[
            RegEntry { hive: HKCU, path: r"Software\Microsoft\GameBar", name: "AutoGameModeEnabled", on: RegValue::Dword(1), off: RegValue::Dword(0) },
            RegEntry { hive: HKCU, path: r"Software\Microsoft\GameBar", name: "AllowAutoGameMode", on: RegValue::Dword(1), off: RegValue::Dword(0) },
        ]),
    },
    Tweak {
        id: "game_dvr",
        title: "Arka plan oyun kaydı (Game DVR)",
        description: "Oynarken arka planda sürekli video kaydı tutar. Kapatmak FPS'i artırır ve takılmaları azaltır.",
        category: "Windows oyun ayarları",
        recommended: false,
        requires_admin: false,
        requires_restart: false,
        mechanism: Mechanism::Registry(&[
            RegEntry { hive: HKCU, path: r"System\GameConfigStore", name: "GameDVR_Enabled", on: RegValue::Dword(1), off: RegValue::Dword(0) },
            RegEntry { hive: HKCU, path: r"Software\Microsoft\Windows\CurrentVersion\GameDVR", name: "AppCaptureEnabled", on: RegValue::Dword(1), off: RegValue::Dword(0) },
        ]),
    },
    Tweak {
        id: "game_bar",
        title: "Xbox Game Bar (Win+G)",
        description: "Oyun içi kaplama. Kullanmıyorsan kapatmak oyun açılışında ek yükü kaldırır.",
        category: "Windows oyun ayarları",
        recommended: false,
        requires_admin: false,
        requires_restart: false,
        mechanism: Mechanism::Registry(&[RegEntry {
            hive: HKCU,
            path: r"Software\Microsoft\GameBar",
            name: "UseNexusForGameBarEnabled",
            on: RegValue::Dword(1),
            off: RegValue::Dword(0),
        }]),
    },
    Tweak {
        id: "fullscreen_optimizations",
        title: "Tam ekran iyileştirmeleri",
        description: "Windows'un tam ekran oyunları pencereli gibi yönetmesi. Bazı eski oyunlarda kapatmak girdi gecikmesini azaltır.",
        category: "Windows oyun ayarları",
        recommended: true,
        requires_admin: false,
        requires_restart: false,
        mechanism: Mechanism::Registry(&[
            RegEntry { hive: HKCU, path: r"System\GameConfigStore", name: "GameDVR_FSEBehaviorMode", on: RegValue::Dword(0), off: RegValue::Dword(2) },
            RegEntry { hive: HKCU, path: r"System\GameConfigStore", name: "GameDVR_HonorUserFSEBehaviorMode", on: RegValue::Dword(0), off: RegValue::Dword(1) },
        ]),
    },
    Tweak {
        id: "mouse_acceleration",
        title: "Fare ivmesi (İşaretçi hassasiyetini artır)",
        description: "Fareyi hızlı hareket ettirince imleci fazladan hızlandırır. Nişan alma için kapalı olması önerilir; anında uygulanır.",
        category: "Girdi",
        recommended: false,
        requires_admin: false,
        requires_restart: false,
        mechanism: Mechanism::MouseAcceleration,
    },
    Tweak {
        id: "window_animations",
        title: "Pencere animasyonları",
        description: "Küçültme/büyütme animasyonları. Kapatmak alt-tab geçişlerini hızlandırır; anında uygulanır.",
        category: "Sistem",
        recommended: false,
        requires_admin: false,
        requires_restart: false,
        mechanism: Mechanism::WindowAnimations,
    },
    Tweak {
        id: "notifications",
        title: "Windows bildirimleri",
        description: "Ekranın köşesinde çıkan bildirim balonları. Yalnız oyun sırasında susturmak için oyun profilindeki \"Bildirimleri sustur\" seçeneğini kullan; oyun bitince geri açılır.",
        category: "Sistem",
        recommended: true,
        requires_admin: false,
        requires_restart: false,
        mechanism: Mechanism::Registry(&[RegEntry {
            hive: HKCU,
            path: r"Software\Microsoft\Windows\CurrentVersion\PushNotifications",
            name: "ToastEnabled",
            on: RegValue::Dword(1),
            off: RegValue::Dword(0),
        }]),
    },
    Tweak {
        id: "hags",
        title: "Donanım hızlandırmalı GPU zamanlaması",
        description: "GPU bellek yönetimini ekran kartına bırakır; destekleyen kartlarda gecikmeyi azaltır.",
        category: "Grafik",
        recommended: true,
        requires_admin: true,
        requires_restart: true,
        mechanism: Mechanism::Registry(&[RegEntry {
            hive: HKLM,
            path: r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers",
            name: "HwSchMode",
            on: RegValue::Dword(2),
            off: RegValue::Dword(1),
        }]),
    },
    Tweak {
        id: "network_throttling",
        title: "Ağ kısıtlamasını kaldır",
        description: "Windows'un multimedya sırasında ağ paketlerini sınırlamasını kapatır; çevrim içi oyunlarda ping dalgalanmasını azaltabilir.",
        category: "Ağ",
        recommended: true,
        requires_admin: true,
        requires_restart: true,
        mechanism: Mechanism::Registry(&[
            RegEntry { hive: HKLM, path: SYSTEM_PROFILE, name: "NetworkThrottlingIndex", on: RegValue::Dword(0xFFFF_FFFF), off: RegValue::Dword(10) },
            RegEntry { hive: HKLM, path: SYSTEM_PROFILE, name: "SystemResponsiveness", on: RegValue::Dword(10), off: RegValue::Dword(20) },
        ]),
    },
    Tweak {
        id: "games_task_priority",
        title: "Oyunlara yüksek zamanlama önceliği (MMCSS)",
        description: "Windows multimedya zamanlayıcısında oyun görevlerine yüksek CPU ve GPU önceliği verir.",
        category: "İşlemci",
        recommended: true,
        requires_admin: true,
        requires_restart: true,
        mechanism: Mechanism::Registry(&[
            RegEntry { hive: HKLM, path: GAMES_TASK, name: "GPU Priority", on: RegValue::Dword(8), off: RegValue::Dword(8) },
            RegEntry { hive: HKLM, path: GAMES_TASK, name: "Priority", on: RegValue::Dword(6), off: RegValue::Dword(2) },
            RegEntry { hive: HKLM, path: GAMES_TASK, name: "Scheduling Category", on: RegValue::Text("High"), off: RegValue::Text("Medium") },
        ]),
    },
];

pub fn find(id: &str) -> Option<&'static Tweak> {
    TWEAKS.iter().find(|t| t.id == id)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TweakState {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub recommended: bool,
    pub requires_admin: bool,
    pub requires_restart: bool,
    /// Geçerli durum; okunamadıysa `None`.
    pub enabled: Option<bool>,
}

#[derive(Debug, thiserror::Error)]
pub enum TweakError {
    #[error("kayıt defteri hatası: {0}")]
    Registry(String),
    #[error("Windows API hatası")]
    Api,
    #[error("Yönetici izni verilmedi")]
    ElevationDenied,
}

// ───────────── Kayıt defteri erişimi ─────────────

#[cfg(windows)]
mod reg {
    use super::{Hive, RegEntry, RegValue, TweakError};
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE};
    use winreg::RegKey;

    fn root(h: Hive) -> RegKey {
        RegKey::predef(match h {
            Hive::CurrentUser => HKEY_CURRENT_USER,
            Hive::LocalMachine => HKEY_LOCAL_MACHINE,
        })
    }

    pub fn read(e: &RegEntry) -> Option<RegValue> {
        let key = root(e.hive).open_subkey_with_flags(e.path, KEY_READ).ok()?;
        match e.on {
            RegValue::Dword(_) => key.get_value::<u32, _>(e.name).ok().map(RegValue::Dword),
            RegValue::Text(_) => key.get_value::<String, _>(e.name).ok().map(|s| {
                // Sabit metinlerden biriyle eşleşiyorsa onu döndür (karşılaştırma için).
                if s.eq_ignore_ascii_case(text_of(e.on)) {
                    e.on
                } else if s.eq_ignore_ascii_case(text_of(e.off)) {
                    e.off
                } else {
                    RegValue::Text("?")
                }
            }),
        }
    }

    fn text_of(v: RegValue) -> &'static str {
        match v {
            RegValue::Text(t) => t,
            RegValue::Dword(_) => "",
        }
    }

    pub fn write(e: &RegEntry, v: RegValue) -> Result<(), TweakError> {
        let (key, _) = root(e.hive)
            .create_subkey_with_flags(e.path, KEY_WRITE)
            .map_err(|err| TweakError::Registry(err.to_string()))?;
        match v {
            RegValue::Dword(d) => key.set_value(e.name, &d),
            RegValue::Text(t) => key.set_value(e.name, &t.to_owned()),
        }
        .map_err(|err| TweakError::Registry(err.to_string()))
    }
}

#[cfg(windows)]
mod spi {
    use windows::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, ANIMATIONINFO, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE,
        SPI_GETANIMATION, SPI_GETMOUSE, SPI_SETANIMATION, SPI_SETMOUSE,
    };

    /// [eşik1, eşik2, hız]; hız 0 ise ivme kapalı.
    pub fn mouse_acceleration() -> Option<bool> {
        let mut v = [0i32; 3];
        // SAFETY: SPI_GETMOUSE 3 elemanlı int dizisi bekler.
        unsafe {
            SystemParametersInfoW(
                SPI_GETMOUSE,
                0,
                Some(v.as_mut_ptr().cast()),
                Default::default(),
            )
        }
        .ok()?;
        Some(v[2] != 0)
    }

    pub fn set_mouse_acceleration(on: bool) -> bool {
        // Windows varsayılanı: 6, 10, 1 (açık); kapalı: 0, 0, 0.
        let mut v: [i32; 3] = if on { [6, 10, 1] } else { [0, 0, 0] };
        // SAFETY: 3 elemanlı dizi; değer kullanıcı profiline yazılır ve tüm pencerelere duyurulur.
        unsafe {
            SystemParametersInfoW(
                SPI_SETMOUSE,
                0,
                Some(v.as_mut_ptr().cast()),
                SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
            )
        }
        .is_ok()
    }

    fn info() -> ANIMATIONINFO {
        ANIMATIONINFO {
            cbSize: u32::try_from(std::mem::size_of::<ANIMATIONINFO>()).unwrap_or(8),
            iMinAnimate: 0,
        }
    }

    pub fn window_animations() -> Option<bool> {
        let mut a = info();
        // SAFETY: cbSize doğru ayarlanmış ANIMATIONINFO.
        unsafe {
            SystemParametersInfoW(
                SPI_GETANIMATION,
                a.cbSize,
                Some((&raw mut a).cast()),
                Default::default(),
            )
        }
        .ok()?;
        Some(a.iMinAnimate != 0)
    }

    pub fn set_window_animations(on: bool) -> bool {
        let mut a = info();
        a.iMinAnimate = i32::from(on);
        // SAFETY: cbSize doğru ayarlanmış ANIMATIONINFO.
        unsafe {
            SystemParametersInfoW(
                SPI_SETANIMATION,
                a.cbSize,
                Some((&raw mut a).cast()),
                SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
            )
        }
        .is_ok()
    }
}

/// Ayarın geçerli durumu. Kayıt defterinde değer yoksa Windows varsayılanı kabul edilir:
/// tüm girdiler "açık" değerindeyse açık, hepsi "kapalı" değerindeyse kapalı, karışıksa ilk girdiye göre.
#[cfg(windows)]
pub fn current(t: &Tweak) -> Option<bool> {
    match t.mechanism {
        Mechanism::MouseAcceleration => spi::mouse_acceleration(),
        Mechanism::WindowAnimations => spi::window_animations(),
        Mechanism::Registry(entries) => {
            let first = entries.first()?;
            match reg::read(first) {
                Some(v) if v == first.on => Some(true),
                Some(v) if v == first.off => Some(false),
                Some(_) => Some(true),
                // Değer hiç yoksa Windows varsayılanı: bu ayarların hepsinde varsayılan "açık".
                None => Some(default_on(t)),
            }
        }
    }
}

#[cfg(not(windows))]
pub fn current(_t: &Tweak) -> Option<bool> {
    None
}

/// Kayıt defterinde değer yokken Windows'un davranışı.
fn default_on(t: &Tweak) -> bool {
    // network_throttling ve games_task_priority "iyileştirme" ayarlarıdır: varsayılan kapalı.
    !matches!(t.id, "network_throttling" | "games_task_priority")
}

/// Yönetici gerektirmeyen ayarı uygular. Yönetici gerektirenler için `apply_elevated`.
#[cfg(windows)]
pub fn apply_direct(t: &Tweak, on: bool) -> Result<(), TweakError> {
    match t.mechanism {
        Mechanism::MouseAcceleration => spi::set_mouse_acceleration(on)
            .then_some(())
            .ok_or(TweakError::Api),
        Mechanism::WindowAnimations => spi::set_window_animations(on)
            .then_some(())
            .ok_or(TweakError::Api),
        Mechanism::Registry(entries) => {
            for e in entries {
                reg::write(e, if on { e.on } else { e.off })?;
            }
            Ok(())
        }
    }
}

#[cfg(not(windows))]
pub fn apply_direct(_t: &Tweak, _on: bool) -> Result<(), TweakError> {
    Err(TweakError::Api)
}

/// Uygulamayı UAC onayıyla yönetici olarak `--admin-task <id> <on|off>` kipinde çalıştırır ve bekler.
#[cfg(windows)]
pub fn apply_elevated(t: &Tweak, on: bool) -> Result<(), TweakError> {
    let exe = std::env::current_exe().map_err(|e| TweakError::Registry(e.to_string()))?;
    let args = format!("--admin-task {} {}", t.id, if on { "on" } else { "off" });
    let code =
        crate::elevate::run_as_admin(&exe, &args).map_err(|_| TweakError::ElevationDenied)?;
    if code == 0 {
        Ok(())
    } else {
        Err(TweakError::Registry(format!(
            "yardımcı görev hata kodu {code}"
        )))
    }
}

#[cfg(not(windows))]
pub fn apply_elevated(_t: &Tweak, _on: bool) -> Result<(), TweakError> {
    Err(TweakError::Api)
}

pub fn state(t: &Tweak) -> TweakState {
    TweakState {
        id: t.id,
        title: t.title,
        description: t.description,
        category: t.category,
        recommended: t.recommended,
        requires_admin: t.requires_admin,
        requires_restart: t.requires_restart,
        enabled: current(t),
    }
}

/// `--admin-task` kipi: yalnız listedeki ve yönetici gerektiren ayarlar. Dönüş: süreç çıkış kodu.
pub fn run_admin_task(args: &[String]) -> i32 {
    let (Some(id), Some(mode)) = (args.first(), args.get(1)) else {
        return 2;
    };
    // Ayar dışı tek yönetici görevi: bekleme belleği temizliği.
    if id == "purge-standby" {
        return if crate::perf::memory::purge_standby_list() {
            0
        } else {
            6
        };
    }
    let Some(t) = find(id).filter(|t| t.requires_admin) else {
        return 3;
    };
    let on = match mode.as_str() {
        "on" => true,
        "off" => false,
        _ => return 4,
    };
    match apply_direct(t, on) {
        Ok(()) => 0,
        Err(_) => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tweak_ids_are_unique_and_admin_flags_match_hive() {
        let mut ids: Vec<_> = TWEAKS.iter().map(|t| t.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), TWEAKS.len());
        for t in TWEAKS {
            if let Mechanism::Registry(entries) = t.mechanism {
                let any_hklm = entries.iter().any(|e| e.hive == Hive::LocalMachine);
                assert_eq!(any_hklm, t.requires_admin, "{}", t.id);
            }
        }
    }

    #[test]
    fn admin_task_rejects_anything_outside_whitelist() {
        let s = |v: &[&str]| v.iter().map(|x| (*x).to_owned()).collect::<Vec<_>>();
        assert_eq!(run_admin_task(&s(&[])), 2);
        assert_eq!(run_admin_task(&s(&["bilinmeyen", "on"])), 3);
        // Yönetici gerektirmeyen ayar yardımcı görevle yazılamaz.
        assert_eq!(run_admin_task(&s(&["game_dvr", "off"])), 3);
        assert_eq!(run_admin_task(&s(&["hags", "belki"])), 4);
    }

    #[cfg(windows)]
    #[test]
    fn reads_real_system_state_for_every_tweak() {
        for t in TWEAKS {
            assert!(current(t).is_some(), "{} okunamadı", t.id);
        }
    }
}

#[cfg(all(test, windows))]
mod real_system {
    use super::*;

    /// Elle: yönetici gerektirmeyen her ayarı ters çevirir, doğrular ve ORİJİNAL değere geri döndürür.
    /// `cargo test tweaks::real_system -- --ignored --nocapture`
    #[test]
    #[ignore = "gerçek Windows ayarlarını geçici olarak değiştirir (sonunda geri alır)"]
    fn toggle_and_restore_every_user_tweak() {
        for t in TWEAKS.iter().filter(|t| !t.requires_admin) {
            let original = current(t).expect("okunamadı");
            apply_direct(t, !original).expect("uygulanamadı");
            let flipped = current(t).expect("tekrar okunamadı");
            apply_direct(t, original).expect("geri alınamadı");
            let restored = current(t).expect("son okuma");
            println!(
                "{:<26} orijinal={original:<5} çevrildi={flipped:<5} geri={restored}",
                t.id
            );
            assert_eq!(flipped, !original, "{} değişmedi", t.id);
            assert_eq!(restored, original, "{} geri alınamadı", t.id);
        }
    }
}
