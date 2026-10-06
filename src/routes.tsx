import { AppShell } from "./components/shell/AppShell";
import { BackupsPage } from "./pages/BackupsPage";
import { GameDetailPage } from "./pages/GameDetailPage";
import { HomePage } from "./pages/HomePage";
import { LibraryPage } from "./pages/LibraryPage";
import { PerformancePage } from "./pages/PerformancePage";
import { SettingsPage } from "./pages/SettingsPage";
import { StatsPage } from "./pages/StatsPage";

export const routes = [
  {
    path: "/",
    element: <AppShell />,
    children: [
      { index: true, element: <HomePage /> },
      { path: "kutuphane", element: <LibraryPage /> },
      { path: "kutuphane/:id", element: <GameDetailPage /> },
      { path: "istatistikler", element: <StatsPage /> },
      { path: "performans", element: <PerformancePage /> },
      { path: "yedekler", element: <BackupsPage /> },
      { path: "ayarlar", element: <SettingsPage /> },
    ],
  },
];
