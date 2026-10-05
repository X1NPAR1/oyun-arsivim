import { invoke } from "@tauri-apps/api/core";
import type { HourRow, SessionRow } from "../stats/aggregate";

export const statsApi = {
  hourly: (fromHour: number, toHour: number, gameId: number | null) =>
    invoke<HourRow[]>("playtime_hourly", { fromHour, toHour, gameId }),
  sessions: (from: number, to: number, gameId: number | null) =>
    invoke<SessionRow[]>("playtime_sessions", { from, to, gameId }),
};
