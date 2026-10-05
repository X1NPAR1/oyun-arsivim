import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { LIBRARY_CHANGED_EVENT, libraryApi } from "../ipc/library";
import { toast } from "../stores/toast";

export const LIBRARY_KEY = ["library"] as const;

/** Kütüphane listesi; Rust `library://changed` gönderdikçe (tarama, görsel indirme) tazelenir. */
export function useLibrary() {
  const client = useQueryClient();

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let disposed = false;
    listen(LIBRARY_CHANGED_EVENT, () => {
      void client.invalidateQueries({ queryKey: LIBRARY_KEY });
    })
      .then((fn) => {
        if (disposed) fn();
        else unlisten = fn;
      })
      .catch(() => {
        // Tauri dışında (testler) olay sistemi yok.
      });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [client]);

  return useQuery({ queryKey: LIBRARY_KEY, queryFn: libraryApi.list });
}

function errorText(err: unknown): string {
  return typeof err === "string" ? err : err instanceof Error ? err.message : "Bilinmeyen hata";
}

export function useScanLibrary() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: libraryApi.scan,
    onSuccess: (r) => {
      void client.invalidateQueries({ queryKey: LIBRARY_KEY });
      const parts = [`${r.found} oyun bulundu`];
      if (r.added) parts.push(`${r.added} yeni`);
      if (r.removed) parts.push(`${r.removed} kaldırılmış`);
      toast.success("Tarama tamamlandı", parts.join(" · "));
    },
    onError: (e) => {
      toast.error("Tarama başarısız", errorText(e));
    },
  });
}

export function useAddManualGame() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (exePath: string) => libraryApi.addManual(exePath),
    onSuccess: (game) => {
      void client.invalidateQueries({ queryKey: LIBRARY_KEY });
      toast.success("Oyun eklendi", game.title);
    },
    onError: (e) => {
      toast.error("Oyun eklenemedi", errorText(e));
    },
  });
}

export function useRemoveManualGame() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => libraryApi.removeManual(id),
    onSuccess: () => {
      void client.invalidateQueries({ queryKey: LIBRARY_KEY });
      toast.success("Kütüphaneden çıkarıldı", "Diskteki dosyalara dokunulmadı.");
    },
    onError: (e) => {
      toast.error("Çıkarılamadı", errorText(e));
    },
  });
}

export function useLaunchGame() {
  return useMutation({
    mutationFn: (id: number) => libraryApi.launch(id),
    onError: (e) => {
      toast.error("Oyun başlatılamadı", errorText(e));
    },
  });
}
