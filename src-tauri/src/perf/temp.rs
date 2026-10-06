//! Geçici dosya temizliği: kullanıcı `%TEMP%` ve `C:\Windows\Temp`.
//!
//! Güvenlik: yalnız bu iki kökün *içindekiler* silinir (kökler kalır), sembolik bağlar izlenmez,
//! kullanımdaki/kilitli dosyalar ve yetki gerektirenler sessizce atlanır. Son 1 saatte değişen
//! dosyalar çalışan kurulumları bozmamak için korunur.
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::Serialize;

/// Bu süreden yeni dosyalara dokunulmaz (kurulum/güncelleme sırasında kullanılıyor olabilir).
pub const MIN_AGE: Duration = Duration::from_secs(60 * 60);

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TempUsage {
    pub bytes: u64,
    pub files: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TempCleanReport {
    pub freed_bytes: u64,
    pub deleted_files: u64,
    /// Kullanımda, yeni veya yetki gerektirdiği için atlanan dosyalar.
    pub skipped_files: u64,
}

/// Temizlenecek kökler: kullanıcı geçici klasörü ve Windows geçici klasörü (varsa).
pub fn roots() -> Vec<PathBuf> {
    let mut out = vec![std::env::temp_dir()];
    if let Some(win) = std::env::var_os("SystemRoot") {
        out.push(PathBuf::from(win).join("Temp"));
    }
    out.retain(|p| p.is_dir());
    out.dedup();
    out
}

fn old_enough(meta: &std::fs::Metadata, now: SystemTime) -> bool {
    meta.modified()
        .ok()
        .and_then(|m| now.duration_since(m).ok())
        .is_some_and(|age| age >= MIN_AGE)
}

/// Kökün içindekilerin boyutu (silinebilecek olanlar).
pub fn usage_in(root: &Path, now: SystemTime) -> TempUsage {
    let mut u = TempUsage::default();
    walk(root, &mut |_, meta| {
        if old_enough(meta, now) {
            u.bytes += meta.len();
            u.files += 1;
        }
    });
    u
}

/// Kökün içini temizler; kökün kendisi silinmez. Boşalan alt klasörler kaldırılır.
pub fn clean_in(root: &Path, now: SystemTime) -> TempCleanReport {
    let mut r = TempCleanReport::default();
    walk(root, &mut |path, meta| {
        if old_enough(meta, now) && std::fs::remove_file(path).is_ok() {
            r.freed_bytes += meta.len();
            r.deleted_files += 1;
        } else {
            r.skipped_files += 1;
        }
    });
    remove_empty_dirs(root, true);
    r
}

/// Dosyaları derinlemesine dolaşır; sembolik bağlar/bağlantı noktaları izlenmez.
fn walk(dir: &Path, f: &mut dyn FnMut(&Path, &std::fs::Metadata)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(meta) = std::fs::symlink_metadata(entry.path()) else {
            continue;
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            walk(&entry.path(), f);
        } else {
            f(&entry.path(), &meta);
        }
    }
}

fn remove_empty_dirs(dir: &Path, is_root: bool) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let is_real_dir = std::fs::symlink_metadata(entry.path())
            .is_ok_and(|m| m.is_dir() && !m.file_type().is_symlink());
        if is_real_dir {
            remove_empty_dirs(&entry.path(), false);
        }
    }
    if !is_root {
        // Yalnız boşsa başarılı olur.
        let _ = std::fs::remove_dir(dir);
    }
}

pub fn usage() -> TempUsage {
    let now = SystemTime::now();
    roots().iter().fold(TempUsage::default(), |mut acc, r| {
        let u = usage_in(r, now);
        acc.bytes += u.bytes;
        acc.files += u.files;
        acc
    })
}

pub fn clean() -> TempCleanReport {
    let now = SystemTime::now();
    let mut total = TempCleanReport::default();
    for root in roots() {
        let r = clean_in(&root, now);
        total.freed_bytes += r.freed_bytes;
        total.deleted_files += r.deleted_files;
        total.skipped_files += r.skipped_files;
    }
    tracing::info!(
        freed_mb = total.freed_bytes / 1_048_576,
        deleted = total.deleted_files,
        skipped = total.skipped_files,
        "geçici dosyalar temizlendi"
    );
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_old_files_keeps_root_and_recent_files() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path();
        std::fs::create_dir_all(root.join("a/b")).unwrap();
        std::fs::write(root.join("old.tmp"), [0u8; 100]).unwrap();
        std::fs::write(root.join("a/b/old2.tmp"), [0u8; 50]).unwrap();
        // "Şimdi"yi 2 saat ileri alarak dosyaları eski say.
        let later = SystemTime::now() + Duration::from_secs(2 * 60 * 60);
        assert_eq!(
            usage_in(root, later),
            TempUsage {
                bytes: 150,
                files: 2
            }
        );
        // Şu an: hepsi yeni → hiçbiri silinmez.
        let r = clean_in(root, SystemTime::now());
        assert_eq!((r.deleted_files, r.skipped_files), (0, 2));
        assert!(root.join("old.tmp").exists());

        let r = clean_in(root, later);
        assert_eq!((r.freed_bytes, r.deleted_files), (150, 2));
        assert!(root.is_dir(), "kök klasör silinmemeli");
        assert!(!root.join("a").exists(), "boşalan alt klasörler kaldırılır");
    }

    #[test]
    fn roots_exist() {
        assert!(roots().iter().all(|r| r.is_dir()));
        assert!(!roots().is_empty());
    }
}
