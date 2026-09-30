---
schema: hybrid/ticket
schema_version: 1.0
id: TK-005
effort: 008-extensoes-do-agente
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-004"]
requirement_refs: ["FR-004"]
acceptance_refs: ["AC-014"]
spec_revision: 1
plan_revision: 1
owned_areas: ["apps/desktop/src/conversation/ChangesPanel.tsx", "apps/desktop/src/conversation/FilesPanel.tsx", "apps/desktop/src/conversation/Preview.tsx", "apps/desktop/src-tauri/src/workspace_files.rs"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-005 — Painéis de Alterações e Arquivos gerados com prévia segura

## Objetivo e limites

Entrega o painel "Alterações" (diff agregado por turno via `turn/diff/updated` e itens `fileChange`) e o painel "Arquivos" (listagem do Workspace da conversa com abrir, revelar e prévia de imagem, Markdown, PDF e HTML em sandbox sem rede).

Não inclui: edição de arquivos no Aura.

## Leitura em ordem

1. Docs app-server: `turn/diff/updated`, item `fileChange`, `fs/readDirectory`, `fs/watch`.
2. `specs/008-extensoes-do-agente/plan.md` → OT-003.
3. `crates/aura-codex/src/mapping.rs` (002).

## Decisões já resolvidas

- Diff renderizado lado a lado ≥ 900 px, unificado abaixo disso; realce de sintaxe por extensão (Shiki sob demanda).
- Listagem via FS do host (não do app-server) restrita ao Workspace da conversa; atualização por `notify` (watcher) com debounce 300 ms.
- Prévia HTML: `iframe sandbox="allow-scripts"` sem `allow-same-origin`, CSP `default-src 'none'; img-src data: blob:; style-src 'unsafe-inline'; script-src 'unsafe-inline'`; carregada por `srcdoc`.
- Liberdade local: layout.

## Mapa de alterações

- Novo: `apps/desktop/src-tauri/src/workspace_files.rs` → `workspace_list`, `workspace_open`, `workspace_reveal`, evento `workspace_changed`.
- Novo: `apps/desktop/src/conversation/{ChangesPanel.tsx,FilesPanel.tsx,Preview.tsx}`.

## Contrato técnico

- Entradas: eventos de diff; FS do workspace.
- Saídas: painéis.
- Invariantes: OT-003; nenhum caminho fora do workspace é listado (canonicalização + prefixo).
- Erros: arquivo grande (> 5 MB) → sem prévia, só abrir.

## Exemplos de aceite

- **AC-014**: `turn/diff/updated` com diff de 2 arquivos → painel mostra ambos com +/−; workspace com `relatorio.md`, `grafico.png`, `pagina.html` → lista; prévia do HTML sem acesso de rede (teste: página com `fetch('https://example.com')` → bloqueado, verificado via console no teste E2E); tentativa de listar `..\\` → rejeitada.

## Dependências e sequência de execução

Depende de: TK-004.

- [ ] TK-005.1 `workspace_list` com canonicalização (unit) red→green.
- [ ] TK-005.2 Painel de alterações (Vitest com diff fixture).
- [ ] TK-005.3 Prévias + teste de sandbox; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-desktop workspace_files`; `pnpm -C apps/desktop test -- ChangesPanel FilesPanel Preview`; `pnpm -C apps/desktop e2e -- --spec e2e/preview-sandbox.spec.ts`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: nenhum.

## Condição de retorno à planejadora

Retornar se o WebView2 não respeitar CSP em `srcdoc` de iframe sandbox.

## Relatório de saída

Relatar resultados e EV refs.
