import { AnimatePresence, motion } from "motion/react";
import { useEffect, useMemo, useRef, useState } from "react";
import { duration, ease } from "../design/motion";
import { logoOf, type GameView } from "../ipc/library";
import { searchKey } from "../library/filter";
import { Icon } from "./Icon";

interface GamePickerProps {
  games: GameView[];
  value: number | null;
  onChange: (id: number | null) => void;
}

function Thumb({ game }: { game: GameView }) {
  const logo = logoOf(game);
  return logo?.kind === "icon" ? (
    <img className="picker__thumb picker__thumb--icon" src={logo.src} alt="" />
  ) : (
    <span className="picker__thumb picker__thumb--mono">{game.title.slice(0, 1)}</span>
  );
}

/** Aranabilir oyun seçici (klavye: ↑/↓, Enter, Esc). */
export function GamePicker({ games, value, onChange }: GamePickerProps) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [cursor, setCursor] = useState(0);
  const ref = useRef<HTMLDivElement>(null);
  const selected = games.find((g) => g.id === value) ?? null;

  const options = useMemo(() => {
    const q = searchKey(query);
    const list = q ? games.filter((g) => searchKey(g.title).includes(q)) : games;
    return [null, ...list] as (GameView | null)[];
  }, [games, query]);

  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (!ref.current?.contains(e.target as Node)) setOpen(false);
    };
    window.addEventListener("mousedown", onDown);
    return () => {
      window.removeEventListener("mousedown", onDown);
    };
  }, [open]);

  const choose = (g: GameView | null) => {
    onChange(g?.id ?? null);
    setOpen(false);
    setQuery("");
  };

  return (
    <div ref={ref} className="picker">
      <button
        type="button"
        className="picker__button"
        aria-haspopup="listbox"
        aria-expanded={open}
        onClick={() => {
          setOpen((o) => !o);
          setCursor(0);
        }}
      >
        {selected ? <Thumb game={selected} /> : <Icon name="squares-four" size={18} />}
        <span className="picker__label">{selected?.title ?? "Tüm oyunlar"}</span>
        <Icon name="caret-right" size={14} className={`picker__caret${open ? " is-open" : ""}`} />
      </button>
      <AnimatePresence>
        {open && (
          <motion.div
            className="picker__panel glass"
            initial={{ opacity: 0, y: -6 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -4 }}
            transition={{ duration: duration.fast, ease: ease.out }}
          >
            <label className="search picker__search">
              <Icon name="magnifying-glass" size={16} />
              <input
                autoFocus
                placeholder="Oyun ara"
                aria-label="Oyun ara"
                value={query}
                onChange={(e) => {
                  setQuery(e.target.value);
                  setCursor(0);
                }}
                onKeyDown={(e) => {
                  if (e.key === "ArrowDown") {
                    e.preventDefault();
                    setCursor((c) => Math.min(c + 1, options.length - 1));
                  } else if (e.key === "ArrowUp") {
                    e.preventDefault();
                    setCursor((c) => Math.max(c - 1, 0));
                  } else if (e.key === "Enter") {
                    choose(options[cursor] ?? null);
                  } else if (e.key === "Escape") {
                    setOpen(false);
                  }
                }}
              />
            </label>
            <ul className="picker__list" role="listbox" aria-label="Oyunlar">
              {options.map((g, i) => (
                <li key={g?.id ?? "all"}>
                  <button
                    type="button"
                    role="option"
                    aria-selected={(g?.id ?? null) === value}
                    className={`picker__option${i === cursor ? " is-cursor" : ""}`}
                    onMouseEnter={() => {
                      setCursor(i);
                    }}
                    onClick={() => {
                      choose(g);
                    }}
                  >
                    {g ? <Thumb game={g} /> : <Icon name="squares-four" size={18} />}
                    <span>{g?.title ?? "Tüm oyunlar"}</span>
                    {(g?.id ?? null) === value && (
                      <Icon name="check" size={16} className="picker__check" />
                    )}
                  </button>
                </li>
              ))}
            </ul>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
