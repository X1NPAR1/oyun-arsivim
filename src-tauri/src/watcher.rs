//! Platform klasörlerini izler: yeni oyun kurulunca veya silinince kütüphane kendiliğinden güncellenir.
//!
//! İzlenen: Steam kütüphanelerinin `steamapps` klasörleri (appmanifest_*.acf), Epic manifest klasörü,
//! Riot metadata klasörü. Kurulumlar çok sayıda dosya olayı üretir; son olaydan sonra `QUIET` kadar
//! sessizlik beklenip tek bir tarama yapılır.
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use notify::{EventKind, RecursiveMode, Watcher};
use tauri::AppHandle;

use crate::commands::{self, SharedState};
use crate::detectors::{epic, steam};

const QUIET: Duration = Duration::from_secs(4);

/// İzlenecek klasörler (var olanlar).
pub fn watch_paths() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let s = steam::SteamDetector::system_default();
    if let Some(root) = s.root() {
        out.extend(
            s.library_paths(root)
                .into_iter()
                .map(|p| p.join("steamapps")),
        );
    }
    if let Some(dir) = epic::EpicDetector::system_default().manifests_dir() {
        out.push(dir.to_path_buf());
    }
    if let Some(p) =
        std::env::var_os("ProgramData").map(|p| PathBuf::from(p).join(r"Riot Games\Metadata"))
    {
        out.push(p);
    }
    out.retain(|p| p.is_dir());
    out.sort();
    out.dedup();
    out
}

/// Bir dosya olayı yeniden taramayı gerektiriyor mu? (manifest/ayar dosyaları; geçici dosyalar değil)
pub fn is_relevant(path: &std::path::Path) -> bool {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    (name.starts_with("appmanifest_") && name.ends_with(".acf"))
        || name.ends_with(".item")
        || name.ends_with(".product_settings.yaml")
}

pub fn spawn(app: AppHandle, state: SharedState) {
    let paths = watch_paths();
    if paths.is_empty() {
        return;
    }
    std::thread::Builder::new()
        .name("klasor-izleyici".into())
        .spawn(move || {
            let (tx, rx) = mpsc::channel();
            let mut watcher =
                match notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                    if let Ok(ev) = res {
                        if matches!(
                            ev.kind,
                            EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_)
                        ) && ev.paths.iter().any(|p| is_relevant(p))
                        {
                            let _ = tx.send(());
                        }
                    }
                }) {
                    Ok(w) => w,
                    Err(err) => {
                        tracing::warn!(%err, "klasör izleyici başlatılamadı");
                        return;
                    }
                };
            for p in &paths {
                // Riot ürün klasörleri alt klasörlerde; diğerleri düz.
                let mode = if p.ends_with("Metadata") {
                    RecursiveMode::Recursive
                } else {
                    RecursiveMode::NonRecursive
                };
                if let Err(err) = watcher.watch(p, mode) {
                    tracing::warn!(path = %p.display(), %err, "klasör izlenemiyor");
                }
            }
            tracing::info!(count = paths.len(), "platform klasörleri izleniyor");
            let mut pending: Option<Instant> = None;
            loop {
                match rx.recv_timeout(Duration::from_millis(500)) {
                    Ok(()) => pending = Some(Instant::now()),
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
                if pending.is_some_and(|t| t.elapsed() >= QUIET) {
                    pending = None;
                    tracing::info!("platform klasöründe değişiklik: kütüphane yeniden taranıyor");
                    if let Err(err) = commands::run_scan(&app, &state) {
                        tracing::warn!(%err, "otomatik tarama başarısız");
                    }
                }
            }
        })
        .map(|_| ())
        .unwrap_or_else(|err| tracing::error!(%err, "izleyici iş parçacığı başlatılamadı"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn only_manifest_files_trigger_rescan() {
        assert!(is_relevant(Path::new(
            r"D:\SteamLibrary\steamapps\appmanifest_730.acf"
        )));
        assert!(is_relevant(Path::new(
            r"C:\ProgramData\Epic\Manifests\ABC.item"
        )));
        assert!(is_relevant(Path::new(
            r"C:\ProgramData\Riot Games\Metadata\valorant.live\valorant.live.product_settings.yaml"
        )));
        assert!(!is_relevant(Path::new(
            r"D:\SteamLibrary\steamapps\downloading\730\file.tmp"
        )));
        assert!(!is_relevant(Path::new(
            r"D:\SteamLibrary\steamapps\libraryfolders.vdf.tmp"
        )));
    }

    #[cfg(windows)]
    #[test]
    fn notify_reports_new_manifest_in_watched_folder() {
        // Gerçek dosya sistemi olayı: geçici "steamapps" klasörüne manifest yazılınca olay gelir.
        let t = tempfile::tempdir().unwrap();
        let (tx, rx) = mpsc::channel();
        let mut w = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(ev) = res {
                if ev.paths.iter().any(|p| is_relevant(p)) {
                    let _ = tx.send(());
                }
            }
        })
        .unwrap();
        w.watch(t.path(), RecursiveMode::NonRecursive).unwrap();
        std::fs::write(t.path().join("appmanifest_999.acf"), "\"AppState\" {}").unwrap();
        assert!(rx.recv_timeout(Duration::from_secs(5)).is_ok());
    }
}
