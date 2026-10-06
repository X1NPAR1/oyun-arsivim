//! Performans Merkezi IPC komutları.
use std::collections::BTreeMap;

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use tauri::State;

use super::boost::{self, PerfProfile};
use super::memory::{self, CleanReport};
use super::monitor::{PerfMonitor, PerfSnapshot};
use super::power::{self, PowerOverview};
use super::processes;
use super::temp;
use super::tweaks::{self, TweakState};
use crate::commands::SharedState;

type Res<T> = Result<T, String>;

fn db_err(e: impl std::fmt::Display) -> String {
    format!("Veritabanı hatası: {e}")
}

const ORIGINALS_KEY: &str = "tweak_originals";

/// Kullanıcının ilk değiştirdiği andaki ayar değerleri ("Varsayılanlara dön" için).
fn load_originals(conn: &Connection) -> BTreeMap<String, bool> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        [ORIGINALS_KEY],
        |r| r.get::<_, String>(0),
    )
    .optional()
    .ok()
    .flatten()
    .and_then(|j| serde_json::from_str(&j).ok())
    .unwrap_or_default()
}

fn save_originals(conn: &Connection, map: &BTreeMap<String, bool>) -> rusqlite::Result<()> {
    let json = serde_json::to_string(map).unwrap_or_else(|_| "{}".into());
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![ORIGINALS_KEY, json],
    )?;
    Ok(())
}

#[tauri::command]
pub fn perf_snapshot(monitor: State<'_, PerfMonitor>) -> PerfSnapshot {
    monitor.snapshot(8)
}

#[tauri::command]
pub fn perf_tweaks() -> Vec<TweakState> {
    tweaks::TWEAKS.iter().map(tweaks::state).collect()
}

/// Ayarı değiştirir; yönetici gerekiyorsa UAC onayı istenir. Dönüş: yeni durum.
#[tauri::command]
pub async fn perf_set_tweak(
    state: State<'_, SharedState>,
    id: String,
    enabled: bool,
) -> Res<TweakState> {
    let t = tweaks::find(&id).ok_or("Bilinmeyen ayar")?;
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let original = tweaks::current(t);
        if let (Some(orig), Ok(conn)) = (original, state.db.lock()) {
            let mut map = load_originals(&conn);
            map.entry(t.id.to_owned()).or_insert(orig);
            save_originals(&conn, &map).map_err(db_err)?;
        }
        let result = if t.requires_admin {
            tweaks::apply_elevated(t, enabled)
        } else {
            tweaks::apply_direct(t, enabled)
        };
        result.map_err(|e| e.to_string())?;
        tracing::info!(id = t.id, enabled, "performans ayarı değiştirildi");
        Ok(tweaks::state(t))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreReport {
    pub restored: Vec<String>,
    pub failed: Vec<String>,
}

/// Değiştirilen tüm ayarları ilk değerlerine döndürür (yönetici ayarları için tek UAC sorusu).
#[tauri::command]
pub async fn perf_restore_defaults(state: State<'_, SharedState>) -> Res<RestoreReport> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let originals = {
            let conn = state.db.lock().map_err(db_err)?;
            load_originals(&conn)
        };
        let mut report = RestoreReport {
            restored: vec![],
            failed: vec![],
        };
        for (id, value) in &originals {
            let Some(t) = tweaks::find(id) else { continue };
            if tweaks::current(t) == Some(*value) {
                report.restored.push(t.title.to_owned());
                continue;
            }
            let r = if t.requires_admin {
                tweaks::apply_elevated(t, *value)
            } else {
                tweaks::apply_direct(t, *value)
            };
            match r {
                Ok(()) => report.restored.push(t.title.to_owned()),
                Err(_) => report.failed.push(t.title.to_owned()),
            }
        }
        if report.failed.is_empty() {
            let conn = state.db.lock().map_err(db_err)?;
            save_originals(&conn, &BTreeMap::new()).map_err(db_err)?;
        }
        Ok(report)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn perf_clean_ram() -> Res<CleanReport> {
    tauri::async_runtime::spawn_blocking(memory::trim_working_sets)
        .await
        .map_err(|e| e.to_string())
}

/// Bekleme listesi temizliği (yönetici, UAC onayıyla).
/// Silinebilecek geçici dosyaların boyutu.
#[tauri::command]
pub async fn perf_temp_usage() -> Res<temp::TempUsage> {
    tauri::async_runtime::spawn_blocking(temp::usage)
        .await
        .map_err(|e| e.to_string())
}

/// %TEMP% ve WindowsTemp temizliği (kilitli/yeni dosyalar atlanır).
#[tauri::command]
pub async fn perf_clean_temp() -> Res<temp::TempCleanReport> {
    tauri::async_runtime::spawn_blocking(temp::clean)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn perf_purge_standby() -> Res<()> {
    tauri::async_runtime::spawn_blocking(|| {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        match crate::elevate::run_as_admin(&exe, "--admin-task purge-standby on") {
            Ok(0) => Ok(()),
            Ok(code) => Err(format!("Bekleme belleği temizlenemedi (kod {code})")),
            Err(_) => Err("Yönetici izni verilmedi".to_owned()),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn power_overview() -> PowerOverview {
    power::overview(power::list())
}

#[tauri::command]
pub fn power_set(guid: String) -> Res<PowerOverview> {
    if !power::set_active(&guid) {
        return Err("Güç planı değiştirilemedi".into());
    }
    Ok(power::overview(power::list()))
}

#[tauri::command]
pub fn perf_profile_get(state: State<'_, SharedState>, game_id: i64) -> Res<PerfProfile> {
    let conn = state.db.lock().map_err(db_err)?;
    boost::load_profile(&conn, game_id).map_err(db_err)
}

#[tauri::command]
pub fn perf_profile_set(
    state: State<'_, SharedState>,
    game_id: i64,
    profile: PerfProfile,
) -> Res<PerfProfile> {
    let conn = state.db.lock().map_err(db_err)?;
    boost::save_profile(&conn, game_id, &profile).map_err(db_err)?;
    Ok(profile)
}

/// Profil etkin oyunların kimlikleri (kütüphanede rozet göstermek için).
/// "Performans modunu aç" için önerilen profil.
#[tauri::command]
pub fn perf_profile_recommended() -> PerfProfile {
    PerfProfile::recommended()
}

#[tauri::command]
pub fn perf_profiled_games(state: State<'_, SharedState>) -> Res<Vec<i64>> {
    let conn = state.db.lock().map_err(db_err)?;
    let mut stmt = conn
        .prepare("SELECT game_id, profile FROM perf_profiles")
        .map_err(db_err)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
        .map_err(db_err)?;
    Ok(rows
        .flatten()
        .filter(|(_, j)| serde_json::from_str::<PerfProfile>(j).is_ok_and(|p| p.enabled))
        .map(|(id, _)| id)
        .collect())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSuggestion {
    pub exe: String,
    pub title: String,
    pub running: bool,
}

#[tauri::command]
pub fn perf_close_suggestions(monitor: State<'_, PerfMonitor>) -> Vec<AppSuggestion> {
    let running: Vec<String> = monitor
        .snapshot(400)
        .top_processes
        .into_iter()
        .map(|p| p.name.to_lowercase())
        .collect();
    processes::SUGGESTIONS
        .iter()
        .map(|(exe, title)| AppSuggestion {
            exe: (*exe).to_owned(),
            title: (*title).to_owned(),
            running: running.contains(&exe.to_lowercase()),
        })
        .collect()
}
