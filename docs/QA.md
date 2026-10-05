# QA — Test stratejisi ve sonuçlar

## Strateji

| Katman           | Araç                       | Kapsam                                                                               |
| ---------------- | -------------------------- | ------------------------------------------------------------------------------------ |
| Rust birim       | `cargo test`               | Ayrıştırıcılar (fixture), süre hesabı, yedek/geri yükleme, puanlama, migration'lar   |
| Rust entegrasyon | `src-tauri/tests/`         | IPC komutları; geçici DB ve geçici klasörlerle                                       |
| Arayüz birim     | Vitest + RTL               | Bileşenler, store'lar, biçimlendiriciler                                             |
| E2E              | WebdriverIO + tauri-driver | Splash → ana sayfa, gezinme, arama, detay, ayar, yedek, performans modu              |
| Görsel           | `design-review` skill'i    | Her fazda ekran görüntüsü + kontrol listesi                                          |
| Dayanıklılık     | fixture üretici            | Bozuk manifest, izin yok, ağ yok, uzun adlar, `C:\Oyunlar\Şahin'in Oyunu`, 1000 oyun |
| Kurulum          | NSIS                       | Temiz kurulum, güncelleme, kaldırma                                                  |

Kapsam hedefi: Rust çekirdek ≥ %80 (`cargo llvm-cov`, Faz 17'de zorunlu kapı).

## Sonuçlar

### Faz 1 — 2026-10-05

| Kontrol                        | Sonuç                                                                                          |
| ------------------------------ | ---------------------------------------------------------------------------------------------- |
| tsc strict                     | GEÇTİ                                                                                          |
| ESLint (0 uyarı)               | GEÇTİ                                                                                          |
| Prettier                       | GEÇTİ                                                                                          |
| Vitest                         | 2/2 GEÇTİ (sürüm gösterimi, IPC hata mesajı)                                                   |
| cargo fmt / clippy -D warnings | GEÇTİ                                                                                          |
| cargo test                     | 2/2 GEÇTİ                                                                                      |
| Release build                  | NSIS 1.18 MB, MSI 1.69 MB                                                                      |
| Gerçek çalıştırma              | Release exe açıldı, Rust'tan gelen sürüm göründü, RAM 23 MB (`review/faz-01/release-main.png`) |

### Faz 2 — 2026-10-05

| Kontrol         | Sonuç                                                                                                                                 |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| Vitest          | 16/16 GEÇTİ (hareket token'ları, yön hesabı, animasyon tercihi, kabuk gezinmesi, SVG yokluğu)                                         |
| cargo test      | 5/5 GEÇTİ (Mica sürüm eşiği, gerçek Windows build okuma)                                                                              |
| Gerçek kullanım | Dev penceresinde menü tıklamalarıyla gezinme, stagger kareleri (`gecis-kutuphane-*.png`), animasyon seviyesi değiştirme; RAM 30–36 MB |

### Faz 3 — 2026-10-05

| Kontrol                | Sonuç                                                                                                              |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------ |
| cargo test             | 37/37 GEÇTİ (VDF, Steam/Epic fixture'ları, senkron, migration, ayarlar, görsel önbelleği, ikon, kısayol)           |
| Vitest                 | 27/27 GEÇTİ                                                                                                        |
| Gerçek sistem taraması | Steam 14 oyun (2 kütüphane: F:, D:), Epic 3 manifest → 2 kurulu + 1 klasörü silinmiş; 2 DLC + Unreal Engine elendi |
| Tepsi / RAM (release)  | Pencere açık 283 MB (7 süreç) → tepside 6 MB (1 süreç); `--hidden` başlangıç 2 MB                                  |
| Tek örnek              | İkinci başlatma mevcut süreçte pencereyi açtı, ikinci kopya kapandı                                                |
| Windows açılışı        | Açınca `HKCU\…\Run` = `oyun-arsivim.exe --hidden`, kapatınca kayıt silindi                                         |
| Masaüstü kısayolu      | Sağ tık → `Counter-Strike 2.url` (`steam://rungameid/730`, ikonlu) oluştu; test sonrası silindi                    |

### Faz 4 — 2026-10-05

| Kontrol           | Sonuç                                                                                                                                                                    |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| cargo test        | 49/49 GEÇTİ (Steam süre ayrıştırma, saat bölme, oturum yaşam döngüsü, takip durum makinesi, eşleyici, günlük)                                                            |
| Vitest            | 40/40 GEÇTİ (toplama, içgörüler, eksen adımı, özet)                                                                                                                      |
| Gerçek takip      | Test oyunu (ping.exe kopyası, Türkçe yol) diyalogdan eklendi, çalıştırıldı → oturum, saatlik süre, istatistik ve günlük kaydı doğrulandı; ardından test oyunu kaldırıldı |
| Steam içe aktarma | 14 oyun, toplam 19.585 sa; CS2 5.979 sa, son 2 hafta 12 sa 48 dk                                                                                                         |
| Vitrin            | 3 hızlı → basışından 1'i uygulandı (kilit); görsel 10 sn merkez yakınlaşma; 10. sn'de otomatik geçiş                                                                     |
| Tam ekran         | 1920×1080 ana sayfa / kütüphane / istatistik düzenleri bozulmasız                                                                                                        |

### Faz 5 — 2026-10-05

| Kontrol                    | Sonuç                                                                                                                                                  |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| cargo test                 | 66+ GEÇTİ (ölçüm, ayarlar, beyaz liste, RAM, güç planı, korumalı süreçler, profil)                                                                     |
| Gerçek ölçüm               | CPU 16 iş parçacığı, RAM 6,6/15,9 GB, GPU %1, VRAM 1.407 MB (RX 6700 XT)                                                                               |
| Kullanıcı ayarları         | 7 ayarın her biri ters çevrildi → doğrulandı → orijinaline döndü                                                                                       |
| Arayüzden ayar             | Game DVR anahtarı: `GameDVR_Enabled` 1 → 0 → 1                                                                                                         |
| Yönetici görevi            | Yetkisiz: HKLM yazımı reddedildi (kod 5, değer değişmedi); beyaz liste dışı reddedildi (kod 3); bekleme belleği ayrıcalıksız güvenle başarısız (kod 6) |
| RAM temizliği              | 109 süreç, 1.277 MB çalışma kümesi; kullanılabilir 10.325 → 10.625 MB                                                                                  |
| Uçtan uca profil (tepside) | Test oyunu: plan Yüksek → Nihai Performans, öncelik High; oyun kapanınca plan Yüksek'e döndü                                                           |
| **Bekleyen**               | UAC onaylı HAGS / ağ kısıtlaması / MMCSS ve bekleme belleği: kullanıcının onay vermesi gerekir                                                         |

### Faz 6 — 2026-10-06

| Kontrol                           | Sonuç                                                                                                    |
| --------------------------------- | -------------------------------------------------------------------------------------------------------- |
| Vitest                            | 45/45 GEÇTİ (ilerleme yumuşatma, çember geometrisi dahil)                                                |
| cargo test                        | GEÇTİ (adım ağırlıkları = 100, durum metni eşikleri)                                                     |
| Soğuk açılış (release)            | Splash 1,5 sn'de görünür, ~0,95 sn kalır, toplam 2,5 sn (hedef < 3 sn)                                   |
| Kaçan olay                        | Splash, kendisinden önce biten açılışın son durumunu (%100 · Hazır!) doğru gösterdi                      |
| Hata ekranı (debug test anahtarı) | Mesaj + Tekrar dene + Günlüğü aç; süreç çökmedi; Tekrar dene günlükte yeniden deneme + yeni sonuç üretti |
| Gerileme                          | Tepsi (18 MB), tek örnek, `--hidden` (pencere yok, 14 MB) çalışıyor                                      |

## Bulunan ve çözülen hatalar

| #   | Faz | Hata                                                           | Regresyon koruması                                  | Çözüm                                                 |
| --- | --- | -------------------------------------------------------------- | --------------------------------------------------- | ----------------------------------------------------- |
| 1   | 1   | LNK1104 msvcrt.lib                                             | cargo build                                         | Linker/SDK yolları makine cargo config'ine sabitlendi |
| 2   | 1   | WiX LGHT0311 (ş, 1252)                                         | release build (CI)                                  | MSI dili tr-TR                                        |
| 3   | 1   | Vitest 5 reddedilen `vi.fn` sonucu test hatası                 | `App.test.tsx`                                      | Düz fonksiyon mock                                    |
| 4   | 2   | Win10'da sert pencere köşeleri + gölgenin dikdörtgen çerçevesi | `review/faz-02/kutuphane-v2.png`                    | Win10'da gölge kapalı, kabuk 16px yuvarlatır          |
| 5   | 2   | Yerel kaydırma çubuğu okları tasarıma aykırı                   | görsel inceleme                                     | İnce, oksuz `::-webkit-scrollbar`                     |
| 6   | 2   | Kullanıcı: degradeler/ışımalar "yapay zekâ hissi" veriyor      | `review/faz-02/v3-*.png`                            | Nötr grafit palet; ışıma ve renkli küreler kaldırıldı |
| 7   | 3   | Klasörü silinmiş GTA V kurulu görünüyordu                      | `epic::scans_directory…`, `steam::finds_games…`     | Kurulum klasörü varlık kontrolü                       |
| 8   | 3   | notepad.exe ikonu çıkarılamadı (.mun)                          | `icon::extracts_icon_from_real_system_exe`          | Shell IShellItemImageFactory yedeği                   |
| 9   | 3   | Wallpaper Engine oyun olarak listeleniyordu                    | `steam` filtre listesi                              | Yazılım appid'leri elendi                             |
| 10  | 4   | Takipçi hiç kayıt yapmıyordu (i64 taşması)                     | `tracker::matcher_refresh_schedule_never_overflows` | `Option<i64>` + `saturating_sub`                      |
| 11  | 4   | Tek günlük veriyle "en az Salı oynuyorsun"                     | `aggregate` "en sessiz gün" testi                   | ≥3 oynanan gün şartı                                  |
| 12  | 4   | Grafik ekseninde tekrar eden etiketler                         | `niceStep` testi                                    | Yuvarlak süre adımları                                |
| 13  | 4   | Vitrinde arka plan logodan sonra geliyordu                     | görsel zaman çizelgesi                              | Önceden çözme + eşzamanlı değişim                     |

## Tasarım değerlendirmeleri

- Faz 1: Tasarım sistemi henüz yok (Faz 2). Pencere, Türkçe başlık ve IPC metni doğru. Varsayılan font görülüyor → Faz 2'de giderilecek.
- Faz 2 (`review/faz-02/`): 1 Sert dikdörtgen yok — GEÇTİ (pencere 16px, yüzeyler 22–28px) · 2 Cam/derinlik — GEÇTİ · 3 Sora/Manrope, Türkçe glifler — GEÇTİ · 4 SVG yok (test + arama) — GEÇTİ · 5 Kontrast — GEÇTİ · 6 Hiyerarşi/taşma — GEÇTİ ("İstatistikler" 64px'e sığıyor) · 7 Yüzen sidebar + özel başlık çubuğu — GEÇTİ · 8 Hareket: stagger ve yönlü geçiş kareleri akıcı — GEÇTİ.
