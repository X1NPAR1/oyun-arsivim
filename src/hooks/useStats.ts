import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { statsApi } from "../ipc/stats";
import { aggregate } from "../stats/aggregate";

export type RangeDays = 7 | 30 | 90 | 365;

/** Seçili aralık (yerel günün başından bugünün sonuna) için toplanmış istatistik. */
export function useStats(days: RangeDays, gameId: number | null) {
  const range = useMemo(() => {
    const now = new Date();
    const from = new Date(now.getFullYear(), now.getMonth(), now.getDate() - (days - 1));
    return { from, to: now };
  }, [days]);
  const fromSec = Math.floor(range.from.getTime() / 1000);
  const toSec = Math.ceil(range.to.getTime() / 1000) + 3600;

  const hourly = useQuery({
    queryKey: ["stats", "hourly", days, gameId],
    queryFn: () => statsApi.hourly(Math.floor(fromSec / 3600), Math.ceil(toSec / 3600), gameId),
    refetchInterval: 60_000,
  });
  const sessions = useQuery({
    queryKey: ["stats", "sessions", days, gameId],
    queryFn: () => statsApi.sessions(fromSec, toSec, gameId),
    refetchInterval: 60_000,
  });

  const result = useMemo(
    () => aggregate(hourly.data ?? [], sessions.data ?? [], range),
    [hourly.data, sessions.data, range],
  );
  return {
    result,
    sessions: sessions.data ?? [],
    isLoading: hourly.isLoading || sessions.isLoading,
    error: hourly.error ?? sessions.error,
  };
}
