import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { freshApp } from "../test/harness";
import { setBridge } from "../ipc/bridge";
import { OverlayApp } from "./OverlayApp";
import { useApp } from "../state/app";
import { filterMenu, insertAtCaret } from "./InputBar";

describe("Overlay", () => {
  it("does not activate the button with Ctrl+Space while the OS shortcut handles it", async () => {
    const bridge = await freshApp({ signedIn: true });
    const commands: string[] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd.startsWith("ptt_")) commands.push(cmd);
      return bridge.invoke<R>(cmd, args);
    } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    (await screen.findByRole("button", { name: "Ditar" })).focus();
    await user.keyboard("{Control>} {/Control}");
    expect(commands).toEqual([]);
  });

  it("allows another dictation attempt after the audio start fails", async () => {
    const bridge = await freshApp({ signedIn: true });
    let attempts = 0;
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd === "ptt_press" && ++attempts === 1) throw { code: "audio", message: "QA microphone unavailable" };
      return bridge.invoke<R>(cmd, args);
    } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const mic = await screen.findByRole("button", { name: "Ditar" });
    await user.click(mic);
    await screen.findByText("QA microphone unavailable");
    expect(mic).toHaveAttribute("aria-disabled", "false");
    await user.click(mic);
    expect(attempts).toBe(2);
    expect(mic).toHaveAttribute("aria-pressed", "true");
  });

  it.each(["listening", "partial"])("Escape cancels dictation while %s without inserting text", async (state) => {
    const bridge = await freshApp({ signedIn: true });
    const commands: string[] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd.startsWith("ptt_")) commands.push(cmd);
      return bridge.invoke<R>(cmd, args);
    } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Ditar" }));
    if (state === "partial") await act(async () => { bridge.emitLocal!("aura://event", { channel: "voice", event: { state: "partial", text: "rascunho parcial" } }); });
    await user.keyboard("{Escape}");
    await waitFor(() => expect(commands).toEqual(["ptt_press", "ptt_cancel"]));
    expect(screen.getByRole("combobox")).toHaveValue("");
  });

  it("does not release before a pending microphone start completes", async () => {
    const bridge = await freshApp({ signedIn: true });
    let complete!: () => void;
    const started = new Promise<void>((resolve) => { complete = resolve; });
    const commands: string[] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd.startsWith("ptt_")) commands.push(cmd);
      if (cmd === "ptt_press") await started;
      return bridge.invoke<R>(cmd, args);
    } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const mic = await screen.findByRole("button", { name: "Ditar" });
    await user.click(mic);
    expect(mic).toHaveAttribute("aria-disabled", "true");
    await user.click(mic);
    expect(commands).toEqual(["ptt_press"]);
    await act(async () => { complete(); await started; });
    await waitFor(() => expect(mic).toHaveAttribute("aria-disabled", "false"));
    expect(mic).toHaveAttribute("aria-pressed", "true");
    await user.click(mic);
    await waitFor(() => expect(commands).toEqual(["ptt_press", "ptt_release"]));
  });

  it("activates dictation with Enter and finishes with Space", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const mic = await screen.findByRole("button", { name: "Ditar" });
    mic.focus();
    await user.keyboard("{Enter}");
    expect(mic).toHaveAttribute("aria-pressed", "true");
    await user.keyboard(" ");
    await waitFor(() => expect(mic).toHaveAttribute("aria-pressed", "false"));
    expect(screen.getByRole("combobox")).toHaveValue("texto ditado de exemplo");
  });

  it("toggles dictation with two clicks without releasing on pointer leave", async () => {
    const bridge = await freshApp({ signedIn: true });
    const commands: string[] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd.startsWith("ptt_")) commands.push(cmd);
      return bridge.invoke<R>(cmd, args);
    } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const mic = await screen.findByRole("button", { name: /Segurar para falar|Ditar/ });
    await user.click(mic);
    expect(commands).toEqual(["ptt_press"]);
    expect(mic).toHaveAttribute("aria-pressed", "true");
    const { fireEvent } = await import("@testing-library/react");
    fireEvent.pointerLeave(mic);
    expect(commands).toEqual(["ptt_press"]);
    await user.click(mic);
    await waitFor(() => expect(commands).toEqual(["ptt_press", "ptt_release"]));
    expect(await screen.findByRole("combobox")).toHaveValue("texto ditado de exemplo");
  });

  it.each(["ptBr", "en"])("does not force the UI language %s when releasing dictation", async (language) => {
    const bridge = await freshApp({ signedIn: true });
    await bridge.invoke("settings_update", { patch: { language } });
    const released: unknown[] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd === "ptt_release") released.push(args.language);
      return bridge.invoke<R>(cmd, args);
    } });
    render(<OverlayApp />);
    const user = userEvent.setup();
    await user.click(await screen.findByRole("button", { name: language === "ptBr" ? "Ditar" : "Dictate" }));
    await user.click(await screen.findByRole("button", { name: language === "ptBr" ? "Concluir ditado" : "Finish dictation" }));
    await waitFor(() => expect(released).toEqual([null]));
  });

  it("sends equal dictations once for each completed recording", async () => {
    const bridge = await freshApp({ signedIn: true });
    await bridge.invoke("settings_update", { patch: { sendAfterDictation: true } });
    const sent: string[] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd === "conversation_send") sent.push((args.request as { text: string }).text);
      return bridge.invoke<R>(cmd, args);
    } });
    render(<OverlayApp />);
    const box = await screen.findByRole("combobox");
    for (let cycle = 0; cycle < 2; cycle++) {
      await act(async () => {
        bridge.emitLocal!("aura://event", { channel: "voice", event: { state: "listening" } });
        bridge.emitLocal!("aura://event", { channel: "voice", event: { state: "done", text: "mesma frase" } });
      });
      await waitFor(() => expect(sent).toHaveLength(cycle + 1));
      const { useSession } = await import("./session");
      await waitFor(() => expect(useSession.getState().sending).toBe(false));
      await act(async () => {
        bridge.emitLocal!("aura://event", { channel: "voice", event: { state: "done", text: "mesma frase" } });
      });
    }
    expect(sent).toEqual(["mesma frase", "mesma frase"]);
    expect(box).toHaveValue("");
  });

  it("keeps an equal transcript after clearing the draft between recordings", async () => {
    const bridge = await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = await screen.findByRole("combobox");
    const cycle = async () => {
      await act(async () => {
        bridge.emitLocal!("aura://event", { channel: "voice", event: { state: "listening" } });
        bridge.emitLocal!("aura://event", { channel: "voice", event: { state: "done", text: "mesma frase" } });
      });
    };
    await cycle();
    expect(box).toHaveValue("mesma frase");
    await user.clear(box);
    await cycle();
    expect(box).toHaveValue("mesma frase");
  });

  it("inserts identical dictation in each cycle but ignores a duplicate terminal event", async () => {
    const bridge = await freshApp({ signedIn: true });
    render(<OverlayApp />);
    const box = await screen.findByRole("combobox");
    const cycle = async () => {
      await act(async () => {
        bridge.emitLocal!("aura://event", { channel: "voice", event: { state: "listening" } });
        bridge.emitLocal!("aura://event", { channel: "voice", event: { state: "done", text: "mesma frase" } });
      });
    };
    await cycle();
    expect(box).toHaveValue("mesma frase");
    await cycle();
    expect(box).toHaveValue("mesma frase mesma frase");
    await act(async () => {
      bridge.emitLocal!("aura://event", { channel: "voice", event: { state: "done", text: "mesma frase" } });
    });
    expect(box).toHaveValue("mesma frase mesma frase");
  });

  it("does not complete a cancelled mock login when its delayed response arrives", async () => {
    const bridge = await freshApp({ signedIn: false, tick: 25 });
    const user = userEvent.setup();
    render(<OverlayApp />);
    let pending: Promise<unknown>;
    await act(async () => { pending = bridge.invoke("auth_login"); });
    await user.click(screen.getByRole("button", { name: "Cancelar" }));
    await act(async () => { await pending; });
    expect(screen.getByRole("button", { name: "Continuar com ChatGPT" })).toBeEnabled();
    expect(screen.queryByRole("dialog", { name: "Você está usando seu plano ChatGPT" })).not.toBeInTheDocument();
    const { useApp } = await import("../state/app");
    expect(useApp.getState().auth?.active).toBeNull();
  });

  it("clears cancelled login progress and returns to the initial card without an error", async () => {
    const bridge = await freshApp({ signedIn: false });
    render(<OverlayApp />);
    await screen.findByRole("button", { name: "Continuar com ChatGPT" });
    await act(async () => {
      bridge.emitLocal?.("aura://event", { channel: "login", event: { state: "waitingBrowser", authorizeUrl: "http://127.0.0.1/qa" } });
    });
    await screen.findByRole("button", { name: "Cancelar" });
    await act(async () => { await bridge.invoke("auth_cancel"); });
    expect(await screen.findByRole("button", { name: "Continuar com ChatGPT" })).toBeEnabled();
    expect(screen.queryByText(/Não foi possível entrar/)).not.toBeInTheDocument();
    const { useApp } = await import("../state/app");
    expect(useApp.getState().login).toBeNull();
  });

  it("an open slash menu refreshes built-in prompts when language changes", async () => {
    const bridge = await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.type(await screen.findByRole("combobox"), "/tldr");
    await screen.findByText("Resuma em até três frases, direto ao ponto:");
    await act(async () => { await bridge.invoke("settings_update", { patch: { language: "en" } }); });
    expect(await screen.findByText("Summarize in up to three sentences, straight to the point:")).toBeInTheDocument();
  });

  it("English uses localized accessible names for context and suggestions", async () => {
    const bridge = await freshApp({ signedIn: true });
    await bridge.invoke("settings_update", { patch: { language: "en" } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = await screen.findByRole("combobox");
    box.focus();
    await user.keyboard("{Control>}{Shift>}s{/Shift}{/Control}");
    expect(await screen.findByRole("list", { name: "Context" })).toBeInTheDocument();
    await user.type(box, "/");
    expect(await screen.findByRole("listbox", { name: "Suggestions" })).toBeInTheDocument();
  });

  it("a provider saved elsewhere unlocks the existing Overlay without remounting", async () => {
    const bridge = await freshApp({ signedIn: false });
    render(<OverlayApp />);
    await screen.findByRole("button", { name: /Continuar com ChatGPT/ });
    await act(async () => {
      await bridge.invoke("providers_save", { draft: { name: "QA local", preset: "ollama" }, credential: null });
      bridge.emitLocal!("aura://event", { channel: "providersChanged", event: {} });
    });
    expect(await screen.findByRole("combobox")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Continuar com ChatGPT/ })).not.toBeInTheDocument();
  });

  it("removing the last provider restores sign-in in the existing Overlay", async () => {
    const bridge = await freshApp({ signedIn: false });
    render(<OverlayApp />);
    await screen.findByRole("button", { name: /Continuar com ChatGPT/ });
    let provider: any;
    await act(async () => {
      provider = await bridge.invoke("providers_save", { draft: { name: "QA local", preset: "ollama" }, credential: null });
    });
    await screen.findByRole("combobox");
    await act(async () => { await bridge.invoke("providers_remove", { id: provider.id }); });
    expect(await screen.findByRole("button", { name: /Continuar com ChatGPT/ })).toBeInTheDocument();
  });

  it("discovered provider models reach the already open model picker", async () => {
    const bridge = await freshApp({ signedIn: false });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await screen.findByRole("button", { name: /Continuar com ChatGPT/ });
    let provider: any;
    await act(async () => {
      provider = await bridge.invoke("providers_save", { draft: { name: "QA local", preset: "ollama" }, credential: null });
    });
    await screen.findByRole("combobox");
    await user.keyboard("{Control>}{ArrowDown}{/Control}");
    await user.click(screen.getByRole("button", { name: /^Modelo .*modo/ }));
    await user.click(screen.getByRole("menuitemradio", { name: "QA local" }));
    await act(async () => { await bridge.invoke("providers_test", { id: provider.id }); });
    expect(await screen.findByRole("menuitemradio", { name: "Llama 3.3 70B" })).toBeInTheDocument();
  });

  it("selects the first keyless provider when no ChatGPT account exists", async () => {
    const bridge = await freshApp({ signedIn: false });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await screen.findByRole("button", { name: /Continuar com ChatGPT/ });
    await act(async () => {
      await bridge.invoke("providers_save", { draft: { name: "QA local", preset: "ollama" }, credential: null });
    });
    await screen.findByRole("combobox");
    await user.keyboard("{Control>}{ArrowDown}{/Control}");
    await user.click(screen.getByRole("button", { name: /^Modelo .*modo/ }));
    expect(screen.getByRole("menuitemradio", { name: "QA local" })).toHaveAttribute("aria-checked", "true");
  });

  it("an older catalog response cannot undo a newly saved provider", async () => {
    const bridge = await freshApp({ signedIn: false });
    let finishOld!: (value: unknown) => void;
    const old = new Promise((resolve) => { finishOld = resolve; });
    let first = true;
    setBridge({ ...bridge, invoke: async <T,>(cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "providers_list" && first) {
        first = false;
        return await old as T;
      }
      return bridge.invoke<T>(cmd, args);
    } });
    render(<OverlayApp />);
    await screen.findByRole("button", { name: /Continuar com ChatGPT/ });
    await act(async () => {
      await bridge.invoke("providers_save", { draft: { name: "QA local", preset: "ollama" }, credential: null });
    });
    await screen.findByRole("combobox");
    await act(async () => { finishOld([]); });
    expect(screen.getByRole("combobox")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Continuar com ChatGPT/ })).not.toBeInTheDocument();
  });

  it("shows a provider connection error without removing the conversation input", async () => {
    const bridge = await freshApp({ signedIn: false });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await screen.findByRole("button", { name: /Continuar com ChatGPT/ });
    await act(async () => {
      await bridge.invoke("providers_save", { draft: { name: "QA local", preset: "ollama" }, credential: null });
      bridge.state.providers[0].status = "error";
      bridge.emitLocal!("aura://event", { channel: "providersChanged", event: {} });
    });
    const box = await screen.findByRole("combobox");
    await user.keyboard("{Control>}{ArrowDown}{/Control}");
    await user.click(screen.getByRole("button", { name: /^Modelo .*modo/ }));
    const row = await screen.findByRole("menuitemradio", { name: /QA local/ });
    expect(within(row).getByText("Erro")).toBeInTheDocument();
    expect(box).toBeInTheDocument();
  });

  it("first run shows the ChatGPT sign-in and the plan-usage notice", async () => {
    await freshApp({ signedIn: false });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const button = await screen.findByRole("button", { name: /Continuar com ChatGPT/ });
    await user.click(button);
    expect(await screen.findByRole("dialog", { name: /plano ChatGPT/ })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Entendi" }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(screen.getByRole("combobox")).toBeInTheDocument();
  });

  it("sends with Enter, streams the answer and expands", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = await screen.findByRole("combobox");
    await user.type(box, "explique este erro{Enter}");
    expect(await screen.findByText("explique este erro")).toBeInTheDocument();
    await waitFor(() => expect(screen.getByText(/tipos incompatíveis/)).toBeInTheDocument());
    // Tool calls appear as collapsible cards.
    expect(screen.getByText("aura.active_window_info")).toBeInTheDocument();
    // Header only exists in the expanded state.
    expect(screen.getByRole("button", { name: "Nova conversa" })).toBeInTheDocument();
  });

  it("an app profile selects its provider and default model for a new conversation (QA-036)", async () => {
    const bridge = await freshApp({ signedIn: true });
    const p = await bridge.invoke<{ id: string }>("providers_save", { draft: { name: "QA Local", preset: "ollama", baseUrl: "http://127.0.0.1:9/v1" }, credential: null });
    bridge.state.profiles = [{ id: "prof_1", name: "Code", processPattern: "code.exe", titleGlob: null, instructions: "", attachScreen: false, defaultMode: null, defaultModel: `aura-${p.id}::qwen3:8b` }];
    const { useSession } = await import("./session");
    await useSession.getState().applyProfile();
    expect(useSession.getState()).toMatchObject({ provider: `aura-${p.id}`, model: "qwen3:8b" });
    // Older profiles stored a bare ChatGPT model id.
    useSession.setState({ provider: `aura-${p.id}`, model: null });
    bridge.state.profiles = [{ ...bridge.state.profiles[0], defaultModel: "gpt-5.5-mini" }];
    await useSession.getState().applyProfile();
    expect(useSession.getState()).toMatchObject({ provider: "aura-chatgpt-plan", model: "gpt-5.5-mini" });
  });

  it("Ctrl+Shift+L reads the last answer and Ctrl+Shift+Enter inserts it into the previous app (QA-035)", async () => {
    const bridge = await freshApp({ signedIn: true });
    const calls: [string, unknown][] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd === "speak" || cmd === "insert_into_app" || cmd === "conversation_steer") calls.push([cmd, args.text]);
      return bridge.invoke<R>(cmd, args);
    } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = await screen.findByRole("combobox");
    await user.type(box, "explique este erro{Enter}");
    await waitFor(() => expect(screen.getByText(/tipos incompatíveis/)).toBeInTheDocument());
    await user.keyboard("{Control>}{Shift>}L{/Shift}{/Control}");
    await waitFor(() => expect(calls.filter(([c]) => c === "speak")).toHaveLength(1));
    expect(String(calls[0][1])).toMatch(/tipos incompatíveis/);
    // Again: stops instead of reading twice.
    await user.keyboard("{Control>}{Shift>}L{/Shift}{/Control}");
    expect(calls.filter(([c]) => c === "speak")).toHaveLength(1);

    await user.click(box);
    await user.keyboard("{Control>}{Shift>}{Enter}{/Shift}{/Control}");
    await waitFor(() => expect(calls.filter(([c]) => c === "insert_into_app")).toHaveLength(1));
    const inserted = String(calls.find(([c]) => c === "insert_into_app")![1]);
    expect(inserted).toMatch(/tipos incompatíveis/);
    // Not a steer/new line in the input.
    expect(calls.some(([c]) => c === "conversation_steer")).toBe(false);
    expect(box).toHaveValue("");

    // A selected code block is inserted instead of the whole answer.
    const code = document.querySelector("pre code");
    expect(code).not.toBeNull();
    const range = document.createRange();
    range.selectNodeContents(code!);
    window.getSelection()!.removeAllRanges();
    window.getSelection()!.addRange(range);
    await user.keyboard("{Control>}{Shift>}{Enter}{/Shift}{/Control}");
    await waitFor(() => expect(calls.filter(([c]) => c === "insert_into_app")).toHaveLength(2));
    expect(calls.filter(([c]) => c === "insert_into_app")[1][1]).toBe(code!.textContent);
  });

  it("asks once before reading an answer with a cloud voice (QA-034)", async () => {
    const bridge = await freshApp({ signedIn: true });
    const p = await bridge.invoke<{ id: string }>("providers_save", { draft: { name: "Voz QA", preset: "custom", baseUrl: "http://127.0.0.1:9/v1" }, credential: null });
    await bridge.invoke("settings_update", { patch: { ttsProvider: p.id } });
    const calls: string[] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd === "speak" || cmd === "speech_consent") calls.push(cmd);
      return bridge.invoke<R>(cmd, args);
    } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.type(await screen.findByRole("combobox"), "explique este erro{Enter}");
    await waitFor(() => expect(screen.getByText(/tipos incompatíveis/)).toBeInTheDocument());
    await user.click(screen.getByRole("button", { name: "Ouvir" }));
    const dialog = await screen.findByRole("dialog", { name: "Enviar a resposta para Voz QA?" });
    expect(calls).toEqual(["speak"]);
    await user.click(within(dialog).getByRole("button", { name: "Enviar e ouvir" }));
    await waitFor(() => expect(calls).toEqual(["speak", "speech_consent", "speak"]));
    expect(bridge.state.settings.ttsCloudConsent).toEqual([p.id]);
    expect(screen.queryByRole("dialog", { name: "Enviar a resposta para Voz QA?" })).toBeNull();
  });

  it("reads finished answers aloud when auto-read is on (QA-034)", async () => {
    const bridge = await freshApp({ signedIn: true });
    await bridge.invoke("settings_update", { patch: { autoRead: true } });
    const spoken: string[] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd === "speak") spoken.push(String(args.text));
      return bridge.invoke<R>(cmd, args);
    } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.type(await screen.findByRole("combobox"), "explique este erro{Enter}");
    await waitFor(() => expect(screen.getByText(/tipos incompatíveis/)).toBeInTheDocument());
    await waitFor(() => expect(spoken).toHaveLength(1));
    expect(spoken[0]).toMatch(/tipos incompatíveis/);
  });

  it("attaches the last minutes of the recent buffer from the context menu (QA-033)", async () => {
    const bridge = await freshApp({ signedIn: true });
    bridge.state.privacy = {
      ...bridge.state.privacy,
      screen: { mode: { type: "recentBuffer", minutes: 5 }, agent: "never" },
      mic: { mode: { type: "recentBuffer", minutes: 5 }, agent: "never" },
      systemAudio: { mode: { type: "off" }, agent: "never" },
    };
    const calls: unknown[] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd === "context_attach_recent") calls.push(args);
      return bridge.invoke<R>(cmd, args);
    } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Adicionar contexto" }));
    await user.click(screen.getByRole("option", { name: /@recente/ }));
    const dialog = screen.getByRole("dialog", { name: "Anexar buffer recente" });
    const minutes = within(dialog).getByRole("spinbutton", { name: "Últimos minutos" });
    expect(minutes).toHaveValue(2);
    expect(minutes).toHaveAttribute("max", "5");
    expect(within(dialog).getByRole("checkbox", { name: "Tela" })).toBeChecked();
    const audio = within(dialog).getByRole("combobox", { name: "Áudio" });
    expect(audio).toHaveValue("mic");
    // System audio has no buffer: not offered.
    expect(within(audio).queryByRole("option", { name: "Sistema" })).toBeNull();
    await user.selectOptions(audio, "mic");
    await user.click(within(dialog).getByRole("button", { name: "Anexar" }));
    await waitFor(() => expect(calls).toEqual([{ tray: "draft", threadId: null, clip: { minutes: 2, screen: true, audio: "mic" } }]));
    expect(await screen.findByText(/Últimos 2 min · tela \+ microfone/)).toBeInTheDocument();
    expect(screen.queryByRole("dialog", { name: "Anexar buffer recente" })).toBeNull();
  });

  it("explains how to turn the recent buffer on when no source keeps one (QA-033)", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Adicionar contexto" }));
    await user.click(screen.getByRole("option", { name: /@recente/ }));
    const dialog = screen.getByRole("dialog", { name: "Anexar buffer recente" });
    expect(within(dialog).getByText("Nenhuma fonte está com o Buffer recente ligado. Ligue em Configurações › Privacidade.")).toBeInTheDocument();
    expect(within(dialog).getByRole("button", { name: "Anexar" })).toBeDisabled();
  });

  it("opens the conversation Settings asked for from the access log (QA-031)", async () => {
    const bridge = await freshApp({ signedIn: true });
    const opened: unknown[] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd === "conversation_open") opened.push(args);
      return bridge.invoke<R>(cmd, args);
    } });
    render(<OverlayApp />);
    await screen.findByRole("button", { name: "Expandir" });
    act(() => useApp.getState().handle({ channel: "openConversation", event: { threadId: "thr_log" } }));
    await waitFor(() => expect(opened).toEqual([{ threadId: "thr_log" }]));
    expect(await screen.findByRole("button", { name: "Nova conversa" })).toBeInTheDocument();
  });

  it("renames a conversation inline from History; Escape and blank names change nothing", async () => {
    const bridge = await freshApp({ signedIn: true });
    bridge.state.history = [{ id: "thr_trip", title: "Plano de viagem", preview: "…", updatedAt: 1, pinned: false, archived: false }];
    const renames: unknown[] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd === "conversation_rename") renames.push(args);
      return bridge.invoke<R>(cmd, args);
    } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Expandir" }));
    await user.click(screen.getByRole("button", { name: "Histórico" }));
    const panel = screen.getByRole("complementary", { name: "Histórico" });
    await user.click(await within(panel).findByRole("button", { name: "Renomear Plano de viagem" }));
    let field = within(panel).getByRole("textbox", { name: "Novo nome da conversa" });
    expect(field).toHaveValue("Plano de viagem");
    await user.clear(field);
    await user.type(field, "Ignorado{Escape}");
    expect(renames).toEqual([]);
    expect(within(panel).getByText("Plano de viagem")).toBeInTheDocument();
    expect(screen.getByRole("complementary", { name: "Histórico" })).toBeInTheDocument();

    await user.click(within(panel).getByRole("button", { name: "Renomear Plano de viagem" }));
    field = within(panel).getByRole("textbox", { name: "Novo nome da conversa" });
    await user.clear(field);
    await user.type(field, "   {Enter}");
    expect(renames).toEqual([]);

    await user.click(within(panel).getByRole("button", { name: "Renomear Plano de viagem" }));
    field = within(panel).getByRole("textbox", { name: "Novo nome da conversa" });
    await user.clear(field);
    await user.type(field, "  Roteiro Lisboa {Enter}");
    expect(renames).toEqual([{ threadId: "thr_trip", name: "Roteiro Lisboa" }]);
    expect(await within(panel).findByText("Roteiro Lisboa")).toBeInTheDocument();
  });

  it("pages through the whole History with Load more and restarts paging on a new search", async () => {
    const bridge = await freshApp({ signedIn: true });
    const pad = (n: number) => String(n).padStart(3, "0");
    // Conversa 120 is the newest.
    bridge.state.history = Array.from({ length: 120 }, (_, i) => ({ id: `thr_${pad(i + 1)}`, title: `Conversa ${pad(i + 1)}`, preview: "…", updatedAt: i + 1, pinned: false, archived: false }));
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Expandir" }));
    await user.click(screen.getByRole("button", { name: "Histórico" }));
    const panel = screen.getByRole("complementary", { name: "Histórico" });
    const rows = () => within(panel).queryAllByText(/^Conversa \d{3}$/);
    await waitFor(() => expect(rows()).toHaveLength(50));
    expect(within(panel).queryByText("Conversa 001")).toBeNull();
    await user.click(within(panel).getByRole("button", { name: "Carregar mais" }));
    await waitFor(() => expect(rows()).toHaveLength(100));
    await user.click(within(panel).getByRole("button", { name: "Carregar mais" }));
    await waitFor(() => expect(rows()).toHaveLength(120));
    expect(within(panel).getByText("Conversa 001")).toBeInTheDocument();
    expect(within(panel).queryByRole("button", { name: "Carregar mais" })).toBeNull();
    expect(new Set(rows().map((r) => r.textContent)).size).toBe(120);

    await user.type(within(panel).getByPlaceholderText("Buscar conversas"), "Conversa 0");
    await waitFor(() => expect(rows()).toHaveLength(50));
    expect(within(panel).getByRole("button", { name: "Carregar mais" })).toBeInTheDocument();
  });

  it("/compactar compacts the conversation locally and shows a compaction item", async () => {
    const bridge = await freshApp({ signedIn: true });
    const calls: string[] = [];
    const sent: string[] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd === "conversation_compact") calls.push(String(args.threadId));
      if (cmd === "conversation_send") sent.push((args.request as { text: string }).text);
      return bridge.invoke<R>(cmd, args);
    } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = await screen.findByRole("combobox");
    await user.type(box, "/comp");
    expect(screen.getByRole("option", { name: /\/compactar/ })).toBeInTheDocument();
    await user.clear(box);
    // Without a conversation there is nothing to compact.
    await user.type(box, "/compactar{Enter}");
    expect(calls).toEqual([]);
    expect(sent).toEqual([]);
    expect(await screen.findByText("Nada para compactar ainda.")).toBeInTheDocument();

    await user.type(box, "primeira mensagem{Enter}");
    await waitFor(() => expect(sent).toEqual(["primeira mensagem"]));
    const { useSession } = await import("./session");
    await waitFor(() => expect(useSession.getState().sending).toBe(false));
    await user.type(box, "/compactar{Enter}");
    await waitFor(() => expect(calls).toEqual([useSession.getState().threadId]));
    expect(sent).toEqual(["primeira mensagem"]);
    expect(await screen.findByText("Conversa compactada")).toBeInTheDocument();
    expect(box).toHaveValue("");
  });

  it("Ctrl+V with an image attaches it as a chip; text paste stays text", async () => {
    const { fireEvent } = await import("@testing-library/react");
    const bridge = await freshApp({ signedIn: true });
    const pasted: Record<string, unknown>[] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd === "attach_clipboard_image") pasted.push(args);
      return bridge.invoke<R>(cmd, args);
    } });
    render(<OverlayApp />);
    const box = await screen.findByRole("combobox");
    const file = (type: string, bytes: number[]) => new File([new Uint8Array(bytes)], "image", { type });
    const clipboard = (items: { kind: string; type: string; file?: File }[], text = "") => ({
      items: items.map((i) => ({ kind: i.kind, type: i.type, getAsFile: () => i.file ?? null })),
      getData: () => text,
    });
    fireEvent.paste(box, { clipboardData: clipboard([{ kind: "file", type: "image/png", file: file("image/png", [137, 80, 78, 71]) }]) });
    // "iVBORw==" is base64 of the bytes 137 80 78 71 (computed independently).
    await waitFor(() => expect(pasted).toEqual([{ tray: "draft", threadId: null, mime: "image/png", data: "iVBORw==" }]));
    expect(await screen.findByText(/^Imagem colada/)).toBeInTheDocument();

    fireEvent.paste(box, { clipboardData: clipboard([{ kind: "file", type: "image/bmp", file: file("image/bmp", [66, 77]) }]) });
    expect(await screen.findByText("Formato de imagem não suportado: image/bmp")).toBeInTheDocument();
    expect(pasted).toHaveLength(1);

    const textPaste = fireEvent.paste(box, { clipboardData: clipboard([{ kind: "string", type: "text/plain" }], "olá") });
    expect(textPaste).toBe(true); // not prevented: the browser inserts the text
    expect(pasted).toHaveLength(1);
  });

  it("the / menu offers only enabled skills", async () => {
    const bridge = await freshApp({ signedIn: true });
    await bridge.invoke("skills_create", { name: "revisar-contrato", description: "Revisa contratos", body: "x" });
    await bridge.invoke("skills_create", { name: "resumir-ata", description: "Resume atas", body: "y" });
    await bridge.invoke("skills_set_enabled", { path: "aura/resumir-ata/SKILL.md", enabled: false });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.type(await screen.findByRole("combobox"), "/r");
    expect(await screen.findByRole("option", { name: /\/revisar-contrato/ })).toBeInTheDocument();
    expect(screen.queryByRole("option", { name: /\/resumir-ata/ })).toBeNull();
  });

  it("opening files replaces History and preserves the draft", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = await screen.findByRole("combobox");
    await user.type(box, "teste de painéis{Enter}");
    await screen.findByRole("button", { name: "Arquivos e alterações" });
    await user.type(box, "rascunho preservado");
    await user.click(screen.getByRole("button", { name: "Histórico" }));
    expect(screen.getByRole("complementary", { name: "Histórico" })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Arquivos e alterações" }));
    expect(screen.getByRole("complementary", { name: "Arquivos e alterações" })).toBeInTheDocument();
    expect(screen.queryByRole("complementary", { name: "Histórico" })).not.toBeInTheDocument();
    expect(box).toHaveValue("rascunho preservado");
  });

  it("Ctrl+H replaces Files with History and closing History preserves the draft", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = await screen.findByRole("combobox");
    await user.type(box, "teste inverso{Enter}");
    await user.click(await screen.findByRole("button", { name: "Arquivos e alterações" }));
    await user.type(box, "outro rascunho");
    await user.keyboard("{Control>}h{/Control}");
    expect(screen.getByRole("complementary", { name: "Histórico" })).toBeInTheDocument();
    expect(screen.queryByRole("complementary", { name: "Arquivos e alterações" })).not.toBeInTheDocument();
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("complementary")).not.toBeInTheDocument();
    expect(box).toHaveValue("outro rascunho");
  });

  it("Shift+Enter inserts a newline instead of sending", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = (await screen.findByRole("combobox")) as HTMLTextAreaElement;
    await user.type(box, "linha 1{Shift>}{Enter}{/Shift}linha 2");
    expect(box.value).toBe("linha 1\nlinha 2");
    expect(screen.queryByText("linha 1")).not.toBeInTheDocument();
  });

  it("approval card accepts with the A key", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.type(await screen.findByRole("combobox"), "/aprovar{Enter}");
    const card = await screen.findByRole("group", { name: "Executar comando" });
    expect(within(card).getByText("pip install requests")).toBeInTheDocument();
    await waitFor(() => expect(card).toHaveFocus());
    await user.keyboard("a");
    // The card says what was decided, not just "answered".
    await waitFor(() => expect(within(card).getByText("Aceito")).toBeInTheDocument());
    await waitFor(() => expect(screen.getByText(/instalei/)).toBeInTheDocument());
  });

  it("agent questions are answered with a form", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.type(await screen.findByRole("combobox"), "/pergunta{Enter}");
    const card = await screen.findByRole("group", { name: "O agente tem uma pergunta" });
    await user.click(within(card).getByLabelText("TypeScript"));
    await user.click(within(card).getByRole("button", { name: "Responder" }));
    await waitFor(() => expect(screen.getByText("Certo, vou usar TypeScript.")).toBeInTheDocument());
  });

  it("shows context usage after a turn", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.type(await screen.findByRole("combobox"), "oi{Enter}");
    expect(await screen.findByRole("button", { name: /% do contexto usado/ })).toBeInTheDocument();
  });

  it("slash menu lists quick commands and completes with Tab", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = (await screen.findByRole("combobox")) as HTMLTextAreaElement;
    await user.type(box, "/tra");
    const list = await screen.findByRole("listbox");
    expect(within(list).getByText("/traduzir")).toBeInTheDocument();
    await user.keyboard("{Tab}");
    expect(box.value).toBe("/traduzir ");
  });

  it("screen capture adds a chip that can be removed", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    const box = await screen.findByRole("combobox");
    box.focus();
    await user.keyboard("{Control>}{Shift>}s{/Shift}{/Control}");
    const chip = await screen.findByText(/Tela · main.rs/);
    expect(chip).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: /Remover Tela/ }));
    await waitFor(() => expect(screen.queryByText(/Tela · main.rs/)).not.toBeInTheDocument());
  });

  it("consent requests from the agent are answered inline", async () => {
    const bridge = await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await screen.findByRole("combobox");
    act(() =>
      bridge.emitLocal!("aura://event", {
        channel: "consent",
        event: { id: "c1", conversation: "u", tool: "screen_capture", source: "screen", reason: "ver o erro", app: "Code.exe" },
      }),
    );
    const dialog = await screen.findByRole("alertdialog", { name: /a tela/ });
    await user.click(within(dialog).getByRole("button", { name: "Permitir uma vez" }));
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
  });

  it("dictation result lands at the caret", async () => {
    const bridge = await freshApp({ signedIn: true });
    render(<OverlayApp />);
    const box = (await screen.findByRole("combobox")) as HTMLTextAreaElement;
    act(() => bridge.emitLocal!("aura://event", { channel: "voice", event: { state: "done", text: "olá mundo" } }));
    await waitFor(() => expect(box.value).toBe("olá mundo"));
  });
});

describe("first run", () => {
  it("walks through privacy, shortcut and voice, then never shows again", async () => {
    const bridge = await freshApp({ signedIn: true });
    bridge.state.settings.onboarded = false;
    const { useApp } = await import("../state/app");
    useApp.getState().setSettings({ ...bridge.state.settings });
    const user = userEvent.setup();
    render(<OverlayApp />);
    expect(await screen.findByText("Quando o agente pode ver sua tela?")).toBeInTheDocument();
    await user.click(screen.getByRole("radio", { name: /Nunca/ }));
    await waitFor(() => expect(useApp.getState().privacy?.screen.agent).toBe("never"));
    await user.click(screen.getByRole("button", { name: "Próximo" }));
    expect(screen.getByText("Abra o Aura de qualquer lugar")).toBeInTheDocument();
    // QA-040: Esc never hides the Overlay; the guide says how to close it.
    expect(screen.getByText("Use o atalho para abrir e fechar o Aura, ou Minimizar para a bandeja. Esc só fecha menus, o histórico e a gravação de voz. O atalho pode ser mudado em Configurações › Atalhos.")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Próximo" }));
    expect(screen.getByText("Fale em vez de digitar")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Concluir" }));
    await waitFor(() => expect(useApp.getState().settings?.onboarded).toBe(true));
    expect(screen.queryByText("Fale em vez de digitar")).not.toBeInTheDocument();
  });
});

describe("input helpers", () => {
  it("inserts with a separating space", () => {
    expect(insertAtCaret("abc", 3, "def")).toEqual(["abc def", 7]);
    expect(insertAtCaret("", 0, "x")).toEqual(["x", 1]);
    expect(insertAtCaret("a b", 2, "c")).toEqual(["a c b", 3]);
  });
  it("filters menu items case-insensitively", () => {
    const items = [{ id: "1", label: "/Traduzir", hint: "", insert: "" }, { id: "2", label: "/tldr", hint: "", insert: "" }];
    expect(filterMenu(items, "tra").map((i) => i.id)).toEqual(["1"]);
  });
});

describe("window behaviour (001 revision 2)", () => {
  function spyCalls(bridge: Awaited<ReturnType<typeof freshApp>>) {
    const calls: string[] = [];
    const invoke = bridge.invoke;
    bridge.invoke = ((cmd: string, args?: Record<string, unknown>) => {
      calls.push(cmd);
      return invoke(cmd, args);
    }) as typeof bridge.invoke;
    return calls;
  }

  it("Esc never hides the Overlay; Minimize to tray does", async () => {
    const bridge = await freshApp({ signedIn: true });
    const calls = spyCalls(bridge);
    const user = userEvent.setup();
    render(<OverlayApp />);
    (await screen.findByRole("combobox")).focus();
    await user.keyboard("{Escape}");
    expect(calls).not.toContain("overlay_hide");
    await user.click(screen.getByRole("button", { name: "Minimizar para a bandeja" }));
    expect(calls).toContain("overlay_hide");
  });

  it("compact Overlay has window controls", async () => {
    await freshApp({ signedIn: true });
    render(<OverlayApp />);
    await screen.findByRole("combobox");
    expect(screen.getByRole("button", { name: "Expandir" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Aparência" })).toBeInTheDocument();
  });

  it("opacity is adjusted from the Overlay itself", async () => {
    await freshApp({ signedIn: true });
    const { useApp } = await import("../state/app");
    const { fireEvent } = await import("@testing-library/react");
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Aparência" }));
    const slider = within(screen.getByRole("dialog", { name: "Aparência" })).getByRole("slider", { name: "Opacidade do Overlay" });
    fireEvent.change(slider, { target: { value: "60" } });
    expect(document.documentElement.style.getPropertyValue("--overlay-opacity")).toBe("0.6");
    await waitFor(() => expect(useApp.getState().settings?.opacity).toBe(0.6), { timeout: 2000 });
  });

  it("model picker offers only the efforts each model supports, shows capabilities and sends the effort", async () => {
    const bridge = await freshApp({ signedIn: true });
    const efforts: unknown[] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd === "conversation_send") efforts.push((args.request as { options?: { effort?: unknown } }).options?.effort ?? null);
      return bridge.invoke<R>(cmd, args);
    } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Expandir" }));
    await user.click(await screen.findByRole("button", { name: /modo Chat/ }));
    const picker = screen.getByRole("dialog", { name: "Modelo e modo" });
    const effortMenu = within(picker).getByRole("menu", { name: "Esforço de raciocínio" });
    // Mock GPT-5.5 (default model): efforts low/medium/high, default medium.
    expect(within(effortMenu).getAllByRole("menuitemradio").map((o) => o.textContent)).toEqual(["Padrão do modelo (médio)", "Baixo", "Médio", "Alto"]);
    const gpt = within(within(picker).getByRole("menu", { name: "Modelo" })).getByRole("menuitemradio", { name: "GPT-5.5" });
    expect(gpt).toHaveTextContent("Imagem");
    expect(gpt).toHaveTextContent("Ferramentas");
    expect(gpt).toHaveTextContent("Raciocínio");
    await user.click(within(effortMenu).getByRole("menuitemradio", { name: "Alto" }));
    await user.keyboard("{Escape}");
    await user.type(screen.getByRole("combobox"), "primeiro{Enter}");
    await waitFor(() => expect(efforts).toEqual(["high"]));

    // GPT-5.5 mini supports low/medium only: "high" falls back to the model default.
    const { useSession } = await import("./session");
    act(() => useSession.getState().setModel("gpt-5.5-mini"));
    expect(useSession.getState().effort).toBeNull();
    await waitFor(() => expect(useSession.getState().sending).toBe(false));
    await user.type(screen.getByRole("combobox"), "segundo{Enter}");
    await waitFor(() => expect(efforts).toEqual(["high", null]));
  });

  it("offers a custom model's own efforts and remembers the effort per model and mode (013)", async () => {
    const bridge = await freshApp({ signedIn: true });
    const { useSession } = await import("./session");
    bridge.state.providers = [{ id: "qa", name: "QA", preset: "custom", wire: "responses", baseUrl: "http://127.0.0.1:9/v1", auth: "none", extraHeaders: {}, credentialHint: null, status: "verified", lastError: null, quirks: { noParallelToolCalls: false, reasoningEffort: false, noStreamOptions: false },
        models: [{ id: "qa-reasoner", displayName: "QA reasoner", contextWindow: null, maxOutput: null, supportsImages: false, supportsTools: true, supportsReasoning: true, estimated: false, manual: true, efforts: ["low", "high", "max"], defaultEffort: "high" }] }];
    useSession.setState({ provider: "aura-qa", model: "qa-reasoner", threadId: null });
    const sent: unknown[] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd === "conversation_send") sent.push((args.request as { options?: { effort?: unknown } }).options?.effort ?? null);
      return bridge.invoke<R>(cmd, args);
    } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Expandir" }));
    await user.click(await screen.findByRole("button", { name: /modo Chat/ }));
    const picker = screen.getByRole("dialog", { name: "Modelo e modo" });
    const efforts = () => within(within(picker).getByRole("menu", { name: "Esforço de raciocínio" })).getAllByRole("menuitemradio");
    const checked = () => efforts().find((o) => o.getAttribute("aria-checked") === "true")?.textContent;
    expect(efforts().map((o) => o.textContent)).toEqual(["Padrão do modelo (alto)", "Baixo", "Alto", "Máximo"]);
    const modes = within(picker).getByRole("menu", { name: "Modo" });

    await user.click(efforts().find((o) => o.textContent === "Baixo")!);
    await waitFor(() => expect(bridge.state.settings.effortPresets["aura-qa::qa-reasoner"]).toEqual({ chat: "low" }));
    await user.click(within(modes).getByRole("menuitemradio", { name: /Tarefa/ }));
    expect(checked()).toBe("Padrão do modelo (alto)");
    await user.click(efforts().find((o) => o.textContent === "Máximo")!);
    await waitFor(() => expect(bridge.state.settings.effortPresets["aura-qa::qa-reasoner"]).toEqual({ chat: "low", task: "max" }));
    await user.click(within(modes).getByRole("menuitemradio", { name: /Chat/ }));
    expect(checked()).toBe("Baixo");
    await user.click(within(modes).getByRole("menuitemradio", { name: /Tarefa/ }));
    expect(checked()).toBe("Máximo");
    await user.keyboard("{Escape}");
    await user.type(screen.getByRole("combobox"), "tarefa{Enter}");
    await waitFor(() => expect(sent).toEqual(["max"]));
  });

  it("switches between Chat, Task and Plan in the middle of a conversation (014)", async () => {
    const bridge = await freshApp({ signedIn: true });
    const modes: unknown[] = [];
    setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
      if (cmd === "conversation_set_mode") modes.push(args.mode);
      return bridge.invoke<R>(cmd, args);
    } });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.type(await screen.findByRole("combobox"), "explique este erro{Enter}");
    await waitFor(() => expect(screen.getByText(/tipos incompatíveis/)).toBeInTheDocument());
    await user.click(await screen.findByRole("button", { name: /modo Chat/ }));
    let picker = screen.getByRole("dialog", { name: "Modelo e modo" });
    await user.click(within(within(picker).getByRole("menu", { name: "Modo" })).getByRole("menuitemradio", { name: /Tarefa/ }));
    await waitFor(() => expect(modes).toEqual([{ mode: "task", granted: [], network: false }]));
    expect(await screen.findByRole("separator", { name: "Modo alterado para Tarefa" })).toBeInTheDocument();
    await user.keyboard("{Escape}");
    await user.click(await screen.findByRole("button", { name: /modo Tarefa/ }));
    picker = screen.getByRole("dialog", { name: "Modelo e modo" });
    await user.click(within(within(picker).getByRole("menu", { name: "Modo" })).getByRole("menuitemradio", { name: /Plano/ }));
    await waitFor(() => expect(modes).toHaveLength(2));
    expect(modes[1]).toEqual({ mode: "plan" });
    expect(screen.getByRole("separator", { name: "Modo alterado para Plano" })).toBeInTheDocument();
    // Choosing the current mode again changes nothing.
    await user.click(within(within(picker).getByRole("menu", { name: "Modo" })).getByRole("menuitemradio", { name: /Plano/ }));
    expect(modes).toHaveLength(2);
    expect(screen.getAllByRole("separator", { name: /Modo alterado/ })).toHaveLength(2);
  });

  it("lists GPT-6.1 Sol in the ChatGPT plan with its efforts up to Max (013)", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Expandir" }));
    await user.click(await screen.findByRole("button", { name: /modo Chat/ }));
    const picker = screen.getByRole("dialog", { name: "Modelo e modo" });
    await user.click(within(within(picker).getByRole("menu", { name: "Modelo" })).getByRole("menuitemradio", { name: "GPT-6.1 Sol" }));
    expect(within(within(picker).getByRole("menu", { name: "Esforço de raciocínio" })).getAllByRole("menuitemradio").map((o) => o.textContent)).toEqual([
      "Padrão do modelo (médio)", "Baixo", "Médio", "Alto", "Muito alto", "Máximo",
    ]);
  });

  it("hides the effort choice for a provider model without reasoning and shows its capabilities", async () => {
    const bridge = await freshApp({ signedIn: true });
    const { useSession } = await import("./session");
    bridge.state.providers = [{ id: "local", name: "Local", preset: "ollama", wire: "chat", baseUrl: "http://127.0.0.1:11434/v1", auth: "none", extraHeaders: {}, credentialHint: null, status: "verified", lastError: null, quirks: { noParallelToolCalls: false, reasoningEffort: false, noStreamOptions: false },
        models: [{ id: "llava", displayName: "LLaVA", contextWindow: null, maxOutput: null, supportsImages: true, supportsTools: false, supportsReasoning: false, estimated: true }] }];
    useSession.setState({ provider: "aura-local", model: "llava", threadId: null });
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Expandir" }));
    await user.click(await screen.findByRole("button", { name: /modo Chat/ }));
    const picker = screen.getByRole("dialog", { name: "Modelo e modo" });
    expect(within(picker).queryByRole("menu", { name: "Esforço de raciocínio" })).toBeNull();
    expect(within(picker).getByText("Este modelo não tem esforço de raciocínio configurável.")).toBeInTheDocument();
    const llava = within(within(picker).getByRole("menu", { name: "Modelo" })).getByRole("menuitemradio", { name: "LLaVA", description: "Imagem" });
    expect(llava).toHaveTextContent("Imagem");
    expect(llava).not.toHaveTextContent("Ferramentas");
    expect(llava).not.toHaveTextContent("Raciocínio");
  });

  it("offers effort for a reasoning provider model only when the provider's wire carries it", async () => {
    const { modelCapabilities } = await import("./session");
    const spec = { id: "r1", displayName: null, contextWindow: null, maxOutput: null, supportsImages: false, supportsTools: true, supportsReasoning: true, estimated: false };
    const provider = (wire: "responses" | "chat" | "anthropic", reasoningEffort: boolean) => ({
      provider: "aura-p", models: [],
      providers: [{ id: "p", name: "P", preset: "custom", wire, baseUrl: "http://127.0.0.1/v1", auth: "none" as const, extraHeaders: {}, credentialHint: null, status: "verified" as const, lastError: null, quirks: { noParallelToolCalls: false, reasoningEffort, noStreamOptions: false }, models: [spec] }],
    });
    expect(modelCapabilities(provider("responses", false), "r1")?.efforts).toEqual(["low", "medium", "high"]);
    expect(modelCapabilities(provider("anthropic", false), "r1")?.efforts).toEqual(["low", "medium", "high"]);
    expect(modelCapabilities(provider("chat", true), "r1")?.efforts).toEqual(["low", "medium", "high"]);
    expect(modelCapabilities(provider("chat", false), "r1")?.efforts).toEqual([]);
    expect(modelCapabilities(provider("chat", false), "r1")?.reasoning).toBe(true);
  });

  it("model picker switches the mode", async () => {
    await freshApp({ signedIn: true });
    const { useSession } = await import("./session");
    const user = userEvent.setup();
    render(<OverlayApp />);
    await user.click(await screen.findByRole("button", { name: "Expandir" }));
    await user.click(await screen.findByRole("button", { name: /modo Chat/ }));
    await user.click(within(screen.getByRole("dialog", { name: "Modelo e modo" })).getByRole("menuitemradio", { name: /Tarefa/ }));
    expect(useSession.getState().mode).toBe("task");
  });
});
