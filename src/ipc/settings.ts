import { invoke } from "@tauri-apps/api/core";
import type { MotionPreference } from "../stores/ui";

/** Rust `settings::Settings` (camelCase). */
export interface AppSettings {
  launchAtStartup: boolean;
  closeToTray: boolean;
  motion: MotionPreference;
}

export const settingsApi = {
  get: () => invoke<AppSettings>("get_settings"),
  update: (patch: Partial<AppSettings>) => invoke<AppSettings>("update_settings", { patch }),
};
