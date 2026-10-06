import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { useRunningStore } from "../stores/running";
import { LIBRARY_KEY, useLaunchGame } from "./useLibrary";

const invoke = vi.fn<(cmd: string, args?: unknown) => Promise<null>>(() => Promise.resolve(null));
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invoke(cmd, args),
  convertFileSrc: (p: string) => p,
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => undefined) }));

function setup() {
  const client = new QueryClient({ defaultOptions: { mutations: { retry: false } } });
  client.setQueryData(LIBRARY_KEY, [{ id: 2, title: "Unturned" }]);
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  );
  return renderHook(() => useLaunchGame(), { wrapper });
}

describe("aynı anda tek oyun", () => {
  beforeEach(() => {
    invoke.mockClear();
    useRunningStore.setState({ games: [], conflict: null });
  });

  it("oyun yokken doğrudan başlatır", async () => {
    const { result } = setup();
    await act(() => result.current.mutateAsync(2));
    expect(invoke).toHaveBeenCalledWith("launch_game", { id: 2 });
  });

  it("başka oyun açıkken başlatmaz, kullanıcıya sorar", async () => {
    useRunningStore.setState({
      games: [{ id: 1, title: "Counter-Strike 2", since: 0, starting: false }],
    });
    const { result } = setup();
    act(() => {
      result.current.mutate(2);
    });
    await waitFor(() => {
      expect(useRunningStore.getState().conflict).not.toBeNull();
    });
    expect(useRunningStore.getState().conflict).toMatchObject({
      current: { title: "Counter-Strike 2" },
      next: { id: 2, title: "Unturned" },
    });
    expect(invoke).not.toHaveBeenCalledWith("launch_game", expect.anything());
  });

  it("aynı oyun açıksa çakışma sayılmaz", async () => {
    useRunningStore.setState({ games: [{ id: 2, title: "Unturned", since: 0, starting: false }] });
    const { result } = setup();
    await act(() => result.current.mutateAsync(2));
    expect(useRunningStore.getState().conflict).toBeNull();
  });
});
