import { useMemo } from "react";
import { Link } from "react-router";
import { GameShowcase } from "../components/home/GameShowcase";
import { Icon } from "../components/Icon";
import { useGameMenu } from "../hooks/useGameMenu";
import { useLaunchGame, useLibrary } from "../hooks/useLibrary";
import { formatBytes } from "../ipc/library";
import { showcaseOrder } from "../library/ordering";

export function HomePage() {
  const { data: games = [], isLoading } = useLibrary();
  const launch = useLaunchGame();
  const gameMenu = useGameMenu();
  const ordered = useMemo(() => showcaseOrder(games), [games]);
  const totalSize = useMemo(
    () => ordered.reduce((sum, g) => sum + (g.sizeBytes ?? 0), 0),
    [ordered],
  );
  const platformCount = useMemo(() => new Set(ordered.map((g) => g.platform)).size, [ordered]);

  if (!isLoading && ordered.length === 0) {
    return (
      <div className="page-content">
        <section className="hero glass">
          <p className="text-caption hero__eyebrow">Hoş geldin</p>
          <h1 className="text-display hero__title">Kütüphanen hazırlanıyor</h1>
          <p className="hero__text">
            Steam ve Epic Games oyunların taranıyor. Diğer oyunlarını kütüphaneden .exe dosyasıyla
            ekleyebilirsin.
          </p>
          <div className="hero__actions">
            <Link to="/kutuphane" className="btn btn--primary">
              <Icon name="squares-four" size={18} />
              Kütüphaneye git
            </Link>
          </div>
        </section>
      </div>
    );
  }

  return (
    <div className="page-content page-content--home">
      <GameShowcase
        games={ordered}
        onPlay={(g) => {
          launch.mutate(g.id);
        }}
        onContextMenu={gameMenu.openFor}
      />
      {gameMenu.menu}
      <div className="stat-row">
        <div className="stat glass">
          <span className="stat__value">{ordered.length}</span>
          <span className="stat__label">kurulu oyun</span>
        </div>
        <div className="stat glass">
          <span className="stat__value">{platformCount}</span>
          <span className="stat__label">platform</span>
        </div>
        <div className="stat glass">
          <span className="stat__value">{formatBytes(totalSize) ?? "—"}</span>
          <span className="stat__label">disk kullanımı</span>
        </div>
      </div>
    </div>
  );
}
