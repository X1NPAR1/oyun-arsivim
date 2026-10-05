import { motion } from "motion/react";
import { EmptyState } from "../components/EmptyState";
import { PageHeader } from "../components/PageHeader";
import { duration, ease, scaleDuration, staggerDelay } from "../design/motion";
import { useMotionLevel } from "../stores/ui";

const PLACEHOLDER_COUNT = 12;

/** Faz 2: yerleşim ve kart giriş animasyonu iskeleti. Gerçek oyunlar Faz 5'ten itibaren gelir. */
export function LibraryPage() {
  const level = useMotionLevel();
  return (
    <div className="page-content">
      <PageHeader
        eyebrow="Kütüphane"
        title="Oyunların"
        subtitle="Tüm platformlardaki oyunların tek bir yerde."
      />
      <EmptyState
        icon="magnifying-glass"
        title="Henüz oyun taranmadı"
        description="Oyun tarama yakında geliyor. Bulunan oyunlar kapak görselleriyle burada sıralanacak."
      />
      <div className="card-grid" aria-hidden>
        {Array.from({ length: PLACEHOLDER_COUNT }, (_, i) => (
          <motion.div
            key={i}
            className="card-skeleton"
            initial={{ opacity: 0, y: level === "full" ? 28 : 0 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{
              delay: level === "off" ? 0 : staggerDelay(i),
              duration: scaleDuration(duration.slow, level),
              ease: ease.out,
            }}
          />
        ))}
      </div>
    </div>
  );
}
