---
name: aura-criar-receita
description: Use quando o usuário pedir uma Receita de reunião nova ou ajustada ("receita para minhas reuniões com fornecedores", "mude as notas da 1:1"). Cria com as ferramentas recipe_list e recipe_save.
---

# Criar uma Receita de reunião

Uma Receita diz como organizar as notas de um tipo de reunião e quanto ajuda o usuário quer ao vivo.

## Passos

1. Chame `recipe_list` para ver as Receitas e evitar nome repetido. Para ajustar uma Receita do usuário, use o mesmo `id` com `replace: true`. As embutidas não mudam: crie uma nova baseada nela.
2. Entenda o tipo de reunião. Se faltar algo que muda o resultado (o que o usuário sempre precisa registrar, tom, nível de ajuda), faça **uma** pergunta curta com a opção recomendada; senão siga com padrões sensatos.
3. Escolha `id` (minúsculas, números e hífens, curto) e `name` legível.
4. Escreva `notes_template`: as seções a preencher, separadas por " · ", na ordem em que o usuário quer ler (3 a 7 seções; sempre inclua compromissos com dono e prazo quando fizer sentido).
5. Escolha `help_level`: `silent` (só transcreve e anota), `onDemand` (padrão), `balanced`, `active`.
6. Chame `recipe_save`. O usuário aprova.
7. Responda com um resumo curto: nome, seções e que ela aparece ao preparar uma reunião.

## Regras

- Nada de segredos nem dados de terceiros nas Receitas.
- Não invente seções que o usuário não precisa.
