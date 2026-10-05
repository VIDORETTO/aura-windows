# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — passed

- Ticket: `—`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-004`, `AC-005`, `AC-006`, `AC-007`, `AC-008`
- Procedure: `cargo test --workspace --exclude aura-desktop; cargo clippy --workspace --all-targets -D warnings; UPDATE_GOLDEN=1 cargo test -p aura-app --test ipc_contract; AURA_CODEX_BIN=<pinned rust-v0.159.0> cargo test -p aura-app --test real_app_server -- --ignored (asks_before_aura_write_tools, byok_turn_and_mcp_tool); pnpm test; pnpm typecheck; pnpm build`
- Execution: `executed`
- Environment: Windows 10 Pro 19045, Rust workspace toolchain, Node 22.17, pnpm 12.5.1, codex-app-server rust-v0.159.0
- Tested revision: `local:c2034d39da4cdb1770593dd99565fbebc1ca2af086ad3d60812900f5108709be`
- Timestamp: `2026-10-05T12:20:09+00:00`
- Observations: Rust 327 passed/5 ignored; UI 188/188; real app-server accepts per-tool approval_mode=prompt and sends mcp_tool_call elicitation before quick_command_save
- Evidence refs: none
- Limitations: No ChatGPT account (plan catalog verified by functions); native E2E not run (tauri-driver/msedgedriver missing)
