import { AnimatePresence, motion } from "motion/react";
import { useCallback, useEffect, useRef, useState, type MouseEvent, type WheelEvent } from "react";
import { ease, scaleDuration, showcase, type NavDirection } from "../../design/motion";
import { artUrl, formatRelative, logoOf, PLATFORM_LABELS, type GameView } from "../../ipc/library";
import { relativeOffset, wheelStep, wrapIndex } from "../../library/carousel";
import { preloadImage, trimPreloaded, useStageStore, type StageImage } from "../../stores/stage";
import { formatDuration } from "../../stats/aggregate";
import { useMotionLevel } from "../../stores/ui";
import { Icon } from "../Icon";

interface GameShowcaseProps {
  games: GameView[];
  onPlay: (game: GameView) => void;
  onContextMenu?: (game: GameView, e: MouseEvent) => void;
}

const TILE_W = 176;
const TILE_GAP = 14;
const VISIBLE_RADIUS = 5;
/** Odaktaki oyunun iki yanında önceden çözülen görsel sayısı (bellek sınırlı). */
const PRELOAD_RADIUS = 2;

function stageOf(g: GameView | undefined): StageImage | null {
  if (!g) return null;
  // 16:9 ekran görüntüsü pencereyi neredeyse kırpmadan doldurur; yoksa geniş hero görseli.
  const full = artUrl(g.art.backdrop) ?? artUrl(g.art.hero);
  if (full) return { src: full, ambient: false };
  const ambient = artUrl(g.art.cover) ?? artUrl(g.art.icon) ?? artUrl(g.art.logo);
  return ambient ? { src: ambient, ambient: true } : null;
}

function heroOf(g: GameView | undefined): string | null {
  return stageOf(g)?.src ?? null;
}

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
 * Ana sayfa vitrini. Seçili oyunun görseli tam pencere sahneye (Stage) yazılır; altta sonsuz
 * döngülü logo şeridi. Gezinme: ← → tuşları, fare tekerleği, tıklama, ok düğmeleri; otomatik akış
 * sağdan sola ilerler. Bir sonraki görsel önce çözülür, sonra logo + bilgi + arka plan AYNI anda
 * değişir (görselin geç gelmesi engellenir).
 */
export function GameShowcase({ games, onPlay, onContextMenu }: GameShowcaseProps) {
  const level = useMotionLevel();
  const setStage = useStageStore((s) => s.setImage);
  const count = games.length;
  const [focus, setFocus] = useState(0);
  const [direction, setDirection] = useState<NavDirection>(1);
  const lockedUntil = useRef(0);
  const wheelAcc = useRef(0);
  const navToken = useRef(0);
  const focusRef = useRef(0);

  const current = games[wrapIndex(focus, count)];

  // Odak değişince sahne görselini güncelle ve komşuları önceden hazırla.
  useEffect(() => {
    setStage(stageOf(current));
    const keep: string[] = [];
    for (let d = -PRELOAD_RADIUS; d <= PRELOAD_RADIUS; d++) {
      const src = heroOf(games[wrapIndex(focus + d, count)]);
      if (src) {
        keep.push(src);
        void preloadImage(src);
      }
    }
    trimPreloaded(keep);
  }, [current, focus, games, count, setStage]);

  // Ana sayfadan çıkınca sahne kapanır (çapraz geçişle).
  useEffect(
    () => () => {
      setStage(null);
    },
    [setStage],
  );

  const go = useCallback(
    (step: number, user = true) => {
      if (count === 0 || step === 0) return;
      // Geçiş sürerken yeni komut yok sayılır: geçişler üst üste binmez.
      if (user && Date.now() < lockedUntil.current) return;
      lockedUntil.current = Date.now() + (level === "off" ? 0 : showcase.navLockMs);
      const next = wrapIndex(focusRef.current + step, count);
      const token = ++navToken.current;
      const src = heroOf(games[next]);
      const apply = () => {
        // Bu arada başka bir gezinme olduysa eski isteği uygulama.
        if (token !== navToken.current) return;
        focusRef.current = next;
        setDirection(step > 0 ? 1 : -1);
        setFocus(next);
      };
      // Çözülmüş görselde anında; değilse en fazla 250 ms bekle (yavaş diskte arayüz donmasın).
      if (src) {
        void Promise.race([preloadImage(src), new Promise((r) => setTimeout(r, 250))]).then(apply);
      } else {
        apply();
      }
    },
    [count, games, level],
  );

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      if (
        target &&
        (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable)
      )
        return;
      if (document.querySelector(".context-menu")) return;
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
    let id = 0;
    const schedule = () => {
      id = window.setTimeout(() => {
        // Pencere arka plandayken akış durur; görünür olunca devam eder.
        if (document.visibilityState === "visible") go(1, false);
        else schedule();
      }, showcase.dwellMs);
    };
    schedule();
    return () => {
      window.clearTimeout(id);
    };
  }, [focus, go, level, count]);

  const onWheel = (e: WheelEvent<HTMLElement>) => {
    if (Date.now() < lockedUntil.current) {
      wheelAcc.current = 0;
      return;
    }
    // Dikey tekerlek de yatay şeritte ilerletir; yatay kaydırma (touchpad) doğrudan kullanılır.
    const delta = Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : e.deltaY;
    const [step, rest] = wheelStep(wheelAcc.current, delta);
    wheelAcc.current = rest;
    if (step !== 0) go(step);
  };

  if (!current) return null;

  const move = level === "full";
  const lastPlayed = formatRelative(current.lastPlayed);
  const pt = current.playtime;
  const infoVariants = {
    enter: (d: NavDirection) => ({ opacity: 0, x: move ? 48 * d : 0 }),
    center: { opacity: 1, x: 0 },
    exit: (d: NavDirection) => ({ opacity: 0, x: move ? -48 * d : 0 }),
  };

  return (
    <section
      className="showcase"
      onWheel={onWheel}
      aria-roledescription="carousel"
      aria-label="Oyunların"
    >
      <div className="showcase__content">
        <AnimatePresence mode="popLayout" initial={false} custom={direction}>
          <motion.div
            key={current.id}
            className="showcase__info"
            custom={direction}
            variants={infoVariants}
            initial="enter"
            animate="center"
            exit="exit"
            transition={{
              duration: scaleDuration(showcase.crossfadeS * 0.6, level),
              ease: ease.inOut,
            }}
            onContextMenu={
              onContextMenu
                ? (e) => {
                    onContextMenu(current, e);
                  }
                : undefined
            }
          >
            <GameLogo game={current} className="showcase__logo" />
            <div className="showcase__meta">
              <span className="showcase__chip">{PLATFORM_LABELS[current.platform]}</span>
              {pt.totalSeconds > 0 && (
                <span className="showcase__stat">
                  <Icon name="clock" size={15} />
                  {formatDuration(pt.totalSeconds)}
                </span>
              )}
              {lastPlayed && <span className="showcase__stat">Son oynama {lastPlayed}</span>}
            </div>
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
                  scale: active ? 1 : 0.86,
                  opacity: visible
                    ? active
                      ? 1
                      : Math.max(0.3, 0.75 - Math.abs(offset) * 0.1)
                    : 0,
                }}
                transition={
                  move ? { duration: showcase.crossfadeS * 0.7, ease: ease.inOut } : { duration: 0 }
                }
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
