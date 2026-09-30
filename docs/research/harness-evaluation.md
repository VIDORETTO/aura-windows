# Avaliação de harness agêntico

Data da pesquisa: 2026-09-29. Fontes primárias consultadas: repositório `openai/codex` (release `rust-v0.159.0`, código de `codex-rs/app-server-protocol` e `codex-rs/model-provider-info`), documentação oficial `developers.openai.com/codex/{app-server,auth,config-advanced,config-reference,skills,windows}`, repositório `openai/openai-agents-python` (v0.22.3) e `openai/openai-agents-js`, issue `openai/codex#36886`.

## Pergunta

Qual base (harness) sustenta um assistente agêntico para Windows que precisa de: login nativo com ChatGPT consumindo a assinatura do usuário, BYOK com endpoint customizado, tools, skills, MCP, aprovações humanas, histórico persistente e baixo consumo de recursos?

## Candidatos

| Critério | Codex app-server (`openai/codex`) | OpenAI Agents SDK (Python/JS) | Codex SDK (TS, embrulha `codex exec`) | Loop próprio |
| --- | --- | --- | --- | --- |
| Login com ChatGPT (assinatura) | **Sim, com contrato oficial para apps de terceiros**: o programa *Sign in with ChatGPT — ChatGPT plan usage* documenta exatamente como configurar o app-server com o access token OAuth do usuário (ver seção abaixo). O login interno do app-server (`account/login/start`) existe, mas autoriza o cliente "Codex" | Só API key nativamente; tecnicamente poderia chamar `api.openai.com/v1/responses` com o token SIWC, mas perderia todo o harness | Herda do CLI (cliente "Codex") | Possível com SIWC, sem harness |
| BYOK / endpoint customizado | Sim: `model_providers.<id>` com `base_url`, `env_key`, `http_headers`, `auth.command`. **Somente `wire_api = "responses"`**; `chat` foi removido (erro explícito no código) | Sim, amplo (Responses, Chat Completions e 100+ via LiteLLM) | Igual ao Codex | Sim |
| Provedor por conversa | Sim: `thread/start.model_provider`, `model`, `config` (overrides), `base_instructions`, `developer_instructions`, `ephemeral` | Sim | Limitado | Sim |
| Tools do cliente | `dynamicTools` + `item/tool/call` (experimental) e servidores MCP | Function tools nativas | Não | Sim |
| MCP | Sim (stdio e HTTP, OAuth, `default_tools_approval_mode`, elicitation) | Sim | Via config | Implementar |
| Skills | Sim, padrão aberto agentskills.io, descoberta por `skills/list`, invocação por item `skill` | Não nativo | Sim | Implementar |
| Aprovações humanas | Sim: comandos, file changes, permissões, MCP, com decisões `accept`/`acceptForSession`/`decline` | Human-in-the-loop básico | Não interativo | Implementar |
| Sandbox Windows nativo | Sim (`elevated`/`unelevated`, MXC) | Sandbox exige Docker no Windows | Sim | Implementar |
| Persistência, fork, compactação | Sim (`thread/list/read/resume/fork`, `thread/compact/start`, memórias) | Sessions | Parcial | Implementar |
| Plano/progresso, diff | `turn/plan/updated`, `turn/diff/updated`, modo plano (collaboration mode) | Não nativo | Não | Implementar |
| Uso/limites da assinatura | `account/rateLimits/read`, `account/usage/read` | N/A | N/A | N/A |
| Entradas multimodais | `text`, `image`, `localImage`, `audio`, `localAudio`, `skill`, `mention` (áudio depende de `inputModalities` do modelo) | Depende do provedor | Texto/imagem | Livre |
| Runtime no Windows | Binário Rust único (`codex-app-server-x86_64-pc-windows-msvc`), ~55–75 MB compactado, ~200 MB descompactado | Python 3.10+ ou Node | Node + binário Codex | Nenhum extra |
| Estabilidade de API | Protocolo versionado com geração de schema (`generate-ts`, `generate-json-schema`); partes marcadas experimentais | Estável | Estável | Própria |
| Licença | Apache-2.0 | MIT | Apache-2.0 | — |

## Sign in with ChatGPT (SIWC) — achado de 2026-09-29

A OpenAI publica em `developers.openai.com/siwc` o programa **Sign in with ChatGPT**, com a capacidade opcional **ChatGPT plan usage**: apps de terceiros obtêm, por OAuth + PKCE com registro dinâmico de cliente (`client_id=dynamic_agent_client`, `agent_name_hint`, `ext_agent_host_id`) e escopo `chatgpt.tokens.use.direct`, um access token (1 h, refresh rotativo de 30 dias) aceito em `POST https://api.openai.com/v1/responses` e cobrado do plano ChatGPT (Plus/Pro) do usuário. A página "Codex app-server" do programa descreve a configuração: provedor `wire_api="responses"`, `base_url="https://api.openai.com/v1"`, `env_key` com o token, `requires_openai_auth=false`, `supports_websockets=false`.

- Disponibilidade: liberado para apps open-source e locais; apps pagos ou hospedados remotamente precisam do formulário de interesse.
- Limites do preview: sem áudio/vídeo de entrada, sem geração de imagem/file search/computer use/`tool_search` hospedados, sem `previous_response_id`; o usuário gerencia limites por app em `chatgpt.com/settings/usage`; Plus compartilha a janela de 5 h entre apps.
- Consequência para o Aura: login via SIWC (ADR 0007), token injetado pelo Gateway local (sem reiniciar o app-server a cada hora), UI conforme as diretrizes de marca e uso.

## Decisão recomendada

Usar **Codex app-server como harness**, executado como processo filho (sidecar) do app, falando JSON-RPC por stdio, com `CODEX_HOME` isolado do Codex pessoal do usuário. Ver [ADR 0001](../adr/0001-codex-app-server-como-harness.md).

Motivos decisivos:

1. É o harness que a própria OpenAI documenta para usar o plano ChatGPT em apps de terceiros (SIWC → "Codex app-server"). O Agents SDK não oferece harness equivalente para esse fluxo.
2. Entrega pronto o que seria mais caro construir: loop agêntico, MCP, skills, aprovações, sandbox Windows, histórico, compactação, memórias, planos e diffs.
3. Permite trocar `base_instructions` por conversa, então o comportamento "assistente de desktop" não fica preso ao prompt de programação.
4. É Rust e roda sem runtime extra, alinhado ao host Tauri/Rust.

## Consequências e mitigação

| Risco | Mitigação planejada |
| --- | --- |
| BYOK só com Responses API | Aura Gateway local traduz Responses ⇄ Chat Completions / Anthropic Messages / Gemini ([ADR 0003](../adr/0003-gateway-responses-local-para-byok.md)). Provedores que já falam Responses (OpenAI, Azure, Ollama, LM Studio, vLLM, OpenRouter) vão direto. |
| Partes experimentais (`dynamicTools`, `collaborationMode`, `additionalContext`, realtime) | Ferramentas do Aura expostas por **MCP** (estável). Experimentais usados só atrás de feature flag e teste de contrato por versão fixada. |
| Churn de versão (releases diárias) | Versão do app-server **fixada** por release do Aura, com SHA-256 verificado, schema TS gerado e commitado, suíte de contrato com transcripts gravados. |
| Tamanho do binário | Download no primeiro uso (instalador pequeno) e início preguiçoso: o processo só sobe quando o usuário abre a primeira conversa e encerra após inatividade configurável. |
| Termos de uso para cliente de terceiros com login ChatGPT | Resolvido pelo SIWC: fluxo oficial liberado para apps open-source/locais; para app pago/fechado, enviar o formulário de interesse antes do lançamento (Q-001). `clientInfo.name = "aura_desktop"` igual ao `agent_name_hint`. |
| Prompt base orientado a código | `base_instructions` próprio do Aura + `developer_instructions` por modo; ferramentas de shell/arquivo desligadas por padrão fora do modo "Tarefa". |

## O que ficou de fora e por quê

- **OpenAI Agents SDK**: excelente para multiagente com API key, mas falha no requisito de login ChatGPT e exigiria embutir Python/Node e Docker para sandbox no Windows.
- **Codex SDK (TS)**: é voltado a automação/CI; não expõe aprovações interativas e eventos ricos para uma UI.
- **Loop próprio**: maior controle, mas reimplementa meses de trabalho e continua sem login ChatGPT suportado.

## Itens a verificar no spike (TK-001 do esforço 002)

- Tamanho real e consumo de memória ociosa do `codex-app-server` no Windows x64.
- Conteúdo de imagem em resultado de tool MCP chega ao modelo como imagem (senão, fallback para `dynamicTools` com `contentItems`).
- `thread/start.base_instructions` substitui integralmente o prompt de código.
- Fluxo SIWC completo (registro dinâmico, troca de código, `chatgpt.tokens.use.direct`) e um turno do app-server via Gateway com o access token.
