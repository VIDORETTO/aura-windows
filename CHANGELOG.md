# Changelog

All notable changes follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- Rust workspace: core domain (settings, gestures, placement, context chips,
  JSON-RPC, redacted logging), SQLite store with DPAPI-sealed vault, privacy
  policy engine, Codex app-server client/supervisor with fake server, Sign in
  with ChatGPT (PKCE, JWKS validation, refresh, revoke), local Responses
  gateway (passthrough, Chat Completions and Anthropic translation), Aura MCP
  server, capture pipeline (redaction, encrypted segments, retention), audio
  hub and recorder, ASR catalog/downloader/worker client, attachment
  ingestion (text, code, HTML, CSV/XLSX/ODS, DOCX/PPTX/ODF, RTF), agent
  extensions (Skills, quick commands, MCP config and importers).
- `aura-app` host core: one composition root with every UI command, agent
  tools with consent flow, attachments, voice (push-to-talk), placement.
- `aura-win` Windows adapters: GDI capture, window inventory, UI Automation,
  Windows OCR, WASAPI (cpal), Credential Manager (chunked), global hotkeys +
  low-level hook, clipboard/SendInput, DXGI probe, Media Foundation H.264.
- Tauri 2 shell (tray, Overlay, Settings, shortcuts) and React 19 UI with
  pt-BR/en, streaming markdown, approvals, consent, history, settings.
- IPC golden contract shared by Rust and TypeScript tests.
- Background recording (recent buffer, continuous, manual recordings with
  WAV/MP4 export) wired to the privacy policy; `screen_recent`/`audio_recent`.
- Region selection over a frozen screen; app profiles; read-aloud (TTS);
  Minibar with completion notifications; first-run onboarding; redacted
  diagnostics export; signed auto-update; agent question/elicitation forms;
  context meter; Task-mode folder grants; files/changes panels with sandboxed
  previews; MCP server status and OAuth; memories toggle.
- Voice: live partial transcripts, cloud ASR fallback, custom vocabulary.
- Attachments: PDF text layer, WAV, and MP3/M4A/video via Media Foundation.
- Accessibility audit (axe) on every screen; E2E suite for Windows; `deny.toml`.
- End-to-end test against the real Codex app-server (opt-in).

- Functional audit fixes (effort 011): voice model install card and hardware-aware
  catalog; microphone/system-audio device choice with live meter test;
  reasoning effort only for capable models; history rename and paging;
  `/compactar`; clipboard image paste; provider edit, protocol/headers and
  manual model capabilities; Skills origin/enable/edit; MCP args with quotes,
  per-tool toggles, status and stderr log; memories review/edit/delete; PDF
  preview fallback; recent buffer 1–30 min; retention days/space; access log
  with exact time, outcome, sealed thumbnail and conversation link; recordings
  player, duration and attach; `@últimos minutos` (screen + mic/system clip);
  TTS voice choice, cloud voice with one-time consent, auto-read;
  `Ctrl+Shift+L` / `Ctrl+Shift+Enter`; profile default model; full pipeline
  diagnostics; resume onboarding; updater placeholder-key detection and
  release guard (`scripts/check-updater-key.mjs`).
- Selection is refreshed when returning to the Overlay (no hide/show needed);
  user-chosen accent color (presets, RGB picker, hex) for the whole UI (effort 012).
- Native E2E specs for every audit finding (`apps/desktop/e2e/specs`).

### Changed
- Aura starts in the tray with a notification (001 AC-001); launching it again,
  the shortcut or the tray opens the Overlay. `Esc` never hides the Overlay.
- `iniciar-aura.bat` builds the speech worker with the local engines.

### Fixed
- Gateway rejected turns with several images (axum 2 MB body limit).
- The open audio segment (last ~10 s) was missing from the recent buffer.
- Access log stayed at `ask` after the user answered the consent.
- Cloud-voice consent survived removing the provider.
- Diagnostics command held a lock across an await (not `Send`).
- Launching the pinned standalone `codex-app-server` (no `app-server` subcommand).
- `thread/start` policy values use the protocol's kebab-case spellings.
- MCP tools sent as namespaces now round-trip through the Chat/Anthropic translation.
- `aura-bench`, CI (Linux + Windows), perf and release workflows.
