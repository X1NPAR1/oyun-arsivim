import { create } from "zustand";
import type { MotionLevel } from "../design/motion";

/** Kullanıcı tercihi; "system" Windows'un "animasyonları azalt" ayarını izler. */
export type MotionPreference = "system" | MotionLevel;
export type Backdrop = "mica" | "none";

export function resolveMotionLevel(pref: MotionPreference, systemReduced: boolean): MotionLevel {
  if (pref !== "system") return pref;
  return systemReduced ? "reduced" : "full";
}

interface UiState {
  motionPreference: MotionPreference;
  systemReducedMotion: boolean;
  backdrop: Backdrop;
  setMotionPreference: (pref: MotionPreference) => void;
  setSystemReducedMotion: (reduced: boolean) => void;
  setBackdrop: (backdrop: Backdrop) => void;
}

export const useUiStore = create<UiState>((set) => ({
  // Kalıcı değer SQLite ayarlarından gelir (useSyncMotionFromSettings).
  motionPreference: "system",
  systemReducedMotion: false,
  backdrop: "none",
  setMotionPreference: (motionPreference) => {
    set({ motionPreference });
  },
  setSystemReducedMotion: (systemReducedMotion) => {
    set({ systemReducedMotion });
  },
  setBackdrop: (backdrop) => {
    set({ backdrop });
  },
}));

export function useMotionLevel(): MotionLevel {
  return useUiStore((s) => resolveMotionLevel(s.motionPreference, s.systemReducedMotion));
}
