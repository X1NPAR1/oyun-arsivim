# DESIGN SYSTEM — "Oyun salonu"

Karanlık, sinematik, konsol arayüzü hissi. Faz 2'de `src/design/tokens.css` + `src/design/tokens.ts` olarak
koda dökülür; kodda sabit değer yazılmaz.

## Renk

| Token                                        | Değer                             | Kullanım                                 |
| -------------------------------------------- | --------------------------------- | ---------------------------------------- |
| `--c-bg-0`                                   | `#07080d`                         | Pencere zemini                           |
| `--c-bg-1`                                   | `#0d0f18`                         | Yükseltilmiş zemin                       |
| `--c-glass`                                  | `rgba(22, 25, 38, 0.55)`          | Cam yüzey dolgusu                        |
| `--c-glass-strong`                           | `rgba(28, 32, 48, 0.78)`          | Okunabilirlik gereken cam                |
| `--c-glass-edge`                             | `rgba(255, 255, 255, 0.08)`       | İnce ışık kenarı (üst kenarda 0.14)      |
| `--c-text-1`                                 | `#eef0f8`                         | Birincil metin (zeminde kontrast 17:1)   |
| `--c-text-2`                                 | `#a6abc2`                         | İkincil metin (≥ 7:1)                    |
| `--c-text-3`                                 | `#6e7391`                         | Pasif metin (yalnız büyük metin)         |
| `--c-accent`                                 | dinamik, varsayılan `#7c5cff`     | Vurgu; oyunun baskın renginden türetilir |
| `--c-accent-2`                               | varsayılan `#22d3ee`              | Degrade ikinci durak                     |
| `--c-success` / `--c-warning` / `--c-danger` | `#34d399` / `#fbbf24` / `#f87171` | Durum                                    |

Dinamik vurgu: kapaktan baskın renk → HSL; doygunluk 55–85 %, açıklık 55–68 % aralığına kıstırılır
(kontrast güvencesi), 600 ms `--ease-smooth` ile geçiş.

## Tipografi

| Token                                                                                                           | Font    | Boyut / satır                          | Ağırlık |
| --------------------------------------------------------------------------------------------------------------- | ------- | -------------------------------------- | ------- |
| `display`                                                                                                       | Sora    | 48 / 56                                | 700     |
| `h1`                                                                                                            | Sora    | 32 / 40                                | 700     |
| `h2`                                                                                                            | Sora    | 24 / 32                                | 600     |
| `h3`                                                                                                            | Sora    | 18 / 26                                | 600     |
| `body`                                                                                                          | Manrope | 15 / 24                                | 500     |
| `small`                                                                                                         | Manrope | 13 / 20                                | 500     |
| `caption`                                                                                                       | Manrope | 11 / 16, +4 % harf aralığı, büyük harf | 700     |
| Fontlar woff2 olarak gömülür, Latin Extended (Türkçe) alt kümesiyle; `font-display: swap` yok — `block` (kısa). |

## Boşluk ve yarıçap

- Boşluk ölçeği (px): 4, 8, 12, 16, 20, 24, 32, 40, 56, 72.
- Yarıçap: `--r-sm 12px` (çip/giriş), `--r-md 16px` (buton, küçük kart), `--r-lg 22px` (kart, panel),
  `--r-xl 28px` (sidebar, büyük yüzey, splash), `--r-pill 999px`. **16 px altı yüzey yarıçapı yasak** (çip hariç).

## Derinlik, gölge, cam

- `--shadow-1`: `0 4px 16px rgba(0,0,0,.35)` · `--shadow-2`: `0 12px 40px rgba(0,0,0,.45)` ·
  `--shadow-glow`: `0 0 40px color-mix(in oklab, var(--c-accent) 35%, transparent)`.
- Cam: `background: var(--c-glass); backdrop-filter: blur(var(--glass-blur)) saturate(140%);`
  `border: 1px solid var(--c-glass-edge)`; üstte 1px iç ışık (`inset 0 1px 0 rgba(255,255,255,.06)`).
  `--glass-blur` 24px (ayardan 0–40px). Bölümler çizgiyle değil boşluk + derinlikle ayrılır.

## Hareket

| Token           | Değer                                            | Kullanım                      |
| --------------- | ------------------------------------------------ | ----------------------------- |
| `--dur-fast`    | 150 ms                                           | Hover, basma                  |
| `--dur-base`    | 240 ms                                           | Buton, çip, küçük geçiş       |
| `--dur-page`    | 380 ms                                           | Sayfa geçişi                  |
| `--dur-slow`    | 450 ms                                           | Paylaşılan öğe, splash geçişi |
| `--stagger`     | 35 ms (maks. 12 öğe gecikmeli; sonrası birlikte) | Kart girişleri                |
| `--ease-out`    | `cubic-bezier(.22, 1, .36, 1)`                   | Giriş                         |
| `--ease-in-out` | `cubic-bezier(.65, 0, .35, 1)`                   | Konum değişimi                |
| `spring.snappy` | stiffness 420, damping 34                        | Kart hover/tilt               |
| `spring.gentle` | stiffness 220, damping 28                        | Sayfa, layout                 |

- Sayfa geçişi: ileri → içerik sağdan 24 px + opacity 0 + scale .985; geri → soldan.
- Kart: hover'da `scale 1.04`, ±6° tilt, parlama gradyanı imleci izler; basma `scale .97`.
- Ken Burns: arka plan 1.08 ölçek, 30 sn yavaş kayma.
- Animasyon seviyesi: **tam** / **azaltılmış** (yalnız opacity, süreler ½, tilt/parallax yok) / **kapalı**.
  Windows "animasyonları azalt" (`prefers-reduced-motion`) açıkken varsayılan "azaltılmış".

## Bileşen kuralları

- **Sidebar:** pencere kenarından 12 px içeride, yüzen cam şerit, `--r-xl`, ikon + etiket, aktif öğe vurgu ışıması.
- **Başlık çubuğu:** 40 px, sürüklenebilir (`data-tauri-drag-region`), sağda yuvarlak cam pencere düğmeleri
  (kapat hover = danger).
- **Kart:** oran 2:3, `--r-lg`, alt kısmında kapak renginden degrade + ad; hiçbir zaman düz kutu.
- **Buton:** birincil = vurgu degradesi + glow, ikincil = cam; `--r-md` veya hap.
- **Giriş alanları:** cam, `--r-sm`, odakta vurgu halkası (2 px, `--c-accent`).
- **İkonlar:** ikon fontu, 20/24 px, metinle aynı renk; SVG yasak.
- **Odak göstergesi:** her etkileşimli öğede görünür 2 px halka + 2 px ofset.
