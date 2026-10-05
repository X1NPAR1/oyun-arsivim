import type { GameView, Platform } from "../ipc/library";

export type InstallFilter = "all" | "installed" | "removed";

export interface LibraryFilter {
  query: string;
  install: InstallFilter;
  platform: Platform | "all";
}

/** Türkçe büyük/küçük harf ve aksan farklarını yok sayan arama anahtarı ("Şahin" ≈ "sahin"). */
export function searchKey(text: string): string {
  return text
    .toLocaleLowerCase("tr-TR")
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .replace(/ı/g, "i")
    .replace(/[^a-z0-9]+/g, " ")
    .trim();
}

export function filterGames(games: readonly GameView[], f: LibraryFilter): GameView[] {
  const q = searchKey(f.query);
  const terms = q ? q.split(" ") : [];
  return games.filter((g) => {
    if (f.install === "installed" && !g.installed) return false;
    if (f.install === "removed" && g.installed) return false;
    if (f.platform !== "all" && g.platform !== f.platform) return false;
    if (terms.length === 0) return true;
    const key = searchKey(g.title);
    return terms.every((t) => key.includes(t));
  });
}
