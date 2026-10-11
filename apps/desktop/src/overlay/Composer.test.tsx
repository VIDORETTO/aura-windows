// 015: `/` does something, `@` adds context; queue, visible mode, message actions.
import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { freshApp } from "../test/harness";
import { setBridge } from "../ipc/bridge";
import { OverlayApp } from "./OverlayApp";
import type { ModelInfo, Settings } from "../ipc/types";

type Bridge = Awaited<ReturnType<typeof freshApp>>;

function spy(bridge: Bridge, hold = false) {
  const calls: { cmd: string; args: Record<string, unknown> }[] = [];
  setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
    calls.push({ cmd, args });
    if (hold && cmd === "conversation_send") {
      const { threadId } = args.request as { threadId: string };
      bridge.emitLocal!("aura://event", { channel: "conversation", event: { type: "turnStarted", threadId, turnId: `held_${calls.length}` } });
      return "held" as R;
    }
    return bridge.invoke<R>(cmd, args);
  } });
  const sent = () => calls.filter((c) => c.cmd === "conversation_send").map((c) => (c.args.request as { text: string }).text);
  const count = (cmd: string) => calls.filter((c) => c.cmd === cmd).length;
  return { calls, sent, count };
}

describe("composer menus (015)", () => {
  it("Enter sends a message ending in @word without capturing; @ filters without accents (AC-001)", async () => {
    const bridge = await freshApp({ signedIn: true });
    const s = spy(bridge);
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = await screen.findByRole("combobox");
    await user.type(box, "@regiao");
    const list = await screen.findByRole("listbox");
    expect(within(list).getAllByRole("option").map((o) => o.textContent)).toEqual([expect.stringContaining("@região")]);
    await user.clear(box);
    await user.type(box, "fala com o @joao{Enter}");
    await waitFor(() => expect(s.sent()).toEqual(["fala com o @joao"]));
    expect(s.count("capture_screen")).toBe(0);
  });

  it("context items are single words that can be typed", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.type(await screen.findByRole("combobox"), "@");
    const labels = within(await screen.findByRole("listbox")).getAllByRole("option").map((o) => o.textContent!.split(" ")[0]);
    expect(labels).toEqual(["@tela", "@região", "@janela", "@seleção", "@arquivo", "@recente"]);
  });

  it("Enter picks the highlighted item, a bare trigger is never sent, Esc keeps the text (017 AC-003)", async () => {
    const bridge = await freshApp({ signedIn: true });
    const s = spy(bridge);
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = (await screen.findByRole("combobox")) as HTMLTextAreaElement;
    // "/" + Enter: the first command (already highlighted) is chosen, nothing is sent.
    await user.type(box, "/{Enter}");
    expect(s.sent()).toEqual([]);
    expect(box).toHaveValue("/plano ");
    await user.clear(box);
    // "@" + Enter: the first context item runs (screen capture); "@" never reaches the chat.
    await user.type(box, "@{Enter}");
    await waitFor(() => expect(s.count("capture_screen")).toBe(1));
    expect(box).toHaveValue("");
    expect(s.sent()).toEqual([]);
    // A bare trigger with the menu closed (Esc) is not sent either.
    await user.type(box, "/{Escape}{Enter}");
    expect(box).toHaveValue("/");
    await user.click(screen.getByRole("button", { name: "Enviar" }));
    expect(s.sent()).toEqual([]);
    await user.type(box, "tl{Escape}");
    expect(box).toHaveValue("/tl");
    expect(screen.queryByRole("listbox")).toBeNull();

    await user.clear(box);
    await user.type(box, "veja  agora");
    box.setSelectionRange(5, 5);
    await user.keyboard("@");
    expect(box).toHaveValue("veja @ agora");
    expect(await screen.findByRole("option", { name: /@tela/ })).toBeInTheDocument();
    await user.keyboard("{ArrowDown}{Enter}");
    // Second item: region selector; the trigger word leaves the text.
    expect(box).toHaveValue("veja agora");
    expect(s.count("region_open")).toBe(1);
  });

  it("choosing a skill adds a Skill chip instead of $text (AC-003)", async () => {
    const bridge = await freshApp({ signedIn: true });
    await bridge.invoke("skills_create", { name: "revisar-contrato", description: "Revisa contratos", body: "x" });
    const s = spy(bridge);
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = await screen.findByRole("combobox");
    await user.type(box, "/rev");
    const list = await screen.findByRole("listbox");
    expect(within(list).getByRole("group", { name: "Skills" })).toBeInTheDocument();
    await user.keyboard("{Enter}");
    expect(box).toHaveValue("");
    expect(await screen.findByText("revisar-contrato")).toBeInTheDocument();
    expect(s.calls.find((c) => c.cmd === "context_attach_skill")?.args).toEqual({ tray: "draft", name: "revisar-contrato" });
    await user.type(box, "analise{Enter}");
    await waitFor(() => expect(s.sent()).toEqual(["analise"]));
  });

  it("built-in commands follow the interface language and /tela leaves the menu (AC-004)", async () => {
    const bridge = await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = await screen.findByRole("combobox");
    await user.type(box, "/");
    let list = await screen.findByRole("listbox");
    expect(within(list).getByRole("option", { name: /^\/plano/ })).toBeInTheDocument();
    expect(within(list).getByRole("option", { name: /^\/compactar/ })).toBeInTheDocument();
    expect(within(list).queryByRole("option", { name: /^\/tela/ })).toBeNull();
    await act(async () => { await bridge.invoke("settings_update", { patch: { language: "en" } }); });
    list = await screen.findByRole("listbox");
    await waitFor(() => expect(within(list).getByRole("option", { name: /^\/plan\b/ })).toBeInTheDocument());
    expect(within(list).getByRole("option", { name: /^\/compact\b/ })).toBeInTheDocument();
    expect(within(list).queryByRole("option", { name: /^\/(screen|tela)/ })).toBeNull();
  });

  it("Enter on a complete alias runs the command in the other language", async () => {
    const bridge = await freshApp({ signedIn: true });
    await bridge.invoke("settings_update", { patch: { language: "en" } });
    const s = spy(bridge);
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = await screen.findByRole("combobox");
    await user.type(box, "hello{Enter}");
    await waitFor(() => expect(s.sent()).toEqual(["hello"]));
    const { useSession } = await import("./session");
    await waitFor(() => expect(useSession.getState().sending).toBe(false));
    await user.type(box, "/compactar{Enter}");
    await waitFor(() => expect(s.count("conversation_compact")).toBe(1));
    expect(box).toHaveValue("");
    expect(s.sent()).toEqual(["hello"]);
  });

  it("a typed quick command shows its argument and text source (AC-005)", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.type(await screen.findByRole("combobox"), "/traduzir ");
    expect(await screen.findByText(/‹inglês›/)).toBeInTheDocument();
    expect(screen.getByText(/usa a seleção ou o texto digitado/)).toBeInTheDocument();
  });

  it("the input is a combobox tied to its suggestions", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = await screen.findByRole("combobox");
    await user.type(box, "/tl");
    const list = await screen.findByRole("listbox");
    expect(box).toHaveAttribute("aria-expanded", "true");
    expect(box).toHaveAttribute("aria-controls", list.id);
    const active = box.getAttribute("aria-activedescendant");
    expect(active && document.getElementById(active)).toHaveAttribute("aria-selected", "true");
  });
});

describe("model picker", () => {
  it("reads legacy default effort and preserves its other modes when saving a model choice (038 AC-006)", async () => {
    const bridge = await freshApp({ signedIn: true });
    await bridge.invoke("settings_update", { patch: { effortPresets: { "aura-chatgpt-plan::default": { chat: "high", plan: "low" } } } });
    const catalog: ModelInfo[] = [{ id: "qa-reasoner", displayName: "QA reasoner", efforts: ["low", "high", "max"], defaultEffort: "high", inputModalities: ["text"], isDefault: true }];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args?: Record<string, unknown>) =>
      cmd === "models_list" ? catalog as R : bridge.invoke<R>(cmd, args),
    });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Modelo QA reasoner, esforço Alto" }));
    await user.click(screen.getByRole("menuitemradio", { name: "Baixo" }));
    await waitFor(async () => {
      const settings = await bridge.invoke<Settings>("settings_get");
      expect(settings.effortPresets).toEqual({
        "aura-chatgpt-plan::qa-reasoner": { chat: "low" },
        "aura-chatgpt-plan::default": { plan: "low" },
      });
    });
  });

  it("remembers effort by actual model and mode across model changes and a fresh Overlay (038 AC-006)", async () => {
    const catalog: ModelInfo[] = [
      { id: "qa-reasoner", displayName: "QA reasoner", efforts: ["low", "high", "max"], defaultEffort: "high", inputModalities: ["text"], isDefault: true },
      { id: "qa-fast", displayName: "QA fast", efforts: [], defaultEffort: null, inputModalities: ["text"], isDefault: false },
    ];
    const bridge = await freshApp({ signedIn: true });
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args?: Record<string, unknown>) =>
      cmd === "models_list" ? catalog as R : bridge.invoke<R>(cmd, args),
    });
    const user = userEvent.setup();
    const view = render(<OverlayApp />);
    await screen.findByRole("button", { name: "Modelo QA reasoner" });
    await user.click(screen.getByRole("button", { name: "Expandir" }));
    await user.click(screen.getByRole("button", { name: /^Modelo QA reasoner, modo Chat/ }));
    await user.click(screen.getByRole("menuitemradio", { name: "Baixo" }));
    await user.click(screen.getByRole("menuitemradio", { name: /Tarefa/ }));
    await user.click(screen.getByRole("menuitemradio", { name: "Máximo" }));
    await user.click(screen.getByRole("menuitemradio", { name: "QA fast" }));
    await user.click(screen.getByRole("menuitemradio", { name: "QA reasoner" }));
    expect(screen.getByRole("button", { name: "Modelo QA reasoner, modo Tarefa, esforço Máximo" })).toBeInTheDocument();
    await user.click(screen.getByRole("menuitemradio", { name: "Chat" }));
    expect(screen.getByRole("button", { name: "Modelo QA reasoner, modo Chat, esforço Baixo" })).toBeInTheDocument();
    await waitFor(async () => {
      const settings = await bridge.invoke<Settings>("settings_get");
      expect(settings.effortPresets["aura-chatgpt-plan::qa-reasoner"]).toEqual({ chat: "low", task: "max" });
    });
    const persisted = await bridge.invoke<Settings>("settings_get");
    view.unmount();
    const restarted = await freshApp({ signedIn: true });
    await restarted.invoke("settings_update", { patch: { effortPresets: persisted.effortPresets } });
    setBridge({ ...restarted, invoke: async <R,>(cmd: string, args?: Record<string, unknown>) =>
      cmd === "models_list" ? catalog as R : restarted.invoke<R>(cmd, args),
    });
    render(<OverlayApp />);
    expect(await screen.findByRole("button", { name: "Modelo QA reasoner, esforço Baixo" })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Modo: Chat" }));
    await user.click(screen.getByRole("menuitemradio", { name: /Tarefa/ }));
    expect(screen.getByRole("button", { name: "Modelo QA reasoner, esforço Máximo" })).toBeInTheDocument();
  });

  it("explains why the effort resets when switching to a model without reasoning (038 AC-007)", async () => {
    const bridge = await freshApp({ signedIn: true });
    const catalog: ModelInfo[] = [
      { id: "qa-reasoner", displayName: "QA reasoner", efforts: ["low", "high", "max"], defaultEffort: "high", inputModalities: ["text"], isDefault: true },
      { id: "qa-fast", displayName: "QA fast", efforts: [], defaultEffort: null, inputModalities: ["text"], isDefault: false },
    ];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args?: Record<string, unknown>) =>
      cmd === "models_list" ? catalog as R : bridge.invoke<R>(cmd, args),
    });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Modelo QA reasoner" }));
    await user.click(screen.getByRole("menuitemradio", { name: "Máximo" }));
    await user.click(screen.getByRole("menuitemradio", { name: "QA fast" }));
    expect(screen.getByRole("button", { name: "Modelo QA fast" })).toBeInTheDocument();
    expect(screen.getByText("Este modelo não tem esforço de raciocínio configurável.")).toBeInTheDocument();
    expect(screen.getByText("O esforço anterior não está disponível neste modelo. Usando o padrão do modelo.")).toBeInTheDocument();
  });

  it("revalidates the captured effort if capabilities change while starting the conversation (038 AC-007)", async () => {
    const bridge = await freshApp({ signedIn: true });
    let catalog: ModelInfo[] = [{ id: "qa-reasoner", displayName: "QA reasoner", efforts: ["low", "high", "max"], defaultEffort: "high", inputModalities: ["text"], isDefault: true }];
    let finishStart!: () => void;
    const starting = new Promise<void>((resolve) => { finishStart = resolve; });
    let began = false;
    const efforts: (string | null)[] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd === "models_list") return catalog as R;
      if (cmd === "conversation_start") {
        began = true;
        const result = await bridge.invoke<R>(cmd, args);
        await starting;
        return result;
      }
      if (cmd === "conversation_send") efforts.push((args.request as { options: { effort: string | null } }).options.effort);
      return bridge.invoke<R>(cmd, args);
    } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Modelo QA reasoner" }));
    await user.click(screen.getByRole("menuitemradio", { name: "QA reasoner" }));
    await user.click(screen.getByRole("menuitemradio", { name: "Máximo" }));
    await user.keyboard("{Escape}");
    await user.type(screen.getByRole("combobox"), "oi{Enter}");
    await waitFor(() => expect(began).toBe(true));
    catalog = [{ ...catalog[0], efforts: ["low", "high"] }];
    act(() => bridge.emitLocal!("aura://event", { channel: "providersChanged", event: {} }));
    await screen.findByText("O esforço anterior não está disponível neste modelo. Usando o padrão do modelo.");
    await act(async () => { finishStart(); });
    await waitFor(() => expect(efforts).toEqual([null]));
  });

  it("announces a removed effort and never sends it after a confirmed catalog change (038 AC-007)", async () => {
    const bridge = await freshApp({ signedIn: true });
    let catalog: ModelInfo[] = [{ id: "qa-reasoner", displayName: "QA reasoner", efforts: ["low", "high", "max"], defaultEffort: "high", inputModalities: ["text"], isDefault: true }];
    const sent: { model: string | null; effort: string | null }[] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd === "models_list") return catalog as R;
      if (cmd === "conversation_send") sent.push((args.request as { options: { model: string | null; effort: string | null } }).options);
      return bridge.invoke<R>(cmd, args);
    } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Modelo QA reasoner" }));
    const picker = screen.getByRole("dialog", { name: "Modelo e esforço" });
    await user.click(within(picker).getByRole("menuitemradio", { name: "QA reasoner" }));
    await user.click(within(picker).getByRole("menuitemradio", { name: "Máximo" }));
    catalog = [{ ...catalog[0], efforts: ["low", "high"] }];
    act(() => bridge.emitLocal!("aura://event", { channel: "providersChanged", event: {} }));
    expect(await screen.findByText("O esforço anterior não está disponível neste modelo. Usando o padrão do modelo.")).toBeInTheDocument();
    expect(within(picker).queryByRole("menuitemradio", { name: "Máximo" })).toBeNull();
    await user.keyboard("{Escape}");
    await user.type(screen.getByRole("combobox"), "oi{Enter}");
    await waitFor(() => expect(sent).toEqual([{ model: "qa-reasoner", effort: null }]));
  });

  it("restores the default model's saved effort when its catalog arrives (038 AC-006 AC-007)", async () => {
    const bridge = await freshApp({ signedIn: true });
    await bridge.invoke("settings_update", { patch: { defaultModel: "qa-reasoner", effortPresets: { "aura-chatgpt-plan::qa-reasoner": { chat: "max" } } } });
    let finish!: (value: unknown) => void;
    const pending = new Promise((resolve) => { finish = resolve; });
    const catalog: ModelInfo[] = [{ id: "qa-reasoner", displayName: "QA reasoner", efforts: ["low", "high", "max"], defaultEffort: "high", inputModalities: ["text"], isDefault: true }];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args?: Record<string, unknown>) =>
      cmd === "models_list" ? await pending as R : bridge.invoke<R>(cmd, args),
    });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: /^Modelo/ }));
    await act(async () => { finish(catalog); });
    expect(await screen.findByRole("button", { name: "Modelo QA reasoner, esforço Máximo" })).toBeInTheDocument();
    expect(screen.getByRole("menuitemradio", { name: "Máximo" })).toHaveAttribute("aria-checked", "true");
  });

  it("allows the next model and effort in a conversation while keeping its provider fixed (038 AC-002)", async () => {
    const bridge = await freshApp({ signedIn: true });
    await bridge.invoke("providers_save", { draft: { name: "Other provider", preset: "ollama" }, credential: null });
    const s = spy(bridge, true);
    const user = userEvent.setup();
    render(<OverlayApp />);
    await screen.findByRole("button", { name: "Modelo GPT-6 Luna" });
    await user.type(screen.getByRole("combobox"), "primeiro{Enter}");
    await waitFor(() => expect(s.sent()).toEqual(["primeiro"]));
    await user.click(screen.getByRole("button", { name: /^Modelo GPT-6 Luna, modo Chat/ }));
    const picker = screen.getByRole("dialog", { name: "Modelo e modo" });
    const providers = within(picker).getByRole("menu", { name: "Provedor" });
    expect(within(providers).getAllByRole("menuitemradio").map((p) => p.textContent)).toEqual(["ChatGPT"]);
    expect(within(picker).getByText("Para trocar de provedor, comece uma nova conversa.")).toBeInTheDocument();
    await user.click(within(picker).getByRole("menuitemradio", { name: "GPT-6 Astra" }));
    await user.click(within(picker).getByRole("menuitemradio", { name: "Alto" }));
    expect(screen.getByRole("button", { name: "Modelo GPT-6 Astra, modo Chat, esforço Alto" })).toBeInTheDocument();
    const start = s.calls.find((c) => c.cmd === "conversation_start")!.args.options as { model: string };
    const first = s.calls.find((c) => c.cmd === "conversation_send")!.args.request as { options: { model: string | null; effort: string | null } };
    expect(start.model).toBe("gpt-6-luna");
    // A null per-turn override inherits the model fixed at conversation/start.
    expect(first.options).toEqual({ model: null, effort: null });
    expect(s.sent()).toEqual(["primeiro"]);
  });

  it("keeps BYOK models available despite a plan failure and a failed provider refresh (038 AC-004)", async () => {
    const bridge = await freshApp({ signedIn: false });
    bridge.state.providers = [{ id: "qa", name: "QA", preset: "custom", wire: "responses", baseUrl: "http://127.0.0.1:9/v1", auth: "none", extraHeaders: {}, credentialHint: null, status: "verified", lastError: null, quirks: { noParallelToolCalls: false, reasoningEffort: false, noStreamOptions: false },
      models: [{ id: "qa-reasoner", displayName: "QA reasoner", contextWindow: null, maxOutput: null, supportsImages: false, supportsTools: true, supportsReasoning: true, estimated: false, manual: true, efforts: ["low", "high", "max"], defaultEffort: "high" }] }];
    let providerUnavailable = false;
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "models_list" || (cmd === "providers_list" && providerUnavailable)) throw { code: "agent_unavailable", message: "Unavailable" };
      return bridge.invoke<R>(cmd, args);
    } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await screen.findByRole("combobox");
    await user.click(screen.getByRole("button", { name: /^Modelo/ }));
    const picker = screen.getByRole("dialog", { name: "Modelo e esforço" });
    await user.click(await within(picker).findByRole("menuitemradio", { name: "QA reasoner" }));
    await user.click(within(picker).getByRole("menuitemradio", { name: "Máximo" }));
    providerUnavailable = true;
    act(() => bridge.emitLocal!("aura://event", { channel: "providersChanged", event: {} }));
    expect(await within(picker).findByText("Não foi possível carregar os modelos.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Modelo QA reasoner, esforço Máximo" })).toBeInTheDocument();
    expect(within(picker).getByRole("menuitemradio", { name: "QA reasoner" })).toBeInTheDocument();
    expect(screen.getByRole("combobox")).toBeInTheDocument();
  });

  it("does not let an older model response replace a newer catalog (038 AC-004)", async () => {
    const bridge = await freshApp({ signedIn: true });
    let finishOld!: (value: unknown) => void;
    const old = new Promise((resolve) => { finishOld = resolve; });
    const newer: ModelInfo[] = [{ id: "qa-new", displayName: "New catalog model", efforts: [], defaultEffort: null, inputModalities: ["text"], isDefault: true }];
    let requests = 0;
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "models_list") return (++requests === 1 ? await old : newer) as R;
      return bridge.invoke<R>(cmd, args);
    } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: /^Modelo/ }));
    const picker = screen.getByRole("dialog", { name: "Modelo e esforço" });
    act(() => bridge.emitLocal!("aura://event", { channel: "providersChanged", event: {} }));
    expect(await within(picker).findByRole("menuitemradio", { name: "New catalog model" })).toBeInTheDocument();
    await act(async () => { finishOld([{ ...newer[0], id: "old", displayName: "Obsolete model" }]); });
    expect(within(picker).getByRole("menuitemradio", { name: "New catalog model" })).toBeInTheDocument();
    expect(within(picker).queryByRole("menuitemradio", { name: "Obsolete model" })).toBeNull();
  });

  it("keeps known models and effort after a failed refresh and updates the open picker on retry (038 AC-004)", async () => {
    const bridge = await freshApp({ signedIn: true });
    let catalog: ModelInfo[] = [{ id: "qa-reasoner", displayName: "QA reasoner", efforts: ["low", "high", "max"], defaultEffort: "high", inputModalities: ["text"], isDefault: true }];
    let unavailable = false;
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "models_list") {
        if (unavailable) throw { code: "agent_unavailable", message: "Unavailable" };
        return catalog as R;
      }
      return bridge.invoke<R>(cmd, args);
    } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Modelo QA reasoner" }));
    const picker = screen.getByRole("dialog", { name: "Modelo e esforço" });
    await user.click(within(picker).getByRole("menuitemradio", { name: "QA reasoner" }));
    await user.click(within(picker).getByRole("menuitemradio", { name: "Máximo" }));
    unavailable = true;
    act(() => bridge.emitLocal!("aura://event", { channel: "providersChanged", event: {} }));
    expect(await within(picker).findByText("Não foi possível carregar os modelos.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Modelo QA reasoner, esforço Máximo" })).toBeInTheDocument();
    expect(within(picker).getByRole("menuitemradio", { name: "QA reasoner" })).toBeInTheDocument();
    unavailable = false;
    catalog = [{ id: "qa-fast", displayName: "QA fast", efforts: [], defaultEffort: null, inputModalities: ["text"], isDefault: true }];
    await user.click(within(picker).getByRole("button", { name: "Tentar novamente" }));
    expect(await within(picker).findByRole("menuitemradio", { name: "QA fast" })).toBeInTheDocument();
    expect(within(picker).queryByRole("menuitemradio", { name: "QA reasoner" })).toBeNull();
  });

  it("distinguishes a pending catalog from a successful empty catalog (038 AC-003)", async () => {
    const bridge = await freshApp({ signedIn: true });
    let finish!: (value: unknown) => void;
    const pending = new Promise((resolve) => { finish = resolve; });
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args?: Record<string, unknown>) =>
      cmd === "models_list" ? await pending as R : bridge.invoke<R>(cmd, args),
    });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: /^Modelo/ }));
    const picker = screen.getByRole("dialog", { name: "Modelo e esforço" });
    expect(within(picker).getByText("Carregando modelos…")).toBeInTheDocument();
    await act(async () => { finish([]); });
    expect(await within(picker).findByText("Nenhum modelo disponível neste provedor.")).toBeInTheDocument();
    expect(within(picker).queryByText("Carregando modelos…")).toBeNull();
  });

  it("recovers a failed model catalog from the compact picker (038 AC-003)", async () => {
    const bridge = await freshApp({ signedIn: true });
    let unavailable = true;
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "models_list" && unavailable) throw { code: "agent_unavailable", message: "Unavailable" };
      return bridge.invoke<R>(cmd, args);
    } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: /^Modelo/ }));
    const picker = screen.getByRole("dialog", { name: "Modelo e esforço" });
    expect(await within(picker).findByText("Não foi possível carregar os modelos.")).toBeInTheDocument();
    unavailable = false;
    await user.click(within(picker).getByRole("button", { name: "Tentar novamente" }));
    expect(await within(picker).findByRole("menuitemradio", { name: "GPT-6 Luna" })).toBeInTheDocument();
    expect(within(picker).queryByText("Não foi possível carregar os modelos.")).toBeNull();
  });

  it("names the model that 'default' stands for", async () => {
    const bridge = await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Expandir" }));
    expect(await screen.findByRole("button", { name: /^Modelo GPT-6 Luna, modo Chat/ })).toBeInTheDocument();
    await act(async () => { await bridge.invoke("settings_update", { patch: { defaultModel: "gpt-6.1-sol" } }); });
    expect(await screen.findByRole("button", { name: /^Modelo GPT-6\.1 Sol, modo Chat/ })).toBeInTheDocument();
    // A default the plan no longer offers (GPT-5.x) falls back to the plan default.
    await act(async () => { await bridge.invoke("settings_update", { patch: { defaultModel: "gpt-5.5" } }); });
    expect(await screen.findByRole("button", { name: /^Modelo GPT-6 Luna, modo Chat/ })).toBeInTheDocument();
  });

  it("offers only the GPT-6 trio and chooses model and effort before the first message (017 AC-001, AC-002)", async () => {
    const bridge = await freshApp({ signedIn: true });
    await act(async () => { await bridge.invoke("settings_update", { patch: { defaultModel: "gpt-5.5" } }); });
    const s = spy(bridge);
    const user = userEvent.setup();
    render(<OverlayApp />);
    // Compact Overlay, no conversation yet: the model pill is there.
    const pill = await screen.findByRole("button", { name: "Modelo GPT-6 Luna" });
    expect(screen.queryByRole("button", { name: "Expandir" })).toBeInTheDocument();
    await user.click(pill);
    const picker = screen.getByRole("dialog", { name: "Modelo e esforço" });
    const models = within(within(picker).getByRole("menu", { name: "Modelo" })).getAllByRole("menuitemradio");
    expect(models.map((m) => m.querySelector("span span")?.textContent)).toEqual(["GPT-6 Luna", "GPT-6.1 Sol", "GPT-6 Astra"]);
    await user.click(models[2]);
    await user.click(within(within(picker).getByRole("menu", { name: "Esforço de raciocínio" })).getByRole("menuitemradio", { name: "Alto" }));
    expect(screen.getByRole("button", { name: "Modelo GPT-6 Astra, esforço Alto" })).toHaveTextContent("GPT-6 Astra·Alto");
    await user.keyboard("{Escape}");
    await user.type(screen.getByRole("combobox"), "oi{Enter}");
    await waitFor(() => expect(s.sent()).toEqual(["oi"]));
    const start = s.calls.find((c) => c.cmd === "conversation_start")!.args.options as { model: string };
    const send = s.calls.find((c) => c.cmd === "conversation_send")!.args.request as { options: { model: string; effort: string } };
    expect(start.model).toBe("gpt-6-astra");
    expect(send.options).toMatchObject({ model: "gpt-6-astra", effort: "high" });
  });

  it("a new plan conversation starts with an explicit catalog model (017 AC-001)", async () => {
    const bridge = await freshApp({ signedIn: true });
    await act(async () => { await bridge.invoke("settings_update", { patch: { defaultModel: "gpt-5.5" } }); });
    const s = spy(bridge);
    const user = userEvent.setup();
    render(<OverlayApp />);
    await screen.findByRole("button", { name: "Modelo GPT-6 Luna" });
    await user.type(screen.getByRole("combobox"), "oi{Enter}");
    await waitFor(() => expect(s.sent()).toEqual(["oi"]));
    expect((s.calls.find((c) => c.cmd === "conversation_start")!.args.options as { model: string }).model).toBe("gpt-6-luna");
  });
});

describe("queue, mode and message actions (015)", () => {
  it("Enter during an answer queues the message and sends it once when the turn ends (AC-006)", async () => {
    const bridge = await freshApp({ signedIn: true });
    const s = spy(bridge, true);
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = await screen.findByRole("combobox");
    await user.type(box, "primeira{Enter}");
    await waitFor(() => expect(s.sent()).toEqual(["primeira"]));
    const { useSession } = await import("./session");
    const threadId = useSession.getState().threadId!;
    await user.type(box, "e depois?{Enter}");
    expect(await screen.findByText("Na fila: e depois?")).toBeInTheDocument();
    expect(box).toHaveValue("");
    expect(s.sent()).toEqual(["primeira"]);
    act(() => bridge.emitLocal!("aura://event", { channel: "conversation", event: { type: "turnCompleted", threadId, turnId: "held_1", status: "completed", error: null } }));
    await waitFor(() => expect(s.sent()).toEqual(["primeira", "e depois?"]));
    expect(screen.queryByText("Na fila: e depois?")).toBeNull();

    // Cancelling the queue discards it.
    await user.type(box, "descartar{Enter}");
    await user.click(await screen.findByRole("button", { name: "Cancelar mensagem na fila" }));
    act(() => bridge.emitLocal!("aura://event", { channel: "conversation", event: { type: "turnCompleted", threadId, turnId: "held_2", status: "completed", error: null } }));
    await new Promise((r) => setTimeout(r, 20));
    expect(s.sent()).toEqual(["primeira", "e depois?"]);
  });

  it("the compact Overlay shows the mode and switches it (AC-007)", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const badge = await screen.findByRole("button", { name: "Modo: Chat" });
    await user.click(badge);
    await user.click(screen.getByRole("menuitemradio", { name: /Tarefa/ }));
    const { useSession } = await import("./session");
    await waitFor(() => expect(useSession.getState().mode).toBe("task"));
    expect(screen.getByRole("button", { name: "Modo: Tarefa" })).toBeInTheDocument();
  });

  it("retry, redo in Task mode and edit on messages (AC-008)", async () => {
    const bridge = await freshApp({ signedIn: true });
    const s = spy(bridge);
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = await screen.findByRole("combobox");
    await user.type(box, "explique este erro{Enter}");
    await waitFor(() => expect(screen.getByText(/tipos incompatíveis/)).toBeInTheDocument());
    const { useSession } = await import("./session");
    await waitFor(() => expect(useSession.getState().sending).toBe(false));
    await waitFor(() => expect(screen.getByRole("button", { name: "Tentar de novo" })).toBeInTheDocument());
    await user.click(screen.getByRole("button", { name: "Tentar de novo" }));
    await waitFor(() => expect(s.sent()).toEqual(["explique este erro", "explique este erro"]));
    expect(useSession.getState().mode).toBe("chat");

    await waitFor(() => expect(useSession.getState().sending).toBe(false));
    await waitFor(() => expect(screen.getAllByRole("button", { name: "Refazer no Modo Tarefa" })).toHaveLength(1));
    await user.click(screen.getByRole("button", { name: "Refazer no Modo Tarefa" }));
    await waitFor(() => expect(s.sent()).toEqual(["explique este erro", "explique este erro", "explique este erro"]));
    expect(useSession.getState().mode).toBe("task");
    await waitFor(() => expect(screen.queryByRole("button", { name: "Refazer no Modo Tarefa" })).toBeNull());

    await user.click(screen.getAllByRole("button", { name: "Editar" })[0]);
    expect(box).toHaveValue("explique este erro");
  });

  it("the context meter asks before compacting (AC-009)", async () => {
    const bridge = await freshApp({ signedIn: true });
    const s = spy(bridge);
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.type(await screen.findByRole("combobox"), "oi{Enter}");
    await user.click(await screen.findByRole("button", { name: /% do contexto usado/ }));
    expect(screen.getByText("Compactar a conversa?")).toBeInTheDocument();
    expect(s.count("conversation_compact")).toBe(0);
    await user.click(screen.getByRole("button", { name: "Compactar" }));
    await waitFor(() => expect(s.count("conversation_compact")).toBe(1));
  });
});

describe("agent-made extensions (017)", () => {
  it("'Create with AI' starts a new Task conversation and sends the request (AC-005)", async () => {
    const bridge = await freshApp({ signedIn: true });
    const s = spy(bridge);
    render(<OverlayApp />);
    await screen.findByRole("combobox");
    await act(async () => {
      await bridge.invoke("agent_task", { text: "$aura-criar-skill Crie uma Skill do Aura para: atas", mode: "task" });
    });
    await waitFor(() => expect(s.sent()).toEqual(["$aura-criar-skill Crie uma Skill do Aura para: atas"]));
    const start = s.calls.find((c) => c.cmd === "conversation_start")!.args.options as { mode: { mode: string } };
    expect(start.mode.mode).toBe("task");
  });

  it("asks before Aura saves an extension and shows what will be saved", async () => {
    const bridge = await freshApp({ signedIn: true });
    const s = spy(bridge);
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = await screen.findByRole("combobox");
    await user.type(box, "crie o comando{Enter}");
    await waitFor(() => expect(s.sent()).toHaveLength(1));
    const { useSession } = await import("./session");
    const threadId = useSession.getState().threadId!;
    act(() =>
      bridge.emitLocal!("aura://event", {
        channel: "conversation",
        event: {
          type: "userInputRequested", threadId, requestId: "req_tool", autoResolveMs: null, source: "aura",
          prompt: {
            _meta: { codex_approval_kind: "mcp_tool_call", tool_params_display: [{ display_name: "name", name: "name", value: "formal" }, { display_name: "template", name: "template", value: "Reescreva formal: {texto}" }] },
            message: 'Allow the aura MCP server to run tool "quick_command_save"?', serverName: "aura", requestedSchema: { type: "object", properties: {} },
          },
        },
      }),
    );
    const card = await screen.findByRole("group", { name: "Salvar este comando rápido?" });
    expect(within(card).getByText("Reescreva formal: {texto}")).toBeInTheDocument();
    await user.click(within(card).getByRole("button", { name: "Permitir" }));
    const respond = s.calls.find((c) => c.cmd === "conversation_respond")!;
    expect(respond.args).toMatchObject({ requestId: "req_tool", decision: { type: "answer", content: {} } });
    expect(await within(card).findByText("Aceito")).toBeInTheDocument();
  });
});
