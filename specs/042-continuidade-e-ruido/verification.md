# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — passed

- Ticket: `—`
- Acceptance: `AC-001`, `AC-002`
- Procedure: `AURA_CODEX_BIN=<fixado> cargo test -p aura-app --test conversation_continuity -- --ignored`
- Execution: `executed`
- Environment: Windows 11 Pro 26200; app-server fixado rust-v0.159.0; provedor Responses roteirizado; sem conta/Internet
- Tested revision: `local:eb5d9fbe821a8cb9a3cc933f80a04b65f2f4463227735d156747081df537765c`
- Timestamp: `2026-10-14T01:45:14+00:00`
- Observations: Red observado antes: Turno 2 recebia 'Previous web input unavailable'/'no longer available'; conversa reaberta recebia <aura-mode> falso. Green: 6/6 (seguimento web, web após reinício, fatos após reinício, sem aviso falso, troca Chat->Tarefa, troca antes do reinício).
- Evidence refs: none
- Limitations: Modelo real (ChatGPT) não exercitado.

## EV-002 — passed

- Ticket: `—`
- Acceptance: `AC-003`
- Procedure: `cargo test -p aura-mcp --test protocol`
- Execution: `executed`
- Environment: Windows 11 Pro 26200
- Tested revision: `local:33c9243cc5de16ad27707b54a12726c32eefa5ec796824e08135ee394607f96d`
- Timestamp: `2026-10-14T01:45:14+00:00`
- Observations: Red: empty_resource_and_prompt_listings falhou com -32601; green 5/5.
- Evidence refs: none
- Limitations: none recorded

## EV-003 — passed

- Ticket: `—`
- Acceptance: `AC-001`, `AC-002`, `AC-003`
- Procedure: `cargo test --workspace --exclude aura-desktop; clippy -D warnings; fmt --check; AURA_CODEX_BIN=<fixado> cargo test -p aura-app -- --ignored --test-threads=1; pnpm -C apps/desktop test; typecheck`
- Execution: `executed`
- Environment: Windows 11 Pro 26200; app-server rust-v0.159.0
- Tested revision: `local:7556ced4236c4bc6c8b8ecda763fda7bef984f172f1ecb16100476b8bdf14351`
- Timestamp: `2026-10-14T01:54:51+00:00`
- Observations: 519 Rust; ignorados 42/42 (task_mode com AURA_SBX_OUTSIDE fora do TEMP); UI 253. Rodada 2: windows_text_files, reminder_times, startup_retry red->green.
- Evidence refs: none
- Limitations: E2E WebDriver não executado (sem msedgedriver); modelo real não exercitado.
