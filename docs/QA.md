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

## Bulunan ve çözülen hatalar

| #   | Faz | Hata                                           | Regresyon koruması | Çözüm                                                 |
| --- | --- | ---------------------------------------------- | ------------------ | ----------------------------------------------------- |
| 1   | 1   | LNK1104 msvcrt.lib                             | cargo build        | Linker/SDK yolları makine cargo config'ine sabitlendi |
| 2   | 1   | WiX LGHT0311 (ş, 1252)                         | release build (CI) | MSI dili tr-TR                                        |
| 3   | 1   | Vitest 5 reddedilen `vi.fn` sonucu test hatası | `App.test.tsx`     | Düz fonksiyon mock                                    |

## Tasarım değerlendirmeleri

- Faz 1: Tasarım sistemi henüz yok (Faz 2). Pencere, Türkçe başlık ve IPC metni doğru. Varsayılan font görülüyor → Faz 2'de giderilecek.
