---
schema: hybrid/ticket
schema_version: 1.0
id: TK-005
effort: 002-conversa-agente-codex
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-003"]
requirement_refs: ["FR-005"]
acceptance_refs: ["AC-013", "AC-014"]
spec_revision: 2
plan_revision: 2
owned_areas: ["crates/aura-core/src/context.rs", "apps/desktop/src/conversation/chips", "apps/desktop/src-tauri/src/context.rs"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-005 — Chips de contexto e envio de imagens

## Objetivo e limites

Entrega o modelo `ContextChip`/`TurnInput` (OT-005), o registro de contribuições de contexto no host, a `ChipsBar`, colar/arrastar imagens, cópia para o Workspace da conversa e envio como `localImage`, com bloqueio quando o modelo não aceita imagem.

Não inclui: captura de tela (004 TK-001 usa esta Interface), anexos não-imagem (007).

## Leitura em ordem

1. `specs/002-conversa-agente-codex/plan.md` → OT-005.
2. `CONTEXT.md` → Chip de contexto, Anexo, Workspace da conversa.
3. `crates/aura-codex/src/service.rs` → `send` (TK-003); `models.rs` → `input_modalities` (TK-004 se concluído; senão usar `model/list` direto).
4. `docs/design/ui-ux.md` → "Chip de contexto".

## Decisões já resolvidas

- `ContextChip { id, kind: Screen|Region|Selection|Audio|File|Image, label, preview_path?, payload: ChipPayload, blocked_reason? }`; `TurnInput::{Text, LocalImage{path}, ...}` (007 acrescenta variantes).
- O host é dono dos Chips pendentes por Overlay (`ContextTray`), a UI só lista/remove/pede adição; conteúdo nunca trafega em base64 pela IPC — só caminhos/miniaturas.
- Imagens coladas são salvas em `workspaces/<id>/attachments/<uuid>.<ext>`; se ainda não há Conversa, em `workspaces/_pending/` e movidas no `start`.
- Limites: 10 imagens/turno, 20 MB cada; formatos PNG/JPEG/WebP/GIF (GIF: primeiro quadro).
- Liberdade local: miniaturas (256 px, WebP).

## Mapa de alterações

- Novo: `crates/aura-core/src/context.rs` → `ContextChip`, `ChipKind`, `ChipPayload`, `TurnInput`, `ContextTray::{add, remove, drain_for_turn, list}`.
- Novo: `apps/desktop/src-tauri/src/context.rs` → comandos `chips_list`, `chip_remove`, `chip_add_image_from_clipboard`, `chip_add_files(paths)`; evento `chips_changed`.
- Existente: `conversation.rs` → `conversation_send` drena a `ContextTray`.
- Novo: `apps/desktop/src/conversation/chips/{ChipsBar.tsx,Chip.tsx}`; tratamento de `paste`/`drop` no `InputBar`.

## Contrato técnico

- Entradas: imagem da área de transferência ou arquivos arrastados.
- Saídas: Chips listados; no envio, `TurnInput::LocalImage{path}` por Chip de imagem, na ordem de adição, após o texto.
- Invariantes: Chip removido nunca é enviado; `drain_for_turn` esvazia a bandeja atomicamente.
- Erros: `ChipError::{TooLarge{max}, UnsupportedFormat, TooMany{max}, ModelWithoutImage}`.
- Efeitos: arquivos copiados para o workspace.

## Exemplos de aceite

- **AC-013**: colar PNG 800×600 → Chip "Imagem 800×600" com miniatura; enviar "o que é isso?" → payload `turn/start.input = [{type:"text", text:"o que é isso?"}, {type:"localImage", path:"…\\workspaces\\<id>\\attachments\\<uuid>.png"}]`; colar outra e remover antes de enviar → não aparece no payload.
- **AC-014**: modelo com `inputModalities:["text"]` + Chip de imagem → Chip com aviso "Modelo atual não aceita imagens"; botão enviar desabilitado com ação "Trocar para modelo compatível" (lista modelos com `image`). Arquivo de 25 MB → `TooLarge{20 MB}`.

## Dependências e sequência de execução

Depende de: TK-003.

- [ ] TK-005.1 Unit `ContextTray` (adicionar/remover/drenar) red→green.
- [ ] TK-005.2 Contrato do payload AC-013.
- [ ] TK-005.3 UI colar/arrastar + Vitest.
- [ ] TK-005.4 AC-014 + limites.
- [ ] TK-005.5 Manual real com imagem; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-core context -p aura-codex`; `pnpm -C apps/desktop test -- chips`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: nenhum específico.

## Condição de retorno à planejadora

Retornar se `localImage` exigir caminho dentro de raiz permitida pelo sandbox e o workspace não for aceito.

## Relatório de saída

Relatar Interface `ContextChip/TurnInput` final (consumida por 004/005/007), resultados e EV refs.
