# Değişiklik Günlüğü

Bu dosya [Keep a Changelog](https://keepachangelog.com/tr/1.1.0/) biçimini ve
[Semantik Sürümleme](https://semver.org/lang/tr/)'yi izler.

## [Yayınlanmadı]

## [0.4.0] - 2026-10-05

### Eklendi

- Steam oynama sürelerinin otomatik içe aktarılması (toplam ve son 2 hafta).
- Oyun süresi takibi: oyun açılınca oturum başlar, kapanınca biter; uygulama tepsideyken de çalışır.
- AFK tespiti (klavye, fare ve oyun kolu); uyku/askıya alma süresi sayılmaz.
- Kartlarda toplam oynama süresi ve bu haftaki süre.
- Ayrıntılı istatistikler: oyun seçimi, 7/30/90/365 gün, aktif ve AFK süre, günün saatleri, haftanın günleri, yoğunluk haritası, içgörüler, tüm zamanların en çok oynananları, oturum geçmişi.
- Ana sayfa: tam pencere sinematik sahne, yavaşça yakınlaşan tam ekran oyun görselleri, yumuşak geçişler, haftalık özet.
- Günlük dosyaları ve Ayarlar'da "Günlük klasörünü aç".

### Değişti

- Ana sayfa görselleri artık kırpılmıyor (16:9 mağaza görselleri, yüksek çözünürlük).
- Geçişler sırasında art arda komutlar yok sayılıyor; geçişler üst üste binmiyor.

## [0.3.0] - 2026-10-05

### Eklendi

- Steam (tüm kütüphane klasörleri) ve Epic Games oyunlarının otomatik bulunması.
- Kalıcı kütüphane: kaldırılan oyunlar silinmez, gri görünür ve "Silindi" etiketi alır; yeniden kurulunca geri gelir.
- Gerçek kapak, logo ve arka plan görselleri (yerel önbellek, internetsiz çalışır).
- Elle oyun ekleme: .exe seç veya pencereye sürükle; exe'nin ikonu logo olarak kullanılır.
- Kütüphanede arama, kurulu/silinen ve platform filtreleri, kart eğim/parlama efektleri.
- Ana sayfa vitrini: oyunun görseliyle sinematik arka plan ve sonsuz logo şeridi; ←/→ tuşları, fare tekerleği ve tıklama ile gezinme.
- Sağ tık menüsü: Oyna, Masaüstüne kısayol oluştur, Oyun klasörünü aç, Oyunu kaldır, Yeniden yükle, Kütüphaneden çıkar.
- Sistem tepsisi: pencere kapatılınca arka planda çok az bellekle çalışma; tepsiden açma, tarama ve çıkış.
- Windows açılışında arka planda (tepside) başlatma seçeneği.
- Özgün uygulama logosu ve ikonları.

### Düzeltildi

- Klasörü silinmiş ama manifesti kalmış oyunlar artık kurulu görünmüyor.

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
