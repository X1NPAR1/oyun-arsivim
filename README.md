# Oyun Arşivim

**Windows için akıllı oyun kütüphanesi ve başlatıcı.** Bilgisayarındaki tüm oyunları otomatik bulur,
tek ve şık bir kütüphanede toplar, oynama süreni izler, kayıt dosyalarını yedekler, oyun açılınca sistemi
performans moduna alır ve sana ne oynayacağını önerir.

> 🚧 Geliştirme aşamasında — güncel durum için [docs/PROGRESS.md](docs/PROGRESS.md).

## Özellikler (yol haritası)

- **Otomatik tarama:** Steam, Epic Games, GOG Galaxy, EA App, Ubisoft Connect, Battle.net, Xbox/Microsoft Store ve elle eklenen oyunlar
- **Sinematik kütüphane:** kapak, arka plan ve logo görselleri; akıcı animasyonlar
- **Tek tıkla başlatma** ve **oynama süresi takibi**
- **Kayıt dosyası yedekleme** ve güvenli geri yükleme
- **Performans modu:** seçtiğin arka plan programlarını kapatır, oyun bitince her şeyi geri alır
- **Akıllı öneriler:** "Bugün ne oynasam?", yarım kalanlar, unutulanlar
- **İstatistikler:** günlük/haftalık/aylık grafikler, ısı haritası

## Kurulum

[Releases](https://github.com/X1NPAR1/oyun-arsivim/releases) sayfasından `Oyun Arşivim_x.y.z_x64-setup.exe`
dosyasını indirip çalıştır. Windows 10 (1809+) veya Windows 11 gerekir; WebView2 yoksa kurulum otomatik yükler.

## Geliştirme

Gereksinimler: Node 22+, pnpm 11, Rust (stable, MSVC), Visual Studio Build Tools (C++).

```bash
pnpm install
pnpm tauri dev      # geliştirme
pnpm check          # tüm test + lint
pnpm tauri build    # kurulum dosyaları
```

Ayrıntılar: [CLAUDE.md](CLAUDE.md), [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [CONTRIBUTING.md](CONTRIBUTING.md).

## Sık sorulanlar

**Oyun dosyalarıma dokunuyor mu?** Hayır. Oyun dosyaları asla silinmez veya değiştirilmez; geri yüklemeden önce mevcut kaydın yedeği alınır.

**İnternet gerekli mi?** Hayır. Görseller ilk kez indirildikten sonra önbellekten çalışır.

## Lisans

[MIT](LICENSE). Üçüncü taraf lisansları: [docs/THIRD_PARTY.md](docs/THIRD_PARTY.md).
