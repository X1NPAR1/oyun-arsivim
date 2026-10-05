import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createMemoryRouter, RouterProvider } from "react-router";
import { vi } from "vitest";
import { routes } from "./routes";

let settings = {
  launchAtStartup: false,
  closeToTray: true,
  motion: "system",
  autoRamClean: false,
  autoRamThreshold: 85,
  closeList: [] as string[],
  restartClosedApps: true,
};

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: { patch?: object }): Promise<unknown> => {
    if (cmd === "list_games") return Promise.resolve([]);
    if (cmd === "get_settings") return Promise.resolve(settings);
    if (cmd === "update_settings") {
      settings = { ...settings, ...args?.patch };
      return Promise.resolve(settings);
    }
    return Promise.resolve("none");
  },
  convertFileSrc: (p: string) => p,
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => undefined) }));
vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => {
    throw new Error("Tauri dışında");
  },
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: () => Promise.resolve(null) }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => {
    throw new Error("Tauri dışında");
  },
}));

function renderAt(path: string) {
  const router = createMemoryRouter(routes, { initialEntries: [path] });
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={client}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return router;
}

describe("uygulama kabuğu", () => {
  it("özel başlık çubuğunu ve pencere düğmelerini gösterir", () => {
    renderAt("/");
    expect(screen.getByRole("button", { name: "Küçült" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Büyüt" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Kapat" })).toBeInTheDocument();
  });

  it("menüden sayfalar arasında gezinir ve aktif öğeyi işaretler", async () => {
    const user = userEvent.setup();
    const router = renderAt("/");
    const nav = screen.getByRole("navigation", { name: "Ana menü" });

    await user.click(within(nav).getByRole("link", { name: /Kütüphane/ }));
    expect(router.state.location.pathname).toBe("/kutuphane");
    expect(await screen.findByRole("heading", { name: "Oyunların" })).toBeInTheDocument();
    expect(within(nav).getByRole("link", { name: /Kütüphane/ })).toHaveAttribute(
      "aria-current",
      "page",
    );
  });

  it("ayarlarda animasyon seviyesi değiştirilebilir ve kalıcıdır", async () => {
    const user = userEvent.setup();
    renderAt("/ayarlar");
    const radio = screen.getByRole("radio", { name: "Kapalı" });
    await user.click(radio);
    expect(radio).toHaveAttribute("aria-checked", "true");
    await vi.waitFor(() => {
      expect(settings.motion).toBe("off");
    });
  });

  it("Windows açılışında başlat ayarı kaydedilir", async () => {
    const user = userEvent.setup();
    renderAt("/ayarlar");
    const sw = await screen.findByRole("switch", { name: "Windows açılışında başlat" });
    await vi.waitFor(() => {
      expect(sw).toBeEnabled();
    });
    await user.click(sw);
    await vi.waitFor(() => {
      expect(settings.launchAtStartup).toBe(true);
    });
    expect(sw).toHaveAttribute("aria-checked", "true");
  });

  it("arayüzde SVG öğesi bulunmaz", () => {
    renderAt("/");
    expect(document.querySelectorAll("svg")).toHaveLength(0);
  });
});
