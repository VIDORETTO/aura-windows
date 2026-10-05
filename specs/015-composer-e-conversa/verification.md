# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — passed

- Ticket: `—`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-004`, `AC-005`, `AC-006`, `AC-007`, `AC-008`, `AC-009`, `AC-010`
- Procedure: `Red/green: composer.ts (Vitest), Host::attach_skill (fake app-server), Composer.test.tsx (menus, skill chip, queue, mode badge, message actions, compaction confirm, alias Enter), persona unit test; full regression cargo test --workspace --exclude aura-desktop, clippy -D warnings, fmt, pnpm test, typecheck; real app-server sandbox test real_app_server_task_mode_confines_writes; native E2E on demo (composer, overlay, settings) and production with the real pinned app-server (compact, mode-switch, skills, model-efforts, panels).`
- Execution: `executed`
- Environment: Windows 11 Pro 26200, WebView2 154.0.4258.53, codex-app-server rust-v0.159.0; builds in target/e2e-demo and target/e2e-prod (target/release untouched)
- Tested revision: `local:1b5e4aebe255e56b929804de5e7bd261052e0139048808f4fae78323031b2cec`
- Timestamp: `2026-10-04T23:52:02+00:00`
- Observations: Rust 313 passed/3 ignored; UI 169/169; all 9 native journeys green. Native run caught a regression (English UI: Enter on /compactar replaced the text with /compact) fixed with a Vitest case. Sandbox: direct write outside workspace denied; escalation asked for one approval; declined -> no file.
- Evidence refs: none
- Limitations: Retry resends as a new turn (no revert); client-side queue; model behaviour under the web-search persona line not evaluated.
