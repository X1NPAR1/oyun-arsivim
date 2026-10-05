import { motion } from "motion/react";
import { Link } from "react-router";
import { Icon } from "../components/Icon";
import { duration, ease, scaleDuration, staggerDelay } from "../design/motion";
import { useMotionLevel } from "../stores/ui";

const HIGHLIGHTS = [
  {
    icon: "game-controller",
    title: "Tüm platformlar",
    text: "Steam, Epic, GOG, EA, Ubisoft, Battle.net, Xbox",
  },
  { icon: "clock", title: "Süre takibi", text: "Her oturum otomatik kaydedilir" },
  { icon: "cloud-arrow-up", title: "Kayıt yedeği", text: "Save dosyaların güvende" },
] as const;

export function HomePage() {
  const level = useMotionLevel();
  const enter = (i: number) => ({
    initial: { opacity: 0, y: level === "full" ? 18 : 0 },
    animate: { opacity: 1, y: 0 },
    transition: {
      delay: level === "off" ? 0 : staggerDelay(i + 1),
      duration: scaleDuration(duration.page, level),
      ease: ease.out,
    },
  });

  return (
    <div className="page-content">
      <motion.section className="hero glass" {...enter(0)}>
        <div className="hero__glow" aria-hidden />
        <p className="text-caption hero__eyebrow">Hoş geldin</p>
        <h1 className="text-display hero__title">Oyun salonun hazırlanıyor</h1>
        <p className="hero__text">
          Bilgisayarındaki oyunlar bulunduğunda burada en son oynadığın oyun sinematik bir sahneyle
          karşılayacak.
        </p>
        <div className="hero__actions">
          <Link to="/kutuphane" className="btn btn--primary">
            <Icon name="squares-four" size={18} />
            Kütüphaneye git
          </Link>
          <Link to="/ayarlar" className="btn btn--glass">
            <Icon name="gear-six" size={18} />
            Ayarlar
          </Link>
        </div>
      </motion.section>

      <div className="highlight-row">
        {HIGHLIGHTS.map((h, i) => (
          <motion.article key={h.title} className="highlight glass" {...enter(i + 1)}>
            <span className="highlight__icon">
              <Icon name={h.icon} size={22} filled />
            </span>
            <div>
              <h3 className="text-h3">{h.title}</h3>
              <p className="highlight__text">{h.text}</p>
            </div>
          </motion.article>
        ))}
      </div>
    </div>
  );
}
