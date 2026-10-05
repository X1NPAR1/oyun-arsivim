import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createMemoryRouter, RouterProvider } from "react-router";
import { vi } from "vitest";
import { routes } from "./routes";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (): Promise<unknown> => Promise.resolve("none"),
}));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => {
    throw new Error("Tauri dışında");
  },
}));

function renderAt(path: string) {
  const router = createMemoryRouter(routes, { initialEntries: [path] });
  render(<RouterProvider router={router} />);
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
    expect(localStorage.getItem("oa.motionPreference")).toBe("off");
  });

  it("arayüzde SVG öğesi bulunmaz", () => {
    renderAt("/");
    expect(document.querySelectorAll("svg")).toHaveLength(0);
  });
});
