import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { AnimatePresence } from "motion/react";
import { useDeferredValue, useEffect, useMemo, useState } from "react";
import { EmptyState } from "../components/EmptyState";
import { Icon } from "../components/Icon";
import { GameCard } from "../components/library/GameCard";
import { PageHeader } from "../components/PageHeader";
import {
  useAddManualGame,
  useLaunchGame,
  useLibrary,
  useRemoveManualGame,
  useScanLibrary,
} from "../hooks/useLibrary";
import { useGameMenu } from "../hooks/useGameMenu";
import { PLATFORM_LABELS, type GameView, type Platform } from "../ipc/library";
import { filterGames, type InstallFilter } from "../library/filter";

const INSTALL_FILTERS: { value: InstallFilter; label: string }[] = [
  { value: "all", label: "Tümü" },
  { value: "installed", label: "Kurulu" },
  { value: "removed", label: "Silinenler" },
];

export function LibraryPage() {
  const { data: games = [], isLoading } = useLibrary();
  const scan = useScanLibrary();
  const addManual = useAddManualGame();
  const removeManual = useRemoveManualGame();
  const launch = useLaunchGame();
  const [query, setQuery] = useState("");
  const [install, setInstall] = useState<InstallFilter>("all");
  const [platform, setPlatform] = useState<Platform | "all">("all");
  const deferredQuery = useDeferredValue(query);
  const gameMenu = useGameMenu();

  const counts = useMemo(
    () => ({
      all: games.length,
      installed: games.filter((g) => g.installed).length,
      removed: games.filter((g) => !g.installed).length,
    }),
    [games],
  );
  const platforms = useMemo(
    () => [...new Set(games.map((g) => g.platform))] as Platform[],
    [games],
  );
  const visible = useMemo(
    () => filterGames(games, { query: deferredQuery, install, platform }),
    [games, deferredQuery, install, platform],
  );

  const pickExe = async () => {
    const selected = await open({
      title: "Oyunun .exe dosyasını seç",
      multiple: false,
      directory: false,
      filters: [{ name: "Oyun (.exe)", extensions: ["exe"] }],
    });
    if (typeof selected === "string") addManual.mutate(selected);
  };

  // Pencereye sürüklenip bırakılan .exe dosyaları kütüphaneye eklenir.
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let disposed = false;
    try {
      getCurrentWebview()
        .onDragDropEvent((event) => {
          if (event.payload.type !== "drop") return;
          for (const path of event.payload.paths) {
            if (path.toLowerCase().endsWith(".exe")) addManual.mutate(path);
          }
        })
        .then((fn) => {
          if (disposed) fn();
          else unlisten = fn;
        })
        .catch(() => undefined);
    } catch {
      // Tauri dışında sürükle-bırak yok.
    }
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [addManual]);

  const onPlay = (g: GameView) => {
    launch.mutate(g.id);
  };
  const onRemove = (g: GameView) => {
    removeManual.mutate(g.id);
  };

  return (
    <div className="page-content">
      <PageHeader
        eyebrow="Kütüphane"
        title="Oyunların"
        subtitle={
          isLoading
            ? "Yükleniyor…"
            : `${counts.installed} kurulu oyun${counts.removed ? ` · ${counts.removed} silinmiş` : ""}`
        }
        actions={
          <>
            <button
              type="button"
              className="btn btn--glass"
              disabled={scan.isPending}
              onClick={() => {
                scan.mutate();
              }}
            >
              <Icon
                name="arrows-clockwise"
                size={18}
                className={scan.isPending ? "is-spinning" : undefined}
              />
              {scan.isPending ? "Taranıyor…" : "Yeniden tara"}
            </button>
            <button
              type="button"
              className="btn btn--primary"
              disabled={addManual.isPending}
              onClick={() => void pickExe()}
            >
              <Icon name="plus" size={18} />
              Oyun ekle
            </button>
          </>
        }
      />

      <div className="toolbar">
        <label className="search">
          <Icon name="magnifying-glass" size={18} />
          <input
            type="search"
            placeholder="Oyun ara"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
            }}
            aria-label="Oyun ara"
          />
        </label>
        <div className="chip-group" role="radiogroup" aria-label="Kurulum durumu">
          {INSTALL_FILTERS.map((f) => (
            <button
              key={f.value}
              type="button"
              role="radio"
              aria-checked={install === f.value}
              className={`chip${install === f.value ? " is-selected" : ""}`}
              onClick={() => {
                setInstall(f.value);
              }}
            >
              {f.label}
              <span className="chip__count">{counts[f.value]}</span>
            </button>
          ))}
        </div>
        {platforms.length > 1 && (
          <div className="chip-group" role="radiogroup" aria-label="Platform">
            {(["all", ...platforms] as const).map((p) => (
              <button
                key={p}
                type="button"
                role="radio"
                aria-checked={platform === p}
                className={`chip${platform === p ? " is-selected" : ""}`}
                onClick={() => {
                  setPlatform(p);
                }}
              >
                {p === "all" ? "Tüm platformlar" : PLATFORM_LABELS[p]}
              </button>
            ))}
          </div>
        )}
      </div>

      {!isLoading && games.length === 0 ? (
        <EmptyState
          icon="game-controller"
          title="Henüz oyun bulunamadı"
          description="Steam ve Epic Games kütüphanelerin taranıyor. Başka bir oyunu eklemek için .exe dosyasını seç ya da pencereye sürükle."
        />
      ) : visible.length === 0 && !isLoading ? (
        <EmptyState
          icon="magnifying-glass"
          title="Eşleşen oyun yok"
          description="Aramayı veya filtreleri değiştirmeyi dene."
        />
      ) : (
        <div className="game-grid">
          <AnimatePresence mode="popLayout" initial={false}>
            {visible.map((g, i) => (
              <GameCard
                key={g.id}
                game={g}
                index={i}
                onPlay={onPlay}
                onRemove={onRemove}
                onContextMenu={gameMenu.openFor}
              />
            ))}
          </AnimatePresence>
        </div>
      )}
      {gameMenu.menu}
    </div>
  );
}
