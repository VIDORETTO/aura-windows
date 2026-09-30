import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { freshApp } from "../test/harness";
import { SettingsApp, pageFromHash } from "./SettingsApp";
import { useApp } from "../state/app";

describe("Settings", () => {
  beforeEach(() => {
    window.location.hash = "#/settings/general";
  });

  it("routes by hash", () => {
    expect(pageFromHash("#/settings/privacy")).toBe("privacy");
    expect(pageFromHash("#/settings/nope")).toBe("general");
    expect(pageFromHash("")).toBe("general");
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
});
