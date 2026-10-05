---
name: aura-criar-comando-rapido
description: Use quando o usuário pedir para criar ou alterar um comando rápido do Aura (atalho digitado como /nome na barra de entrada, ex. /formal, /traduzir). Cria com a ferramenta aura quick_command_save.
---

# Criar um comando rápido do Aura

Um comando rápido é um modelo de prompt expandido localmente quando o usuário digita `/nome` na barra do Aura. Ele não chama o agente para ser criado; só vira o texto do pedido.

## Marcadores do modelo

- `{selecao}` — texto selecionado no app de onde o usuário veio; se não houver, o texto digitado depois do comando.
- `{texto}` — só o texto digitado depois do comando.
- `{args}` ou `{args:padrão}` — a primeira palavra depois do comando (ex.: `/traduzir espanhol` → `espanhol`); `padrão` vale quando nada é digitado.
- `{tela}` — anexa uma captura da tela antes de enviar.

Exemplo: nome `formal`, modelo `Reescreva em tom formal, mantendo o sentido e o idioma:\n\n{selecao}`.

## Passos

1. Entenda o que o comando deve fazer e de onde vem o texto (seleção, texto digitado, tela). Pergunte só se for ambíguo de verdade.
2. Chame `extensions_list` (servidor `aura`) para ver os comandos existentes. Não use nomes de comandos embutidos (`tldr`, `traduzir`, `reescrever`, `explicar`, `corrigir`, `resumir-tela`) nem `plano`, `plan`, `compactar`, `compact`, `tela`, `screen`. Para alterar um comando seu, use `replace: true`.
3. Nome: minúsculas, números e hífens, curto e fácil de digitar.
4. Modelo: instrução clara e objetiva no idioma do usuário, terminando com o marcador de entrada (normalmente `{selecao}`).
5. Chame `quick_command_save` com `name` e `template`. O usuário aprova a gravação.
6. Responda mostrando como usar (`/nome`, com um exemplo) e que dá para desativar em Configurações › Extensões.
