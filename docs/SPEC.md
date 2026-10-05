# SPEC — İşlevsel tanım ve kullanıcı akışları

## 1. Açılış (splash)

- Çerçevesiz, yuvarlatılmış, gölgeli küçük pencere; PNG logo ölçeklenme + parlama ile belirir.
- Canvas degrade dönen çember + %0→%100 sayaç. Altında kayarak değişen durum metni.
- İlerleme Rust `startup` modülünden gelir; her adımın ağırlığı vardır (DB %10, ayarlar %5, platform başına
  tarama, görseller, save kontrolü). "Az kaldı..." > %80, "Bitmek üzere..." > %95, "Hazır!" = %100.
- İlk açılış dışında tarama arka plana alınır; splash hedefi 1,5–3 sn.
- Hata: mesaj + "Tekrar dene" + "Günlüğü aç". Ana pencere içerik hazır olunca görünür (beyaz flaş yok).

## 2. Oyun tarama

- Dedektörler: Steam, Epic, GOG, EA, Ubisoft, Battle.net, Xbox, elle, klasör önerileri.
- Tekilleştirme anahtarı: (normalleştirilmiş ad + kurulum klasörü) veya platform kimliği eşleşmesi;
  birden fazla kaynak tek kayıtta `sources[]` olarak tutulur.
- Kaldırılan oyun: `installed = false`, geçmiş korunur. Klasör izleme ile yeni kurulumlar otomatik eklenir.
- Akış: Kütüphane → "Oyun ekle" → exe seç / sürükle-bırak → ad, kapak, parametreler → Kaydet.

## 3. Görseller

Steam CDN (kapak 600×900, header, hero, logo) → SteamGridDB (anahtar varsa) → yedek kapak üretici
(exe ikonu + baskın renk degradesi + ad). Kullanıcı her görseli değiştirebilir. Önbellek + thumbnail +
blur placeholder; çevrimdışı tam çalışma.

## 4. Başlatma ve süre

- Steam `steam://rungameid/<appid>`, Epic `com.epicgames.launcher://apps/<ns>%3A<id>%3A<app>?action=launch&silent=true`,
  GOG Galaxy `goggalaxy://openGameView/<id>` veya doğrudan exe, EA/Ubisoft/Battle.net protokol veya exe, Xbox
  `shell:AppsFolder\<AUMID>`, elle: exe + çalışma dizini + parametreler.
- Süreç izleme: başlatmadan sonra 120 sn içinde kurulum klasörü altındaki exe'den doğan süreç(ler) oyun sayılır;
  hepsi kapanınca oturum biter. Uyku/hazırda bekleme aralıkları ve 15 dk üzeri boşta (oyun arka planda) düşülür.
- Tepsi modu: pencere kapalıyken de izleme sürer (ayara bağlı).

## 5. Save yedekleme

- Ludusavi manifest (YAML) → oyun başına save yolları (`<home>`, `<storeUserId>`, registry vb. yer tutucuları).
- Yedek: `<yedek klasörü>/<oyun>/<YYYYMMDD-HHMMSS>.zip` + `manifest.json` (dosya listesi + SHA-256).
- Otomatik: oyun kapanınca / zamanlanmış; son N yedek tutulur (varsayılan 10).
- Geri yükleme: oyun çalışıyorsa engellenir → mevcut save'in güvenlik yedeği → zip doğrulama → çıkarma.

## 6. Performans modu

- Oyun bazında aç/kapa. Başlatınca: kullanıcı listesindeki süreçleri kapat (önce nazik `WM_CLOSE`, sonra
  zaman aşımıyla sonlandırma), güç planını Yüksek Performans'a al, isteğe bağlı bildirim sessize alma.
- Kapanınca: güç planı geri, süreçleri (istenirse) yeniden başlat. Durum `perf-state.json`'a yazılır;
  çökme sonrası açılışta geri alınır. Korumalı liste: sistem/güvenlik süreçleri (csrss, lsass, winlogon,
  explorer, MsMpEng, svchost, dwm, services, smss, wininit, audiodg, Oyun Arşivim'in kendisi…).

## 7. Akıllı özellikler

- "Bugün ne oynasam?": puan = yarım kalma (son 30 gün oynanmış, toplam < tahmini) + unutulmuşluk
  (son oynama > 60 gün) + hiç oynanmamış + tür/platform alışkanlık benzerliği + rastgelelik; kart çevirme sunumu.
- İçgörüler: aylık toplam, en çok oynanan, hiç açılmayan sayısı, haftalık değişim %.
- Disk: boyut, uzun süredir oynanmayan büyük oyunlar → platform kaldırma ekranını aç (kendisi silmez).
- Koleksiyonlar: Son oynananlar, Favoriler, Yarım kalanlar, Hiç oynanmamışlar, Platforma göre + kullanıcı.
- Ctrl+K komut paleti: oyunlar, platformlar, koleksiyonlar, komutlar; fuzzy arama.

## 8. Sayfalar

Ana sayfa · Kütüphane (ızgara/liste, sırala: ad/son oynama/süre/eklenme/boyut; filtre: platform/kurulu/koleksiyon)
· Oyun detay · İstatistikler · Yedekler · Ayarlar.

## 9. Ayarlar

Genel (açılışta başlat, tepsiye küçült, dil) · Kütüphane (platformlar, ek klasörler, otomatik tarama, gizlenenler)
· Görünüm (tema, dinamik renk, animasyon seviyesi, kart boyutu, cam yoğunluğu) · Yedekleme · Performans modu
· Entegrasyonlar (SteamGridDB) · Veri (dışa/içe aktar, önbellek temizle, log klasörü, sıfırlama) · Güncellemeler · Hakkında.
Tüm ayarlar anında uygulanır ve kalıcıdır.
