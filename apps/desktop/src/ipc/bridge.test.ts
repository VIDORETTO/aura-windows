import { getBridge, HOST_EVENT, setBridge } from "./bridge";
import { createMockBridge } from "./mock";
import type { AuthStatus, HostEvent } from "./types";

describe("Bridge initialization", () => {
  afterEach(() => {
    setBridge(null);
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
    vi.doUnmock("@tauri-apps/api/core");
  });

  it("shares the backend across concurrent callers and delivers login events to its subscriber", async () => {
    setBridge(null);
    const events: HostEvent[] = [];
    const [subscriber, caller, catalog] = await Promise.all([getBridge(), getBridge(), getBridge()]);
    const off = await subscriber.listen<HostEvent>(HOST_EVENT, (event) => events.push(event));
    await caller.invoke("auth_login");
    expect(events.filter((event) => event.channel === "login").map((event) => event.event.state)).toEqual(["waitingBrowser", "completed"]);
    expect((await catalog.invoke<AuthStatus>("auth_status")).active?.signedIn).toBe(true);
    expect(caller).toBe(subscriber);
    expect(catalog).toBe(subscriber);
    off();
  });

  it("preserves an injected bridge when an older initialization finishes", async () => {
    setBridge(null);
    const oldInitialization = getBridge();
    const injected = createMockBridge({ tick: 0, signedIn: true });
    setBridge(injected);
    await oldInitialization;
    expect(await getBridge()).toBe(injected);
    expect((await (await getBridge()).invoke<AuthStatus>("auth_status")).active?.signedIn).toBe(true);
  });

  it("starts a fresh shared backend after resetting a pending initialization", async () => {
    setBridge(null);
    const oldInitialization = getBridge();
    setBridge(null);
    const [fresh, other] = await Promise.all([getBridge(), getBridge()]);
    const old = await oldInitialization;
    expect(fresh).not.toBe(old);
    expect(other).toBe(fresh);
    expect(await getBridge()).toBe(fresh);
  });

  it("retries a failed native adapter initialization instead of keeping its rejected promise", async () => {
    setBridge(null);
    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};
    // The native adapter is an external boundary; no local module is mocked.
    vi.doMock("@tauri-apps/api/core", () => { throw new Error("QA native adapter unavailable"); });
    await expect(getBridge()).rejects.toMatchObject({ cause: { message: "QA native adapter unavailable" } });
    vi.doMock("@tauri-apps/api/core", () => ({ invoke: async () => "QA native adapter recovered" }));
    const recovered = await getBridge();
    expect(recovered.isTauri).toBe(true);
    expect(await recovered.invoke("qa_probe")).toBe("QA native adapter recovered");
  });
});
