import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";
import { settingsApi, type AppSettings } from "../ipc/settings";
import { toast } from "../stores/toast";
import { useUiStore } from "../stores/ui";

export const SETTINGS_KEY = ["settings"] as const;

export function useSettings() {
  return useQuery({ queryKey: SETTINGS_KEY, queryFn: settingsApi.get });
}

/** Ayarı anında uygular (iyimser güncelleme); Rust reddederse eski değere döner ve sebebi gösterir. */
export function useUpdateSettings() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (patch: Partial<AppSettings>) => settingsApi.update(patch),
    onMutate: async (patch) => {
      await client.cancelQueries({ queryKey: SETTINGS_KEY });
      const previous = client.getQueryData<AppSettings>(SETTINGS_KEY);
      if (previous) client.setQueryData(SETTINGS_KEY, { ...previous, ...patch });
      return { previous };
    },
    onError: (err, _patch, ctx) => {
      if (ctx?.previous) client.setQueryData(SETTINGS_KEY, ctx.previous);
      toast.error("Ayar kaydedilemedi", typeof err === "string" ? err : String(err));
    },
    onSuccess: (saved) => {
      client.setQueryData(SETTINGS_KEY, saved);
    },
  });
}

/** Kalıcı animasyon tercihini (SQLite) arayüz durumuna aktarır. */
export function useSyncMotionFromSettings() {
  const { data } = useSettings();
  const setMotion = useUiStore((s) => s.setMotionPreference);
  useEffect(() => {
    if (data) setMotion(data.motion);
  }, [data, setMotion]);
}
