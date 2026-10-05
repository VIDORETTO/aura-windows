// 017: search, bulk on/off, MCP editing and "Create with AI" in Extensions;
// the global Settings search.
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { freshApp } from "../test/harness";
import { setBridge } from "../ipc/bridge";
import { SettingsApp } from "./SettingsApp";
import { buildIndex, searchIndex } from "./search";
import { matchesQuery } from "./Extensions";

type Bridge = Awaited<ReturnType<typeof freshApp>>;

function spy(bridge: Bridge) {
  const calls: { cmd: string; args: Record<string, unknown> }[] = [];
  setBridge({ ...bridge, invoke: async <R,>(cmd: string, args: Record<string, unknown> = {}) => {
    calls.push({ cmd, args });
    return bridge.invoke<R>(cmd, args);
  } });
  return calls;
}

async function openExtensions(bridge: Bridge) {
  await bridge.invoke("skills_create", { name: "revisar-contrato", description: "Revisa contratos de aluguel", body: "x" });
  await bridge.invoke("skills_create", { name: "resumir-ata", description: "Resume atas de reunião", body: "x" });
  window.location.hash = "#/settings/extensions";
  render(<SettingsApp />);
  await screen.findByRole("listitem", { name: "resumir-ata" });
}

describe("Extensions (017)", () => {
  it("searches without accents and turns only the listed items off (AC-004)", async () => {
    const bridge = await freshApp({ signedIn: true });
    const user = userEvent.setup();
    await openExtensions(bridge);
    const skills = screen.getByRole("heading", { name: "Skills" }).closest("section")!;
    expect(within(skills).getByRole("status", { name: "Ativas em Skills" })).toHaveTextContent("3 de 3 ativas");
    await user.type(screen.getByRole("searchbox", { name: "Buscar extensões" }), "REUNIAO");
    expect(within(skills).queryByRole("listitem", { name: "revisar-contrato" })).toBeNull();
    expect(within(skills).getByRole("listitem", { name: "resumir-ata" })).toBeInTheDocument();
    await user.click(within(skills).getByRole("button", { name: "Desativar todas em Skills" }));
    await waitFor(() => expect(bridge.state.disabledSkills).toEqual(["aura/resumir-ata/SKILL.md"]));
    await user.clear(screen.getByRole("searchbox", { name: "Buscar extensões" }));
    await waitFor(() => expect(within(skills).getByRole("status", { name: "Ativas em Skills" })).toHaveTextContent("2 de 3 ativas"));
    // Everything back on at once.
    await user.click(within(skills).getByRole("button", { name: "Ativar todas em Skills" }));
    await waitFor(() => expect(bridge.state.disabledSkills).toEqual([]));
    // Quick commands: bulk off acts on the filtered ones only.
    const quick = screen.getByRole("heading", { name: "Comandos rápidos" }).closest("section")!;
    await user.type(screen.getByRole("searchbox", { name: "Buscar extensões" }), "traduz");
    await user.click(within(quick).getByRole("button", { name: "Desativar todas em Comandos rápidos" }));
    await waitFor(() => expect(bridge.state.quick.filter((q) => !q.enabled).map((q) => q.name)).toEqual(["traduzir"]));
  });

  it("'Create with AI' sends the request to the agent in Task mode (AC-005)", async () => {
    const bridge = await freshApp({ signedIn: true });
    const calls = spy(bridge);
    const user = userEvent.setup();
    await openExtensions(bridge);
    await user.click(screen.getByRole("button", { name: "Criar Skill com IA" }));
    const panel = screen.getByRole("group", { name: "Criar Skill com IA" });
    await user.type(within(panel).getByRole("textbox", { name: "Descreva o que você quer" }), "revisa contratos de aluguel");
    await user.click(within(panel).getByRole("button", { name: "Enviar ao agente" }));
    const task = calls.find((c) => c.cmd === "agent_task")!;
    expect(task.args.mode).toBe("task");
    expect(task.args.text).toContain("revisa contratos de aluguel");
    expect(task.args.text).toContain("aura-criar-skill");
    expect(screen.queryByRole("group", { name: "Criar Skill com IA" })).toBeNull();
    expect(await screen.findByText("Pedido enviado ao Aura. Acompanhe no Overlay.")).toBeInTheDocument();
  });

  it("edits an MCP server saved without its secret and turns it on when the secret is typed", async () => {
    const bridge = await freshApp({ signedIn: true });
    bridge.state.mcp = [{ name: "github", transport: { type: "stdio", command: "npx", args: ["-y", "@modelcontextprotocol/server-github"], env: { GITHUB_TOKEN: { kind: "secret" } }, cwd: null }, enabled: false, disabledTools: [], approvalMode: "askForWrites", startupTimeoutSec: null, toolTimeoutSec: null }];
    const calls = spy(bridge);
    const user = userEvent.setup();
    await openExtensions(bridge);
    await user.click(screen.getByRole("button", { name: "Editar github" }));
    const form = screen.getByRole("group", { name: "Editar github" });
    expect(within(form).getByRole("textbox", { name: "Comando" })).toHaveValue("npx");
    await user.type(within(form).getByLabelText(/^Valor de GITHUB_TOKEN/), "ghp_x");
    await user.click(within(form).getByRole("button", { name: "Salvar" }));
    const save = calls.find((c) => c.cmd === "mcp_save")!;
    expect(save.args.secrets).toEqual([["GITHUB_TOKEN", "ghp_x"]]);
    expect(save.args.spec).toMatchObject({ name: "github", enabled: true, transport: { env: { GITHUB_TOKEN: { kind: "secret" } } } });
  });

  it("matches every word in any order, without accents", () => {
    expect(matchesQuery("ata reuniao", "resumir-ata", "Resume atas de reunião")).toBe(true);
    expect(matchesQuery("ata github", "resumir-ata", "Resume atas")).toBe(false);
    expect(matchesQuery("  ", "x")).toBe(true);
  });
});

describe("Settings search (017 AC-008)", () => {
  it("finds settings by words and opens the page", async () => {
    await freshApp({ signedIn: true });
    window.location.hash = "#/settings/general";
    const user = userEvent.setup();
    render(<SettingsApp />);
    await user.keyboard("{Control>}k{/Control}");
    const box = screen.getByRole("combobox", { name: "Buscar configurações" });
    expect(box).toHaveFocus();
    await user.type(box, "microfone");
    const results = screen.getByRole("listbox", { name: "Resultados da busca" });
    const options = within(results).getAllByRole("option");
    expect(options.length).toBeGreaterThan(0);
    expect(options.every((o) => /Voz|Privacidade|Atalhos/.test(o.textContent ?? ""))).toBe(true);
    const voice = options.find((o) => o.textContent?.endsWith("Voz"))!;
    await user.click(voice);
    await waitFor(() => expect(window.location.hash).toBe("#/settings/voice"));
    expect(screen.getByRole("link", { name: "Voz" })).toHaveAttribute("aria-current", "page");
    await user.type(box, "xyzw");
    expect(within(screen.getByRole("listbox", { name: "Resultados da busca" })).getByText("Nada encontrado")).toBeInTheDocument();
  });

  it("indexes page texts without placeholders and ranks word starts first", () => {
    const index = buildIndex(
      { "voice.mic": "Microfone", "voice.mic.hint": "Testa o microfone {name}", "general.theme": "Tema", "input.send": "Enviar" },
      { voice: "Voz", general: "Geral" },
    );
    expect(index.map((e) => [e.page, e.text])).toEqual([
      ["voice", "Voz"],
      ["general", "Geral"],
      ["voice", "Microfone"],
      ["voice", "Testa o microfone"],
      ["general", "Tema"],
    ]);
    expect(searchIndex(index, "microfone").map((e) => e.text)).toEqual(["Microfone", "Testa o microfone"]);
    expect(searchIndex(index, "enviar")).toEqual([]);
  });
});
