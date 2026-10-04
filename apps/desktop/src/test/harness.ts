// Fresh mock backend + stores for each UI test.
import { setBridge } from "../ipc/bridge";
import { createMockBridge, type MockOptions } from "../ipc/mock";
import { bootstrap, resetBootstrap, useApp } from "../state/app";
import { useConversation } from "../state/conversation";
import { CHATGPT_PLAN, useSession } from "../overlay/session";
import { stopSpeaking, useSpeech } from "../lib/speech";

export async function freshApp(opts: MockOptions = {}) {
  const bridge = createMockBridge({ tick: 0, ...opts });
  setBridge(bridge);
  resetBootstrap();
  useSession.setState({ provider: CHATGPT_PLAN, model: null, effort: null });
  useApp.setState({ catalogRevision: 0, voiceResultRevision: 0 });
  useConversation.getState().reset();
  stopSpeaking();
  useSpeech.setState({ pendingConsent: null });
  useApp.setState({ ready: false, settings: null, privacy: null, auth: null, login: null, welcome: false, consents: [], downloads: {}, voice: { state: "idle" }, notices: [], conversationRequest: null });
  useSession.setState({ threadId: null, ephemeral: false, mode: "chat", granted: [], chips: [], overlayMode: "compact", historyOpen: false, workOpen: false, minibar: false, sending: false, models: [], providers: [], quickNames: [], profile: null });
  await bootstrap();
  return bridge;
}
