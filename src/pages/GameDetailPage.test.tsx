import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createMemoryRouter, RouterProvider } from "react-router";
import { vi } from "vitest";
import { routes } from "../routes";
import { EMPTY_PLAYTIME } from "../test/fixtures";

const game = {
  id: 7,
  platform: "steam",
  platformId: "730",
  title: "Counter-Strike 2",
  installDir: "D:\\SteamLibrary\\steamapps\\common\\Counter-Strike Global Offensive",
  executable: null,
  launchUri: "steam://rungameid/730",
  launchArgs: null,
  sizeBytes: 73_984_302_411,
  lastPlayed: null,
  installed: true,
  addedAt: 1_790_000_000,
  removedAt: null,
  art: {
    cover: null,
    logo: null,
    hero: null,
    icon: null,
    backdrop: null,
    accent: null,
    logoDark: false,
  },
  playtime: { ...EMPTY_PLAYTIME, totalSeconds: 358_753 * 60, platformMinutes: 358_753 },
};
const notes: Record<number, string> = { 7: "Ayarlar: düşük gölge" };

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (p: string) => p,
  invoke: (cmd: string, args?: { id?: number; notes?: string }): Promise<unknown> => {
    switch (cmd) {
      case "list_games":
        return Promise.resolve([game]);
      case "get_game_notes":
        return Promise.resolve(notes[args?.id ?? 0] ?? "");
      case "playtime_hourly":
      case "playtime_sessions":
        return Promise.resolve([]);
      case "perf_profile_get":
        return Promise.resolve({
          enabled: false,
          highPriority: false,
          powerPlan: false,
          cleanRam: false,
          closeApps: false,
          muteNotifications: false,
        });
      case "get_settings":
        return Promise.resolve({
          launchAtStartup: false,
          closeToTray: true,
          motion: "off",
          autoRamClean: false,
          autoRamThreshold: 85,
          closeList: [],
          restartClosedApps: true,
        });
      default:
        return Promise.resolve(null);
    }
  },
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => undefined) }));
vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => {
    throw new Error("Tauri dışında");
  },
}));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => {
    throw new Error("Tauri dışında");
  },
}));

function renderAt(path: string) {
  const router = createMemoryRouter(routes, {
    initialEntries: ["/kutuphane", path],
    initialIndex: 1,
  });
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={client}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return router;
}

describe("oyun detay sayfası", () => {
  it("oyun bilgilerini, süreyi ve kayıtlı notu gösterir", async () => {
    renderAt("/kutuphane/7");
    expect(await screen.findByRole("heading", { name: /Counter-Strike 2/ })).toBeInTheDocument();
    expect(screen.getByText(/5\.979 sa toplam/)).toBeInTheDocument();
    expect(await screen.findByDisplayValue("Ayarlar: düşük gölge")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Oyna/ })).toBeEnabled();
  });

  it("Esc ile önceki sayfaya döner", async () => {
    const user = userEvent.setup();
    const router = renderAt("/kutuphane/7");
    await screen.findByRole("heading", { name: /Counter-Strike 2/ });
    await user.keyboard("{Escape}");
    expect(router.state.location.pathname).toBe("/kutuphane");
  });

  it("bilinmeyen oyunda anlaşılır boş durum", async () => {
    renderAt("/kutuphane/999");
    expect(await screen.findByText("Oyun bulunamadı")).toBeInTheDocument();
  });
});
