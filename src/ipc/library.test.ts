import { vi } from "vitest";
import { formatBytes, formatRelative, logoOf, type GameView } from "./library";

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (p: string) => `asset://${p}`,
  invoke: () => Promise.resolve(null),
}));

function game(art: Partial<GameView["art"]>): GameView {
  return {
    id: 1,
    platform: "steam",
    platformId: "730",
    title: "Counter-Strike 2",
    installDir: "",
    executable: null,
    launchUri: "",
    sizeBytes: null,
    lastPlayed: null,
    installed: true,
    addedAt: 0,
    removedAt: null,
    art: { cover: null, logo: null, hero: null, icon: null, ...art },
  };
}

describe("kütüphane yardımcıları", () => {
  it("logo önceliği: Steam logosu, sonra exe ikonu", () => {
    expect(logoOf(game({ logo: "a.png", icon: "b.png" }))).toEqual({
      src: "asset://a.png",
      kind: "logo",
    });
    expect(logoOf(game({ icon: "b.png" }))).toEqual({ src: "asset://b.png", kind: "icon" });
    expect(logoOf(game({}))).toBeNull();
  });

  it("boyutu Türkçe biçimler", () => {
    expect(formatBytes(73_984_302_411)).toBe("68,9 GB");
    expect(formatBytes(5 * 1024 ** 2)).toBe("5 MB");
    expect(formatBytes(null)).toBeNull();
    expect(formatBytes(0)).toBeNull();
  });

  it("göreli zaman", () => {
    const now = 1_800_000_000_000;
    const s = now / 1000;
    expect(formatRelative(s - 30, now)).toBe("az önce");
    expect(formatRelative(s - 3 * 3600, now)).toBe("3 saat önce");
    expect(formatRelative(s - 5 * 86400, now)).toBe("5 gün önce");
    expect(formatRelative(s - 400 * 86400, now)).toBe("1 yıl önce");
    expect(formatRelative(null, now)).toBeNull();
  });
});
