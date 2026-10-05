import { EmptyState } from "../components/EmptyState";
import { PageHeader } from "../components/PageHeader";

export function StatsPage() {
  return (
    <div className="page-content">
      <PageHeader
        eyebrow="İstatistikler"
        title="Oyun alışkanlıkların"
        subtitle="Günlük, haftalık ve aylık oynama süreleri."
      />
      <EmptyState
        icon="chart-line-up"
        title="Henüz oynama verisi yok"
        description="Oyun oynadıkça süreler burada grafiklere ve ısı haritasına dönüşecek."
      />
    </div>
  );
}
