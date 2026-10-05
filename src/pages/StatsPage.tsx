import { useMemo, useState } from "react";
import { BarChart, type BarSeries } from "../components/charts/BarChart";
import { Heatmap } from "../components/charts/Heatmap";
import { CHART_THEME } from "../components/charts/theme";
import { EmptyState } from "../components/EmptyState";
import { GamePicker } from "../components/GamePicker";
import { PageHeader } from "../components/PageHeader";
import { useLibrary } from "../hooks/useLibrary";
import { useStats, type RangeDays } from "../hooks/useStats";
import { PLATFORM_LABELS } from "../ipc/library";
import { formatDuration, formatHourRange, WEEKDAYS, WEEKDAYS_SHORT } from "../stats/aggregate";

const RANGES: { value: RangeDays; label: string }[] = [
  { value: 7, label: "7 gün" },
  { value: 30, label: "30 gün" },
  { value: 90, label: "90 gün" },
  { value: 365, label: "1 yıl" },
];

const HOUR_LABELS = Array.from({ length: 24 }, (_, h) => String(h).padStart(2, "0"));
const hoursFmt = (s: number) => formatDuration(s);

function Kpi({ label, value, hint }: { label: string; value: string; hint?: string }) {
  return (
    <div className="kpi glass">
      <span className="kpi__label">{label}</span>
      <span className="kpi__value">{value}</span>
      {hint && <span className="kpi__hint">{hint}</span>}
    </div>
  );
}

function ChartCard({
  title,
  children,
  wide,
}: {
  title: string;
  children: React.ReactNode;
  wide?: boolean;
}) {
  return (
    <section className={`chart-card glass${wide ? " chart-card--wide" : ""}`}>
      <h2 className="text-h3">{title}</h2>
      {children}
    </section>
  );
}

export function StatsPage() {
  const { data: games = [] } = useLibrary();
  const [days, setDays] = useState<RangeDays>(30);
  const [gameId, setGameId] = useState<number | null>(null);
  const { result, sessions, isLoading } = useStats(days, gameId);
  const game = games.find((g) => g.id === gameId) ?? null;
  const titleOf = useMemo(() => new Map(games.map((g) => [g.id, g.title])), [games]);

  const pickerGames = useMemo(
    () =>
      [...games].sort(
        (a, b) =>
          b.playtime.totalSeconds - a.playtime.totalSeconds ||
          a.title.localeCompare(b.title, "tr-TR"),
      ),
    [games],
  );

  const dailyLabels = useMemo(
    () =>
      result.daily.map((d) =>
        days <= 30
          ? d.date.toLocaleDateString("tr-TR", { day: "numeric", month: "short" })
          : d.date.toLocaleDateString("tr-TR", { day: "numeric", month: "numeric" }),
      ),
    [result.daily, days],
  );
  const dailyTitles = useMemo(
    () =>
      result.daily.map((d) =>
        d.date.toLocaleDateString("tr-TR", { weekday: "long", day: "numeric", month: "long" }),
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
  const hourSeries = useMemo<BarSeries[]>(
    () => [{ label: "Oynama", data: result.byHour, color: CHART_THEME.active }],
    [result.byHour],
  );
  const weekdaySeries = useMemo<BarSeries[]>(
    () => [{ label: "Oynama", data: result.byWeekday, color: CHART_THEME.active }],
    [result.byWeekday],
  );

  // Tüm zamanlar: Steam'in bildirdiği + Oyun Arşivim'in ölçtüğü (büyük olan).
  const lifetime = useMemo(
    () => pickerGames.filter((g) => g.playtime.totalSeconds > 0).slice(0, 10),
    [pickerGames],
  );
  const lifetimeLabels = useMemo(() => lifetime.map((g) => g.title), [lifetime]);
  const lifetimeSeries = useMemo<BarSeries[]>(
    () => [
      {
        label: "Toplam",
        data: lifetime.map((g) => g.playtime.totalSeconds),
        color: CHART_THEME.active,
      },
    ],
    [lifetime],
  );

  const hasTracked = result.totals.total > 0;
  const ins = result.insights;
  const insights: string[] = [];
  if (hasTracked) {
    if (ins.peakWindow)
      insights.push(`En çok ${formatHourRange(ins.peakWindow)} arasında oynuyorsun.`);
    if (ins.busiestDay !== null) insights.push(`En aktif günün ${WEEKDAYS[ins.busiestDay] ?? ""}.`);
    if (ins.quietestDay !== null && ins.quietestDay !== ins.busiestDay)
      insights.push(`En az ${WEEKDAYS[ins.quietestDay] ?? ""} günleri oynuyorsun.`);
    insights.push(
      `Son ${String(days)} günün ${String(ins.activeDays)} gününde oyun açtın; sürenin %${String(Math.round(result.totals.afkRatio * 100))}'i AFK geçti.`,
    );
  }

  return (
    <div className="page-content">
      <PageHeader
        eyebrow="İstatistikler"
        title={game ? game.title : "Oyun alışkanlıkların"}
        subtitle={
          game
            ? `${PLATFORM_LABELS[game.platform]} · ayrıntılı oynama istatistikleri`
            : "Tüm oyunlarının oynama süreleri ve alışkanlıkların."
        }
      />

      <div className="toolbar">
        <GamePicker games={pickerGames} value={gameId} onChange={setGameId} />
        <div className="chip-group" role="radiogroup" aria-label="Zaman aralığı">
          {RANGES.map((r) => (
            <button
              key={r.value}
              type="button"
              role="radio"
              aria-checked={days === r.value}
              className={`chip${days === r.value ? " is-selected" : ""}`}
              onClick={() => {
                setDays(r.value);
              }}
            >
              {r.label}
            </button>
          ))}
        </div>
      </div>

      <div className="kpi-grid">
        <Kpi
          label={`Son ${String(days)} gün`}
          value={formatDuration(result.totals.total)}
          hint="Oyun Arşivim ölçümü"
        />
        <Kpi label="Aktif oynama" value={formatDuration(result.totals.active)} />
        <Kpi
          label="AFK"
          value={formatDuration(result.totals.idle)}
          hint={
            hasTracked ? `%${String(Math.round(result.totals.afkRatio * 100))} oran` : undefined
          }
        />
        <Kpi
          label="Oturum"
          value={String(result.sessions.count)}
          hint={
            result.sessions.count ? `ort. ${formatDuration(result.sessions.average)}` : undefined
          }
        />
        <Kpi label="En uzun oturum" value={formatDuration(result.sessions.longest)} />
        {game ? (
          <Kpi
            label="Tüm zamanlar"
            value={formatDuration(game.playtime.totalSeconds)}
            hint={
              game.playtime.platformMinutes !== null
                ? `Steam'e göre · son 2 hafta ${formatDuration((game.playtime.platformTwoWeeksMinutes ?? 0) * 60)}`
                : "Oyun Arşivim ölçümü"
            }
          />
        ) : (
          <Kpi
            label="Tüm zamanlar"
            value={formatDuration(games.reduce((s, g) => s + g.playtime.totalSeconds, 0))}
            hint="Steam + Oyun Arşivim"
          />
        )}
      </div>

      {!hasTracked && !isLoading && (
        <EmptyState
          icon="chart-line-up"
          title="Bu aralıkta ölçülmüş oyun yok"
          description="Oyun Arşivim artık oynadığın oyunları otomatik izliyor (tepsideyken de). Oynadıkça günlük süreler, AFK oranı, saat ve gün dağılımları burada belirecek."
        />
      )}

      {hasTracked && (
        <>
          {insights.length > 0 && (
            <ul className="insights glass">
              {insights.map((t) => (
                <li key={t}>{t}</li>
              ))}
            </ul>
          )}
          <div className="chart-grid">
            <ChartCard title="Günlük oynama" wide>
              <BarChart
                labels={dailyLabels}
                titles={dailyTitles}
                series={dailySeries}
                format={hoursFmt}
                stacked
                ariaLabel="Günlük aktif ve AFK oynama süresi"
              />
            </ChartCard>
            <ChartCard title="Günün saatleri">
              <BarChart
                labels={HOUR_LABELS}
                series={hourSeries}
                format={hoursFmt}
                ariaLabel="Günün saatlerine göre oynama"
              />
            </ChartCard>
            <ChartCard title="Haftanın günleri">
              <BarChart
                labels={WEEKDAYS_SHORT}
                titles={WEEKDAYS}
                series={weekdaySeries}
                format={hoursFmt}
                ariaLabel="Haftanın günlerine göre oynama"
              />
            </ChartCard>
            <ChartCard title="Yoğunluk haritası" wide>
              <Heatmap data={result.heat} />
            </ChartCard>
          </div>
        </>
      )}

      {!game && lifetime.length > 0 && (
        <ChartCard title="Tüm zamanların en çok oynananları" wide>
          <BarChart
            labels={lifetimeLabels}
            series={lifetimeSeries}
            format={hoursFmt}
            horizontal
            height={Math.max(160, lifetime.length * 34)}
            ariaLabel="Tüm zamanlarda en çok oynanan oyunlar"
          />
        </ChartCard>
      )}

      {sessions.length > 0 && (
        <section className="chart-card glass chart-card--wide">
          <h2 className="text-h3">Oturum geçmişi</h2>
          <div className="table-wrap">
            <table className="data-table">
              <thead>
                <tr>
                  {!game && <th>Oyun</th>}
                  <th>Başlangıç</th>
                  <th className="num">Süre</th>
                  <th className="num">Aktif</th>
                  <th className="num">AFK</th>
                </tr>
              </thead>
              <tbody>
                {sessions.slice(0, 50).map((s) => (
                  <tr key={s.id}>
                    {!game && <td>{titleOf.get(s.gameId) ?? "—"}</td>}
                    <td>
                      {new Date(s.startedAt * 1000).toLocaleString("tr-TR", {
                        day: "numeric",
                        month: "short",
                        hour: "2-digit",
                        minute: "2-digit",
                      })}
                      {s.open && <span className="live-dot" title="Şu an oynanıyor" />}
                    </td>
                    <td className="num">{formatDuration(s.activeSeconds + s.idleSeconds)}</td>
                    <td className="num">{formatDuration(s.activeSeconds)}</td>
                    <td className="num">{formatDuration(s.idleSeconds)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>
      )}
    </div>
  );
}
