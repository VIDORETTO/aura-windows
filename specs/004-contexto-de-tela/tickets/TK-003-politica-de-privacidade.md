---
schema: hybrid/ticket
schema_version: 1.0
id: TK-003
effort: 004-contexto-de-tela
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-001"]
requirement_refs: ["FR-003", "FR-007"]
acceptance_refs: ["AC-003", "AC-005", "AC-006", "AC-008"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-policy", "crates/aura-capture/src/windows.rs", "crates/aura-capture/src/redact.rs", "crates/aura-store/src/migrations/0004_capture.sql", "crates/aura-store/src/access_log_repo.rs", "apps/desktop/src/privacy"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-003 — Motor de Política de privacidade, exclusões, Pausa e Registro de acesso

## Objetivo e limites

Entrega `aura-policy` (Modos, Permissões, exclusões, pausa, `decide`), `WindowInventory` + `redact`, a aplicação obrigatória da Política em toda captura (OT-001), a Pausa (bandeja + atalho), "Anexar tela ao abrir" com Chip bloqueado, o Registro de acesso e a seção "Privacidade" das Configurações.

Não inclui: Permissão do agente em ação (TK-004 usa `decide` com `Requester::Agent`), buffers (TK-005).

## Leitura em ordem

1. `CONTEXT.md` → Política de privacidade, Janela excluída, Pausa, Permissão do agente, Registro de acesso.
2. `specs/004-contexto-de-tela/plan.md` → `aura-policy`, "Data" (lista padrão), OT-001/OT-002/OT-005.
3. `crates/aura-capture/src/source.rs` (TK-001).
4. `apps/desktop/src-tauri/src/tray.rs` (001/002) → itens e badges.

## Decisões já resolvidas

- `AccessRequest{source, requester: User|Agent{tool, conversation}, target: Monitor|Window|Range, visible_windows}`; `decide` ordem: Pausa → Modo Desligado → alvo excluído (Deny) → janelas excluídas visíveis (AllowRedacted) → Permissão do agente (para `Agent`) → Allow.
- Regras casam processo (case-insensitive, curinga `*`), título (glob) e classe; regras padrão marcadas `builtin` (desativáveis, não removíveis).
- Redação: retângulo `DWMWA_EXTENDED_FRAME_BOUNDS` da janela excluída, interseção com o monitor, preenchido com `--surface-strong` + ícone de cadeado.
- Pausa persiste entre reinícios; atalho padrão `Ctrl+Shift+Alt+P` configurável.
- Liberdade local: layout da seção e do registro.

## Mapa de alterações

- Novo: `crates/aura-policy/{Cargo.toml,src/lib.rs,src/rules.rs,src/decide.rs,src/defaults.rs}`.
- Novo: `crates/aura-capture/src/windows.rs` (`WindowInventory`, `Win32Inventory`, `StaticInventory`), `src/redact.rs`.
- Existente: `crates/aura-capture/src/lib.rs` → API pública `capture(target, &Decision)`.
- Novo: `crates/aura-store/src/migrations/0004_capture.sql`, `access_log_repo.rs`, `exclusion_repo.rs`.
- Existente: `apps/desktop/src-tauri/src/capture.rs` → passa por `decide`; `tray.rs` → Pausa.
- Novo: `apps/desktop/src/privacy/{PrivacySection.tsx,ExclusionRules.tsx,AccessLog.tsx,PauseToggle.tsx}`.

## Contrato técnico

- Entradas: `Policy` (configurações), `Grants`, `AccessRequest`.
- Saídas: `Decision`; linha no `access_log` para toda requisição do agente e para toda captura anexada a turno.
- Invariantes: OT-001, OT-002.
- Erros: regra inválida (glob) → `PolicyError::InvalidRule`.
- Efeitos: ícone de bandeja muda na Pausa.

## Exemplos de aceite

- **AC-005** (unit `decide` + `redact`): janelas visíveis `[chrome.exe (0,0,960,1080), KeePassXC.exe (960,0,960,1080)]`, alvo monitor → `AllowRedacted([(960,0,960,1080)])`; `redact` sobre frame sintético → metade direita com cor de bloco; alvo `Window(KeePassXC)` → `Deny(Excluded)`; `KeePassXC` maximizado → `Deny(Excluded)`. Integração Windows com processo dublê renomeado `KeePassXC.exe` (janela simples) → PNG com metade coberta.
- **AC-006**: `paused=true` → `Deny(Paused)` para User e Agent; UI: Pausa pela bandeja e atalho alterna ícone.
- **AC-003**: "Anexar tela ao abrir" + Aplicativo anterior `Bitwarden.exe` → Chip bloqueado "Janela excluída: Bitwarden"; com Pausa → "Pausa de privacidade ativa".
- **AC-008**: capturar e enviar num turno → `access_log` com `{source:Screen, requester:User, decision:Allow|AllowRedacted, conversation}`; tela Registro lista com miniatura.

## Dependências e sequência de execução

Depende de: TK-001.

- [ ] TK-003.1 Unit `decide` (tabela de casos, um por vez) red→green.
- [ ] TK-003.2 Unit `redact` red→green.
- [ ] TK-003.3 `Win32Inventory` + integração dublê KeePassXC.
- [ ] TK-003.4 Pausa, anexar ao abrir, registro; Vitest.
- [ ] TK-003.5 Evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-policy -p aura-capture redact windows`; `cargo nextest run -p aura-capture --features win-integration exclusion`; `pnpm -C apps/desktop test -- privacy`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: runner sem desktop.

## Condição de retorno à planejadora

Retornar se janelas excluídas elevadas não forem enumeráveis/posicionáveis a partir de processo não elevado (exigiria bloquear captura quando houver janela elevada desconhecida).

## Relatório de saída

Relatar regras padrão finais, tabela de casos, EV refs.
