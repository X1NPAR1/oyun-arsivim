# PLATFORMS — Tespit ve başlatma yöntemleri

> **Doğrulandı (2026-10-05, Faz 3):** Steam ve Epic bu makinede gerçek verilerle test edildi. Ek kural: manifest olsa bile
> kurulum klasörü yoksa oyun kurulu sayılmaz. Steam'de oyun olmayan yazılımlar (ör. Wallpaper Engine 431960) elenir.
> Yeni Steam oyunlarının görselleri karma adreslerdedir → mağaza API'si `appdetails` `header_image` yedeği.
> Kaldırma: Steam `steam://uninstall/<appid>`, yükleme `steam://install/<appid>`; Epic yükleme `…?action=install`,
> kaldırma için dış adres yok → `com.epicgames.launcher://store/library`.

## Steam

- **Kurulum yolu:** `HKCU\Software\Valve\Steam\SteamPath` (yedek: `HKLM\SOFTWARE\WOW6432Node\Valve\Steam\InstallPath`).
- **Kütüphaneler:** `<steam>\steamapps\libraryfolders.vdf` (KeyValues metin formatı) → her `"path"` girdisi.
- **Oyunlar:** `<lib>\steamapps\appmanifest_<appid>.acf` → `appid`, `name`, `installdir`, `SizeOnDisk`,
  `StateFlags` (4 = tam kurulu), `LastPlayed`. Kurulum klasörü: `<lib>\steamapps\common\<installdir>`.
- **Filtre:** Steamworks Common Redistributables (228980), Proton/Steam Linux Runtime, SteamVR, dedicated server'lar,
  "Soundtrack"/"Tool" adları; bilinen appid listesi + ad desenleri.
- **Başlatma:** `steam://rungameid/<appid>`. **Görseller:** `https://cdn.cloudflare.steamstatic.com/steam/apps/<appid>/`
  `library_600x900_2x.jpg`, `header.jpg`, `library_hero.jpg`, `logo.png`.
- **Sınırlama:** Steam hesabındaki kurulu olmayan oyunlar listelenmez (yalnız yerel).

## Epic Games

- **Manifestler:** `%ProgramData%\Epic\EpicGamesLauncher\Data\Manifests\*.item` (JSON): `DisplayName`,
  `InstallLocation`, `LaunchExecutable`, `CatalogNamespace`, `CatalogItemId`, `AppName`, `InstallSize`,
  `bIsIncompleteInstall`. DLC'ler `MainGameAppName != AppName` ile elenir.
- **Başlatma:** `com.epicgames.launcher://apps/<ns>%3A<itemId>%3A<appName>?action=launch&silent=true`.

## GOG Galaxy

- **Kayıt defteri:** `HKLM\SOFTWARE\WOW6432Node\GOG.com\Games\<gameId>` → `gameName`, `path`, `exe`, `launchCommand`, `workingDir`.
- **Yedek:** `%ProgramData%\GOG.com\Galaxy\storage\galaxy-2.0.db` (salt okunur açılır).
- **Başlatma:** Galaxy kuruluysa `goggalaxy://openGameView/<gameId>` yerine doğrudan exe (Galaxy'siz kurulumlar da çalışır).

## EA App

- **Kayıt defteri:** `HKLM\SOFTWARE\WOW6432Node\Electronic Arts\<Oyun>` ve `HKLM\SOFTWARE\WOW6432Node\EA Games\<Oyun>` → `Install Dir`.
- **Ek:** `%ProgramData%\EA Desktop\InstallData\*\installerdata.xml`; `__Installer\installerdata.xml` içinde contentID.
- **Başlatma:** `origin2://game/launch?offerIds=<id>` (EA App uyumlu) veya exe.
- **Sınırlama:** Yerel manifest şifreli olabilir; tespit kayıt defteri + kurulum klasörüne dayanır.

## Ubisoft Connect

- **Kayıt defteri:** `HKLM\SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs\<gameId>` → `InstallDir`.
- **Ad:** `HKLM\...\Uninstall\Uplay Install <gameId>` → `DisplayName`.
- **Başlatma:** `uplay://launch/<gameId>/0`.

## Battle.net

- **Kayıt defteri:** `HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*` içinde
  `Publisher = Blizzard Entertainment` ve `InstallLocation`.
- **Ek:** `%ProgramData%\Battle.net\Agent\product.db` (protobuf) — ürün kodları.
- **Başlatma:** `battlenet://<ürünKodu>` (ör. `WoW`, `Pro`, `OSI`), yoksa Battle.net.exe `--exec="launch <kod>"`.

## Xbox / Microsoft Store

- **Tespit:** WinRT `PackageManager.FindPackagesByUserSecurityId("")` (yönetici gerekmez); `InstalledPath` içinde
  `MicrosoftGame.config` olan paketler oyundur. Ad: `ShellVisuals DefaultDisplayName`; logo: `Square480x480Logo` → `Square150x150Logo` → `StoreLogo` (ölçek varyantları).
- **Başlatma:** `shell:AppsFolder\<PackageFamilyName>!<AppId>`.
- **Sınırlama:** `WindowsApps` klasörü erişime kapalıdır; exe ikonu ve boyut alınamayabilir; bazı oyunlar
  "Game" kategorisinde işaretli değildir — `MicrosoftGame.config` olmayan paketler oyun sayılmaz.

## Riot Games (Faz 7 — prompt listesinde yoktu, bu makinede VALORANT kurulu olduğu için eklendi)

- **Tespit:** `%ProgramData%\Riot Games\Metadata\<ürün>.<hat>\<ürün>.<hat>.product_settings.yaml` →
  `product_install_full_path`. Ürünler: valorant, league_of_legends, bacon (LoR), lion (2XKO); istemci ve TFT (LoL içinde) elenir.
- **Başlatma:** `RiotClientInstalls.json` → `rc_default` istemcisi + `--launch-product=<ürün> --launch-patchline=<hat>`.
- **Simge:** istemci değil, oyun klasöründeki kendi exe'si.

## Rockstar Games Launcher

- **Tespit:** `HKLM\SOFTWARE\WOW6432Node\Rockstar Games\<oyun>` → `InstallFolder` (Launcher, Social Club, SDK elenir).
- **Başlatma:** klasördeki `Play*.exe`, yoksa kökteki en büyük exe. Steam/Epic sürümlerinde kayıt boştur → o platformlardan gelir.

## Tekilleştirme ve klasör izleme (Faz 7)

- Aynı kurulum klasörü birden fazla platformdan gelirse öncelik: Steam > Epic > GOG > Ubisoft > Battle.net > Riot > Rockstar > EA > Xbox > Elle.
  Gerçek vaka: EA app'in Battlefield 1 kaydı Steam klasörünü gösteriyor → tek kayıt (Steam).
- `notify` ile izlenen klasörler: Steam `steamapps` (tüm kütüphaneler), Epic manifestleri, Riot metadata. Yalnız manifest dosyaları
  (`appmanifest_*.acf`, `*.item`, `*.product_settings.yaml`) tetikler; son olaydan 4 sn sonra tek tarama.

## Doğrulama (Faz 7, bu makine)

| Platform                   | Sonuç                                                                     |
| -------------------------- | ------------------------------------------------------------------------- |
| GOG / Ubisoft / Battle.net | Kurulu değil → boş liste; sahte kayıt defteriyle test edildi              |
| EA app                     | Battlefield 1 kaydı bulundu → Steam kopyasıyla tekilleştirildi            |
| Rockstar                   | RDR2 / GTA V kayıtlarında klasör yok (Steam/Epic) → doğru şekilde atlandı |
| Riot                       | VALORANT bulundu, oyun simgesi exe'den                                    |
| Xbox                       | Minecraft for Windows, Minecraft Launcher, Roblox; paket logoları         |

## Elle ekleme ve klasör tarama

- Elle: exe seçimi veya sürükle-bırak; ad exe'nin `FileDescription`/`ProductName` sürüm bilgisinden önerilir.
- Klasör tarama: seçili klasörlerde derinlik ≤ 3, `unins*`, `*setup*`, `crash*`, `UnityCrashHandler`, `vc_redist` vb. elenir;
  yalnız öneri olarak sunulur, onaysız eklenmez.

## Tekilleştirme

Aynı `install_dir` (normalleştirilmiş, büyük/küçük harf duyarsız) veya aynı normalleştirilmiş ad + platform-kimliği
eşleşmesi → tek oyun, çoklu `game_sources`.
