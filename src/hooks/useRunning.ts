import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { RUNNING_CHANGED_EVENT, libraryApi, type RunningGame } from "../ipc/library";
import { useRunningStore } from "../stores/running";

/** Çalışan oyun listesini Rust'tan alır ve `games://running` olaylarıyla güncel tutar (kabukta bir kez). */
export function useRunningSync() {
  const setGames = useRunningStore((s) => s.setGames);
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let disposed = false;
    libraryApi
      .running()
      .then((games) => {
        if (!disposed) setGames(Array.isArray(games) ? games : []);
      })
      .catch(() => undefined);
    listen<RunningGame[]>(RUNNING_CHANGED_EVENT, (e) => {
      setGames(e.payload);
    })
      .then((fn) => {
        if (disposed) fn();
        else unlisten = fn;
      })
      .catch(() => undefined);
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [setGames]);
}

/** Oyun listeden düşene kadar bekler (en fazla `timeoutMs`). */
export function waitUntilStopped(id: number, timeoutMs = 15000): Promise<boolean> {
  return new Promise((resolve) => {
    const gone = (games: RunningGame[]) => !games.some((g) => g.id === id);
    if (gone(useRunningStore.getState().games)) {
      resolve(true);
      return;
    }
    const timer = window.setTimeout(() => {
      unsub();
      resolve(false);
    }, timeoutMs);
    const unsub = useRunningStore.subscribe((s) => {
      if (gone(s.games)) {
        window.clearTimeout(timer);
        unsub();
        resolve(true);
      }
    });
  });
}
