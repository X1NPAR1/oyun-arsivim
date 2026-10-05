//! Kayıt defteri okuma soyutlaması: dedektörler gerçek kayıt defterine bağımlı olmadan test edilir.
//! Yollar `HKLM\...` / `HKCU\...` biçimindedir.
#[cfg(any(test, not(windows)))]
use std::collections::BTreeMap;

pub trait RegReader: Send + Sync {
    /// Anahtarın alt anahtar adları (yoksa boş).
    fn subkeys(&self, path: &str) -> Vec<String>;
    /// Metin veya sayı değeri metin olarak (yoksa `None`).
    fn value(&self, path: &str, name: &str) -> Option<String>;
}

#[cfg(windows)]
pub struct SystemRegistry;

#[cfg(windows)]
impl SystemRegistry {
    fn open(path: &str) -> Option<winreg::RegKey> {
        use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
        let (hive, rest) = path.split_once('\\')?;
        let root = match hive {
            "HKLM" => winreg::RegKey::predef(HKEY_LOCAL_MACHINE),
            "HKCU" => winreg::RegKey::predef(HKEY_CURRENT_USER),
            _ => return None,
        };
        root.open_subkey(rest).ok()
    }
}

#[cfg(windows)]
impl RegReader for SystemRegistry {
    fn subkeys(&self, path: &str) -> Vec<String> {
        Self::open(path)
            .map(|k| k.enum_keys().flatten().collect())
            .unwrap_or_default()
    }

    fn value(&self, path: &str, name: &str) -> Option<String> {
        let key = Self::open(path)?;
        key.get_value::<String, _>(name)
            .ok()
            .or_else(|| key.get_value::<u32, _>(name).ok().map(|v| v.to_string()))
            .map(|s| s.trim_end_matches('\0').to_owned())
    }
}

/// Testler için bellek içi kayıt defteri.
#[cfg(any(test, not(windows)))]
#[derive(Default)]
pub struct FakeRegistry {
    keys: BTreeMap<String, BTreeMap<String, String>>,
}

#[cfg(any(test, not(windows)))]
impl FakeRegistry {
    pub fn with(mut self, path: &str, values: &[(&str, &str)]) -> Self {
        let entry = self.keys.entry(path.to_lowercase()).or_default();
        for (k, v) in values {
            entry.insert(k.to_lowercase(), (*v).to_owned());
        }
        self
    }
}

#[cfg(any(test, not(windows)))]
impl RegReader for FakeRegistry {
    fn subkeys(&self, path: &str) -> Vec<String> {
        let prefix = format!("{}\\", path.to_lowercase());
        let mut out: Vec<String> = self
            .keys
            .keys()
            .filter_map(|k| k.strip_prefix(&prefix))
            .map(|rest| rest.split('\\').next().unwrap_or(rest).to_owned())
            .collect();
        out.dedup();
        out
    }

    fn value(&self, path: &str, name: &str) -> Option<String> {
        self.keys
            .get(&path.to_lowercase())?
            .get(&name.to_lowercase())
            .cloned()
    }
}

/// Üretimde kullanılan okuyucu.
pub fn system() -> Box<dyn RegReader> {
    #[cfg(windows)]
    {
        Box::new(SystemRegistry)
    }
    #[cfg(not(windows))]
    {
        Box::new(FakeRegistry::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_registry_lists_direct_children_case_insensitively() {
        let r = FakeRegistry::default()
            .with(r"HKLM\SOFTWARE\GOG.com\Games\1", &[("gameName", "A")])
            .with(r"HKLM\SOFTWARE\GOG.com\Games\2\Sub", &[("x", "y")]);
        assert_eq!(r.subkeys(r"HKLM\software\gog.com\games"), vec!["1", "2"]);
        assert_eq!(
            r.value(r"HKLM\SOFTWARE\GOG.com\Games\1", "GAMENAME")
                .as_deref(),
            Some("A")
        );
        assert!(r.value(r"HKLM\x", "y").is_none());
    }

    #[cfg(windows)]
    #[test]
    fn reads_real_registry() {
        let r = SystemRegistry;
        assert!(!r.subkeys(r"HKLM\SOFTWARE\Microsoft").is_empty());
        assert!(r
            .value(
                r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion",
                "CurrentBuildNumber"
            )
            .is_some());
    }
}
