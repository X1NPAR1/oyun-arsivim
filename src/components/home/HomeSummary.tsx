import { Link } from "react-router";
import type { IconName } from "../../design/iconCodes";
import { formatRelative } from "../../ipc/library";
import type { Summary } from "../../library/summary";
import { formatDuration } from "../../stats/aggregate";
import { Icon } from "../Icon";

function Item({
  icon,
  label,
  value,
  hint,
}: {
  icon: IconName;
  label: string;
  value: string;
  hint?: string;
}) {
  return (
    <div className="summary__item">
      <span className="summary__icon">
        <Icon name={icon} size={18} />
      </span>
      <div className="summary__text">
        <span className="summary__label">{label}</span>
        <span className="summary__value" title={value}>
          {value}
        </span>
        {hint && <span className="summary__hint">{hint}</span>}
      </div>
    </div>
  );
}

export function HomeSummary({ summary }: { summary: Summary }) {
  const { weekSeconds, topWeek, lastPlayed, installed, lifetimeSeconds } = summary;
  return (
    <Link to="/istatistikler" className="summary glass" aria-label="İstatistiklere git">
      <Item
        icon="clock"
        label="Bu hafta"
        value={weekSeconds > 0 ? formatDuration(weekSeconds) : "Henüz oynanmadı"}
      />
      <Item
        icon="star"
        label="Haftanın favorisi"
        value={topWeek?.title ?? "—"}
        hint={topWeek ? formatDuration(topWeek.playtime.weekSeconds) : undefined}
      />
      <Item
        icon="game-controller"
        label="Son oynanan"
        value={lastPlayed?.title ?? "—"}
        hint={formatRelative(lastPlayed?.lastPlayed ?? null) ?? undefined}
      />
      <Item
        icon="squares-four"
        label="Kütüphane"
        value={`${String(installed)} oyun`}
        hint={lifetimeSeconds > 0 ? `${formatDuration(lifetimeSeconds)} toplam` : undefined}
      />
    </Link>
  );
}
