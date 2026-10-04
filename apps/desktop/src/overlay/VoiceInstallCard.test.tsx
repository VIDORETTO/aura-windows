import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { freshApp } from "../test/harness";
import { setBridge } from "../ipc/bridge";
import type { ModelView } from "../ipc/types";
import { OverlayApp } from "./OverlayApp";
import { useApp } from "../state/app";

const recommended: ModelView = {
  entry: { id: "qa-voice", name: "QA Voice", family: "whisper", engine: "ggml-whisper", description: "QA model", languages: ["*"], speed: 0.8, accuracy: 0.7, streaming: false, minRamMb: 1024, gpuRecommended: false, license: "MIT", sourceUrl: "http://127.0.0.1/qa" },
  sizeBytes: 10485760, installed: false, selected: false, recommended: true, downloading: false,
};

async function offerFixture(models: () => ModelView[] = () => [recommended]) {
  const bridge = await freshApp({ signedIn: true });
  await bridge.invoke("settings_update", { patch: { language: "en" } });
  const installs: unknown[] = [], cancels: unknown[] = [], selections: unknown[] = [];
  let presses = 0;
  setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
    if (cmd === "voice_models") return models() as R;
    if (cmd === "ptt_press") { presses++; throw { code: "asr", message: "model_missing" }; }
    if (cmd === "voice_install") { installs.push(args.id); return undefined as R; }
    if (cmd === "voice_cancel_install") { cancels.push(args.id); return undefined as R; }
    if (cmd === "voice_select") { selections.push(args.id); return undefined as R; }
    return bridge.invoke<R>(cmd, args);
  } });
  const user = userEvent.setup();
  render(<OverlayApp />);
  await user.click(await screen.findByRole("button", { name: "Dictate" }));
  await screen.findByRole("region", { name: "Install local dictation" });
  const progress = async (event: Record<string, unknown>) => act(async () => {
    bridge.emitLocal!("aura://event", { channel: "download", event: { id: "qa-voice", bytes: 0, total: 10485760, done: false, error: null, ...event } });
  });
  return { bridge, user, installs, cancels, selections, progress, presses: () => presses };
}

it("keeps the active download when the interface language changes its recommendation", async () => {
  let models = [recommended];
  const f = await offerFixture(() => models);
  await f.user.click(screen.getByRole("button", { name: "Download (10 MB)" }));
  await f.progress({ bytes: 5242880 });
  models = [{ ...recommended, recommended: false, downloading: true }, { ...recommended, entry: { ...recommended.entry, id: "other", name: "Other Voice" } }];
  await act(async () => { await f.bridge.invoke("settings_update", { patch: { language: "pt-BR" } }); });
  const card = await screen.findByRole("region", { name: "Instalar ditado local" });
  expect(card).toHaveTextContent("QA Voice");
  expect(card).not.toHaveTextContent("Other Voice");
  await f.user.click(screen.getByRole("button", { name: "Cancelar" }));
  expect(f.cancels).toEqual(["qa-voice"]);
});

it("selects the completed model without automatically opening the microphone", async () => {
  const f = await offerFixture();
  await f.user.click(screen.getByRole("button", { name: "Download (10 MB)" }));
  await f.progress({ bytes: 5242880 });
  expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "50");
  await f.progress({ done: true, bytes: 0, total: null });
  await waitFor(() => expect(f.selections).toEqual(["qa-voice"]));
  await waitFor(() => expect(screen.queryByRole("region", { name: "Install local dictation" })).not.toBeInTheDocument());
  expect(f.presses()).toBe(1);
  expect(screen.getByRole("button", { name: "Dictate" })).toHaveAttribute("aria-pressed", "false");
});

it("cancels a download without a failure warning and permits retry", async () => {
  const f = await offerFixture();
  await f.user.click(screen.getByRole("button", { name: "Download (10 MB)" }));
  await f.user.click(screen.getByRole("button", { name: "Cancel" }));
  expect(f.cancels).toEqual(["qa-voice"]);
  await f.progress({ error: "cancelled" });
  expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  expect(useApp.getState().notices).toEqual([]);
  expect(screen.getByRole("status")).toHaveTextContent("Download cancelled");
  await f.user.click(screen.getByRole("button", { name: "Download (10 MB)" }));
  expect(f.installs).toEqual(["qa-voice", "qa-voice"]);
  expect(screen.getByRole("button", { name: "Loading…" })).toBeDisabled();
});

it("offers the recommended model and its download size when dictation has no model", async () => {
  const bridge = await freshApp({ signedIn: true });
  await bridge.invoke("settings_update", { patch: { language: "en" } });
  const downloads: unknown[] = [];
  setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
    if (cmd === "voice_models") return [recommended] as R;
    if (cmd === "ptt_press") throw { code: "asr", message: "model_missing" };
    if (cmd === "voice_install") { downloads.push(args.id); return undefined as R; }
    return bridge.invoke<R>(cmd, args);
  } });
  const user = userEvent.setup();
  render(<OverlayApp />);
  await user.click(await screen.findByRole("button", { name: "Dictate" }));
  const card = await screen.findByRole("region", { name: "Install local dictation" });
  expect(card).toHaveTextContent("QA Voice");
  await user.click(screen.getByRole("button", { name: "Download (10 MB)" }));
  expect(downloads).toEqual(["qa-voice"]);
});
