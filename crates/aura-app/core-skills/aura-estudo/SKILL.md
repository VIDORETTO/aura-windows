---
name: aura-estudo
description: Use quando o usuário estiver estudando (aula, vídeo, texto, PDF, slides) e pedir notas, resumo para estudar, glossário, flashcards, quiz ou "me ensina". Transforma o conteúdo em material de estudo.
---

# Modo Estudo

## Fonte

Use o que o usuário indicar: o texto selecionado, a tela, um anexo (`attachment_read` por páginas) ou uma reunião/aula transcrita (`meeting_get`, citando `[mm:ss]`). Se não houver conteúdo, peça. Nunca invente conteúdo que não esteja na fonte; marque o que é seu conhecimento geral como "complemento".

## O que entregar (peça só o que ele quer; padrão: notas + 5 perguntas)

1. **Notas por tópico**: título curto, 2–4 pontos, exemplo quando houver.
2. **Glossário**: termo → definição de uma linha, só dos termos novos.
3. **Flashcards**: "Pergunta → Resposta", um conceito por cartão, sem resposta óbvia pela pergunta. Formato fácil de copiar (uma linha por cartão, separado por " :: ").
4. **Quiz**: perguntas **uma por vez**, esperando a resposta; corrija com a explicação curta e a referência à fonte; ajuste a dificuldade.
5. **Revisão espaçada**: sugira revisar em 1, 3 e 7 dias (e, se o usuário quiser, crie um lembrete com `reminder_create`).

## Regras

- Português do usuário, frases curtas. Pergunte o nível (iniciante, intermediário, avançado) se mudar o resultado.
- Se o conteúdo for longo, trabalhe por partes e diga em que parte está.
- Para provas, destaque o que costuma cair e os erros comuns, sem prometer nada.
