---
schema: hybrid/change
schema_version: 1.0
effort_id: 029-habilidades-do-dia-a-dia
revision: 1
status: draft
profile: compact
---

# Change: Habilidades do dia a dia: Ensaio, Estudo, Carreira, Documentos e Ajuda agora

## Objetivo e limites

Candidatos CAND-042, 046, 050, 052 e 066. Skills internas e comandos que cobrem: ensaiar uma conversa difícil com feedback, estudar (notas, glossário, flashcards, quiz), carreira (currículo, carta, histórias STAR), entender documentos (contrato, boleto, bula) e "me ajude agora" sobre tela e áudio recente. Tudo com os fatos do usuário, sem inventar.

## Contrato de comportamento

- Entradas: [tipo e pré-condições]
- Saída: [resultado observável]
- Erros/invariantes: [casos relevantes]
- Compatibilidade: [o que deve permanecer igual]

## Requisitos e aceite

- **FR-001** — O Aura MUST ter Skills internas para ensaio, estudo, carreira e documentos, invisíveis ao usuário e sempre disponíveis ao agente.
- **AC-001** — Dado o início do Aura, então `aura-ensaio`, `aura-estudo`, `aura-carreira` e `aura-documentos` existem em `core-skills`, com nome e descrição válidos, e não aparecem no catálogo do usuário. **Feito e testado (05/10).**
- **FR-002** — Comandos rápidos MUST chamar cada Skill e `/ajuda` MUST anexar a tela e sugerir o áudio recente.
- **AC-002** — Dado `/documento` com texto selecionado, então o prompt começa com `$aura-documentos` e termina no texto; sem texto, erro de entrada; `/ensaio pedir aumento` começa com `$aura-ensaio`; `/ajuda` anexa a tela e cita `audio_recent`. **Feito e testado (05/10).**
- **FR-003** — As Skills MUST seguir o Preparo e nunca inventar fatos.
- **AC-003** — Dado o texto de cada Skill, então pede preparo em três velocidades (ensaio, carreira), cita a fonte (estudo, documentos) e proíbe inventar fatos. **Verificado por leitura; comportamento real do agente não verificado.**

## Leitura e mapa de alterações

- `crates/aura-app/core-skills` → `aura-ensaio`, `aura-estudo`, `aura-carreira`, `aura-documentos` e comandos `/ensaio`, `/estudo`, `/carreira`, `/documento`, `/ajuda`; new.

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
