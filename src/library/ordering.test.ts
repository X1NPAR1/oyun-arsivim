import type { GameView } from "../ipc/library";
import { showcaseOrder } from "./ordering";

const base = {
  platformId: "",
  installDir: "",
  executable: null,
  launchUri: "",
  sizeBytes: null,
  addedAt: 0,
  removedAt: null,
  art: { cover: null, logo: null, hero: null, icon: null },
};
const g = (id: number, title: string, lastPlayed: number | null, installed = true): GameView => ({
  ...base,
  id,
  title,
  lastPlayed,
  installed,
  platform: "steam",
});

describe("vitrin sırası", () => {
  it("yalnız kurulu oyunlar, son oynanan önce, sonra ada göre", () => {
    const order = showcaseOrder([
      g(1, "Unturned", null),
      g(2, "CS2", 500),
      g(3, "Silinen", 900, false),
      g(4, "Assetto", null),
      g(5, "GTA", 700),
    ]);
    expect(order.map((x) => x.id)).toEqual([5, 2, 4, 1]);
  });
});
