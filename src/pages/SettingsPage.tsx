import { motion } from "motion/react";
import { PageHeader } from "../components/PageHeader";
import { Toggle } from "../components/Toggle";
import { spring } from "../design/motion";
import { useSettings, useUpdateSettings } from "../hooks/useSettings";
import { useMotionLevel, useUiStore, type MotionPreference } from "../stores/ui";

const MOTION_OPTIONS: { value: MotionPreference; label: string }[] = [
  { value: "system", label: "Sistem" },
  { value: "full", label: "Tam" },
  { value: "reduced", label: "Azaltılmış" },
  { value: "off", label: "Kapalı" },
];

export function SettingsPage() {
  const { data: settings } = useSettings();
  const update = useUpdateSettings();
  const preference = useUiStore((s) => s.motionPreference);
  const setPreference = useUiStore((s) => s.setMotionPreference);
  const level = useMotionLevel();

  return (
    <div className="page-content">
      <PageHeader
        eyebrow="Ayarlar"
        title="Uygulamayı kişiselleştir"
        subtitle="Değişiklikler anında uygulanır ve kaydedilir."
      />

      <section className="settings-group glass" aria-labelledby="settings-general">
        <h2 id="settings-general" className="text-h3">
          Genel
        </h2>
        <div className="setting-row">
          <div>
            <p className="setting-row__title">Windows açılışında başlat</p>
            <p className="setting-row__hint">
              Bilgisayar açılınca pencere açmadan arka planda, sistem tepsisinde başlar.
            </p>
          </div>
          <Toggle
            label="Windows açılışında başlat"
            checked={settings?.launchAtStartup ?? false}
            disabled={!settings}
            onChange={(v) => {
              update.mutate({ launchAtStartup: v });
            }}
          />
        </div>
        <div className="setting-row">
          <div>
            <p className="setting-row__title">Kapatınca sistem tepsisinde çalış</p>
            <p className="setting-row__hint">
              Pencere kapatılınca arayüz bellekten tamamen boşaltılır; uygulama tepside az RAM ile
              çalışmaya devam eder. Tamamen çıkmak için tepsi menüsünden “Çıkış”.
            </p>
          </div>
          <Toggle
            label="Kapatınca sistem tepsisinde çalış"
            checked={settings?.closeToTray ?? true}
            disabled={!settings}
            onChange={(v) => {
              update.mutate({ closeToTray: v });
            }}
          />
        </div>
      </section>

      <section className="settings-group glass" aria-labelledby="settings-appearance">
        <h2 id="settings-appearance" className="text-h3">
          Görünüm
        </h2>
        <div className="setting-row">
          <div>
            <p className="setting-row__title">Animasyon seviyesi</p>
            <p className="setting-row__hint">
              "Sistem", Windows'taki "Animasyonları azalt" ayarını izler.
            </p>
          </div>
          <div className="segmented" role="radiogroup" aria-label="Animasyon seviyesi">
            {MOTION_OPTIONS.map((opt) => {
              const selected = preference === opt.value;
              return (
                <button
                  key={opt.value}
                  type="button"
                  role="radio"
                  aria-checked={selected}
                  className={`segmented__item${selected ? " is-selected" : ""}`}
                  onClick={() => {
                    setPreference(opt.value);
                    update.mutate({ motion: opt.value });
                  }}
                >
                  {selected && (
                    <motion.span
                      layoutId="motion-pref"
                      className="segmented__thumb"
                      transition={level === "full" ? spring.snappy : { duration: 0 }}
                    />
                  )}
                  <span className="segmented__label">{opt.label}</span>
                </button>
              );
            })}
          </div>
        </div>
      </section>
    </div>
  );
}
