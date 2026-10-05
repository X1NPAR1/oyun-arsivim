/** Sonsuz döngülü şerit için yardımcılar (saf fonksiyonlar, test edilebilir). */

export function wrapIndex(index: number, count: number): number {
  if (count <= 0) return 0;
  return ((index % count) + count) % count;
}

/**
 * Bir öğenin odaktaki öğeye göre en kısa yoldaki konumu: döngüde sondaki öğe, baştakinin solunda
 * (-1) görünür. Sonuç [-floor(n/2), ceil(n/2)) aralığındadır.
 */
export function relativeOffset(index: number, focus: number, count: number): number {
  if (count <= 0) return 0;
  let d = wrapIndex(index - focus, count);
  if (d >= Math.ceil(count / 2)) d -= count;
  return d;
}

/**
 * Fare tekerleği birikimi: küçük dokunmatik yüzey hareketleri birikir, eşiği aşınca tek adım atılır.
 * Dönüş: [atılacak adım (-1, 0, 1), kalan birikim].
 */
export function wheelStep(accumulated: number, delta: number, threshold = 60): [number, number] {
  const total = accumulated + delta;
  if (total >= threshold) return [1, 0];
  if (total <= -threshold) return [-1, 0];
  return [0, total];
}
