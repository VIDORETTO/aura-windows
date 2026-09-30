// Types mirroring the Rust serde shapes crossing the Tauri IPC boundary.
// Contract: `crates/aura-app/tests/ipc_contract.rs` writes golden JSON to
// `src/ipc/__fixtures__/` and `src/ipc/contract.test.ts` checks these types
// and the store reducers against it. Change both sides together.

// ------------------------------------------------------------------ settings

export type Theme = "system" | "light" | "dark";
export type Language = "ptBr" | "en";
export type FocusLoss = "hide" | "keepOpen";

export interface Settings {
  theme: Theme;
  opacity: number;
  language: Language;
  invokeShortcut: string;
  doubleTapCtrl: boolean;
  startWithWindows: boolean;
  focusLoss: FocusLoss;
  privacyPauseShortcut: string;
  pushToTalkShortcut: string;
  globalVoiceShortcut: string;
  attachScreenOnOpen: boolean;
  sendAfterDictation: boolean;
  defaultModel: string | null;
  defaultEffort: string | null;
  personalInstructions: string;
  appServerIdleMinutes: number;
  workerIdleMinutes: number;
  memories: boolean;
  asrLanguage: string | null;
  asrVocabulary: string[];
  cloudAsrProvider: string | null;
  onboarded: boolean;
}

export type SettingsPatch = Partial<Settings>;

// ------------------------------------------------------------------- privacy

export type Source = "screen" | "mic" | "systemAudio" | "selection";
export type CaptureMode =
  | { type: "off" }
  | { type: "onDemand" }
  | { type: "recentBuffer"; minutes: number }
  | { type: "manual" }
  | { type: "continuous" };
export type AgentPermission = "never" | "ask" | "always";

export interface SourcePolicy {
  mode: CaptureMode;
  agent: AgentPermission;
}

export interface ExclusionRule {
  id: string;
  process: string | null;
  titleGlob: string | null;
  class: string | null;
  enabled: boolean;
  builtin: boolean;
}

export interface PrivacyView {
  screen: SourcePolicy;
  mic: SourcePolicy;
  systemAudio: SourcePolicy;
  paused: boolean;
  exclusions: ExclusionRule[];
}

export interface PrivacyState {
  paused: boolean;
  screen: string;
  mic: string;
  systemAudio: string;
  /** Sources recording right now: `screen`, `mic`, `system`. */
  recording: string[];
}

export interface Recording {
  id: string;
  source: string;
  title: string;
  startedAt: number;
  endedAt: number | null;
  bytes: number;
}

export interface AccessLogEntry {
  at: number;
  source: string;
  requester: string;
  tool: string | null;
  conversation: string | null;
  decision: string;
  reason: string | null;
}

export interface ConsentRequest {
  id: string;
  conversation: string;
  tool: string;
  source: string;
  reason: string;
  app: string | null;
}

export type ConsentAnswer = "once" | "conversation" | "deny";

// ---------------------------------------------------------------------- auth

export interface ChatGptAccount {
  clientId: string;
  subject: string;
  email: string | null;
  scopes: string[];
  planUsageEnabled: boolean;
  expiresAt: number | null;
  active: boolean;
  welcomed: boolean;
  signedIn: boolean;
}

export interface AuthStatus {
  accounts: ChatGptAccount[];
  active: ChatGptAccount | null;
}

export type LoginProgress =
  | { state: "waitingBrowser"; authorizeUrl: string }
  | { state: "completed"; account: ChatGptAccount; firstTime: boolean }
  | { state: "failed"; reason: string }
  | { state: "cancelled" };

// ----------------------------------------------------------------- providers

export type Wire = "responses" | "chat" | "anthropic";
export type AuthStyle = "bearer" | "api-key-header" | "x-api-key" | "none";

export interface Quirks {
  noParallelToolCalls: boolean;
  reasoningEffort: boolean;
  noStreamOptions: boolean;
}

export interface Preset {
  id: string;
  name: string;
  wire: Wire;
  baseUrl: string;
  auth: string;
  credentialRequired: boolean;
  headers: Record<string, string>;
  quirks: Quirks;
  transcriptionModels: string[];
  tts: boolean;
}

export interface ModelSpec {
  id: string;
  displayName: string | null;
  contextWindow: number | null;
  maxOutput: number | null;
  supportsImages: boolean;
  supportsTools: boolean;
  supportsReasoning: boolean;
  estimated: boolean;
}

export interface Provider {
  id: string;
  name: string;
  preset: string;
  wire: Wire;
  baseUrl: string;
  auth: AuthStyle;
  extraHeaders: Record<string, string>;
  models: ModelSpec[];
  credentialHint: string | null;
  status: "unverified" | "verified" | "error";
  lastError: string | null;
  quirks: Quirks;
}

export interface ProviderDraft {
  id?: string | null;
  name: string;
  preset: string;
  wire?: Wire | null;
  baseUrl?: string | null;
  extraHeaders?: Record<string, string>;
}

// ------------------------------------------------------------- conversations

export type ConversationMode =
  | { mode: "chat" }
  | { mode: "task"; granted: string[]; network: boolean }
  | { mode: "plan" };

export interface StartOptions {
  mode?: ConversationMode;
  provider?: string;
  model?: string | null;
  ephemeral?: boolean;
  personalInstructions?: string;
  conversationInstructions?: string;
  previousSummary?: string | null;
  configOverrides?: Record<string, unknown>;
}

export interface StartedConversation {
  threadId: string;
  conversationUuid: string;
  workspace: string;
  ephemeral: boolean;
}

export interface TurnOptions {
  model?: string | null;
  effort?: string | null;
}

export interface SendRequest {
  threadId: string;
  text: string;
  tray: string;
  acceptsImages?: boolean;
  options?: TurnOptions;
}

export interface HistoryQuery {
  search?: string | null;
  archived?: boolean;
  cursor?: string | null;
  limit?: number | null;
}

export interface ConversationSummary {
  id: string;
  title: string;
  preview: string;
  updatedAt: number;
  pinned: boolean;
  archived: boolean;
}

export interface HistoryPage {
  items: ConversationSummary[];
  nextCursor: string | null;
}

export interface TranscriptMessage {
  role: "user" | "assistant";
  text: string;
}

export interface ModelInfo {
  id: string;
  displayName: string;
  efforts: string[];
  defaultEffort: string | null;
  inputModalities: string[];
  isDefault: boolean;
}

export type ApprovalDecision =
  | { type: "accept" }
  | { type: "acceptForSession" }
  | { type: "decline" }
  | { type: "cancel" }
  | { type: "answer"; content: unknown }
  | { type: "grant"; permissions: unknown; session: boolean };

export type ToolKind = "command" | "fileChange" | "mcp" | "dynamic" | "webSearch" | "imageView" | "collab" | "other";
export type ItemStatus = "inProgress" | "completed" | "failed" | "declined";
export type TurnStatus = "completed" | "interrupted" | "failed";

export interface FileChangeSummary {
  path: string;
  added: number;
  removed: number;
  diff: string;
}

export interface PlanStep {
  step: string;
  status: "pending" | "inProgress" | "completed" | string;
}

export type TurnError =
  | { kind: "planUsageLimit" }
  | { kind: "planNotEligible" }
  | { kind: "unsupportedCapability"; param: string | null }
  | { kind: "usageLimit"; retryAfterSecs: number | null }
  | { kind: "noConnection" }
  | { kind: "sessionExpired" }
  | { kind: "contextTooLong" }
  | { kind: "appServerCrashed" }
  | { kind: "other"; code: string; message: string };

export type AppServerState =
  | { state: "stopped" }
  | { state: "downloading"; bytes: number; total: number | null }
  | { state: "starting" }
  | { state: "ready"; version: string }
  | { state: "restarting"; attempt: number }
  | { state: "failed"; reason: string };

export type ConversationEvent =
  | { type: "turnStarted"; threadId: string; turnId: string }
  | { type: "messageDelta"; threadId: string; itemId: string; delta: string }
  | { type: "messageCompleted"; threadId: string; itemId: string; text: string }
  | { type: "reasoningDelta"; threadId: string; itemId: string; delta: string }
  | {
      type: "toolCall";
      threadId: string;
      itemId: string;
      kind: ToolKind;
      title: string;
      status: ItemStatus;
      detail: string | null;
    }
  | { type: "fileChanges"; threadId: string; itemId: string; changes: FileChangeSummary[]; status: ItemStatus }
  | {
      type: "approvalRequested";
      threadId: string;
      requestId: string;
      kind: "command" | "fileChange" | "permissions";
      command: string | null;
      cwd: string | null;
      reason: string | null;
      changes: FileChangeSummary[];
      options: string[];
    }
  | {
      type: "userInputRequested";
      threadId: string | null;
      requestId: string;
      prompt: unknown;
      autoResolveMs: number | null;
      source: string;
    }
  | { type: "requestResolved"; requestId: string }
  | { type: "planUpdated"; threadId: string; turnId: string; explanation: string | null; steps: PlanStep[] }
  | { type: "planProposed"; threadId: string; itemId: string; text: string }
  | { type: "diffUpdated"; threadId: string; turnId: string; diff: string }
  | { type: "tokenUsage"; threadId: string; used: number; window: number | null }
  | { type: "compacted"; threadId: string }
  | { type: "turnCompleted"; threadId: string; turnId: string; status: TurnStatus; error: TurnError | null }
  | { type: "appServerState"; state: AppServerState };

// ------------------------------------------------------------------- context

export type ChipKind = "screen" | "region" | "selection" | "audio" | "clip" | "file" | "image" | "skill";

export type ChipPayload =
  | { type: "image"; path: string }
  | { type: "images"; paths: string[]; caption: string | null }
  | { type: "text"; text: string }
  | { type: "skill"; name: string; path: string }
  | { type: "mixed"; parts: ChipPayload[] };

export interface ContextChip {
  id: string;
  kind: ChipKind;
  label: string;
  previewPath: string | null;
  payload: ChipPayload;
  tokenEstimate: number;
  blockedReason: string | null;
}

export interface AttachmentInfo {
  id: string;
  conversation: string;
  fileName: string;
  stored: string;
  hash: string;
  summary: string;
  kind: string;
  tokens: number;
  warnings: string[];
}

// ---------------------------------------------------------------- extensions

export interface QuickCommand {
  name: string;
  template: string;
  builtin: boolean;
  enabled: boolean;
}

export interface Expansion {
  promptText: string;
  display: string;
  needsScreen: boolean;
}

export type EnvValue = { kind: "plain"; value: string } | { kind: "secret" };

export type McpTransport =
  | { type: "stdio"; command: string; args: string[]; env: Record<string, EnvValue>; cwd: string | null }
  | { type: "http"; url: string; bearerSecret: boolean; headers: Record<string, string> };

export type McpApprovalMode = "alwaysAsk" | "askForWrites" | "auto";

export interface McpServerSpec {
  name: string;
  transport: McpTransport;
  enabled: boolean;
  disabledTools: string[];
  approvalMode: McpApprovalMode;
  startupTimeoutSec: number | null;
  toolTimeoutSec: number | null;
}

export interface DetectedServer {
  app: "claudeDesktop" | "cursor" | "vsCode" | "codexCli";
  source: string;
  spec: McpServerSpec;
  secretNames: string[];
  warnings: string[];
}

export interface SkillReview {
  manifest: { name: string; description: string; extra: Record<string, string> };
  skillMd: string;
  files: { path: string; bytes: number; isScript: boolean }[];
  warning: string;
}

// --------------------------------------------------------------------- voice

export interface ModelEntry {
  id: string;
  name: string;
  family: string;
  engine: string;
  description: string;
  languages: string[];
  speed: number;
  accuracy: number;
  streaming: boolean;
  minRamMb: number;
  gpuRecommended: boolean;
  license: string;
  sourceUrl: string;
}

export interface ModelView {
  entry: ModelEntry;
  sizeBytes: number;
  installed: boolean;
  selected: boolean;
  recommended: boolean;
  downloading: boolean;
}

export type PttState =
  | { state: "idle" }
  | { state: "listening" }
  | { state: "partial"; text: string }
  | { state: "transcribing" }
  | { state: "done"; text: string }
  | { state: "empty" }
  | { state: "cancelled" }
  | { state: "failed"; error: string };

export interface AudioDevice {
  id: string;
  name: string;
  isDefault: boolean;
}

// --------------------------------------------------------------- diagnostics

export interface Diagnostics {
  version: string;
  os: string;
  dataDir: string;
  gatewayPort: number;
  appServer: AppServerState;
  appServerLaunches: number;
  providers: number;
  mcpServers: number;
  pendingConsents: number;
}

export interface AppProfile {
  id: string;
  name: string;
  processPattern: string;
  titleGlob: string | null;
  instructions: string;
  attachScreen: boolean;
  defaultMode: "chat" | "task" | "plan" | null;
  defaultModel: string | null;
}

export interface McpStatus {
  name: string;
  tools: string[];
  auth: string | null;
}

export interface WorkspaceFile {
  path: string;
  absolute: string;
  bytes: number;
  modified: number;
}

export interface HostError {
  code: string;
  message: string;
}

export interface PreviousApp {
  window: number;
  pid: number;
  processName: string;
  title: string;
  monitorId: string;
}

export type OverlayMode = "compact" | "expanded";

// -------------------------------------------------------------------- events

export type HostEvent =
  | { channel: "conversation"; event: ConversationEvent }
  | { channel: "login"; event: LoginProgress }
  | { channel: "consent"; event: ConsentRequest }
  | { channel: "consentResolved"; event: { id: string } }
  | { channel: "privacy"; event: PrivacyState }
  | { channel: "download"; event: { id: string; bytes: number; total: number | null; done: boolean; error: string | null } }
  | { channel: "voice"; event: PttState }
  | { channel: "notice"; event: { level: "info" | "warning" | "error"; message: string } }
  | { channel: "settings"; event: Settings };
