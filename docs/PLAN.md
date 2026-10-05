# PLAN — Oyun Arşivim

## 1. Hedef

Windows'ta günlük kullanıma hazır, sinematik ve animasyonlu bir oyun kütüphanesi: otomatik tarama (8 kaynak),
başlatma, süre takibi, save yedekleme, performans modu, akıllı öneriler, istatistikler. 18 fazda `v1.0.0`.

## 2. Mimari özet

```
┌──────────── React (WebView2) ─────────────┐      ┌──────────────── Rust (Tauri 2) ────────────────┐
│ pages/  components/  design/ (token)      │      │ startup ─ splash ilerleme olayları             │
│ stores/ (Zustand)  queries (TanStack)     │ IPC  │ detectors/{steam,epic,gog,ea,ubisoft,          │
│ ipc/ (specta'dan üretilen tipler)         │◄────►│            battlenet,xbox,manual,folder}       │
│ i18n/ (tr, en)                            │ olay │ db (SQLite+migration) settings logging         │
└───────────────────────────────────────────┘      │ launcher tracker backup performance artwork    │
                                                   │ recommendations  tray  updater                 │
                                                   └────────────────────────────────────────────────┘
```

Ayrıntı: `docs/ARCHITECTURE.md`. Tasarım: `docs/DESIGN_SYSTEM.md`. Özellikler: `docs/SPEC.md`.

## 3. Fazlar, görevler, bağımlılıklar, kabul kriterleri

| Faz | Görevler                                                                                                                                                                                                                                                   | Bağımlılık | Kabul kriteri                                                                      | Etiket  |
| --- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- | ---------------------------------------------------------------------------------- | ------- |
| 1   | Araçlar (Rust→D:, VS BT, WebView2), Tauri+React iskeleti, kalite araçları, hafıza/doküman dosyaları, aitmpl bileşenleri + 4 proje skill'i, GitHub deposu, CI + Release iş akışları, updater anahtarı                                                       | —          | Boş uygulama açılıyor, CI yeşil, v0.1.0 yayınlandı                                 | v0.1.0  |
| 2   | Design token'ları (CSS değişkenleri + TS), Tailwind, Sora/Manrope woff2, ikon fontu, özel başlık çubuğu, Mica/Acrylic + yedek, yüzen cam sidebar, router + yönlü sayfa geçişleri, reduced-motion & animasyon seviyesi                                      | 1          | Animasyonlu geçişler, tasarım kurallarına uygun, ekran görüntüleri değerlendirildi | v0.2.0  |
| 3   | Splash penceresi (çerçevesiz, yuvarlak), PNG logo animasyonu, canvas degrade çember, gerçek ilerleme (`startup://progress`), kayan durum metni, hata + Tekrar dene/Günlüğü aç, titreşimsiz ana pencereye geçiş; uzun işlemler için ortak ilerleme bileşeni | 2          | Titreme yok, ilerleme gerçek, hata yönetiliyor                                     | v0.3.0  |
| 4   | SQLite + migration sistemi, şema v1, ayar deposu (tipli, anında kalıcı), rotating log, crash-state dosyası, specta tip üretimi                                                                                                                             | 3          | Migration/ayar testleri geçiyor, ayarlar kalıcı                                    | v0.4.0  |
| 5   | `GameDetector` trait, Steam (registry + libraryfolders.vdf + appmanifest .acf, filtreler), Epic (.item manifestleri), elle ekleme (exe seç/sürükle-bırak, düzenleme), fixture üretici                                                                      | 4          | Fixture testleri + gerçek sistem testi başarılı                                    | v0.5.0  |
| 6   | GOG, EA, Ubisoft, Battle.net, Xbox dedektörleri; tekilleştirme; `notify` ile klasör izleme; klasör tarama önerileri                                                                                                                                        | 5          | Tüm dedektörler test edildi, sınırlamalar PLATFORMS.md'de                          | v0.6.0  |
| 7   | Artwork: Steam CDN görselleri, SteamGridDB (DPAPI şifreli anahtar), exe ikonu + degrade yedek kapak üretici, thumbnail + blur placeholder önbelleği, offline                                                                                               | 5          | Her oyunun şık kapağı var, internetsiz çalışıyor                                   | v0.7.0  |
| 8   | Kütüphane: stagger kart girişi, layout animasyonları, ızgara/liste, sıralama, filtre, arama, TanStack Virtual, tilt/shine                                                                                                                                  | 7          | 1000 oyunda 60 FPS, akıcı animasyon                                                | v0.8.0  |
| 9   | Oyun detay sayfası, paylaşılan öğe geçişi (kapak → hero), baskın renk çıkarımı → dinamik vurgu                                                                                                                                                             | 8          | Kart ↔ detay geçişi kusursuz                                                       | v0.9.0  |
| 10  | Launcher (protokol/exe/parametre), süreç ağacı + klasör eşleme ile oyun süreci tespiti, oturum kaydı, uyku/boşta düşme, sistem tepsisi arka plan izleme                                                                                                    | 5, 9       | Farklı platformlardan başlatma, süreler doğru                                      | v0.10.0 |
| 11  | Ludusavi manifest, save konumu çözümleme, zip+SHA-256 yedek, otomatik/zamanlı, limit, güvenli geri yükleme (önce güvenlik yedeği, oyun açıkken engel)                                                                                                      | 10         | Sahte oyunla yedek/geri yükleme, veri kaybı yok                                    | v0.11.0 |
| 12  | Performans modu: seçili süreçleri kapatma (korumalı liste), güç planı, odak yardımı, geri alma, çökme sonrası durum dosyasıyla geri alma                                                                                                                   | 10         | Oyun kapanınca ve çökme sonrası geri alınıyor                                      | v0.12.0 |
| 13  | Ana sayfa (öne çıkan, devam et, öneriler, haftalık özet), "Bugün ne oynasam?" (kart çevirme), içgörüler, disk içgörüleri, akıllı + kullanıcı koleksiyonları, Ctrl+K fuzzy komut paleti                                                                     | 10         | Öneriler anlamlı, arama hızlı (<16 ms/tuş)                                         | v0.13.0 |
| 14  | İstatistikler: Chart.js (günlük/haftalık/aylık, platform dağılımı, en çok oynanan), canvas ısı haritası                                                                                                                                                    | 10         | Grafikler doğru veriyle                                                            | v0.14.0 |
| 15  | Ayarlar sayfası (tüm bölümler, anında uygulanır), i18n TR/EN, Windows açılışında başlat                                                                                                                                                                    | 4          | Tüm ayarlar çalışıyor, metin taşması yok                                           | v0.15.0 |
| 16  | Tauri Updater + `latest.json` (GitHub Releases), "Şimdi kontrol et"                                                                                                                                                                                        | 1          | Eski → yeni güncelleme test edildi                                                 | v0.16.0 |
| 17  | Performans optimizasyonu, erişilebilirlik (klavye, odak, kontrast), dayanıklılık (bozuk manifest, Türkçe yollar, 1000 oyun, ağsız), E2E tamamlanması, %80 Rust kapsamı                                                                                     | 1–16       | Performans hedefleri tutuldu, E2E tam                                              | v0.17.0 |
| 18  | Son cilalama, README + ekran görüntüleri, NSIS temiz kurulum/güncelleme/kaldırma testi                                                                                                                                                                     | 17         | Kurulum temiz sistemde sorunsuz                                                    | v1.0.0  |

## 4. Performans hedefleri

Soğuk açılış < 3 sn (ilk tarama hariç) · boşta RAM < 150 MB · 500+ oyunda 60 FPS · küçük kurulum.

## 5. Faz kapanış protokolü

Prompt §4.2 — `.claude/skills/phase-close/SKILL.md` ile otomatikleştirildi.

## 6. Riskler

| Risk                                                                | Önlem                                                                                                     |
| ------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| Xbox/MS Store oyunlarının exe'leri `WindowsApps` altında erişilemez | `shell:AppsFolder\<AUMID>` ile başlatma; süre takibi paket kurulum klasörüne göre; sınırlama belgelenecek |
| Launcher üzerinden açılan oyunlarda asıl süreci bulmak              | Süreç ağacı + kurulum klasörü eşleşmesi + zaman penceresi; test edilebilir saf mantık                     |
| Performans modunda yanlış süreç kapatma                             | Yalnızca kullanıcı listesi + sabit korumalı liste + kendi kullanıcısına ait süreçler                      |
| E2E'de tauri-driver Windows'ta msedgedriver sürüm eşleşmesi         | CI'da WebView2 sürümüne göre sürücü indirme betiği                                                        |
