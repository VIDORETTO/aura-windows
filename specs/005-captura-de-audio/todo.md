# TODO gerado

<!-- GENERATED: hybrid tickets are the canonical source. Edit the ticket, then render again. -->

## [ ] TK-001 — TK-001 — Fontes de áudio (microfone e sistema), hub e medidor
Status: `implemented` | Bloqueado por: nenhum

- [ ] TK-001.1 Unit reamostragem e nível com `SyntheticAudio` (red→green).
- [ ] TK-001.2 `AudioHub` fan-out (dois assinantes, um stream) red→green.
- [ ] TK-001.3 WASAPI mic/loopback (Windows) + fallback (AC-002).
- [ ] TK-001.4 UI e evidências.

## [ ] TK-002 — TK-002 — Buffer recente, Gravação manual e Contínuo para áudio
Status: `implemented` | Bloqueado por: TK-001

- [ ] TK-002.1 Recorder com sintético + cifragem (red→green).
- [ ] TK-002.2 Integração Windows AC-004.
- [ ] TK-002.3 UI de modos; H-013; evidências.

## [ ] TK-003 — TK-003 — Indicadores e Pausa de privacidade para áudio
Status: `implemented` | Bloqueado por: TK-002

- [ ] TK-003.1 Unit gate + sintético (red→green).
- [ ] TK-003.2 Indicadores; Vitest; manual Windows; evidências.

## [ ] TK-004 — TK-004 — Anexar áudio recente transcrito e ferramenta `audio_recent`
Status: `implemented` | Bloqueado por: TK-003

- [ ] TK-004.1 Unit `interleave` red→green.
- [ ] TK-004.2 `from_range` + Chip com `Transcriber` falso.
- [ ] TK-004.3 Tool MCP (AC-008).
- [ ] TK-004.4 Composto tela+áudio; manual real; evidências.
