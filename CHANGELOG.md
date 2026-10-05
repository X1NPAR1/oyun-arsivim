# Değişiklik Günlüğü

Bu dosya [Keep a Changelog](https://keepachangelog.com/tr/1.1.0/) biçimini ve
[Semantik Sürümleme](https://semver.org/lang/tr/)'yi izler.

## [Yayınlanmadı]

## [0.1.0] - 2026-10-05

### Eklendi

- Tauri 2 + React 19 + TypeScript (strict) + Vite proje iskeleti.
- Rust ↔ arayüz IPC bağlantısını doğrulayan `app_info` komutu ve testleri.
- Kalite araçları: ESLint (strict type-checked), Prettier, Vitest + Testing Library, clippy, rustfmt.
- Gizli bilgi tarayıcısı (`pnpm secrets`).
- Windows kurulum dosyaları: NSIS (Türkçe/İngilizce) ve MSI (Türkçe).
- GitHub Actions: her push'ta CI (lint, test, build), etikette imzalı release.
- Proje dokümantasyonu: plan, işlevsel tanım, mimari, tasarım sistemi, platformlar, QA, lisanslar.
- Claude Code proje skill'leri: tam kontrol, faz kapanışı, dedektör ekleme, tasarım incelemesi.

### Güvenlik

- Katı Content Security Policy ve en az yetkili capability yapılandırması.
