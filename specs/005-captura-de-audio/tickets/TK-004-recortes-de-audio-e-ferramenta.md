---
schema: hybrid/ticket
schema_version: 1.0
id: TK-004
effort: 005-captura-de-audio
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-003"]
requirement_refs: ["FR-003", "FR-004"]
acceptance_refs: ["AC-007", "AC-008", "AC-009"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-audio/src/clips.rs", "crates/aura-mcp/src/tools/audio.rs", "apps/desktop/src/conversation/chips/AudioChip.tsx"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-004 — Anexar áudio recente transcrito e ferramenta `audio_recent`

## Objetivo e limites

Entrega `AudioClips::from_range` (decifra no worker, gera WAV 16 kHz no Workspace da conversa), transcrição via `Transcriber` (006), intercalação "Você"/"Sistema", o Chip de áudio e combinado tela+áudio, e a tool MCP `audio_recent`.

Não inclui: diarização de múltiplos falantes no Áudio do sistema.

## Leitura em ordem

1. `specs/005-captura-de-audio/spec.md` → US-003.
2. `crates/aura-asr/src/transcriber.rs` (006 TK-003) → `Transcriber::transcribe(wav) -> Transcript{segments[{start,end,text}], language}`.
3. `crates/aura-capture/src/clips.rs` (004 TK-007) → padrão de Recorte e Chip.
4. `crates/aura-mcp/src/tools/screen.rs` (004 TK-004) → padrão de tool com Permissão.

## Decisões já resolvidas

- Formato da Transcrição anexada: bloco de texto
  `Transcrição (últimos 2:00, Microfone+Sistema)\n[00:12] Você: …\n[00:15] Sistema: …`.
- "Tela + áudio" = um Chip composto com keyframes (004) + Transcrição do mesmo intervalo.
- `audio_recent{minutes≤30, source}` retorna o mesmo formato; negado conforme Política.
- Liberdade local: layout do Chip.

## Mapa de alterações

- Novo: `crates/aura-audio/src/clips.rs` → `AudioClips::from_range`, `interleave(mic, sys) -> Vec<Line>`.
- Novo: `crates/aura-mcp/src/tools/audio.rs`.
- Novo: `apps/desktop/src/conversation/chips/AudioChip.tsx`; menu `@áudio`.

## Contrato técnico

- Entradas: intervalo e Fonte(s).
- Saídas: `ContextChip{kind: Audio}`; no envio, `TurnInput::Text(transcrição)` + arquivo WAV no workspace.
- Invariantes: OT-003 (005); sem áudio no intervalo → `ClipError::Empty`.
- Erros: `Transcriber` indisponível (sem modelo baixado) → Chip com ação "Baixar modelo de voz".

## Exemplos de aceite

- **AC-007** (unit `interleave`): mic `[{0.5,"oi"}]`, sys `[{0.2,"bem-vindo"},{1.0,"tudo bem?"}]` → linhas `[00:00] Sistema: bem-vindo`, `[00:00] Você: oi`, `[00:01] Sistema: tudo bem?` (ordenado por início). Integração com `Transcriber` falso → payload de texto exato acima e WAV no workspace.
- **AC-008**: cliente MCP de teste, Permissão Nunca → `denied`; Sempre → texto no formato definido.
- **AC-009**: Chip composto → payload com keyframes (`localImage`) seguidos do texto da transcrição do mesmo intervalo.

## Dependências e sequência de execução

Depende de: TK-003; 006-voz-e-asr/TK-003 e 004-contexto-de-tela/TK-007 (outros esforços).

- [ ] TK-004.1 Unit `interleave` red→green.
- [ ] TK-004.2 `from_range` + Chip com `Transcriber` falso.
- [ ] TK-004.3 Tool MCP (AC-008).
- [ ] TK-004.4 Composto tela+áudio; manual real; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-audio clips -p aura-mcp audio`; `pnpm -C apps/desktop test -- AudioChip`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: sem modelo ASR baixado → teste usa falso.

## Condição de retorno à planejadora

Retornar se a transcrição de 30 min exceder o tempo aceitável (> 60 s) com o modelo padrão, exigindo transcrição incremental em background.

## Relatório de saída

Relatar resultados e EV refs.
