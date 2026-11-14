---
schema: hybrid/spec
schema_version: 1.0
effort_id: 047-acervo
revision: 1
status: accepted
profile: standard
---

# Acervo: índice local cifrado e busca do agente

## Problem and desired result

O Aura guarda Notas, Salvos e Reuniões, mas o agente só os encontra por varredura de palavra exata (`note_search`, `meeting_search`). A busca não tem relevância, não acha paráfrase, tem custo crescente (auditoria A-05) e não junta as fontes. Resultado desejado: o **Acervo**, um índice local e cifrado das Lembranças (plano §5), com uma ferramenta `memory_search` que o agente usa para lembrar por significado e por palavra, com filtros de período e fonte e origem citada em cada trecho. É a primeira fatia da Fase 1 do Segundo Cérebro (CAND-073): nada novo é capturado, só o que o Aura já guarda é indexado.

## Consumers and actors

- Usuário que conversa com o agente e pergunta sobre algo que já disse, anotou ou discutiu em reunião.
- Agente (Codex) que chama `memory_search`.
- Consumidores técnicos:
  - `aura-app` (orquestração e ferramenta);
  - `aura-store` (banco `memory.db`);
  - `aura-worker` (embeddings, ADR 0014);
  - `aura-bench` (conjunto dourado).

## Scope

### Included

- Banco `memory.db` separado do `aura.db` e cifrado por inteiro (ADR 0012), com FTS5 e vetores.
- Lembranças das fontes que já existem:
  - Notas e Salvos (um item cada);
  - falas de Reunião, em trechos de 30–60 s com orador, título da Reunião e Projeto.
- Indexação incremental e idempotente, com embeddings calculados no worker (bekko-a8m, ADR 0014) em lote, com a máquina ociosa.
- Ferramenta `memory_search` para o agente: busca híbrida por RRF, filtros por período e fonte, e cada trecho com id, fonte, origem e data.
- Permissão do agente para a fonte Acervo (Nunca, Perguntar, Sempre) e registro de cada acesso no Registro de acesso.
- Apagar uma Nota, um Salvo ou uma Reunião apaga as Lembranças dela.
- Conjunto dourado do S1 no `aura-bench`, com acerto@1, acerto@10, MRR e latência.

### Excluded

- Conversas com o agente (esforço 048), página Memória na UI (049), Cofre de notas (050).
- Páginas lidas e anexos, que entram numa fatia seguinte do Acervo.
- Momentos de tela (Fase 2), Consolidação (Fase 3) e `memory_count`/`memory_list` (I-11): ficam para fatias seguintes.
- Mudança nas ferramentas existentes `note_search` e `meeting_search`, que continuam iguais.

## User journeys and scenarios

### US-001 — O agente lembra do que foi dito em reunião e anotado (P1)

Como usuário, quero perguntar "o que combinamos sobre o orçamento em outubro?" e receber a resposta com a reunião e a nota de onde veio, sem eu dizer onde procurar.

Demonstração independente: com Reuniões e Notas de exemplo, o agente chama `memory_search` e a resposta cita a origem.

#### Acceptance scenarios

- **AC-001** — Dadas uma Reunião de outubro com a fala "fechamos o orçamento de nuvem em R$ 15 mil por mês" e uma Nota "orçamento de marketing: R$ 8 mil", `memory_search("quanto ficou o orçamento de nuvem?")` devolve o trecho da Reunião em 1º lugar, com a fonte `reuniao`, o id da Reunião, o orador, o instante da fala e a data.
- **AC-002** — Uma pergunta por paráfrase, sem as palavras do trecho ("quanto vamos gastar por mês com servidores?"), ainda devolve o trecho da Reunião entre os 3 primeiros.
- **AC-003** — Uma pergunta com termo exato (código "PAG-1187" presente só numa Nota) devolve essa Nota em 1º lugar.
- **AC-004** — O filtro `periodo` (de, até) e `fonte` (`nota`, `salvo`, `reuniao`) restringe o resultado antes do ranqueamento: com período de novembro, a Reunião de outubro não aparece.

### US-002 — O Acervo se mantém sozinho e é privado (P1)

Como usuário, quero que o índice acompanhe o que eu crio e apago, sem eu fazer nada, e que fique cifrado.

Demonstração independente: criar, editar e apagar Notas e Reuniões e observar o índice.

#### Acceptance scenarios

- **AC-005** — Uma Nota criada aparece no `memory_search` lexical na hora e no vetorial depois do lote do worker; editar a Nota troca a Lembrança; indexar duas vezes não duplica.
- **AC-006** — Apagar uma Nota ou uma Reunião apaga suas Lembranças e seus vetores; a busca não as devolve mais.
- **AC-007** — O arquivo `memory.db` não é legível sem a chave (não começa com `SQLite format 3`), e a chave fica no cofre existente (DPAPI).
- **AC-008** — Com o worker indisponível (sem modelo baixado), `memory_search` funciona só com o lexical e informa que a busca semântica está incompleta.

### US-003 — O usuário controla o acesso do agente (P1)

#### Acceptance scenarios

- **AC-009** — Com a permissão do Acervo em "Nunca", o agente recebe recusa e nada é lido; em "Perguntar", o pedido passa pelo consentimento existente; em "Sempre", passa direto. Cada acesso fica no Registro de acesso com a consulta e o número de trechos devolvidos.

### US-004 — Qualidade medida (P2)

#### Acceptance scenarios

- **AC-010** — O `aura-bench` roda o conjunto dourado do S1 contra o Acervo real (mesmo indexador e mesma busca) e relata acerto@1, acerto@10, MRR e p50/p95 de `memory_search`. O MRR fica a no máximo 0,02 do medido no S1 (0,892) com o mesmo modelo.
- **AC-011** — `memory_search` tem p95 ≤ 300 ms com 500 mil trechos num desktop (`aura-bench`, base sintética), conforme o plano §8.1.

## Requirements

- **FR-001** — Banco `memory.db` cifrado, separado do `aura.db`, com retenção própria.
- **FR-002** — Lembranças de Notas, Salvos e falas de Reunião, com origem, data e cabeçalho por regra.
- **FR-003** — Indexação incremental, idempotente e com remoção pela linhagem.
- **FR-004** — `memory_search` híbrido com filtros e origem.
- **FR-005** — Permissão e Registro de acesso para o Acervo.
- **FR-006** — Conjunto dourado e orçamento de latência no `aura-bench`.

## Limits, errors, and compatibility

- Sem mudança de formato nas ferramentas e tabelas existentes. A ferramenta nova entra no contrato MCP do Aura (ADR 0006) e qualquer mudança de IPC segue o contrato dourado (ADR 0009).
- O build passa a exigir Perl nativo do Windows (ADR 0012).
- Até 20 trechos por chamada, com texto de cada trecho limitado a ~1.000 caracteres.
- Falha do worker não bloqueia a busca lexical.

## Hypotheses and dependencies

- ADR 0012 (armazenamento): aceito ou revisado pelo S2 do esforço 046 antes do plano deste esforço.
- ADR 0014 (embeddings): aceito.
- Catálogo e download verificado de modelos (`aura-asr`) reaproveitados para o modelo de embeddings.

## Success criteria

### Delivery-verifiable

- **SC-001** — AC-001 a AC-011 com evidência; suíte do workspace, clippy e fmt passam.

### Post-delivery observation

- **SC-002** — Em uso real, o agente acerta perguntas sobre reuniões e notas antigas sem o usuário indicar onde procurar.

## Decisions and open questions

- Fontes da primeira fatia: Notas, Salvos e Reuniões. Conversas ficam no 048.
- O plano técnico espera o S2 (ADR 0012).
