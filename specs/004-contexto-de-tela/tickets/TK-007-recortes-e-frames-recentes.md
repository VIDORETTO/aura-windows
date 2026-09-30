---
schema: hybrid/ticket
schema_version: 1.0
id: TK-007
effort: 004-contexto-de-tela
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-004", "TK-005"]
requirement_refs: ["FR-006", "FR-004"]
acceptance_refs: ["AC-012", "AC-014"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-capture/src/clips.rs", "crates/aura-mcp/src/tools/screen_recent.rs", "apps/desktop/src/conversation/chips/ClipChip.tsx"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-007 — Anexar "últimos X minutos" e ferramenta de frames recentes

## Objetivo e limites

Entrega `Clips::from_range(source, from, to)` (promove segmentos a Recorte e copia para o workspace), `sample_keyframes(range, max)` via `aura-worker` (`media.keyframes`), o Chip de Recorte ("Anexar últimos 1/2/5 min") e a tool MCP `screen_recent{minutes, max_frames≤8}`.

Não inclui: áudio do Recorte (005 TK-004 acrescenta), vídeo como anexo externo (007).

## Leitura em ordem

1. `specs/004-contexto-de-tela/spec.md` → AC-012, AC-014.
2. `crates/aura-capture/src/segments.rs` (TK-005); `crates/aura-mcp/src/tools/screen.rs` (TK-004).
3. `crates/aura-worker` (006 TK-003) → protocolo do worker e método `media.keyframes` (criar se ainda não existir, no mesmo padrão).
4. `crates/aura-core/src/context.rs` (002) → `ChipKind::Clip`, `TurnInput::LocalImage`.

## Decisões já resolvidas

- Amostragem uniforme: `k = min(max, frames)` índices `round(i*(n-1)/(k-1))`; se `k=1`, último frame.
- Keyframes redimensionados a ≤ 1280 px; enviados como `localImage` com legenda de texto `"[t−02:00] app — título"` quando disponível no índice.
- Recorte copiado (segmentos cifrados) para `workspaces/<id>/clips/<clip>/`; nunca decifrado em disco.
- `screen_recent` passa por Permissão do agente (Tela) e só funciona com Buffer recente/Contínuo ativos (senão `unavailable`).
- Liberdade local: textos do menu.

## Mapa de alterações

- Novo: `crates/aura-capture/src/clips.rs` → `Clips::from_range`, `sample_indices(n, k)`.
- Novo: `crates/aura-mcp/src/tools/screen_recent.rs`.
- Existente/Novo: `crates/aura-worker/src/media.rs` → `media.keyframes{segments, indices}` (decifra via chave recebida por pipe, decodifica MF, retorna PNGs).
- Novo: `apps/desktop/src/conversation/chips/ClipChip.tsx`; menu `@tela` com "Últimos 1/2/5 min".

## Contrato técnico

- Entradas: `from`, `to`, `max_frames`.
- Saídas: `Vec<PathBuf>` de keyframes + metadados; Chip `Clip{duration}`.
- Invariantes: frames já estavam redigidos na gravação (TK-005); `max_frames ≤ 8`.
- Erros: intervalo sem segmentos → `ClipError::Empty`; worker indisponível → `Unavailable`.

## Exemplos de aceite

- **AC-012** (unit): `sample_indices(120, 8)` → `[0, 17, 34, 51, 68, 85, 102, 119]`; `sample_indices(5, 8)` → `[0,1,2,3,4]`; `sample_indices(10, 1)` → `[9]`. Integração: buffer sintético com frames numerados (número desenhado em pixels) → keyframes correspondem aos índices; `screen_recent` sem buffer → `code:"unavailable"`.
- **AC-014**: buffer ativo + "Anexar últimos 2 min" → Chip "Tela · últimos 2:00" com miniatura; envio → payload com ≤ 8 `localImage` na ordem temporal + texto com legendas; pasta `clips/<id>` no workspace contém `.seg` cifrados.

## Dependências e sequência de execução

Depende de: TK-004, TK-005; 006-voz-e-asr/TK-003 (outro esforço; `aura-worker`).

- [ ] TK-007.1 Unit `sample_indices` red→green.
- [ ] TK-007.2 `media.keyframes` no worker (integração Windows).
- [ ] TK-007.3 `Clips::from_range` + Chip (AC-014).
- [ ] TK-007.4 `screen_recent` (AC-012); evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-capture clips -p aura-mcp screen_recent`; `cargo nextest run -p aura-worker --features win-integration keyframes`; `pnpm -C apps/desktop test -- ClipChip`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: sem desktop/encoder.

## Condição de retorno à planejadora

Retornar se decodificar keyframes no worker exceder 2 s para 8 frames de 2 min (reavaliar formato/segmentação).

## Relatório de saída

Relatar resultados, tempos de extração, EV refs.
