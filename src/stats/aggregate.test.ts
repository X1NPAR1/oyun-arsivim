import {
  aggregate,
  formatAxis,
  formatDuration,
  formatHourRange,
  mondayIndex,
  niceStep,
  type HourRow,
} from "./aggregate";

/** Yerel saatle oluşturulan zamanın UTC saat numarası (testler saat diliminden bağımsız). */
function hourOf(y: number, m: number, d: number, h: number): number {
  return Math.floor(new Date(y, m - 1, d, h).getTime() / 3_600_000);
}

const rows: HourRow[] = [
  // 6 Ekim 2026 Salı 21:00 — 40 dk aktif, 10 dk AFK
  { gameId: 1, hour: hourOf(2026, 10, 6, 21), activeSeconds: 2400, idleSeconds: 600 },
  // Salı 22:00 — 60 dk
  { gameId: 1, hour: hourOf(2026, 10, 6, 22), activeSeconds: 3600, idleSeconds: 0 },
  // 10 Ekim Cumartesi 14:00 — oyun 2, 30 dk
  { gameId: 2, hour: hourOf(2026, 10, 10, 14), activeSeconds: 1800, idleSeconds: 0 },
];

describe("istatistik toplama", () => {
  const range = { from: new Date(2026, 9, 5), to: new Date(2026, 9, 11, 23) };
  const r = aggregate(rows, [], range);

  it("toplamlar ve AFK oranı", () => {
    expect(r.totals).toEqual({ active: 7800, idle: 600, total: 8400, afkRatio: 600 / 8400 });
  });

  it("boş günler dahil günlük seri (yerel gün)", () => {
    expect(r.daily).toHaveLength(7);
    const tue = r.daily.find((d) => d.key === "2026-10-06");
    expect(tue).toMatchObject({ active: 6000, idle: 600 });
    expect(r.insights.activeDays).toBe(2);
  });

  it("saat ve hafta günü dağılımı (Pazartesi başlangıçlı)", () => {
    expect(r.byHour[21]).toBe(3000);
    expect(r.byHour[22]).toBe(3600);
    expect(r.byWeekday[1]).toBe(6600); // Salı
    expect(r.byWeekday[5]).toBe(1800); // Cumartesi
    expect(r.heat[1]?.[22]).toBe(3600);
  });

  it("içgörüler: en yoğun gün/saat, en sessiz gün, yoğun pencere", () => {
    expect(r.insights.busiestDay).toBe(1);
    expect(r.insights.peakHour).toBe(22);
    expect(r.insights.peakWindow).toEqual([21, 23]);
    // Yalnız 2 farklı günde oynandı → "en sessiz gün" söylenmez.
    expect(r.insights.quietestDay).toBeNull();
  });

  it("en çok oynananlar sıralı", () => {
    expect(r.topGames).toEqual([
      { gameId: 1, seconds: 6600 },
      { gameId: 2, seconds: 1800 },
    ]);
  });

  it("oturum istatistikleri", () => {
    const s = aggregate(
      [],
      [
        {
          id: 1,
          gameId: 1,
          startedAt: 0,
          endedAt: 0,
          activeSeconds: 3000,
          idleSeconds: 600,
          open: false,
        },
        {
          id: 2,
          gameId: 1,
          startedAt: 0,
          endedAt: 0,
          activeSeconds: 1200,
          idleSeconds: 0,
          open: false,
        },
      ],
      range,
    ).sessions;
    expect(s).toEqual({ count: 2, average: 2400, longest: 3600 });
  });

  it("veri yoksa içgörü yok", () => {
    const empty = aggregate([], [], range);
    expect(empty.insights).toMatchObject({
      busiestDay: null,
      quietestDay: null,
      peakHour: null,
      peakWindow: null,
    });
    expect(empty.totals.afkRatio).toBe(0);
  });
});

describe("en sessiz gün", () => {
  it("en az 3 farklı günde oynandıysa oynanan günler arasından seçilir", () => {
    const many: HourRow[] = [
      { gameId: 1, hour: hourOf(2026, 10, 5, 20), activeSeconds: 3600, idleSeconds: 0 },
      { gameId: 1, hour: hourOf(2026, 10, 7, 20), activeSeconds: 600, idleSeconds: 0 },
      { gameId: 1, hour: hourOf(2026, 10, 9, 20), activeSeconds: 1800, idleSeconds: 0 },
    ];
    const r = aggregate(many, [], { from: new Date(2026, 9, 5), to: new Date(2026, 9, 11, 23) });
    expect(r.insights.quietestDay).toBe(2); // Çarşamba; hiç oynanmayan günler sayılmaz
    expect(r.insights.busiestDay).toBe(0);
  });
});

describe("biçimlendirme", () => {
  it("süre", () => {
    expect(formatDuration(0)).toBe("0 dk");
    expect(formatDuration(30)).toBe("1 dk'dan az");
    expect(formatDuration(45 * 60)).toBe("45 dk");
    expect(formatDuration(8100)).toBe("2 sa 15 dk");
    expect(formatDuration(7200)).toBe("2 sa");
    expect(formatDuration(358_753 * 60)).toBe("5.979 sa");
  });

  it("eksen adımı ve etiketi tekrarsız", () => {
    expect(niceStep(75)).toBe(60);
    expect(niceStep(3000)).toBe(900);
    expect(niceStep(8 * 3600)).toBe(7200);
    expect(formatAxis(900)).toBe("15 dk");
    expect(formatAxis(5400)).toBe("1,5 sa");
    expect(formatAxis(0)).toBe("0");
  });

  it("saat aralığı ve hafta günü", () => {
    expect(formatHourRange([23, 1])).toBe("23:00–01:00");
    expect(mondayIndex(new Date(2026, 9, 11))).toBe(6); // Pazar
  });
});
