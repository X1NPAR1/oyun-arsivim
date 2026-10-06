/**
 * Görselden baskın renk çıkarımı ve vurgu rengine dönüştürme (dinamik tema).
 * Kontrast güvencesi: doygunluk ve açıklık aralığa kıstırılır (docs/DESIGN_SYSTEM.md → Renk).
 */

export interface Rgb {
  r: number;
  g: number;
  b: number;
}

export function rgbToHsl({ r, g, b }: Rgb): [number, number, number] {
  const rn = r / 255;
  const gn = g / 255;
  const bn = b / 255;
  const max = Math.max(rn, gn, bn);
  const min = Math.min(rn, gn, bn);
  const l = (max + min) / 2;
  if (max === min) return [0, 0, l];
  const d = max - min;
  const s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
  let h: number;
  if (max === rn) h = (gn - bn) / d + (gn < bn ? 6 : 0);
  else if (max === gn) h = (bn - rn) / d + 2;
  else h = (rn - gn) / d + 4;
  return [h * 60, s, l];
}

/**
 * Pikselleri renk tonu kovalarına ayırır; çok koyu, çok açık ve griye yakın pikseller sayılmaz.
 * En kalabalık kovanın ortalaması döner. Uygun piksel yoksa `null` (vurgu varsayılanda kalır).
 */
export function dominantColor(pixels: Uint8ClampedArray, step = 4): Rgb | null {
  const buckets = new Map<number, { n: number; r: number; g: number; b: number; s: number }>();
  for (let i = 0; i + 3 < pixels.length; i += 4 * step) {
    const a = pixels[i + 3] ?? 0;
    if (a < 200) continue;
    const c = { r: pixels[i] ?? 0, g: pixels[i + 1] ?? 0, b: pixels[i + 2] ?? 0 };
    const [h, s, l] = rgbToHsl(c);
    if (s < 0.25 || l < 0.12 || l > 0.88) continue;
    const key = Math.round(h / 20) % 18;
    const bucket = buckets.get(key) ?? { n: 0, r: 0, g: 0, b: 0, s: 0 };
    bucket.n += 1;
    bucket.r += c.r;
    bucket.g += c.g;
    bucket.b += c.b;
    bucket.s += s;
    buckets.set(key, bucket);
  }
  let best: { n: number; r: number; g: number; b: number; s: number } | null = null;
  for (const b of buckets.values()) {
    // Doygun renkler hafif öne çıkar (soluk arka plan renkleri yerine ana renk).
    if (!best || b.n * (0.6 + b.s / b.n) > best.n * (0.6 + best.s / best.n)) best = b;
  }
  if (!best || best.n < 8) return null;
  return {
    r: Math.round(best.r / best.n),
    g: Math.round(best.g / best.n),
    b: Math.round(best.b / best.n),
  };
}

/** Vurgu rengine çevirir: doygunluk %35–65, açıklık %55–65 (koyu zeminde okunur, cırtlak değil). */
export function toAccent(c: Rgb): string {
  const [h, s, l] = rgbToHsl(c);
  const sat = Math.min(0.65, Math.max(0.35, s));
  const light = Math.min(0.65, Math.max(0.55, l));
  return `hsl(${String(Math.round(h))} ${String(Math.round(sat * 100))}% ${String(Math.round(light * 100))}%)`;
}

/** Opak piksellerin ortalama algısal parlaklığı (0–1); opak piksel yoksa `null`. */
export function averageLuminance(pixels: Uint8ClampedArray): number | null {
  let sum = 0;
  let n = 0;
  for (let i = 0; i + 3 < pixels.length; i += 4) {
    if ((pixels[i + 3] ?? 0) < 128) continue;
    const r = (pixels[i] ?? 0) / 255;
    const g = (pixels[i + 1] ?? 0) / 255;
    const b = (pixels[i + 2] ?? 0) / 255;
    sum += 0.2126 * r + 0.7152 * g + 0.0722 * b;
    n++;
  }
  return n === 0 ? null : sum / n;
}

/** Koyu zeminde okunmayacak kadar koyu logo eşiği. */
export const DARK_LOGO_THRESHOLD = 0.28;

/**
 * Opak piksellerin ne kadarı koyu? Karma logolarda (ör. koyu yazı + beyaz "2") ortalama yanıltır;
 * çoğunluk koyuysa logo okunmaz.
 */
export function darkPixelShare(
  pixels: Uint8ClampedArray,
  threshold = DARK_LOGO_THRESHOLD,
): number | null {
  let dark = 0;
  let n = 0;
  for (let i = 0; i + 3 < pixels.length; i += 4) {
    if ((pixels[i + 3] ?? 0) < 128) continue;
    const lum =
      0.2126 * ((pixels[i] ?? 0) / 255) +
      0.7152 * ((pixels[i + 1] ?? 0) / 255) +
      0.0722 * ((pixels[i + 2] ?? 0) / 255);
    if (lum < threshold) dark++;
    n++;
  }
  return n === 0 ? null : dark / n;
}
