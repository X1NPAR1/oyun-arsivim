/** Açılış çemberi geometrisi: üstten (−90°) saat yönünde doluyor. */
export function ringGeometry(size: number, stroke: number, percent: number) {
  const c = size / 2;
  const r = c - stroke;
  const start = -Math.PI / 2;
  const clamped = Math.min(100, Math.max(0, percent));
  return { c, r, stroke, start, end: start + (Math.PI * 2 * clamped) / 100 };
}

/**
 * Gösterilen yüzdeyi hedefe yumuşakça yaklaştırır; asla hedefi geçmez ve geri gitmez.
 * Büyük sıçramalarda hızlı, küçük farklarda yavaş (sayaç sayarak artar).
 */
export function stepTowards(shown: number, target: number): number {
  if (target <= shown) return shown;
  const diff = target - shown;
  const next = shown + Math.max(0.25, diff * 0.12);
  return Math.min(target, next);
}
