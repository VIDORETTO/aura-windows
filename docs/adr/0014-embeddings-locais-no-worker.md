---
status: proposed
---

# Embeddings locais no worker, com modelo escolhido por medição em português

Os embeddings do Acervo são calculados localmente no `aura-worker`, com ONNX Runtime (o `ort` que o workspace já usa), na CPU. O modelo é baixado pelo catálogo, do mesmo jeito que os modelos de voz. Embeddings em nuvem ficam só como opção futura, com consentimento próprio. O modelo padrão sai do spike S1, num conjunto dourado em português com perguntas escritas por uma pessoa, entre três finalistas de licença permissiva e 384 dimensões:

- `ibm-granite/granite-embedding-97m-multilingual-r2` (Apache-2.0);
- `hotchpotch/bekko-embedding-v1-a25m`, com transformer em int8 (MIT);
- `hotchpotch/bekko-embedding-v1-a8m`, com o ONNX padrão (MIT).

O `intfloat/multilingual-e5-small` fica só como linha de base. Cada vetor guarda o identificador do modelo que o gerou. A busca usa o vetor como sinal principal e o FTS5 para termos exatos (códigos, nomes, números) e para contagem e listagem. **A fusão não usa RRF de pesos iguais**: o peso do lexical, ou a regra de quando usá-lo, sai do mesmo conjunto dourado.

Evidência: `docs/research/segundo-cerebro-medicoes-2026-11.md` §2. São 50 documentos sintéticos (+1.000 distratores) e 30 perguntas; ONNX Runtime 1.31.1, 1 thread, Ryzen 7 7800X3D.

| Modelo | Acerto na 1ª posição | ms por documento |
| --- | ---: | ---: |
| e5-small (linha de base) | 0,73 | 6,9 |
| bekko-a8m | 0,80 | 6,5 |
| granite-97m-r2 | 0,87 | 17 |
| bekko-a25m int8 | 0,93 | 16 |
| granite-311m-r2 | 1,00 | 62 |

A fusão RRF de pesos iguais derrubou o acerto na 1ª posição de todos em 7–23 pontos. No MTEB-BR, o granite-97m-r2 também supera o e5-small (0,557 contra 0,507 de nDCG@10).

## Considered options

- **multilingual-e5-small**, o padrão histórico: o mais fraco em perguntas por paráfrase, entre os medidos.
- **granite-embedding-311m-multilingual-r2**: a melhor qualidade, mas ~9× o custo por documento, 313 MB e 768 dimensões (dobra o tamanho do índice).
- **EmbeddingGemma-300m**: o melhor modelo pequeno no MTEB-BR, mas os termos do Gemma não são licença permissiva.
- **Embeddings em nuvem**: qualidade alta, mas tira dados do PC fora de um Turno e custa por chamada.

## Consequences

- CPU do worker: 6,5–17 ms por trecho num desktop rápido. A 2.000 trechos por dia útil, são ~13–34 s de CPU por dia (calculado, não medido), rodando com a máquina ociosa.
- Download de 98–199 MB e pico de 450–660 MB no processo durante o cálculo (medido em Python; o worker em Rust deve medir o próprio).
- Trocar de modelo exige recalcular todos os vetores; o identificador do modelo por vetor permite fazer isso aos poucos.
- O cartão do Bekko recomenda OpenVINO em vez de ONNX Runtime para CPU [autodeclarado]; o S1 mede as duas formas antes de escolher.
- O conjunto dourado do S1 vira teste permanente no `aura-bench`: toda troca de modelo, de corte de trecho ou de fusão roda contra ele.
