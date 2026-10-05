---
name: aura-criar-skill
description: Use quando o usuário pedir para criar, melhorar ou reescrever uma Skill do Aura (instruções reutilizáveis que o agente carrega sozinho). Cria com a ferramenta aura skill_save.
---

# Criar uma Skill do Aura

Uma Skill é uma pasta com `SKILL.md`: um cabeçalho com `name` e `description` e instruções em Markdown. O agente lê só o nome e a descrição de todas as Skills e abre as instruções quando a descrição combina com o pedido. Por isso a descrição decide se a Skill será usada.

## Passos

1. Entenda o objetivo. Se faltar algo que muda o resultado (público, formato de saída, idioma, ferramentas), faça **uma** pergunta curta; senão, siga com padrões sensatos.
2. Chame `extensions_list` (servidor `aura`) para ver as Skills existentes e evitar nome repetido. Para melhorar uma Skill do Aura que já existe, use o mesmo nome com `replace: true`.
3. Escolha o nome: minúsculas, números e hífens, curto e descritivo (`revisar-contrato`, `resumir-ata`). Nada de `aura-` no início (reservado).
4. Escreva a descrição (até 1024 caracteres) dizendo **quando** usar: gatilhos, tipos de arquivo, palavras que o usuário diria. Terceira pessoa, no idioma do usuário.
5. Escreva as instruções: objetivo, passos numerados, critérios de qualidade, formato da resposta e um exemplo curto. Seja específico; evite generalidades que o modelo já faria sem a Skill. Prefira menos de 300 linhas.
6. Chame `skill_save` com `name`, `description` e `instructions`. O usuário aprova a gravação.
7. Responda com um resumo curto: nome, quando ela é usada e que dá para revisar ou desativar em Configurações › Extensões.

## Regras

- Nunca inclua segredos, senhas ou tokens nas instruções.
- Não peça para a Skill executar comandos destrutivos sem confirmação do usuário.
- Se a ferramenta devolver erro de validação, corrija o campo e tente de novo uma vez.
