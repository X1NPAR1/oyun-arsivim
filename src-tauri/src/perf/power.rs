//! Güç planları (Windows Power API — `powercfg` metni yerine; Türkçe plan adları konsolda bozuluyordu).
//!
//! Kural: hiçbir zaman yeni plan OLUŞTURULMAZ/kopyalanmaz. Var olan bir plan etkinleştirilir; oyun
//! bitince önceki plana dönülür. (Bu makinede başka araçların bıraktığı 563 kopya plan vardı.)
use serde::Serialize;

/// Windows'un yerleşik plan kimlikleri.
#[cfg(test)]
pub const BALANCED: &str = "381b4222-f694-41f0-9685-ff5bb260df2e";
pub const HIGH_PERFORMANCE: &str = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";
pub const ULTIMATE_PERFORMANCE: &str = "e9a42b02-d5df-448d-aa00-03f14749eb61";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerScheme {
    pub guid: String,
    pub name: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerOverview {
    /// Aynı adlı kopyalar tekilleştirilmiş liste (etkin olan önce gelir).
    pub schemes: Vec<PowerScheme>,
    pub active: Option<PowerScheme>,
    /// Aynı ada sahip fazladan plan sayısı (başka araçların bıraktığı kopyalar).
    pub duplicate_count: usize,
    /// Oyun için önerilen plan (Nihai → Yüksek performans).
    pub recommended: Option<PowerScheme>,
}

/// Aynı ada sahip planları tekilleştirir; etkin olanı veya ilkini tutar.
pub fn overview(all: Vec<PowerScheme>) -> PowerOverview {
    let total = all.len();
    let mut unique: Vec<PowerScheme> = Vec::new();
    for s in all {
        match unique
            .iter_mut()
            .find(|u| u.name.eq_ignore_ascii_case(&s.name))
        {
            Some(existing) => {
                if s.active {
                    *existing = s;
                }
            }
            None => unique.push(s),
        }
    }
    unique.sort_by_key(|s| !s.active);
    let active = unique.iter().find(|s| s.active).cloned();
    let recommended = pick_game_scheme(&unique);
    PowerOverview {
        duplicate_count: total - unique.len(),
        schemes: unique,
        active,
        recommended,
    }
}

/// Oyun için en iyi mevcut plan: Nihai Performans (yerleşik GUID veya adı) → Yüksek performans.
pub fn pick_game_scheme(schemes: &[PowerScheme]) -> Option<PowerScheme> {
    let by_guid = |g: &str| schemes.iter().find(|s| s.guid.eq_ignore_ascii_case(g));
    let by_name = |words: &[&str]| {
        schemes
            .iter()
            .find(|s| words.iter().any(|w| s.name.to_lowercase().contains(w)))
    };
    by_guid(ULTIMATE_PERFORMANCE)
        .or_else(|| by_name(&["nihai", "ultimate"]))
        .or_else(|| by_guid(HIGH_PERFORMANCE))
        .or_else(|| by_name(&["yüksek performans", "high performance"]))
        .cloned()
}

#[cfg(windows)]
mod platform {
    use super::PowerScheme;
    use windows::core::GUID;
    use windows::Win32::Foundation::{LocalFree, HLOCAL, WIN32_ERROR};
    use windows::Win32::System::Power::{
        PowerEnumerate, PowerGetActiveScheme, PowerReadFriendlyName, PowerSetActiveScheme,
        ACCESS_SCHEME,
    };

    fn guid_to_string(g: &GUID) -> String {
        format!("{g:?}")
            .trim_matches(|c| c == '{' || c == '}')
            .to_lowercase()
    }

    pub fn parse_guid(s: &str) -> Option<GUID> {
        let hex: String = s.chars().filter(char::is_ascii_hexdigit).collect();
        if hex.len() != 32 {
            return None;
        }
        u128::from_str_radix(&hex, 16).ok().map(GUID::from_u128)
    }

    fn friendly_name(g: &GUID) -> String {
        let mut size = 0u32;
        // SAFETY: önce boyut sorulur, sonra UTF-16 arabelleğe okunur.
        unsafe {
            if PowerReadFriendlyName(None, Some(g), None, None, None, &mut size) != WIN32_ERROR(0)
                || size == 0
            {
                return String::new();
            }
            let mut buf = vec![0u8; size as usize];
            if PowerReadFriendlyName(None, Some(g), None, None, Some(buf.as_mut_ptr()), &mut size)
                != WIN32_ERROR(0)
            {
                return String::new();
            }
            let wide: Vec<u16> = buf
                .as_chunks::<2>()
                .0
                .iter()
                .map(|c| u16::from_le_bytes(*c))
                .collect();
            String::from_utf16_lossy(&wide)
                .trim_end_matches('\0')
                .to_owned()
        }
    }

    pub fn active_guid() -> Option<String> {
        // SAFETY: dönen GUID işaretçisi LocalFree ile serbest bırakılır.
        unsafe {
            let mut p: *mut GUID = std::ptr::null_mut();
            if PowerGetActiveScheme(None, &mut p) != WIN32_ERROR(0) || p.is_null() {
                return None;
            }
            let s = guid_to_string(&*p);
            let _ = LocalFree(Some(HLOCAL(p.cast())));
            Some(s)
        }
    }

    pub fn list() -> Vec<PowerScheme> {
        let active = active_guid();
        let mut out = Vec::new();
        for index in 0.. {
            let mut g = GUID::zeroed();
            let mut size = u32::try_from(std::mem::size_of::<GUID>()).unwrap_or(16);
            // SAFETY: GUID boyutunda arabellek; liste sonunda hata döner ve döngü biter.
            let rc = unsafe {
                PowerEnumerate(
                    None,
                    None,
                    None,
                    ACCESS_SCHEME,
                    index,
                    Some((&raw mut g).cast()),
                    &mut size,
                )
            };
            if rc != WIN32_ERROR(0) {
                break;
            }
            let guid = guid_to_string(&g);
            out.push(PowerScheme {
                active: active.as_deref() == Some(guid.as_str()),
                name: friendly_name(&g),
                guid,
            });
        }
        out
    }

    pub fn set_active(guid: &str) -> bool {
        let Some(g) = parse_guid(guid) else {
            return false;
        };
        // SAFETY: geçerli GUID; kullanıcı düzeyinde izin verilen işlem.
        unsafe { PowerSetActiveScheme(None, Some(&g)) == WIN32_ERROR(0) }
    }
}

#[cfg(not(windows))]
mod platform {
    use super::PowerScheme;
    pub fn list() -> Vec<PowerScheme> {
        Vec::new()
    }
    pub fn active_guid() -> Option<String> {
        None
    }
    pub fn set_active(_guid: &str) -> bool {
        false
    }
}

pub fn list() -> Vec<PowerScheme> {
    platform::list()
}

pub fn active_guid() -> Option<String> {
    platform::active_guid()
}

pub fn set_active(guid: &str) -> bool {
    platform::set_active(guid)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(guid: &str, name: &str, active: bool) -> PowerScheme {
        PowerScheme {
            guid: guid.into(),
            name: name.into(),
            active,
        }
    }

    #[test]
    fn dedupes_copies_and_picks_game_scheme() {
        let o = overview(vec![
            s(BALANCED, "Dengeli", false),
            s(
                "aaaa0000-0000-0000-0000-000000000001",
                "Nihai Performans",
                false,
            ),
            s(
                "aaaa0000-0000-0000-0000-000000000002",
                "Nihai Performans",
                false,
            ),
            s(HIGH_PERFORMANCE, "Yüksek performans", true),
        ]);
        assert_eq!(o.duplicate_count, 1);
        assert_eq!(o.schemes.len(), 3);
        assert_eq!(o.schemes[0].guid, HIGH_PERFORMANCE, "etkin plan önce");
        assert_eq!(o.recommended.unwrap().name, "Nihai Performans");
    }

    #[test]
    fn falls_back_to_high_performance() {
        let pick = pick_game_scheme(&[
            s(BALANCED, "Balanced", true),
            s(HIGH_PERFORMANCE, "High performance", false),
        ]);
        assert_eq!(pick.unwrap().guid, HIGH_PERFORMANCE);
        assert!(pick_game_scheme(&[s(BALANCED, "Dengeli", true)]).is_none());
    }

    #[cfg(windows)]
    #[test]
    fn reads_real_power_schemes_with_turkish_names() {
        let all = list();
        assert!(!all.is_empty());
        assert_eq!(all.iter().filter(|s| s.active).count(), 1);
        assert!(all.iter().all(|s| !s.name.is_empty()));
        assert!(active_guid().is_some());
    }

    #[cfg(windows)]
    #[test]
    fn guid_roundtrip() {
        let g = platform::parse_guid(HIGH_PERFORMANCE).unwrap();
        assert_eq!(
            format!("{g:?}").trim_matches(['{', '}']).to_lowercase(),
            HIGH_PERFORMANCE
        );
    }
}

#[cfg(all(test, windows))]
mod real_system {
    #[test]
    #[ignore = "gerçek güç planı özetini ve RAM temizliğini yazdırır"]
    fn print_overview_and_clean() {
        let o = super::overview(super::list());
        println!(
            "planlar (tekil): {:?}",
            o.schemes
                .iter()
                .map(|s| (&s.name, s.active))
                .collect::<Vec<_>>()
        );
        println!(
            "kopya sayısı: {} · önerilen: {:?}",
            o.duplicate_count,
            o.recommended.map(|s| s.name)
        );
        let r = crate::perf::memory::trim_working_sets();
        println!(
            "RAM temizliği: {} süreç, {} MB çalışma kümesi boşaltıldı, kullanılabilir {} → {} MB",
            r.processes,
            r.freed_bytes / 1_048_576,
            r.available_before / 1_048_576,
            r.available_after / 1_048_576
        );
    }
}
