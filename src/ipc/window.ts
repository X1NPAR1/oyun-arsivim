import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Backdrop } from "../stores/ui";

export function getWindowBackdrop(): Promise<Backdrop> {
  return invoke<Backdrop>("window_backdrop");
}

/** Pencere denetimleri; Tauri dışında (tarayıcıda önizleme, testler) sessizce no-op. */
function current() {
  try {
    return getCurrentWindow();
  } catch {
    return null;
  }
}

export const windowControls = {
  minimize: () => current()?.minimize(),
  toggleMaximize: () => current()?.toggleMaximize(),
  close: () => current()?.close(),
  show: () => current()?.show(),
  isMaximized: async () => (await current()?.isMaximized()) ?? false,
  onResized: (cb: () => void) => current()?.onResized(cb),
};
