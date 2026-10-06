//! Görsel analizi: baskın renkten vurgu rengi (dinamik tema) ve logonun koyu olup olmadığı.
//! Görsel indirildiğinde bir kez hesaplanır, `*_meta.json` olarak saklanır. Arayüzde canvas ile
//! yapılamaz: `asset:` protokolündeki görseller farklı köken sayılır, piksel okuma engellenir.
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ArtMeta {
    /// CSS rengi, ör. `hsl(24 65% 58%)`; uygun renk yoksa `None`.
    pub accent: Option<String>,
    /// Logo koyu zeminde okunmayacak kadar koyu mu (beyaz silüet çizilir).
    pub logo_dark: bool,
}

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let (r, g, b) = (
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
    );
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < f32::EPSILON {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if (max - r).abs() < f32::EPSILON {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if (max - g).abs() < f32::EPSILON {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    (h * 60.0, s, l)
}

/// Ton kovaları; gri/çok koyu/çok açık pikseller sayılmaz. En kalabalık (doygunluk ağırlıklı) kova.
pub fn dominant_rgb(rgba: &[u8]) -> Option<(u8, u8, u8)> {
    let mut buckets = [(0u32, 0u64, 0u64, 0u64, 0f32); 18];
    for px in rgba.as_chunks::<4>().0 {
        if px[3] < 200 {
            continue;
        }
        let (h, s, l) = rgb_to_hsl(px[0], px[1], px[2]);
        if s < 0.25 || !(0.12..=0.88).contains(&l) {
            continue;
        }
        // h ∈ [0, 360) → kova 0..18
        let key = ((h / 20.0).round() as usize) % 18;
        let b = &mut buckets[key];
        b.0 += 1;
        b.1 += u64::from(px[0]);
        b.2 += u64::from(px[1]);
        b.3 += u64::from(px[2]);
        b.4 += s;
    }
    let score = |b: &(u32, u64, u64, u64, f32)| {
        let n = b.0 as f32;
        if n == 0.0 {
            0.0
        } else {
            n * (0.6 + b.4 / n)
        }
    };
    let best = buckets
        .iter()
        .max_by(|a, b| score(a).total_cmp(&score(b)))?;
    if best.0 < 8 {
        return None;
    }
    let n = u64::from(best.0);
    Some(((best.1 / n) as u8, (best.2 / n) as u8, (best.3 / n) as u8))
}

/// Okunur vurgu: doygunluk %35–65, açıklık %55–65.
pub fn to_accent((r, g, b): (u8, u8, u8)) -> String {
    let (h, s, l) = rgb_to_hsl(r, g, b);
    let s = s.clamp(0.35, 0.65);
    let l = l.clamp(0.55, 0.65);
    format!(
        "hsl({} {}% {}%)",
        h.round() as i32,
        (s * 100.0).round() as i32,
        (l * 100.0).round() as i32
    )
}

/// Opak piksellerin ne kadarı koyu (algısal parlaklık < 0.28)? Karma logolarda ortalama yanıltır.
pub fn dark_share(rgba: &[u8]) -> Option<f32> {
    let (mut dark, mut n) = (0u32, 0u32);
    for px in rgba.as_chunks::<4>().0 {
        if px[3] < 128 {
            continue;
        }
        let lum = 0.2126 * f32::from(px[0]) / 255.0
            + 0.7152 * f32::from(px[1]) / 255.0
            + 0.0722 * f32::from(px[2]) / 255.0;
        if lum < 0.28 {
            dark += 1;
        }
        n += 1;
    }
    (n > 0).then(|| dark as f32 / n as f32)
}

fn small_rgba(path: &Path) -> Option<Vec<u8>> {
    let img = image::open(path).ok()?;
    Some(img.thumbnail(64, 64).to_rgba8().into_raw())
}

/// Vurgu kaynağı (kapak → hero → ikon) ve logodan meta üretir.
pub fn analyze(color_source: Option<&Path>, logo: Option<&Path>) -> ArtMeta {
    ArtMeta {
        accent: color_source
            .and_then(small_rgba)
            .and_then(|px| dominant_rgb(&px))
            .map(to_accent),
        logo_dark: logo
            .and_then(small_rgba)
            .and_then(|px| dark_share(&px))
            .is_some_and(|s| s > 0.5),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgba(colors: &[((u8, u8, u8), usize)]) -> Vec<u8> {
        colors
            .iter()
            .flat_map(|((r, g, b), n)| std::iter::repeat_n([*r, *g, *b, 255], *n).flatten())
            .collect()
    }

    #[test]
    fn dominant_ignores_grey_black_white() {
        let px = rgba(&[
            ((230, 120, 20), 100),
            ((20, 20, 20), 40),
            ((128, 128, 128), 40),
            ((30, 80, 200), 40),
        ]);
        let c = dominant_rgb(&px).unwrap();
        let (h, _, _) = rgb_to_hsl(c.0, c.1, c.2);
        assert!((20.0..40.0).contains(&h), "turuncu bekleniyordu: {h}");
        assert!(dominant_rgb(&rgba(&[((10, 10, 10), 50), ((250, 250, 250), 50)])).is_none());
    }

    #[test]
    fn accent_is_clamped_like_the_ui() {
        assert_eq!(to_accent((255, 0, 0)), "hsl(0 65% 55%)");
    }

    #[test]
    fn mixed_logo_counts_as_dark_when_majority_dark() {
        let s = dark_share(&rgba(&[((20, 30, 80), 6), ((250, 250, 250), 4)])).unwrap();
        assert!((s - 0.6).abs() < 1e-6);
        assert!(dark_share(&[0, 0, 0, 0]).is_none());
    }

    #[test]
    fn analyze_reads_real_files() {
        let t = tempfile::tempdir().unwrap();
        let cover = t.path().join("c.png");
        image::RgbaImage::from_pixel(32, 32, image::Rgba([40, 90, 220, 255]))
            .save(&cover)
            .unwrap();
        let logo = t.path().join("l.png");
        image::RgbaImage::from_pixel(32, 8, image::Rgba([15, 20, 60, 255]))
            .save(&logo)
            .unwrap();
        let m = analyze(Some(&cover), Some(&logo));
        assert!(m.accent.unwrap().starts_with("hsl(2"));
        assert!(m.logo_dark);
        assert_eq!(analyze(None, None), ArtMeta::default());
    }
}
