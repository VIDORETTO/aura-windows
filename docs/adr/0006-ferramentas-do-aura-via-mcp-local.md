---
status: accepted
---

# Ferramentas do Aura expostas ao Codex por MCP local

Ferramentas de tela, áudio e contexto do desktop são publicadas por um servidor MCP streamable HTTP servido pelo próprio host em `127.0.0.1` (porta aleatória, token por execução) e registrado como `mcp_servers.aura` no `CODEX_HOME` do Aura. Toda chamada passa pelo motor de política (`aura-policy`) e entra no log de auditoria. MCP é estável entre versões do Codex, suporta aprovação por tool e é o mesmo mecanismo usado para MCPs de terceiros.

## Considered options

- `dynamicTools` do app-server: executa no cliente e retorna `contentItems`, mas é experimental; fica como fallback caso imagens em resultados MCP não cheguem ao modelo (hipótese H-003).

## Consequences

- O servidor MCP precisa estar de pé antes de `thread/start`; o supervisor do app-server o inicia primeiro.
