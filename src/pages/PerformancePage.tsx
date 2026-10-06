import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import { useSearchParams } from "react-router";
import { Gauge } from "../components/charts/Gauge";
import { GamePicker } from "../components/GamePicker";
import { Icon } from "../components/Icon";
import { PageHeader } from "../components/PageHeader";
import { Toggle } from "../components/Toggle";
import { useLibrary } from "../hooks/useLibrary";
import { useSettings, useUpdateSettings } from "../hooks/useSettings";
import { formatGB, formatMB, perfApi, type PerfProfile, type TweakState } from "../ipc/perf";
import { toast } from "../stores/toast";

function errorText(e: unknown): string {
  return typeof e === "string" ? e : e instanceof Error ? e.message : "Bilinmeyen hata";
}

// ───────────── Canlı izleme ─────────────

function LiveMonitor() {
  const { data: s } = useQuery({
    queryKey: ["perf", "snapshot"],
    queryFn: perfApi.snapshot,
    refetchInterval: 1000,
    // Sayfa açıkken ölçülür; pencere arka plandayken sorgu durur (gereksiz yük yok).
    refetchIntervalInBackground: false,
    staleTime: 0,
  });
  const memPercent = s ? (s.memoryUsed / s.memoryTotal) * 100 : null;
  return (
    <>
      <div className="gauge-row">
        <Gauge
          label="İşlemci"
          percent={s?.cpuPercent ?? null}
          detail={s ? `${String(s.cpuCores.length)} iş parçacığı` : "Ölçülüyor…"}
        />
        <Gauge
          label="Bellek (RAM)"
          percent={memPercent}
          detail={s ? `${formatGB(s.memoryUsed)} / ${formatGB(s.memoryTotal)}` : "Ölçülüyor…"}
        />
        <Gauge
          label="Ekran kartı"
          percent={s?.gpuPercent ?? null}
          detail={s?.gpuMemoryUsed != null ? `VRAM ${formatMB(s.gpuMemoryUsed)}` : "Sayaç yok"}
        />
      </div>
      {s && (
        <div className="perf-split">
          <section className="perf-card glass">
            <h2 className="text-h3">Çekirdekler</h2>
            <p className="perf-card__hint">{s.cpuName}</p>
            <div className="cores" aria-label="Çekirdek başına işlemci kullanımı">
              {s.cpuCores.map((c, i) => (
                <div
                  key={i}
                  className="core"
                  title={`Çekirdek ${String(i + 1)}: %${String(Math.round(c))}`}
                >
                  <div className="core__bar" style={{ height: `${String(Math.max(3, c))}%` }} />
                </div>
              ))}
            </div>
          </section>
          <section className="perf-card glass">
            <h2 className="text-h3">En çok bellek kullananlar</h2>
            <table className="data-table">
              <thead>
                <tr>
                  <th>Süreç</th>
                  <th className="num">Bellek</th>
                  <th className="num">İşlemci</th>
                </tr>
              </thead>
              <tbody>
                {s.topProcesses.slice(0, 6).map((p) => (
                  <tr key={p.pid}>
                    <td>{p.name}</td>
                    <td className="num">{formatMB(p.memoryBytes)}</td>
                    <td className="num">
                      %{p.cpuPercent.toLocaleString("tr-TR", { maximumFractionDigits: 1 })}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </section>
        </div>
      )}
    </>
  );
}

// ───────────── Geçici dosyalar ─────────────

function TempCard() {
  const qc = useQueryClient();
  const usage = useQuery({ queryKey: ["perf", "temp"], queryFn: perfApi.tempUsage });
  const clean = useMutation({
    mutationFn: perfApi.cleanTemp,
    onSuccess: (r) => {
      toast.success(
        "Geçici dosyalar temizlendi",
        `${r.deletedFiles.toLocaleString("tr-TR")} dosya, ${formatMB(r.freedBytes)} boşaltıldı` +
          (r.skippedFiles > 0
            ? ` · kullanımdaki ${r.skippedFiles.toLocaleString("tr-TR")} dosya atlandı`
            : ""),
      );
      void qc.invalidateQueries({ queryKey: ["perf", "temp"] });
    },
    onError: (e) => {
      toast.error("Temizlenemedi", errorText(e));
    },
  });
  const size = usage.data
    ? `${formatMB(usage.data.bytes)} · ${usage.data.files.toLocaleString("tr-TR")} dosya`
    : usage.isLoading
      ? "Hesaplanıyor…"
      : "—";
  return (
    <section className="perf-card glass">
      <h2 className="text-h3">Geçici dosyalar</h2>
      <p className="temp-size" aria-live="polite">
        <span className="temp-size__label">Temizlenebilir</span>
        <span className="temp-size__value">{size}</span>
      </p>
      <div className="perf-actions">
        <button
          type="button"
          className="btn btn--primary"
          disabled={clean.isPending}
          onClick={() => {
            clean.mutate();
          }}
        >
          <Icon name="trash" size={18} />
          {clean.isPending ? "Temizleniyor…" : "Geçici dosyaları temizle"}
        </button>
      </div>
      <p className="perf-card__hint">
        Kullanıcı geçici klasörü (%TEMP%) ve WindowsTemp temizlenir. Son bir saatte değişen ve
        kullanımdaki dosyalara dokunulmaz.
      </p>
    </section>
  );
}

// ───────────── RAM ─────────────

function RamCard() {
  const { data: settings } = useSettings();
  const update = useUpdateSettings();
  const clean = useMutation({
    mutationFn: perfApi.cleanRam,
    onSuccess: (r) => {
      toast.success(
        "Bellek temizlendi",
        `${String(r.processes)} süreçten ${formatMB(r.freedBytes)} boşaltıldı · kullanılabilir ${formatGB(r.availableAfter)}`,
      );
    },
    onError: (e) => {
      toast.error("Temizlenemedi", errorText(e));
    },
  });
  const purge = useMutation({
    mutationFn: perfApi.purgeStandby,
    onSuccess: () => {
      toast.success("Bekleme belleği boşaltıldı");
    },
    onError: (e) => {
      toast.error("Bekleme belleği temizlenemedi", errorText(e));
    },
  });
  const threshold = settings?.autoRamThreshold ?? 85;
  return (
    <section className="perf-card glass">
      <h2 className="text-h3">Bellek temizliği</h2>
      <div className="perf-actions">
        <button
          type="button"
          className="btn btn--primary"
          disabled={clean.isPending}
          onClick={() => {
            clean.mutate();
          }}
        >
          <Icon name="broom" size={18} />
          {clean.isPending ? "Temizleniyor…" : "RAM'i şimdi temizle"}
        </button>
        <button
          type="button"
          className="btn btn--glass"
          disabled={purge.isPending}
          onClick={() => {
            purge.mutate();
          }}
        >
          <Icon name="memory" size={18} />
          Bekleme belleğini boşalt
          <span className="badge badge--admin">Yönetici</span>
        </button>
      </div>
      <p className="perf-card__hint">
        Süreçlerin kullanılmayan bellek sayfaları serbest bırakılır; veri kaybı olmaz.
      </p>
      <div className="setting-row">
        <div>
          <p className="setting-row__title">Otomatik RAM temizliği</p>
          <p className="setting-row__hint">
            Bellek kullanımı %{threshold} üzerine çıkınca (en fazla 5 dakikada bir).
          </p>
        </div>
        <Toggle
          label="Otomatik RAM temizliği"
          checked={settings?.autoRamClean ?? false}
          disabled={!settings}
          onChange={(v) => {
            update.mutate({ autoRamClean: v });
          }}
        />
      </div>
      <label className="slider">
        <span>Eşik: %{threshold}</span>
        <input
          type="range"
          min={60}
          max={95}
          step={5}
          value={threshold}
          disabled={!settings}
          onChange={(e) => {
            update.mutate({ autoRamThreshold: Number(e.target.value) });
          }}
        />
      </label>
    </section>
  );
}

// ───────────── Güç planı ─────────────

function PowerCard() {
  const client = useQueryClient();
  const { data } = useQuery({ queryKey: ["perf", "power"], queryFn: perfApi.power });
  const set = useMutation({
    mutationFn: perfApi.setPower,
    onSuccess: (o) => {
      client.setQueryData(["perf", "power"], o);
      toast.success("Güç planı değişti", o.active?.name);
    },
    onError: (e) => {
      toast.error("Güç planı değiştirilemedi", errorText(e));
    },
  });
  return (
    <section className="perf-card glass">
      <h2 className="text-h3">Güç planı</h2>
      <div className="chip-group">
        {data?.schemes.map((s) => (
          <button
            key={s.guid}
            type="button"
            className={`chip${s.active ? " is-selected" : ""}`}
            aria-pressed={s.active}
            onClick={() => {
              if (!s.active) set.mutate(s.guid);
            }}
          >
            {s.name}
            {data.recommended?.guid === s.guid && <span className="chip__count">önerilen</span>}
          </button>
        ))}
      </div>
      <p className="perf-card__hint">
        Oyun profilinde "Güç planı" açıksa oyun başlayınca önerilen plana geçilir, oyun bitince
        önceki plana dönülür. Yeni plan oluşturulmaz.
        {data &&
          data.duplicateCount > 0 &&
          ` Bilgisayarında başka araçların bıraktığı ${data.duplicateCount.toLocaleString("tr-TR")} kopya plan var; listede tek gösteriliyor.`}
      </p>
    </section>
  );
}

// ───────────── Windows ayarları ─────────────

function TweaksCard() {
  const client = useQueryClient();
  const { data: tweaks = [] } = useQuery({ queryKey: ["perf", "tweaks"], queryFn: perfApi.tweaks });
  const set = useMutation({
    mutationFn: ({ id, enabled }: { id: string; enabled: boolean }) =>
      perfApi.setTweak(id, enabled),
    onSuccess: (t) => {
      client.setQueryData<TweakState[]>(["perf", "tweaks"], (old) =>
        old?.map((x) => (x.id === t.id ? t : x)),
      );
      toast.success(
        t.title,
        `${t.enabled ? "Açıldı" : "Kapatıldı"}${t.requiresRestart ? " · yeniden başlatınca etkin olur" : ""}`,
      );
    },
    onError: (e) => {
      toast.error("Ayar değiştirilemedi", errorText(e));
    },
  });
  const restore = useMutation({
    mutationFn: perfApi.restoreDefaults,
    onSuccess: (r) => {
      void client.invalidateQueries({ queryKey: ["perf", "tweaks"] });
      if (r.failed.length) toast.error("Bazı ayarlar geri alınamadı", r.failed.join(", "));
      else
        toast.success(
          "Ayarlar ilk hâline döndü",
          r.restored.length ? r.restored.join(", ") : "Değişiklik yoktu",
        );
    },
    onError: (e) => {
      toast.error("Geri alınamadı", errorText(e));
    },
  });
  const applyRecommended = async () => {
    for (const t of tweaks) {
      if (t.enabled !== null && t.enabled !== t.recommended && !t.requiresAdmin) {
        await set.mutateAsync({ id: t.id, enabled: t.recommended }).catch(() => undefined);
      }
    }
  };
  const groups = useMemo(() => {
    const map = new Map<string, TweakState[]>();
    for (const t of tweaks) map.set(t.category, [...(map.get(t.category) ?? []), t]);
    return [...map.entries()];
  }, [tweaks]);

  return (
    <section className="perf-card glass perf-card--wide">
      <div className="perf-card__head">
        <h2 className="text-h3">Windows oyun ayarları</h2>
        <div className="perf-actions">
          <button type="button" className="btn btn--glass" onClick={() => void applyRecommended()}>
            <Icon name="rocket-launch" size={18} />
            Önerilenleri uygula
          </button>
          <button
            type="button"
            className="btn btn--glass"
            disabled={restore.isPending}
            onClick={() => {
              restore.mutate();
            }}
          >
            <Icon name="arrows-clockwise" size={18} />
            Varsayılanlara dön
          </button>
        </div>
      </div>
      {groups.map(([category, items]) => (
        <div key={category} className="tweak-group">
          <p className="tweak-group__title">{category}</p>
          {items.map((t) => (
            <div key={t.id} className="setting-row tweak">
              <div>
                <p className="setting-row__title">
                  {t.title}
                  <span
                    className={`badge ${t.enabled === t.recommended ? "badge--ok" : "badge--hint"}`}
                  >
                    Önerilen: {t.recommended ? "Açık" : "Kapalı"}
                  </span>
                  {t.requiresAdmin && <span className="badge badge--admin">Yönetici</span>}
                  {t.requiresRestart && <span className="badge badge--hint">Yeniden başlatma</span>}
                </p>
                <p className="setting-row__hint">{t.description}</p>
              </div>
              <Toggle
                label={t.title}
                checked={t.enabled ?? false}
                disabled={t.enabled === null || set.isPending}
                onChange={(v) => {
                  set.mutate({ id: t.id, enabled: v });
                }}
              />
            </div>
          ))}
        </div>
      ))}
    </section>
  );
}

// ───────────── Oyuna özel profil ─────────────

const PROFILE_OPTIONS: {
  key: Exclude<keyof PerfProfile, "enabled">;
  title: string;
  hint: string;
}[] = [
  {
    key: "highPriority",
    title: "Yüksek işlemci önceliği",
    hint: "Oyun süreçleri Windows'ta önce çalıştırılır.",
  },
  {
    key: "powerPlan",
    title: "Performans güç planı",
    hint: "Oyun açıkken en yüksek güç planı, kapanınca eski plan.",
  },
  {
    key: "cleanRam",
    title: "Açılışta RAM temizliği",
    hint: "Oyun başlarken arka plandaki belleği boşaltır.",
  },
  {
    key: "muteNotifications",
    title: "Bildirimleri sustur",
    hint: "Oyun boyunca Windows bildirimleri gösterilmez.",
  },
  {
    key: "closeApps",
    title: "Seçili uygulamaları kapat",
    hint: "Aşağıdaki listedeki uygulamalar oyun başlarken kapatılır.",
  },
];

function ProfileCard({ initialGame }: { initialGame: number | null }) {
  const { data: games = [] } = useLibrary();
  const installed = useMemo(() => games.filter((g) => g.installed), [games]);
  const [gameId, setGameId] = useState<number | null>(initialGame ?? installed[0]?.id ?? null);
  const client = useQueryClient();
  const { data: profile } = useQuery({
    queryKey: ["perf", "profile", gameId],
    queryFn: () => perfApi.profile(gameId ?? 0),
    enabled: gameId !== null,
  });
  const save = useMutation({
    mutationFn: (p: PerfProfile) => perfApi.setProfile(gameId ?? 0, p),
    onSuccess: (p) => {
      client.setQueryData(["perf", "profile", gameId], p);
      void client.invalidateQueries({ queryKey: ["perf", "profiled"] });
    },
    onError: (e) => {
      toast.error("Profil kaydedilemedi", errorText(e));
    },
  });
  const game = installed.find((g) => g.id === gameId);

  const enableRecommended = async () => {
    const rec = await perfApi.recommendedProfile();
    save.mutate(rec);
    toast.success(
      "Performans modu açıldı",
      game ? `${game.title} açılınca uygulanacak.` : undefined,
    );
  };

  return (
    <section className="perf-card glass">
      <h2 className="text-h3">Oyuna özel performans modu</h2>
      <GamePicker games={installed} value={gameId} onChange={setGameId} />
      {gameId === null || !profile ? (
        <p className="perf-card__hint">Bir oyun seç.</p>
      ) : (
        <>
          <div className="setting-row">
            <div>
              <p className="setting-row__title">Performans modu</p>
              <p className="setting-row__hint">
                Oyun açılınca otomatik uygulanır, kapanınca her şey geri alınır.
              </p>
            </div>
            <Toggle
              label="Performans modu"
              checked={profile.enabled}
              onChange={(v) => {
                if (v && !Object.values({ ...profile, enabled: false }).some(Boolean))
                  void enableRecommended();
                else save.mutate({ ...profile, enabled: v });
              }}
            />
          </div>
          {PROFILE_OPTIONS.map((o) => (
            <div
              key={o.key}
              className={`setting-row profile-opt${profile.enabled ? "" : " is-disabled"}`}
            >
              <div>
                <p className="setting-row__title">{o.title}</p>
                <p className="setting-row__hint">{o.hint}</p>
              </div>
              <Toggle
                label={o.title}
                checked={profile[o.key]}
                disabled={!profile.enabled}
                onChange={(v) => {
                  save.mutate({ ...profile, [o.key]: v });
                }}
              />
            </div>
          ))}
        </>
      )}
    </section>
  );
}

function CloseListCard() {
  const { data: settings } = useSettings();
  const update = useUpdateSettings();
  const { data: suggestions = [] } = useQuery({
    queryKey: ["perf", "suggestions"],
    queryFn: perfApi.closeSuggestions,
  });
  const [custom, setCustom] = useState("");
  const list = settings?.closeList ?? [];
  const toggle = (exe: string) => {
    const has = list.some((x) => x.toLowerCase() === exe.toLowerCase());
    update.mutate({
      closeList: has ? list.filter((x) => x.toLowerCase() !== exe.toLowerCase()) : [...list, exe],
    });
  };
  const addCustom = () => {
    let exe = custom.trim();
    if (!exe) return;
    if (!exe.toLowerCase().endsWith(".exe")) exe += ".exe";
    if (!list.some((x) => x.toLowerCase() === exe.toLowerCase()))
      update.mutate({ closeList: [...list, exe] });
    setCustom("");
  };
  const extra = list.filter(
    (x) => !suggestions.some((s) => s.exe.toLowerCase() === x.toLowerCase()),
  );
  return (
    <section className="perf-card glass">
      <h2 className="text-h3">Oyun sırasında kapatılacaklar</h2>
      <p className="perf-card__hint">
        Profilde "Seçili uygulamaları kapat" açık olan oyunlarda bu uygulamalar nazikçe kapatılır.
        Sistem ve güvenlik süreçleri asla kapatılmaz.
      </p>
      <div className="chip-group">
        {suggestions.map((s) => {
          const on = list.some((x) => x.toLowerCase() === s.exe.toLowerCase());
          return (
            <button
              key={s.exe}
              type="button"
              className={`chip${on ? " is-selected" : ""}`}
              aria-pressed={on}
              onClick={() => {
                toggle(s.exe);
              }}
            >
              {s.title}
              {s.running && <span className="chip__count">açık</span>}
            </button>
          );
        })}
        {extra.map((x) => (
          <button
            key={x}
            type="button"
            className="chip is-selected"
            aria-pressed
            onClick={() => {
              toggle(x);
            }}
          >
            {x} <Icon name="x" size={12} />
          </button>
        ))}
      </div>
      <div className="inline-add">
        <label className="search">
          <Icon name="plus" size={16} />
          <input
            value={custom}
            placeholder="Başka uygulama (ör. Discord.exe)"
            aria-label="Kapatılacak uygulama adı"
            onChange={(e) => {
              setCustom(e.target.value);
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter") addCustom();
            }}
          />
        </label>
        <button type="button" className="btn btn--glass" onClick={addCustom}>
          Ekle
        </button>
      </div>
      <div className="setting-row">
        <div>
          <p className="setting-row__title">Oyun bitince yeniden aç</p>
          <p className="setting-row__hint">
            Kapatılan uygulamalar oyun kapanınca tekrar başlatılır.
          </p>
        </div>
        <Toggle
          label="Oyun bitince yeniden aç"
          checked={settings?.restartClosedApps ?? true}
          disabled={!settings}
          onChange={(v) => {
            update.mutate({ restartClosedApps: v });
          }}
        />
      </div>
    </section>
  );
}

export function PerformancePage() {
  const [params] = useSearchParams();
  const initialGame = params.get("oyun") ? Number(params.get("oyun")) : null;
  return (
    <div className="page-content">
      <PageHeader
        eyebrow="Performans"
        title="Performans merkezi"
        subtitle="Canlı sistem durumu, oyun ayarları ve oyuna özel performans modu."
      />
      <LiveMonitor />
      <div className="perf-split">
        <RamCard />
        <PowerCard />
      </div>
      <div className="perf-split">
        <ProfileCard initialGame={initialGame} />
        <CloseListCard />
      </div>
      <TempCard />
      <TweaksCard />
    </div>
  );
}
