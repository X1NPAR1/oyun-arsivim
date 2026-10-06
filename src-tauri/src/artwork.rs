//! Oyun görselleri: yerel önbellek, Steam CDN indirme, exe ikonundan logo.
//!
//! Dosya adları: `<platform>_<anahtar>_<tür>.<uzantı>`. Bulunamayan görseller `.missing` işaret
//! dosyasıyla hatırlanır; böylece her açılışta tekrar istek atılmaz ve internetsiz çalışılır.
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::detectors::Platform;
use crate::library::Game;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Artwork {
    pub cover: Option<String>,
    pub logo: Option<String>,
    pub hero: Option<String>,
    pub icon: Option<String>,
    /// 16:9 tam ekran sahne görseli (Steam mağaza ekran görüntüsü, 1920×1080).
    pub backdrop: Option<String>,
    /// Görselden türetilen vurgu rengi (dinamik tema).
    pub accent: Option<String>,
    /// Logo koyu mu (koyu sahnede beyaz silüet çizilir).
    pub logo_dark: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Cover,
    Logo,
    Hero,
    Icon,
    Backdrop,
}

impl Kind {
    fn file_suffix(self) -> &'static str {
        match self {
            Kind::Cover => "cover.jpg",
            Kind::Logo => "logo.png",
            // 2x sürüm: tam ekran sahnede bulanık görünmesin (eski 1x önbellek ad değişikliğiyle yenilenir).
            Kind::Hero => "hero2x.jpg",
            Kind::Icon => "icon.png",
            Kind::Backdrop => "backdrop.jpg",
        }
    }
}

/// Ağ erişimi soyutlaması: testler sahte uygulama verir.
pub trait Fetcher: Send + Sync {
    /// 200 → `Some(bayt)`, 404 → `None`, ağ hatası → `Err`.
    fn get(&self, url: &str) -> Result<Option<Vec<u8>>, String>;
}

pub struct HttpFetcher {
    agent: ureq::Agent,
}

impl HttpFetcher {
    pub fn new() -> Self {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(15)))
            .http_status_as_error(false)
            .user_agent(concat!("OyunArsivim/", env!("CARGO_PKG_VERSION")))
            .build();
        Self {
            agent: config.into(),
        }
    }
}

impl Default for HttpFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Fetcher for HttpFetcher {
    fn get(&self, url: &str) -> Result<Option<Vec<u8>>, String> {
        let mut resp = self.agent.get(url).call().map_err(|e| e.to_string())?;
        match resp.status().as_u16() {
            200 => resp
                .body_mut()
                .with_config()
                .limit(20 * 1024 * 1024)
                .read_to_vec()
                .map(Some)
                .map_err(|e| e.to_string()),
            404 | 403 => Ok(None),
            other => Err(format!("HTTP {other}")),
        }
    }
}

pub struct ArtworkStore {
    dir: PathBuf,
}

const STEAM_CDN: &str = "https://cdn.cloudflare.steamstatic.com/steam/apps";

impl ArtworkStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn file(&self, platform: Platform, key: &str, kind: Kind) -> PathBuf {
        let safe: String = key
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        self.dir.join(format!(
            "{}_{}_{}",
            platform.as_str(),
            safe,
            kind.file_suffix()
        ))
    }

    fn missing_marker(path: &Path) -> PathBuf {
        let mut p = path.as_os_str().to_owned();
        p.push(".missing");
        PathBuf::from(p)
    }

    fn existing(path: PathBuf) -> Option<String> {
        path.is_file().then(|| path.to_string_lossy().into_owned())
    }

    fn key(game: &Game) -> String {
        match game.platform {
            Platform::Manual => game.id.to_string(),
            _ => game.platform_id.clone(),
        }
    }

    /// Önbellekte olanları döndürür; ağa çıkmaz.
    pub fn cached(&self, game: &Game) -> Artwork {
        let key = Self::key(game);
        Artwork {
            cover: Self::existing(self.file(game.platform, &key, Kind::Cover)),
            logo: Self::existing(self.file(game.platform, &key, Kind::Logo)),
            hero: Self::existing(self.file(game.platform, &key, Kind::Hero)),
            icon: Self::existing(self.file(game.platform, &key, Kind::Icon)),
            backdrop: Self::existing(self.file(game.platform, &key, Kind::Backdrop)),
            ..self.meta(game.platform, &key)
        }
    }

    fn meta_path(&self, platform: Platform, key: &str) -> PathBuf {
        let base = self.file(platform, key, Kind::Cover);
        PathBuf::from(base.to_string_lossy().replace("_cover.jpg", "_meta.json"))
    }

    fn meta(&self, platform: Platform, key: &str) -> Artwork {
        let m: crate::art_analysis::ArtMeta = std::fs::read(self.meta_path(platform, key))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        Artwork {
            accent: m.accent,
            logo_dark: m.logo_dark,
            ..Artwork::default()
        }
    }

    /// Görseller değiştiyse (veya meta hiç yoksa) vurgu rengi ve logo tonunu yeniden hesaplar.
    fn refresh_meta(&self, game: &Game, force: bool) -> bool {
        let key = Self::key(game);
        let path = self.meta_path(game.platform, &key);
        if path.is_file() && !force {
            return false;
        }
        let pick = |k: Kind| Some(self.file(game.platform, &key, k)).filter(|p| p.is_file());
        let color = pick(Kind::Cover)
            .or_else(|| pick(Kind::Hero))
            .or_else(|| pick(Kind::Icon));
        let meta = crate::art_analysis::analyze(color.as_deref(), pick(Kind::Logo).as_deref());
        serde_json::to_vec(&meta).is_ok_and(|b| std::fs::write(&path, b).is_ok())
    }

    /// Eksik görselleri hazırlar. Hata oyun başına loglanır; biri başarısız diye diğerleri durmaz.
    /// Dönen değer: yeni dosya üretildi mi (arayüzün yenilenmesi için).
    pub fn ensure(&self, game: &Game, fetcher: &dyn Fetcher) -> bool {
        if let Err(err) = std::fs::create_dir_all(&self.dir) {
            tracing::warn!(%err, "görsel klasörü oluşturulamadı");
            return false;
        }
        let key = Self::key(game);
        let mut changed = false;
        if game.platform == Platform::Steam {
            let id = &game.platform_id;
            let sources = [
                (
                    Kind::Cover,
                    vec![
                        format!("{STEAM_CDN}/{id}/library_600x900_2x.jpg"),
                        format!("{STEAM_CDN}/{id}/library_600x900.jpg"),
                    ],
                ),
                (Kind::Logo, vec![format!("{STEAM_CDN}/{id}/logo.png")]),
                (
                    Kind::Hero,
                    vec![
                        format!("{STEAM_CDN}/{id}/library_hero_2x.jpg"),
                        format!("{STEAM_CDN}/{id}/library_hero.jpg"),
                        format!("{STEAM_CDN}/{id}/header.jpg"),
                    ],
                ),
            ];
            for (kind, urls) in sources {
                changed |= self.download(&self.file(game.platform, &key, kind), &urls, fetcher);
            }
            changed |=
                self.store_backdrop(id, &self.file(game.platform, &key, Kind::Backdrop), fetcher);
            changed |= self.store_header_fallback(
                id,
                &self.file(game.platform, &key, Kind::Hero),
                fetcher,
            );
        }
        // Steam dışındaki platformlarda logo: Xbox paketinin resmi logosu, diğerlerinde oyunun exe ikonu.
        if game.platform != Platform::Steam {
            let target = self.file(game.platform, &key, Kind::Icon);
            if !target.is_file() {
                changed |= self.make_icon(game, &target);
            }
        }
        // Yeni görsel geldiyse renk/ton yeniden hesaplanır; meta yoksa ilk kez üretilir.
        changed |= self.refresh_meta(game, changed);
        changed
    }

    fn make_icon(&self, game: &Game, target: &Path) -> bool {
        let dir = Path::new(&game.install_dir);
        if game.platform == Platform::Xbox {
            return match xbox_logo(dir).and_then(|src| image::open(src).ok()) {
                Some(img) => img
                    .save_with_format(target, image::ImageFormat::Png)
                    .is_ok(),
                None => false,
            };
        }
        // Riot'ta `executable` istemcidir (Riot logosu); oyunun kendi exe'si klasörde aranır.
        let exe = match game.platform {
            Platform::Riot => crate::detectors::regplatforms::pick_main_exe(dir),
            _ => game
                .executable
                .as_deref()
                .map(PathBuf::from)
                .filter(|p| p.is_file())
                .or_else(|| crate::detectors::regplatforms::pick_main_exe(dir)),
        };
        let Some(exe) = exe else { return false };
        match crate::icon::extract_png(&exe, target) {
            Ok(()) => true,
            Err(err) => {
                tracing::debug!(exe = %exe.display(), %err, "exe ikonu çıkarılamadı");
                false
            }
        }
    }

    /// Tam ekran sahne için 16:9 görsel: mağazanın ilk ekran görüntüsü (1920×1080). Hero görselleri
    /// ~3:1 olduğundan pencereyi doldururken yanlardan çok kırpılıyordu. Bir kez denenir.
    fn store_backdrop(&self, appid: &str, target: &Path, fetcher: &dyn Fetcher) -> bool {
        let tried = PathBuf::from(format!("{}.store", target.display()));
        if target.is_file() || tried.is_file() {
            return false;
        }
        let url = format!(
            "https://store.steampowered.com/api/appdetails?appids={appid}&filters=screenshots"
        );
        let shot = match fetcher.get(&url) {
            Ok(Some(body)) => serde_json::from_slice::<serde_json::Value>(&body)
                .ok()
                .and_then(|v| {
                    v[appid]["data"]["screenshots"][0]["path_full"]
                        .as_str()
                        .map(str::to_owned)
                }),
            Ok(None) => None,
            Err(err) => {
                tracing::debug!(appid, %err, "mağaza API'sine ulaşılamadı");
                return false; // ağ hatası: sonra yeniden dene
            }
        };
        let _ = std::fs::write(&tried, b"");
        match shot {
            Some(url) => self.download(target, &[url], fetcher),
            None => false,
        }
    }

    /// Yeni Steam oyunlarının görselleri karma (hash) içeren adreslerdedir ve eski CDN yolunda yoktur.
    /// Mağaza API'si `header_image` adresini verir; hero olarak kullanılır. Bir kez denenir.
    fn store_header_fallback(&self, appid: &str, hero: &Path, fetcher: &dyn Fetcher) -> bool {
        let marker = Self::missing_marker(hero);
        let tried = PathBuf::from(format!("{}.store", hero.display()));
        if hero.is_file() || !marker.is_file() || tried.is_file() {
            return false;
        }
        let url =
            format!("https://store.steampowered.com/api/appdetails?appids={appid}&filters=basic");
        let header = match fetcher.get(&url) {
            Ok(Some(body)) => serde_json::from_slice::<serde_json::Value>(&body)
                .ok()
                .and_then(|v| v[appid]["data"]["header_image"].as_str().map(str::to_owned)),
            Ok(None) => None,
            Err(err) => {
                tracing::debug!(appid, %err, "mağaza API'sine ulaşılamadı");
                return false; // ağ hatası: sonra yeniden dene
            }
        };
        let _ = std::fs::write(&tried, b"");
        let Some(header) = header else { return false };
        let _ = std::fs::remove_file(&marker);
        self.download(hero, &[header], fetcher)
    }

    fn download(&self, target: &Path, urls: &[String], fetcher: &dyn Fetcher) -> bool {
        let marker = Self::missing_marker(target);
        if target.is_file() || marker.is_file() {
            return false;
        }
        for url in urls {
            match fetcher.get(url) {
                Ok(Some(bytes)) if image::guess_format(&bytes).is_ok() => {
                    // Önce geçici dosyaya yaz, sonra taşı: yarım dosya önbelleğe girmesin.
                    let tmp = target.with_extension("part");
                    if std::fs::write(&tmp, &bytes)
                        .and_then(|()| std::fs::rename(&tmp, target))
                        .is_ok()
                    {
                        return true;
                    }
                    let _ = std::fs::remove_file(&tmp);
                    return false;
                }
                Ok(_) => continue,
                Err(err) => {
                    // Ağ hatası: işaretleme yapma, bir sonraki taramada yeniden denenir.
                    tracing::debug!(url, %err, "görsel indirilemedi");
                    return false;
                }
            }
        }
        let _ = std::fs::write(marker, b"");
        false
    }

    /// Eski sürümlerin bıraktığı 1x hero önbelleğini siler (yerini hero2x aldı).
    pub fn cleanup_legacy(&self) {
        let Ok(entries) = std::fs::read_dir(&self.dir) else {
            return;
        };
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.contains("_hero.jpg") {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }

    /// Masaüstü kısayolu için üretilen .ico dosyasının yolu.
    pub fn shortcut_icon_path(&self, game: &Game) -> PathBuf {
        let _ = std::fs::create_dir_all(&self.dir);
        self.dir.join(format!(
            "{}_{}_shortcut.ico",
            game.platform.as_str(),
            game.id
        ))
    }

    /// Elle eklenen oyun silinince görsellerini de temizler.
    pub fn remove_for(&self, game: &Game) {
        let key = Self::key(game);
        for kind in [
            Kind::Cover,
            Kind::Logo,
            Kind::Hero,
            Kind::Icon,
            Kind::Backdrop,
        ] {
            let p = self.file(game.platform, &key, kind);
            let _ = std::fs::remove_file(Self::missing_marker(&p));
            let _ = std::fs::remove_file(p);
        }
    }
}

/// Xbox paketinin en büyük resmi logosu: MicrosoftGame.config → Square480x480Logo, Square150x150Logo,
/// StoreLogo. Dosya yoksa ölçek varyantları (`Logo.scale-200.png`) arasından en büyüğü seçilir.
pub fn xbox_logo(dir: &Path) -> Option<PathBuf> {
    let config = std::fs::read_to_string(dir.join("MicrosoftGame.config")).ok()?;
    ["Square480x480Logo", "Square150x150Logo", "StoreLogo"]
        .iter()
        .find_map(|attr| {
            let rel = crate::detectors::xbox::xml_attr(&config, "ShellVisuals", attr)?;
            let path = dir.join(rel.replace('/', "\\"));
            if path.is_file() {
                return Some(path);
            }
            let stem = path.file_stem()?.to_string_lossy().to_lowercase();
            std::fs::read_dir(path.parent()?)
                .ok()?
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.file_name().is_some_and(|n| {
                        n.to_string_lossy()
                            .to_lowercase()
                            .starts_with(&format!("{stem}."))
                    })
                })
                .max_by_key(|p| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0))
        })
}

#[cfg(test)]
mod xbox_logo_tests {
    use super::*;

    #[test]
    fn picks_largest_declared_logo_or_scale_variant() {
        let t = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(t.path().join("Assets")).unwrap();
        std::fs::write(
            t.path().join("MicrosoftGame.config"),
            r#"<ShellVisuals Square480x480Logo="Assets/Large.png" StoreLogo="Assets/Store.png"/>"#,
        )
        .unwrap();
        std::fs::write(t.path().join("Assets/Large.scale-100.png"), vec![0u8; 10]).unwrap();
        std::fs::write(t.path().join("Assets/Large.scale-400.png"), vec![0u8; 40]).unwrap();
        let p = xbox_logo(t.path()).unwrap();
        assert!(p.ends_with("Large.scale-400.png"));
        assert!(xbox_logo(&t.path().join("yok")).is_none());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct FakeFetcher {
        responses: HashMap<String, Result<Option<Vec<u8>>, String>>,
        calls: Mutex<Vec<String>>,
    }

    impl Fetcher for FakeFetcher {
        fn get(&self, url: &str) -> Result<Option<Vec<u8>>, String> {
            self.calls.lock().unwrap().push(url.to_owned());
            self.responses.get(url).cloned().unwrap_or(Ok(None))
        }
    }

    fn png() -> Vec<u8> {
        let img = image::RgbaImage::new(2, 2);
        let mut out = std::io::Cursor::new(Vec::new());
        img.write_to(&mut out, image::ImageFormat::Png).unwrap();
        out.into_inner()
    }

    fn steam_game(id: &str) -> Game {
        Game {
            id: 1,
            platform: Platform::Steam,
            platform_id: id.into(),
            title: "CS2".into(),
            install_dir: String::new(),
            executable: None,
            launch_uri: String::new(),
            launch_args: None,
            size_bytes: None,
            last_played: None,
            installed: true,
            added_at: 0,
            removed_at: None,
        }
    }

    #[test]
    fn downloads_with_fallback_and_remembers_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ArtworkStore::new(tmp.path().to_path_buf());
        let mut responses = HashMap::new();
        responses.insert(
            format!("{STEAM_CDN}/730/library_600x900.jpg"),
            Ok(Some(png())),
        );
        responses.insert(format!("{STEAM_CDN}/730/logo.png"), Ok(Some(png())));
        let fetcher = FakeFetcher {
            responses,
            calls: Mutex::new(vec![]),
        };
        let game = steam_game("730");

        assert!(store.ensure(&game, &fetcher));
        let art = store.cached(&game);
        assert!(art.cover.is_some(), "2x yoksa 1x kapak kullanılır");
        assert!(art.logo.is_some());
        assert!(art.hero.is_none());

        // İkinci çağrı: hiç istek atılmaz (var olanlar + .missing işaretleri).
        let before = fetcher.calls.lock().unwrap().len();
        assert!(!store.ensure(&game, &fetcher));
        assert_eq!(fetcher.calls.lock().unwrap().len(), before);
    }

    #[test]
    fn network_error_is_retried_later_and_non_images_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ArtworkStore::new(tmp.path().to_path_buf());
        let mut responses = HashMap::new();
        responses.insert(
            format!("{STEAM_CDN}/1/library_600x900_2x.jpg"),
            Err("ağ yok".to_string()),
        );
        responses.insert(
            format!("{STEAM_CDN}/1/logo.png"),
            Ok(Some(b"<html>".to_vec())),
        );
        let fetcher = FakeFetcher {
            responses,
            calls: Mutex::new(vec![]),
        };
        let game = steam_game("1");
        store.ensure(&game, &fetcher);
        assert!(
            store.cached(&game).logo.is_none(),
            "HTML görsel olarak kaydedilmez"
        );
        let cover = store.file(Platform::Steam, "1", Kind::Cover);
        assert!(
            !ArtworkStore::missing_marker(&cover).exists(),
            "ağ hatası işaretlenmez"
        );
    }
}

#[cfg(test)]
mod store_fallback_tests {
    use super::*;
    use std::sync::Mutex;

    struct Fake(Mutex<Vec<String>>);
    impl Fetcher for Fake {
        fn get(&self, url: &str) -> Result<Option<Vec<u8>>, String> {
            self.0.lock().unwrap().push(url.to_owned());
            if url.contains("filters=screenshots") {
                return Ok(Some(
                    br#"{"9":{"success":true,"data":{"screenshots":[{"path_full":"https://x/ss.jpg"}]}}}"#.to_vec(),
                ));
            }
            if url.contains("appdetails") {
                return Ok(Some(
                    br#"{"9":{"success":true,"data":{"header_image":"https://x/hdr.jpg"}}}"#
                        .to_vec(),
                ));
            }
            if url == "https://x/hdr.jpg" || url == "https://x/ss.jpg" {
                let mut out = std::io::Cursor::new(Vec::new());
                image::RgbaImage::new(2, 2)
                    .write_to(&mut out, image::ImageFormat::Png)
                    .unwrap();
                return Ok(Some(out.into_inner()));
            }
            Ok(None)
        }
    }

    #[test]
    fn uses_store_header_when_cdn_has_no_art() {
        let tmp = tempfile::tempdir().unwrap();
        let store = ArtworkStore::new(tmp.path().to_path_buf());
        let game = Game {
            id: 1,
            platform: Platform::Steam,
            platform_id: "9".into(),
            title: "CS:GO".into(),
            install_dir: String::new(),
            executable: None,
            launch_uri: String::new(),
            launch_args: None,
            size_bytes: None,
            last_played: None,
            installed: true,
            added_at: 0,
            removed_at: None,
        };
        let fake = Fake(Mutex::new(vec![]));
        store.ensure(&game, &fake);
        assert!(store.cached(&game).hero.is_some());
        assert!(
            store.cached(&game).backdrop.is_some(),
            "16:9 ekran görüntüsü sahne görseli olur"
        );
        // Mağaza API'si yalnız bir kez çağrılır.
        let n = fake.0.lock().unwrap().len();
        store.ensure(&game, &fake);
        assert_eq!(fake.0.lock().unwrap().len(), n);
    }
}
