import { useEffect, useState } from "react";
import { windowControls } from "../../ipc/window";
import { Icon } from "../Icon";

/** Windows başlık çubuğunun yerini alan sürüklenebilir çubuk ve uygulama stilinde pencere düğmeleri. */
export function TitleBar() {
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const sync = () => {
      void windowControls.isMaximized().then((value) => {
        setMaximized(value);
        document.documentElement.dataset.maximized = String(value);
      });
    };
    sync();
    void windowControls.onResized(sync)?.then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, []);

  return (
    <header className="titlebar" data-tauri-drag-region>
      <div className="titlebar__brand" data-tauri-drag-region>
        <span className="titlebar__mark" aria-hidden />
        <span className="titlebar__name" data-tauri-drag-region>
          Oyun Arşivim
        </span>
      </div>
      <div className="titlebar__controls">
        <button
          type="button"
          className="win-btn"
          aria-label="Küçült"
          onClick={() => void windowControls.minimize()}
        >
          <Icon name="minus" size={14} />
        </button>
        <button
          type="button"
          className="win-btn"
          aria-label={maximized ? "Önceki boyut" : "Büyüt"}
          onClick={() => void windowControls.toggleMaximize()}
        >
          <Icon name={maximized ? "copy" : "square"} size={13} />
        </button>
        <button
          type="button"
          className="win-btn win-btn--close"
          aria-label="Kapat"
          onClick={() => void windowControls.close()}
        >
          <Icon name="x" size={14} />
        </button>
      </div>
    </header>
  );
}
