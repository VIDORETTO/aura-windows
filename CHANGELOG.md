# Changelog

All notable changes follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.2.0] - 2026-10-05

### Added
- YOLO mode: in Task mode the agent runs without sandbox and never asks for
  permission (commands, file changes, permissions and MCP tool calls are
  accepted). Turned on in Settings › General after a risk warning and typing
  ACEITO / ACCEPT (checked by the host); the Overlay shows "Task · YOLO".
- Extensions: search, bulk on/off per section, MCP server editing (secrets
  kept when left blank) and "Create with AI" for skills, quick commands and
  MCP servers (agent in Task mode).
- Aura MCP tools `extensions_list`, `skill_save`, `quick_command_save`,
  `mcp_server_save` (writes need the user's approval) and built-in agent
  skills in `core-skills` that the user cannot see or turn off.
- Settings search (Ctrl+K / Ctrl+F) across every page.

### Changed
- ChatGPT plan offers only GPT-6 Luna, GPT-6.1 Sol and GPT-6 Astra; new plan
  conversations always start with an explicit catalog model.
- Model and reasoning effort can be chosen in the compact Overlay before the
  first message; efforts are chips and the chosen effort shows in the pill.
- `/` and `@` menus: Enter picks the highlighted item; a bare `/` or `@` is
  never sent.

## [0.1.0] - 2026-10-05

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
- GPT-6.1 Sol (`gpt-6.1-sol`, efforts low…max, 1,050,000 context) in the
  ChatGPT plan list and in model discovery of compatible providers; `max`
  reasoning effort (effort 013).
- Reasoning effort per model and mode (Chat, Task, Plan): remembered from the
  Overlay picker and editable in Settings › Account & models.
- Manual provider models declare their accepted efforts and default effort.
- Switching Chat/Task/Plan mid-conversation now reaches the agent: a divider in
  the conversation and a one-time mode note on the next turn (the thread's
  start-time instructions used to keep the first mode) (effort 014).
- Input bar (effort 015): `/` does something (commands, quick commands, Skills)
  and `@` adds context, in labelled sections; both follow the caret and filter
  without accents. Choosing a Skill adds a Skill chip (no `$name` text). A typed
  quick command shows its argument and text source. Enter during an answer
  queues the message (Ctrl+Enter still steers). The compact Overlay shows the
  mode. Messages offer Try again, Redo in Task mode and Edit. The context meter
  asks before compacting. The Persona asks for web search on live data.
- Opt-in real app-server test: Task mode denies writes outside the workspace and
  asks for approval on escalation.
- Archived conversations: an "Archived" filter in History with Unarchive
  (`thread/unarchive`); About shows the version and links (effort 016).

### Changed
- Desktop, not web page (effort 016): the right-click menu keeps only editing
  actions (no Save as/Print/Back/Reload/Inspect; none on empty space) and browser
  shortcuts (F5, Ctrl+P, Ctrl+S, Alt+←, Ctrl+F) are off in release builds;
  destructive actions confirm inside the app instead of the browser's dialog.
- History "Load more" no longer skips conversations created in the same second
  as the last row (whole-second app-server cursor).
- The Settings default model applies only to the ChatGPT plan and an app
  profile's model wins over it; the picker names the effective model.
- `@` items are single typable words; quick command previews show `‹inglês›`
  instead of raw placeholders; Settings pages open at the top; the effort table
  fits its card; approvals say Accepted/Declined; duplicated rows removed.
- Input bar: Enter on a message ending in `@word` sends it (it used to capture
  the screen); a bare `/` is never sent; Esc closes the menu without erasing the
  text; built-in commands follow the interface language and `/tela` left the
  menu (still accepted when typed). The window button "Compactar" is now
  "Recolher"/"Collapse" so it no longer reads like compacting the conversation.
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
