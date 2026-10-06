import { invoke } from "@tauri-apps/api/core";
import { motion } from "motion/react";
import { Icon } from "../components/Icon";
import { toast } from "../stores/toast";
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
  const systemReduced = useUiStore((s) => s.systemReducedMotion);

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
              "Sistem" Windows'un animasyon ayarını izler.{" "}
              {systemReduced
                ? 'Bu bilgisayarda Windows animasyonları kapalı; "Sistem" seçilirse uygulama da sadeleşir.'
                : "Windows animasyonları açık."}
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
      <section className="settings-group glass" aria-labelledby="settings-data">
        <h2 id="settings-data" className="text-h3">
          Veri
        </h2>
        <div className="setting-row">
          <div>
            <p className="setting-row__title">Günlük dosyaları</p>
            <p className="setting-row__hint">
              Hata bildirimi için son 7 günün günlükleri. Kişisel klasör yolları maskelenir.
            </p>
          </div>
          <button
            type="button"
            className="btn btn--glass"
            onClick={() => {
              invoke("open_log_dir").catch((e: unknown) => {
                toast.error("Klasör açılamadı", String(e));
              });
            }}
          >
            <Icon name="folder-open" size={18} />
            Günlük klasörünü aç
          </button>
        </div>
      </section>
    </div>
  );
}
