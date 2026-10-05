---
schema: hybrid/change
schema_version: 1.0
effort_id: 019-dia-a-dia-texto
revision: 1
status: draft
profile: compact
---

# Change: Dia a Dia: ações sobre texto e tela

## Objetivo e limites

Candidatos: CAND-058, CAND-060, CAND-061, CAND-064, CAND-065, CAND-067. Origem: plano `docs/project/plano-reuniao-e-configuracao-assistida.md`. Funções simples sobre o texto selecionado e a tela, sem configuração: reescrever, checar golpe, responder, retomar o que fazia, colar em outro formato e ler em voz alta.

Nota: **Feito (05/10):** comandos embutidos (`/formal`, `/curto`, `/amigavel`, `/golpe`, `/responder`, `/parei`, `/colar`, `/ler`), "Substituir seleção" com antes → depois e Desfazer, leitura da seleção. **Falta:** mini-menu flutuante com atalho próprio (janela do shell, só valida no Windows), expirar o alvo da substituição ao trocar de conversa, validação real no Windows.

## Contrato de comportamento

- Entradas: ver requisitos e critérios abaixo.
- Saída: ver critérios de aceite.
- Erros/invariantes: segredos nunca expostos; captura só pela Política de privacidade.
- Compatibilidade: comportamento existente do Overlay, Comandos rápidos e Configurações.

## Requisitos e aceite

- **FR-001** — Comandos rápidos embutidos MUST cobrir reescrita (/formal, /curto, /amigavel), checagem de golpe (/golpe), rascunho de resposta (/responder) e retomada (/parei).
- **AC-001** — Dada a seleção "Seu Pix foi bloqueado, clique aqui", quando o usuário envia `/golpe`, então o prompt enviado termina com a seleção e pede veredito, sinais, o que fazer e como confirmar; `/curto` sem texto devolve erro de entrada; `/responder informal` anexa a tela e usa "Tom: informal" (padrão "cordial"); `/parei` manda o agente usar `screen_recent`. **Feito e testado (05/10).**
- **FR-002** — O menu na seleção MUST oferecer as ações em um gesto e devolver o resultado ao app (substituir ou copiar) com antes → depois e Desfazer.
- **AC-002** — Dado texto selecionado em outro app, quando o usuário aperta o atalho do menu e escolhe "Mais curto", então o resultado aparece com antes → depois e "Substituir" cola no app anterior; "Desfazer" restaura o texto original. **Pendente.**
- **FR-003** — O Aura MUST ler a seleção em voz alta e colar em outro formato.
- **AC-003** — Dado texto selecionado, quando o usuário escolhe "Ler", então a voz offline lê sem blocos de código; "Colar como lista" converte a área de transferência e cola. **Pendente.**

## Leitura e mapa de alterações

- `[path]` → `[symbol]` — [motivo]; [existing/new].

## Plano breve

Seam: [interface sob teste]. Abordagem: [decisão técnica local]. Dependências: [none ou refs].

## Sequência e tarefas

- [ ] C-001 Escrever caso relevante e observar red pelo motivo esperado.
- [ ] C-002 Implementar o mínimo e observar green.
- [ ] C-003 Executar regressão e registrar evidência.

## Validação e evidência

Comando/procedimento: `[exact command]`

Resultado executado: [pending]

Limitações: [o que não foi verificado]. Comando apenas identificado na configuração: [none ou registro].

## Condição de retorno

Retornar se o contrato, símbolo, dependência ou decisão material não puder ser satisfeito sem ampliar o escopo.

## Estado

`change.md` é a fonte canônica deste esforço compacto. Não crie `tasks.md` ou tickets paralelos para esta mudança.
