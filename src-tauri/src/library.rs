//! Kütüphane: dedektör sonuçlarını veritabanıyla senkronlar, elle eklenen oyunları yönetir.
//!
//! Kural: oyun kayıtları asla otomatik silinmez. Taramada bulunamayan oyun `installed = 0` olur ve
//! `removed_at` yazılır; geçmişi korunur. Yeniden kurulursa kendiliğinden geri gelir.
use std::collections::HashSet;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::Serialize;

use crate::detectors::{DetectedGame, Platform};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: i64,
    pub platform: Platform,
    pub platform_id: String,
    pub title: String,
    pub install_dir: String,
    pub executable: Option<String>,
    pub launch_uri: String,
    pub size_bytes: Option<i64>,
    pub last_played: Option<i64>,
    pub installed: bool,
    pub added_at: i64,
    pub removed_at: Option<i64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncReport {
    pub found: usize,
    pub added: usize,
    pub removed: usize,
    pub restored: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum LibraryError {
    #[error(transparent)]
    Db(#[from] rusqlite::Error),
    #[error("dosya bulunamadı: {0}")]
    NotFound(String),
    #[error("yalnızca .exe dosyaları eklenebilir")]
    NotExecutable,
    #[error("bu oyun zaten kütüphanende")]
    AlreadyExists,
    #[error("oyun bulunamadı")]
    UnknownGame,
}

const SELECT: &str = "SELECT id, platform, platform_id, title, install_dir, executable, launch_uri,
    size_bytes, last_played, installed, added_at, removed_at FROM games";

fn row_to_game(row: &Row<'_>) -> rusqlite::Result<Game> {
    let platform: String = row.get(1)?;
    Ok(Game {
        id: row.get(0)?,
        platform: Platform::parse(&platform).unwrap_or(Platform::Manual),
        platform_id: row.get(2)?,
        title: row.get(3)?,
        install_dir: row.get(4)?,
        executable: row.get(5)?,
        launch_uri: row.get(6)?,
        size_bytes: row.get(7)?,
        last_played: row.get(8)?,
        installed: row.get::<_, i64>(9)? != 0,
        added_at: row.get(10)?,
        removed_at: row.get(11)?,
    })
}

/// Kurulu oyunlar önce, sonra son oynanmaya ve ada göre.
pub fn list(conn: &Connection) -> Result<Vec<Game>, LibraryError> {
    let mut stmt = conn.prepare(&format!(
        "{SELECT} ORDER BY installed DESC, COALESCE(last_played, 0) DESC, title COLLATE NOCASE"
    ))?;
    let games = stmt
        .query_map([], row_to_game)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(games)
}

pub fn get(conn: &Connection, id: i64) -> Result<Game, LibraryError> {
    conn.query_row(&format!("{SELECT} WHERE id = ?1"), [id], row_to_game)
        .optional()?
        .ok_or(LibraryError::UnknownGame)
}

/// Taranan platformların sonuçlarını uygular. `scanned` yalnızca başarıyla taranan platformları
/// içermelidir; hata veren bir dedektörün oyunları yanlışlıkla "kaldırıldı" sayılmasın.
pub fn sync(
    conn: &mut Connection,
    detected: &[DetectedGame],
    scanned: &[Platform],
    now: i64,
) -> Result<SyncReport, LibraryError> {
    let tx = conn.transaction()?;
    let mut report = SyncReport {
        found: detected.len(),
        ..SyncReport::default()
    };
    let mut present: HashSet<(&'static str, &str)> = HashSet::new();

    for game in detected {
        present.insert((game.platform.as_str(), game.platform_id.as_str()));
        let existing: Option<(i64, i64)> = tx
            .query_row(
                "SELECT id, installed FROM games WHERE platform = ?1 AND platform_id = ?2",
                params![game.platform.as_str(), game.platform_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let size = game.size_bytes.and_then(|s| i64::try_from(s).ok());
        match existing {
            None => {
                tx.execute(
                    "INSERT INTO games (platform, platform_id, title, install_dir, executable,
                        launch_uri, size_bytes, last_played, installed, added_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9)",
                    params![
                        game.platform.as_str(),
                        game.platform_id,
                        game.title,
                        game.install_dir,
                        game.executable,
                        game.launch_uri,
                        size,
                        game.last_played,
                        now
                    ],
                )?;
                report.added += 1;
            }
            Some((id, installed)) => {
                if installed == 0 {
                    report.restored += 1;
                }
                // last_played: platformun bildirdiği ile bizim kaydettiğimizin büyüğü.
                tx.execute(
                    "UPDATE games SET title = ?2, install_dir = ?3, executable = ?4, launch_uri = ?5,
                        size_bytes = ?6, last_played = MAX(COALESCE(last_played, 0), COALESCE(?7, 0)),
                        installed = 1, removed_at = NULL
                     WHERE id = ?1",
                    params![
                        id,
                        game.title,
                        game.install_dir,
                        game.executable,
                        game.launch_uri,
                        size,
                        game.last_played
                    ],
                )?;
            }
        }
    }

    for platform in scanned {
        let mut stmt =
            tx.prepare("SELECT id, platform_id FROM games WHERE platform = ?1 AND installed = 1")?;
        let rows = stmt
            .query_map([platform.as_str()], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        drop(stmt);
        for (id, pid) in rows {
            if !present.contains(&(platform.as_str(), pid.as_str())) {
                tx.execute(
                    "UPDATE games SET installed = 0, removed_at = ?2 WHERE id = ?1",
                    params![id, now],
                )?;
                report.removed += 1;
            }
        }
    }

    // Elle eklenenler: exe hâlâ yerinde mi?
    let manual: Vec<(i64, Option<String>, i64)> = {
        let mut stmt =
            tx.prepare("SELECT id, executable, installed FROM games WHERE platform = 'manual'")?;
        let rows = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };
    for (id, exe, installed) in manual {
        let exists = exe.as_deref().is_some_and(|p| Path::new(p).is_file());
        if exists && installed == 0 {
            tx.execute(
                "UPDATE games SET installed = 1, removed_at = NULL WHERE id = ?1",
                [id],
            )?;
            report.restored += 1;
        } else if !exists && installed == 1 {
            tx.execute(
                "UPDATE games SET installed = 0, removed_at = ?2 WHERE id = ?1",
                params![id, now],
            )?;
            report.removed += 1;
        }
    }

    tx.commit()?;
    Ok(report)
}

/// Elle oyun ekler. Kimlik, exe yolunun küçük harfli biçimidir (aynı exe iki kez eklenmez).
pub fn add_manual(
    conn: &Connection,
    exe_path: &Path,
    title: Option<&str>,
    now: i64,
) -> Result<Game, LibraryError> {
    if !exe_path.is_file() {
        return Err(LibraryError::NotFound(exe_path.display().to_string()));
    }
    let is_exe = exe_path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("exe"));
    if !is_exe {
        return Err(LibraryError::NotExecutable);
    }
    let exe = exe_path.to_string_lossy().into_owned();
    let platform_id = exe.to_lowercase();
    let title = title
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| default_title(exe_path));
    let install_dir = exe_path
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    let inserted = conn.execute(
        "INSERT OR IGNORE INTO games (platform, platform_id, title, install_dir, executable,
            launch_uri, installed, added_at)
         VALUES ('manual', ?1, ?2, ?3, ?4, '', 1, ?5)",
        params![platform_id, title, install_dir, exe, now],
    )?;
    if inserted == 0 {
        return Err(LibraryError::AlreadyExists);
    }
    get(conn, conn.last_insert_rowid())
}

/// Elle eklenen oyunu kütüphaneden çıkarır (diskteki dosyalara dokunmaz).
pub fn remove_manual(conn: &Connection, id: i64) -> Result<(), LibraryError> {
    let n = conn.execute(
        "DELETE FROM games WHERE id = ?1 AND platform = 'manual'",
        [id],
    )?;
    if n == 0 {
        return Err(LibraryError::UnknownGame);
    }
    Ok(())
}

pub fn rename(conn: &Connection, id: i64, title: &str) -> Result<Game, LibraryError> {
    let title = title.trim();
    if !title.is_empty() {
        conn.execute(
            "UPDATE games SET title = ?2 WHERE id = ?1",
            params![id, title],
        )?;
    }
    get(conn, id)
}

/// "Oyun Klasörü\\MyGame-Win64-Shipping.exe" → "MyGame".
fn default_title(exe: &Path) -> String {
    let stem = exe
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let cleaned = [
        "-Win64-Shipping",
        "-Win32-Shipping",
        "_x64",
        "_x86",
        "-x64",
        "64",
    ]
    .iter()
    .fold(stem, |acc, suffix| {
        acc.strip_suffix(suffix).map(str::to_owned).unwrap_or(acc)
    });
    let spaced = cleaned.replace(['_', '-'], " ");
    let t = spaced.trim();
    if t.is_empty() {
        "Adsız oyun".to_owned()
    } else {
        t.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn game(platform: Platform, id: &str, title: &str) -> DetectedGame {
        DetectedGame {
            platform,
            platform_id: id.to_owned(),
            title: title.to_owned(),
            install_dir: format!("D:\\Oyunlar\\{title}"),
            executable: None,
            launch_uri: format!("steam://rungameid/{id}"),
            size_bytes: Some(10),
            last_played: Some(100),
        }
    }

    #[test]
    fn sync_adds_marks_removed_and_restores() {
        let mut conn = db::open_in_memory().unwrap();
        let cs = game(Platform::Steam, "730", "Counter-Strike 2");
        let un = game(Platform::Steam, "304930", "Unturned");

        let r = sync(&mut conn, &[cs.clone(), un.clone()], &[Platform::Steam], 1).unwrap();
        assert_eq!((r.found, r.added, r.removed), (2, 2, 0));

        // Unturned kaldırıldı → silinmez, işaretlenir.
        let r = sync(&mut conn, std::slice::from_ref(&cs), &[Platform::Steam], 2).unwrap();
        assert_eq!(r.removed, 1);
        let games = list(&conn).unwrap();
        assert_eq!(games.len(), 2);
        let removed = games.iter().find(|g| g.platform_id == "304930").unwrap();
        assert!(!removed.installed);
        assert_eq!(removed.removed_at, Some(2));
        assert_eq!(
            games[0].title, "Counter-Strike 2",
            "kurulu oyunlar önce gelir"
        );

        // Yeniden kuruldu → geri gelir.
        let r = sync(&mut conn, &[cs, un], &[Platform::Steam], 3).unwrap();
        assert_eq!(r.restored, 1);
        assert!(list(&conn)
            .unwrap()
            .iter()
            .all(|g| g.installed && g.removed_at.is_none()));
    }

    #[test]
    fn failed_platform_scan_does_not_mark_games_removed() {
        let mut conn = db::open_in_memory().unwrap();
        sync(
            &mut conn,
            &[game(Platform::Epic, "abc", "Dying Light")],
            &[Platform::Epic],
            1,
        )
        .unwrap();
        // Epic bu turda taranamadı (scanned listesinde yok).
        let r = sync(&mut conn, &[], &[Platform::Steam], 2).unwrap();
        assert_eq!(r.removed, 0);
        assert!(list(&conn).unwrap()[0].installed);
    }

    #[test]
    fn last_played_never_goes_backwards() {
        let mut conn = db::open_in_memory().unwrap();
        let mut g = game(Platform::Steam, "1", "A");
        g.last_played = Some(500);
        sync(&mut conn, &[g.clone()], &[Platform::Steam], 1).unwrap();
        g.last_played = Some(100);
        sync(&mut conn, &[g], &[Platform::Steam], 2).unwrap();
        assert_eq!(list(&conn).unwrap()[0].last_played, Some(500));
    }

    #[test]
    fn manual_game_lifecycle() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("Şahin'in Oyunu");
        std::fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("Sahin_Game-Win64-Shipping.exe");
        std::fs::write(&exe, b"MZ").unwrap();
        let mut conn = db::open_in_memory().unwrap();

        let g = add_manual(&conn, &exe, None, 1).unwrap();
        assert_eq!(g.title, "Sahin Game");
        assert_eq!(g.platform, Platform::Manual);
        assert!(g.installed);
        assert!(matches!(
            add_manual(&conn, &exe, None, 1),
            Err(LibraryError::AlreadyExists)
        ));

        // exe silinince "kaldırıldı", geri gelince "kurulu".
        std::fs::remove_file(&exe).unwrap();
        sync(&mut conn, &[], &[], 2).unwrap();
        assert!(!get(&conn, g.id).unwrap().installed);
        std::fs::write(&exe, b"MZ").unwrap();
        sync(&mut conn, &[], &[], 3).unwrap();
        assert!(get(&conn, g.id).unwrap().installed);

        assert_eq!(rename(&conn, g.id, "  Şahin  ").unwrap().title, "Şahin");
        remove_manual(&conn, g.id).unwrap();
        assert!(list(&conn).unwrap().is_empty());
    }

    #[test]
    fn manual_add_validates_input() {
        let tmp = tempfile::tempdir().unwrap();
        let conn = db::open_in_memory().unwrap();
        let txt = tmp.path().join("not.txt");
        std::fs::write(&txt, "x").unwrap();
        assert!(matches!(
            add_manual(&conn, &txt, None, 1),
            Err(LibraryError::NotExecutable)
        ));
        assert!(matches!(
            add_manual(&conn, &tmp.path().join("yok.exe"), None, 1),
            Err(LibraryError::NotFound(_))
        ));
    }

    #[test]
    fn steam_games_cannot_be_removed_via_manual_removal() {
        let mut conn = db::open_in_memory().unwrap();
        sync(
            &mut conn,
            &[game(Platform::Steam, "730", "CS2")],
            &[Platform::Steam],
            1,
        )
        .unwrap();
        let id = list(&conn).unwrap()[0].id;
        assert!(matches!(
            remove_manual(&conn, id),
            Err(LibraryError::UnknownGame)
        ));
    }
}
