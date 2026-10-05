# Değişiklik Günlüğü

Bu dosya [Keep a Changelog](https://keepachangelog.com/tr/1.1.0/) biçimini ve
[Semantik Sürümleme](https://semver.org/lang/tr/)'yi izler.

## [Yayınlanmadı]

## [0.2.0] - 2026-10-05

### Eklendi

- Tasarım sistemi: nötr grafit renk paleti, tipografi, yarıçap, gölge, cam ve hareket token'ları. Arayüz renksizdir; renk oyun görsellerinden gelir.
- Gömülü Sora ve Manrope fontları (Türkçe karakter desteğiyle), Phosphor ikon fontu.
- Özel başlık çubuğu ve uygulama stilinde pencere düğmeleri.
- Windows 11'de Mica arka planı; Windows 10'da derinlikli yedek zemin ve yuvarlak pencere köşeleri.
- Kenarlardan boşluklu, yüzen cam kenar menüsü ve kayan aktif öğe ışıması.
- Yönlü sayfa geçişleri (ileri sağdan, geri soldan) ve sırayla kayan kart girişleri.
- Animasyon seviyesi ayarı (Sistem / Tam / Azaltılmış / Kapalı); Windows "animasyonları azalt" ayarına uyum.
- Ana sayfa, Kütüphane, İstatistikler, Yedekler ve Ayarlar sayfa iskeletleri.

### Değişti

- Pencere içerik hazır olunca gösteriliyor (açılışta beyaz flaş yok).

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
