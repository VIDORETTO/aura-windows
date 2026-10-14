---
status: accepted
---

# Histórico local mantém os resultados web

## Contexto

O ADR 0010 entregava páginas, consultas e raciocínio da pesquisa web por referências
opacas, apagadas ao fim de cada Turno, para que o texto não ficasse nos rollouts do
Codex. Em conversas reais (12/10/2026), o Turno seguinte recebia "Previous web result
is no longer available" no lugar da própria pesquisa; o agente concluía que não tinha
fonte e se retratava de uma resposta correta. O usuário decidiu (13/10/2026) que
privacidade vale até onde não prejudica a experiência: o Aura roda localmente e é
open source.

## Decisão

Resultados e argumentos das ferramentas `web_search`/`web_fetch` passam em claro pelo
gateway e ficam no histórico local da Conversa normal, como qualquer outra saída de
ferramenta. Conversa Efêmera continua sem rollout. O resolvedor de referências fica
apenas para responder a handles de conversas gravadas antes desta mudança.

## Consequências

O agente mantém o contexto da pesquisa entre Turnos, após reiniciar e após
compactação. Páginas lidas e não citadas ficam no perfil local do usuário. Limites,
cancelamento, desativação da web e bloqueio de destinos privados não mudam.
Detalhes e evidências: `specs/042-continuidade-e-ruido/change.md`.
