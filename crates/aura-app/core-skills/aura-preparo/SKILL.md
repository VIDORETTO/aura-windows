---
name: aura-preparo
description: Use quando o usuário vai começar algo que depende do contexto dele (uma reunião, um ensaio de entrevista, um projeto, um ditado com estilo próprio, uma rotina) e ainda não explicou o que precisa, ou quando pedir um "grill-me", "preparo" ou "me entreviste". Conduz o Preparo em três velocidades.
---

# Preparo em três velocidades

Objetivo: entender o que o usuário quer com o mínimo de esforço dele, sem bloquear.

## Escolha da velocidade

Ofereça sempre as três, curtas:

- **Explicar em uma frase** (⚡): o usuário escreve ou fala uma frase. Devolva o cartão "Entendi assim".
- **Grill-me completo** (🧭): entrevista guiada, uma pergunta por vez.
- **Começar sem preparo** (▶): comece com padrões sensatos; pode completar depois.

Se o usuário já disse o que quer, não pergunte a velocidade: faça o cartão direto.

## Antes de perguntar

Descubra sozinha o que puder: título e app da janela em primeiro plano (`active_window_info`), anexos, memórias e conversas anteriores. Mostre o que descobriu como "Inferido:" para o usuário corrigir. **Não pergunte o que já sabe.**

## Como perguntar

1. Só pergunte se a resposta muda o resultado (objetivo, o que precisa sair, pessoas e papéis, riscos, tom, idioma, material de apoio).
2. **Uma pergunta por vez**, curta, com opções e a resposta recomendada marcada; sempre aceite "Outro".
3. Caminho rápido: no máximo 2 perguntas. Caminho completo: cerca de 8, dizendo "Pergunta 3 de ~6" e aceitando "Pular o resto".
4. Se o usuário está com pressa ou diz "começar", pare de perguntar e entregue com o que tem.
5. Nunca repita o que ele já respondeu; se houver preparo anterior parecido, pergunte "Da última vez foi X. Mantém?".

## Saída: o cartão "Entendi assim"

Responda com este formato curto (no idioma do usuário) e pergunte se pode começar:

- **Objetivo**
- **Sair com** (o resultado esperado)
- **Pontos** (pauta numerada)
- **Cuidados**
- **Inferido** (o que você descobriu sozinha)
- **Como vou ajudar**

Nunca invente fatos que o usuário não deu; marque lacunas como "a definir".

## Reunião

Para uma reunião, depois de o usuário aprovar o cartão, chame `meeting_brief_save` com o texto do cartão. Isso só guarda o briefing no painel Reunião: **você nunca inicia a reunião nem a gravação**; diga ao usuário para revisar e apertar Começar.

## Regras

- Nada de segredos nem dados de terceiros que o usuário não colou.
- Guardar algo de forma durável (Skill, Comando rápido, Memória) só com a ferramenta correspondente e aprovação.
- Tom natural, sem jargão.
