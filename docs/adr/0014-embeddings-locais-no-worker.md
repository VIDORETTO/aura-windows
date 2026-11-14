---
status: accepted
---

# Embeddings locais no worker: bekko-a8m com e5-small de reserva, e busca híbrida por RRF

Os embeddings do Acervo são calculados localmente no `aura-worker`, com ONNX Runtime (o `ort` que o workspace já usa), na CPU, e o modelo é baixado pelo catálogo, como os modelos de voz. Embeddings em nuvem ficam só como opção futura, com consentimento próprio.

- **Modelo padrão:** `hotchpotch/bekko-embedding-v1-a8m` (MIT), arquivo `onnx/model.onnx` padrão (tabela de embeddings em int8, transformer em fp32), 384 dimensões, sem prefixo, pooling por média, normalização L2.
- **Reserva:** `intfloat/multilingual-e5-small` (MIT), com prefixos `query:`/`passage:`, para máquinas com pouca RAM ou se o Bekko der problema no ONNX Runtime.
- Cada vetor guarda o identificador do modelo que o gerou.

A busca é **híbrida por RRF (k = 60) com pesos iguais** entre vetor e FTS5 (`unicode61 remove_diacritics 2`). O FTS5 também atende contagem e listagem. Cada trecho é indexado com um cabeçalho montado por regra (fonte, título, data), sem LLM.

Evidência: spike S1 do esforço 046 (`specs/046-spikes-segundo-cerebro/s1.md`):
- runtime em Rust com o `ort` do Aura;
- conjunto dourado sintético de 240 documentos em 6 fontes e 100 perguntas de outro autor;
- paridade com o Python verificada.

| Modelo | MRR só vetor | MRR com RRF 1 : 1 | ms por documento | Pico de RAM |
| --- | ---: | ---: | ---: | ---: |
| bekko-a8m | 0,804 | 0,892 | 16,1–17,0 | 553 MB |
| e5-small | 0,801 | 0,894 | 17,1–17,5 | 435 MB |
| bekko-a25m int8 | 0,832 | 0,897 | 21,0–21,2 | 554 MB |
| granite-97m-r2 | 0,751 | 0,838 | 22,7 | 414 MB |

O FTS5 sozinho deu 0,867. Pelo critério definido antes de medir, a diferença de MRR abaixo de 0,02 conta como empate e desempata pela menor latência por documento: vence o bekko-a8m.

## Considered options

- **multilingual-e5-small como padrão:** qualidade empatada e 118 MB a menos de RAM, mas ~5% mais lento por documento. Fica como reserva.
- **bekko-a25m:** o melhor vetor puro (0,832), mas +30% de latência para +0,005 de MRR com RRF.
- **granite-embedding-97m-multilingual-r2:** à frente no MTEB-BR e nas medições anteriores, mas o pior neste conjunto (vetor 0,751; RRF 0,838) e mais lento.
- **granite-311m-r2:** melhor qualidade nas medições anteriores, mas ~9× o custo e 768 dimensões.
- **EmbeddingGemma-300m:** os termos do Gemma não são licença permissiva.
- **RRF com peso 0,5 no lexical** ou **lexical só com termo raro:** iguais ou piores que o RRF 1 : 1. A proposta I-19 das medições ("não usar RRF de pesos iguais") vinha de um conjunto pequeno e enviesado contra o lexical e fica revertida.
- **Embeddings em nuvem:** tiram dados do PC fora de um Turno e custam por chamada.

## Consequences

- CPU do worker: ~16–17 ms por trecho de ~100 tokens num desktop rápido; a 2.000 trechos por dia útil, ~35 s de CPU por dia (calculado), com a máquina ociosa.
- Download de 130 MB (modelo) + 34 MB (tokenizador); pico de ~550 MB no processo durante o cálculo. O worker é sob demanda (ADR 0005).
- O cartão do Bekko recomenda OpenVINO em CPU (5,5× mais rápido num Ryzen 9, [autodeclarado]). Medir na Fase 1 antes de mudar de runtime.
- Trocar de modelo exige recalcular os vetores; o identificador por vetor permite fazer isso aos poucos.
- O conjunto dourado do S1 vira teste permanente no `aura-bench`: toda troca de modelo, de corte de trecho ou de fusão roda contra ele. Quando houver perguntas reais do usuário, o conjunto passa a incluí-las.
- Limites da evidência: conjunto sintético escrito por modelos, 100 perguntas (diferenças menores que ~0,02 não são distinguíveis), desktop rápido.
