---
schema: hybrid/change
schema_version: 1.0
effort_id: 017-modelos-gpt6-extensoes-agenticas
revision: 1
status: closed
profile: compact
---

# Change: Modelos GPT-6, seletor antes da conversa, extensões com IA e busca nas Configurações

## Objetivo e limites

Pedido do usuário em 05/10/2026:

1. O plano ChatGPT mostra GPT-5.6 e anteriores; manter **apenas** GPT-6 Luna (`gpt-6-luna`), GPT-6.1 Sol (`gpt-6.1-sol`) e GPT-6 Astra (`gpt-6-astra`).
2. Não dá para escolher modelo e esforço de raciocínio antes de iniciar a conversa (o Overlay compacto só mostra o modo).
3. Extensões: buscar, habilitar/desabilitar em massa Skills, servidores MCP e Comandos rápidos; busca geral nas Configurações.
4. Criar Skills, Comandos rápidos e servidores MCP com a IA, em modo agêntico (Tarefa).
5. Skills obrigatórias para o próprio agente (criar Skill, Comando rápido, servidor MCP), invisíveis para o usuário e sem como desativar.
6. `@` e `/`: com uma opção destacada, Enter deve escolhê-la; `/` ou `@` soltos nunca vão para o chat.

Fora do escopo: atualizar o app-server fixado; provedores BYOK (descoberta continua igual); editar Skills de sistema do Codex; o agente nunca recebe nem grava segredos.

## Contrato de comportamento

- Entradas: lista `/v1/models` do plano + `model/list`; seletor do Overlay; barra de entrada com menus; Configurações › Extensões; busca das Configurações; ferramentas MCP `aura.*` chamadas pelo agente.
- Saída:
  - Plano ChatGPT: só os três modelos, nessa ordem (Luna, Sol, Astra), mesmo que o servidor liste outros; os que o servidor não lista são acrescentados. Esforços vêm do `model/list` quando ele conhece o modelo; senão do catálogo do Aura. Padrão: o primeiro dos três que o servidor marcar como listado (Luna quando todos aparecem). Um "Modelo padrão" salvo que não é mais oferecido é ignorado; a conversa sempre começa com um modelo explícito do catálogo.
  - O Overlay compacto (antes da conversa) mostra um seletor "modelo · esforço · modo"; o mesmo painel do Overlay expandido (provedor, modelo, esforço em botões, modo). O esforço aparece no rótulo.
  - Menus `@`/`/`: o primeiro item já vem destacado; Enter ou Tab escolhem o destacado, inclusive com o gatilho sozinho. Um comando completo digitado (`/compactar`) continua executando no Enter. Texto que é só `/` ou `@` nunca é enviado nem posto na fila.
  - Extensões: campo de busca (nome, descrição, comando/URL, sem acento), contagem "x de y ativas" e botões "Ativar todas"/"Desativar todas" que agem só sobre os itens filtrados, por seção (Skills, Servidores MCP, Comandos rápidos). Servidor MCP pode ser editado (segredo em branco mantém o atual).
  - "Criar com IA" em cada seção abre um campo "Descreva o que você quer"; enviar abre o Overlay, inicia uma nova conversa em modo Tarefa e manda o pedido ao agente.
  - Ferramentas MCP do Aura para o agente: `extensions_list` (somente leitura), `skill_save`, `quick_command_save`, `mcp_server_save`. As de escrita pedem aprovação do usuário (Codex `approval_mode = "prompt"`). Servidor que precisa de segredo é salvo desligado, e o agente orienta o usuário a informar o segredo em Configurações › Extensões. Toda gravação avisa as janelas (`extensionsChanged`).
  - Skills internas (`aura-criar-skill`, `aura-criar-comando-rapido`, `aura-adicionar-servidor-mcp`) ficam em `<AURA_HOME>/core-skills`, são regravadas a cada início, entram nas raízes de Skills do Codex e não aparecem no catálogo do usuário nem podem ser desativadas.
  - Configurações: campo "Buscar configurações" (Ctrl+K ou Ctrl+F) acima do menu lateral; resultados por página com o texto encontrado; escolher um resultado abre a página e destaca o item.
- Erros/invariantes: nomes inválidos devolvem o erro da validação existente (`skill`, `quick`, `mcp`); o nome `aura` e Skills internas são reservados; nenhuma ferramenta aceita valores de segredo.
- Compatibilidade: provedores BYOK, preferências de esforço (013), modo visível no compacto (015 AC-007), fila durante a resposta, Ctrl+Enter, Ctrl+Shift+Enter.

## Requisitos e aceite

- **FR-001** — O plano ChatGPT MUST oferecer só GPT-6 Luna, GPT-6.1 Sol e GPT-6 Astra.
- **AC-001** — Dada a lista do servidor `gpt-5.6-sol, gpt-6-luna, gpt-5.5`, então o catálogo é exatamente `GPT-6 Luna, GPT-6.1 Sol, GPT-6 Astra`, Luna é o padrão; com `model/list` trazendo Luna com esforços `low, medium, high`, Luna oferece exatamente esses; com o padrão salvo `gpt-5.5`, a nova conversa começa com `gpt-6-luna`.
- **FR-002** — Modelo e esforço MUST ser escolhidos antes da conversa.
- **AC-002** — No Overlay compacto sem conversa, o seletor mostra "GPT-6 Luna · Chat"; escolher GPT-6 Astra e "Alto" faz o primeiro turno sair com `model = gpt-6-astra`, `effort = high`, e o rótulo passa a "GPT-6 Astra · Alto · Chat".
- **FR-003** — Enter MUST escolher o item destacado dos menus `@` e `/`.
- **AC-003** — Com "/" e Enter, o texto vira "/plano " (pt-BR) e nada é enviado; com "@" e Enter, a captura de tela é pedida e o texto fica vazio; com "/compactar" e Enter numa conversa, a compactação roda; "fala com o @joao" + Enter envia o texto.
- **FR-004** — Extensões MUST ter busca e ações em massa.
- **AC-004** — Com Skills `revisar-contrato` e `resumir-ata`, buscar "ata" mostra só `resumir-ata`; "Desativar todas" desativa só ela; a contagem mostra "1 de 2 ativas".
- **FR-005** — O usuário MUST poder pedir à IA para criar Skill, Comando rápido ou servidor MCP.
- **AC-005** — Em Skills, "Criar com IA" + "revisa contratos de aluguel" + Enviar chama `agent_task` com modo `task` e um texto que contém o pedido; o Overlay inicia uma conversa nova em Tarefa e envia esse texto.
- **FR-006** — O agente MUST ter ferramentas e Skills internas para criar extensões, com aprovação.
- **AC-006** — `skill_save {name: "resumir-ata", description: "Resume atas", instructions: "…"}` cria a Skill do Aura; `quick_command_save {name: "formal", template: "Reescreva formal: {texto}"}` cria o comando; `mcp_server_save` stdio com `secret_env: ["GITHUB_TOKEN"]` salva o servidor desligado e a resposta diz para informar o segredo nas Configurações; `name: "aura"` devolve erro; o `config.toml` gerado tem `approval_mode = "prompt"` para as três ferramentas de escrita e o app-server fixado aceita esse config.
- **AC-007** — As Skills internas existem em `core-skills` após iniciar, estão nas raízes passadas ao Codex e não aparecem em `skills_catalog`; `set_skill_enabled` numa delas devolve erro `skill`.
- **FR-007** — As Configurações MUST ter busca.
- **AC-008** — Buscar "microfone" lista resultados da página Voz; escolher um abre Voz; Ctrl+K foca a busca; "xyzw" mostra "Nada encontrado".

## Leitura e mapa de alterações

- `crates/aura-core/src/model_catalog.rs` → `KNOWN` (três modelos, limites opcionais); existing.
- `crates/aura-codex/src/models.rs` → `parse_plan_models` (só o catálogo); existing.
- `crates/aura-gateway/src/discovery.rs` → limites opcionais; existing.
- `crates/aura-mcp/src/tools.rs` → `EXTENSIONS_LIST`, `SKILL_SAVE`, `QUICK_COMMAND_SAVE`, `MCP_SERVER_SAVE`, `WRITE_TOOLS`; existing.
- `crates/aura-app/src/tools.rs` → `ExtensionsAccess`, handlers; `core_skills.rs` + `core-skills/*/SKILL.md`; new.
- `crates/aura-app/src/host.rs` → raízes de Skills, filtro do catálogo, `agent_task`, eventos; `events.rs` → `AgentTask`, `ExtensionsChanged`; existing.
- `crates/aura-codex/src/home.rs` → `approval_mode = "prompt"` por ferramenta; existing.
- `apps/desktop/src-tauri/src/{commands,main}.rs` → `agent_task`; existing.
- `apps/desktop/src/overlay/{InputBar,ModelPicker,Header,session,OverlayApp}.tsx`, `settings/{Extensions,SettingsApp,SettingsSearch}.tsx`, `ipc/*`, `i18n/*`; existing/new.

## Plano breve

Seam: funções puras Rust (`parse_plan_models`, `render`, handlers de `HostTools` com `ExtensionsAccess` falso), Host com app-server falso, Vitest (InputBar, ModelPicker compacto, Extensões, busca), contrato dourado, app-server fixado real para o `config.toml`. Abordagem: catálogo curado no núcleo; ferramentas de escrita no servidor MCP do Aura com aprovação do Codex; Skills internas como raiz extra filtrada do catálogo; pedido "Criar com IA" via evento do host para o Overlay. Dependências: none.

## Sequência e tarefas

- [x] C-001 Catálogo GPT-6 (red/green Rust) + modelo explícito no início.
- [x] C-002 Seletor no compacto e menus `@`/`/` (Vitest).
- [x] C-003 Ferramentas MCP, Skills internas, `agent_task` (Rust + contrato).
- [x] C-004 Extensões: busca, massa, edição MCP, Criar com IA; busca nas Configurações (Vitest).
- [x] C-005 Regressão, app real e evidência.

## Validação e evidência

Comando/procedimento: `cargo test --workspace --exclude aura-desktop`, `cargo clippy --workspace --all-targets -- -D warnings`, `pnpm -C apps/desktop test`, `pnpm -C apps/desktop typecheck`, `UPDATE_GOLDEN=1 cargo test -p aura-app --test ipc_contract`.

Resultado executado: EV-001 (AC-001..AC-008) passou. Rust 327 passed / 5 ignored, clippy `-D warnings` limpo (workspace + `aura-desktop` com sidecars preparados), `rustfmt --check` limpo, contrato dourado atualizado; UI 188/188, typecheck e `vite build`. App-server fixado real (rust-v0.159.0): `real_app_server_asks_before_aura_write_tools` (config com `approval_mode = "prompt"` aceito; elicitation `mcp_tool_call` antes de `quick_command_save`; gravado só após aceitar) e `real_app_server_byok_turn_and_mcp_tool` verdes.

Limitações: sem conta ChatGPT no ambiente, os modelos do plano são verificados pelas funções de catálogo (esforços de Luna/Astra vêm do `model/list` real; o catálogo do Aura só é usado se o servidor não os descrever). O agente real criando extensões depende de um turno com o plano (coberto com modelo simulado no app-server real). Jornadas E2E nativas (`composer.e2e.ts` atualizada) não rodaram: `tauri-driver`/`msedgedriver` ausentes nesta máquina. Achado: o teste de paginação do Histórico já levava ~5,1 s nesta máquina antes da mudança; recebeu timeout de 20 s.

## Condição de retorno

Retornar se o contrato, símbolo, dependência ou decisão material não puder ser satisfeito sem ampliar o escopo.

## Estado

`change.md` é a fonte canônica deste esforço compacto. Não crie `tasks.md` ou tickets paralelos para esta mudança.
