---
schema: hybrid/change
schema_version: 1.0
effort_id: 018-modo-yolo
revision: 1
status: closed
profile: compact
---

# Change: Modo YOLO: Tarefa sem pedidos de permissão

## Objetivo e limites

Pedido do usuário em 05/10/2026: uma opção "You Only Live Once" (YOLO) para a IA aceitar tudo sem pedir permissão. Configurada em Configurações, com aviso, e só liga quando o usuário escreve `ACEITO` (pt-BR) ou `ACCEPT` (en). Ligada, a IA não pede mais permissão no modo Tarefa: faz tudo o que precisa.

Fora do escopo: modos Chat e Plano (continuam somente leitura); perguntas do agente (`requestUserInput` com perguntas) e formulários MCP que pedem dados — não são permissões e continuam aparecendo; consentimento de privacidade das fontes de captura (tela/áudio), que segue a política de privacidade (ADR de privacidade, 004).

## Contrato de comportamento

- Entradas: `Settings.yolo` (só muda pelo comando `yolo_set(enabled, confirmation)`, nunca por patch); modo da conversa; pedidos do app-server (`item/commandExecution/requestApproval`, `item/fileChange/requestApproval`, `item/permissions/requestApproval`, `mcpServer/elicitation/request` com `_meta.codex_approval_kind = "mcp_tool_call"`).
- Saída:
  - Ligar exige `confirmation` igual a `ACEITO` ou `ACCEPT` (sem diferenciar maiúsculas, espaços nas pontas ignorados); desligar não exige nada.
  - Com YOLO ligado, conversas em Tarefa rodam com `sandbox = "danger-full-access"` / `sandboxPolicy = {"type": "dangerFullAccess"}` e `approvalPolicy = "never"`, valendo já no próximo turno de conversas existentes.
  - Pedidos de aprovação que ainda chegarem numa conversa em Tarefa são aceitos pelo host sem aparecer na tela: comando e alteração de arquivos → `accept`; permissões → todas as pedidas, na sessão; ferramenta MCP → aceita.
  - Configurações › Geral: seção "Modo YOLO" com aviso dos riscos; ligar abre um campo para digitar a palavra e o botão "Ligar YOLO" só habilita com a palavra certa; ligado, mostra estado e "Desligar".
  - Overlay: em Tarefa com YOLO ligado, o selo do modo mostra "Tarefa · YOLO" em destaque de perigo.
- Erros/invariantes: palavra errada → erro `invalid` e nada muda; Chat e Plano nunca recebem `danger-full-access` nem aceite automático; perguntas do agente nunca são respondidas sozinhas.
- Compatibilidade: com YOLO desligado (padrão) nada muda; configurações antigas sem o campo carregam com `yolo = false`.

## Requisitos e aceite

- **FR-001** — Ligar o YOLO MUST exigir a palavra de confirmação.
- **AC-001** — `yolo_set(true, "talvez")` → erro `invalid`, `yolo = false`; `yolo_set(true, " aceito ")` → `yolo = true`; `yolo_set(true, "ACCEPT")` também; `yolo_set(false, "")` → `yolo = false`; patch de configurações não altera `yolo`.
- **FR-002** — Com YOLO, o modo Tarefa MUST rodar sem pedir permissão.
- **AC-002** — `thread_params`/`turn_overrides` de Tarefa com YOLO devolvem `danger-full-access`/`dangerFullAccess` e `never`; Chat e Plano com YOLO continuam `read-only`/`on-request`.
- **AC-003** — Numa conversa em Tarefa com YOLO, um pedido de execução de comando é respondido `accept` sem evento para a UI; com YOLO desligado ou em Chat, o evento chega e nada é respondido; uma pergunta do agente sempre chega à UI.
- **AC-004** — App-server fixado real aceita os parâmetros de Tarefa com YOLO (`thread/start` + `turn/start`).
- **FR-003** — A UI MUST avisar e mostrar o estado.
- **AC-005** — Em Configurações, "Ligar YOLO" fica desabilitado até digitar "ACEITO" (pt-BR) / "ACCEPT" (en); ligado, o Overlay em Tarefa mostra "Tarefa · YOLO".

## Leitura e mapa de alterações

- `crates/aura-core/src/settings.rs` → `Settings.yolo`; existing.
- `crates/aura-codex/src/modes.rs` → `thread_params`, `turn_overrides` com `yolo`; existing.
- `crates/aura-codex/src/approvals.rs` → `PendingRequests::yolo_decision`; existing.
- `crates/aura-codex/src/service.rs` → `set_yolo`, aceite automático no roteador; existing.
- `crates/aura-app/src/host.rs` → `set_yolo`; shell `yolo_set`; existing.
- `apps/desktop/src/settings/General.tsx` (seção YOLO), `overlay/ModelPicker.tsx` (`ModeBadge`), `ipc/*`, `i18n/*`; existing.

## Plano breve

Seam: funções puras de modo, `PendingRequests`, serviço contra app-server falso, Host, Vitest, app-server real. Abordagem: flag no serviço lida a cada turno/pedido; palavra validada no host. Dependências: none.

## Sequência e tarefas

- [x] C-001 Settings + `set_yolo` (red/green).
- [x] C-002 Parâmetros de modo e aceite automático no serviço.
- [x] C-003 UI (Configurações, selo) e contrato.
- [x] C-004 Regressão, app-server real, evidência.

## Validação e evidência

Comando/procedimento: `cargo test --workspace --exclude aura-desktop`, `cargo clippy --workspace --all-targets -- -D warnings`, `pnpm -C apps/desktop test`, `pnpm -C apps/desktop typecheck`, app-server real ignorado.

Resultado executado: EV-001 (AC-001..AC-005) passou. Rust 330 passed / 6 ignored, clippy `-D warnings` limpo (workspace + `aura-desktop`), fmt limpo, contrato dourado atualizado (`yolo` em Settings); UI 191/191 e typecheck. App-server fixado real (rust-v0.159.0): `real_app_server_runs_task_mode_with_yolo` verde (`danger-full-access` + `never` em `thread/start` e `turn/start`).

Limitações: o aceite automático foi exercitado com o app-server falso (pedido de comando); o mapeamento dos demais tipos (alteração de arquivos, permissões, ferramenta MCP; perguntas nunca) está em `PendingRequests::register`, sem teste unitário dedicado (a inclusão desse teste foi bloqueada pelo classificador de permissões nesta sessão). Sem conta ChatGPT, nenhum turno real do plano em YOLO.

## Condição de retorno

Retornar se o contrato, símbolo, dependência ou decisão material não puder ser satisfeito sem ampliar o escopo.

## Estado

`change.md` é a fonte canônica deste esforço compacto. Não crie `tasks.md` ou tickets paralelos para esta mudança.
