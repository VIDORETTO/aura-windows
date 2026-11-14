---
schema: hybrid/spec
schema_version: 1.0
effort_id: 046-spikes-segundo-cerebro
revision: 1
status: accepted
profile: standard
---

# Spikes do Segundo Cérebro

## Problem and desired result

Os ADRs 0012 a 0014 (propostos) e a revisão 2 do plano do Segundo Cérebro dependem de números medidos fora do Aura (`docs/research/segundo-cerebro-medicoes-2026-11.md`). Antes de abrir a Fase 1, cada decisão precisa de evidência de um protótipo com as mesmas bibliotecas e o mesmo runtime do Aura (Rust, `ort`, `rusqlite` 0.40, SQLCipher, crate `windows` 0.62). Resultado desejado: um relatório por spike, com pergunta, experimento, limites, evidência e decisão, que permita aceitar, revisar ou recusar cada ADR. Este é um esforço de **pesquisa e protótipo**: o código dos spikes fica em `tools/spikes/`, fora do workspace e fora do produto.

## Consumers and actors

O dono do produto, que delegou as decisões ao agente em 13/11/2026 (S1 com perguntas escritas pelo agente; S2 executado pelo agente; S3 e S4 rodados pelo usuário; S6 depois). Consumidores: os ADRs 0012–0014, o plano e os esforços da Fase 1.

## Scope

### Included

- **S2 (ADR 0012):** busca vetorial no desenho proposto (BLOB binário + int8, SQLCipher, filtro por faixa de `rowid`), em Rust, com vetores do modelo do S1 em português. Inclui partida a frio e uma aproximação de máquina modesta.
- **S1 (ADR 0014):** os finalistas em Rust (`ort` + tokenizador) num conjunto dourado de 240 documentos e 100 perguntas, mais a fusão com FTS5 em variantes.
- **S3 e S4 (ADR 0013):** a ferramenta de campo que o usuário vai rodar num dia real, mais duas verificações curtas sobre o modo de acessibilidade do Chrome.
- **S6 (consentimento, LGPD):** registrado como bloqueado até o parecer jurídico.

### Excluded

- Código de produto (`crates/`, `apps/`), mudanças de esquema, IPC ou UI.
- A escolha final de interface do Segundo Cérebro.
- Medição de CI com Perl (exige mudar o workflow; vira tarefa da Fase 1).

## User journeys and scenarios

### US-001 — Decidir o armazenamento vetorial (P1)

Como dono do produto, quero saber se o desenho do ADR 0012 cumpre o orçamento com cifra, para aceitar ou revisar o ADR.

Demonstração independente: o relatório do S2 com p50/p95, recall e disco para 100 mil, 500 mil e ~1 milhão de vetores, nos casos com e sem filtro, a frio e a quente.

#### Acceptance scenarios

- **AC-001** — O protótipo implementa o desenho do ADR 0012 com o mesmo `rusqlite`/SQLCipher do Aura e mede p50/p95 de busca (k = 10) para 100 mil, 500 mil e ~1 milhão de vetores de 384 dimensões, sem filtro e com filtro de 30 dias, com cache dimensionado e com o cache padrão.
- **AC-002** — O recall@10 é medido contra a busca exata em float32 **com vetores do modelo escolhido no S1, gerados de texto em português**, para 100, 200 e 400 candidatos no primeiro passo.
- **AC-003** — A primeira consulta depois de abrir o banco (frio) e as seguintes (quente) são medidas separadamente, junto com o tempo de abrir o banco e carregar o índice.
- **AC-004** — Uma aproximação de máquina modesta (processo limitado a 1 núcleo e com o estado máximo do processador reduzido no plano de energia, restaurado depois) é medida e rotulada como aproximação.

### US-002 — Escolher o modelo de embeddings (P1)

Como dono do produto, quero o modelo padrão escolhido por medição no runtime do Aura, para aceitar ou revisar o ADR 0014.

Demonstração independente: o relatório do S1 com qualidade, latência e RAM dos finalistas, no conjunto dourado.

#### Acceptance scenarios

- **AC-005** — Um conjunto dourado sintético, sem dados do usuário, tem 240 documentos (reunião, conversa, nota, página, anexo, tela) e 100 perguntas. Documentos e perguntas são escritos por autores diferentes (subagentes), e as perguntas não veem resultados. O viés fica declarado.
- **AC-006** — Os finalistas (granite-97m-r2, bekko-a25m int8, bekko-a8m padrão) e o e5-small de base são medidos em Rust (`ort` + tokenizador, 1 thread): acerto@1/@10, MRR, posição média, latência por documento e por pergunta, RAM do processo e arquivo.
- **AC-007** — As fusões com FTS5 são medidas contra o vetor puro: RRF de pesos iguais, RRF com peso 0,5 no lexical, e "lexical só se a pergunta tem termo raro".
- **AC-008** — O relatório recomenda um modelo e uma fusão padrão, com o critério explícito (qualidade primeiro, depois custo).

### US-003 — Medir a captura no uso real (P2)

Como usuário, quero rodar uma ferramenta num dia normal e devolver só números, para validar o ADR 0013.

Demonstração independente: a ferramenta roda, grava só contagens, tempos e tamanhos, e encerra sozinha.

#### Acceptance scenarios

- **AC-009** — Uma ferramenta de campo implementa o desenho do ADR 0013:
  - ganchos separados e assentamento de 1,5 s;
  - UIA com orçamento de 200 ms fora dos navegadores e apps Chromium;
  - OCR do Windows na resolução nativa da janela;
  - WebP sem perdas só para medir o tamanho (a imagem não é guardada);
  - WGC por janela para "mudou algo".

  Ela grava só contagens, tempos, tamanhos e o nome do executável (nunca texto nem imagem) e tem limite de duração e um jeito simples de parar.
- **AC-010** — A ferramenta é testada pelo agente por alguns minutos e o roteiro de uso fica documentado.
- **AC-011** — Duas verificações curtas no Chrome: (a) ler só o nome UIA da raiz liga ou não "Web accessibility"; (b) quanto tempo o Chrome leva para desligar o modo depois que o cliente UIA encerra.
- **AC-012** — O resultado do uso real fica pendente do usuário; o ticket correspondente fica bloqueado até ele rodar.

### US-004 — Consentimento (P3)

- **AC-013** — S6 fica registrado como bloqueado até o parecer jurídico, com as perguntas a fazer.

## Requirements

- **FR-001** — Os protótipos usam as mesmas versões de `rusqlite`, `ort`, `windows` e do toolchain do workspace.
- **FR-002** — Nenhum dado do usuário entra em arquivo, log ou commit; só o conjunto dourado sintético e os números.
- **FR-003** — Cada spike termina num relatório com pergunta, experimento, limites, evidência e decisão, e na atualização do status do ADR correspondente.
- **FR-004** — Os protótipos não entram no workspace do produto nem mudam o build dele.

## Limits, errors, and compatibility

- Máquina de medição: Ryzen 7 7800X3D (desktop). A máquina modesta é aproximada (AC-004), não real.
- A CPU do usuário fica livre: medições pesadas rodam com prioridade baixa e em janelas curtas.
- Corpus e perguntas sintéticos, escritos por modelos: o viés fica declarado no relatório.

## Hypotheses and dependencies

- `ort` 2.0.0-rc.12 (já no lockfile pelo `transcribe-rs`), `tokenizers` (Apache-2.0) e `rusqlite` 0.40.2 com `bundled-sqlcipher-vendored-openssl`. O Strawberry Perl portátil fica só no ambiente da medição.
- Modelos ONNX baixados do Hugging Face, fora do repositório.

## Success criteria

### Delivery-verifiable

- **SC-001** — Relatórios do S1 e do S2 com números e decisão; ADRs 0012 e 0014 atualizados (aceitos ou revisados).
- **SC-002** — A ferramenta de campo do S3/S4 compila, roda e tem roteiro.

### Post-delivery observation

- **SC-003** — Os números do dia real do usuário (S3/S4) confirmam ou revisam o ADR 0013.

## Decisions and open questions

- Delegação do usuário (13/11/2026): o agente decide a Q-017, escreve as perguntas do S1 e executa o S2; o usuário roda o S3/S4; o S6 fica para depois.
- Nenhuma decisão material pendente para S1/S2.
