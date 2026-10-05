import { useEffect, useState } from "react";
import { getAppInfo, type AppInfo } from "./ipc/appInfo";

type LoadState =
  { kind: "loading" } | { kind: "ready"; info: AppInfo } | { kind: "error"; message: string };

export default function App() {
  const [state, setState] = useState<LoadState>({ kind: "loading" });

  useEffect(() => {
    let cancelled = false;
    getAppInfo()
      .then((info) => {
        if (!cancelled) setState({ kind: "ready", info });
      })
      .catch((err: unknown) => {
        if (!cancelled) setState({ kind: "error", message: String(err) });
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <main className="app-root">
      <h1>Oyun Arşivim</h1>
      {state.kind === "loading" && <p>Yükleniyor…</p>}
      {state.kind === "ready" && (
        <p data-testid="version">
          {state.info.name} v{state.info.version}
        </p>
      )}
      {state.kind === "error" && <p role="alert">Sürüm bilgisi alınamadı: {state.message}</p>}
    </main>
  );
}
