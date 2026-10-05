// 024: the Meeting panel never starts anything by itself.
import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi } from "vitest";
import { api } from "../ipc/commands";
import { freshApp } from "../test/harness";
import { useApp } from "../state/app";
import { MeetingPanel, mmss } from "./MeetingPanel";

describe("meeting panel", () => {
  it("formats minutes and seconds", () => {
    expect(mmss(125_000)).toBe("02:05");
    expect(mmss(-5)).toBe("00:00");
  });

  it("shows the three ways to prepare and starts nothing on open", async () => {
    await freshApp({ signedIn: true });
    const start = vi.spyOn(api, "meetingStart");
    render(<MeetingPanel />);
    expect(await screen.findByText("Nova reunião")).toBeInTheDocument();
    expect(screen.getByText(/Só começa quando você apertar Começar/)).toBeInTheDocument();
    expect(screen.getByLabelText("⚡ Explicar em uma frase")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Grill-me completo/ })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Começar sem preparo/ })).toBeInTheDocument();
    expect(start).not.toHaveBeenCalled();
  });

  it("one sentence goes to the agent with the prepare skill; grill-me too", async () => {
    await freshApp({ signedIn: true });
    const task = vi.spyOn(api, "agentTask").mockResolvedValue(undefined);
    const user = userEvent.setup();
    render(<MeetingPanel />);
    const send = await screen.findByRole("button", { name: "Preparar" });
    expect(send).toBeDisabled();
    await user.type(screen.getByLabelText("⚡ Explicar em uma frase"), "1:1 com a Ana");
    await user.click(send);
    expect(task).toHaveBeenCalledWith(expect.stringContaining("$aura-preparo"), "chat");
    expect(task.mock.calls[0][0]).toContain("1:1 com a Ana");
    await user.click(screen.getByRole("button", { name: /Grill-me completo/ }));
    expect(task.mock.calls[1][0]).toContain("grill-me completo");
  });

  it("starts without preparing, shows the live panel and ends", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<MeetingPanel />);
    await user.click(await screen.findByRole("button", { name: /Começar sem preparo/ }));
    expect(await screen.findByRole("timer")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Perdi o fio" })).toBeInTheDocument();
    expect(screen.getByText(/Ainda sem falas/)).toBeInTheDocument();
    const task = vi.spyOn(api, "agentTask").mockResolvedValue(undefined);
    await user.click(screen.getByRole("button", { name: "Perdi o fio" }));
    expect(task.mock.calls[0][0]).toMatch(/meeting_get com meeting_id=m1/);
    await user.click(screen.getByRole("button", { name: "Pausar" }));
    expect(screen.getByRole("button", { name: "Retomar" })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Encerrar" }));
    expect(await screen.findByText("Nova reunião")).toBeInTheDocument();
    expect(screen.getByText("Reuniões salvas")).toBeInTheDocument();
  });

  it("a briefing prepared by the agent waits for the user to press Start", async () => {
    await freshApp({ signedIn: true });
    const start = vi.spyOn(api, "meetingStart");
    vi.spyOn(api, "meetingBrief").mockResolvedValue("Objetivo: fechar o orçamento");
    const user = userEvent.setup();
    render(<MeetingPanel />);
    const card = await screen.findByLabelText("Entendi assim", { selector: "section" });
    expect(within(card).getByRole("textbox")).toHaveValue("Objetivo: fechar o orçamento");
    expect(start).not.toHaveBeenCalled();
    await user.click(within(card).getByRole("button", { name: "Começar" }));
    expect(start).toHaveBeenCalledWith("Reunião", "other", "Objetivo: fechar o orçamento");
  });

  it("'esqueci de iniciar' saves the last minutes as a meeting; saved meetings can be read", async () => {
    await freshApp({ signedIn: true });
    const user = userEvent.setup();
    render(<MeetingPanel />);
    await user.selectOptions(await screen.findByLabelText("Minutos para trás"), "10");
    await user.click(screen.getByRole("button", { name: "Salvar como reunião" }));
    const saved = await screen.findByRole("button", { name: /Reunião · / });
    await act(async () => {
      useApp.setState({ meetingRevision: useApp.getState().meetingRevision + 1 });
    });
    await user.click(saved);
    expect(await screen.findByRole("button", { name: "Resumo e ações" })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Apagar reunião" }));
    await waitFor(() => expect(screen.queryByText("Reuniões salvas")).toBeNull());
  });
});
