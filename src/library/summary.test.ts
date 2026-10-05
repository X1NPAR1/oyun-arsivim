import type { GameView } from "../ipc/library";
import { EMPTY_PLAYTIME } from "../test/fixtures";
import { summarize } from "./summary";

function g(
  id: number,
  week: number,
  total: number,
  lastPlayed: number | null,
  installed = true,
): GameView {
  return {
    id,
    platform: "steam",
    platformId: String(id),
    title: "G" + String(id),
    installDir: "",
    executable: null,
    launchUri: "",
    launchArgs: null,
    sizeBytes: null,
    lastPlayed,
    installed,
    addedAt: 0,
    removedAt: null,
    art: { cover: null, logo: null, hero: null, icon: null, backdrop: null },
    playtime: { ...EMPTY_PLAYTIME, weekSeconds: week, totalSeconds: total },
  };
}

describe("ana sayfa özeti", () => {
  it("haftalık toplam, haftanın favorisi, son oynanan, kütüphane", () => {
    const s = summarize([g(1, 3600, 10_000, 50), g(2, 7200, 500, 90), g(3, 0, 0, null, false)]);
    expect(s.weekSeconds).toBe(10_800);
    expect(s.topWeek?.id).toBe(2);
    expect(s.lastPlayed?.id).toBe(2);
    expect(s.installed).toBe(2);
    expect(s.lifetimeSeconds).toBe(10_500);
  });

  it("veri yoksa boş değerler", () => {
    const s = summarize([g(1, 0, 0, null)]);
    expect(s.topWeek).toBeNull();
    expect(s.lastPlayed).toBeNull();
  });
});
