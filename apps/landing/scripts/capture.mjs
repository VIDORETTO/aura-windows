// Captures the real Aura UI (desktop preview + demo backend) for the landing page.
// Run `pnpm -C apps/desktop dev --port 1421` first, then `pnpm -C apps/landing capture`.
// Only synthetic data is injected; output goes to src/assets/screens (PNG, 2x).
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const here = path.dirname(fileURLToPath(import.meta.url));
const OUT = path.resolve(here, "../src/assets/screens");
const BASE = process.env.AURA_PREVIEW_URL || "http://127.0.0.1:1421/";

async function seed(page) {
  await page.evaluate(async () => {
    const { getBridge } = await import("/src/ipc/bridge.ts");
    const { useApp, bootstrap } = await import("/src/state/app.ts");
    await bootstrap();
    const b = await getBridge();
    const s = b.state;
    const now = Date.parse("2026-10-13T14:00:00-03:00");
    s.account = { clientId: "presentation", subject: "demo", email: "voce@exemplo.com", scopes: ["openid"], planUsageEnabled: true, expiresAt: null, active: true, welcomed: true, signedIn: true };
    s.settings = { ...s.settings, theme: "dark", opacity: 1, defaultModel: "gpt-6.1-sol", memories: true, asrLanguage: "pt", effortPresets: { "chatgpt-plan": { "gpt-6.1-sol:chat": "medium" } } };
    const model = (id, name, images = true) => ({ id, displayName: name, contextWindow: 128000, maxOutput: 16000, supportsImages: images, supportsTools: true, supportsReasoning: false, estimated: true });
    const quirks = { noParallelToolCalls: false, reasoningEffort: false, noStreamOptions: false };
    s.providers = [
      { id: "demo-openai", name: "OpenAI", preset: "openai", wire: "responses", baseUrl: "https://api.openai.com/v1", auth: "bearer", extraHeaders: {}, models: [model("modelo-exemplo", "Modelo de demonstração")], credentialHint: "••••demo", status: "verified", lastError: null, quirks },
      { id: "demo-local", name: "Ollama · local", preset: "ollama", wire: "chat", baseUrl: "http://127.0.0.1:11434/v1", auth: "bearer", extraHeaders: {}, models: [model("modelo-local", "Modelo local", false)], credentialHint: null, status: "verified", lastError: null, quirks },
    ];
    s.skills = [["aura-estudo", "Estude com explicações, exemplos e perguntas de revisão."], ["aura-documentos", "Entenda documentos e organize os pontos principais."], ["aura-preparo", "Prepare reuniões, entrevistas e apresentações."]].map(([name, description]) => ({ manifest: { name, description, extra: {} }, skillMd: description, files: [{ path: "SKILL.md", bytes: 240, isScript: false }], warning: "" }));
    s.mcp = [{ name: "documentos-demo", transport: { type: "stdio", command: "servidor-mcp-demo", args: [], env: {}, cwd: null }, enabled: true, disabledTools: [], approvalMode: "askForWrites", startupTimeoutSec: null, toolTimeoutSec: null }];
    s.history = [["h1", "Resumo da reunião do projeto", "Prioridades, decisões e próximos passos."], ["h2", "Novidades da WCAG 2.2", "Critérios novos, com fontes do W3C."], ["h3", "Entender este trecho de código", "Explicação do fluxo e possíveis melhorias."], ["h4", "Preparar a apresentação", "Estrutura, exemplos e pontos de discussão."]].map(([id, title, preview], i) => ({ id, title, preview, updatedAt: now / 1000 - i * 3600, pinned: i === 0, archived: false }));
    s.privacy.screen = { mode: { type: "recentBuffer", minutes: 10 }, agent: "ask" };
    s.accessLog = [
      { id: 1, at: now / 1000 - 480, source: "screen", requester: "agent", tool: "screen_capture", conversation: null, decision: "allow", reason: "user_request", hasThumbnail: false, threadId: null },
      { id: 2, at: now / 1000 - 240, source: "screen", requester: "agent", tool: "screen_text", conversation: null, decision: "deny", reason: "excluded_window", hasThumbnail: false, threadId: null },
    ];
    s.meetingBrief = "Alinhar prioridades do projeto, revisar o escopo e definir os próximos passos.";
    s.projectList = [{ id: "p1", name: "Lançamento", instructions: "Lançamento do Aura." }];
    s.meetingList = [{ id: "meeting-demo", title: "Alinhamento do projeto", kind: "work", briefing: s.meetingBrief, origin: "live", status: "ended", startedAt: now - 900000, endedAt: now - 300000, projectId: "p1" }];
    s.meetingLines["meeting-demo"] = [
      { id: 1, meetingId: "meeting-demo", t0: 0, t1: 19000, speaker: "you", text: "Quero mostrar o app pelo que ele permite fazer no dia a dia." },
      { id: 2, meetingId: "meeting-demo", t0: 22000, t1: 46000, speaker: "them", text: "Vamos começar pela conversa e depois mostrar contexto, voz e tarefas." },
      { id: 3, meetingId: "meeting-demo", t0: 48000, t1: 60000, speaker: "note", text: "Prioridade: exemplos claros e imagens de cada área." },
      { id: 4, meetingId: "meeting-demo", t0: 65000, t1: 88000, speaker: "you", text: "Também quero mostrar os controles de privacidade." },
    ];
    s.actionList = [{ id: "a1", meetingId: "meeting-demo", text: "Revisar o material de lançamento", owner: "you", due: "Amanhã", status: "open", t0: 65000, createdAt: now, overdue: false }];
    const original = b.invoke.bind(b);
    b.invoke = async (cmd, args) => {
      if (cmd === "workspace_files") return [{ path: "resumo-do-projeto.md", absolute: "C:/demo/resumo-do-projeto.md", bytes: 1840, modified: now / 1000 }, { path: "proximos-passos.md", absolute: "C:/demo/proximos-passos.md", bytes: 920, modified: now / 1000 }];
      if (cmd === "workspace_read") return "# Resumo do projeto\n\n## Objetivo\nApresentar o Aura no fluxo de trabalho do Windows.\n\n## Próximos passos\n- Revisar a página de lançamento\n- Selecionar as imagens\n- Organizar exemplos de contexto e voz";
      return original(cmd, args);
    };
    useApp.getState().setSettings(s.settings);
    useApp.setState({ auth: { accounts: [s.account], active: s.account }, privacy: s.privacy, appServer: { state: "ready", version: "demonstração" }, ready: true, welcome: false, notices: [] });
    window.presentationBridge = b;
    window.useAppStore = useApp;
  });
  await page.waitForTimeout(220);
}

const settled = async (page) => {
  await page.evaluate(() => document.fonts.ready);
  await page.waitForTimeout(220);
};
const shot = (page, name, opts = {}) => page.screenshot({ path: path.join(OUT, `${name}.png`), ...opts });
async function view(page, name, width, height) {
  await page.setViewportSize({ width, height });
  await settled(page);
  await shot(page, name);
  console.log(`captured ${name} (${width}x${height})`);
}
async function shell(page, name, width) {
  await page.setViewportSize({ width, height: 440 });
  await settled(page);
  const box = await page.locator(".overlay-shell").boundingBox();
  await shot(page, name, { clip: { x: 0, y: 0, width, height: Math.ceil(box.height) } });
  console.log(`captured ${name} (${width}x${Math.ceil(box.height)})`);
}

const chat = [
  { type: "user", id: "u1", text: "Me ajude a organizar o lançamento do meu projeto.", chips: [] },
  { type: "assistant", id: "m1", streaming: false, text: "Podemos organizar o lançamento em **três partes**:\n\n1. **O problema:** por que o projeto existe.\n2. **O produto em ação:** telas com exemplos do dia a dia.\n3. **Os cuidados:** contexto, voz e privacidade.\n\nComece com uma visão geral e depois mostre cada recurso com uma imagem." },
];
const web = [
  { type: "user", id: "w0", text: "Quais são as novidades da WCAG 2.2? Cite as fontes.", chips: [] },
  { type: "tool", item: { id: "t1", kind: "webSearch", title: "Busca na internet", status: "completed", detail: "WCAG 2.2 novos critérios", startedAt: 0, finishedAt: 1400 } },
  { type: "tool", item: { id: "t2", kind: "webSearch", title: "Leitura de página", status: "completed", detail: "https://www.w3.org/WAI/standards-guidelines/wcag/new-in-22/", startedAt: 0, finishedAt: 2100 } },
  {
    type: "assistant", id: "w1", streaming: false,
    text: "A WCAG 2.2 acrescenta **9 critérios** ao que já existia na 2.1 [[aura-source:W1]]. Os principais:\n\n- **Foco não encoberto:** o item com foco não pode ficar escondido atrás de banners ou barras fixas.\n- **Alvo mínimo:** áreas de clique com pelo menos 24 × 24 px.\n- **Autenticação acessível:** login sem exigir que a pessoa memorize ou transcreva códigos [[aura-source:W2]].\n\nO critério 4.1.1 (Análise) foi removido por estar obsoleto [[aura-source:W1]].",
    sources: [
      { sourceId: "W1", title: "What's New in WCAG 2.2 | W3C WAI", url: "https://www.w3.org/WAI/standards-guidelines/wcag/new-in-22/", snippet: "", publishedAt: null, retrievedAt: "2026-10-13T17:00:00Z", kind: "pageContent" },
      { sourceId: "W2", title: "Web Content Accessibility Guidelines (WCAG) 2.2", url: "https://www.w3.org/TR/WCAG22/", snippet: "", publishedAt: null, retrievedAt: "2026-10-13T17:00:00Z", kind: "searchSnippet" },
    ],
  },
];

(async () => {
  await fs.mkdir(OUT, { recursive: true });
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({ viewport: { width: 1180, height: 850 }, deviceScaleFactor: 2, colorScheme: "dark" });

  // Settings pages: one section each.
  await page.goto(`${BASE}#/settings/privacy`);
  await page.waitForSelector("main section");
  await seed(page);
  for (const [id, sections] of [["privacy", [0, 2, 3]], ["extensions", [0]], ["voice", [0]]]) {
    await page.evaluate((id) => (location.hash = `#/settings/${id}`), id);
    await page.setViewportSize({ width: 1180, height: 1800 });
    await settled(page);
    await page.locator("main").evaluate((e) => (e.scrollTop = 0));
    for (const i of sections) {
      const section = page.locator("main section").nth(i);
      if ((await section.count()) === 0) continue;
      await section.screenshot({ path: path.join(OUT, `${id}-secao-${i + 1}.png`) });
    }
    console.log(`captured settings/${id}`);
  }

  // Overlay states.
  await page.goto(BASE);
  await page.reload();
  await page.waitForSelector(".overlay-shell");
  await seed(page);
  await page.evaluate(async (chat) => {
    const { useSession } = await import("/src/overlay/session.ts");
    const { useConversation, emptyThread } = await import("/src/state/conversation.ts");
    window.session = useSession;
    window.conversation = useConversation;
    window.emptyThread = emptyThread;
    window.put = (blocks, extra = {}) => useConversation.setState({ threads: { demo: { ...emptyThread("demo"), blocks, tokens: { used: 2400, window: 200000 }, lastStatus: "completed", ...extra } }, requests: {} });
    await useSession.getState().loadCatalog();
    useSession.setState({ overlayMode: "expanded", mode: "chat", model: "gpt-6.1-sol", effort: "medium", threadId: "demo", historyOpen: false, workOpen: false, meetingOpen: false, minibar: false });
    window.put(chat);
  }, chat);
  await view(page, "conversa", 820, 610);

  await page.evaluate((web) => window.put(web), web);
  await view(page, "pesquisa-web", 820, 640);

  await page.evaluate((chat) => window.put(chat), chat);
  await page.locator("textarea").first().fill("@");
  await view(page, "contexto", 820, 610);
  await page.locator("textarea").first().fill("");

  await page.locator('header button[aria-expanded]').first().click();
  await view(page, "modelo", 820, 610);
  await page.keyboard.press("Escape");

  await page.evaluate(() => {
    const blocks = [
      { type: "user", id: "u2", text: "Crie um arquivo com os próximos passos do projeto.", chips: [] },
      { type: "plan", id: "p1", text: "Vou organizar os próximos passos e salvar o resultado no workspace da conversa." },
      { type: "approval", approval: { requestId: "approval-demo", kind: "fileChange", command: null, cwd: null, reason: "Salvar os próximos passos no workspace da conversa.", changes: [{ path: "proximos-passos.md", kind: "add", diff: "+ # Próximos passos\n+ - Revisar a página\n+ - Selecionar as imagens" }], options: ["accept", "acceptForSession", "decline"] } },
    ];
    session.setState({ mode: "task" });
    conversation.setState({ threads: { demo: { ...emptyThread("demo"), blocks, running: true, turnId: "demo-turn", tokens: { used: 4200, window: 200000 } } }, requests: { "approval-demo": "demo" } });
  });
  await view(page, "tarefas", 820, 610);

  await page.evaluate(() => {
    session.setState({ mode: "task", workOpen: true });
    window.put([{ type: "assistant", id: "done", text: "Organizei o resumo do projeto e os próximos passos. Os arquivos estão no workspace da conversa.", streaming: false }]);
  });
  await page.setViewportSize({ width: 1180, height: 630 });
  await settled(page);
  await page.getByText("resumo-do-projeto.md", { exact: true }).first().click();
  await view(page, "arquivos", 1180, 630);

  await page.evaluate((chat) => { session.setState({ workOpen: false, historyOpen: true, mode: "chat" }); window.put(chat); }, chat);
  await view(page, "historico", 1180, 630);

  await page.evaluate(() => session.setState({ historyOpen: false, meetingOpen: true }));
  await page.setViewportSize({ width: 1180, height: 1800 });
  await settled(page);
  await page.evaluate(async () => {
    const b = window.presentationBridge;
    b.state.meetingNow = { ...b.state.meetingList[0], status: "active", startedAt: Date.now() - 125000, endedAt: null };
    window.useAppStore.setState((s) => ({ meetingRevision: s.meetingRevision + 1 }));
    window.put([
      { type: "user", id: "q", text: "Quais são os pontos principais até agora?", chips: [] },
      { type: "assistant", id: "a", streaming: false, text: "Até agora, vocês alinharam:\n\n- Começar pela **conversa no Overlay**.\n- Mostrar **contexto, voz e tarefas** com exemplos.\n- Destacar os controles de **privacidade**." },
    ]);
  });
  await view(page, "reuniao", 1180, 650);

  await page.evaluate((chat) => { session.setState({ meetingOpen: false, minibar: true }); conversation.setState({ threads: { demo: { ...emptyThread("demo"), blocks: chat, running: true } }, requests: {} }); }, chat);
  await shell(page, "minibar", 680);

  await page.evaluate(() => { session.setState({ minibar: false, overlayMode: "compact", threadId: null }); });

  await page.locator("textarea").first().fill("Explique este trecho");
  await shell(page, "overlay-pergunta", 820);
  await page.locator("textarea").first().fill("");

  await page.evaluate(() => window.useAppStore.getState().handle({ channel: "voice", event: { state: "partial", text: "Resuma este documento em três tópicos" } }));
  await shell(page, "voz", 820);

  await browser.close();
  console.log(`done: ${OUT}`);
})().catch((e) => {
  console.error(e);
  process.exit(1);
});
