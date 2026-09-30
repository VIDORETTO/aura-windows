# Security policy

Aura sees your screen, hears your microphone and holds API keys, so security
reports are taken seriously.

## Reporting a vulnerability

Please **do not open a public issue**. Use GitHub's private vulnerability
reporting ("Security" tab → "Report a vulnerability") with:

- affected version and Windows build;
- steps to reproduce or a proof of concept;
- impact (what an attacker can read or do).

You will get an acknowledgement within 7 days. Coordinated disclosure after a
fix is released; credit is given unless you prefer otherwise.

## Scope and design

The threat model and controls live in `docs/architecture/overview.md`
("Segurança") and the ADRs. Highlights:

- Secrets (ChatGPT tokens, BYOK keys, MCP tokens) live only in Windows
  Credential Manager; the database is sealed with a DPAPI-protected key.
- The loopback gateway and MCP server bind to `127.0.0.1` on a random port,
  require a per-run token and reject requests with an `Origin` header.
- The Overlay is excluded from screen capture (`WDA_EXCLUDEFROMCAPTURE`).
- Every agent access to screen/audio passes the privacy policy (allow / ask /
  deny) and is written to the local access log.
- Sidecars (Codex app-server, speech models) are verified by pinned SHA-256
  before use.
- Logs are redacted (`Bearer`, `sk-…`, JWTs) before they hit disk.

Out of scope: attacks that already require code execution as the same
Windows user (they can read the same vault).
