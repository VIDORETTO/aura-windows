// Replays the Rust-generated golden file (crates/aura-app/tests/ipc_contract.rs)
// through the TypeScript types and reducers: if either side changes shape,
// one of the two test suites fails.
import contract from "./__fixtures__/contract.json";
import type { ContextChip, ConversationEvent, HostEvent, McpServerSpec, Settings } from "./types";
import { reduce, type ConvState } from "../state/conversation";
import { useApp } from "../state/app";
import { createMockBridge } from "./mock";

const events = contract.events as unknown as HostEvent[];

describe("IPC contract", () => {
  it("conversation events fold into the expected thread", () => {
    let st: ConvState = { threads: {}, requests: {} };
    for (const e of events) if (e.channel === "conversation") st = reduce(st, e.event as ConversationEvent);
    const t = st.threads["thr_1"];
    expect(t).toBeDefined();
    const assistant = t.blocks.find((b) => b.type === "assistant");
    expect(assistant).toMatchObject({ type: "assistant", text: "Olá, mundo", streaming: false });
    const tool = t.blocks.find((b) => b.type === "tool");
    expect(tool && tool.type === "tool" && tool.item.status).toBe("completed");
    const approval = t.blocks.find((b) => b.type === "approval");
    expect(approval && approval.type === "approval" && approval.resolved).toBe("resolved");
    expect(t.plan?.steps.map((s) => s.status)).toEqual(["completed", "inProgress"]);
    expect(t.tokens).toEqual({ used: 1234, window: 200000 });
    const err = t.blocks.find((b) => b.type === "error");
    expect(err && err.type === "error" && err.error).toEqual({ kind: "usageLimit", retryAfterSecs: 60 });
    expect(t.running).toBe(false);
    expect(st.requests).toEqual({});
  });

  it("every host channel is handled by the app store", () => {
    const handle = useApp.getState().handle;
    for (const e of events) expect(() => handle(e)).not.toThrow();
    const s = useApp.getState();
    expect(s.voice).toEqual({ state: "done", text: "texto" });
    expect(s.downloads["codex"]).toMatchObject({ bytes: 10, total: 100 });
    expect(s.consents).toEqual([]); // requested then resolved
    expect(s.appServer).toEqual({ state: "ready", version: "0.159.0" });
    expect(s.audioLevels).toEqual({ mic: -12.5 });
  });

  it("settings keys match the TS type and the mock defaults", () => {
    const rust = contract.settings as unknown as Settings;
    const mock = createMockBridge().state.settings;
    expect(Object.keys(rust).sort()).toEqual(Object.keys(mock).sort());
    expect(rust.invokeShortcut).toBe(mock.invokeShortcut);
  });

  it("chips, MCP specs, decisions and modes use the documented tags", () => {
    const attachment = contract.attachmentChip as unknown as ContextChip;
    expect(attachment.attachmentLabel).toEqual({ fileName: "1 linhas.txt", parts: [{ type: "count", amount: 1, unit: "line" }] });
    const chip = contract.chip as unknown as ContextChip;
    expect(chip.payload).toEqual({ type: "image", path: "C:/tmp/a.png" });
    expect(chip.previewPath).toBe("C:/tmp/a.png");
    const [stdio, http] = contract.mcpServers as unknown as McpServerSpec[];
    expect(stdio.transport.type).toBe("stdio");
    expect(stdio.approvalMode).toBe("askForWrites");
    expect(http.transport).toMatchObject({ type: "http", bearerSecret: true });
    expect(contract.decisions.map((d) => d.type)).toEqual(["accept", "acceptForSession", "decline", "cancel", "answer"]);
    expect(contract.modes.map((m) => m.mode)).toEqual(["chat", "task", "plan"]);
    expect(contract.captureModes.map((m) => m.type)).toEqual(["off", "onDemand", "recentBuffer", "manual", "continuous"]);
    expect(contract.preset).toMatchObject({ id: "groq", credentialRequired: true, wire: "chat" });
    expect(contract.privacy.screen).toEqual({ mode: { type: "onDemand" }, agent: "ask" });
  });
});
