# TODO gerado

<!-- GENERATED: hybrid tickets are the canonical source. Edit the ticket, then render again. -->

## [ ] TK-001 — TK-001 — Captura de tela sob demanda como Chip (Marco 1)
Status: `implemented` | Bloqueado por: nenhum

- [ ] TK-001.1 Unit com `SyntheticSource` → Chip (red→green).
- [ ] TK-001.2 Integração Windows AC-001 (magenta) red→green com `WgcSource`.
- [ ] TK-001.3 AC-002 janela.
- [ ] TK-001.4 UI `/screen`, botão, atalho; Vitest.
- [ ] TK-001.5 Demonstração do Marco 1 (vídeo: atalho → login → `/screen` → pergunta → resposta); evidências.

## [ ] TK-002 — TK-002 — Seleção de região com tela congelada
Status: `implemented` | Bloqueado por: TK-001

- [ ] TK-002.1 Unit `crop` red→green.
- [ ] TK-002.2 Vitest do seletor (conversão de escala).
- [ ] TK-002.3 Janela seletora e Chip; roteiro manual; evidências.

## [ ] TK-003 — TK-003 — Motor de Política de privacidade, exclusões, Pausa e Registro de acesso
Status: `implemented` | Bloqueado por: TK-001

- [ ] TK-003.1 Unit `decide` (tabela de casos, um por vez) red→green.
- [ ] TK-003.2 Unit `redact` red→green.
- [ ] TK-003.3 `Win32Inventory` + integração dublê KeePassXC.
- [ ] TK-003.4 Pausa, anexar ao abrir, registro; Vitest.
- [ ] TK-003.5 Evidências.

## [ ] TK-004 — TK-004 — Servidor MCP do Aura com ferramentas de tela e Permissão do agente
Status: `implemented` | Bloqueado por: TK-003

- [ ] TK-004.1 Servidor MCP + tool `active_window_info` com cliente de teste (red→green).
- [ ] TK-004.2 `screen_capture` com Permissões (AC-007) um modo por vez.
- [ ] TK-004.3 `screen_text` UIA/OCR (AC-011).
- [ ] TK-004.4 Registro no Codex + header por thread; confirmar override por thread.
- [ ] TK-004.5 Cartão de Permissão; AC-010 manual; evidências.

## [ ] TK-005 — TK-005 — Buffer recente de tela (últimos N minutos)
Status: `implemented` | Bloqueado por: TK-003

- [ ] TK-005.1 Unit `retention::plan` red→green (casos: vazio, dentro, fora, fronteira).
- [ ] TK-005.2 Recorder com `SyntheticSource`/`NullEncoder` + cifragem (red→green).
- [ ] TK-005.3 `MfH264Encoder` em memória (Windows) + verificação de reprodução do fMP4 decifrado.
- [ ] TK-005.4 UI de modos e indicadores; Vitest.
- [ ] TK-005.5 Benchmark H-005; evidências.

## [ ] TK-006 — TK-006 — Gravação manual, modo Contínuo, Retenção e gerenciador de Gravações
Status: `implemented` | Bloqueado por: TK-005

- [ ] TK-006.1 Unit retenção contínua red→green.
- [ ] TK-006.2 `Recordings` manual com `NullEncoder` (AC-015) red→green.
- [ ] TK-006.3 Protocolo de mídia + player (manual).
- [ ] TK-006.4 UI Gravações; Vitest; evidências.

## [ ] TK-007 — TK-007 — Anexar "últimos X minutos" e ferramenta de frames recentes
Status: `implemented` | Bloqueado por: TK-004, TK-005

- [ ] TK-007.1 Unit `sample_indices` red→green.
- [ ] TK-007.2 `media.keyframes` no worker (integração Windows).
- [ ] TK-007.3 `Clips::from_range` + Chip (AC-014).
- [ ] TK-007.4 `screen_recent` (AC-012); evidências.
