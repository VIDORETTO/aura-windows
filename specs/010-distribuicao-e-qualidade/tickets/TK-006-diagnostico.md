---
schema: hybrid/ticket
schema_version: 1.0
id: TK-006
effort: 010-distribuicao-e-qualidade
type: delivery
status: implemented
ticket_revision: 3
requires: []
requirement_refs: ["FR-006"]
acceptance_refs: ["AC-011"]
spec_revision: 1
plan_revision: 1
owned_areas: ["apps/desktop/src-tauri/src/diagnostics.rs", "apps/desktop/src/diagnostics"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-006 — Painel de diagnóstico e exportação redigida

## Objetivo e limites

Entrega `DiagnosticsReport` (estado de app-server, Gateway, MCP, worker, capturas, conta sem tokens, disco, versões), a tela Diagnóstico e `export_zip` com logs redigidos e configuração sem segredos, seguido de varredura final de padrões de segredo.

Não inclui: envio automático a servidor.

## Leitura em ordem

1. `crates/aura-core/src/logging.rs` (001 TK-006) → `RedactionLayer`.
2. Estados expostos por `AppServerSupervisor` (002), `GatewayHandle` (002/003), `WorkerClient` (006), `aura-capture` (004/005), `AuthService` (002).
3. `specs/010-distribuicao-e-qualidade/plan.md` → OT-002.

## Decisões já resolvidas

- Conteúdo do zip: `report.json`, `logs/*.log` (últimos 5 arquivos), `config.redacted.toml`, `settings.redacted.json`; nunca rollouts, capturas, anexos, transcrições.
- Varredura final: regex `sk-[A-Za-z0-9_-]{10,}`, `eyJ[A-Za-z0-9_-]{10,}\.`, `Bearer\s+\S+`, `oaiapp_\S+` → substituir por `[REDACTED]` e registrar contagem.
- Liberdade local: layout.

## Mapa de alterações

- Novo: `apps/desktop/src-tauri/src/diagnostics.rs`; `apps/desktop/src/diagnostics/DiagnosticsView.tsx`.

## Contrato técnico

- Entradas: estado dos serviços.
- Saídas: tela e `.zip`.
- Invariantes: OT-002.

## Exemplos de aceite

- **AC-011**: com serviços simulados (app-server `Ready 0.159.0`, Gateway porta X, worker `Stopped`) → relatório com esses campos; log contendo `Bearer abc.def.ghi` e `sk-live-123456789012` → zip com `[REDACTED]` e nenhum match dos padrões (teste abre o zip e varre); zip não contém arquivos de `codex-home/sessions` nem `captures/`.

## Dependências e sequência de execução

Depende de: 001-fundacao-overlay/TK-006 (outro esforço).

- [ ] TK-006.1 Varredura/redação do zip (unit) red→green.
- [ ] TK-006.2 Relatório com serviços simulados.
- [ ] TK-006.3 UI; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-desktop diagnostics`; `pnpm -C apps/desktop test -- DiagnosticsView`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: nenhum.

## Condição de retorno à planejadora

Nenhuma prevista.

## Relatório de saída

Relatar resultados e EV refs.
