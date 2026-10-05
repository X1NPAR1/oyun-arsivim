//! Dönen (rotating) günlük dosyaları: `<yerel veri>/logs/oyun-arsivim.YYYY-MM-DD.log`, son 7 gün.
//!
//! Kişisel veri yazılmaz: kullanıcı adı içeren yollar `%USERPROFILE%` ile maskelenir.
use std::path::{Path, PathBuf};

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{Builder, Rotation};
use tracing_subscriber::EnvFilter;

pub const MAX_LOG_FILES: usize = 7;

/// Günlükleyiciyi başlatır. Dönen bekçi (guard) uygulama boyunca yaşamalıdır; düşerse son kayıtlar
/// diske yazılmadan kaybolur.
pub fn init(dir: &Path) -> Option<WorkerGuard> {
    if let Err(err) = std::fs::create_dir_all(dir) {
        eprintln!("günlük klasörü oluşturulamadı: {err}");
        return None;
    }
    let appender = Builder::new()
        .rotation(Rotation::DAILY)
        .filename_prefix("oyun-arsivim")
        .filename_suffix("log")
        .max_log_files(MAX_LOG_FILES)
        .build(dir)
        .map_err(|err| eprintln!("günlük dosyası açılamadı: {err}"))
        .ok()?;
    let (writer, guard) = tracing_appender::non_blocking(appender);
    let filter = EnvFilter::try_from_env("OYUN_ARSIVIM_LOG")
        .unwrap_or_else(|_| EnvFilter::new("info,tao=warn,wry=warn"));
    let result = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(writer)
        .with_ansi(false)
        .with_target(true)
        .try_init();
    if result.is_err() {
        return None; // zaten başlatılmış (testler)
    }
    install_panic_hook();
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "Oyun Arşivim başladı");
    Some(guard)
}

/// Paniği (beklenmeyen çökme) günlüğe yazar; sessizce kaybolmasın.
fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!(panic = %mask_personal(&info.to_string()), "beklenmeyen hata (panik)");
        default(info);
    }));
}

/// Kullanıcı klasörü yolunu maskeler: `C:\Users\ahmet\…` → `%USERPROFILE%\…`.
pub fn mask_personal(text: &str) -> String {
    match std::env::var("USERPROFILE") {
        Ok(home) if !home.is_empty() => replace_case_insensitive(text, &home, "%USERPROFILE%"),
        _ => text.to_owned(),
    }
}

fn replace_case_insensitive(text: &str, needle: &str, with: &str) -> String {
    let lower = text.to_lowercase();
    let n = needle.to_lowercase();
    if n.is_empty() || lower.len() != text.len() {
        // Büyük/küçük harf dönüşümü uzunluğu değiştirdiyse (nadir Unicode) birebir değiştir.
        return text.replace(needle, with);
    }
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while let Some(pos) = lower[i..].find(&n) {
        out.push_str(&text[i..i + pos]);
        out.push_str(with);
        i += pos + n.len();
    }
    out.push_str(&text[i..]);
    out
}

pub fn log_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("logs")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_user_profile_case_insensitively() {
        assert_eq!(
            replace_case_insensitive(
                "C:\\Users\\Ahmet\\x and c:\\users\\ahmet\\y",
                "C:\\Users\\Ahmet",
                "%U%"
            ),
            "%U%\\x and %U%\\y"
        );
        assert_eq!(
            replace_case_insensitive("yok", "C:\\Users\\a", "%U%"),
            "yok"
        );
    }

    #[test]
    fn writes_rotating_log_file() {
        let tmp = tempfile::tempdir().unwrap();
        let appender = Builder::new()
            .rotation(Rotation::DAILY)
            .filename_prefix("oyun-arsivim")
            .filename_suffix("log")
            .max_log_files(MAX_LOG_FILES)
            .build(tmp.path())
            .unwrap();
        let (mut writer, guard) = tracing_appender::non_blocking(appender);
        std::io::Write::write_all(&mut writer, b"deneme\n").unwrap();
        drop(guard); // bekçi düşünce tampon diske yazılır
        let files: Vec<_> = std::fs::read_dir(tmp.path()).unwrap().flatten().collect();
        assert_eq!(files.len(), 1);
        let name = files[0].file_name().to_string_lossy().into_owned();
        assert!(
            name.starts_with("oyun-arsivim.") && name.ends_with(".log"),
            "{name}"
        );
    }
}
