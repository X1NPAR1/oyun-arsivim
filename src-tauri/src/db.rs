//! SQLite bağlantısı ve sıralı migration'lar. Her migration bir kez, işlem (transaction) içinde uygulanır.
use std::path::Path;

use rusqlite::Connection;

/// Sıralı şema migration'ları. Yayınlanmış bir migration ASLA değiştirilmez; yenisi eklenir.
const MIGRATIONS: &[&str] = &[
    // v1 — oyunlar
    "CREATE TABLE games (
        id           INTEGER PRIMARY KEY AUTOINCREMENT,
        platform     TEXT    NOT NULL,
        platform_id  TEXT    NOT NULL,
        title        TEXT    NOT NULL,
        install_dir  TEXT    NOT NULL DEFAULT '',
        executable   TEXT,
        launch_uri   TEXT    NOT NULL DEFAULT '',
        size_bytes   INTEGER,
        last_played  INTEGER,
        installed    INTEGER NOT NULL DEFAULT 1,
        added_at     INTEGER NOT NULL,
        removed_at   INTEGER,
        UNIQUE (platform, platform_id)
    );
    CREATE INDEX idx_games_installed ON games (installed);",
    // v2 — ayarlar (anahtar → JSON)
    "CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
];

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("veritabanı hatası: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("veritabanı klasörü oluşturulamadı: {0}")]
    Io(#[from] std::io::Error),
}

pub fn open(path: &Path) -> Result<Connection, DbError> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut conn = Connection::open(path)?;
    configure(&conn)?;
    migrate(&mut conn)?;
    Ok(conn)
}

#[cfg(test)]
pub fn open_in_memory() -> Result<Connection, DbError> {
    let mut conn = Connection::open_in_memory()?;
    configure(&conn)?;
    migrate(&mut conn)?;
    Ok(conn)
}

fn configure(conn: &Connection) -> Result<(), DbError> {
    // WAL: okuma/yazma çakışmalarında arayüzü bloklamaz; foreign_keys ileride ilişkiler için.
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(())
}

pub fn schema_version(conn: &Connection) -> Result<usize, DbError> {
    let v: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    Ok(usize::try_from(v).unwrap_or(0))
}

fn migrate(conn: &mut Connection) -> Result<(), DbError> {
    let current = schema_version(conn)?;
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(current) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(
            None,
            "user_version",
            i64::try_from(index + 1).unwrap_or(i64::MAX),
        )?;
        tx.commit()?;
        tracing::info!(version = index + 1, "veritabanı migration'ı uygulandı");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_apply_once_and_are_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("alt klasör/kütüphane.sqlite");
        let conn = open(&path).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), MIGRATIONS.len());
        drop(conn);
        // İkinci açılış migration'ları tekrar çalıştırmamalı.
        let conn = open(&path).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), MIGRATIONS.len());
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
