/**
 * İstatistik toplama (saf fonksiyonlar). Girdi Rust'tan gelen UTC saat kovaları ve oturumlardır;
 * gün/saat dağılımı burada YEREL saat dilimine göre hesaplanır (yaz saati dahil doğru).
 */

export interface HourRow {
  gameId: number;
  hour: number; // unix saniye / 3600 (UTC)
  activeSeconds: number;
  idleSeconds: number;
}

export interface SessionRow {
  id: number;
  gameId: number;
  startedAt: number;
  endedAt: number;
  activeSeconds: number;
  idleSeconds: number;
  open: boolean;
}

export interface DayPoint {
  key: string; // YYYY-MM-DD (yerel)
  date: Date;
  active: number;
  idle: number;
}

export interface StatsResult {
  totals: { active: number; idle: number; total: number; afkRatio: number };
  daily: DayPoint[];
  /** 0–23 yerel saat; aktif + AFK saniye. */
  byHour: number[];
  /** 0 = Pazartesi … 6 = Pazar. */
  byWeekday: number[];
  /** [haftanın günü][saat] toplam saniye. */
  heat: number[][];
  topGames: { gameId: number; seconds: number }[];
  sessions: { count: number; average: number; longest: number };
  insights: {
    busiestDay: number | null;
    quietestDay: number | null;
    peakHour: number | null;
    peakWindow: [number, number] | null;
    activeDays: number;
  };
}

const MIN_WEEKDAYS_FOR_QUIET = 3;

export const WEEKDAYS = ["Pazartesi", "Salı", "Çarşamba", "Perşembe", "Cuma", "Cumartesi", "Pazar"];
export const WEEKDAYS_SHORT = ["Pzt", "Sal", "Çar", "Per", "Cum", "Cmt", "Paz"];

/** JS getDay (0 = Pazar) → Pazartesi başlangıçlı indeks. */
export function mondayIndex(date: Date): number {
  return (date.getDay() + 6) % 7;
}

export function dayKey(date: Date): string {
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  return `${String(date.getFullYear())}-${m}-${d}`;
}

function startOfDay(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}

/** Aralıktaki her yerel gün (boş günler dahil) — grafikte boşluklar görünür kalsın. */
export function daysBetween(from: Date, to: Date): Date[] {
  const out: Date[] = [];
  for (
    let d = startOfDay(from);
    d <= to;
    d = new Date(d.getFullYear(), d.getMonth(), d.getDate() + 1)
  ) {
    out.push(d);
  }
  return out;
}

function argMax(values: number[]): number | null {
  let best = -1;
  let bestValue = 0;
  values.forEach((v, i) => {
    if (v > bestValue) {
      bestValue = v;
      best = i;
    }
  });
  return best < 0 ? null : best;
}

/** En yoğun ardışık 2 saatlik pencere (gece yarısını saran pencereler dahil). */
function peakWindow(byHour: number[]): [number, number] | null {
  let best: [number, number] | null = null;
  let bestSum = 0;
  for (let h = 0; h < 24; h++) {
    const sum = (byHour[h] ?? 0) + (byHour[(h + 1) % 24] ?? 0);
    if (sum > bestSum) {
      bestSum = sum;
      best = [h, (h + 2) % 24];
    }
  }
  return best;
}

export function aggregate(
  rows: readonly HourRow[],
  sessions: readonly SessionRow[],
  range: { from: Date; to: Date },
): StatsResult {
  const days = daysBetween(range.from, range.to);
  const dayIndex = new Map(days.map((d, i) => [dayKey(d), i]));
  const daily: DayPoint[] = days.map((date) => ({ key: dayKey(date), date, active: 0, idle: 0 }));
  const byHour = Array.from({ length: 24 }, () => 0);
  const byWeekday = Array.from({ length: 7 }, () => 0);
  const heat = Array.from({ length: 7 }, () => Array.from({ length: 24 }, () => 0));
  const perGame = new Map<number, number>();
  let active = 0;
  let idle = 0;

  for (const r of rows) {
    const date = new Date(r.hour * 3600 * 1000);
    const total = r.activeSeconds + r.idleSeconds;
    active += r.activeSeconds;
    idle += r.idleSeconds;
    const i = dayIndex.get(dayKey(date));
    const point = i === undefined ? undefined : daily[i];
    if (point) {
      point.active += r.activeSeconds;
      point.idle += r.idleSeconds;
    }
    const h = date.getHours();
    const wd = mondayIndex(date);
    byHour[h] = (byHour[h] ?? 0) + total;
    byWeekday[wd] = (byWeekday[wd] ?? 0) + total;
    const row = heat[wd];
    if (row) row[h] = (row[h] ?? 0) + total;
    perGame.set(r.gameId, (perGame.get(r.gameId) ?? 0) + total);
  }

  const lengths = sessions.map((s) => s.activeSeconds + s.idleSeconds);
  const total = active + idle;
  // En sessiz gün: yalnız oynanan günler arasından ve en az 3 farklı günde oynandıysa anlamlıdır
  // (tek günlük veriyle "en az Salı oynuyorsun" demek yanıltıcı olurdu).
  const playedWeekdays = byWeekday.map((v, i) => ({ v, i })).filter((x) => x.v > 0);
  const quietest =
    playedWeekdays.length >= MIN_WEEKDAYS_FOR_QUIET
      ? playedWeekdays.reduce((min, x) => (x.v < min.v ? x : min)).i
      : null;

  return {
    totals: { active, idle, total, afkRatio: total > 0 ? idle / total : 0 },
    daily,
    byHour,
    byWeekday,
    heat,
    topGames: [...perGame.entries()]
      .map(([gameId, seconds]) => ({ gameId, seconds }))
      .sort((a, b) => b.seconds - a.seconds),
    sessions: {
      count: sessions.length,
      average: lengths.length ? lengths.reduce((a, b) => a + b, 0) / lengths.length : 0,
      longest: lengths.length ? Math.max(...lengths) : 0,
    },
    insights: {
      busiestDay: argMax(byWeekday),
      quietestDay: quietest,
      peakHour: argMax(byHour),
      peakWindow: peakWindow(byHour),
      activeDays: daily.filter((d) => d.active + d.idle > 0).length,
    },
  };
}

/** "2 sa 15 dk", "45 dk", "5.979 sa" (büyük değerlerde dakika gösterilmez). */
export function formatDuration(seconds: number): string {
  const s = Math.max(0, Math.round(seconds));
  if (s < 60) return s === 0 ? "0 dk" : "1 dk'dan az";
  const minutes = Math.floor(s / 60);
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  if (hours === 0) return `${String(minutes)} dk`;
  if (hours >= 100 || rest === 0) return `${hours.toLocaleString("tr-TR")} sa`;
  return `${String(hours)} sa ${String(rest)} dk`;
}

export function formatHourRange([a, b]: [number, number]): string {
  const f = (h: number) => `${String(h).padStart(2, "0")}:00`;
  return `${f(a)}–${f(b)}`;
}

const STEPS = [60, 300, 600, 900, 1800, 3600, 7200, 10800, 21600, 43200, 86400, 172800, 360000];

/** Süre ekseni için yuvarlak adım: en fazla ~4 aralık, dakika/saat katları. */
export function niceStep(maxSeconds: number, targetTicks = 4): number {
  const raw = Math.max(60, maxSeconds / targetTicks);
  return STEPS.find((s) => s >= raw) ?? Math.ceil(raw / 360000) * 360000;
}

/** Eksen etiketi: kısa biçim ("15 dk", "1,5 sa"). */
export function formatAxis(seconds: number): string {
  if (seconds === 0) return "0";
  if (seconds < 3600) return `${String(Math.round(seconds / 60))} dk`;
  return `${(seconds / 3600).toLocaleString("tr-TR", { maximumFractionDigits: 1 })} sa`;
}
