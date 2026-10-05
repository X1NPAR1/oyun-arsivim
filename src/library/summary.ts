import type { GameView } from "../ipc/library";

export interface Summary {
  weekSeconds: number;
  topWeek: GameView | null;
  lastPlayed: GameView | null;
  installed: number;
  lifetimeSeconds: number;
}

/** Ana sayfa özetinin saf hesaplaması (test edilebilir). */
export function summarize(games: readonly GameView[]): Summary {
  let weekSeconds = 0;
  let topWeek: GameView | null = null;
  let lastPlayed: GameView | null = null;
  let lifetimeSeconds = 0;
  let installed = 0;
  for (const g of games) {
    weekSeconds += g.playtime.weekSeconds;
    lifetimeSeconds += g.playtime.totalSeconds;
    if (g.installed) installed++;
    if (
      g.playtime.weekSeconds > 0 &&
      (!topWeek || g.playtime.weekSeconds > topWeek.playtime.weekSeconds)
    )
      topWeek = g;
    if (g.lastPlayed && (!lastPlayed || g.lastPlayed > (lastPlayed.lastPlayed ?? 0)))
      lastPlayed = g;
  }
  return { weekSeconds, topWeek, lastPlayed, installed, lifetimeSeconds };
}
