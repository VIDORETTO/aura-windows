// Conversation state. `reduce` is pure (tested against the Rust golden
// fixture); the store batches streaming deltas per animation frame so a fast
// model never causes one React render per token.

import { create } from "zustand";
import type {
  ContextChip,
  ConversationEvent,
  FileChangeSummary,
  ItemStatus,
  PlanStep,
  ToolKind,
  TurnError,
  TurnStatus,
} from "../ipc/types";

export interface ToolItem {
  id: string;
  kind: ToolKind;
  title: string;
  status: ItemStatus;
  detail: string | null;
  startedAt: number;
  finishedAt?: number;
}

export interface Approval {
  requestId: string;
  kind: "command" | "fileChange" | "permissions";
  command: string | null;
  cwd: string | null;
  reason: string | null;
  changes: FileChangeSummary[];
  options: string[];
}

export type Block =
  | { type: "user"; id: string; text: string; chips: ContextChip[]; display?: string }
  | { type: "assistant"; id: string; text: string; streaming: boolean }
  | { type: "reasoning"; id: string; text: string }
  | { type: "tool"; item: ToolItem }
  | { type: "files"; id: string; changes: FileChangeSummary[]; status: ItemStatus }
  | { type: "approval"; approval: Approval; resolved?: string }
  | { type: "plan"; id: string; text: string }
  | { type: "input"; requestId: string; source: string; prompt: unknown; autoResolveMs: number | null; resolved?: string }
  | { type: "error"; id: string; error: TurnError };

export interface Thread {
  id: string;
  blocks: Block[];
  running: boolean;
  turnId: string | null;
  plan: { explanation: string | null; steps: PlanStep[] } | null;
  diff: string | null;
  tokens: { used: number; window: number | null } | null;
  lastStatus: TurnStatus | null;
}

export function emptyThread(id: string): Thread {
  return { id, blocks: [], running: false, turnId: null, plan: null, diff: null, tokens: null, lastStatus: null };
}

/** Requests not tied to a thread (e.g. MCP elicitation) live here. */
export const GLOBAL_THREAD = "__global__";

export interface ConvState {
  threads: Record<string, Thread>;
  /** requestId → threadId, to resolve approvals. */
  requests: Record<string, string>;
}

const now = () => Date.now();

function upsertBlock(blocks: Block[], match: (b: Block) => boolean, make: () => Block, update: (b: Block) => Block): Block[] {
  const i = blocks.findIndex(match);
  if (i === -1) return [...blocks, make()];
  const copy = blocks.slice();
  copy[i] = update(blocks[i]);
  return copy;
}

/** Pure reducer: one conversation event → next state. */
export function reduce(state: ConvState, ev: ConversationEvent): ConvState {
  if (ev.type === "appServerState") return state;
  if (ev.type === "requestResolved") {
    const threadId = state.requests[ev.requestId];
    if (!threadId) return state;
    const t = state.threads[threadId];
    const blocks = t.blocks.map((b) =>
      (b.type === "approval" && b.approval.requestId === ev.requestId && !b.resolved) ||
      (b.type === "input" && b.requestId === ev.requestId && !b.resolved)
        ? { ...b, resolved: "resolved" }
        : b,
    );
    const { [ev.requestId]: _, ...requests } = state.requests;
    return { threads: { ...state.threads, [threadId]: { ...t, blocks } }, requests };
  }
  const threadId = ev.type === "userInputRequested" ? (ev.threadId ?? GLOBAL_THREAD) : ev.threadId;
  if (!threadId) return state;
  const t = state.threads[threadId] ?? emptyThread(threadId);
  let next: Thread = t;
  let requests = state.requests;

  switch (ev.type) {
    case "turnStarted":
      next = { ...t, running: true, turnId: ev.turnId, lastStatus: null, plan: null, diff: null };
      break;
    case "messageDelta":
      next = {
        ...t,
        blocks: upsertBlock(
          t.blocks,
          (b) => b.type === "assistant" && b.id === ev.itemId,
          () => ({ type: "assistant", id: ev.itemId, text: ev.delta, streaming: true }),
          (b) => (b.type === "assistant" ? { ...b, text: b.text + ev.delta } : b),
        ),
      };
      break;
    case "messageCompleted":
      next = {
        ...t,
        blocks: upsertBlock(
          t.blocks,
          (b) => b.type === "assistant" && b.id === ev.itemId,
          () => ({ type: "assistant", id: ev.itemId, text: ev.text, streaming: false }),
          (b) => (b.type === "assistant" ? { ...b, text: ev.text, streaming: false } : b),
        ),
      };
      break;
    case "reasoningDelta":
      next = {
        ...t,
        blocks: upsertBlock(
          t.blocks,
          (b) => b.type === "reasoning" && b.id === ev.itemId,
          () => ({ type: "reasoning", id: ev.itemId, text: ev.delta }),
          (b) => (b.type === "reasoning" ? { ...b, text: b.text + ev.delta } : b),
        ),
      };
      break;
    case "toolCall":
      next = {
        ...t,
        blocks: upsertBlock(
          t.blocks,
          (b) => b.type === "tool" && b.item.id === ev.itemId,
          () => ({ type: "tool", item: { id: ev.itemId, kind: ev.kind, title: ev.title, status: ev.status, detail: ev.detail, startedAt: now() } }),
          (b) =>
            b.type === "tool"
              ? {
                  type: "tool",
                  item: { ...b.item, status: ev.status, detail: ev.detail ?? b.item.detail, title: ev.title || b.item.title, finishedAt: ev.status === "inProgress" ? undefined : now() },
                }
              : b,
        ),
      };
      break;
    case "fileChanges":
      next = {
        ...t,
        blocks: upsertBlock(
          t.blocks,
          (b) => b.type === "files" && b.id === ev.itemId,
          () => ({ type: "files", id: ev.itemId, changes: ev.changes, status: ev.status }),
          (b) => (b.type === "files" ? { ...b, changes: ev.changes, status: ev.status } : b),
        ),
      };
      break;
    case "approvalRequested":
      requests = { ...requests, [ev.requestId]: threadId };
      next = {
        ...t,
        blocks: [
          ...t.blocks,
          {
            type: "approval",
            approval: { requestId: ev.requestId, kind: ev.kind, command: ev.command, cwd: ev.cwd, reason: ev.reason, changes: ev.changes, options: ev.options },
          },
        ],
      };
      break;
    case "userInputRequested":
      requests = { ...requests, [ev.requestId]: threadId };
      next = {
        ...t,
        blocks: [...t.blocks, { type: "input", requestId: ev.requestId, source: ev.source, prompt: ev.prompt, autoResolveMs: ev.autoResolveMs }],
      };
      break;
    case "planUpdated":
      next = { ...t, plan: { explanation: ev.explanation, steps: ev.steps } };
      break;
    case "planProposed":
      next = { ...t, blocks: [...t.blocks, { type: "plan", id: ev.itemId, text: ev.text }] };
      break;
    case "diffUpdated":
      next = { ...t, diff: ev.diff };
      break;
    case "tokenUsage":
      next = { ...t, tokens: { used: ev.used, window: ev.window } };
      break;
    case "compacted":
      break;
    case "turnCompleted": {
      const blocks = t.blocks.map((b) => (b.type === "assistant" && b.streaming ? { ...b, streaming: false } : b));
      next = {
        ...t,
        running: false,
        lastStatus: ev.status,
        blocks: ev.error ? [...blocks, { type: "error", id: `err_${ev.turnId}`, error: ev.error }] : blocks,
      };
      break;
    }
  }
  return { threads: { ...state.threads, [threadId]: next }, requests };
}

interface Store extends ConvState {
  activeId: string | null;
  setActive: (id: string | null) => void;
  addUserMessage: (threadId: string, text: string, chips: ContextChip[], display?: string) => void;
  loadTranscript: (threadId: string, messages: { role: string; text: string }[]) => void;
  markApproval: (requestId: string, how: string) => void;
  apply: (ev: ConversationEvent) => void;
  reset: () => void;
}

let pending: ConversationEvent[] = [];
let scheduled = false;
const raf: (cb: () => void) => void =
  typeof requestAnimationFrame === "function" ? (cb) => requestAnimationFrame(() => cb()) : (cb) => setTimeout(cb, 16);

export const useConversation = create<Store>((set, get) => ({
  threads: {},
  requests: {},
  activeId: null,
  setActive: (id) => set({ activeId: id }),
  addUserMessage: (threadId, text, chips, display) =>
    set((s) => {
      const t = s.threads[threadId] ?? emptyThread(threadId);
      const block: Block = { type: "user", id: `u_${now()}_${t.blocks.length}`, text, chips, display };
      return { threads: { ...s.threads, [threadId]: { ...t, blocks: [...t.blocks, block], running: true } } };
    }),
  loadTranscript: (threadId, messages) =>
    set((s) => ({
      threads: {
        ...s.threads,
        [threadId]: {
          ...emptyThread(threadId),
          blocks: messages.map((m, i): Block =>
            m.role === "user"
              ? { type: "user", id: `h_${i}`, text: m.text, chips: [] }
              : { type: "assistant", id: `h_${i}`, text: m.text, streaming: false },
          ),
        },
      },
    })),
  markApproval: (requestId, how) =>
    set((s) => {
      const threadId = s.requests[requestId];
      if (!threadId) return s;
      const t = s.threads[threadId];
      return {
        threads: {
          ...s.threads,
          [threadId]: {
            ...t,
            blocks: t.blocks.map((b) =>
              (b.type === "approval" && b.approval.requestId === requestId) || (b.type === "input" && b.requestId === requestId) ? { ...b, resolved: how } : b,
            ),
          },
        },
      };
    }),
  apply: (ev) => {
    // Deltas are coalesced per frame; everything else flushes immediately
    // (after pending deltas, preserving order).
    pending.push(ev);
    const isDelta = ev.type === "messageDelta" || ev.type === "reasoningDelta";
    const flush = () => {
      scheduled = false;
      const batch = pending;
      pending = [];
      if (batch.length === 0) return;
      let st: ConvState = { threads: get().threads, requests: get().requests };
      for (const e of batch) st = reduce(st, e);
      set(st);
    };
    if (!isDelta) {
      flush();
    } else if (!scheduled) {
      scheduled = true;
      raf(flush);
    }
  },
  reset: () => {
    pending = [];
    set({ threads: {}, requests: {}, activeId: null });
  },
}));
