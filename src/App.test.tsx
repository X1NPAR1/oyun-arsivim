import { render, screen } from "@testing-library/react";
import { vi } from "vitest";
import App from "./App";

type InvokeImpl = (cmd: string) => Promise<unknown>;
let invokeImpl: InvokeImpl = () => Promise.resolve(undefined);
const calls: string[] = [];

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string): Promise<unknown> => {
    calls.push(cmd);
    return invokeImpl(cmd);
  },
}));

describe("App", () => {
  beforeEach(() => {
    calls.length = 0;
  });

  it("Rust'tan gelen sürümü gösterir", async () => {
    invokeImpl = () => Promise.resolve({ name: "Oyun Arşivim", version: "0.1.0" });
    render(<App />);
    expect(await screen.findByTestId("version")).toHaveTextContent("Oyun Arşivim v0.1.0");
    expect(calls).toEqual(["app_info"]);
  });

  it("IPC hatasını anlaşılır şekilde gösterir", async () => {
    invokeImpl = () => Promise.reject(new Error("bağlantı yok"));
    render(<App />);
    expect(await screen.findByRole("alert")).toHaveTextContent("bağlantı yok");
  });
});
