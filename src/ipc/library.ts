import { convertFileSrc, invoke } from "@tauri-apps/api/core";

export type Platform = "steam" | "epic" | "manual";

/** Rust `artwork::Artwork` — yerel önbellek dosya yolları. */
export interface Artwork {
  cover: string | null;
  logo: string | null;
  hero: string | null;
  icon: string | null;
}

/** Rust `commands::GameView` (camelCase). */
export interface GameView {
  id: number;
  platform: Platform;
  platformId: string;
  title: string;
  installDir: string;
  executable: string | null;
  launchUri: string;
  sizeBytes: number | null;
  lastPlayed: number | null;
  installed: boolean;
  addedAt: number;
  removedAt: number | null;
  art: Artwork;
}

export interface SyncReport {
  found: number;
  added: number;
  removed: number;
  restored: number;
}

export const LIBRARY_CHANGED_EVENT = "library://changed";

export const PLATFORM_LABELS: Record<Platform, string> = {
  steam: "Steam",
  epic: "Epic Games",
  manual: "Elle eklendi",
};

export const libraryApi = {
  list: () => invoke<GameView[]>("list_games"),
  scan: () => invoke<SyncReport>("scan_library"),
  addManual: (exePath: string, title?: string) =>
    invoke<GameView>("add_manual_game", { exePath, title: title ?? null }),
  removeManual: (id: number) => invoke<null>("remove_manual_game", { id }),
  rename: (id: number, title: string) => invoke<null>("rename_game", { id, title }),
  launch: (id: number) => invoke<null>("launch_game", { id }),
};

/** Önbellekteki görseli webview'de gösterilebilir `asset:` adresine çevirir. */
export function artUrl(path: string | null): string | null {
  return path ? convertFileSrc(path) : null;
}

/** Kartın logosu: Steam logosu → exe ikonu → yok (tipografik logo). */
export function logoOf(game: GameView): { src: string; kind: "logo" | "icon" } | null {
  const logo = artUrl(game.art.logo);
  if (logo) return { src: logo, kind: "logo" };
  const icon = artUrl(game.art.icon);
  return icon ? { src: icon, kind: "icon" } : null;
}

export function formatBytes(bytes: number | null): string | null {
  if (bytes === null || bytes <= 0) return null;
  const gb = bytes / 1024 ** 3;
  if (gb >= 1) return `${gb.toLocaleString("tr-TR", { maximumFractionDigits: 1 })} GB`;
  const mb = bytes / 1024 ** 2;
  return `${Math.max(1, Math.round(mb)).toLocaleString("tr-TR")} MB`;
}

/** "3 gün önce" gibi göreli zaman; Unix saniye alır. */
export function formatRelative(unixSeconds: number | null, now = Date.now()): string | null {
  if (!unixSeconds) return null;
  const diff = Math.max(0, now / 1000 - unixSeconds);
  const minute = 60;
  const hour = 60 * minute;
  const day = 24 * hour;
  if (diff < hour) return "az önce";
  if (diff < day) return `${Math.floor(diff / hour)} saat önce`;
  if (diff < 30 * day) return `${Math.floor(diff / day)} gün önce`;
  if (diff < 365 * day) return `${Math.floor(diff / (30 * day))} ay önce`;
  return `${Math.floor(diff / (365 * day))} yıl önce`;
}

/** Rust `game_actions::PlatformAction`. */
export type PlatformAction =
  { kind: "openUri"; value: string } | { kind: "guided"; value: { uri: string; hint: string } };

export const gameActionsApi = {
  uninstall: (id: number) => invoke<PlatformAction>("uninstall_game", { id }),
  install: (id: number) => invoke<PlatformAction>("install_game", { id }),
  createShortcut: (id: number) => invoke<string>("create_desktop_shortcut", { id }),
  openFolder: (id: number) => invoke<null>("open_game_folder", { id }),
};
