import { AnimatePresence, motion } from "motion/react";
import { useCallback, useEffect, useRef, useState, type MouseEvent, type WheelEvent } from "react";
import { duration, ease, scaleDuration, spring, type NavDirection } from "../../design/motion";
import { artUrl, formatRelative, logoOf, PLATFORM_LABELS, type GameView } from "../../ipc/library";
import { relativeOffset, wheelStep, wrapIndex } from "../../library/carousel";
import { useMotionLevel } from "../../stores/ui";
import { Icon } from "../Icon";

interface GameShowcaseProps {
  games: GameView[];
  onPlay: (game: GameView) => void;
  onContextMenu?: (game: GameView, e: MouseEvent) => void;
}

const AUTOPLAY_MS = 8000;
/** Kullanıcı etkileşiminden sonra otomatik akış bu süre duraklar. */
const RESUME_AFTER_MS = 15000;
const WHEEL_COOLDOWN_MS = 280;
const TILE_W = 168;
const TILE_GAP = 14;
const VISIBLE_RADIUS = 4;

function GameLogo({ game, className }: { game: GameView; className: string }) {
  const logo = logoOf(game);
  if (logo?.kind === "logo") {
    return (
      <img
        className={`${className} ${className}--logo`}
        src={logo.src}
        alt={game.title}
        draggable={false}
      />
    );
  }
  return (
    <span className={`${className} ${className}--text`}>
      {logo && <img className={`${className}__icon`} src={logo.src} alt="" draggable={false} />}
      <span>{game.title}</span>
    </span>
  );
}

/**
 * Ana sayfa vitrini: seçili oyunun görseli sinematik arka plan, altında sonsuz döngülü logo şeridi.
 * Gezinme: ← → tuşları, fare tekerleği, tıklama, ok düğmeleri; otomatik akış sağdan sola ilerler.
 */
export function GameShowcase({ games, onPlay, onContextMenu }: GameShowcaseProps) {
  const level = useMotionLevel();
  const count = games.length;
  const [focus, setFocus] = useState(0);
  const [direction, setDirection] = useState<NavDirection>(1);
  const pausedUntil = useRef(0);
  const wheelAcc = useRef(0);
  const lastWheel = useRef(0);

  const go = useCallback(
    (step: number, user = true) => {
      if (count === 0 || step === 0) return;
      setDirection(step > 0 ? 1 : -1);
      setFocus((f) => wrapIndex(f + step, count));
      if (user) pausedUntil.current = Date.now() + RESUME_AFTER_MS;
    },
    [count],
  );

  // Liste küçülürse odak sınır dışında kalmasın.
  const current = games[wrapIndex(focus, count)];

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      if (
        target &&
        (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable)
      )
        return;
      if (e.key === "ArrowRight") {
        e.preventDefault();
        go(1);
      } else if (e.key === "ArrowLeft") {
        e.preventDefault();
        go(-1);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, [go]);

  useEffect(() => {
    if (level === "off" || count < 2) return;
    const id = window.setInterval(() => {
      if (Date.now() >= pausedUntil.current && document.visibilityState === "visible") go(1, false);
    }, AUTOPLAY_MS);
    return () => {
      window.clearInterval(id);
    };
  }, [go, level, count]);

  const onWheel = (e: WheelEvent<HTMLElement>) => {
    const now = Date.now();
    if (now - lastWheel.current < WHEEL_COOLDOWN_MS) return;
    // Dikey tekerlek de yatay şeritte ilerletir; yatay kaydırma (touchpad) doğrudan kullanılır.
    const delta = Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : e.deltaY;
    const [step, rest] = wheelStep(wheelAcc.current, delta);
    wheelAcc.current = rest;
    if (step !== 0) {
      lastWheel.current = now;
      go(step);
    }
  };

  if (!current) return null;

  const hero = artUrl(current.art.hero) ?? artUrl(current.art.cover);
  const move = level === "full";
  const lastPlayed = formatRelative(current.lastPlayed);
  const infoVariants = {
    enter: (d: NavDirection) => ({ opacity: 0, x: move ? 40 * d : 0 }),
    center: { opacity: 1, x: 0 },
    exit: (d: NavDirection) => ({ opacity: 0, x: move ? -40 * d : 0 }),
  };

  return (
    <section
      className="showcase"
      onWheel={onWheel}
      aria-roledescription="carousel"
      aria-label="Oyunların"
    >
      <div className="showcase__stage" aria-hidden>
        <AnimatePresence initial={false}>
          <motion.div
            key={current.id}
            className={`showcase__backdrop${hero ? "" : " is-empty"}`}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: scaleDuration(0.7, level), ease: ease.inOut }}
          >
            {hero && (
              <img
                className={`showcase__image${move ? " is-drifting" : ""}`}
                src={hero}
                alt=""
                draggable={false}
              />
            )}
          </motion.div>
        </AnimatePresence>
        <div className="showcase__scrim" />
      </div>

      <div className="showcase__content">
        <AnimatePresence mode="popLayout" initial={false} custom={direction}>
          <motion.div
            key={current.id}
            className="showcase__info"
            onContextMenu={
              onContextMenu
                ? (e) => {
                    onContextMenu(current, e);
                  }
                : undefined
            }
            custom={direction}
            variants={infoVariants}
            initial="enter"
            animate="center"
            exit="exit"
            transition={{ duration: scaleDuration(duration.slow, level), ease: ease.out }}
          >
            <GameLogo game={current} className="showcase__logo" />
            <p className="showcase__meta">
              {PLATFORM_LABELS[current.platform]}
              {lastPlayed && <> · Son oynama {lastPlayed}</>}
            </p>
            <div className="showcase__actions">
              <button
                type="button"
                className="btn btn--primary btn--lg"
                onClick={() => {
                  onPlay(current);
                }}
              >
                <Icon name="play" size={20} filled />
                Oyna
              </button>
            </div>
          </motion.div>
        </AnimatePresence>
      </div>

      <div className="rail">
        <button
          type="button"
          className="rail__nav"
          aria-label="Önceki oyun"
          onClick={() => {
            go(-1);
          }}
        >
          <Icon name="arrow-left" size={18} />
        </button>
        <div className="rail__viewport">
          {games.map((g, i) => {
            const offset = relativeOffset(i, focus, count);
            const visible = Math.abs(offset) <= VISIBLE_RADIUS;
            const active = offset === 0;
            return (
              <motion.button
                key={g.id}
                type="button"
                className={`rail__tile${active ? " is-active" : ""}`}
                aria-label={g.title}
                aria-current={active ? "true" : undefined}
                tabIndex={visible ? 0 : -1}
                initial={false}
                animate={{
                  x: offset * (TILE_W + TILE_GAP),
                  scale: active ? 1 : 0.88,
                  opacity: visible ? (active ? 1 : 0.55) : 0,
                }}
                transition={move ? spring.gentle : { duration: 0 }}
                style={{ pointerEvents: visible ? "auto" : "none" }}
                onClick={() => {
                  go(offset);
                }}
                onContextMenu={
                  onContextMenu
                    ? (e) => {
                        onContextMenu(g, e);
                      }
                    : undefined
                }
              >
                <GameLogo game={g} className="rail__logo" />
              </motion.button>
            );
          })}
        </div>
        <button
          type="button"
          className="rail__nav"
          aria-label="Sonraki oyun"
          onClick={() => {
            go(1);
          }}
        >
          <Icon name="arrow-right" size={18} />
        </button>
      </div>
    </section>
  );
}
