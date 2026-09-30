# AGENTS.md — Aura

Instruções para agentes (Codex, Claude Code e outros) que trabalham neste repositório.

## O que é

Aura é um assistente de IA residente para Windows: atalho global → Overlay translúcido → conversa agêntica via Codex app-server (login ChatGPT ou BYOK), com contexto de tela/áudio controlado por política de privacidade, voz com ASR local e anexos multimodais.

## Leia antes de mudar algo

0. `docs/HANDOFF.md` — estado real de cada área, o que nunca rodou e a fila de trabalho.
1. `CONTEXT.md` — vocabulário do domínio. Use os termos canônicos em código, testes e UI.
2. `docs/architecture/overview.md` — processos, módulos, dados, orçamentos de desempenho.
3. `docs/adr/` — decisões difíceis de reverter. Não contradiga um ADR sem propor um novo.
4. O `spec.md`, `plan.md` e o ticket do esforço em `specs/<effort>/`.

## Fluxo de trabalho (Hybrid Development Kit)

- Skills em `.agents/skills/hybrid-*`; runner em `.hybrid/hybrid.py`.
- Um ticket por vez: `python .hybrid/hybrid.py package --project . --effort <id> --ticket TK-xxx --json` deve retornar `ready: true` antes de implementar.
- Estado do ticket só com `ticket update`; evidência só com `evidence add` após execução real.
- Nunca edite `todo.md`, `docs/project/backlog.md` ou `verification.md`: são projeções (`render`).
- Mudança de comportamento volta para `spec.md` (nova revisão); detalhe técnico reversível vai para `plan.md`.

## Testes (TDD)

- Red → green por comportamento, um caso por vez, na seam pública definida no `plan.md`.
- Oráculo independente (literal da spec ou exemplo calculado à parte); nada de recompor o algoritmo no teste.
- Mocks só onde há variação real: SO (captura, áudio, DPAPI), rede, relógio, processos externos (app-server, worker).
- Comandos: `cargo test` (Rust, membros padrão; no Windows `cargo test --workspace --exclude aura-desktop`), `pnpm -C apps/desktop test` e `pnpm -C apps/desktop typecheck` (UI), `cargo run -p aura-bench -- all` (desempenho).
- Verificação de tipos dos adaptadores Windows a partir de Linux: `cargo check -p aura-win --target x86_64-pc-windows-msvc`.
- Contrato IPC: `UPDATE_GOLDEN=1 cargo test -p aura-app --test ipc_contract` após mudar formatos (ADR 0009).

## Convenções

- Rust edition 2024, `clippy -D warnings`, `rustfmt`. Erros com `thiserror` em crates de biblioteca, `anyhow` só em binários.
- Nenhuma API do Windows fora de `crates/aura-win`, `aura-store::protect` (DPAPI) e `apps/desktop/src-tauri`.
- Orquestração e comandos da UI vivem em `crates/aura-app` (testável em qualquer SO); o shell Tauri só encaminha.
- UI: React 19 + TS estrito, Tailwind 4 com tokens de `docs/design/ui-ux.md`, estado em Zustand, IPC pelo cliente tipado `src/ipc/commands.ts` (tipos em `src/ipc/types.ts`, contrato dourado — ADR 0009).
- Segredos nunca em logs, arquivos de config ou webview.
- Textos de UI em `apps/desktop/src/i18n/{pt-BR,en}.ts` (mesmas chaves; teste de paridade).

## Limites

- Não atualizar a versão fixada do `codex-app-server` sem ticket próprio (regenerar schema + suíte de contrato).
- Não adicionar dependência com licença não permissiva.
- Não commitar modelos, binários de sidecar ou capturas.
