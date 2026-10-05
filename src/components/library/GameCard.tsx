import { motion } from "motion/react";
import { useRef, useState, type MouseEvent, type PointerEvent } from "react";
import { duration, ease, scaleDuration, spring, staggerDelay } from "../../design/motion";
import { artUrl, formatBytes, logoOf, PLATFORM_LABELS, type GameView } from "../../ipc/library";
import { useMotionLevel } from "../../stores/ui";
import { Icon } from "../Icon";

interface GameCardProps {
  game: GameView;
  index: number;
  onPlay: (game: GameView) => void;
  onRemove?: (game: GameView) => void;
  onContextMenu?: (game: GameView, e: MouseEvent) => void;
}

const MAX_TILT_DEG = 6;

export function GameCard({ game, index, onPlay, onRemove, onContextMenu }: GameCardProps) {
  const level = useMotionLevel();
  const artRef = useRef<HTMLDivElement>(null);
  const [coverFailed, setCoverFailed] = useState(false);
  const cover = coverFailed ? null : artUrl(game.art.cover);
  const logo = logoOf(game);
  const banner = artUrl(game.art.hero);
  const removed = !game.installed;
  const meta = [PLATFORM_LABELS[game.platform], formatBytes(game.sizeBytes)].filter(Boolean);

  // Eğim ve parlama: React durumunu değil CSS değişkenlerini günceller (her harekette yeniden çizim yok).
  const onPointerMove = (e: PointerEvent<HTMLDivElement>) => {
    const el = artRef.current;
    if (!el || level !== "full") return;
    const r = el.getBoundingClientRect();
    const px = (e.clientX - r.left) / r.width;
    const py = (e.clientY - r.top) / r.height;
    el.style.setProperty("--ry", `${((px - 0.5) * 2 * MAX_TILT_DEG).toFixed(2)}deg`);
    el.style.setProperty("--rx", `${((0.5 - py) * 2 * MAX_TILT_DEG).toFixed(2)}deg`);
    el.style.setProperty("--shine-x", `${(px * 100).toFixed(1)}%`);
    el.style.setProperty("--shine-y", `${(py * 100).toFixed(1)}%`);
  };
  const onPointerLeave = () => {
    const el = artRef.current;
    if (!el) return;
    el.style.setProperty("--rx", "0deg");
    el.style.setProperty("--ry", "0deg");
  };

  return (
    <motion.article
      layout={level === "full" ? "position" : false}
      className={`game-card${removed ? " is-removed" : ""}`}
      initial={{ opacity: 0, y: level === "full" ? 24 : 0 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, scale: 0.96 }}
      transition={{
        opacity: {
          duration: scaleDuration(duration.slow, level),
          delay: staggerDelay(index),
          ease: ease.out,
        },
        y: {
          duration: scaleDuration(duration.slow, level),
          delay: staggerDelay(index),
          ease: ease.out,
        },
        layout: spring.gentle,
      }}
      aria-label={`${game.title}${removed ? " (silindi)" : ""}`}
      onContextMenu={
        onContextMenu
          ? (e) => {
              onContextMenu(game, e);
            }
          : undefined
      }
    >
      <div
        ref={artRef}
        className="game-card__art"
        onPointerMove={onPointerMove}
        onPointerLeave={onPointerLeave}
      >
        {cover ? (
          <img
            className="game-card__cover"
            src={cover}
            alt=""
            loading="lazy"
            decoding="async"
            draggable={false}
            onError={() => {
              setCoverFailed(true);
            }}
          />
        ) : (
          <div className="game-card__fallback">
            {banner && !logo ? (
              <img className="game-card__banner" src={banner} alt="" draggable={false} />
            ) : logo ? (
              <img
                className={`game-card__logo game-card__logo--${logo.kind}`}
                src={logo.src}
                alt=""
                draggable={false}
              />
            ) : (
              <span className="game-card__monogram" aria-hidden>
                {game.title.slice(0, 1).toLocaleUpperCase("tr-TR")}
              </span>
            )}
            <span className="game-card__fallback-title">{game.title}</span>
          </div>
        )}
        <span className="game-card__shine" aria-hidden />
        {removed && <span className="badge badge--removed">Silindi</span>}
        <div className="game-card__actions">
          {!removed && (
            <button
              type="button"
              className="icon-btn icon-btn--light"
              aria-label={`${game.title} oyununu oyna`}
              onClick={() => {
                onPlay(game);
              }}
            >
              <Icon name="play" size={18} filled />
            </button>
          )}
          {onRemove && game.platform === "manual" && (
            <button
              type="button"
              className="icon-btn"
              aria-label={`${game.title} oyununu kütüphaneden çıkar`}
              onClick={() => {
                onRemove(game);
              }}
            >
              <Icon name="trash" size={16} />
            </button>
          )}
        </div>
      </div>
      <div className="game-card__meta">
        <h3 className="game-card__title" title={game.title}>
          {game.title}
        </h3>
        <p className="game-card__sub">{meta.join(" · ")}</p>
      </div>
    </motion.article>
  );
}
