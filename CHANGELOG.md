# Değişiklik Günlüğü

Bu dosya [Keep a Changelog](https://keepachangelog.com/tr/1.1.0/) biçimini ve
[Semantik Sürümleme](https://semver.org/lang/tr/)'yi izler.

## [Yayınlanmadı]

## [0.9.0] - 2026-10-06

### Eklendi

- Aynı anda tek oyun: bir oyun açıkken başka bir oyun başlatılmak istenirse "Şu an X oynuyorsunuz" penceresi açılır; isterseniz açık oyun kapatılıp yenisi başlatılır.
- Başlık çubuğunun sağ üstünde oval "Oyun adı · Oynanıyor" göstergesi (açılırken "Açılıyor"); tıklayınca oyunun detay sayfası açılır.
- Performans → Geçici dosyalar: %TEMP% ve `C:\Windows\Temp` temizliği, temizlenebilir boyut gösterimi. Son bir saatte değişen ve kullanımdaki dosyalara dokunulmaz.

### Düzeltildi

- Minecraft Launcher "bulunamadı" hatasıyla açılmıyordu: Xbox paketlerinin uygulama kimliği yanlış okunuyordu (`<Applications>` etiketi `<Application>` sanılıyordu).
- Sürüm notları GitHub sürüm sayfasına artık CHANGELOG'dan aktarılıyor.

## [0.8.0] - 2026-10-06

### Eklendi

- Oyun detay sayfası: tam ekran sahne görseli, kütüphane kartından detaya kesintisiz paylaşılan öğe geçişi, Oyna düğmesi, son 30 günün grafiği, oturum listesi, oyun bilgileri, oyuna özel performans profili ve kişisel notlar. Esc ile geri dönülür.
- Dinamik vurgu rengi: detay sayfası oyunun görselinden çıkarılan rengi kullanır.
- Koyu logolar (ör. Counter-Strike 2) koyu sahnede otomatik olarak beyaz silüet şeklinde çizilir.

### Değişti

- Ana sayfada oyunlar arası geçiş kilidi 2 saniyeden 0,5 saniyeye indirildi.
- Animasyonlar varsayılan olarak tam açık; Windows'ta animasyonlar kapalı olsa bile uygulama hareketli kalır (Ayarlar'dan değiştirilebilir).

### Düzeltildi

- Görsel renk ve logo analizi artık arka uçta yapılıyor; `asset:` görsellerinde tuval okuma engeli nedeniyle çalışmayan renk/logo tespiti düzeldi.

## [0.7.0] - 2026-10-06

### Eklendi

- GOG Galaxy, EA app, Ubisoft Connect, Battle.net, Rockstar Games, Riot Games (VALORANT, League of Legends…) ve Xbox / Microsoft Store oyunlarının otomatik bulunması.
- Aynı oyun birden fazla platformdan bulunursa tek kayıt gösterilir.
- Yeni kurulan veya silinen oyunlar kütüphaneye kendiliğinden yansır (platform klasörleri izlenir).
- Xbox oyunlarında resmi paket logoları, Riot oyunlarında oyunun kendi simgesi.

## [0.6.0] - 2026-10-06

### Eklendi

- Açılış ekranı: logo animasyonu, dönen ilerleme çemberi, gerçek yüzde ve adım adım durum metinleri.
- Açılışta bir sorun olursa anlaşılır hata mesajı, "Tekrar dene" ve "Günlüğü aç".

### Değişti

- Ana pencere içerik hazır olunca açılır; açılış ekranı yumuşakça kapanır (boş/beyaz ekran yok).
- Ana sayfada oyunlar arası geçiş kilidi 0,5 saniyeye indirildi.

## [0.5.0] - 2026-10-05

### Eklendi

- Performans merkezi: canlı işlemci (çekirdek başına), bellek, ekran kartı kullanımı ve video belleği; en çok bellek kullanan uygulamalar.
- Tek tıkla RAM temizliği, otomatik RAM temizliği (ayarlanabilir eşik) ve bekleme belleğini boşaltma (yönetici onayıyla).
- Güç planı seçimi; oyun sırasında en yüksek plana geçip oyun bitince geri dönme.
- Windows oyun ayarları: Oyun Modu, arka plan kaydı (Game DVR), Xbox Game Bar, tam ekran iyileştirmeleri, fare ivmesi, pencere animasyonları, bildirimler, donanım hızlandırmalı GPU zamanlaması, ağ kısıtlaması, oyun zamanlama önceliği. "Önerilenleri uygula" ve "Varsayılanlara dön".
- Oyuna özel performans modu: yüksek işlemci önceliği, güç planı, açılışta RAM temizliği, bildirimleri susturma, seçili uygulamaları kapatma. Oyun bitince her şey otomatik geri alınır; uygulama çökse bile bir sonraki açılışta geri alınır.
- Oyun kartı sağ tık menüsünde "Performans modu…".

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
