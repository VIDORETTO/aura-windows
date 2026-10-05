# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — passed

- Ticket: `—`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-004`, `AC-005`
- Procedure: `cargo test --workspace --exclude aura-desktop; cargo clippy --workspace --all-targets -D warnings; UPDATE_GOLDEN=1 cargo test -p aura-app --test ipc_contract; AURA_CODEX_BIN=<pinned rust-v0.159.0> cargo test -p aura-app --test real_app_server -- --ignored yolo; pnpm test; pnpm typecheck`
- Execution: `executed`
- Environment: Windows 10 Pro 19045, Node 22.17, pnpm 12.5.1, codex-app-server rust-v0.159.0
- Tested revision: `local:1975c2db5a27431ea12ca2eedd1da6d3ff73d8bf1041dab9b0c077639b60f7cf`
- Timestamp: `2026-10-05T12:35:28+00:00`
- Observations: Rust 330 passed/6 ignored; UI 191/191; real app-server accepts danger-full-access + never
- Evidence refs: none
- Limitations: Auto-accept exercised with the fake app-server command request only; no dedicated unit test for other request types
