import { AnimatePresence, motion } from "motion/react";
import { ease, showcase } from "../../design/motion";
import { useStageStore } from "../../stores/stage";
import { useMotionLevel } from "../../stores/ui";

/**
 * Tam pencere sahne. Her görsel %100 ölçekte (tam ekran) belirir ve ekranda kaldığı süre boyunca
 * merkeze doğru doğrusal olarak yakınlaşır. Geçişte yeni görsel çapraz geçişle gelirken eskisi
 * yakınlaşmaya devam ederek solar — hareket hiç durmaz.
 */
export function Stage() {
  const image = useStageStore((s) => s.image);
  const level = useMotionLevel();
  const zoom = level === "full";
  const fade =
    level === "off" ? 0 : level === "reduced" ? showcase.crossfadeS / 2 : showcase.crossfadeS;
  // Yakınlaşma, görüntülenme süresi + çıkış geçişi boyunca sürer (solarken de durmasın).
  const zoomSeconds = showcase.dwellMs / 1000 + showcase.crossfadeS;

  return (
    <div className={`stage${image ? " is-on" : ""}`} aria-hidden>
      <AnimatePresence initial={false}>
        {image && (
          <motion.div
            key={image.src}
            className="stage__layer"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: fade, ease: ease.inOut }}
          >
            <motion.img
              className={`stage__image${image.ambient ? " is-ambient" : ""}`}
              src={image.src}
              alt=""
              draggable={false}
              decoding="async"
              initial={{ scale: 1 }}
              animate={{ scale: zoom && !image.ambient ? showcase.zoomTo : 1 }}
              transition={{ duration: zoomSeconds, ease: "linear" }}
            />
          </motion.div>
        )}
      </AnimatePresence>
      <div className="stage__scrim" />
    </div>
  );
}
