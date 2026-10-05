// Hareket token'ları (docs/DESIGN_SYSTEM.md → Hareket). CSS tarafı tokens.css'te; bu dosya Framer Motion içindir.
import type { Transition } from "motion/react";

export type MotionLevel = "full" | "reduced" | "off";

export const duration = {
  fast: 0.15,
  base: 0.24,
  page: 0.38,
  slow: 0.45,
} as const;

export const ease = {
  out: [0.22, 1, 0.36, 1],
  inOut: [0.65, 0, 0.35, 1],
} as const;

export const spring = {
  snappy: { type: "spring", stiffness: 420, damping: 34 },
  gentle: { type: "spring", stiffness: 220, damping: 28 },
} as const satisfies Record<string, Transition>;

/** Kart girişlerinde öğe başına gecikme; ilk 12 öğeden sonra hepsi birlikte gelir. */
export const STAGGER_STEP = 0.035;
export const STAGGER_MAX_ITEMS = 12;

export function staggerDelay(index: number): number {
  return Math.min(index, STAGGER_MAX_ITEMS) * STAGGER_STEP;
}

/** Seviyeye göre süre ölçeği: azaltılmışta yarı süre, kapalıda sıfır. */
export function scaleDuration(seconds: number, level: MotionLevel): number {
  if (level === "off") return 0;
  return level === "reduced" ? seconds / 2 : seconds;
}

export type NavDirection = 1 | -1;

/**
 * Sayfa geçiş varyantları. İleri gidişte (1) içerik sağdan, geri dönüşte (-1) soldan gelir.
 * Azaltılmış seviyede yalnız opaklık kullanılır (kayma/ölçek yok).
 */
export function pageVariants(level: MotionLevel) {
  const move = level === "full";
  return {
    initial: (dir: NavDirection) => ({
      opacity: 0,
      x: move ? 24 * dir : 0,
      scale: move ? 0.985 : 1,
    }),
    animate: {
      opacity: 1,
      x: 0,
      scale: 1,
      transition: { duration: scaleDuration(duration.page, level), ease: ease.out },
    },
    exit: (dir: NavDirection) => ({
      opacity: 0,
      x: move ? -24 * dir : 0,
      scale: move ? 0.985 : 1,
      transition: { duration: scaleDuration(duration.base, level), ease: ease.inOut },
    }),
  };
}

/**
 * Ana sayfa vitrini zamanlaması (kullanıcı isteği: sinematik, yavaş ve üst üste binmeyen geçişler;
 * genel 150–450 ms kuralının bilinçli istisnası, bkz. MEMORY.md).
 */
export const showcase = {
  /** Her oyunun ekranda kaldığı süre; görsel bu süre boyunca merkeze doğru yakınlaşır. */
  dwellMs: 10_000,
  /** Görseller arası çapraz geçiş. */
  crossfadeS: 1.8,
  /** Geçiş sürerken yeni gezinme komutları yok sayılır. */
  navLockMs: 1_800,
  /** Yakınlaşma: %100 → %112. */
  zoomTo: 1.12,
} as const;
