---
status: accepted
---

# Gateway Responses local para BYOK

Como o Codex só aceita provedores com `wire_api = "responses"`, o host Aura expõe em loopback um endpoint `/v1/responses/<provedor>` que traduz a Responses API para Chat Completions (OpenAI-compatível, incluindo Gemini, Groq, DeepSeek, Mistral) e Anthropic Messages, injetando a chave lida do Windows Credential Manager. Provedores que já falam Responses (OpenAI, Azure OpenAI, OpenRouter, Ollama, LM Studio, vLLM) também passam pelo gateway, em **modo passagem** (sem tradução, só reescrita de URL e injeção de credencial). Assim nenhuma chave BYOK entra no ambiente, na configuração ou na linha de comando do app-server; o Codex só conhece um token de loopback válido por execução.

## Considered options

- Configurar provedores Responses diretamente no Codex com `auth.command`: exigiria um executável auxiliar para ler o cofre e exporia a chave ao processo do Codex; descartado em favor do modo passagem.

- LiteLLM como proxy: Python embutido, pesado e mais uma superfície de atualização.
- Restringir BYOK a provedores Responses: descumpre "endpoint customizado" para a maioria dos provedores compatíveis com OpenAI.

## Consequences

- O gateway é um Module profundo com contrato testado por fixtures SSE gravadas de cada provedor.
- Recursos exclusivos da Responses API (tools hospedadas como `web_search` da OpenAI, reasoning criptografado) não existem em provedores traduzidos; a UI mostra capacidades por provedor/modelo.
