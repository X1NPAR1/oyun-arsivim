import { AnimatePresence, motion } from "motion/react";
import { spring } from "../design/motion";
import { useToastStore } from "../stores/toast";
import { Icon } from "./Icon";

const ICONS = { success: "check", error: "warning", info: "info" } as const;

export function Toaster() {
  const toasts = useToastStore((s) => s.toasts);
  const dismiss = useToastStore((s) => s.dismiss);
  return (
    <div className="toaster" role="status" aria-live="polite">
      <AnimatePresence initial={false}>
        {toasts.map((t) => (
          <motion.div
            key={t.id}
            layout
            className={`toast toast--${t.kind} glass`}
            initial={{ opacity: 0, y: 16, scale: 0.98 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, x: 24 }}
            transition={spring.snappy}
          >
            <Icon name={ICONS[t.kind]} size={18} filled className="toast__icon" />
            <div className="toast__body">
              <p className="toast__title">{t.title}</p>
              {t.message && <p className="toast__message">{t.message}</p>}
            </div>
            <button
              type="button"
              className="toast__close"
              aria-label="Kapat"
              onClick={() => {
                dismiss(t.id);
              }}
            >
              <Icon name="x" size={14} />
            </button>
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  );
}
