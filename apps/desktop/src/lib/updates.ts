// Signed auto-update (010 TK-002) through the Tauri updater plugin.
import { inTauri } from "../ipc/bridge";
import { api } from "../ipc/commands";

export type UpdateCheck = { kind: "none" } | { kind: "available"; version: string; notes: string } | { kind: "unsupported" } | { kind: "notConfigured" };

export async function checkForUpdate(): Promise<UpdateCheck> {
  // A build without the release signing key cannot verify updates (QA-038).
  if (!(await api.updaterConfigured().catch(() => false))) return { kind: "notConfigured" };
  if (!inTauri()) return { kind: "unsupported" };
  const { check } = await import("@tauri-apps/plugin-updater");
  const update = await check();
  return update ? { kind: "available", version: update.version, notes: update.body ?? "" } : { kind: "none" };
}

/** Downloads, verifies the signature, installs and restarts. */
export async function installUpdate(onProgress: (pct: number) => void): Promise<void> {
  const { check } = await import("@tauri-apps/plugin-updater");
  const { relaunch } = await import("@tauri-apps/plugin-process");
  const update = await check();
  if (!update) return;
  let total = 0;
  let done = 0;
  await update.downloadAndInstall((e) => {
    if (e.event === "Started") total = e.data.contentLength ?? 0;
    if (e.event === "Progress") {
      done += e.data.chunkLength;
      if (total) onProgress(Math.round((done / total) * 100));
    }
  });
  await relaunch();
}
