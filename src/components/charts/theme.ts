/**
 * Grafik teması — tasarım token'larıyla aynı değerler (canvas CSS değişkeni okuyamaz).
 * Seri renkleri dataviz doğrulayıcısından geçti (koyu yüzey #17181b): aktif mavi ile AFK gri arası
 * CVD ΔE 16.8, normal ΔE 17.1. AFK bilinçli olarak nötr gri: "pasif zaman" geri planda kalır.
 */
export const CHART_THEME = {
  surface: "#17181b",
  grid: "rgba(255, 255, 255, 0.06)",
  text1: "#ededee",
  text2: "#9b9ea6",
  text3: "#6b6e76",
  tooltipBg: "rgba(24, 25, 29, 0.96)",
  font: "'Manrope Variable', sans-serif",
  active: "#3987e5",
  idle: "#6b6e76",
} as const;

/** Isı haritası için tek tonlu sıralı skala: yüzeye yakın koyudan parlak maviye. */
export function heatColor(t: number): string {
  if (t <= 0) return "rgba(255, 255, 255, 0.035)";
  const clamped = Math.min(1, Math.max(0, t));
  // Algısal olarak düzgün ilerlemesi için karekök eğrisi; düşük değerler de görünür kalır.
  const k = Math.sqrt(clamped);
  const from = [30, 52, 92];
  const to = [110, 170, 255];
  const c = from.map((f, i) => Math.round(f + ((to[i] ?? f) - f) * k));
  return `rgb(${String(c[0])}, ${String(c[1])}, ${String(c[2])})`;
}
