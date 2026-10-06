import { AnimatePresence, motion } from "motion/react";
import { useNavigate } from "react-router";
import { useRunningStore } from "../../stores/running";
import { useMotionLevel } from "../../stores/ui";

/** Başlık çubuğunda oval gösterge: çalışan oyunun adı + "Oynanıyor" (tıklayınca detay sayfası). */
export function NowPlaying() {
  const game = useRunningStore((s) => s.games[0] ?? null);
  const navigate = useNavigate();
  const level = useMotionLevel();
  const dur = level === "off" ? 0 : 0.35;
  return (
    <AnimatePresence>
      {game && (
        <motion.button
          key={game.id}
          type="button"
          className={`now-playing${game.starting ? " is-starting" : ""}`}
          title={`${game.title} — ayrıntılar`}
          aria-live="polite"
          initial={{ opacity: 0, y: -6, scale: 0.94 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          exit={{ opacity: 0, y: -6, scale: 0.94 }}
          transition={{ duration: dur, ease: [0.22, 1, 0.36, 1] }}
          onClick={() => void navigate(`/kutuphane/${String(game.id)}`)}
        >
          <span className="now-playing__dot" aria-hidden />
          <span className="now-playing__title">{game.title}</span>
          <span className="now-playing__state">{game.starting ? "Açılıyor" : "Oynanıyor"}</span>
        </motion.button>
      )}
    </AnimatePresence>
  );
}
