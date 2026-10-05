# ARCHITECTURE

## Katmanlar

1. **Rust çekirdek** — sistem erişimi, tarama, veritabanı, izleme, yedekleme. Saf mantık (ayrıştırıcılar,
   süre hesabı, puanlama) I/O'dan ayrılır ve doğrudan birim test edilir; I/O kökleri enjekte edilebilir.
2. **IPC** — Tauri komutları (`#[tauri::command]`) ve olaylar. Tipler `specta`/`tauri-specta` ile
   `src/bindings.ts`'e üretilir (Faz 4); elle tip tekrarı yok.
3. **React arayüz** — sunum ve etkileşim. Dosya sistemine asla doğrudan erişmez; veri TanStack Query ile
   komutlardan, anlık durum Zustand'dan.

## Rust modülleri (hedef)

| Modül             | Sorumluluk                                                                     |
| ----------------- | ------------------------------------------------------------------------------ |
| `startup`         | Başlatma adımları, ağırlıklı ilerleme, `startup://progress` olayı, hata durumu |
| `db`              | SQLite bağlantı havuzu, migration çalıştırıcı, depo (repository) fonksiyonları |
| `settings`        | Tipli ayarlar, varsayılanlar, anında kalıcılık, değişiklik olayı               |
| `logging`         | `tracing` + günlük döndürme (7 dosya), kişisel veri maskeleme                  |
| `detectors/*`     | `GameDetector` trait + platform modülleri, tekilleştirme, `notify` izleyici    |
| `artwork`         | İndirme, önbellek, thumbnail, baskın renk, yedek kapak                         |
| `launcher`        | Platforma göre başlatma                                                        |
| `tracker`         | Süreç izleme, oturumlar, uyku/boşta düşme                                      |
| `backup`          | Ludusavi manifest, zip + hash, geri yükleme                                    |
| `performance`     | Süreç kapatma, güç planı, durum dosyası, geri alma                             |
| `recommendations` | Öneri puanlama, içgörüler                                                      |
| `tray`, `updater` | Sistem tepsisi, güncelleme                                                     |

## IPC komutları (mevcut + planlanan)

| Komut                                                                | Faz    | Dönüş                       |
| -------------------------------------------------------------------- | ------ | --------------------------- |
| `app_info`                                                           | 1      | `AppInfo { name, version }` |
| `startup_retry`, `open_log_dir`                                      | 3      | `()`                        |
| `get_settings`, `update_settings`                                    | 4      | `Settings`                  |
| `list_games`, `get_game`, `add_manual_game`, `update_game`, `rescan` | 5–6    | `Game[]`, `Game`            |
| `set_artwork`, `refresh_artwork`                                     | 7      | `Artwork`                   |
| `launch_game`                                                        | 10     | `SessionId`                 |
| `list_sessions`, `stats_summary`                                     | 10, 14 | …                           |
| `list_backups`, `backup_now`, `restore_backup`                       | 11     | …                           |
| `perf_profile_get/set`                                               | 12     | …                           |
| `recommendations`, `insights`, `collections_*`                       | 13     | …                           |
| `check_update`, `install_update`                                     | 16     | …                           |

## Faz 3–4'te eklenenler

- Modüller: `detectors/{steam,epic,steam_playtime,vdf}`, `db` (migration v1–v3), `library`, `artwork`, `icon`,
  `game_actions`, `settings`, `tray`, `tracker`, `playtime`, `logging`.
- Tablolar: `games` (+ `platform_minutes`, `platform_2wks_minutes`), `settings(key, value JSON)`,
  `sessions(game_id, started_at, ended_at, active_seconds, idle_seconds, open)`,
  `playtime_hourly(game_id, hour UTC, active_seconds, idle_seconds)`.
- Komutlar: `list_games`, `scan_library`, `add_manual_game`, `remove_manual_game`, `rename_game`, `launch_game`,
  `get_settings`, `update_settings`, `uninstall_game`, `install_game`, `create_desktop_shortcut`, `open_game_folder`,
  `playtime_hourly`, `playtime_sessions`, `open_log_dir`, `window_backdrop`, `app_info`.
- Arka plan iş parçacıkları: açılış taraması + görsel indirme, `oyun-takipci` (süre takibi).

## Faz 5 — Performans Merkezi

- `perf/monitor` (sysinfo + PDH), `perf/tweaks` (beyaz liste, SPI, `--admin-task`), `elevate` (ShellExecuteEx runas),
  `perf/memory`, `perf/power` (Power API), `perf/processes` (öncelik, kapatma, korumalı liste), `perf/boost` (profil uygula/geri al).
- Tablo: `perf_profiles(game_id, profile JSON)` (migration v4). Durum dosyası: `<yerel veri>/perf-state.json`.
- Komutlar: `perf_snapshot`, `perf_tweaks`, `perf_set_tweak`, `perf_restore_defaults`, `perf_clean_ram`, `perf_purge_standby`,
  `power_overview`, `power_set`, `perf_profile_get/set/recommended`, `perf_profiled_games`, `perf_close_suggestions`.

## Olaylar

`startup://progress {percent, stepKey, message}` · `startup://error {message, logPath}` · `scan://progress` ·
`library://changed` · `session://started|ended` · `task://progress {id, percent, message, cancellable}`.

## Veri modeli (şema v1 — Faz 4'te kesinleşir)

```sql
games(id INTEGER PK, title TEXT, sort_title TEXT, install_dir TEXT, exe_path TEXT, launch_args TEXT,
      installed INTEGER, size_bytes INTEGER, favorite INTEGER, hidden INTEGER, notes TEXT,
      perf_mode INTEGER, accent_color TEXT, added_at INTEGER, last_played_at INTEGER, total_seconds INTEGER)
game_sources(game_id FK, platform TEXT, platform_id TEXT, launch_uri TEXT, UNIQUE(platform, platform_id))
artwork(game_id FK, kind TEXT CHECK(kind IN('cover','hero','logo','icon')), path TEXT, source TEXT, blurhash TEXT)
sessions(id PK, game_id FK, started_at INTEGER, ended_at INTEGER, active_seconds INTEGER)
collections(id PK, name TEXT, smart_rule TEXT NULL) ; collection_games(collection_id, game_id)
backups(id PK, game_id FK, path TEXT, created_at INTEGER, size_bytes INTEGER, sha256 TEXT, kind TEXT)
settings(key TEXT PK, value TEXT JSON)
schema_migrations(version INTEGER PK, applied_at INTEGER)
```

## Olay akışları

- **Açılış:** main → `startup::run` (ayrı thread) → adım adım `startup://progress` → splash gösterir →
  bitince ana pencere `show()` + splash `close()`.
- **Oyun başlatma:** UI `launch_game` → launcher → tracker süreci bulur → `session://started` → perf modu
  uygulanır → süreç biter → oturum yazılır → perf geri alınır → otomatik yedek → `session://ended`.

## Güvenlik

- Capability: pencere başına en az yetki. CSP: `default-src 'self'`; görseller `asset:` protokolüyle
  yalnız önbellek klasöründen. Dış bağlantılar `opener` ile varsayılan tarayıcıda.
- SteamGridDB anahtarı DPAPI (`CryptProtectData`) ile şifreli.
