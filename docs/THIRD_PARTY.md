# THIRD_PARTY — Dış kaynaklar ve lisanslar

Lisansı belirsiz hiçbir şey kullanılmaz. Yeni bağımlılık eklenince bu dosya aynı commit'te güncellenir.

## Çalışma zamanı (Rust)

| Paket                                                 | Lisans                                            | Kullanım                                                     |
| ----------------------------------------------------- | ------------------------------------------------- | ------------------------------------------------------------ |
| tauri 2                                               | MIT / Apache-2.0                                  | Masaüstü çatısı                                              |
| tauri-plugin-opener 2                                 | MIT / Apache-2.0                                  | Dış bağlantıları varsayılan tarayıcıda açma                  |
| serde, serde_json                                     | MIT / Apache-2.0                                  | Serileştirme                                                 |
| winreg 0.55                                           | MIT                                               | Windows kayıt defteri (sürüm tespiti, platform dedektörleri) |
| rusqlite 0.40 (bundled SQLite)                        | MIT (SQLite: kamu malı)                           | Yerel veritabanı                                             |
| ureq 3 + rustls + webpki-roots                        | MIT/Apache-2.0; webpki-roots: CDLA-Permissive-2.0 | Görsel indirme (HTTPS)                                       |
| image 0.25 (png, jpeg, ico)                           | MIT / Apache-2.0                                  | Görsel doğrulama, ikon/ICO üretimi                           |
| windows 0.61                                          | MIT / Apache-2.0                                  | Windows API (ikon çıkarma, kısayol, Known Folder)            |
| tauri-plugin-dialog / -autostart / -single-instance 2 | MIT / Apache-2.0                                  | Dosya seçici, Windows açılışı, tek örnek                     |
| auto-launch 0.6 (autostart bağımlılığı)               | MIT                                               | HKCU\…\Run kaydı                                             |
| thiserror 2, tracing 0.1                              | MIT / Apache-2.0                                  | Hata türleri, günlükleme                                     |
| tempfile 3 (yalnız test)                              | MIT / Apache-2.0                                  | Geçici test klasörleri                                       |

## Çalışma zamanı (arayüz)

| Paket                                          | Lisans           | Kullanım           |
| ---------------------------------------------- | ---------------- | ------------------ |
| react, react-dom 19                            | MIT              | Arayüz             |
| @tauri-apps/api 2, @tauri-apps/plugin-opener 2 | MIT / Apache-2.0 | IPC                |
| motion 14 (Framer Motion)                      | MIT              | Animasyonlar       |
| react-router 8                                 | MIT              | Sayfa yönlendirme  |
| zustand 5                                      | MIT              | Arayüz durumu      |
| @tanstack/react-query 5                        | MIT              | IPC veri önbelleği |
| tailwindcss 4, @tailwindcss/vite               | MIT              | Stil               |

## Geliştirme araçları

| Paket                                                                                                  | Lisans           |
| ------------------------------------------------------------------------------------------------------ | ---------------- |
| vite, @vitejs/plugin-react                                                                             | MIT              |
| typescript                                                                                             | Apache-2.0       |
| eslint, @eslint/js, typescript-eslint, eslint-plugin-react-hooks, eslint-plugin-react-refresh, globals | MIT              |
| prettier                                                                                               | MIT              |
| vitest, jsdom, @testing-library/*                                                                      | MIT              |
| @tauri-apps/cli                                                                                        | MIT / Apache-2.0 |
| NSIS (bundler)                                                                                         | zlib/libpng      |
| WiX Toolset 3.14 (bundler)                                                                             | MS-RL            |

## Claude Code bileşenleri

| Kaynak                                                       | Lisans                                                  |
| ------------------------------------------------------------ | ------------------------------------------------------- |
| aitmpl / claude-code-templates (agent, skill, command, hook) | MIT                                                     |
| skill-creator (Anthropic)                                    | Apache-2.0 (`.claude/skills/skill-creator/LICENSE.txt`) |

## Fontlar, ikonlar, görseller, veri

| Kaynak                                                          | Lisans      | Not                                                                                                                    |
| --------------------------------------------------------------- | ----------- | ---------------------------------------------------------------------------------------------------------------------- |
| Sora (Sora Project Authors) — `@fontsource-variable/sora` 5.3   | SIL OFL 1.1 | Başlık fontu, değişken ağırlık, Latin + Latin Extended (Türkçe) woff2                                                  |
| Manrope (Mikhail Sharanda) — `@fontsource-variable/manrope` 5.3 | SIL OFL 1.1 | Metin fontu, değişken ağırlık, Latin + Latin Extended woff2                                                            |
| Phosphor Icons 2.1 — `@phosphor-icons/web`                      | MIT         | Yalnızca `Phosphor.woff2` ve `Phosphor-Fill.woff2` alınır (`scripts/vendor-icons.mjs`); paketteki SVG font kullanılmaz |

| Oyun Arşivim logosu (`assets/brand/`, `scripts/render-logo.ps1`) | Projeye ait (MIT) | Betikle çizilen özgün PNG; Tauri ikonları bundan üretilir |
| Steam kapak/logo/hero görselleri | Valve / yayıncılar | Yalnızca kullanıcının kendi kütüphanesindeki oyunlar için yerel önbelleğe indirilir, depoya girmez, yeniden dağıtılmaz |

Planlanan: Ludusavi manifest (MIT, Faz 11); Steam CDN görselleri yalnızca kullanıcının kendi kütüphanesi için önbelleğe alınır, yeniden dağıtılmaz (Faz 7).
