---
schema: hybrid/spec
schema_version: "1.0"
effort_id: 003-byok-gateway
revision: 1
status: accepted
profile: standard
---

# Specification: BYOK com endpoint customizado

## Problem and desired result

Nem todo usuário tem ChatGPT pago, e muitos já têm chaves de outros provedores, endpoints corporativos ou modelos locais. O resultado é poder cadastrar Provedores BYOK (endpoint, formato de API, Credencial, modelos), testar a conexão e usá-los em qualquer Conversa com as mesmas capacidades agênticas — inclusive quando o provedor só fala Chat Completions ou Anthropic Messages.

## Consumers and actors

- Usuário que configura e escolhe Provedores.
- Codex app-server, que consome provedores via Responses API.
- Provedores externos: OpenAI API, Azure OpenAI, OpenRouter, Anthropic, Google Gemini (endpoint OpenAI-compatível), Groq, DeepSeek, Mistral, xAI, Together, Ollama, LM Studio, vLLM, endpoints genéricos.

## Scope

### Included

- Cadastro, edição, remoção e teste de Provedores BYOK; Credenciais no cofre do Windows.
- Presets para provedores conhecidos e opção "Personalizado".
- Provedores que falam Responses API atendidos pelo Gateway em modo passagem (sem tradução).
- Gateway local que traduz Responses API ⇄ Chat Completions e ⇄ Anthropic Messages, com streaming, tool calls e imagens.
- Descoberta de modelos e capacidades (texto/imagem/tools) e escolha de Provedor/modelo por Conversa.

### Excluded

- Rodar LLM local embutido (usa-se Ollama/LM Studio via BYOK).
- Ferramentas hospedadas exclusivas da OpenAI em provedores traduzidos (por exemplo, busca web hospedada) — a UI indica indisponibilidade.
- Balanceamento/fallback automático entre provedores (futuro).

## User journeys and scenarios

### US-001 — Cadastrar um Provedor com minha chave (Priority: P1)

Independent demonstration: cadastrar Groq com chave, testar, ver modelos.

#### Acceptance scenarios

- **AC-001** — Dado a tela Provedores, quando o usuário escolhe um preset (por exemplo, OpenRouter), informa a Credencial e salva, então o Provedor aparece na lista com estado "verificado" após um teste bem-sucedido, e a Credencial não é exibida novamente (apenas os 4 últimos caracteres).
- **AC-002** — Dado "Personalizado", quando o usuário informa URL base, formato (Responses, Chat Completions, Anthropic), cabeçalhos extras opcionais e Credencial opcional, então o Provedor é salvo e testado.
- **AC-003** — Dado um teste de conexão, quando o endpoint responde 401/403, 404, timeout (10 s) ou certificado inválido, então o estado mostra o motivo específico e o Provedor fica "com erro" sem ser removido.
- **AC-004** — Dado um Provedor salvo, quando o usuário o remove, então a Credencial é apagada do cofre do Windows e o Provedor some das opções; Conversas antigas que o usavam mostram "Provedor removido" ao tentar continuar.
- **AC-005** — Dado qualquer Credencial salva, quando se inspeciona `%LOCALAPPDATA%\Aura` (banco, `config.toml`, logs), então a Credencial não aparece em texto claro em nenhum arquivo.

### US-002 — Conversar usando um Provedor BYOK (Priority: P1)

#### Acceptance scenarios

- **AC-006** — Dado um Provedor que fala Responses API (OpenAI, Azure, OpenRouter, Ollama, LM Studio, vLLM), quando o usuário inicia uma Conversa com ele, então a Conversa funciona com streaming, ferramentas e imagens (quando o modelo suporta), com o Gateway apenas repassando as requisições (sem tradução) e injetando a Credencial.
- **AC-007** — Dado um Provedor Chat Completions (por exemplo, Groq, DeepSeek, Gemini), quando o usuário conversa, então a resposta chega em streaming com o mesmo comportamento visível de AC-005 do esforço 002.
- **AC-008** — Dado um Provedor Chat Completions com modelo que suporta tools, quando o agente precisa de uma ferramenta (por exemplo, uma Ferramenta do Aura ou comando no Modo Tarefa), então a chamada de ferramenta, o resultado e a continuação da resposta funcionam em streaming, inclusive com várias chamadas em paralelo.
- **AC-009** — Dado um Provedor Chat Completions com modelo de visão, quando o turno contém imagem, então a imagem chega ao modelo; dado modelo sem visão, então o envio é bloqueado como no esforço 002 (AC-014).
- **AC-010** — Dado um Provedor Anthropic, quando o usuário conversa com ou sem ferramentas e imagens, então o comportamento equivale a AC-007–AC-009.
- **AC-011** — Dado um erro do provedor traduzido (401, 429 com `retry-after`, 5xx, contexto excedido), quando o turno falha, então o Overlay mostra a mesma categoria de erro do esforço 002 (AC-008), incluindo o tempo de espera do 429 quando informado.

### US-003 — Escolher Provedor e modelo por Conversa (Priority: P1)

#### Acceptance scenarios

- **AC-012** — Dado Provedores configurados, quando o usuário abre o seletor de modelo, então vê grupos por Provedor (ChatGPT primeiro), com ícones de capacidade (imagem, ferramentas, raciocínio) por modelo; escolher um modelo BYOK inicia as próximas Conversas nele.
- **AC-013** — Dado um Provedor com listagem de modelos (`/models`), quando o usuário atualiza, então os modelos são descobertos; dado provedor sem listagem, então o usuário pode digitar o id do modelo e marcar capacidades manualmente.
- **AC-014** — Dado o limite da assinatura ChatGPT atingido, quando o banner de limite aparece, então a ação "Usar outro provedor" abre o seletor com os Provedores BYOK e continua a Conversa atual numa nova Conversa com o histórico resumido como contexto.

## Requirements

- **FR-001** — O sistema MUST gerenciar Provedores BYOK (presets e personalizado) com Credenciais exclusivamente no cofre do Windows.
- **FR-002** — O sistema MUST testar conexões e reportar falhas por categoria.
- **FR-003** — O sistema MUST atender todos os Provedores BYOK por um Gateway local: repassando provedores Responses API e traduzindo provedores Chat Completions e Anthropic.
- **FR-004** — O Gateway MUST preservar streaming de texto, raciocínio (quando exposto), tool calls (inclusive paralelas), imagens de entrada, uso de tokens e erros.
- **FR-005** — O sistema MUST descobrir ou aceitar declaração manual de modelos e capacidades e permitir escolher Provedor/modelo por Conversa.
- **FR-006** — O Gateway MUST aceitar conexões apenas do app-server do Aura (loopback + token por execução).

## Limits, errors, and compatibility

- Timeout de conexão 10 s; timeout ocioso de stream 120 s.
- Tool calls com argumentos JSON inválidos vindos do provedor são repassados como erro de ferramenta ao agente, não derrubam o turno.
- Reasoning criptografado e itens exclusivos da Responses API não são reproduzidos em provedores traduzidos; o conteúdo `reasoning_content`/`thinking` visível é mapeado para resumo de raciocínio quando existir.
- Ao trocar de Provedor numa Conversa existente, uma nova Conversa é criada (o Codex fixa o provedor na thread); o histórico anterior é resumido e injetado.
- Proxies HTTP do sistema são respeitados.

## Hypotheses and dependencies

- Hipótese H-007: a tradução preserva tool calls em streaming. Check: fixtures reais por provedor (TK-004, TK-006).
- Hipótese H-011: Ollama/LM Studio/OpenRouter expõem `/v1/responses` compatível o bastante para o Codex. Check: TK-002 com cada um.
- Dependência: esforço 002 (seletor de modelo, `ConfigContributor`, `thread/start.modelProvider`). Estado: planejado.

## Success criteria

### Delivery-verifiable

- **SC-001** — AC-001–AC-014 com evidência; suíte de contrato do Gateway com fixtures de OpenAI Chat, Groq, Gemini, DeepSeek e Anthropic.
- **SC-002** — Uma tarefa agêntica de referência (ler um arquivo do workspace e resumir) concluída com Groq, Gemini e Anthropic.

### Post-delivery observation

- **SC-003** — Percentual de Conversas BYOK que falham por erro de tradução < 2% no beta.

## Decisions and open questions

- Presets iniciais: OpenAI API, Azure OpenAI, OpenRouter, Anthropic, Google Gemini, Groq, DeepSeek, Mistral, xAI, Together, Ollama, LM Studio, vLLM, Personalizado.
- Chave de API da OpenAI é tratada como Provedor BYOK (preset "OpenAI API"), não como modo de login.
