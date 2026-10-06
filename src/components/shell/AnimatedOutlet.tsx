import { AnimatePresence, LayoutGroup, motion } from "motion/react";
import { useState, type ReactNode } from "react";
import { useLocation, useOutlet } from "react-router";
import { pageVariants, type NavDirection } from "../../design/motion";
import { navDirection } from "../../navigation/routes";
import { useMotionLevel } from "../../stores/ui";

/** Çıkış animasyonu sırasında eski sayfanın içeriğini dondurur (yoksa yeni sayfa iki kez görünür). */
function Frozen({ children }: { children: ReactNode }) {
  const [frozen] = useState(children);
  return frozen;
}

/**
 * Sayfa değişimlerini kayma + solma + hafif ölçekle canlandırır. Eski ve yeni sayfa aynı ızgara
 * hücresinde üst üste biner; böylece yeni sayfa eskisinin çıkışını beklemez.
 */
export function AnimatedOutlet() {
  const location = useLocation();
  const outlet = useOutlet();
  const level = useMotionLevel();
  // Önceki yoldan türetilen yön (React'in "önceki değeri state'te tut" deseni; effect gerektirmez).
  const [previousPath, setPreviousPath] = useState(location.pathname);
  const [direction, setDirection] = useState<NavDirection>(1);
  if (previousPath !== location.pathname) {
    setDirection(navDirection(previousPath, location.pathname));
    setPreviousPath(location.pathname);
  }

  const variants = pageVariants(level);

  return (
    // LayoutGroup: kart ↔ detay paylaşılan öğe geçişi sayfalar arasında çalışsın.
    <div className="page-stack">
      <LayoutGroup>
        <AnimatePresence initial={false} custom={direction}>
          <motion.main
            key={location.pathname}
            className="page"
            custom={direction}
            variants={variants}
            initial="initial"
            animate="animate"
            exit="exit"
          >
            <Frozen>{outlet}</Frozen>
          </motion.main>
        </AnimatePresence>
      </LayoutGroup>
    </div>
  );
}
