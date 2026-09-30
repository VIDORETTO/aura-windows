---
schema: hybrid/spec
schema_version: "1.0"
effort_id: 008-extensoes-do-agente
revision: 1
status: accepted
profile: standard
---

# Specification: Extensões do agente

## Problem and desired result

Um agente só vira ferramenta de trabalho quando o usuário pode ensiná-lo procedimentos, conectá-lo aos seus sistemas e acompanhar o que ele faz em tarefas longas. O resultado é gerenciar Skills e Servidores MCP pela interface, invocar Skills e Comandos rápidos com `/`, planejar antes de executar, acompanhar progresso, alterações e arquivos gerados, e controlar Memórias e instruções pessoais.

## Consumers and actors

- Usuário avançado (configura) e usuário comum (invoca `/`).
- Codex app-server (skills, MCP, planos, diffs, memórias).
- Servidores MCP de terceiros.

## Scope

### Included

- Skills: listar, ativar/desativar, importar (pasta/zip) com revisão, criar/editar, invocar por `/` e implicitamente.
- Servidores MCP: adicionar stdio/HTTP, OAuth, status, ferramentas com liga/desliga, modo de aprovação por servidor, importação de configurações de outros apps.
- Comandos rápidos embutidos e personalizados com variáveis de contexto.
- Modo plano, painel de progresso, alterações do agente (diffs) e arquivos gerados.
- Memórias (ativar, revisar, excluir) e instruções pessoais.

### Excluded

- Marketplace público de skills/plugins do Aura (futuro); plugins do ecossistema ChatGPT/Codex (avaliar depois).
- Subagentes/monitores em background (futuro).

## User journeys and scenarios

### US-001 — Skills (Priority: P1)

#### Acceptance scenarios

- **AC-001** — Dado a tela Skills, quando aberta, então lista cada Skill com nome, descrição, origem (Aura, Usuário `~/.agents/skills`, Sistema) e chave ativar/desativar; desativar remove a Skill das Conversas novas.
- **AC-002** — Dado uma pasta ou `.zip` com `SKILL.md`, quando o usuário importa, então o Aura valida `name` e `description`, mostra o conteúdo completo e os scripts incluídos para revisão com o aviso "Skills podem instruir o agente a executar comandos", e só instala após confirmação; inválida → motivo.
- **AC-003** — Dado o editor de Skill, quando o usuário cria "revisar-contrato" com descrição e instruções e salva, então ela aparece na lista e no menu `/` sem reiniciar.
- **AC-004** — Dado o menu `/`, quando o usuário digita parte do nome e escolhe uma Skill, então o turno é enviado com a Skill explicitamente anexada e o Chip "Skill: revisar-contrato" aparece na mensagem.

### US-002 — Servidores MCP (Priority: P1)

#### Acceptance scenarios

- **AC-005** — Dado "Adicionar servidor" stdio (comando, argumentos, variáveis), quando salvo, então o status mostra "conectado" com a lista de ferramentas, ou "erro" com o motivo e as últimas linhas de log do servidor.
- **AC-006** — Dado um servidor HTTP que exige OAuth, quando o usuário clica "Conectar", então o navegador abre a autorização e, ao concluir, o status fica "conectado"; tokens ficam no cofre do Windows.
- **AC-007** — Dado a lista de ferramentas de um servidor, quando o usuário desliga uma ferramenta, então ela não fica disponível ao agente nas Conversas novas.
- **AC-008** — Dado o modo de aprovação do servidor ("sempre perguntar", "perguntar para escrita", "automático"), quando o agente chama uma ferramenta, então a Aprovação (002) aparece conforme o modo; ferramentas marcadas como destrutivas sempre pedem Aprovação.
- **AC-009** — Dado configurações MCP de outros apps no computador (Claude Desktop, Cursor, VS Code, Codex CLI), quando o usuário escolhe "Importar", então vê os servidores detectados, seleciona e importa sem copiar segredos em texto claro (segredos vão ao cofre).

### US-003 — Comandos rápidos (Priority: P1)

#### Acceptance scenarios

- **AC-010** — Dado o menu `/`, quando o usuário escolhe um Comando rápido embutido (`/tldr`, `/traduzir <idioma>`, `/reescrever`, `/explicar`, `/corrigir`, `/resumir-tela`), então o prompt pronto é aplicado ao contexto disponível (seleção, tela, anexos ou texto digitado), e o texto enviado mostra o comando de forma compacta.
- **AC-011** — Dado o editor de Comandos rápidos, quando o usuário cria `/email-formal` com o modelo "Reescreva de forma formal: {selecao}", então ao usar o comando com uma seleção ativa o turno recebe o modelo preenchido; sem seleção, usa o texto digitado.

### US-004 — Planejar e acompanhar (Priority: P2)

#### Acceptance scenarios

- **AC-012** — Dado o Modo plano (botão ou `/plano`), quando o usuário pede uma tarefa, então o agente pode ler/pesquisar mas não executar comandos nem alterar arquivos, e apresenta um plano com ações "Executar plano", "Continuar planejando" e "Sair do modo plano".
- **AC-013** — Dado um turno com plano, quando o agente atualiza etapas, então o painel Progresso mostra cada etapa com estado (pendente, em andamento, concluída) em tempo real.
- **AC-014** — Dado um turno no Modo Tarefa que alterou arquivos, quando o usuário abre "Alterações", então vê o diff por arquivo; em "Arquivos", vê os arquivos gerados no Workspace da conversa com abrir, revelar na pasta e prévia (imagens, Markdown, HTML em sandbox sem rede, PDF).

### US-005 — Memórias e instruções (Priority: P2)

#### Acceptance scenarios

- **AC-015** — Dado Memórias ativadas, quando o usuário revisa a tela Memórias, então vê o que o agente guardou, pode editar ou excluir itens e desativar a função; com Memórias desativadas nada novo é guardado.
- **AC-016** — Dado "Instruções pessoais" preenchidas (ex.: "responda em português, seja direto"), quando uma Conversa nova começa, então essas instruções são aplicadas junto à Persona; por Conversa, o usuário pode acrescentar instruções extras.

## Requirements

- **FR-001** — O sistema MUST gerenciar Skills (listar, ativar/desativar, importar com revisão, criar/editar) e permitir invocação explícita por `/`.
- **FR-002** — O sistema MUST gerenciar Servidores MCP (stdio/HTTP, OAuth, status, ferramentas, modos de aprovação, importação) com segredos no cofre.
- **FR-003** — O sistema MUST oferecer Comandos rápidos embutidos e personalizados com variáveis de contexto.
- **FR-004** — O sistema MUST oferecer Modo plano, painel de progresso, alterações e arquivos gerados.
- **FR-005** — O sistema MUST permitir controlar Memórias e instruções pessoais.

## Limits, errors, and compatibility

- Skills seguem o padrão agentskills.io; o Aura não executa scripts de Skills fora do sandbox do agente.
- Servidores MCP stdio exigem runtime próprio (Node, Python, etc.) instalado pelo usuário; o Aura detecta ausência (`npx`/`uvx`) e orienta.
- Modo plano usa preset de colaboração do Codex quando disponível na versão fixada; se experimental, fica atrás de feature flag com fallback por instruções (sandbox somente leitura + `developerInstructions` de planejamento).
- Prévia HTML roda em iframe sandbox sem rede e sem acesso ao host.

## Hypotheses and dependencies

- H-017: o preset "plan" (`collaborationMode`) está disponível e estável o suficiente na versão fixada. Check: TK-004.
- Dependências: 002 (aprovações, modos, eventos de plano/diff), 004 TK-004 (servidor MCP do Aura aparece como servidor "Aura" somente leitura na lista).

## Success criteria

### Delivery-verifiable

- **SC-001** — AC-001–AC-016 com evidência; teste com servidor MCP de referência (`@modelcontextprotocol/server-everything`) e Skill de exemplo.

### Post-delivery observation

- **SC-002** — ≥ 20% dos usuários ativos do beta criam ou importam ao menos uma Skill ou Comando rápido.

## Decisions and open questions

- Raiz de Skills do Aura: `%LOCALAPPDATA%\Aura\skills`; `~/.agents/skills` do usuário incluído por padrão (desligável), com origem indicada.
- O servidor MCP do Aura (004) aparece na lista como "Aura (integrado)", não removível.
