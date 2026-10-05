---
schema: hybrid/change
schema_version: 1.0
effort_id: 033-palavras-vigiadas
revision: 1
status: draft
profile: compact
---

# Change: Palavras vigiadas na reunião

## Objetivo e limites

Candidato CAND-055. O usuário lista palavras (cliente, prazo, preço) em Configurações › Privacidade; quando uma é dita durante uma Reunião ativa, o Overlay avisa com o trecho e o tempo. Só age em Reunião iniciada pelo usuário; também configurável pela IA (`meetingWatchWords`).

## Contrato de comportamento

- Entradas: [tipo e pré-condições]
- Saída: [resultado observável]
- Erros/invariantes: [casos relevantes]
- Compatibilidade: [o que deve permanecer igual]

## Requisitos e aceite

- **FR-001** — A detecção MUST casar palavra inteira sem diferenciar maiúsculas nem acentos, e frases por trecho.
- **AC-001** — Dado [preço, prazo final, ok], "Qual o PRECO?" acerta preço; "o Prazo final é sexta, ok" acerta prazo final e ok; "precoce e okay" não acerta. **Feito e testado (05/10).**
- **FR-002** — A lista MUST ser aparada, sem vazios, com no máximo 50 itens de 64 caracteres, e editável nas configurações.
- **AC-002** — Dado " preço ", "", "prazo", a lista salva é [preço, prazo]; 65 caracteres é recusado; o campo da UI grava uma palavra por linha. **Feito e testado (05/10).**

## Leitura e mapa de alterações

- `crates/aura-app/src/meeting.rs` → `watch_hits` + aviso em `Host::meeting_poll` e campo em Privacidade; new.

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
