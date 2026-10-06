import { create } from "zustand";
import type { RunningGame } from "../ipc/library";

/** Başka bir oyun çalışırken başlatma isteği: kullanıcıya sorulur. */
export interface LaunchConflict {
  current: RunningGame;
  next: { id: number; title: string };
}

interface RunningState {
  games: RunningGame[];
  conflict: LaunchConflict | null;
  setGames: (games: RunningGame[]) => void;
  setConflict: (conflict: LaunchConflict | null) => void;
}

/** Çalışan oyunlar (Rust `games://running`) ve bekleyen "aynı anda tek oyun" sorusu. */
export const useRunningStore = create<RunningState>((set) => ({
  games: [],
  conflict: null,
  setGames: (games) => {
    set({ games });
  },
  setConflict: (conflict) => {
    set({ conflict });
  },
}));

/** Başlatma, kullanıcıya soru sorulduğu için durduruldu (hata bildirimi gösterilmez). */
export class LaunchConflictError extends Error {
  constructor() {
    super("Başka bir oyun çalışıyor");
    this.name = "LaunchConflictError";
  }
}
