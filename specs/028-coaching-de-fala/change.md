---
schema: hybrid/change
schema_version: 1.0
effort_id: 028-coaching-de-fala
revision: 1
status: draft
profile: compact
---

# Change: Coaching de fala privado

## Objetivo e limites

Candidato CAND-045. Números sobre a fala do próprio usuário numa reunião salva (tempo de fala, ritmo, muletas, maior turno, perguntas), calculados localmente e só mostrados a ele. Fora do escopo: avaliar outras pessoas, tom de voz, expressão facial.

## Contrato de comportamento

- Entradas: [tipo e pré-condições]
- Saída: [resultado observável]
- Erros/invariantes: [casos relevantes]
- Compatibilidade: [o que deve permanecer igual]

## Requisitos e aceite

- **FR-001** — O Aura MUST calcular, só da fala do usuário, tempo de fala, palavras por minuto, muletas e perguntas.
- **AC-001** — Dada uma reunião com 30 s de fala de "Eles" e 20 s do usuário (39 palavras, 5 muletas, 1 pergunta), então `talkPercent=40`, `wordsPerMinute=117`, `fillerCount=5`, `questionsAsked=1`, `longestTurnSeconds=20`; reunião vazia ou só de "Eles" não divide por zero. **Feito e testado (05/10).**
- **FR-002** — O usuário MUST ver os números na reunião salva e o agente MUST oferecê-los só como feedback ao usuário.
- **AC-002** — Dada uma reunião salva, o painel mostra "Sua fala (só você vê)" com % de fala, palavras/min, muletas e perguntas; a ferramenta `meeting_stats` devolve os mesmos números. **Feito e testado (05/10).**

## Leitura e mapa de alterações

- `crates/aura-app/src/speech_stats.rs` → `compute` — estatísticas da fala; new.

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
