# DESIGN SYSTEM — "Oyun salonu"

Karanlık, sinematik, konsol arayüzü hissi. Faz 2'de `src/design/tokens.css` + `src/design/tokens.ts` olarak
koda dökülür; kodda sabit değer yazılmaz.

## Renk

**İlke:** Arayüz renksizdir (nötr grafit); renk yalnızca oyunların kendi görsellerinden gelir. Neon degrade,
ışıma (glow), renkli arka plan küreleri yasak — "şablon/yapay zekâ" görünümü yerine konsol arayüzleri
(PS5, Xbox, Steam Big Picture) gibi sakin ve içerik odaklı.

| Token                                        | Değer                             | Kullanım                                                           |
| -------------------------------------------- | --------------------------------- | ------------------------------------------------------------------ |
| `--c-bg-0`                                   | `#0f1012`                         | Pencere zemini (üstte `#131417`'den çok hafif geçiş)               |
| `--c-bg-1`                                   | `#16171a`                         | Yükseltilmiş zemin                                                 |
| `--c-surface` / `--c-surface-hover`          | `#1b1c20` / `#222328`             | Düz yüzeyler                                                       |
| `--c-glass`                                  | `rgba(24, 25, 29, 0.72)`          | Cam yüzey (oyun görseli üstünde)                                   |
| `--c-glass-edge`                             | `rgba(255, 255, 255, 0.06)`       | İnce kenar (üstte 0.09)                                            |
| `--c-text-1`                                 | `#ededee`                         | Birincil metin; birincil buton zemini                              |
| `--c-text-2`                                 | `#9b9ea6`                         | İkincil metin                                                      |
| `--c-text-3`                                 | `#6b6e76`                         | Etiket/pasif                                                       |
| `--c-on-light`                               | `#0f1012`                         | Açık zemin üstündeki metin                                         |
| `--c-accent`                                 | dinamik, varsayılan `#4f8ff7`     | Yalnız odak halkası, seçim, ilerleme; oyunun görselinden türetilir |
| `--c-success` / `--c-warning` / `--c-danger` | `#3fb97a` / `#e0a63a` / `#e5534b` | Durum                                                              |

Dinamik vurgu: kapaktan baskın renk → HSL; doygunluk 35–65 %, açıklık 55–65 % aralığına kıstırılır, 600 ms geçiş.

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

- `--shadow-1`: `0 2px 8px rgba(0,0,0,.28)` · `--shadow-2`: `0 10px 30px rgba(0,0,0,.35)` · ışıma gölgesi yok.
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

- **Sidebar:** pencere kenarından 12 px içeride, yüzen cam şerit, `--r-xl`, ikon + etiket; aktif öğe `rgba(255,255,255,.09)` yüzey + dolu ikon.
- **Pencere:** Win11'de Mica + OS yuvarlak köşeleri; Win10'da opak degrade zemin, OS gölgesi kapalı, kabuk `--r-md` ile yuvarlatılır (büyütülünce 0).
- **Başlık çubuğu:** 40 px, sürüklenebilir (`data-tauri-drag-region`), sağda yuvarlak cam pencere düğmeleri
  (kapat hover = danger).
- **Kart:** oran 2:3, `--r-lg`, alt kısmında kapak renginden degrade + ad; hiçbir zaman düz kutu.
- **Buton:** birincil = açık zemin (`--c-text-1`) + koyu yazı, ikincil = yarı saydam yüzey; hap biçimi. Degrade/ışıma yok.
- **Giriş alanları:** cam, `--r-sm`, odakta vurgu halkası (2 px, `--c-accent`).
- **İkonlar:** ikon fontu, 20/24 px, metinle aynı renk; SVG yasak.
- **Odak göstergesi:** her etkileşimli öğede görünür 2 px halka + 2 px ofset.
