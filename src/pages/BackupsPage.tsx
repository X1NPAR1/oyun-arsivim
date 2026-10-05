import { EmptyState } from "../components/EmptyState";
import { PageHeader } from "../components/PageHeader";

export function BackupsPage() {
  return (
    <div className="page-content">
      <PageHeader
        eyebrow="Yedekler"
        title="Kayıt dosyaların"
        subtitle="Save yedekleri, geri yükleme ve depolama kullanımı."
      />
      <EmptyState
        icon="cloud-arrow-up"
        title="Henüz yedek yok"
        description="Oyunların bulunduğunda kayıt dosyaları otomatik olarak yedeklenebilecek."
      />
    </div>
  );
}
