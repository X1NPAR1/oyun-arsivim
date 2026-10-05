# PROGRESS — Faz bazlı checklist

Durum: ⬜ başlamadı · 🟨 sürüyor · ✅ tamamlandı

## Faz 1 — Temel altyapı ✅ (v0.1.0)

- [x] prompt.md okundu, plan yapıldı
- [x] Rust 1.99 (D:), VS 2022 BT (D:) bağlandı, WebView2 154 doğrulandı, Node/pnpm/git/gh mevcut
- [x] Tauri 2 + React 19 + TS strict + Vite iskeleti, SVG'ler kaldırıldı
- [x] ESLint (strictTypeChecked), Prettier, Vitest + RTL, clippy, rustfmt
- [x] `app_info` IPC komutu + Rust ve arayüz testleri
- [x] Release build: NSIS + MSI (tr-TR)
- [x] Release uygulaması gerçekten açıldı, ekran görüntüsü `review/faz-01/`
- [x] aitmpl bileşenleri kuruldu, 4 proje skill'i yazıldı
- [x] Hafıza/doküman dosyaları
- [x] Updater imza anahtarı (`D:\OyunArsivim\keys`)
- [x] GitHub deposu (private), secrets, CI yeşil, v0.1.0 release

## Faz 2 — Tasarım sistemi ✅ (v0.2.0)

- [x] Token'lar: `src/design/tokens.css` (renk, tipografi, yarıçap, gölge, cam, hareket) + `motion.ts`
- [x] Tailwind 4 teması token'lardan
- [x] Sora + Manrope gömülü (OFL), Türkçe glifler doğrulandı
- [x] Phosphor ikon fontu (yalnız woff2), tip güvenli `Icon`
- [x] Özel başlık çubuğu (sürükleme, küçült/büyüt/kapat)
- [x] Win11 Mica / Win10 CSS yedeği + yuvarlak pencere köşeleri
- [x] Yüzen cam sidebar, kayan aktif ışıma
- [x] Yönlü sayfa geçişleri (ileri sağdan, geri soldan), stagger kart girişi
- [x] Animasyon seviyesi (sistem/tam/azaltılmış/kapalı) + prefers-reduced-motion
- [x] Testler (16 arayüz + 5 Rust), gerçek pencerede ekran görüntüleri `review/faz-02/`
- [x] Kullanıcı geri bildirimiyle nötr tasarıma revizyon (`review/faz-02/v3-*.png`)
- [x] Release + CI

## Faz 3 — Gerçek kütüphane, vitrin, tepsi ✅ (v0.3.0)

- [x] VDF/ACF ayrıştırıcı, Steam dedektörü (çoklu kütüphane, filtre), Epic dedektörü (DLC/motor filtresi)
- [x] Klasörü silinmiş oyunlar kurulu sayılmıyor (gerçek GTA V vakası)
- [x] SQLite + migration'lar (oyunlar, ayarlar); "Silindi" durumu, geri gelme
- [x] Görsel önbelleği: Steam kapak/logo/hero, mağaza API yedeği, exe ikonu (2 yöntem)
- [x] Kütüphane sayfası: gerçek kartlar, arama (Türkçe duyarsız), kurulu/silinen/platform filtreleri, tilt/parlama
- [x] Elle exe ekleme: dosya seçici + sürükle-bırak, exe ikonu logo olur
- [x] Ana sayfa vitrini: oyun görseli arka plan, sonsuz logo şeridi; ←/→, tekerlek, tıklama, otomatik akış
- [x] Sağ tık menüsü: Oyna, Masaüstüne kısayol, Klasörü aç, Kaldır, Yeniden yükle, Kütüphaneden çıkar
- [x] Sistem tepsisi, düşük RAM (6 MB), tek örnek, Windows açılışında arka planda başlatma
- [x] Ayarlar: Genel (açılışta başlat, tepside çalış), Görünüm — SQLite'ta kalıcı
- [x] Özgün logo ve uygulama ikonları
- [x] Testler: Rust 37, arayüz 27; gerçek sistem taraması; uçtan uca tepsi/kısayol/autostart testleri

## Faz 4 — Oynama süresi, takip, istatistikler, sinematik ana sayfa ✅ (v0.4.0)

- [x] Steam sürelerinin içe aktarılması (toplam + son 2 hafta, çoklu hesap)
- [x] Oyun süreci takipçisi (tepsideyken de), AFK (klavye/fare + gamepad), uyku düşme, kesinti toleransı
- [x] Oturumlar + saatlik aktif/AFK kovaları (migration v3)
- [x] Kartlarda toplam süre + bu hafta / son 2 hafta
- [x] İstatistikler: oyun seçici, aralık, KPI'lar, günlük aktif/AFK, saat ve gün dağılımı, ısı haritası, içgörüler, tüm zamanlar sıralaması, oturum tablosu
- [x] Ana sayfa: tam pencere sahne, 16:9 görseller, 10 sn merkez yakınlaşma, 1,8 sn çapraz geçiş, komut kilidi, haftalık özet şeridi
- [x] Tam ekran (1920×1080) düzen kontrolü; içerik geniş ekranda ortalı
- [x] Dönen günlük dosyaları + "Günlük klasörünü aç"
- [x] Testler: Rust 49, arayüz 40; gerçek takip uçtan uca testi

## Faz 5 — Performans Merkezi ⬜

## Faz 5b — Splash ⬜

## Faz 6 — Veritabanı, ayarlar, günlükleme ⬜

## Faz 7 — Elle oyun ekleme ⬜

## Faz 8 — Diğer dedektörler, tekilleştirme, izleme ⬜

## Faz 9 — Görsel sistemi (SteamGridDB, yedek kapak) ⬜

## Faz 10 — Kütüphane sayfası ⬜

## Faz 11 — Oyun detay + paylaşılan öğe geçişi ⬜

## Faz 12 — Başlatma, süre takibi, tepsi ⬜

## Faz 13 — Save yedekleme ⬜

## Faz 14 — Öneriler, koleksiyonlar, komut paleti ⬜

## Faz 15 — İstatistikler ⬜

## Faz 16 — Ayarlar + i18n ⬜

## Faz 17 — Otomatik güncelleme ⬜

## Faz 18 — Optimizasyon, erişilebilirlik, dayanıklılık ⬜

## Faz 19 — Cilalama, v1.0.0 ⬜
