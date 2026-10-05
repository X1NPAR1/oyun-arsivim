import { create } from "zustand";
import type { MotionLevel } from "../design/motion";

/** Kullanıcı tercihi; "system" Windows'un "animasyonları azalt" ayarını izler. */
export type MotionPreference = "system" | MotionLevel;
export type Backdrop = "mica" | "none";

const STORAGE_KEY = "oa.motionPreference";

function readPreference(): MotionPreference {
  // Faz 4'te ayarlar SQLite'a taşınana kadar geçici kalıcılık.
  try {
    const value = localStorage.getItem(STORAGE_KEY);
    if (value === "system" || value === "full" || value === "reduced" || value === "off") {
      return value;
    }
  } catch {
    // Depolama erişilemiyorsa varsayılan kullanılır.
  }
  return "system";
}

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
  motionPreference: readPreference(),
  systemReducedMotion: false,
  backdrop: "none",
  setMotionPreference: (motionPreference) => {
    try {
      localStorage.setItem(STORAGE_KEY, motionPreference);
    } catch {
      // Kalıcılık başarısız olsa da oturum içinde ayar geçerli kalır.
    }
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
