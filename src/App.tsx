import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useEffect } from "react";
import { createHashRouter, RouterProvider } from "react-router";
import { useSyncMotionFromSettings } from "./hooks/useSettings";
import { getWindowBackdrop } from "./ipc/window";
import { routes } from "./routes";
import { useMotionLevel, useUiStore } from "./stores/ui";

// Hash yönlendirme: masaüstünde adres çubuğu yok ve yeniden yüklemede sunucu yönlendirmesi gerekmez.
const router = createHashRouter(routes);

// Veri Rust olaylarıyla tazelenir; pencereye odaklanınca tekrar sorgulamaya gerek yok.
const queryClient = new QueryClient({
  defaultOptions: { queries: { refetchOnWindowFocus: false, staleTime: 30_000, retry: 1 } },
});

/** Animasyon seviyesi ve pencere arka planını <html> özniteliklerine yansıtır (CSS bunları kullanır). */
function useEnvironmentSync() {
  const level = useMotionLevel();
  const backdrop = useUiStore((s) => s.backdrop);
  const setSystemReducedMotion = useUiStore((s) => s.setSystemReducedMotion);
  const setBackdrop = useUiStore((s) => s.setBackdrop);

  useEffect(() => {
    const query = window.matchMedia("(prefers-reduced-motion: reduce)");
    const sync = () => {
      setSystemReducedMotion(query.matches);
    };
    sync();
    query.addEventListener("change", sync);
    return () => {
      query.removeEventListener("change", sync);
    };
  }, [setSystemReducedMotion]);

  useEffect(() => {
    getWindowBackdrop()
      .then(setBackdrop)
      .catch(() => {
        // Tauri dışında (tarayıcı önizlemesi) opak CSS zemini kalır.
      });
  }, [setBackdrop]);

  useEffect(() => {
    document.documentElement.dataset.motion = level;
    document.documentElement.dataset.backdrop = backdrop;
  }, [level, backdrop]);
}

function SettingsBridge() {
  useSyncMotionFromSettings();
  return null;
}

export default function App() {
  useEnvironmentSync();
  return (
    <QueryClientProvider client={queryClient}>
      <SettingsBridge />
      <RouterProvider router={router} />
    </QueryClientProvider>
  );
}
