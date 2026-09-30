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

### Fixed
- Launching the pinned standalone `codex-app-server` (no `app-server` subcommand).
- `thread/start` policy values use the protocol's kebab-case spellings.
- MCP tools sent as namespaces now round-trip through the Chat/Anthropic translation.
- `aura-bench`, CI (Linux + Windows), perf and release workflows.
