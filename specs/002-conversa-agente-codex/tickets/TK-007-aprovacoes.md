---
schema: hybrid/ticket
schema_version: 1.0
id: TK-007
effort: 002-conversa-agente-codex
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-009"]
requirement_refs: ["FR-007"]
acceptance_refs: ["AC-018", "AC-019", "AC-020", "AC-021"]
spec_revision: 2
plan_revision: 2
owned_areas: ["crates/aura-codex/src/approvals.rs", "apps/desktop/src/conversation/approvals", "apps/desktop/src-tauri/src/tray.rs"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-007 — Aprovações, perguntas do agente e elicitation MCP

## Objetivo e limites

Entrega o tratamento de server requests (`item/commandExecution/requestApproval`, `item/fileChange/requestApproval`, `item/permissions/requestApproval`, `item/tool/requestUserInput`, `mcpServer/elicitation/request`) como `ApprovalRequested`/`UserInputRequested`, os cartões na UI com atalhos, regras "nesta conversa" e o indicador de pendência na bandeja.

Não inclui: política de permissão de captura (004 TK-003 — Permissão do agente é outro conceito).

## Leitura em ordem

1. Docs atuais do app-server → "Command execution approvals", "File change approvals", "`tool/requestUserInput`", "Permission requests", "MCP server elicitation requests", `serverRequest/resolved`.
2. `specs/002-conversa-agente-codex/plan.md` → OT-004.
3. `crates/aura-codex/src/rpc.rs` → `server_requests()`/`Responder` (TK-001).
4. `docs/design/ui-ux.md` → "Cartão de Aprovação".

## Decisões já resolvidas

- Decisões enviadas exatamente nos formatos do protocolo: `accept`, `acceptForSession`, `decline`, `cancel`; permissões respondem apenas o subconjunto concedido com `scope:"turn"|"session"`.
- "Aceitar nesta conversa" = `acceptForSession` (o app-server guarda a regra); o Aura não mantém regras próprias paralelas.
- Pendências vivem no host (`PendingRequests`) e sobrevivem ao esconder o Overlay; são limpas em `serverRequest/resolved`.
- Elicitation `url` abre o navegador somente após clique explícito.
- Liberdade local: layout do diff resumido (arquivos + contagem de linhas + expandir).

## Mapa de alterações

- Novo: `crates/aura-codex/src/approvals.rs` → `PendingRequests`, `Decision`, `PermissionGrant`, mapeamento server request → evento.
- Existente: `service.rs` → `respond(request_id, decision)`.
- Novo: `apps/desktop/src/conversation/approvals/{ApprovalCard.tsx,DiffSummary.tsx,UserInputForm.tsx,ElicitationForm.tsx}`.
- Existente: `apps/desktop/src-tauri/src/tray.rs` → badge de pendência.

## Contrato técnico

- Entradas: server requests do app-server.
- Saídas: eventos para UI; respostas JSON-RPC ao app-server.
- Invariantes: OT-004 (nenhuma resposta sem ação do usuário); uma resposta por request.
- Erros: responder request já resolvido → ignorado com log `debug`.
- Efeitos: badge de bandeja enquanto `PendingRequests` não vazio.

## Exemplos de aceite

- **AC-018**: falso envia `requestApproval{command:"pip install requests", cwd:"C:\\…\\workspaces\\x", reason:"instalar dependência"}` → cartão com os três campos; clicar "Aceitar" → resposta `accept`; "Recusar" → `decline` e item `declined`; "Aceitar nesta conversa" → `acceptForSession`.
- **AC-019**: `fileChange` com 2 arquivos (`a.md` +3/−1, `b.txt` +10) → cartão lista ambos com contagens; expandir mostra diff.
- **AC-020**: `requestUserInput` com 2 perguntas (uma com opções, uma `isOther`) e `autoResolutionMs: 30000` → formulário com contagem regressiva; responder envia respostas; sem resposta em 30 s → UI mostra "resolvido automaticamente" ao receber `serverRequest/resolved`. Elicitation `form` com schema `{nome: string}` → formulário gerado.
- **AC-021**: request pendente + Overlay oculto → ícone da bandeja com badge; reabrir → cartão visível com foco no botão "Recusar" (padrão seguro); nenhuma resposta enviada até ação.

## Dependências e sequência de execução

Depende de: TK-009 (Modo Tarefa torna aprovações observáveis no manual real).

- [ ] TK-007.1 Contrato AC-018 red→green.
- [ ] TK-007.2 AC-019, AC-020 (um por vez).
- [ ] TK-007.3 AC-021 + badge.
- [ ] TK-007.4 UI + Vitest (atalhos `A`/`R`).
- [ ] TK-007.5 Manual real em Modo Tarefa; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-codex approvals`; `pnpm -C apps/desktop test -- approvals`; roteiro manual.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: nenhum.

## Condição de retorno à planejadora

Retornar se algum tipo de server request da versão fixada não tiver representação segura na UI (por exemplo, formatos de permissão novos).

## Relatório de saída

Relatar tipos cobertos, resultados, prints, EV refs.
