import type { GameView } from "../ipc/library";
import { filterGames, searchKey } from "./filter";

function g(id: number, title: string, platform: GameView["platform"], installed = true): GameView {
  return {
    id,
    platform,
    platformId: String(id),
    title,
    installDir: "",
    executable: null,
    launchUri: "",
    sizeBytes: null,
    lastPlayed: null,
    installed,
    addedAt: 0,
    removedAt: installed ? null : 1,
    art: { cover: null, logo: null, hero: null, icon: null },
  };
}

const GAMES = [
  g(1, "Counter-Strike 2", "steam"),
  g(2, "Unturned", "steam", false),
  g(3, "Grand Theft Auto V", "epic"),
  g(4, "Şahin'in Oyunu", "manual"),
];

describe("kütüphane filtresi", () => {
  it("Türkçe karakter ve büyük/küçük harf duyarsız arar", () => {
    expect(searchKey("ŞAHİN'in Oyunu")).toBe("sahin in oyunu");
    expect(filterGames(GAMES, { query: "sahin", install: "all", platform: "all" })).toHaveLength(1);
    expect(filterGames(GAMES, { query: "counter 2", install: "all", platform: "all" })[0]?.id).toBe(
      1,
    );
  });

  it("kurulu / silinen filtresi", () => {
    expect(
      filterGames(GAMES, { query: "", install: "removed", platform: "all" }).map((x) => x.id),
    ).toEqual([2]);
    expect(filterGames(GAMES, { query: "", install: "installed", platform: "all" })).toHaveLength(
      3,
    );
  });

  it("platform filtresi", () => {
    expect(
      filterGames(GAMES, { query: "", install: "all", platform: "epic" }).map((x) => x.id),
    ).toEqual([3]);
  });
});
