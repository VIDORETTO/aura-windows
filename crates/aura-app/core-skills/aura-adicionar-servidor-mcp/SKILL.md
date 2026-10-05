---
name: aura-adicionar-servidor-mcp
description: Use quando o usuário pedir para adicionar, conectar ou configurar um servidor MCP no Aura (GitHub, Notion, banco de dados, sistema de arquivos, uma URL /mcp…). Configura com a ferramenta aura mcp_server_save sem nunca tocar em segredos.
---

# Adicionar um servidor MCP ao Aura

Servidores MCP dão ferramentas novas ao agente. Há dois transportes:

- `stdio`: um programa local (`command` + `args`), por exemplo `npx -y @modelcontextprotocol/server-github`.
- `http`: uma URL (`https://…/mcp`), com token opcional (`bearer: true`) ou login OAuth pelo botão "Conectar" nas Configurações.

## Passos

1. Descubra qual servidor o usuário quer. Se ele não souber o pacote ou a URL, use a busca na web para achar a documentação oficial do servidor e confirme o comando, os argumentos e as variáveis de ambiente exigidas. Prefira pacotes oficiais ou muito usados; avise se a origem for desconhecida.
2. Chame `extensions_list` (servidor `aura`) para ver os servidores existentes e evitar nome repetido. O nome `aura` é reservado.
3. Separe variáveis comuns (`env`, ex.: caminhos, região) das **secretas** (tokens, chaves, senhas). Secretas vão só pelo nome em `secret_env` (stdio) ou `bearer: true` (http).
4. Chame `mcp_server_save` com `name`, `transport` e os campos do transporte. Use `approval: "askForWrites"` (padrão) a menos que o usuário peça outra coisa. O usuário aprova a gravação.
5. Se o servidor precisa de segredo, ele é salvo **desligado**. Explique ao usuário: abrir Configurações › Extensões › Servidores MCP, editar o servidor, colar o segredo e ligá-lo. Diga onde obter o token (link da documentação).
6. Avise que o servidor vale nas próximas conversas e que o estado (Conectado/Erro) aparece em Configurações › Extensões.

## Regras

- **Nunca** peça, aceite, repita nem grave segredos na conversa ou em `env`. Se o usuário colar um token, diga para não fazer isso e para usar o campo das Configurações.
- Não instale nada nem rode comandos para "testar" o servidor; o Aura inicia o servidor sozinho.
- Para Windows, `npx` e `uvx` funcionam como comando direto; não use `cmd /c` a menos que a documentação exija.
