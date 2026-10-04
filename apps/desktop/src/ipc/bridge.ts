// Chooses the transport: real Tauri IPC inside the app, an in-memory mock
// backend in a plain browser (`pnpm dev`) and in tests.

import type { HostEvent } from "./types";

export type Invoke = <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
export type Unlisten = () => void;
export type Listen = <T>(event: string, handler: (payload: T) => void) => Promise<Unlisten>;

export interface Bridge {
  invoke: Invoke;
  listen: Listen;
  /** Emits a local event (mock only; no-op in Tauri). */
  emitLocal?: (event: string, payload: unknown) => void;
  isTauri: boolean;
}

export const HOST_EVENT = "aura://event";

let bridge: Bridge | null = null;
let initializing: Promise<Bridge> | null = null;

export function inTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function tauriBridge(): Promise<Bridge> {
  const core = await import("@tauri-apps/api/core");
  const ev = await import("@tauri-apps/api/event");
  return {
    isTauri: true,
    invoke: (cmd, args) => core.invoke(cmd, args),
    listen: async (event, handler) => ev.listen(event, (e) => handler(e.payload as never)),
  };
}

export function getBridge(): Promise<Bridge> {
  if (bridge) return Promise.resolve(bridge);
  if (initializing) return initializing;
  const pending = (inTauri()
    ? tauriBridge()
    : import("./mock").then(({ createMockBridge }) => createMockBridge()))
    .then((created) => {
      if (initializing === pending) bridge = created;
      return created;
    })
    .catch((error) => {
      if (initializing === pending) initializing = null;
      throw error;
    });
  initializing = pending;
  return pending;
}

/** Tests inject a bridge (usually a fresh mock). */
export function setBridge(b: Bridge | null) {
  initializing = null;
  bridge = b;
}

export async function onHostEvent(handler: (e: HostEvent) => void): Promise<Unlisten> {
  return (await getBridge()).listen<HostEvent>(HOST_EVENT, handler);
}
