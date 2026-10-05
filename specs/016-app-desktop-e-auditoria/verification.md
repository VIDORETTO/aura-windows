# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — passed

- Ticket: `—`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-004`, `AC-005`, `AC-006`
- Procedure: `Pure-function tests (context_menu_keep, start_model, page_cursor, templatePreview); service vs fake app-server (unarchive); Vitest (in-app confirmation, archived view, @ labels, previews, settings scroll, approval decision, picker default name); real pinned app-server ignored tests (sandbox confinement, archive/unarchive); full regression; native E2E demo (overlay, settings, composer) and production (compact, mode-switch, skills, model-efforts, memories, history, panels); real OS right-click (mouse_event) with screen capture; screenshot tour of every Overlay state and Settings page before/after.`
- Execution: `executed`
- Environment: Windows 11 Pro 26200, WebView2 154.0.4258.53, codex-app-server rust-v0.159.0; builds in target/e2e-demo and target/e2e-prod
- Tested revision: `local:aa886bb5e4721e1bc04aaab3ff22980cda84f85fc64c8dab59236fe6aee95ff0`
- Timestamp: `2026-10-05T01:33:36+00:00`
- Observations: Rust 319 passed/4 ignored, clippy clean; UI 177/177; native journeys green; History Load more 56 rows (51 before); right-click: input menu shows only editing items, header shows none.
- Evidence refs: none
- Limitations: Before-state of the right-click menu not capturable (production Overlay excluded from capture); recent-clip journey needs real audio playback and transcription; paging fix relies on the pinned app-server cursor format.
