import { useMemo } from "react";
import { Link } from "react-router";
import { GameShowcase } from "../components/home/GameShowcase";
import { HomeSummary } from "../components/home/HomeSummary";
import { Icon } from "../components/Icon";
import { useGameMenu } from "../hooks/useGameMenu";
import { useLaunchGame, useLibrary } from "../hooks/useLibrary";
import { showcaseOrder } from "../library/ordering";
import { summarize } from "../library/summary";

export function HomePage() {
  const { data: games = [], isLoading } = useLibrary();
  const launch = useLaunchGame();
  const gameMenu = useGameMenu();
  const ordered = useMemo(() => showcaseOrder(games), [games]);
  const summary = useMemo(() => summarize(games), [games]);

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
      <HomeSummary summary={summary} />
      {gameMenu.menu}
    </div>
  );
}
