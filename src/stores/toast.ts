import { create } from "zustand";

export type ToastKind = "success" | "error" | "info";

export interface Toast {
  id: number;
  kind: ToastKind;
  title: string;
  message?: string;
}

interface ToastState {
  toasts: Toast[];
  push: (t: Omit<Toast, "id">) => void;
  dismiss: (id: number) => void;
}

const LIFETIME_MS = 4500;
let nextId = 1;

export const useToastStore = create<ToastState>((set, get) => ({
  toasts: [],
  push: (t) => {
    const id = nextId++;
    // En fazla 3 bildirim; eskiler düşer.
    set({ toasts: [...get().toasts.slice(-2), { ...t, id }] });
    setTimeout(() => {
      get().dismiss(id);
    }, LIFETIME_MS);
  },
  dismiss: (id) => {
    set({ toasts: get().toasts.filter((t) => t.id !== id) });
  },
}));

export const toast = {
  success: (title: string, message?: string) => {
    useToastStore.getState().push({ kind: "success", title, message });
  },
  error: (title: string, message?: string) => {
    useToastStore.getState().push({ kind: "error", title, message });
  },
  info: (title: string, message?: string) => {
    useToastStore.getState().push({ kind: "info", title, message });
  },
};
