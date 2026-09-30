---
schema: hybrid/ticket
schema_version: 1.0
id: TK-009
effort: 002-conversa-agente-codex
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-003"]
requirement_refs: ["FR-009"]
acceptance_refs: ["AC-024", "AC-025"]
spec_revision: 2
plan_revision: 2
owned_areas: ["crates/aura-codex/src/modes.rs", "apps/desktop/src/conversation/ModeSwitch.tsx", "apps/desktop/src/conversation/FolderGrants.tsx"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-009 — Modos Chat e Tarefa com Workspace da conversa

## Objetivo e limites

Entrega a escolha de modo por Conversa, o mapeamento para `sandbox`/`approvalPolicy`/`cwd`/`writableRoots`/`developerInstructions`, a concessão de pastas no Modo Tarefa, o toggle de rede para comandos e a verificação/configuração do sandbox do Windows.

Não inclui: modo plano (008), subagentes.

## Leitura em ordem

1. `specs/002-conversa-agente-codex/plan.md` → "Mapeamento de modos".
2. Docs atuais: https://developers.openai.com/codex/windows (sandbox `elevated`/`unelevated`), app-server `windowsSandbox/readiness`, `windowsSandbox/setupStart`, `sandboxPolicy` no `turn/start`.
3. `crates/aura-codex/src/service.rs` → `start` (TK-003).

## Decisões já resolvidas

- Modo escolhido no cabeçalho antes do primeiro turno; mudar de modo depois = aplicar overrides no próximo `turn/start` (`sandboxPolicy`, `approvalPolicy`) e registrar em `conversations_meta.mode`.
- Pastas concedidas via seletor nativo; lista por Conversa; remover concessão vale do próximo turno em diante.
- Sandbox padrão `unelevated`; se `windowsSandbox/readiness` indicar `notConfigured`, mostrar cartão "Preparar sandbox" que chama `setupStart{mode:"unelevated"}`; oferta de `elevated` em Configurações > Avançado com explicação de UAC.
- Liberdade local: textos e layout.

## Mapa de alterações

- Novo: `crates/aura-codex/src/modes.rs` → `ConversationMode{Chat, Task{granted: Vec<PathBuf>, network: bool}}`, `fn thread_params(mode, workspace) -> ThreadStartOverrides`, `fn turn_overrides(mode, workspace)`.
- Existente: `service.rs` → `start(opts{mode})`, `set_mode`.
- Novo: `apps/desktop/src/conversation/{ModeSwitch.tsx,FolderGrants.tsx,SandboxSetupCard.tsx}`.

## Contrato técnico

- Entradas: modo, pastas, toggle de rede.
- Saídas: parâmetros de thread/turn conforme tabela do plano.
- Invariantes: Chat nunca envia `workspaceWrite`; Tarefa nunca inclui raiz fora de `[workspace] ∪ granted`.
- Erros: pasta inexistente/sem acesso → `ModeError::InvalidGrant(path)`.
- Efeitos: `workspaces/<id>` criado antes do primeiro turno.

## Exemplos de aceite

- **AC-024** (unit + contrato): `thread_params(Chat, W)` → `{sandbox:"readOnly", approvalPolicy:"onRequest", cwd:W}`; `turn_overrides(Task{granted:[D], network:false}, W)` → `sandboxPolicy{type:"workspaceWrite", writableRoots:[W, D], networkAccess:false}`. Manual real: no Chat, "crie hello.txt na minha Área de Trabalho" → nenhum arquivo criado; na Tarefa com Área de Trabalho concedida → cartão de Aprovação e arquivo criado após aceitar.
- **AC-025**: nova Conversa + primeiro envio → `workspaces/<thread_id>` existe e é o `cwd` do `thread/start`.

## Dependências e sequência de execução

Depende de: TK-003.

- [ ] TK-009.1 Unit `thread_params`/`turn_overrides` (red→green por caso).
- [ ] TK-009.2 Contrato AC-025.
- [ ] TK-009.3 UI de modo e pastas + Vitest.
- [ ] TK-009.4 Sandbox readiness/setup; manual Win11.
- [ ] TK-009.5 Evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-codex modes`; `pnpm -C apps/desktop test -- ModeSwitch`; roteiro manual.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: política corporativa bloqueando setup de sandbox → registrar e testar `unelevated`.

## Condição de retorno à planejadora

Retornar se o sandbox `unelevated` falhar em Windows 10 22H2 sem alternativa, ou se `writableRoots` não aceitar pastas fora do `cwd`.

## Relatório de saída

Relatar símbolos, matriz de modos testada, EV refs.
