import type { GameView } from "../ipc/library";

/** Vitrinde kurulu oyunlar: son oynanan önce, hiç oynanmamışlar ada göre sonda. */
export function showcaseOrder(games: readonly GameView[]): GameView[] {
  return games
    .filter((g) => g.installed)
    .sort(
      (a, b) =>
        (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0) || a.title.localeCompare(b.title, "tr-TR"),
    );
}
