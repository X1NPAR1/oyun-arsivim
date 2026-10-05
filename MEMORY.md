# MEMORY.md — Kararlar, hatalar, bilinen sorunlar, sıradaki adımlar

## Sıradaki adım

Faz 3 — Steam + Epic tespiti ve ana sayfada gerçek oyun logolarıyla yatay animasyonlu şerit
(fare tekerleği, ← → tuşları, tıklama). Dal: `faz/03-oyun-seridi`. Ardından Faz 4 Performans Merkezi.

## Kararlar (gerekçeli)

| Tarih      | Karar                                                                                           | Gerekçe                                                                                                                |
| ---------- | ----------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| 2026-10-05 | Proje `D:\OyunArsivim\app`, araçlar `D:\OyunArsivim\tools`                                      | F: yasak; C:'de yalnız ~10 GB boş, Rust target + cargo cache GB'larca yer tutar                                        |
| 2026-10-05 | Rust D:'ye `CARGO_HOME`/`RUSTUP_HOME` ile kuruldu (kullanıcı ortam değişkeni, admin gerekmez)   | C: alanı ve F: yasağı                                                                                                  |
| 2026-10-05 | Linker/SDK yolu makineye özel `tools\cargo\config.toml` ile sabitlendi                          | rustc vswhere ile F:'deki eksik VS 2026'yı seçiyor ve `msvcrt.lib` bulamıyordu (LNK1104). Depoya girmez, CI etkilenmez |
| 2026-10-05 | `VSLANG=1033`                                                                                   | Türkçe link.exe çıktısı rustc'de "Non-UTF-8 output" uyarısı üretiyordu → clippy sıfır uyarı hedefi                     |
| 2026-10-05 | React 19 (şablon varsayılanı)                                                                   | Prompt "React 18+" diyor; 19 kararlı ve Framer Motion 12 ile uyumlu                                                    |
| 2026-10-05 | MSI dili `tr-TR`                                                                                | WiX varsayılan 1252 kod sayfası "ş" içermiyor (LGHT0311); tr-TR = 1254                                                 |
| 2026-10-05 | NSIS Türkçe+İngilizce, dil seçicili; WebView2 bootstrapper                                      | Küçük kurulum; WebView2 olmayan sistemde otomatik kurulum                                                              |
| 2026-10-05 | CSP katı (`default-src 'self'`), yalnız `core:default` + `opener:default`                       | En az yetki; ihtiyaç oldukça faz faz açılacak                                                                          |
| 2026-10-05 | Release profili `opt-level = "s"`, LTO, strip                                                   | Kurulum boyutu hedefi (ilk build: NSIS 1.18 MB)                                                                        |
| 2026-10-05 | **Tasarım revizyonu:** nötr grafit palet, ışıma/degrade/renkli küre yok, birincil buton beyaz   | Kullanıcı geri bildirimi: mor-turkuaz degradeler "yapay zekâ hissi" veriyordu. Renk oyun görsellerinden gelecek        |
| 2026-10-05 | Faz sırası değişti: 3 = oyun tespiti + logo şeridi, 4 = Performans Merkezi                      | Kullanıcı isteği (bkz. PLAN.md revizyon notu)                                                                          |
| 2026-10-05 | Win10'da Acrylic yerine CSS yedek zemin (degrade + yavaş hareketli ışık küreleri)               | Acrylic Win10'da sürükleme/boyutlandırmada bilinen ciddi takılma yapıyor; Win11'de Mica                                |
| 2026-10-05 | Win10'da OS gölgesi kapalı, köşeleri kabuk yuvarlatır (16px, büyütülünce 0)                     | Win10 köşe yuvarlatmıyor; gölge açıkken yuvarlak köşelerin arkasında dikdörtgen çerçeve kalıyordu                      |
| 2026-10-05 | Pencere `visible:false` başlar, ilk boyamadan sonra `show()`                                    | Beyaz flaş olmasın (prompt §8.6)                                                                                       |
| 2026-10-05 | İkonlar: Phosphor woff2 + üretilmiş tip güvenli kod noktası tablosu (`src/design/iconCodes.ts`) | Paket CSS'i SVG font referansı içeriyor (kural ihlali); kod noktası tablosuyla yanlış ikon adı derleme hatası olur     |
| 2026-10-05 | Hash router                                                                                     | Masaüstünde adres çubuğu yok; yeniden yüklemede asset sunucusu yönlendirmesi gerekmez                                  |
| 2026-10-05 | Sayfa geçişlerinde eski/yeni sayfa aynı ızgara hücresinde üst üste (AnimatePresence sync)       | `mode="wait"` çıkışı bekletir (~620 ms); "animasyon kullanıcıyı bekletmemeli"                                          |
| 2026-10-05 | Animasyon tercihi şimdilik localStorage'da                                                      | Faz 4'te SQLite ayar deposuna taşınacak                                                                                |
| 2026-10-05 | Updater anahtarı Faz 1'de üretildi, secrets'a kondu; updater eklentisi Faz 16'da açılacak       | Anahtar baştan sabit olmalı (değişirse eski sürümler güncellenemez)                                                    |

## aitmpl bileşenleri (`npx claude-code-templates@latest`, 2026-10-05)

| Bileşen                                            | Tür     | Neden                                                                                                                                                                           | Faz            |
| -------------------------------------------------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------- |
| development-team/frontend-developer                | agent   | React mimarisi, bileşen tasarımı (zorunlu)                                                                                                                                      | 2, 8, 9, 13–15 |
| development-team/ui-ux-designer                    | agent   | Tasarım sistemi ve akışlar                                                                                                                                                      | 2, 3, 8, 9     |
| development-tools/code-reviewer                    | agent   | Faz sonu kod incelemesi                                                                                                                                                         | her faz        |
| development-tools/debugger                         | agent   | Zor hataların kök neden analizi                                                                                                                                                 | gerektikçe     |
| development-tools/test-engineer                    | agent   | Test stratejisi, fixture'lar                                                                                                                                                    | 4–12, 17       |
| performance-testing/react-performance-optimization | agent   | 1000 oyunda 60 FPS                                                                                                                                                              | 8, 17          |
| devops-infrastructure/security-engineer            | agent   | CSP, capability, DPAPI, süreç kapatma güvenliği                                                                                                                                 | 7, 12, 17      |
| documentation/changelog-generator                  | agent   | CHANGELOG/release notları                                                                                                                                                       | her faz        |
| accessibility/accessibility-tester                 | agent   | Klavye, odak, kontrast                                                                                                                                                          | 15, 17         |
| development/skill-creator                          | skill   | Proje skill'leri (zorunlu)                                                                                                                                                      | 1 (+ yenileri) |
| development/rust-pro                               | skill   | Rust arka uç                                                                                                                                                                    | 3–12           |
| development/react-best-practices                   | skill   | React desenleri/performans                                                                                                                                                      | 2+             |
| creative-design/tailwind-patterns                  | skill   | Tailwind token düzeni                                                                                                                                                           | 2+             |
| development/github-actions-creator                 | skill   | CI/release iş akışları                                                                                                                                                          | 1, 16          |
| development/e2e-testing-patterns                   | skill   | E2E                                                                                                                                                                             | 3, 17          |
| development/gh-fix-ci                              | skill   | Kırmızı CI düzeltme                                                                                                                                                             | her faz        |
| development/powershell-windows                     | skill   | Windows betikleri, süreç/güç planı                                                                                                                                              | 10, 12         |
| deployment/prepare-release                         | command | Sürüm hazırlığı                                                                                                                                                                 | her faz        |
| security/dependency-audit                          | command | Bağımlılık denetimi                                                                                                                                                             | 17             |
| security/env-file-protection                       | hook    | `.env` yazımını engeller                                                                                                                                                        | sürekli        |
| ~~security/secret-scanner~~                        | hook    | **Kaldırıldı:** Python gerektiriyor, makinede yalnız Store kısayolu var → her Bash çağrısı hata verirdi. Aynı desenler `scripts/secret-scan.mjs`'e taşındı (commit öncesi + CI) | —              |

Proje skill'leri (skill-creator formatı): `full-check`, `phase-close`, `add-detector`, `design-review`.

## Çözülen hatalar

| Hata                                                                                                        | Çözüm                                                                                                                          |
| ----------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `LNK1104: msvcrt.lib açılamıyor`                                                                            | rustc F:'deki eksik VS 2026'yı seçiyordu → `tools\cargo\config.toml` ile linker + LIB/INCLUDE D:'deki VS 2022 BT'ye sabitlendi |
| WiX `LGHT0311` (kod sayfası 1252)                                                                           | `bundle.windows.wix.language = ["tr-TR"]`                                                                                      |
| Vitest 5: `vi.fn().mockRejectedValue(...)` test hata olarak raporlanıyor (App hatayı yakalasa bile)         | IPC mock'u düz fonksiyon + değiştirilebilir implementasyonla yazıldı (`src/App.test.tsx`)                                      |
| `pnpm config set --location project` kök dizinde (`D:\OyunArsivim`) çalışma alanı oluşturdu                 | Silindi; `storeDir` proje içindeki `pnpm-workspace.yaml`'a yazıldı (YAML'da ters eğik çizgi kaçışına dikkat: `/` kullan)       |
| React 19 `react-hooks/refs`: render sırasında ref okuma/yazma (AnimatedOutlet yön hesabı)                   | "Önceki değeri state'te tut" deseni (render sırasında koşullu `setState`)                                                      |
| Türkçe link.exe çıktısı "Non-UTF-8 output" uyarısı (`VSLANG` işe yaramadı — BT'de yalnız Türkçe dil paketi) | Makine cargo config'ine `rustflags = ['-Alinker-messages']`                                                                    |
| Bash aracı tırnak içeren heredoc'ları ayrıştıramıyor                                                        | Uzun betikler scratchpad'e dosya olarak yazılıp çalıştırılıyor                                                                 |
| Bash aracında çok büyük çoklu heredoc ayrıştırma hatası                                                     | Dosyalar Write aracıyla tek tek yazılıyor                                                                                      |

## Bilinen sorunlar

- (yok)

## Platform tespit notları

Ayrıntılar `docs/PLATFORMS.md`'de; Faz 5–6'da gerçek sistemde doğrulanacak.
