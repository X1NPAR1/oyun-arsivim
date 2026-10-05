import type { PlaytimeSummary } from "../ipc/library";

export const EMPTY_PLAYTIME: PlaytimeSummary = {
  totalSeconds: 0,
  trackedSeconds: 0,
  idleSeconds: 0,
  weekSeconds: 0,
  platformMinutes: null,
  platformTwoWeeksMinutes: null,
  sessions: 0,
};
