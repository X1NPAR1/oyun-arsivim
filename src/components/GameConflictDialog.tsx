import { AnimatePresence, motion } from "motion/react";
import { useEffect, useRef, useState } from "react";
import { libraryApi } from "../ipc/library";
import { useMotionLevel } from "../stores/ui";
import { waitUntilStopped } from "../hooks/useRunning";
import { useRunningStore } from "../stores/running";
import { toast } from "../stores/toast";
import { Icon } from "./Icon";

function errorText(err: unknown): string {
  return typeof err === "string" ? err : err instanceof Error ? err.message : "Bilinmeyen hata";
}

/**
 * Aynı anda tek oyun kuralı: bir oyun açıkken başka bir oyun başlatılmak istenirse sorulur.
 * Onaylanırsa açık oyun kapatılır (yalnız o oyunun süreçleri), kapandığı görülünce yenisi açılır.
 */
export function GameConflictDialog() {
  const conflict = useRunningStore((s) => s.conflict);
  const setConflict = useRunningStore((s) => s.setConflict);
  const [busy, setBusy] = useState(false);
  const primary = useRef<HTMLButtonElement>(null);
  const level = useMotionLevel();

  const close = () => {
    if (!busy) setConflict(null);
  };

  useEffect(() => {
    if (!conflict) return;
    primary.current?.focus();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.stopPropagation();
        close();
      }
    };
    window.addEventListener("keydown", onKey, true);
    return () => {
      window.removeEventListener("keydown", onKey, true);
    };
  });

  const switchGame = async () => {
    if (!conflict) return;
    setBusy(true);
    try {
      await libraryApi.stop(conflict.current.id);
      await waitUntilStopped(conflict.current.id);
      // Süreçlerin tamamen çıkması ve başlatıcının toparlanması için kısa bir pay.
      await new Promise((r) => window.setTimeout(r, 1500));
      await libraryApi.launch(conflict.next.id);
      toast.success(`${conflict.current.title} kapatıldı`, `${conflict.next.title} açılıyor…`);
      setConflict(null);
    } catch (e) {
      toast.error("Oyun değiştirilemedi", errorText(e));
    } finally {
      setBusy(false);
    }
  };

  const starting = conflict?.current.starting ?? false;
  const dur = level === "off" ? 0 : 0.22;

  return (
    <AnimatePresence>
      {conflict && (
        <motion.div
          className="dialog-backdrop"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: dur }}
          onMouseDown={(e) => {
            if (e.target === e.currentTarget) close();
          }}
        >
          <motion.div
            className="dialog glass"
            role="alertdialog"
            aria-modal="true"
            aria-labelledby="conflict-title"
            aria-describedby="conflict-desc"
            initial={{ opacity: 0, scale: 0.96, y: 8 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.97, y: 4 }}
            transition={{ duration: dur, ease: [0.22, 1, 0.36, 1] }}
          >
            <span className="dialog__icon" aria-hidden>
              <Icon name="game-controller" size={22} />
            </span>
            <h2 id="conflict-title" className="dialog__title">
              Şu an {conflict.current.title} {starting ? "açılıyor" : "oynuyorsunuz"}
            </h2>
            <p id="conflict-desc" className="dialog__text">
              {starting
                ? `${conflict.current.title} başlatılıyor. Açıldıktan sonra kapatıp ${conflict.next.title} oyununu açabilirsiniz.`
                : `Aynı anda tek oyun açılabilir. İsterseniz ${conflict.current.title} oyununu kapatıp ${conflict.next.title} oyununu açabilirsiniz.`}
            </p>
            {!starting && (
              <p className="dialog__note">
                Kaydedilmemiş ilerleme kaybolabilir; önce oyun içinden kaydetmeniz önerilir.
              </p>
            )}
            <div className="dialog__actions">
              <button type="button" className="btn btn--glass" onClick={close} disabled={busy}>
                {starting ? "Tamam" : "Vazgeç"}
              </button>
              {!starting && (
                <button
                  ref={primary}
                  type="button"
                  className="btn btn--primary"
                  disabled={busy}
                  onClick={() => void switchGame()}
                >
                  {busy ? "Değiştiriliyor…" : `Kapat ve ${conflict.next.title} aç`}
                </button>
              )}
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
