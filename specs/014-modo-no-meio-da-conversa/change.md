---
schema: hybrid/change
schema_version: 1.0
effort_id: 014-modo-no-meio-da-conversa
revision: 1
status: closed
profile: compact
---

# Change: Trocar entre Chat, Tarefa e Plano no meio da conversa

## Objetivo e limites

Pedido do usuário em 04/10/2026: poder trocar entre os modos Chat, Tarefa e Plano no meio de uma conversa.

Causa encontrada: o seletor já permitia a troca e o Aura já mudava a política de sandbox do turno seguinte, mas as instruções de desenvolvedor da thread (que dizem "Modo Chat: não altere arquivos…") são fixadas no `thread/start` do app-server e nunca mudavam. O agente continuava se comportando no modo inicial. O app-server fixado (rust-v0.159.0) só aceita trocar instruções por turno com campos experimentais (`collaborationMode`), que o Aura não habilita.

Fora do escopo: habilitar a API experimental do app-server; trocar provedor no meio da conversa (continua bloqueado).

## Contrato de comportamento

- Entradas: troca de modo no seletor com uma conversa aberta.
- Saída: uma linha "Modo alterado para …" na conversa; no turno seguinte, a política de sandbox do modo novo e um aviso `<aura-mode>…</aura-mode>` depois do texto do usuário, dizendo ao agente que o modo novo substitui a instrução anterior (no idioma da interface). O aviso vai uma vez por troca; depois de reiniciar o app, o primeiro turno repete o modo atual (o modo anunciado não é conhecido).
- Erros/invariantes: o aviso nunca aparece ao reabrir a conversa (removido da transcrição); escolher o modo atual não faz nada; se a troca falhar no host, o modo volta ao anterior.
- Compatibilidade: conversas que não trocam de modo não recebem aviso.

## Requisitos e aceite

- **FR-001** — O usuário MUST poder trocar o modo no meio da conversa, e o agente MUST seguir o modo novo a partir do turno seguinte.
- **AC-001** — Dada uma conversa iniciada em Chat, quando o usuário troca para Tarefa e envia, então aparece "Modo alterado para Tarefa", o turno vai com sandbox `workspaceWrite` e o pedido ao modelo traz o aviso do Modo Tarefa uma única vez; trocando para Plano, o próximo turno vai `readOnly` com o aviso do Modo Plano; reabrindo a conversa, as mensagens do usuário não mostram os avisos.

## Leitura e mapa de alterações

- `crates/aura-codex/src/modes.rs` → `mode_change_note`, `strip_mode_notes`; new.
- `crates/aura-codex/src/service.rs` → `ThreadCtx.announced`, `send`, `set_mode`, `set_language`, `open`; existing.
- `crates/aura-app/src/host.rs` → `apply_runtime_settings` (idioma); existing.
- `apps/desktop/src/overlay/session.ts`, `ModelPicker.tsx`, `Messages.tsx`, `state/conversation.ts`; existing.

## Plano breve

Seam: `CodexService` contra o app-server falso (parâmetros de `turn/start` gravados e transcrição), store/UI no Vitest, app nativo com o app-server real e upstream loopback. Abordagem: aviso de modo no turno, após a entrada do usuário. Dependências: none.

## Sequência e tarefas

- [x] C-001 Red/green do serviço (aviso, uma vez, idioma, transcrição limpa).
- [x] C-002 Red/green da UI (divisor, sem duplicar).
- [x] C-003 E2E nativo, regressão e evidência.

## Validação e evidência

Comando/procedimento: `cargo test -p aura-codex --test service switching_mode`, `pnpm -C apps/desktop test`, `pnpm -C apps/desktop/e2e test --spec ./specs/mode-switch.e2e.ts`.

Resultado executado: EV-001 — serviço 14/14, UI 147/147; nativo no app-server real: aviso do Modo Tarefa no 2º turno, ausente no 3º, aviso do Modo Plano no 4º, instrução "Chat mode" do início ainda presente (causa confirmada), transcrição reaberta sem avisos; regressões `model-efforts` e `compact`.

Limitações: o comportamento do modelo real diante do aviso não foi avaliado (upstream loopback); a garantia de segurança continua sendo a sandbox do turno.

## Condição de retorno

Retornar se o contrato, símbolo, dependência ou decisão material não puder ser satisfeito sem ampliar o escopo.

## Estado

`change.md` é a fonte canônica deste esforço compacto. Não crie `tasks.md` ou tickets paralelos para esta mudança.
