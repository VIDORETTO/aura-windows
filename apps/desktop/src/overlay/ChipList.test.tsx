import { act, render, screen } from "@testing-library/react";
import { freshApp } from "../test/harness";
import type { ContextChip } from "../ipc/types";
import { ChipList } from "./ChipList";

it("localizes attachment counts and plural while preserving filenames and legacy chips", async () => {
  const bridge = await freshApp();
  await bridge.invoke("settings_update", { patch: { language: "en" } });
  const base = { kind: "file", previewPath: null, payload: { type: "text", text: "content" }, tokenEstimate: 3, blockedReason: null } as const;
  const chips: ContextChip[] = [
    { ...base, id: "one", label: "1 linhas.txt · 1 linhas", attachmentLabel: { fileName: "1 linhas.txt", parts: [{ type: "count", amount: 1, unit: "line" }] } },
    { ...base, id: "two", label: "two.txt · 2 linhas", attachmentLabel: { fileName: "two.txt", parts: [{ type: "count", amount: 2, unit: "line" }] } },
    { ...base, id: "legacy", label: "legacy label" },
  ];
  render(<ChipList chips={chips} />);
  expect(screen.getByText("1 linhas.txt · 1 line")).toBeInTheDocument();
  expect(screen.getByText("two.txt · 2 lines")).toBeInTheDocument();
  expect(screen.getByText("legacy label")).toBeInTheDocument();
  await act(async () => { await bridge.invoke("settings_update", { patch: { language: "ptBr" } }); });
  expect(screen.getByText("1 linhas.txt · 1 linha")).toBeInTheDocument();
  expect(screen.getByText("two.txt · 2 linhas")).toBeInTheDocument();
});
