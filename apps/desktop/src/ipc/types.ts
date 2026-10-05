// Types mirroring the Rust serde shapes crossing the Tauri IPC boundary.
// Contract: `crates/aura-app/tests/ipc_contract.rs` writes golden JSON to
// `src/ipc/__fixtures__/` and `src/ipc/contract.test.ts` checks these types
// and the store reducers against it. Change both sides together.

// ------------------------------------------------------------------ settings

export type Theme = "system" | "light" | "dark";
export type Language = "ptBr" | "en";

export interface Settings {
  theme: Theme;
  opacity: number;
  language: Language;
  invokeShortcut: string;
  doubleTapCtrl: boolean;
  startWithWindows: boolean;
  /** Hide when another window takes focus (default off: stays open). */
  hideOnBlur: boolean;
  privacyPauseShortcut: string;
  pushToTalkShortcut: string;
  globalVoiceShortcut: string;
  attachScreenOnOpen: boolean;
  sendAfterDictation: boolean;
  microphoneDeviceId: string | null;
  systemAudioDeviceId: string | null;
  /** SKILL.md paths turned off (managed by the skills commands). */
  disabledSkills: string[];
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
  /** Offline Windows voice (null = by UI language). */
  ttsVoice: string | null;
  /** BYOK provider used as a cloud voice (null = offline). */
  ttsProvider: string | null;
  ttsCloudVoice: string;
  /** Providers the user agreed to send answers to; set by `speech_consent`. */
  ttsCloudConsent: string[];
  autoRead: boolean;
  /** Accent color `#rrggbb` (null = Aura's default). */
  accentColor: string | null;
  /** Reasoning effort per `provider::model` and mode (013). */
  effortPresets: Record<string, ModeEfforts>;
  /** YOLO (018): Task mode never asks for permission; set by `yolo_set`. */
  yolo: boolean;
  /** Hide every Aura window from screenshots, recordings and screen sharing. */
  hideFromCapture: boolean;
  /** Keep a Meeting's audio after it ends (default: only the text stays). */
  meetingKeepAudio: boolean;
  /** Mask CPF, cards, e-mails and phones before the model reads a Meeting. */
  meetingRedactPii: boolean;
}

export interface ModeEfforts {
  chat?: string | null;
  task?: string | null;
  plan?: string | null;
}

export interface SpeechVoice {
  id: string;
  name: string;
  language: string;
}

export interface SpeechOptions {
  voices: SpeechVoice[];
  cloud: { id: string; name: string }[];
}

export type SettingsPatch = Partial<Settings>;

// ------------------------------------------------------------------- privacy

/** Meeting (023): started by the user, never automatically. */
export interface Meeting {
  id: string;
  title: string;
  kind: string;
  briefing: string;
  origin: "live" | "buffer";
  status: "active" | "ended" | "interrupted";
  startedAt: number;
  endedAt: number | null;
}

export interface Utterance {
  id: number;
  meetingId: string;
  /** Milliseconds since the meeting started. */
  t0: number;
  t1: number;
  /** `note` = written by the user during the meeting. */
  speaker: "you" | "them" | "note";
  text: string;
}

export interface MeetingHit {
  meetingId: string;
  title: string;
  startedAt: number;
  t0: number;
  speaker: "you" | "them";
  text: string;
}

export interface Recipe {
  id: string;
  name: string;
  description: string;
  notesTemplate: string;
  helpLevel: "silent" | "onDemand" | "balanced" | "active";
  builtin: boolean;
}

export interface HidingStatus {
  window: string;
  hidden: boolean;
}

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
  /** Applied whenever a segment is written (004 AC-016). */
  retention: { days: number; maxGb: number; applyToManual: boolean };
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
  /** Recorded span, once it has ended. */
  durationMs: number | null;
}

/** Last minutes of the recent buffer to attach (004 AC-014, 005 AC-007/009). */
export interface RecentClip {
  minutes: number;
  screen: boolean;
  audio: "mic" | "system" | "both" | null;
}

/** A playable file of a recording (decrypted copy in the session cache). */
export interface PlaybackMedia {
  kind: "audio" | "video";
  /** `mic`, `system` or `screen`. */
  source: string;
  mime: string;
  path: string;
}

export interface RecordingAttach {
  chips: ContextChip[];
  failed: string[];
}

export interface AccessLogEntry {
  id: number;
  at: number;
  source: string;
  requester: string;
  tool: string | null;
  conversation: string | null;
  decision: string;
  /** Stable code: `covered:N`, `consent`, `user`, `timeout`, a deny reason… */
  reason: string | null;
  hasThumbnail: boolean;
  /** Thread of the agent conversation, when it still exists. */
  threadId: string | null;
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
  /** Entered or corrected by the user; discovery keeps it (absent = false). */
  manual?: boolean;
  /** Reasoning efforts the model accepts (013); empty = low/medium/high. */
  efforts?: string[];
  defaultEffort?: string | null;
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
  attachmentLabel?: {
    fileName: string;
    parts: ({ type: "count"; amount: number; unit: "line" | "word" | "page" | "sheet" | "slide" } | { type: "image" })[];
  };
  previewPath: string | null;
  payload: ChipPayload;
  tokenEstimate: number;
  blockedReason: string | null;
  /** Folder of files kept with the chip (Clip frames/audio). */
  filesDir?: string;
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

export interface MemoryView {
  /** memory_summary.md: given to new conversations. */
  summary: string;
  /** MEMORY.md: registry the agent searches. */
  registry: string;
  facts: string[];
}

export interface SkillEntry {
  name: string;
  description: string;
  path: string;
  origin: "aura" | "user" | "system";
  enabled: boolean;
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
  gateway: { port: number; reachable: boolean };
  /** MCP servers as the running app-server sees them. */
  mcp: { name: string; tools: number; error: string | null }[];
  worker: { installed: boolean; running: boolean; spawns: number; gpu: boolean; model: string | null };
  capture: { paused: boolean; active: string[]; recording: boolean };
  /** Active ChatGPT account (e-mail masked; never tokens). */
  account: { email: string | null; signedIn: boolean; planUsageEnabled: boolean } | null;
  disk: { freeBytes: number | null; auraBytes: number };
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
  /** Why the server did not start; null when connected. */
  error: string | null;
}

export interface McpDiagnosis {
  connected: boolean;
  exitCode: number | null;
  log: string[];
}

export interface PdfPreview {
  totalPages: number;
  /** First pages that have a text layer. */
  pages: { number: number; text: string }[];
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

export type AudioSourceKind = "mic" | "systemAudio";

export type HostEvent =
  | { channel: "providersChanged"; event: Record<string, never> }
  | { channel: "audioLevel"; event: { source: AudioSourceKind; dbfs: number } }
  | { channel: "conversation"; event: ConversationEvent }
  | { channel: "login"; event: LoginProgress }
  | { channel: "consent"; event: ConsentRequest }
  | { channel: "consentResolved"; event: { id: string } }
  | { channel: "privacy"; event: PrivacyState }
  | { channel: "download"; event: { id: string; bytes: number; total: number | null; done: boolean; error: string | null } }
  | { channel: "voice"; event: PttState }
  | { channel: "notice"; event: { level: "info" | "warning" | "error"; message: string } }
  | { channel: "openConversation"; event: { threadId: string } }
  | { channel: "agentTask"; event: { text: string; mode: ConversationMode["mode"] } }
  | { channel: "extensionsChanged"; event: Record<string, never> }
  | { channel: "reminder"; event: { id: string; text: string } }
  | { channel: "meeting"; event: { id: string; status: "active" | "updated" | "ended" | "briefing" } }
  | { channel: "settings"; event: Settings };
