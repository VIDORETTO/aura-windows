import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { freshApp } from "../test/harness";
import { setBridge } from "../ipc/bridge";
import { SettingsApp, pageFromHash } from "./SettingsApp";
import { useApp } from "../state/app";

describe("Settings", () => {
  it("returns fixed dictation language to automatic without changing the UI language", async () => {
    const bridge = await freshApp({ signedIn: true });
    window.location.hash = "#/settings/voice";
    const user = userEvent.setup();
    render(<SettingsApp />);
    const language = await screen.findByRole("combobox", { name: "Idioma do ditado" });
    await user.selectOptions(language, "es");
    await waitFor(() => expect(bridge.state.settings.asrLanguage).toBe("es"));
    await user.selectOptions(language, "");
    await waitFor(() => expect(bridge.state.settings.asrLanguage).toBeNull());
    expect(language).toHaveValue("");
    expect(screen.getByRole("option", { name: "Automático (detectar idioma)" })).toBeInTheDocument();
    expect(bridge.state.settings.language).toBe("ptBr");
  });

  beforeEach(() => {
    window.location.hash = "#/settings/general";
  });

  it("routes by hash", () => {
    expect(pageFromHash("#/settings/privacy")).toBe("privacy");
    expect(pageFromHash("#/settings/nope")).toBe("general");
    expect(pageFromHash("")).toBe("general");
  });

  it("saves a selected microphone, restores it on remount and returns to the OS default", async () => {
    const bridge = await freshApp({ signedIn: true });
    const originalInvoke = bridge.invoke;
    bridge.invoke = async <T,>(command: string, args?: Record<string, unknown>) => command === "audio_devices" && args?.system === false
      ? [{ id: "mic-a", name: "Microphone A", isDefault: true }, { id: "mic-b", name: "Microphone B", isDefault: false }] as T
      : originalInvoke<T>(command, args);
    window.location.hash = "#/settings/voice";
    const user = userEvent.setup();
    const view = render(<SettingsApp />);
    const microphones = await screen.findByRole("combobox", { name: "Microfone" });
    await screen.findByRole("option", { name: "Microphone B" });
    await user.selectOptions(microphones, "mic-b");
    await waitFor(() => expect(bridge.state.settings.microphoneDeviceId).toBe("mic-b"));
    expect(bridge.state.settings.asrLanguage).toBeNull();
    view.unmount();
    render(<SettingsApp />);
    const restored = await screen.findByRole("combobox", { name: "Microfone" });
    await screen.findByRole("option", { name: "Microphone B" });
    expect(restored).toHaveValue("mic-b");
    await user.selectOptions(restored, "");
    await waitFor(() => expect(bridge.state.settings.microphoneDeviceId).toBeNull());
    expect(restored).toHaveValue("");
  });

  it("English Diagnostics localizes the engine state", async () => {
    const bridge = await freshApp({ signedIn: true });
    await bridge.invoke("settings_update", { patch: { language: "en" } });
    window.location.hash = "#/settings/diagnostics";
    render(<SettingsApp />);
    expect(await screen.findByText("Ready (mock) · 1×")).toBeInTheDocument();
    expect(screen.queryByText(/pronto/)).not.toBeInTheDocument();
  });

  it("Diagnostics shows gateway, MCP, worker, captures, account and disk (QA-037)", async () => {
    const bridge = await freshApp({ signedIn: true });
    bridge.state.diagnosticsExtra = {
      gateway: { port: 51234, reachable: true },
      mcp: [{ name: "aura", tools: 2, error: null }, { name: "docs", tools: 0, error: "connection refused" }],
      worker: { installed: true, running: false, spawns: 3, gpu: false, model: "parakeet-tdt-0.6b-v3" },
      capture: { paused: false, active: ["screen", "mic"], recording: true },
      account: { email: "g***@example.com", signedIn: true, planUsageEnabled: true },
      disk: { freeBytes: 53687091200, auraBytes: 1610612736 },
    };
    window.location.hash = "#/settings/diagnostics";
    render(<SettingsApp />);
    expect(await screen.findByText("127.0.0.1:51234 · respondendo")).toBeInTheDocument();
    expect(screen.getByText("aura · 2 ferramentas")).toBeInTheDocument();
    expect(screen.getByText("docs · erro: connection refused")).toBeInTheDocument();
    expect(screen.getByText("Instalado · parado · CPU · parakeet-tdt-0.6b-v3 · 3 inícios")).toBeInTheDocument();
    expect(screen.getByText("Tela, Microfone · gravação manual em andamento")).toBeInTheDocument();
    expect(screen.getByText("g***@example.com · conectada")).toBeInTheDocument();
    expect(screen.getByText("50.0 GB livres · Aura usa 1.5 GB")).toBeInTheDocument();
  });

  it("says updates are not configured while the build has no signing key (QA-038)", async () => {
    const bridge = await freshApp({ signedIn: true });
    const calls: string[] = [];
    const invoke = bridge.invoke;
    bridge.invoke = (async <R,>(cmd: string, args?: Record<string, unknown>) => {
      calls.push(cmd);
      return invoke<R>(cmd, args);
    }) as typeof bridge.invoke;
    window.location.hash = "#/settings/about";
    const user = userEvent.setup();
    render(<SettingsApp />);
    await user.click(await screen.findByRole("button", { name: "Verificar atualizações" }));
    expect(await screen.findByText("Atualizações automáticas não estão configuradas nesta compilação: falta a chave pública de assinatura.")).toBeInTheDocument();
    expect(calls).toContain("updater_configured");
  });

  it("resumes the first-run guide from Settings (QA-039)", async () => {
    const bridge = await freshApp({ signedIn: true });
    expect(bridge.state.settings.onboarded).toBe(true);
    const calls: string[] = [];
    const invoke = bridge.invoke;
    bridge.invoke = (async <R,>(cmd: string, args?: Record<string, unknown>) => {
      calls.push(cmd);
      return invoke<R>(cmd, args);
    }) as typeof bridge.invoke;
    window.location.hash = "#/settings/general";
    const user = userEvent.setup();
    render(<SettingsApp />);
    await user.click(await screen.findByRole("button", { name: "Retomar primeiros passos" }));
    await waitFor(() => expect(bridge.state.settings.onboarded).toBe(false));
    expect(calls).toContain("onboarding_resume");
  });

  it("chooses the accent color from presets or a custom hex and applies it (012)", async () => {
    const bridge = await freshApp({ signedIn: true });
    window.location.hash = "#/settings/general";
    const user = userEvent.setup();
    render(<SettingsApp />);
    const accent = () => document.documentElement.style.getPropertyValue("--accent");
    await user.click(await screen.findByRole("radio", { name: "Verde" }));
    await waitFor(() => expect(bridge.state.settings.accentColor).toBe("#2e8b3e"));
    expect(accent()).toBe("#2e8b3e");
    expect(screen.getByRole("radio", { name: "Verde" })).toBeChecked();

    const hex = screen.getByRole("textbox", { name: "Código hex da cor" });
    await user.clear(hex);
    await user.type(hex, "#e4572x{Enter}");
    expect(await screen.findByText("Use o formato #RRGGBB.")).toBeInTheDocument();
    expect(bridge.state.settings.accentColor).toBe("#2e8b3e");
    await user.clear(hex);
    await user.type(hex, "#E4572E{Enter}");
    await waitFor(() => expect(bridge.state.settings.accentColor).toBe("#e4572e"));
    expect(accent()).toBe("#e4572e");
    expect(screen.getByLabelText("Cor personalizada")).toHaveValue("#e4572e");

    await user.click(screen.getByRole("radio", { name: "Padrão" }));
    await waitFor(() => expect(bridge.state.settings.accentColor).toBeNull());
    expect(accent()).toBe("");
  });

  it("English MCP form localizes every field including secret fields", async () => {
    const bridge = await freshApp({ signedIn: true });
    await bridge.invoke("settings_update", { patch: { language: "en" } });
    window.location.hash = "#/settings/extensions";
    const user = userEvent.setup();
    render(<SettingsApp />);
    await user.click(await screen.findByRole("button", { name: "Add server" }));
    expect(screen.getByRole("textbox", { name: "Name" })).toBeInTheDocument();
    expect(screen.getByRole("combobox", { name: "Transport" })).toBeInTheDocument();
    expect(screen.getByRole("textbox", { name: "Secret variable (optional)" })).toBeInTheDocument();
    expect(screen.getByLabelText("Secret value")).toBeInTheDocument();
    await user.selectOptions(screen.getByRole("combobox", { name: "Transport" }), "http");
    expect(screen.getByLabelText("Token (optional)")).toBeInTheDocument();
  });

  it("saves MCP arguments with quoted paths, empty values and quotes as separate arguments", async () => {
    const bridge = await freshApp({ signedIn: true });
    const saved: any[] = [];
    const invoke = bridge.invoke;
    bridge.invoke = (async <R,>(cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "mcp_save") saved.push(args);
      return invoke<R>(cmd, args);
    }) as typeof bridge.invoke;
    window.location.hash = "#/settings/extensions";
    const user = userEvent.setup();
    render(<SettingsApp />);
    await user.click(await screen.findByRole("button", { name: "Adicionar servidor" }));
    await user.type(screen.getByRole("textbox", { name: "Nome" }), "qa");
    await user.type(screen.getByRole("textbox", { name: "Comando" }), "node");
    const args = screen.getByRole("textbox", { name: /^Argumentos/ });
    await user.type(args, '"C:\\QA Folder\\server.js" --label "QA label" --empty "');
    expect(screen.getByRole("alert")).toHaveTextContent("Aspas sem fechamento");
    expect(screen.getByRole("button", { name: "Salvar" })).toBeDisabled();
    await user.type(args, '"');
    expect(screen.getByRole("list", { name: "Argumentos interpretados" })).toHaveTextContent("C:\\QA Folder\\server.js--labelQA label--empty(vazio)");
    await user.click(screen.getByRole("button", { name: "Salvar" }));
    await waitFor(() => expect(saved).toHaveLength(1));
    expect(saved[0].spec.transport.args).toEqual(["C:\\QA Folder\\server.js", "--label", "QA label", "--empty", ""]);
    expect(await screen.findByText('node "C:\\QA Folder\\server.js" --label "QA label" --empty ""')).toBeInTheDocument();
  });

  it("shows MCP servers as connected or failed and the last log lines of a failure", async () => {
    const bridge = await freshApp({ signedIn: true });
    const spec = (name: string, command: string) => ({ name, transport: { type: "stdio", command, args: [], env: {}, cwd: null }, enabled: true, disabledTools: [], approvalMode: "askForWrites", startupTimeoutSec: null, toolTimeoutSec: null });
    await bridge.invoke("mcp_save", { spec: spec("qa-ok", "node"), secrets: [], bearer: null });
    await bridge.invoke("mcp_save", { spec: spec("qa-fail", "fail"), secrets: [], bearer: null });
    window.location.hash = "#/settings/extensions";
    const user = userEvent.setup();
    render(<SettingsApp />);
    const ok = await screen.findByRole("status", { name: "Estado de qa-ok" });
    expect(ok).toHaveTextContent("Conectado");
    const fail = screen.getByRole("status", { name: "Estado de qa-fail" });
    expect(fail).toHaveTextContent("Erro: MCP startup failed: connection closed: initialize response");
    await user.click(screen.getByRole("button", { name: "Ver log de qa-fail" }));
    const log = await screen.findByRole("log", { name: "Log de qa-fail" });
    expect(log).toHaveTextContent("Saiu com código 3");
    expect(log).toHaveTextContent("starting");
    expect(log).toHaveTextContent("missing QA_TOKEN");
    expect(screen.queryByRole("button", { name: "Ver log de qa-ok" })).toBeNull();
  });

  it("turns off one MCP tool for new conversations and can turn it back on", async () => {
    const bridge = await freshApp({ signedIn: true });
    await bridge.invoke("mcp_save", { spec: { name: "qa", transport: { type: "stdio", command: "node", args: [], env: {}, cwd: null }, enabled: true, disabledTools: [], approvalMode: "askForWrites", startupTimeoutSec: null, toolTimeoutSec: null }, secrets: [], bearer: null });
    window.location.hash = "#/settings/extensions";
    const user = userEvent.setup();
    render(<SettingsApp />);
    await user.click(await screen.findByRole("button", { name: "Ferramentas de qa" }));
    const write = await screen.findByRole("checkbox", { name: "qa_write" });
    expect(write).toBeChecked();
    await user.click(write);
    await waitFor(() => expect(bridge.state.mcp[0].disabledTools).toEqual(["qa_write"]));
    // The server no longer reports it, but it stays listed (off) so it can return.
    expect(await screen.findByRole("checkbox", { name: "qa_write" })).not.toBeChecked();
    expect(screen.getByRole("checkbox", { name: "qa_echo" })).toBeChecked();
    expect(screen.getByText("Vale para conversas novas.")).toBeInTheDocument();
    await user.click(screen.getByRole("checkbox", { name: "qa_write" }));
    await waitFor(() => expect(bridge.state.mcp[0].disabledTools).toEqual([]));
  });

  it("chooses the recent-buffer length between 1 and 30 minutes", async () => {
    const bridge = await freshApp({ signedIn: true });
    const calls: any[] = [];
    const invoke = bridge.invoke;
    bridge.invoke = (async <R,>(cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "privacy_set_source") calls.push(args);
      return invoke<R>(cmd, args);
    }) as typeof bridge.invoke;
    window.location.hash = "#/settings/privacy";
    const user = userEvent.setup();
    render(<SettingsApp />);
    const modes = await screen.findAllByRole("combobox", { name: "Captura" });
    await user.selectOptions(modes[0], "recentBuffer");
    const minutes = await screen.findByRole("spinbutton", { name: "Minutos do buffer de Tela" });
    expect(minutes).toHaveAttribute("min", "1");
    expect(minutes).toHaveAttribute("max", "30");
    await user.clear(minutes);
    await user.type(minutes, "25");
    await user.tab();
    await waitFor(() => expect(calls.at(-1)).toMatchObject({ source: "screen", mode: { type: "recentBuffer", minutes: 25 } }));
    await user.clear(minutes);
    await user.type(minutes, "45");
    await user.tab();
    await waitFor(() => expect(calls.at(-1)).toMatchObject({ mode: { type: "recentBuffer", minutes: 30 } }));
    expect(minutes).toHaveValue(30);
  });

  it("configures retention in days and space and says when it applies", async () => {
    const bridge = await freshApp({ signedIn: true });
    window.location.hash = "#/settings/privacy";
    const user = userEvent.setup();
    render(<SettingsApp />);
    const days = await screen.findByRole("spinbutton", { name: "Manter gravações por (dias)" });
    expect(days).toHaveValue(7);
    const space = screen.getByRole("spinbutton", { name: "Espaço máximo (GB)" });
    expect(space).toHaveValue(20);
    expect(screen.getByText("Os limites valem para gravações contínuas e são aplicados a cada novo trecho gravado; os mais antigos saem primeiro.")).toBeInTheDocument();
    await user.clear(days);
    await user.type(days, "3");
    await user.clear(space);
    await user.type(space, "5");
    await user.click(screen.getByRole("switch", { name: "Aplicar também às gravações manuais" }));
    await user.click(screen.getByRole("button", { name: "Salvar retenção" }));
    await waitFor(() => expect(bridge.state.privacy.retention).toEqual({ days: 3, maxGb: 5, applyToManual: true }));
  });

  it("access log shows exact time, coverage, thumbnail and the conversation link (QA-031)", async () => {
    const bridge = await freshApp({ signedIn: true });
    bridge.state.accessLog = [
      { id: 9, at: 1791030896, source: "screen", requester: "agent", tool: "screen_capture", conversation: "uuid-9", decision: "allowRedacted", reason: "covered:2", hasThumbnail: true, threadId: "thr-9" },
      { id: 8, at: 1791030800, source: "mic", requester: "agent", tool: "audio_recent", conversation: "uuid-8", decision: "deny", reason: "timeout", hasThumbnail: false, threadId: null },
    ];
    const opened: unknown[] = [];
    const invoke = bridge.invoke;
    bridge.invoke = (async <R,>(cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "privacy_open_conversation") opened.push(args);
      return invoke<R>(cmd, args);
    }) as typeof bridge.invoke;
    window.location.hash = "#/settings/privacy";
    const user = userEvent.setup();
    render(<SettingsApp />);
    const first = await screen.findByRole("listitem", { name: /screen_capture/ });
    expect(within(first).getByText(/03\/10\/2026/)).toBeInTheDocument();
    expect(within(first).getByText("2 áreas cobertas")).toBeInTheDocument();
    expect(within(first).getByText("Tela")).toBeInTheDocument();
    await user.click(within(first).getByRole("button", { name: "Ver miniatura" }));
    expect(await within(first).findByRole("img", { name: "Miniatura do que o agente recebeu" })).toHaveAttribute(
      "src",
      "data:image/png;base64,iVBORw0KGgo=",
    );
    await user.click(within(first).getByRole("button", { name: "Abrir conversa" }));
    await waitFor(() => expect(opened).toEqual([{ threadId: "thr-9" }]));
    const second = screen.getByRole("listitem", { name: /audio_recent/ });
    expect(within(second).getByText("Sem resposta do usuário")).toBeInTheDocument();
    expect(within(second).queryByRole("button", { name: "Ver miniatura" })).toBeNull();
    expect(within(second).queryByRole("button", { name: "Abrir conversa" })).toBeNull();
  });

  it("recordings show duration, play in Aura and attach to the conversation (QA-032)", async () => {
    const bridge = await freshApp({ signedIn: true });
    bridge.state.recordings = [{ id: "rec_9", source: "screen,mic", title: "Reunião", startedAt: 1791030000, endedAt: 1791030061, bytes: 2_400_000, durationMs: 61_400 }];
    const attached: unknown[] = [];
    const invoke = bridge.invoke;
    bridge.invoke = (async <R,>(cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "recording_attach") attached.push(args);
      return invoke<R>(cmd, args);
    }) as typeof bridge.invoke;
    window.location.hash = "#/settings/privacy";
    const user = userEvent.setup();
    render(<SettingsApp />);
    const item = await screen.findByRole("listitem", { name: "Reunião" });
    expect(within(item).getByText(/1:01/)).toBeInTheDocument();
    await user.click(within(item).getByRole("button", { name: "Reproduzir" }));
    const audio = await within(item).findByLabelText("Microfone — Reunião");
    expect(audio.tagName).toBe("AUDIO");
    expect(audio).toHaveAttribute("src", expect.stringContaining("mic.wav"));
    expect(within(item).getByLabelText("Tela 1 — Reunião").tagName).toBe("VIDEO");
    await user.click(within(item).getByRole("button", { name: "Anexar à conversa" }));
    await waitFor(() => expect(attached).toEqual([{ id: "rec_9" }]));
    expect(await screen.findByText("Gravação anexada à conversa: 2 arquivo(s).")).toBeInTheDocument();
  });

  it("chooses the reading voice, a cloud voice and auto-read (QA-034)", async () => {
    const bridge = await freshApp({ signedIn: true });
    const p = await bridge.invoke<{ id: string }>("providers_save", { draft: { name: "Voz QA", preset: "custom", baseUrl: "http://127.0.0.1:9/v1" }, credential: null });
    window.location.hash = "#/settings/voice";
    const user = userEvent.setup();
    render(<SettingsApp />);
    const voice = await screen.findByRole("combobox", { name: "Voz de leitura" });
    await waitFor(() => expect(within(voice).getByRole("option", { name: "Microsoft Maria (pt-BR)" })).toBeInTheDocument());
    await user.selectOptions(voice, "win-maria");
    await waitFor(() => expect(bridge.state.settings).toMatchObject({ ttsVoice: "win-maria", ttsProvider: null }));
    await user.selectOptions(voice, `cloud:${p.id}`);
    await waitFor(() => expect(bridge.state.settings.ttsProvider).toBe(p.id));
    expect(await screen.findByText("O texto das respostas será enviado a Voz QA. O Aura pede confirmação no primeiro uso.")).toBeInTheDocument();
    expect(screen.getByRole("textbox", { name: "Voz na nuvem" })).toHaveValue("alloy");
    await user.click(screen.getByRole("switch", { name: "Ler respostas automaticamente" }));
    await waitFor(() => expect(bridge.state.settings.autoRead).toBe(true));
  });

  it("reviews, edits and forgets remembered facts", async () => {
    const bridge = await freshApp({ signedIn: true });
    bridge.state.memorySummary = "# Resumo\n- Prefere unidades métricas\n- Mora em Lisboa\n";
    bridge.state.memoryRegistry = "# Registro\n- Mora em Lisboa\n";
    window.location.hash = "#/settings/general";
    const user = userEvent.setup();
    render(<SettingsApp />);
    const list = await screen.findByRole("list", { name: "Fatos lembrados" });
    expect(within(list).getAllByRole("listitem").map((li) => li.firstChild?.textContent)).toEqual(["Prefere unidades métricas", "Mora em Lisboa"]);
    await user.click(within(list).getByRole("button", { name: "Esquecer: Mora em Lisboa" }));
    await waitFor(() => expect(within(list).queryByText("Mora em Lisboa")).toBeNull());
    expect(bridge.state.memoryRegistry).toBe("# Registro\n");

    await user.click(screen.getByRole("button", { name: "Editar memórias" }));
    const summary = screen.getByRole("textbox", { name: "Resumo (enviado às conversas novas)" });
    await user.clear(summary);
    await user.type(summary, "- Prefere unidades do SI");
    await user.click(screen.getByRole("button", { name: "Salvar memórias" }));
    expect(await within(screen.getByRole("list", { name: "Fatos lembrados" })).findByText("Prefere unidades do SI")).toBeInTheDocument();

    // Asked inside the app, never with the browser's confirm dialog.
    const confirm = vi.spyOn(window, "confirm");
    await user.click(screen.getByRole("button", { name: "Esquecer tudo" }));
    expect(screen.getByText("Apagar tudo o que o agente lembra?")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Cancelar" }));
    expect(bridge.state.memorySummary).not.toBe("");
    await user.click(screen.getByRole("button", { name: "Esquecer tudo" }));
    await user.click(screen.getByRole("button", { name: "Confirmar" }));
    expect(await screen.findByText("Nada lembrado ainda.")).toBeInTheDocument();
    expect(bridge.state.memorySummary).toBe("");
    expect(confirm).not.toHaveBeenCalled();
    confirm.mockRestore();
  });

  it("each Settings page opens at the top", async () => {
    await freshApp({ signedIn: true });
    window.location.hash = "#/settings/privacy";
    render(<SettingsApp />);
    const main = await screen.findByRole("main");
    main.scrollTop = 900;
    await act(async () => { window.location.hash = "#/settings/voice"; window.dispatchEvent(new HashChangeEvent("hashchange")); });
    await waitFor(() => expect(main.scrollTop).toBe(0));
  });

  it("quick commands show a readable preview, not raw placeholders", async () => {
    await freshApp({ signedIn: true });
    window.location.hash = "#/settings/extensions";
    render(<SettingsApp />);
    expect(await screen.findByText("Traduza para ‹inglês›:")).toBeInTheDocument();
    expect(screen.queryByText(/\{selecao\}/)).toBeNull();
  });

  it("erasing all data asks inside the app first", async () => {
    const bridge = await freshApp({ signedIn: true });
    const erased: string[] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd === "erase_all_data") { erased.push(cmd); return undefined as R; }
      return bridge.invoke<R>(cmd, args);
    } });
    const confirm = vi.spyOn(window, "confirm");
    window.location.hash = "#/settings/diagnostics";
    const user = userEvent.setup();
    render(<SettingsApp />);
    await user.click(await screen.findByRole("button", { name: "Apagar todos os meus dados" }));
    expect(erased).toEqual([]);
    expect(screen.getByText("Isso apaga conversas, chaves, gravações e configurações. Continuar?")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Confirmar" }));
    await waitFor(() => expect(erased).toEqual(["erase_all_data"]));
    expect(confirm).not.toHaveBeenCalled();
    confirm.mockRestore();
  });

  it("lists skills with origin, toggles them and edits an Aura skill", async () => {
    const bridge = await freshApp({ signedIn: true });
    await bridge.invoke("skills_create", { name: "revisar-contrato", description: "Revisa contratos", body: "Leia com calma." });
    window.location.hash = "#/settings/extensions";
    const user = userEvent.setup();
    render(<SettingsApp />);
    const aura = await screen.findByRole("listitem", { name: "revisar-contrato" });
    expect(aura).toHaveTextContent("Aura");
    const system = screen.getByRole("listitem", { name: "skill-creator" });
    expect(system).toHaveTextContent("Sistema");
    expect(within(system).queryByRole("button", { name: "Editar revisar-contrato" })).toBeNull();
    const toggle = within(aura).getByRole("switch", { name: "Ativar revisar-contrato" });
    expect(toggle).toHaveAttribute("aria-checked", "true");
    await user.click(toggle);
    await waitFor(() => expect(bridge.state.disabledSkills).toEqual(["aura/revisar-contrato/SKILL.md"]));
    expect(within(await screen.findByRole("listitem", { name: "revisar-contrato" })).getByRole("switch")).toHaveAttribute("aria-checked", "false");

    await user.click(within(screen.getByRole("listitem", { name: "revisar-contrato" })).getByRole("button", { name: "Editar revisar-contrato" }));
    const description = screen.getByRole("textbox", { name: "Descrição (quando usar)" });
    expect(description).toHaveValue("Revisa contratos");
    expect(screen.getByRole("textbox", { name: "Instruções" })).toHaveValue("Leia com calma.");
    await user.clear(description);
    await user.type(description, "Revisa contratos de aluguel");
    await user.click(screen.getByRole("button", { name: "Salvar" }));
    expect(await screen.findByText("Revisa contratos de aluguel")).toBeInTheDocument();
  });

  it("the existing quick-command catalog changes built-in prompts but preserves custom text", async () => {
    const bridge = await freshApp({ signedIn: true });
    await bridge.invoke("quick_save", { name: "qa-custom", template: "Texto pessoal em português", replace: false });
    window.location.hash = "#/settings/extensions";
    render(<SettingsApp />);
    await screen.findByText(/Resuma em até três frases/);
    await act(async () => { await bridge.invoke("settings_update", { patch: { language: "en" } }); });
    expect(await screen.findByText(/Summarize in up to three sentences/)).toBeInTheDocument();
    expect(screen.getByText("Texto pessoal em português")).toBeInTheDocument();
  });

  it("changes theme and opacity through the host", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<SettingsApp />);
    const theme = await screen.findByDisplayValue("Sistema");
    await user.selectOptions(theme, "dark");
    await waitFor(() => expect(useApp.getState().settings?.theme).toBe("dark"));
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
  });

  it("pauses privacy and manages exclusions", async () => {
    await freshApp({ signedIn: true });
    window.location.hash = "#/settings/privacy";
    const user = userEvent.setup();
    render(<SettingsApp />);
    await user.click(await screen.findByRole("switch", { name: "Pausar toda captura" }));
    await waitFor(() => expect(useApp.getState().privacy?.paused).toBe(true));
    await user.type(screen.getByPlaceholderText(/Título contém/), "*Banco*");
    await user.click(screen.getByRole("button", { name: "Adicionar" }));
    expect(await screen.findByText(/“\*Banco\*”/)).toBeInTheDocument();
  });

  it("manual recording needs a source in manual mode, then records", async () => {
    await freshApp({ signedIn: true });
    window.location.hash = "#/settings/privacy";
    const user = userEvent.setup();
    render(<SettingsApp />);
    await user.click(await screen.findByRole("button", { name: /Iniciar gravação/ }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Gravação manual");
    const modes = screen.getAllByRole("combobox", { name: "Captura" });
    await user.selectOptions(modes[1], "manual");
    await user.click(screen.getByRole("button", { name: /Iniciar gravação/ }));
    expect(await screen.findByRole("button", { name: /Parar gravação/ })).toBeInTheDocument();
    await waitFor(() => expect(useApp.getState().recording).toEqual(["screen", "mic"]));
    await user.click(screen.getByRole("button", { name: /Parar gravação/ }));
    expect(await screen.findByRole("button", { name: /Exportar/ })).toBeEnabled();
  });

  it("adds and tests a BYOK provider", async () => {
    await freshApp({ signedIn: true });
    window.location.hash = "#/settings/providers";
    const user = userEvent.setup();
    render(<SettingsApp />);
    await user.click(await screen.findByRole("button", { name: "Adicionar provedor" }));
    await user.selectOptions(screen.getByDisplayValue("OpenAI"), "groq");
    await user.type(screen.getByPlaceholderText("sk-…"), "gsk_abcd1234");
    await user.click(screen.getByRole("button", { name: "Salvar" }));
    const row = await screen.findByText("Groq");
    await waitFor(() => expect(within(row.parentElement!.parentElement!).getByText("Verificado")).toBeInTheDocument());
    expect(screen.getByText(/••••1234/)).toBeInTheDocument();
  });

  it("edits a saved provider's name and URL keeping its id and key, and replaces the key only when typed", async () => {
    const bridge = await freshApp({ signedIn: true });
    const saved = await bridge.invoke<{ id: string }>("providers_save", { draft: { name: "Groq", preset: "groq" }, credential: "gsk_old_1111" });
    const drafts: unknown[] = [];
    const invoke = bridge.invoke;
    bridge.invoke = (async <R,>(cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "providers_save") drafts.push(args);
      return invoke<R>(cmd, args);
    }) as typeof bridge.invoke;
    window.location.hash = "#/settings/providers";
    const user = userEvent.setup();
    render(<SettingsApp />);
    await user.click(await screen.findByRole("button", { name: "Editar Groq" }));
    const name = screen.getByRole("textbox", { name: "Nome" });
    expect(name).toHaveValue("Groq");
    await user.clear(name);
    await user.type(name, "Groq trabalho");
    const url = screen.getByRole("textbox", { name: "URL base" });
    await user.clear(url);
    await user.type(url, "https://proxy.example/v1");
    expect(screen.getByLabelText("Chave de API")).toHaveAttribute("placeholder", "Manter a chave atual (••••1111)");
    await user.click(screen.getByRole("button", { name: "Salvar" }));
    await waitFor(() => expect(drafts).toHaveLength(1));
    expect(drafts[0]).toMatchObject({ draft: { id: saved.id, name: "Groq trabalho", preset: "groq", baseUrl: "https://proxy.example/v1" }, credential: null });
    expect(await screen.findByText("Groq trabalho")).toBeInTheDocument();
    expect(screen.getByText(/proxy\.example.*••••1111/)).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Editar Groq trabalho" }));
    await user.type(screen.getByLabelText("Chave de API"), "gsk_new_2222");
    await user.click(screen.getByRole("button", { name: "Salvar" }));
    await waitFor(() => expect(drafts).toHaveLength(2));
    expect(drafts[1]).toMatchObject({ draft: { id: saved.id }, credential: "gsk_new_2222" });
    expect(await screen.findByText(/••••2222/)).toBeInTheDocument();
    expect(bridge.state.providers).toHaveLength(1);
  });

  it("adds, corrects and removes a model by hand for a provider without /models", async () => {
    const bridge = await freshApp({ signedIn: true });
    await bridge.invoke("providers_save", { draft: { id: "local", name: "Local", preset: "custom", wire: "chat", baseUrl: "http://127.0.0.1:9/v1" }, credential: null });
    const calls: [string, any][] = [];
    const invoke = bridge.invoke;
    bridge.invoke = (async <R,>(cmd: string, args?: Record<string, unknown>) => {
      if (cmd.startsWith("providers_model")) calls.push([cmd, args]);
      return invoke<R>(cmd, args);
    }) as typeof bridge.invoke;
    window.location.hash = "#/settings/providers";
    const user = userEvent.setup();
    render(<SettingsApp />);
    await user.click(await screen.findByRole("button", { name: "Modelos de Local" }));
    await user.click(screen.getByRole("button", { name: "Adicionar modelo" }));
    await user.type(screen.getByRole("textbox", { name: "ID do modelo" }), "qwen3:8b");
    await user.type(screen.getByRole("textbox", { name: "Nome de exibição" }), "Qwen 3 8B");
    await user.type(screen.getByRole("spinbutton", { name: "Janela de contexto (tokens)" }), "32768");
    await user.click(screen.getByRole("checkbox", { name: "Ferramentas" }));
    await user.click(screen.getByRole("checkbox", { name: "Raciocínio" }));
    await user.click(screen.getByRole("button", { name: "Salvar modelo" }));
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0]).toEqual(["providers_model_save", { id: "local", model: { id: "qwen3:8b", displayName: "Qwen 3 8B", contextWindow: 32768, maxOutput: null, supportsImages: false, supportsTools: true, supportsReasoning: true, estimated: false, efforts: [], defaultEffort: null } }]);
    const row = await screen.findByRole("listitem", { name: "Qwen 3 8B" });
    expect(row).toHaveTextContent("Ferramentas · Raciocínio");
    expect(row).toHaveTextContent("Manual");

    await user.click(within(row).getByRole("button", { name: "Editar Qwen 3 8B" }));
    expect(screen.getByRole("textbox", { name: "ID do modelo" })).toHaveValue("qwen3:8b");
    await user.click(screen.getByRole("checkbox", { name: "Imagem" }));
    await user.click(screen.getByRole("button", { name: "Salvar modelo" }));
    await waitFor(() => expect(calls).toHaveLength(2));
    expect(calls[1][1].model).toMatchObject({ id: "qwen3:8b", supportsImages: true });

    await user.click(within(await screen.findByRole("listitem", { name: "Qwen 3 8B" })).getByRole("button", { name: "Remover Qwen 3 8B" }));
    await waitFor(() => expect(calls.at(-1)).toEqual(["providers_model_remove", { id: "local", modelId: "qwen3:8b" }]));
    await waitFor(() => expect(screen.queryByRole("listitem", { name: "Qwen 3 8B" })).toBeNull());
  });

  it("a custom provider model declares its efforts and default (013)", async () => {
    const bridge = await freshApp({ signedIn: true });
    await bridge.invoke("providers_save", { draft: { id: "local", name: "Local", preset: "custom", wire: "responses", baseUrl: "http://127.0.0.1:9/v1" }, credential: null });
    const calls: any[] = [];
    const invoke = bridge.invoke;
    bridge.invoke = (async <R,>(cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "providers_model_save") calls.push(args);
      return invoke<R>(cmd, args);
    }) as typeof bridge.invoke;
    window.location.hash = "#/settings/providers";
    const user = userEvent.setup();
    render(<SettingsApp />);
    await user.click(await screen.findByRole("button", { name: "Modelos de Local" }));
    await user.click(screen.getByRole("button", { name: "Adicionar modelo" }));
    await user.type(screen.getByRole("textbox", { name: "ID do modelo" }), "qa-reasoner");
    expect(screen.queryByRole("group", { name: "Esforços aceitos" })).toBeNull();
    await user.click(screen.getByRole("checkbox", { name: "Raciocínio" }));
    const group = screen.getByRole("group", { name: "Esforços aceitos" });
    for (const e of ["Máximo", "Baixo", "Alto"]) await user.click(within(group).getByRole("checkbox", { name: e }));
    const def = screen.getByRole("combobox", { name: "Esforço padrão" });
    expect(within(def).getAllByRole("option").map((o) => o.textContent)).toEqual(["Nenhum (o do provedor)", "Baixo", "Alto", "Máximo"]);
    await user.selectOptions(def, "high");
    await user.click(screen.getByRole("button", { name: "Salvar modelo" }));
    await waitFor(() => expect(calls).toHaveLength(1));
    expect(calls[0].model).toMatchObject({ id: "qa-reasoner", supportsReasoning: true, efforts: ["low", "high", "max"], defaultEffort: "high" });
    const row = await screen.findByRole("listitem", { name: "qa-reasoner" });
    expect(row).toHaveTextContent("Baixo, Alto, Máximo");
  });

  it("edits the reasoning effort of each model per mode (013)", async () => {
    const bridge = await freshApp({ signedIn: true });
    window.location.hash = "#/settings/account";
    const user = userEvent.setup();
    render(<SettingsApp />);
    const table = await screen.findByRole("table", { name: "Esforço por modelo e modo" });
    const row = within(table).getByRole("row", { name: /GPT-6\.1 Sol/ });
    const chat = within(row).getByRole("combobox", { name: "GPT-6.1 Sol — Chat" });
    expect(within(chat).getAllByRole("option").map((o) => o.textContent)).toEqual(["Padrão do modelo (médio)", "Baixo", "Médio", "Alto", "Muito alto", "Máximo"]);
    await user.selectOptions(within(row).getByRole("combobox", { name: "GPT-6.1 Sol — Tarefa" }), "max");
    await waitFor(() => expect(bridge.state.settings.effortPresets["aura-chatgpt-plan::gpt-6.1-sol"]).toEqual({ task: "max" }));
    await user.selectOptions(within(row).getByRole("combobox", { name: "GPT-6.1 Sol — Tarefa" }), "");
    await waitFor(() => expect(bridge.state.settings.effortPresets["aura-chatgpt-plan::gpt-6.1-sol"]).toBeUndefined());
  });

  it("configures a custom provider's format and extra headers", async () => {
    const bridge = await freshApp({ signedIn: true });
    const drafts: any[] = [];
    const invoke = bridge.invoke;
    bridge.invoke = (async <R,>(cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "providers_save") drafts.push(args);
      return invoke<R>(cmd, args);
    }) as typeof bridge.invoke;
    window.location.hash = "#/settings/providers";
    const user = userEvent.setup();
    render(<SettingsApp />);
    await user.click(await screen.findByRole("button", { name: "Adicionar provedor" }));
    await user.selectOptions(screen.getByDisplayValue("OpenAI"), "custom");
    const format = screen.getByRole("combobox", { name: "Formato da API" });
    expect([...format.querySelectorAll("option")].map((o) => o.textContent)).toEqual(["Responses (OpenAI)", "Chat Completions", "Anthropic Messages"]);
    await user.selectOptions(format, "anthropic");
    await user.type(screen.getByRole("textbox", { name: "URL base" }), "https://llm.example/v1");
    await user.click(screen.getByRole("button", { name: "Adicionar cabeçalho" }));
    await user.type(screen.getByRole("textbox", { name: "Nome do cabeçalho 1" }), "X-Tenant");
    await user.type(screen.getByRole("textbox", { name: "Valor do cabeçalho 1" }), "alpha");
    await user.click(screen.getByRole("button", { name: "Adicionar cabeçalho" }));
    await user.type(screen.getByRole("textbox", { name: "Nome do cabeçalho 2" }), "X Bad");
    expect(screen.getByRole("button", { name: "Salvar" })).toBeDisabled();
    expect(screen.getByText("Nome de cabeçalho inválido: X Bad")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Remover cabeçalho 2" }));
    await user.click(screen.getByRole("button", { name: "Salvar" }));
    await waitFor(() => expect(drafts).toHaveLength(1));
    expect(drafts[0].draft).toMatchObject({ preset: "custom", wire: "anthropic", baseUrl: "https://llm.example/v1", extraHeaders: { "X-Tenant": "alpha" } });
  });

  it("shows comparable speed, accuracy, languages and hardware requirements for each model", async () => {
    await freshApp({ signedIn: true });
    window.location.hash = "#/settings/voice";
    render(<SettingsApp />);
    const parakeet = (await screen.findByText("Parakeet TDT 0.6B v3")).closest("[data-model]") as HTMLElement;
    expect(within(parakeet).getByRole("meter", { name: "Velocidade" })).toHaveAttribute("aria-valuenow", "90");
    expect(within(parakeet).getByRole("meter", { name: "Precisão" })).toHaveAttribute("aria-valuenow", "85");
    expect(within(parakeet).getByText("Idiomas: pt, en, es")).toBeInTheDocument();
    expect(within(parakeet).getByText("CPU · mín. 2 GB de RAM")).toBeInTheDocument();
    const large = screen.getByText("Whisper Large v3 Turbo (Q5)").closest("[data-model]") as HTMLElement;
    expect(within(large).getByText("GPU recomendada · mín. 2 GB de RAM")).toBeInTheDocument();
    const small = screen.getByText("Whisper Small").closest("[data-model]") as HTMLElement;
    expect(within(small).getByText("Idiomas: multilíngue")).toBeInTheDocument();
  });

  it("tests the microphone and system audio with live meters and saves the output device", async () => {
    const bridge = await freshApp({ signedIn: true });
    const calls: [string, Record<string, unknown> | undefined][] = [];
    const invoke = bridge.invoke;
    bridge.invoke = ((cmd: string, args?: Record<string, unknown>) => {
      if (cmd.startsWith("audio_test")) calls.push([cmd, args]);
      return invoke(cmd, args);
    }) as typeof bridge.invoke;
    window.location.hash = "#/settings/voice";
    const user = userEvent.setup();
    render(<SettingsApp />);
    await user.click(await screen.findByRole("button", { name: "Testar microfone" }));
    expect(calls).toEqual([["audio_test_start", { source: "mic", device: null }]]);
    // −6 dBFS on a −60…0 scale → 90.
    act(() => useApp.getState().handle({ channel: "audioLevel", event: { source: "mic", dbfs: -6 } }));
    expect(screen.getByRole("meter", { name: "Nível do microfone" })).toHaveAttribute("aria-valuenow", "90");
    await user.click(screen.getByRole("button", { name: "Parar teste do microfone" }));
    expect(calls.at(-1)).toEqual(["audio_test_stop", { source: "mic" }]);
    expect(screen.queryByRole("meter", { name: "Nível do microfone" })).toBeNull();

    const output = screen.getByRole("combobox", { name: "Áudio do sistema" });
    await user.selectOptions(output, "Alto-falantes");
    await waitFor(() => expect(bridge.state.settings.systemAudioDeviceId).toBe("Alto-falantes"));
    await user.click(screen.getByRole("button", { name: "Testar áudio do sistema" }));
    expect(calls.at(-1)).toEqual(["audio_test_start", { source: "systemAudio", device: null }]);
    act(() => useApp.getState().handle({ channel: "audioLevel", event: { source: "systemAudio", dbfs: -60 } }));
    expect(screen.getByRole("meter", { name: "Nível do áudio do sistema" })).toHaveAttribute("aria-valuenow", "0");
    expect(screen.getByRole("button", { name: "Tocar som de teste" })).toBeInTheDocument();
  });

  it("shows catalog descriptions in the interface language", async () => {
    const bridge = await freshApp({ signedIn: true });
    await bridge.invoke("settings_update", { patch: { language: "en" } });
    window.location.hash = "#/settings/voice";
    render(<SettingsApp />);
    const parakeet = (await screen.findByText("Parakeet TDT 0.6B v3")).closest("[data-model]") as HTMLElement;
    expect(within(parakeet).getByText(/Fast on CPU, 25 European languages including Portuguese\./)).toBeInTheDocument();
    expect(within(parakeet).queryByText(/Rápido/)).toBeNull();
  });

  it("downloads a voice model with progress", async () => {
    await freshApp({ signedIn: true });
    window.location.hash = "#/settings/voice";
    const user = userEvent.setup();
    render(<SettingsApp />);
    const buttons = await screen.findAllByRole("button", { name: "Baixar" });
    await user.click(buttons[0]);
    await waitFor(() => expect(screen.getByText("Em uso")).toBeInTheDocument());
  });
});

describe("App profiles", () => {
  it("creates a profile prefilled from the app in front", async () => {
    await freshApp({ signedIn: true });
    window.location.hash = "#/settings/profiles";
    const user = userEvent.setup();
    render(<SettingsApp />);
    await user.click(await screen.findByRole("button", { name: "Criar perfil para este app" }));
    expect(screen.getByDisplayValue("Code.exe")).toBeInTheDocument();
    await user.type(screen.getByRole("textbox", { name: "Instruções" }), "Responda com TypeScript");
    await user.click(screen.getByRole("button", { name: "Salvar" }));
    expect(await screen.findByText("Responda com TypeScript")).toBeInTheDocument();
  });

  it("chooses the profile's default model among ChatGPT and provider models (QA-036)", async () => {
    const bridge = await freshApp({ signedIn: true });
    const p = await bridge.invoke<{ id: string }>("providers_save", { draft: { name: "QA Local", preset: "ollama", baseUrl: "http://127.0.0.1:9/v1" }, credential: null });
    bridge.state.providers = bridge.state.providers.map((x) =>
      x.id === p.id ? { ...x, models: [{ id: "qwen3:8b", displayName: "Qwen 3 8B", contextWindow: null, maxOutput: null, supportsImages: false, supportsTools: true, supportsReasoning: false, estimated: true }] } : x,
    );
    window.location.hash = "#/settings/profiles";
    const user = userEvent.setup();
    render(<SettingsApp />);
    await user.click(await screen.findByRole("button", { name: "Criar perfil para este app" }));
    const model = screen.getByRole("combobox", { name: "Modelo padrão" });
    await waitFor(() => expect(within(model).getByRole("option", { name: "QA Local · Qwen 3 8B" })).toBeInTheDocument());
    expect(within(model).getByRole("option", { name: "ChatGPT · GPT-6.1 Sol" })).toBeInTheDocument();
    await user.selectOptions(model, "QA Local · Qwen 3 8B");
    await user.click(screen.getByRole("button", { name: "Salvar" }));
    await waitFor(() => expect(bridge.state.profiles[0]?.defaultModel).toBe(`aura-${p.id}::qwen3:8b`));
    expect(await screen.findByText("QA Local · Qwen 3 8B")).toBeInTheDocument();
  });
});
