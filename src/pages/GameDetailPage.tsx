import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { AnimatePresence, motion } from "motion/react";
import { useEffect, useMemo, useRef, useState } from "react";
import { useNavigate, useParams } from "react-router";
import { BarChart, type BarSeries } from "../components/charts/BarChart";
import { CHART_THEME } from "../components/charts/theme";
import { EmptyState } from "../components/EmptyState";
import { Icon } from "../components/Icon";
import { Toggle } from "../components/Toggle";
import { ease, scaleDuration, spring } from "../design/motion";
import { useGameMenu } from "../hooks/useGameMenu";
import { useLaunchGame, useLibrary } from "../hooks/useLibrary";
import { useStats } from "../hooks/useStats";
import { artUrl, formatBytes, formatRelative, logoOf, PLATFORM_LABELS } from "../ipc/library";
import { perfApi, type PerfProfile } from "../ipc/perf";
import { formatDuration } from "../stats/aggregate";
import { toast } from "../stores/toast";
import { useMotionLevel } from "../stores/ui";

/** "Oyna" düğmesi: basınca dolan ilerleme, sonra durum metni. */
function PlayButton({
  onLaunch,
  disabled,
}: {
  onLaunch: () => Promise<unknown>;
  disabled: boolean;
}) {
  const [phase, setPhase] = useState<"idle" | "launching" | "done">("idle");
  const level = useMotionLevel();
  const start = () => {
    if (phase !== "idle") return;
    setPhase("launching");
    onLaunch()
      .then(() => {
        window.setTimeout(
          () => {
            setPhase("done");
            window.setTimeout(() => {
              setPhase("idle");
            }, 2500);
          },
          level === "off" ? 0 : 1400,
        );
      })
      .catch(() => {
        setPhase("idle");
      });
  };
  return (
    <button
      type="button"
      className={`btn btn--primary btn--lg play-btn is-${phase}`}
      disabled={disabled}
      onClick={start}
    >
      <span className="play-btn__fill" aria-hidden />
      <span className="play-btn__label">
        <Icon name={phase === "done" ? "check" : "play"} size={20} filled />
        {phase === "idle" ? "Oyna" : phase === "launching" ? "Başlatılıyor…" : "Başlatıldı"}
      </span>
    </button>
  );
}

function NotesCard({ gameId }: { gameId: number }) {
  const { data } = useQuery({
    queryKey: ["notes", gameId],
    queryFn: () => invoke<string>("get_game_notes", { id: gameId }),
  });
  const [text, setText] = useState<string | null>(null);
  const [saved, setSaved] = useState(true);
  const timer = useRef<number | undefined>(undefined);
  const value = text ?? data ?? "";
  const onChange = (v: string) => {
    setText(v);
    setSaved(false);
    window.clearTimeout(timer.current);
    // Yazmayı bırakınca otomatik kaydet.
    timer.current = window.setTimeout(() => {
      invoke("set_game_notes", { id: gameId, notes: v })
        .then(() => {
          setSaved(true);
        })
        .catch((e: unknown) => {
          toast.error("Not kaydedilemedi", String(e));
        });
    }, 600);
  };
  return (
    <section className="detail-card glass">
      <div className="detail-card__head">
        <h2 className="text-h3">Notlar</h2>
        <span className="detail-card__hint">{saved ? "Kaydedildi" : "Kaydediliyor…"}</span>
      </div>
      <textarea
        className="notes"
        value={value}
        placeholder="Bu oyunla ilgili notların: görevler, ayarlar, kodlar…"
        aria-label="Oyun notları"
        onChange={(e) => {
          onChange(e.target.value);
        }}
      />
    </section>
  );
}

function ProfileQuick({ gameId }: { gameId: number }) {
  const client = useQueryClient();
  const navigate = useNavigate();
  const { data: profile } = useQuery({
    queryKey: ["perf", "profile", gameId],
    queryFn: () => perfApi.profile(gameId),
  });
  const save = useMutation({
    mutationFn: async (enabled: boolean) => {
      const base: PerfProfile =
        enabled && profile && !Object.values({ ...profile, enabled: false }).some(Boolean)
          ? await perfApi.recommendedProfile()
          : { ...(profile ?? (await perfApi.recommendedProfile())), enabled };
      return perfApi.setProfile(gameId, { ...base, enabled });
    },
    onSuccess: (p) => {
      client.setQueryData(["perf", "profile", gameId], p);
      toast.success(p.enabled ? "Performans modu açık" : "Performans modu kapalı");
    },
  });
  return (
    <div className="perf-quick glass">
      <Icon name="gauge" size={18} />
      <span>Performans modu</span>
      <Toggle
        label="Performans modu"
        checked={profile?.enabled ?? false}
        disabled={!profile}
        onChange={(v) => {
          save.mutate(v);
        }}
      />
      <button
        type="button"
        className="perf-quick__more"
        aria-label="Performans ayrıntıları"
        onClick={() => void navigate(`/performans?oyun=${String(gameId)}`)}
      >
        <Icon name="caret-right" size={14} />
      </button>
    </div>
  );
}

export function GameDetailPage() {
  const { id } = useParams();
  const gameId = Number(id);
  const navigate = useNavigate();
  const level = useMotionLevel();
  const { data: games = [], isLoading } = useLibrary();
  const game = games.find((g) => g.id === gameId);
  const launch = useLaunchGame();
  const menu = useGameMenu();
  const { result, sessions } = useStats(30, Number.isFinite(gameId) ? gameId : null);
  const [heroReady, setHeroReady] = useState(false);

  const cover = game ? artUrl(game.art.cover) : null;
  const hero = game ? (artUrl(game.art.backdrop) ?? artUrl(game.art.hero)) : null;
  const logo = game ? logoOf(game) : null;
  const darkLogo = logo?.kind === "logo" && game?.art.logoDark === true;
  const accent = game?.art.accent ?? null;

  // Dinamik vurgu: oyunun görselinden baskın renk; sayfadan çıkınca varsayılana döner.
  useEffect(() => {
    if (!accent) return;
    document.documentElement.style.setProperty("--c-accent", accent);
    return () => {
      document.documentElement.style.removeProperty("--c-accent");
    };
  }, [accent]);

  // Esc ile geri.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !document.querySelector(".context-menu")) void navigate(-1);
    };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
    };
  }, [navigate]);

  const dailyLabels = useMemo(
    () =>
      result.daily.map((d) =>
        d.date.toLocaleDateString("tr-TR", { day: "numeric", month: "short" }),
      ),
    [result.daily],
  );
  const dailySeries = useMemo<BarSeries[]>(
    () => [
      { label: "Aktif", data: result.daily.map((d) => d.active), color: CHART_THEME.active },
      { label: "AFK", data: result.daily.map((d) => d.idle), color: CHART_THEME.idle },
    ],
    [result.daily],
  );

  if (!game) {
    return (
      <div className="page-content">
        {!isLoading && (
          <EmptyState
            icon="game-controller"
            title="Oyun bulunamadı"
            description="Bu oyun kütüphanende yok veya kaldırılmış."
          />
        )}
      </div>
    );
  }

  const pt = game.playtime;
  const last = formatRelative(game.lastPlayed);
  const fade = { duration: scaleDuration(0.5, level), ease: ease.out };

  return (
    <div className="detail">
      {/* Paylaşılan öğe: karttaki kapak buraya büyüyerek gelir, sonra geniş görsele geçer. */}
      <motion.div
        layoutId={`art-${String(game.id)}`}
        className="detail__art"
        transition={level === "full" ? spring.gentle : { duration: 0 }}
      >
        {cover && <img className="detail__cover" src={cover} alt="" draggable={false} />}
        {hero && (
          <motion.img
            className="detail__hero"
            src={hero}
            alt=""
            draggable={false}
            initial={{ opacity: 0 }}
            animate={{ opacity: heroReady ? 1 : 0 }}
            transition={fade}
            onLoad={() => {
              setHeroReady(true);
            }}
          />
        )}
      </motion.div>
      <div className="detail__scrim" />

      <div className="detail__content">
        <button
          type="button"
          className="detail__back icon-btn"
          aria-label="Geri"
          onClick={() => void navigate(-1)}
        >
          <Icon name="arrow-left" size={18} />
        </button>

        <motion.header
          className="detail__head"
          initial={{ opacity: 0, y: level === "full" ? 16 : 0 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ ...fade, delay: level === "full" ? 0.15 : 0 }}
        >
          {logo?.kind === "logo" ? (
            <img
              className={`detail__logo${darkLogo ? " is-dark-logo" : ""}`}
              src={logo.src}
              alt={game.title}
            />
          ) : (
            <h1 className="detail__title">
              {logo && <img className="detail__title-icon" src={logo.src} alt="" />}
              {game.title}
            </h1>
          )}
          <div className="detail__meta">
            <span className="showcase__chip">{PLATFORM_LABELS[game.platform]}</span>
            {!game.installed && <span className="badge badge--removed-inline">Silindi</span>}
            {pt.totalSeconds > 0 && (
              <span className="showcase__stat">
                <Icon name="clock" size={15} />
                {formatDuration(pt.totalSeconds)} toplam
              </span>
            )}
            {pt.weekSeconds > 0 && (
              <span className="showcase__stat">Bu hafta {formatDuration(pt.weekSeconds)}</span>
            )}
            {last && <span className="showcase__stat">Son oynama {last}</span>}
          </div>
          <div className="detail__actions">
            <PlayButton disabled={!game.installed} onLaunch={() => launch.mutateAsync(game.id)} />
            <ProfileQuick gameId={game.id} />
            <button
              type="button"
              className="icon-btn detail__more"
              aria-label="Diğer işlemler"
              onClick={(e) => {
                menu.openFor(game, e);
              }}
            >
              <Icon name="list" size={18} />
            </button>
          </div>
        </motion.header>

        <AnimatePresence>
          <motion.div
            className="detail__panels"
            initial={{ opacity: 0, y: level === "full" ? 24 : 0 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ ...fade, delay: level === "full" ? 0.25 : 0 }}
          >
            <section className="detail-card glass detail-card--wide">
              <div className="detail-card__head">
                <h2 className="text-h3">Son 30 gün</h2>
                <span className="detail-card__hint">
                  {formatDuration(result.totals.total)} · AFK %
                  {String(Math.round(result.totals.afkRatio * 100))} ·{" "}
                  {String(result.sessions.count)} oturum
                </span>
              </div>
              {result.totals.total > 0 ? (
                <BarChart
                  labels={dailyLabels}
                  series={dailySeries}
                  format={formatDuration}
                  stacked
                  height={200}
                  ariaLabel="Son 30 günün günlük oynama süresi"
                />
              ) : (
                <p className="detail-card__empty">
                  Oyun Arşivim bu oyunu henüz oynarken görmedi.
                  {pt.platformTwoWeeksMinutes
                    ? ` Steam'e göre son 2 haftada ${formatDuration(pt.platformTwoWeeksMinutes * 60)} oynadın.`
                    : ""}
                </p>
              )}
            </section>

            <section className="detail-card glass">
              <h2 className="text-h3">Son oturumlar</h2>
              {sessions.length === 0 ? (
                <p className="detail-card__empty">Henüz kayıtlı oturum yok.</p>
              ) : (
                <ul className="session-list">
                  {sessions.slice(0, 6).map((s) => (
                    <li key={s.id}>
                      <span>
                        {new Date(s.startedAt * 1000).toLocaleString("tr-TR", {
                          day: "numeric",
                          month: "short",
                          hour: "2-digit",
                          minute: "2-digit",
                        })}
                      </span>
                      <strong>{formatDuration(s.activeSeconds + s.idleSeconds)}</strong>
                    </li>
                  ))}
                </ul>
              )}
            </section>

            <section className="detail-card glass">
              <h2 className="text-h3">Bilgiler</h2>
              <dl className="facts">
                <dt>Platform</dt>
                <dd>{PLATFORM_LABELS[game.platform]}</dd>
                {formatBytes(game.sizeBytes) && (
                  <>
                    <dt>Boyut</dt>
                    <dd>{formatBytes(game.sizeBytes)}</dd>
                  </>
                )}
                <dt>Kütüphaneye eklendi</dt>
                <dd>
                  {new Date(game.addedAt * 1000).toLocaleDateString("tr-TR", {
                    day: "numeric",
                    month: "long",
                    year: "numeric",
                  })}
                </dd>
                {game.installDir && (
                  <>
                    <dt>Konum</dt>
                    <dd className="facts__path" title={game.installDir}>
                      {game.installDir}
                    </dd>
                  </>
                )}
              </dl>
            </section>

            <NotesCard gameId={game.id} />
          </motion.div>
        </AnimatePresence>
      </div>
      {menu.menu}
    </div>
  );
}
