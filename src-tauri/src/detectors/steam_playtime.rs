//! Steam'in yerelde tuttuğu oynama süreleri.
//!
//! `<steam>/userdata/<hesap>/config/localconfig.vdf` →
//! `UserLocalConfigStore/Software/Valve/Steam/apps/<appid>/{Playtime, Playtime2wks, LastPlayed}`.
//! Süreler dakikadır. Bilgisayarda birden fazla Steam hesabı varsa hesaplar ayrı oynadığı için toplanır.
//! Steam bu dosyayı oyun kapanınca günceller.
use std::collections::HashMap;
use std::path::Path;

use super::vdf::{self, Vdf};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SteamPlaytime {
    pub total_minutes: u64,
    pub two_weeks_minutes: u64,
    pub last_played: Option<i64>,
}

fn apps_node(doc: &Vdf) -> Option<&Vdf> {
    doc.get("UserLocalConfigStore")?
        .get("Software")?
        .get("Valve")?
        .get("Steam")?
        .get("apps")
}

/// Tek bir localconfig.vdf içeriğini ayrıştırır.
pub fn parse_localconfig(text: &str) -> HashMap<String, SteamPlaytime> {
    let mut out = HashMap::new();
    let Ok(doc) = vdf::parse(text) else {
        return out;
    };
    let Some(apps) = apps_node(&doc) else {
        return out;
    };
    for (appid, node) in apps.entries() {
        let minutes = |key: &str| {
            node.str(key)
                .and_then(|v| v.trim().parse::<u64>().ok())
                .unwrap_or(0)
        };
        let entry = SteamPlaytime {
            total_minutes: minutes("Playtime"),
            two_weeks_minutes: minutes("Playtime2wks"),
            last_played: node
                .str("LastPlayed")
                .and_then(|v| v.trim().parse::<i64>().ok())
                .filter(|t| *t > 0),
        };
        if entry.total_minutes > 0 || entry.last_played.is_some() {
            out.insert(appid.clone(), entry);
        }
    }
    out
}

/// Tüm Steam hesaplarının sürelerini birleştirir.
pub fn read_all(steam_root: &Path) -> HashMap<String, SteamPlaytime> {
    let mut merged: HashMap<String, SteamPlaytime> = HashMap::new();
    let Ok(users) = std::fs::read_dir(steam_root.join("userdata")) else {
        return merged;
    };
    for user in users.flatten() {
        let file = user.path().join("config").join("localconfig.vdf");
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        for (appid, p) in parse_localconfig(&text) {
            let e = merged.entry(appid).or_default();
            e.total_minutes += p.total_minutes;
            e.two_weeks_minutes += p.two_weeks_minutes;
            e.last_played = e.last_played.max(p.last_played);
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#""UserLocalConfigStore"
{
	"apptickets" { "730" "4a00" }
	"Software"
	{
		"Valve"
		{
			"Steam"
			{
				"apps"
				{
					"730"
					{
						"LastPlayed"		"1790880013"
						"Playtime"		"358753"
						"Playtime2wks"		"540"
						"cloud" { "last_sync_state" "synchronized" }
					}
					"304930"
					{
						"Playtime"		"120"
					}
					"999"
					{
						"cloud" { "x" "y" }
					}
				}
			}
		}
	}
}"#;

    #[test]
    fn parses_playtime_fields() {
        let map = parse_localconfig(SAMPLE);
        assert_eq!(
            map["730"],
            SteamPlaytime {
                total_minutes: 358_753,
                two_weeks_minutes: 540,
                last_played: Some(1_790_880_013)
            }
        );
        assert_eq!(map["304930"].total_minutes, 120);
        assert!(!map.contains_key("999"), "süresi olmayan uygulama atlanır");
        assert!(parse_localconfig("bozuk {").is_empty());
    }

    #[test]
    fn merges_multiple_accounts() {
        let tmp = tempfile::tempdir().unwrap();
        for user in ["111", "222"] {
            let dir = tmp.path().join("userdata").join(user).join("config");
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("localconfig.vdf"), SAMPLE).unwrap();
        }
        // Yapılandırma klasörü olmayan hesap sorun çıkarmaz.
        std::fs::create_dir_all(tmp.path().join("userdata/333")).unwrap();
        let map = read_all(tmp.path());
        assert_eq!(map["730"].total_minutes, 2 * 358_753);
        assert!(read_all(&tmp.path().join("yok")).is_empty());
    }
}
