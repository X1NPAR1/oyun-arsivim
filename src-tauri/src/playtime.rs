//! Oynama süresi verisi: platformdan içe aktarılan süreler, oturumlar ve saatlik (aktif / AFK) kovalar.
//!
//! Saatlik kovalar UTC saat numarasıyla (`unix / 3600`) tutulur; yerel saat dilimine göre gün/saat
//! dağılımı arayüzde hesaplanır (yaz saati dahil doğru sonuç için).
use std::collections::HashMap;

use rusqlite::{params, Connection};
use serde::Serialize;

use crate::detectors::steam_playtime::SteamPlaytime;

const HOUR: i64 = 3600;
const WEEK_HOURS: i64 = 24 * 7;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaytimeSummary {
    /// Gösterilecek toplam: platform süresi ile kendi ölçümümüzün büyüğü.
    pub total_seconds: i64,
    /// Oyun Arşivim'in ölçtüğü toplam süre (aktif + AFK).
    pub tracked_seconds: i64,
    pub idle_seconds: i64,
    /// Son 7 gün (kendi ölçümümüz).
    pub week_seconds: i64,
    /// Platformun bildirdiği toplam (Steam), dakika.
    pub platform_minutes: Option<i64>,
    /// Platformun bildirdiği son 2 hafta (Steam), dakika.
    pub platform_two_weeks_minutes: Option<i64>,
    pub sessions: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HourRow {
    pub game_id: i64,
    pub hour: i64,
    pub active_seconds: i64,
    pub idle_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRow {
    pub id: i64,
    pub game_id: i64,
    pub started_at: i64,
    pub ended_at: i64,
    pub active_seconds: i64,
    pub idle_seconds: i64,
    pub open: bool,
}

pub fn summary_all(conn: &Connection, now: i64) -> rusqlite::Result<HashMap<i64, PlaytimeSummary>> {
    let week_from = now / HOUR - WEEK_HOURS;
    let mut stmt = conn.prepare(
        "SELECT g.id, g.platform_minutes, g.platform_2wks_minutes,
            COALESCE((SELECT SUM(active_seconds + idle_seconds) FROM playtime_hourly h WHERE h.game_id = g.id), 0),
            COALESCE((SELECT SUM(idle_seconds) FROM playtime_hourly h WHERE h.game_id = g.id), 0),
            COALESCE((SELECT SUM(active_seconds + idle_seconds) FROM playtime_hourly h
                      WHERE h.game_id = g.id AND h.hour >= ?1), 0),
            (SELECT COUNT(*) FROM sessions s WHERE s.game_id = g.id)
         FROM games g",
    )?;
    let rows = stmt.query_map([week_from], |r| {
        let platform_minutes: Option<i64> = r.get(1)?;
        let tracked: i64 = r.get(3)?;
        Ok((
            r.get::<_, i64>(0)?,
            PlaytimeSummary {
                total_seconds: tracked.max(platform_minutes.unwrap_or(0) * 60),
                tracked_seconds: tracked,
                idle_seconds: r.get(4)?,
                week_seconds: r.get(5)?,
                platform_minutes,
                platform_two_weeks_minutes: r.get(2)?,
                sessions: r.get(6)?,
            },
        ))
    })?;
    rows.collect()
}

/// Steam'in yerel sürelerini oyunlara yazar. Dönüş: güncellenen oyun sayısı.
pub fn apply_steam_import(
    conn: &Connection,
    data: &HashMap<String, SteamPlaytime>,
) -> rusqlite::Result<usize> {
    let mut updated = 0;
    let mut stmt = conn.prepare(
        "UPDATE games SET platform_minutes = ?2, platform_2wks_minutes = ?3,
            last_played = MAX(COALESCE(last_played, 0), COALESCE(?4, 0))
         WHERE platform = 'steam' AND platform_id = ?1",
    )?;
    for (appid, p) in data {
        updated += stmt.execute(params![
            appid,
            i64::try_from(p.total_minutes).unwrap_or(i64::MAX),
            i64::try_from(p.two_weeks_minutes).unwrap_or(i64::MAX),
            p.last_played
        ])?;
    }
    Ok(updated)
}

/// `[from, to)` aralığındaki süreyi saat sınırlarından bölerek saatlik kovalara ekler.
/// `idle` true ise süre AFK sayılır.
pub fn add_interval(
    conn: &Connection,
    game_id: i64,
    from: i64,
    to: i64,
    idle: bool,
) -> rusqlite::Result<()> {
    let mut t = from;
    while t < to {
        let hour = t.div_euclid(HOUR);
        let hour_end = (hour + 1) * HOUR;
        let chunk = to.min(hour_end) - t;
        let (active, idle_s) = if idle { (0, chunk) } else { (chunk, 0) };
        conn.execute(
            "INSERT INTO playtime_hourly (game_id, hour, active_seconds, idle_seconds) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (game_id, hour) DO UPDATE SET
                active_seconds = active_seconds + excluded.active_seconds,
                idle_seconds = idle_seconds + excluded.idle_seconds",
            params![game_id, hour, active, idle_s],
        )?;
        t += chunk;
    }
    Ok(())
}

pub fn open_session(conn: &Connection, game_id: i64, now: i64) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO sessions (game_id, started_at, ended_at, open) VALUES (?1, ?2, ?2, 1)",
        params![game_id, now],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn extend_session(
    conn: &Connection,
    id: i64,
    now: i64,
    active: i64,
    idle: i64,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE sessions SET ended_at = ?2, active_seconds = active_seconds + ?3,
            idle_seconds = idle_seconds + ?4 WHERE id = ?1",
        params![id, now, active, idle],
    )?;
    conn.execute(
        "UPDATE games SET last_played = ?2 WHERE id = (SELECT game_id FROM sessions WHERE id = ?1)",
        params![id, now],
    )?;
    Ok(())
}

pub fn close_session(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("UPDATE sessions SET open = 0 WHERE id = ?1", [id])?;
    // Çok kısa açılışlar (başlatıcı açılıp kapanması) istatistiği kirletmesin.
    conn.execute(
        "DELETE FROM sessions WHERE id = ?1 AND active_seconds + idle_seconds < 30",
        [id],
    )?;
    Ok(())
}

/// Uygulama çökmüş/kapatılmışsa açık kalan oturumları kapatır (son kayıtlı ana kadar sayılmıştır).
pub fn close_stale_sessions(conn: &Connection) -> rusqlite::Result<usize> {
    conn.execute("UPDATE sessions SET open = 0 WHERE open = 1", [])
}

pub fn hourly_rows(
    conn: &Connection,
    from_hour: i64,
    to_hour: i64,
    game_id: Option<i64>,
) -> rusqlite::Result<Vec<HourRow>> {
    let mut stmt = conn.prepare(
        "SELECT game_id, hour, active_seconds, idle_seconds FROM playtime_hourly
         WHERE hour >= ?1 AND hour < ?2 AND (?3 IS NULL OR game_id = ?3) ORDER BY hour",
    )?;
    let rows = stmt.query_map(params![from_hour, to_hour, game_id], |r| {
        Ok(HourRow {
            game_id: r.get(0)?,
            hour: r.get(1)?,
            active_seconds: r.get(2)?,
            idle_seconds: r.get(3)?,
        })
    })?;
    rows.collect()
}

pub fn sessions_between(
    conn: &Connection,
    from: i64,
    to: i64,
    game_id: Option<i64>,
) -> rusqlite::Result<Vec<SessionRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, game_id, started_at, ended_at, active_seconds, idle_seconds, open FROM sessions
         WHERE ended_at >= ?1 AND started_at < ?2 AND (?3 IS NULL OR game_id = ?3)
         ORDER BY started_at DESC LIMIT 500",
    )?;
    let rows = stmt.query_map(params![from, to, game_id], |r| {
        Ok(SessionRow {
            id: r.get(0)?,
            game_id: r.get(1)?,
            started_at: r.get(2)?,
            ended_at: r.get(3)?,
            active_seconds: r.get(4)?,
            idle_seconds: r.get(5)?,
            open: r.get::<_, i64>(6)? != 0,
        })
    })?;
    rows.collect()
}

#[cfg(test)]
pub fn open_session_for(conn: &Connection, game_id: i64) -> rusqlite::Result<Option<i64>> {
    use rusqlite::OptionalExtension;
    conn.query_row(
        "SELECT id FROM sessions WHERE game_id = ?1 AND open = 1 ORDER BY id DESC LIMIT 1",
        [game_id],
        |r| r.get(0),
    )
    .optional()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::detectors::{DetectedGame, Platform};
    use crate::library;

    fn setup() -> (Connection, i64) {
        let mut conn = db::open_in_memory().unwrap();
        let g = DetectedGame {
            platform: Platform::Steam,
            platform_id: "730".into(),
            title: "CS2".into(),
            install_dir: "D:\\x".into(),
            executable: None,
            launch_uri: String::new(),
            size_bytes: None,
            last_played: None,
        };
        library::sync(&mut conn, &[g], &[Platform::Steam], 1).unwrap();
        let id = library::list(&conn).unwrap()[0].id;
        (conn, id)
    }

    #[test]
    fn intervals_split_across_hour_boundaries() {
        let (conn, id) = setup();
        // 10:50 → 11:20 (UTC): 600 sn 10. saate, 1200 sn 11. saate.
        let t = 1_000 * HOUR + 50 * 60;
        add_interval(&conn, id, t, t + 30 * 60, false).unwrap();
        add_interval(&conn, id, t + 30 * 60, t + 35 * 60, true).unwrap();
        let rows = hourly_rows(&conn, 0, i64::MAX, Some(id)).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!((rows[0].hour, rows[0].active_seconds), (1000, 600));
        assert_eq!((rows[1].active_seconds, rows[1].idle_seconds), (1200, 300));
    }

    #[test]
    fn summary_uses_larger_of_platform_and_tracked() {
        let (conn, id) = setup();
        let mut map = HashMap::new();
        map.insert(
            "730".to_string(),
            SteamPlaytime {
                total_minutes: 120,
                two_weeks_minutes: 30,
                last_played: Some(50),
            },
        );
        map.insert("999".to_string(), SteamPlaytime::default());
        assert_eq!(apply_steam_import(&conn, &map).unwrap(), 1);
        let now = 2_000 * HOUR;
        add_interval(&conn, id, now - 600, now, false).unwrap();
        let s = &summary_all(&conn, now).unwrap()[&id];
        assert_eq!(s.platform_minutes, Some(120));
        assert_eq!(s.platform_two_weeks_minutes, Some(30));
        assert_eq!(s.tracked_seconds, 600);
        assert_eq!(s.week_seconds, 600);
        assert_eq!(s.total_seconds, 7200);
    }

    #[test]
    fn session_lifecycle_and_short_session_cleanup() {
        let (conn, id) = setup();
        let s = open_session(&conn, id, 100).unwrap();
        assert_eq!(open_session_for(&conn, id).unwrap(), Some(s));
        extend_session(&conn, s, 200, 80, 20).unwrap();
        close_session(&conn, s).unwrap();
        let rows = sessions_between(&conn, 0, 1_000, None).unwrap();
        assert_eq!(rows.len(), 1);
        assert!(!rows[0].open);
        assert_eq!(library::get(&conn, id).unwrap().last_played, Some(200));

        let short = open_session(&conn, id, 300).unwrap();
        extend_session(&conn, short, 310, 10, 0).unwrap();
        close_session(&conn, short).unwrap();
        assert_eq!(
            sessions_between(&conn, 0, 1_000, None).unwrap().len(),
            1,
            "30 sn altı silinir"
        );

        open_session(&conn, id, 400).unwrap();
        assert_eq!(close_stale_sessions(&conn).unwrap(), 1);
    }
}
