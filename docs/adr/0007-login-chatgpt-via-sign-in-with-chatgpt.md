---
status: accepted
---

# Login ChatGPT via "Sign in with ChatGPT" com uso do plano, entregue ao Codex pelo Gateway

O Aura usa o fluxo oficial **Sign in with ChatGPT (SIWC) — ChatGPT plan usage** para clientes open-source/locais: OAuth 2.0 + OIDC com PKCE no navegador do sistema, registro dinâmico (`client_id=dynamic_agent_client` na primeira vez, `agent_name_hint="Aura"`, `ext_agent_host_id` estável por instalação), callback loopback `http://127.0.0.1:<porta>/auth/callback`, escopos `openid profile email offline_access resource.invoke chatgpt.tokens.use.direct` e `resource=https://api.openai.com/v1`. O host guarda as credenciais (client_id emitido, id/access/refresh tokens) no Windows Credential Manager, renova o access token (1 h) antes do vencimento e o injeta no Gateway local, que repassa as requisições `POST /v1/responses` do Codex app-server para `https://api.openai.com/v1` com `Authorization: Bearer <access_token>`. O app-server vê apenas o provedor `aura-chatgpt-plan` (`wire_api="responses"`, `requires_openai_auth=false`, `supports_websockets=false`) apontando para o Gateway.

## Considered options

- Login gerenciado pelo próprio Codex (`account/login/start {type:"chatgpt"}`): autoriza o cliente "Codex", não o Aura; é o fluxo dos clientes do próprio Codex e não o contrato documentado para apps de terceiros. Descartado.
- `ACCESS_TOKEN` no ambiente do app-server (receita da documentação): exige reiniciar o app-server a cada renovação (1 h) e retomar threads; o Gateway evita reinícios e mantém o token fora do processo do Codex.
- Usar `chatgpt.com/backend-api/codex` com o token do Codex CLI: sem contrato (issue `openai/codex#36886`). Descartado.

## Consequences

- Distribuição: o uso do plano está liberado para apps open-source e locais; oferecer num app pago ou hospedado exige o formulário de interesse da OpenAI (decisão de produto Q-001).
- Limitações do fluxo (preview): sem entrada de áudio/vídeo (ASR local continua necessário), sem ferramentas hospedadas de geração de imagem/file search/computer use/`tool_search`; `store:false` e `stream:true` obrigatórios (o app-server já faz).
- `account/rateLimits/read` do Codex não se aplica; a UI mostra "Usando plano ChatGPT" + "Gerenciar uso" (`https://chatgpt.com/settings/usage`) e trata os códigos `subscription_sharing_*`.
- Lista de modelos vem de `GET https://api.openai.com/v1/models` com o access token (visibilidade `list`), não do catálogo embutido do app-server.
- Diretrizes de UI da OpenAI se aplicam: botão "Continue with ChatGPT" com marca aprovada, modal de boas-vindas no primeiro uso, indicador de uso do plano e modal de limite com "Gerenciar uso" como ação principal.
