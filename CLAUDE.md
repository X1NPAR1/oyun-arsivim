# CLAUDE.md — Oyun Arşivim

Windows için akıllı oyun kütüphanesi ve başlatıcı. Tüm platformlardaki oyunları bulur, tek bir sinematik
kütüphanede toplar, oynama süresini izler, kayıt dosyalarını yedekler, performans modu ve akıllı öneriler sunar.

**Her oturum başında oku:** `CLAUDE.md` → `MEMORY.md` → `docs/PLAN.md` → `docs/PROGRESS.md`.
Kaldığın fazı `docs/PROGRESS.md`'den bul. Her anlamlı adımdan sonra `MEMORY.md` + `docs/PROGRESS.md` güncelle.

## KESİN KURALLAR

1. **F: diski yasak.** Geliştirme sırasında F:'ye hiçbir şey yazılmaz/kurulmaz. Uygulama F:'deki oyunları okuyabilir,
   kullanıcı açıkça seçmedikçe oraya yazmaz. (F:'de eksik bir VS 2026 kurulumu var — kullanılmaz, bkz. Araçlar.)
2. **SVG yok** (görsel, ikon, ikon kütüphanesi). İkon fontu veya PNG/WebP 1x/2x/3x; grafikler canvas.
3. **Sistem fontu yok.** Sora (başlık) + Manrope (metin), OFL, gömülü. Türkçe glif zorunlu.
4. **Dikdörtgen kutu yok.** 16–28px yarıçap, cam yüzeyler, çizgisiz ayrım (docs/DESIGN_SYSTEM.md).
5. **Gizli bilgi Git'e girmez.** `.env`, anahtarlar, `keys/` gitignore'da; commit öncesi `pnpm secrets`.
6. **Kullanıcı verisine saygı.** Oyun dosyası silinmez/değiştirilmez; geri yüklemeden önce güvenlik yedeği;
   performans modu yalnızca kullanıcının seçtiği süreçleri kapatır, korumalı listeye dokunmaz.
7. **Lisans disiplini.** Her bağımlılık/font/ikon/görsel `docs/THIRD_PARTY.md`'de.
8. **Önce plan, sonra uygulama.** Onay beklenmez; fazlar otomatik ilerler (prompt §4).

## Yollar

| Ne                        | Yol                                                                                                                |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| Proje (git deposu)        | `D:\OyunArsivim\app`                                                                                               |
| Rust (rustup/cargo)       | `D:\OyunArsivim\tools\rustup`, `D:\OyunArsivim\tools\cargo` (kullanıcı ortam değişkeni `CARGO_HOME`/`RUSTUP_HOME`) |
| Makineye özel cargo ayarı | `D:\OyunArsivim\tools\cargo\config.toml` (linker + LIB/INCLUDE → VS 2022 BT, `VSLANG=1033`)                        |
| pnpm store                | `D:\OyunArsivim\tools\pnpm-store` (`pnpm-workspace.yaml` → `storeDir`)                                             |
| VS 2022 Build Tools (C++) | `D:\Tools\VS2022BT` (MSVC 14.44.35207) + Windows SDK 10.0.19041 (`C:\Program Files (x86)\Windows Kits\10`)         |
| Updater imza anahtarı     | `D:\OyunArsivim\keys\updater.key` (+ `.pub`, `updater.password`) — GitHub Secrets'ta da var                        |
| Ekran görüntüleri         | `review/faz-XX/`                                                                                                   |
| WiX (tauri indirir)       | `%LOCALAPPDATA%\tauri\WixTools314`                                                                                 |

Bash'te araçlar için: `. scripts/dev-env.sh` (PATH + CARGO_HOME/RUSTUP_HOME).

## Araç sürümleri

Node 24.15.0 · pnpm 11.9.0 · Rust 1.99.0 (stable, MSVC) · Tauri 2.12.1 (CLI + crate) · Git 2.54 · gh 2.96 ·
WebView2 154 · VS 2022 Build Tools 17.x.

## Teknoloji yığını

Tauri 2 · React 19 + TypeScript strict + Vite 8 · Tailwind CSS + CSS değişkenleri · Framer Motion · Zustand ·
TanStack Query/Virtual · Chart.js (canvas) · SQLite (rusqlite, migration'lı) · winreg/windows/sysinfo/notify/zip/image ·
specta ile Rust→TS tip üretimi · Vitest + RTL · cargo test · WebdriverIO + tauri-driver (E2E).

## Klasör yapısı

```
src/                 React arayüz (pages/, components/, design/, stores/, ipc/, i18n/, assets/)
src/ipc/             Tip güvenli IPC sarmalayıcıları (arayüz dosya sistemine asla doğrudan erişmez)
src-tauri/src/       Rust: detectors/, launcher, tracker, backup, performance, artwork, db, recommendations,
                     settings, startup, logging
src-tauri/tests/     Entegrasyon testleri + fixtures/
e2e/                 Uçtan uca testler
scripts/             Yardımcı betikler (secret-scan, set-version, dev-env)
docs/                PLAN, SPEC, ARCHITECTURE, DESIGN_SYSTEM, PLATFORMS, QA, THIRD_PARTY, PROGRESS
.claude/             aitmpl agent/skill/command'ları + proje skill'leri (full-check, phase-close, add-detector, design-review)
```

## Komutlar

| Amaç                 | Komut                                                                                                   |
| -------------------- | ------------------------------------------------------------------------------------------------------- |
| Geliştirme           | `pnpm tauri dev`                                                                                        |
| Tam kontrol          | `bash .claude/skills/full-check/scripts/full-check.sh [--build]` veya `pnpm check`                      |
| Arayüz testleri      | `pnpm test`                                                                                             |
| Rust testleri        | `pnpm rust:test`                                                                                        |
| Lint / tip           | `pnpm lint`, `pnpm typecheck`, `pnpm rust:lint`, `pnpm rust:fmt`                                        |
| Release build        | `pnpm tauri build` → `src-tauri/target/release/bundle/{nsis,msi}`                                       |
| Gizli bilgi taraması | `pnpm secrets`                                                                                          |
| Sürüm yükseltme      | `bash .claude/skills/phase-close/scripts/bump-version.sh 0.X.0`                                         |
| Faz yayını           | `bash .claude/skills/phase-close/scripts/release.sh NN ad 0.X.0 "feat: ..."`                            |
| Ekran görüntüsü      | `powershell -File .claude/skills/design-review/scripts/capture-window.ps1 -Phase NN -Name ad [-Launch]` |

## Kod standartları

- TypeScript strict + `noUncheckedIndexedAccess`; `any` yok; ESLint `strictTypeChecked`, `--max-warnings 0`.
- Rust: `unwrap()`/`expect()` yalnızca testlerde; `thiserror` (modül hataları) + `anyhow` (uygulama sınırı);
  `clippy -D warnings`, `cargo fmt`.
- İsimlendirme: Rust `snake_case` modül/fonksiyon, `PascalCase` tip; TS bileşen `PascalCase.tsx`,
  hook `useX.ts`, yardımcı `camelCase.ts`; IPC komutları `snake_case` (`list_games`), olaylar `alan://olay`
  (`startup://progress`).
- Yorumlar "neden"i anlatır. Bileşenler küçük ve tek sorumlu. Kullanıcıya görünen metinler Türkçe (i18n).
- Tasarım değerleri yalnızca token'lardan (docs/DESIGN_SYSTEM.md); sabit renk/süre yazılmaz.
- Commit: Conventional Commits (`feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`).

## GitHub

- Depo: `X1NPAR1/oyun-arsivim` (private) — https://github.com/X1NPAR1/oyun-arsivim
- Dallar: `main` her zaman çalışır; faz işleri `faz/NN-kisa-ad`, testler geçince `--no-ff` ile main'e.
- Etiketler: Faz N → `v0.N.0`, Faz 18 → `v1.0.0`. 1.0 öncesi release'ler pre-release.
- Secrets: `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
- İş akışları: `.github/workflows/ci.yml` (her push/PR), `release.yml` (`v*` etiketi).
