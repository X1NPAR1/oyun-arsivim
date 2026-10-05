import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useState, type MouseEvent, type ReactNode } from "react";
import { ContextMenu, type MenuEntry } from "../components/ContextMenu";
import { gameActionsApi, type GameView, type PlatformAction } from "../ipc/library";
import { toast } from "../stores/toast";
import { LIBRARY_KEY, useLaunchGame, useRemoveManualGame } from "./useLibrary";

function errorText(err: unknown): string {
  return typeof err === "string" ? err : err instanceof Error ? err.message : "Bilinmeyen hata";
}

function reportAction(action: PlatformAction, done: string) {
  if (action.kind === "guided") toast.info("Platform açıldı", action.value.hint);
  else toast.success(done);
}

/**
 * Bir oyun için sağ tık menüsü. Kaldırma/yükleme platformun kendi onaylı akışıyla yapılır;
 * uygulama oyun dosyalarını kendisi silmez.
 */
export function useGameMenu(): {
  openFor: (game: GameView, e: MouseEvent) => void;
  menu: ReactNode;
} {
  const client = useQueryClient();
  const launch = useLaunchGame();
  const removeManual = useRemoveManualGame();
  const [state, setState] = useState<{ game: GameView; x: number; y: number } | null>(null);
  const close = useCallback(() => {
    setState(null);
  }, []);

  const openFor = useCallback((game: GameView, e: MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setState({ game, x: e.clientX, y: e.clientY });
  }, []);

  const run = (task: () => Promise<unknown>, onError: string) => {
    task().catch((err: unknown) => {
      toast.error(onError, errorText(err));
    });
  };

  let items: MenuEntry[] = [];
  if (state) {
    const g = state.game;
    const canInstall = g.platform !== "manual";
    items = [
      ...(g.installed
        ? ([
            {
              id: "play",
              label: "Oyna",
              icon: "play",
              onSelect: () => {
                launch.mutate(g.id);
              },
            },
            {
              id: "shortcut",
              label: "Masaüstüne kısayol oluştur",
              icon: "monitor",
              onSelect: () => {
                run(async () => {
                  await gameActionsApi.createShortcut(g.id);
                  toast.success("Kısayol oluşturuldu", `${g.title} masaüstüne eklendi.`);
                }, "Kısayol oluşturulamadı");
              },
            },
            {
              id: "folder",
              label: "Oyun klasörünü aç",
              icon: "folder-open",
              onSelect: () => {
                run(() => gameActionsApi.openFolder(g.id), "Klasör açılamadı");
              },
            },
          ] satisfies MenuEntry[])
        : canInstall
          ? ([
              {
                id: "install",
                label: "Yeniden yükle",
                icon: "download-simple",
                onSelect: () => {
                  run(async () => {
                    reportAction(await gameActionsApi.install(g.id), "Yükleme ekranı açıldı");
                  }, "Yükleme başlatılamadı");
                },
              },
            ] satisfies MenuEntry[])
          : []),
      "separator",
      ...(g.installed
        ? ([
            {
              id: "uninstall",
              label: "Oyunu kaldır",
              icon: "prohibit",
              danger: true,
              onSelect: () => {
                run(async () => {
                  reportAction(await gameActionsApi.uninstall(g.id), "Kaldırma ekranı açıldı");
                  // Kaldırma bitince klasör izlenir; şimdilik bir sonraki taramada güncellenir.
                  void client.invalidateQueries({ queryKey: LIBRARY_KEY });
                }, "Kaldırma başlatılamadı");
              },
            },
          ] satisfies MenuEntry[])
        : []),
      ...(g.platform === "manual"
        ? ([
            {
              id: "remove",
              label: "Kütüphaneden çıkar",
              icon: "trash",
              danger: true,
              onSelect: () => {
                removeManual.mutate(g.id);
              },
            },
          ] satisfies MenuEntry[])
        : []),
    ];
    // Baştaki/sondaki veya art arda gelen ayırıcıları temizle.
    items = items.filter(
      (it, i, arr) =>
        it !== "separator" || (i > 0 && i < arr.length - 1 && arr[i - 1] !== "separator"),
    );
  }

  return {
    openFor,
    menu: (
      <ContextMenu
        open={state !== null}
        x={state?.x ?? 0}
        y={state?.y ?? 0}
        items={items}
        label={state ? `${state.game.title} menüsü` : "Oyun menüsü"}
        onClose={close}
      />
    ),
  };
}
