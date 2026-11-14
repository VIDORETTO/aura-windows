# Medições do Segundo Cérebro (novembro de 2026)

Data: 2026-11-12. Complementa `docs/research/segundo-cerebro-complemento-2026-11.md` (citado como
**[C §n]**) com números medidos num PC Windows, o que a sessão anterior, numa VPS Linux, não
conseguiu fazer. Adianta os spikes S1 a S4 de `docs/project/plano-segundo-cerebro.md` §9, mas não
os substitui: os spikes oficiais rodam no código do Aura. Não alterei o plano nem código do
repositório; as mudanças propostas estão na seção 9, "Impacto no plano".

**Como ler.** **[medido]** marca número medido neste PC, nas condições da seção 1. Número calculado
ou extrapolado diz "não medido". Afirmação de terceiros traz o link; **[autodeclarado]** marca
número do próprio fornecedor; **[não verificado]**, o que não confirmei na fonte primária.
"Não encontrado" diz onde procurei. Conteúdo de páginas foi tratado como dado.

**Privacidade.** Nenhum dado do usuário entrou nas medições: documentos e perguntas do T1 são
sintéticos; o T2 usa vetores públicos; o T3 grava só contagens, tempos e tamanhos de texto; o T4
usa páginas públicas renderizadas num perfil temporário do Chrome; o T5 captura uma janela de
teste própria. Scripts, modelos, vetores, imagens e conjuntos de dados ficaram em
`%USERPROFILE%\aura-medicoes` e não entram no git.

## 0. Resumo

**Medido neste PC:** T1 (embeddings em português, 7 variantes de modelo), T2 (busca vetorial de
100 mil a ~1 milhão de vetores, com e sem SQLCipher, e a compilação no MSVC), T3 (custo de
`SetWinEventHook` e da leitura por UIA em 10 minutos de uso sintético, e o efeito nos
navegadores), T4 (10 formatos de imagem × OCR do Windows em pt-BR) e T5 (`DirtyRegionMode` e
`MinUpdateInterval` do WGC). **Por fonte:** como o Recall reconhece janela anônima e uma avaliação
independente do Graphiti (T7).

**Continua sem resposta:** Reddit (bloqueado também daqui; não usei a conta do usuário, T6);
orientação da ANPD; resultado independente do Letta; teste do Recall (o PC não é Copilot+).

**As três descobertas que mais mudam o plano**

1. **SQLCipher multiplica o custo da busca vetorial, e o sqlite-vec não ajuda no Windows.** Com
   a cifra e o cache padrão, toda busca que relê o índice fica 5–10× mais lenta: o primeiro passo
   binário passa de 300 ms a ~1 milhão de vetores. Além disso, no build MSVC o sqlite-vec roda sem
   SIMD (o AVX não compila no `cl.exe`) e o filtro por metadado continua lendo tudo. O que
   funcionou: vetores em BLOB numa tabela comum, em ordem cronológica, filtrando o período por
   faixa de `rowid` e varrendo em Rust (31 ms a 500 mil, com cifra), ou um cache de páginas do
   tamanho do índice binário (66 ms a ~1 milhão). A saída I-3 vira candidata a desenho principal
   (I-21, I-22).
2. **Ler janelas por UI Automation não é barato, e nos navegadores é perigoso.** O algoritmo atual
   do Aura levou 2,6 s por leitura no WhatsApp e 1,4 s de p95 no Word. Uma única leitura do Chrome
   por UIA ligou a acessibilidade completa dele: numa página muito dinâmica, cada tecla passou a
   levar ~12 s e o navegador ocupou ~2,3 núcleos. Navegadores devem ir por captura + OCR, não por
   UIA, e a leitura precisa de orçamento de tempo (I-25, I-31).
3. **Reduzir a janela para 1.280 px antes do OCR destrói o OCR; o formato quase não importa.** Na
   resolução nativa, todos os formatos dão 92–96% de palavras reconhecidas e o WebP sem perdas é o
   menor (44 KB). Reduzindo de 1.920 para 1.280 px, o código cai para 24% e o texto de 12 px para
   16%. OCR antes de reduzir; AVIF fora (0,3–0,9 s por imagem) (I-28).

Também mudam o plano: a fusão RRF de pesos iguais **piorou** a busca em todos os modelos (I-19);
granite-97m-r2 e bekko-a25m superam o e5-small (I-20); o WGC só entrega quadro quando algo muda e
diz exatamente o retângulo alterado (I-29); a janela anônima do Chrome só se reconhece pela UIA,
não pelo título (I-27).

---

## 1. Hardware, sistema e versões

| Item | Valor |
| --- | --- |
| CPU | AMD Ryzen 7 7800X3D, 8 núcleos / 16 threads, 4,2 GHz (WMI), L2 8 MB, L3 96 MB |
| RAM | 32 GB (31,1 GB visíveis) |
| Placa | Gigabyte B650M D3HP (desktop; sem bateria, sempre na tomada) |
| GPU | AMD Radeon RX 9060 XT (não usada nas medições, exceto a captura do T5) |
| Disco | Kingston NV3 1 TB NVMe (sistema e dados) |
| Monitor principal | 1920 × 1080, 96 DPI (escala 100%); há um segundo monitor |
| Windows | Windows 11 Pro 25H2, build **26200.9974** |
| Plano de energia | Equilibrado (`381b4222-…`) |
| Python | 3.14.2 (venv isolado) |
| Rust | 1.98.1 (stable, MSVC), Visual Studio Build Tools 2022 17.14, Windows SDK 10.0.26100 |
| onnxruntime | 1.31.1 (Python, CPUExecutionProvider) |
| tokenizers / numpy / psutil | 0.23.3 / 2.5.4 / 7.2.2 |
| rusqlite / libsqlite3-sys | 0.40.2 / 0.38.2 (os mesmos do `Cargo.lock` do Aura) |
| SQLite / SQLCipher | 3.53.2 (`bundled`) / 4.14.0 community sobre SQLite 3.51.3 (`bundled-sqlcipher-vendored-openssl`) |
| sqlite-vec | 0.1.9 (crate e extensão) |
| crate `windows` | 0.62.2 (a mesma do Aura) |
| Pillow | 13.0.0 (libjpeg-turbo 3.2.0, libwebp 1.6.0, libavif 1.4.2 com aom 3.14.1 para codificar e dav1d 1.5.3 para decodificar) |
| Chrome / Edge / VS Code / Word | 157.0.8103.51 / 157.0.4341.46 / 1.123.0 / 16.0.20527.20196 |

**Condições.** Antes de medir fechei AMDChat, ChatGPT, Spotify e Firefox. Ficaram abertos o
Codex, o Termius e este agente. Cada medição de tempo tem aquecimento e pelo menos 5 repetições;
reporto p50 e p95. As medições de tempo rodaram uma de cada vez, com três exceções registradas:
a primeira rodada de 100 mil vetores do T2 coincidiu com o fim do T1 e foi descartada; a primeira
rodada das opções rápidas cifradas a 500 mil coincidiu com uma compilação e foi repetida (§3.4); e
o usuário usou o PC durante parte do T3 (§4.2, §4.5).

---

## 2. T1 — Embeddings em português (adianta o S1)

### 2.1 Método

- **Conjunto:** 50 documentos curtos, sintéticos, escritos por mim (13 trechos de reunião, 13
  telas, 12 notas, 12 páginas lidas; 51 tokens em média), sem dados do usuário. Há seis pares
  parecidos de propósito (embalagens, incidente, receitas, mesma pessoa, saúde, juros).
- **Perguntas:** 30, escritas por um subagente (Claude) que leu só os 50 documentos e não viu
  nenhum resultado. Pedido: 10 com termo exato do documento e 20 por paráfrase, evitando as
  palavras distintivas. **Viés:** o usuário não escreveu as perguntas; documentos e perguntas
  vêm da mesma família de modelo; as perguntas por paráfrase repetem só **8%** das palavras de 4+
  letras do documento certo (as de termo exato, 66%), o que desfavorece a busca lexical mais do
  que perguntas reais tenderiam a fazer.
- **Segunda rodada:** os mesmos 50 documentos e 30 perguntas com **1.000 distratores** (primeiro
  parágrafo de 1.000 artigos aleatórios da Wikipédia em português, dump 20231101 do
  [wikimedia/wikipedia](https://huggingface.co/datasets/wikimedia/wikipedia), CC BY-SA 3.0,
  semente 42), para o acerto nos 10 primeiros deixar de ser trivial.
- **Modelos:** ONNX Runtime 1.31.1 (Python 3.14.2), CPU, `intra_op_num_threads = 1`,
  `inter_op_num_threads = 1`, execução sequencial, lote 1, truncamento em 512 tokens, um processo
  por modelo (para o pico de RAM). Prefixos conforme o cartão de cada modelo: e5 usa
  `query: `/`passage: `; granite r2 e Bekko não usam prefixo (`config_sentence_transformers.json`
  com prompts vazios; FAQ do Bekko: "No. bekko is trained without prefixes"). Pooling: média
  (e5, Bekko) ou CLS (granite), sempre normalizado.
- **Arquivos ONNX:** e5 `onnx/model_qint8_avx512_vnni.onnx`; granite `onnx/model_quint8_avx2.onnx`;
  Bekko em duas variantes, porque o "int8" do Bekko não é o mesmo que o dos outros: o
  `onnx/model.onnx` padrão guarda só a tabela de embeddings em int8 e o transformer em fp32, e os
  arquivos `*_quint8_avx2_not_recommended.onnx` quantizam o transformer; o cartão chama estes de
  experimentais e recomenda OpenVINO, não ONNX Runtime, para CPU
  ([cartão do a8m](https://huggingface.co/hotchpotch/bekko-embedding-v1-a8m)) [autodeclarado].
- **Latência:** 3 rodadas de aquecimento em 5 documentos; depois 5 passadas completas → 250
  amostras por documento e 150 por pergunta; p50 e p95.
- **RAM:** pico do conjunto de trabalho do processo Python (`psutil`, `peak_wset`), que inclui
  Python, ONNX Runtime e tokenizador; o processo vazio tem 32 MB.
- **Busca:** cosseno sobre todos os documentos. Lexical: SQLite FTS5 com
  `unicode61 remove_diacritics 2` (o tokenizador proposto no plano §5.2), pergunta convertida em
  OR das palavras com 2+ letras fora de uma lista de palavras vazias, ordenada por `bm25`.
  Híbrida: RRF com k = 60 e pesos iguais. Variante: FTS5 com AND de todas as palavras.

### 2.2 Resultados [medido, §1]

Custo por modelo (ONNX Runtime, 1 thread; documentos de 51 tokens e perguntas de ~25 tokens em
média):

| Modelo (arquivo ONNX) | Arquivo | Dim. | ms/documento p50 / p95 | ms/pergunta p50 / p95 | Pico de RAM do processo | Carga |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| multilingual-e5-small (`qint8_avx512_vnni`) — base | 118,3 MB | 384 | 6,9 / 8,4 | 4,1 / 5,2 | 467 MB | 0,55 s |
| granite-embedding-97m-multilingual-r2 (`quint8_avx2`) | 98,2 MB | 384 | 17,2 / 22,0 | 9,5 / 11,7 | 445 MB | 0,90 s |
| bekko-embedding-v1-a8m (`model.onnx` padrão) | 130,1 MB | 384 | 6,5 / 8,9 | 3,5 / 4,6 | 585 MB | 0,89 s |
| bekko-embedding-v1-a8m (`quint8_avx2_not_recommended`) | 106,2 MB | 384 | 4,8 / 6,2 | 2,6 / 3,4 | 561 MB | 0,88 s |
| bekko-embedding-v1-a25m (`model.onnx` padrão) | 199,3 MB | 384 | 23,3 / 30,2 | 12,6 / 16,2 | 659 MB | 1,20 s |
| bekko-embedding-v1-a25m (`quint8_avx2_not_recommended`) | 124,0 MB | 384 | 16,2 / 21,0 | 8,7 / 11,5 | 585 MB | 1,18 s |
| granite-embedding-311m-multilingual-r2 (`quint8_avx2`), opcional | 313,4 MB | 768 | 61,9 / 86,6 | 32,4 / 40,4 | 746 MB | 1,81 s |

Qualidade só com vetor (30 perguntas). "Pos. média" é a posição média do documento certo.

| Modelo | 50 docs: acerto@1 | @3 | @10 | MRR | Pos. média | 1.050 docs: acerto@1 | @10 | MRR | Pos. média |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| FTS5 sozinho (BM25, OR) | 0,500 | 0,567 | 0,867 | 0,584 | 9,1 | 0,467 | 0,700 | 0,543 | 145,6 |
| e5-small (base) | 0,733 | 0,900 | 0,967 | 0,809 | 2,5 | 0,700 | 0,967 | 0,778 | 2,8 |
| granite-97m-r2 | 0,867 | 1,000 | 1,000 | 0,928 | 1,2 | 0,800 | 1,000 | 0,889 | 1,3 |
| bekko-a8m (padrão) | 0,800 | 0,967 | 1,000 | 0,872 | 1,4 | 0,800 | 1,000 | 0,872 | 1,4 |
| bekko-a8m (int8 completo) | 0,767 | 0,900 | 1,000 | 0,845 | 1,7 | 0,767 | 1,000 | 0,844 | 1,7 |
| bekko-a25m (padrão) | 0,933 | 0,967 | 1,000 | 0,953 | 1,2 | 0,933 | 1,000 | 0,953 | 1,2 |
| bekko-a25m (int8 completo) | 0,933 | 0,967 | 1,000 | 0,953 | 1,2 | 0,933 | 1,000 | 0,953 | 1,2 |
| granite-311m-r2 | 1,000 | 1,000 | 1,000 | 1,000 | 1,0 | 1,000 | 1,000 | 1,000 | 1,0 |

Por estilo de pergunta (50 documentos): nas 10 de **termo exato**, todos os modelos e o FTS5
acertam 10/10 na primeira posição. Nas 20 por **paráfrase**, acerto@1: FTS5 0,25; e5-small 0,60;
bekko-a8m int8 completo 0,65; bekko-a8m 0,70; granite-97m 0,80; bekko-a25m 0,90 (as duas
variantes); granite-311m 1,00. Com 1.050 documentos: FTS5 0,20 (acerto@10 de 0,55); e5-small
0,55; granite-97m 0,70; bekko-a8m 0,70; bekko-a25m 0,90; granite-311m 1,00.

Busca híbrida (FTS5 + vetor, RRF k = 60):

| Modelo | Só vetor: acerto@1 / MRR | + FTS5 (OR), 50 docs | + FTS5 (OR), 1.050 docs: @1 / @10 / MRR | + FTS5 (AND) |
| --- | ---: | ---: | ---: | ---: |
| e5-small | 0,733 / 0,809 | 0,667 / 0,770 | 0,667 / 0,867 / 0,748 | igual ao vetor |
| granite-97m-r2 | 0,867 / 0,928 | 0,700 / 0,806 | 0,700 / 0,867 / 0,769 | igual ao vetor |
| bekko-a8m | 0,800 / 0,872 | 0,700 / 0,783 | 0,667 / 0,867 / 0,743 | igual ao vetor |
| bekko-a25m | 0,933 / 0,953 | 0,700 / 0,800 | 0,667 / 0,867 / 0,746 | igual ao vetor |
| granite-311m-r2 | 1,000 / 1,000 | 0,800 / 0,850 | 0,733 / 0,867 / 0,780 | igual ao vetor |

Com AND, o FTS5 só casa 1 das 30 perguntas (perguntas naturais têm palavras que o documento não
tem), e a fusão fica idêntica ao vetor sozinho.

### 2.3 O que isso diz

> **Atualização (13/11/2026, spike S1 do esforço 046):** com um conjunto maior e menos enviesado
> (240 documentos, 100 perguntas de outro autor, sobreposição de 32% nas por paráfrase), em Rust
> com o `ort` do Aura, a fusão RRF de pesos iguais **melhorou** todos os modelos, e o granite-97m-r2
> foi o pior. As conclusões 1 e 3 abaixo valem só para este conjunto pequeno; a decisão está em
> `specs/046-spikes-segundo-cerebro/s1.md` e no ADR 0014.

1. **A busca híbrida com RRF ingênuo não compensou o modelo menor; piorou todos.** Nas perguntas
   por termo exato o vetor já acertava tudo; nas por paráfrase, o BM25 com OR traz documentos
   errados que dividem palavras comuns, e com peso igual eles empurram o certo para baixo (o
   acerto@1 cai 10–35 pontos). Isto contradiz a receita do plano §5.3(2) tal como está escrita
   ("Fusão por posição recíproca") e o número da Anthropic citado na pesquisa (S7), que foi
   medido em outro tipo de corpus. Ressalva forte: as perguntas por paráfrase deste conjunto
   quase não repetem palavras do documento (8%), o que é pior para o lexical do que o uso real.
2. **No custo do e5-small, o bekko-a8m padrão foi melhor** (acerto@1 0,80 contra 0,73; MRR 0,87
   contra 0,81; mesma latência, 6,5 contra 6,9 ms por documento), mas usa ~120 MB a mais de RAM
   de pico e o cartão não recomenda ONNX Runtime.
3. **O granite-97m-r2 confirma o MTEB-BR** (melhor que o e5-small, sobretudo em paráfrase: 0,80
   contra 0,60), ao custo de ~2,5× a latência (17 contra 7 ms por documento).
4. **O bekko-a25m foi o melhor dos pequenos** (0,93; paráfrase 0,90) e a versão com transformer
   int8 manteve exatamente a mesma qualidade com 30% menos tempo (16 ms) e arquivo de 124 MB.
5. O granite-311m-r2 acertou tudo, mas custa ~9× o e5-small por documento (62 ms) e 313 MB.
6. Para o "Bekko em português" (§11 do complemento): **respondido em parte**. Neste conjunto
   pequeno e sintético, o a8m ficou entre o e5-small e o granite-97m, e o a25m acima do
   granite-97m. Não há benchmark público independente em português.

### 2.4 Ressalvas

- 30 perguntas: cada pergunta vale 3,3 pontos; diferenças de 1–2 perguntas (e5 contra bekko-a8m,
  bekko-a8m contra granite-97m) não são significativas. O conjunto oficial do S1 precisa de
  perguntas escritas por uma pessoa e de documentos reais do uso.
- Documentos e perguntas sintéticos, escritos por modelos da mesma família; os 1.000
  distratores são de outro domínio (enciclopédia), mais fáceis de descartar que distratores
  reais (outras reuniões, outras telas).
- Latência num desktop rápido (Ryzen 7 7800X3D) com 1 thread; num notebook comum espere mais.
  O pico de RAM inclui Python e o tokenizador (o `tokenizer.json` do Bekko tem 34 MB).
- Bekko: medido em ONNX Runtime, que o próprio cartão desaconselha para CPU (diz que OpenVINO é
  5,5× mais rápido num Ryzen 9 7950X) [autodeclarado]; OpenVINO não foi testado.

---

## 3. T2 — Busca vetorial com 100 mil a 1 milhão de vetores (adianta o S2)

### 3.1 Compila no MSVC com SQLCipher?

- **Versões:** o workspace pede `rusqlite = "0.40"` com `bundled`; o `Cargo.lock` trava
  `rusqlite 0.40.2` → `libsqlite3-sys 0.38.2`, que embute **SQLite 3.53.2** (sem cifra) e
  **SQLCipher 4.14.0 community sobre SQLite 3.51.3** (`sqlcipher/sqlite3.c`,
  `CIPHER_VERSION_NUMBER 4.14.0`). O complemento §5.1 citava `libsqlite3-sys 0.40` com SQLCipher
  4.20.0: isso é o `master` do rusqlite, não o que o Aura compila hoje.
- **Features de SQLCipher** no `Cargo.toml` do `rusqlite 0.40.2`: `sqlcipher` (liga a uma
  SQLCipher do sistema), `bundled-sqlcipher` (compila a SQLCipher embutida, mas exige OpenSSL do
  sistema via `OPENSSL_DIR`; no Windows o `build.rs` faz `panic!("Missing environment variable
  OPENSSL_DIR…")`) e `bundled-sqlcipher-vendored-openssl` (compila também o OpenSSL, pelo crate
  `openssl-src`).
- **Resultado [medido]:** `bundled-sqlcipher-vendored-openssl` + `sqlite-vec 0.1.9` **compila e
  funciona no MSVC** (Build Tools 2022 17.14, SDK 10.0.26100), desde que haja um **Perl nativo do
  Windows**. Com o Perl do Git for Windows (MSYS, 5.38.2) o `Configure` do OpenSSL falha:
  `Can't locate Locale/Maketext/Simple.pm in @INC`. Com o **Strawberry Perl 5.42.3 portátil**
  (descompactado fora do sistema, só no PATH da sessão) compilou em **5 min 10 s** (release,
  frio). NASM não é necessário: o `openssl-src` já configura com `no-asm`. Instalado: só o Perl
  portátil.
- **Prova de funcionamento [medido]:** `PRAGMA key` aceito, `PRAGMA cipher_version` =
  `4.14.0 community`, `vec_version()` = `v0.1.9`, KNN com `k = ?` e com `ORDER BY distance LIMIT`,
  FTS5 com `remove_diacritics 2` acha "reunião" buscando "reuniao", e o arquivo não começa com
  `SQLite format 3` (está cifrado).
- **SIMD do sqlite-vec no MSVC:** o crate `sqlite-vec 0.1.9` compila o C só com
  `-DSQLITE_CORE` (`build.rs`); as rotinas AVX ficam atrás de `SQLITE_VEC_ENABLE_AVX`, e ligar
  essa macro com `/arch:AVX2` **não compila no `cl.exe`** (o código usa
  `__attribute__((aligned(32)))`, sintaxe de GCC/Clang) [medido]. Ou seja: no build MSVC do Aura,
  a distância float e int8 do sqlite-vec é escalar. clang-cl não estava instalado; não testado.

### 3.2 Método da medição

- **Vetores reais:** [tazarov/dbpedia-entities-all-MiniLM-L6-v2-1M](https://huggingface.co/datasets/tazarov/dbpedia-entities-all-MiniLM-L6-v2-1M)
  (licença MIT declarada no cartão; exportação do Chroma com 981.300 entidades do DBpedia,
  embeddings `all-MiniLM-L6-v2`, 384 dimensões, todos com norma 1). 1.000 vetores sorteados
  (semente 42) viram consultas e saem da base; base de 980.300. Tamanhos: 100.000, 500.000 e
  **980.300** (chamado de "~1 milhão"). Ressalva: são vetores de documento em inglês de outro
  modelo; o recall da quantização binária depende do modelo e deve ser refeito com o modelo
  escolhido no S1.
- **Período:** cada vetor recebe um "dia" em ordem cronológica (`dia = i × 365 / N`, como um
  Acervo que só cresce); o filtro pega os últimos 30 dias (~8,2% das linhas).
- **Opções** (um arquivo de banco por opção, para medir o disco):
  - `vec_f32`, `vec_i8`, `vec_bit`: tabela `vec0` com `float[384]`, `int8[384]`
    (`vec_quantize_int8(?, 'unit')`) ou `bit[384]` (`vec_quantize_binary`), coluna de metadado
    `dia integer`; consulta `WHERE e MATCH ? AND k = 10` (com filtro: `AND dia >= 335`).
  - `vec_bit+f32` e `vec_bit+i8`: `vec_bit` com `k = 100`, depois busca os 100 por `id` numa
    tabela comum com o vetor em float32 ou int8 e reordena em Rust.
  - `blob_f32`, `blob_i8`, `blob_bit+f32` (a saída I-3): tabela comum
    `t(id integer primary key, dia, e blob)`, varredura em Rust (`SELECT id, e FROM t WHERE id >= ?`;
    o filtro de período vira faixa de `rowid`). Produto escalar com AVX2/FMA e contagem de bits
    com POPCNT, escolhidos em tempo de execução; int8 por distância L2.
- **Consultas:** 3 de aquecimento, depois 100 consultas (100 mil) ou 50 (500 mil e ~1 milhão)
  sem cifra; com SQLCipher a 500 mil e ~1 milhão, 50 consultas nas opções rápidas e **10** nas de
  força bruta (cada uma leva segundos). Uma thread. p50 e p95 por consulta, em ms.
- **Recall@10:** contra a busca exata em float32 feita em memória, sobre a mesma base (com filtro,
  só nos últimos 30 dias).
- **SQLite:** padrões (`page_size` 4096, `cache_size` −2000 = 2 MB, sem mmap), journal WAL só na
  carga. Com cifra, `PRAGMA key` e os padrões do SQLCipher 4. Binário em release, `opt-level 3`,
  alvo x86-64 genérico.
- **Disco:** tamanho do arquivo depois de `wal_checkpoint(TRUNCATE)`.
- Comando: `vecbench bench <N> <dir> <consultas> <saida.jsonl>` (código em
  `%USERPROFILE%\aura-medicoes\t2-vec\src\main.rs`, ~330 linhas).

### 3.3 Resultados sem cifra [medido, §1]

p50 / p95 em ms por consulta (k = 10), recall@10 entre parênteses. 100 mil: 100 consultas; 500
mil e ~1 milhão: 50.

| Opção | Filtro | 100 mil | 500 mil | ~1 milhão (980.300) |
| --- | --- | ---: | ---: | ---: |
| vec0 float32 (exata) | — | 88 / 95 (1,00) | 450 / 486 (1,00) | 895 / 914 (1,00) |
| vec0 int8 | — | 38 / 40 (0,95) | 200 / 220 (0,96) | 391 / 424 (0,94) |
| vec0 binário | — | 3,0 / 3,1 (0,47) | 15 / 15 (0,49) | 31 / 34 (0,50) |
| vec0 binário → float32 (100) | — | 10 / 11 (0,90) | 43 / 44 (0,88) | 83 / 85 (0,88) |
| vec0 binário → int8 (100) | — | 10 / 11 (0,89) | 43 / 47 (0,87) | 83 / 88 (0,86) |
| BLOB float32, varredura em Rust (exata) | — | 121 / 142 (1,00) | 618 / 900 (1,00) | 1.239 / 1.275 (1,00) |
| BLOB int8, varredura em Rust | — | 31 / 37 (0,95) | 162 / 184 (0,96) | 335 / 347 (0,94) |
| BLOB binário → float32 (100), em Rust | — | 8,7 / 9,0 (0,90) | 44 / 46 (0,88) | 88 / 92 (0,88) |
| vec0 float32 (exata) | 30 dias | 82 / 108 (1,00) | 342 / 369 (1,00) | 683 / 742 (1,00) |
| vec0 int8 | 30 dias | 26 / 27 (0,95) | 96 / 98 (0,95) | 193 / 211 (0,95) |
| vec0 binário | 30 dias | 4,8 / 4,9 (0,52) | 16 / 25 (0,47) | 31 / 32 (0,44) |
| vec0 binário → float32 (100) | 30 dias | 9,9 / 10 (0,94) | 21 / 21 (0,93) | 38 / 39 (0,89) |
| vec0 binário → int8 (100) | 30 dias | 8,7 / 9,0 (0,91) | 20 / 22 (0,90) | 38 / 38 (0,87) |
| BLOB float32 (exata) | 30 dias | 14 / 14 (1,00) | 49 / 51 (1,00) | 99 / 114 (1,00) |
| BLOB int8 | 30 dias | 1,2 / 2,7 (0,95) | 12 / 13 (0,95) | 25 / 32 (0,95) |
| BLOB binário → float32 (100) | 30 dias | 5,5 / 7,4 (0,94) | 6,6 / 6,9 (0,93) | 10 / 14 (0,89) |

### 3.4 Resultados com SQLCipher [medido, §1]

Mesmo formato. A 500 mil e ~1 milhão, as opções de força bruta (`vec0 float32`, `vec0 int8`,
`BLOB float32`) usaram 10 consultas; as demais, 50. As opções rápidas foram repetidas num
processo próprio, sem nada rodando em paralelo (a primeira rodada coincidiu com uma compilação).

| Opção | Filtro | 100 mil | 500 mil | ~1 milhão |
| --- | --- | ---: | ---: | ---: |
| vec0 float32 (exata) | — | 914 / 938 (1,00) | 4.581 / 4.653 (1,00) | 9.003 / 9.095 (1,00) |
| vec0 int8 | — | 253 / 322 (0,95) | 1.257 / 1.275 (0,96) | 2.461 / 2.471 (0,93) |
| vec0 binário | — | 33 / 35 (0,47)¹ | 165 / 168 (0,49) | 331 / 346 (0,50) |
| vec0 binário → float32 (100) | — | 9,2 / 11 (0,90)¹ | 205 / 211 (0,88) | 392 / 441 (0,88) |
| vec0 binário → int8 (100) | — | 8,7 / 9,1 (0,89)¹ | 203 / 209 (0,87) | 389 / 406 (0,86) |
| BLOB float32 (exata) | — | 1.186 / 1.213 (1,00) | 5.972 / 5.988 (1,00) | 11.663 / 11.771 (1,00) |
| BLOB int8 | — | 247 / 305 (0,95) | 1.228 / 1.250 (0,96) | 2.417 / 2.608 (0,94) |
| BLOB binário → float32 (100) | — | 16 / 18 (0,90) | 217 / 294 (0,88) | 413 / 429 (0,88) |
| vec0 float32 (exata) | 30 dias | 907 / 930 (1,00) | 4.503 / 4.920 (1,00) | 8.828 / 8.847 (1,00) |
| vec0 int8 | 30 dias | 238 / 246 (0,95) | 1.176 / 1.194 (0,98) | 2.313 / 2.435 (0,97) |
| vec0 binário | 30 dias | 42 / 43 (0,52) | 189 / 266 (0,47) | 369 / 376 (0,44) |
| vec0 binário → float32 (100) | 30 dias | 52 / 53 (0,94) | 205 / 211 (0,93) | 389 / 399 (0,89) |
| vec0 binário → int8 (100) | 30 dias | 44 / 45 (0,91) | 203 / 221 (0,90) | 386 / 394 (0,87) |
| BLOB float32 (exata) | 30 dias | 96 / 99 (1,00) | 489 / 509 (1,00) | 959 / 974 (1,00) |
| BLOB int8 | 30 dias | 1,2 / 1,4 (0,95) | 100 / 101 (0,95) | 197 / 201 (0,95) |
| **BLOB binário → float32 (100)** | 30 dias | 10 / 11 (0,94) | **31 / 31 (0,93)** | **47 / 48 (0,89)** |

¹ Em 100 mil com cifra, o binário variou de 1 a 33 ms conforme o que o processo tinha feito
antes (rodando depois das buscas float32/int8 no mesmo processo: ~1 ms; sozinho: 33 ms,
reproduzido duas vezes; sem cifra não varia: 2,75 ms). Causa não identificada. A 500 mil e ~1
milhão o resultado é estável.

**Cache maior com cifra** (~1 milhão, `PRAGMA cache_size = -120000`, 120 MB por conexão, 50
consultas):

| Opção | Sem filtro | 30 dias |
| --- | ---: | ---: |
| vec0 binário | 11 / 12 (0,50) | 7,2 / 9,2 (0,44) |
| vec0 binário → float32 (100) | 66 / 69 (0,88) | 18 / 20 (0,89) |
| BLOB binário → float32 (100) | 62 / 69 (0,88) | 7,2 / 8,1 (0,89) |

**Disco** (MB; sem cifra / com cifra; carga em segundos entre parênteses):

| Arquivo | 100 mil | 500 mil | ~1 milhão |
| --- | ---: | ---: | ---: |
| vec0 float32 | 158 / 161 | 786 (8 s) / 802 (14 s) | 1.540 (14 s) / 1.570 (27 s) |
| vec0 int8 | 42 / 43 | 208 / 213 | 409 / 417 |
| vec0 binário | 8,0 / 8,1 | 40 / 41 | 79 / 80 |
| BLOB float32 | 205 / 205 | 1.027 / 1.027 | 2.013 / 2.013 |
| BLOB int8 | 41 / 41 | 205 / 205 | 403 / 403 |
| BLOB binário | 6,0 / 6,2 | 30 / 31 | 59 / 61 |

Combinações: "binário + reordenação em float32" ocupa índice binário + vetores float32
(~1 milhão: 79 + 2.013 = 2,1 GB com BLOB; ou 79 + 1.540 se o float32 ficar no vec0); "binário +
int8" ocupa 79 + 403 = **482 MB** — um quarto do disco, com recall@10 só 0,01–0,02 menor.

### 3.5 O que isso diz

> **Atualização (13/11/2026, spike S2 do esforço 046):** com o desenho do ADR 0012, vetores reais do
> bekko-a8m em português e o mesmo SQLCipher, o recall@10 com 100 candidatos foi 0,84 (200: 0,90).
> Com cache dimensionado, 1 milhão sem filtro deu 70 ms de p95 no desktop e 104 ms numa
> aproximação de máquina modesta. A decisão está em `specs/046-spikes-segundo-cerebro/s2.md` e no
> ADR 0012, agora aceito.

1. **Sem cifra, a meta de p95 ≤ 300 ms a 500 mil só cai com binário (ou int8 com filtro).**
   float32 exato no vec0 dá 486 ms de p95; int8, 220 ms; binário + reordenação, 44 ms.
2. **No build MSVC, int8 é ~2,2× mais rápido que float32** no vec0 (200 contra 450 ms a 500
   mil). Isso contraria o benchmark citado no complemento (int8 = float32 num i9), que deve ter
   usado um build com SIMD. Aqui a distância é escalar (§3.1), e o int8 lê 4× menos bytes.
3. **Com SQLCipher, tudo que relê o índice inteiro fica 5–10× mais lento**: o vec0 lê todos os
   blocos de vetores a cada consulta, e cada página de 4 KB é decifrada e verificada (HMAC-SHA512, conforme `PRAGMA cipher_hmac_algorithm`)
   porque o cache padrão é de 2 MB. Até o binário passa da meta a ~1 milhão (331 / 346 ms).
4. **O que resolve com cifra:** (a) um cache de páginas que segure o índice binário (~80 MB por
   milhão de vetores): binário → float32 a ~1 milhão cai para 66 ms; ou (b) filtrar o período
   **pela ordem de inserção** antes de buscar: o BLOB binário → float32 dos últimos 30 dias leva
   31 ms a 500 mil e 47 ms a ~1 milhão com o cache padrão.
5. **O filtro por metadado do vec0 quase não ajuda**: ele continua lendo todos os blocos (o
   binário com filtro de 30 dias leva o mesmo que sem filtro). O BLOB com faixa de `rowid`
   lê só ~8% das páginas.
6. **Recall da reordenação com 100 candidatos: 0,86–0,90.** Com estes vetores (MiniLM, inglês),
   o primeiro passo binário perde ~10% dos 10 vizinhos exatos. Não testei mais candidatos
   (só a quantidade 100, proposta em I-2).
7. **Disco:** o BLOB float32 ocupa 30% mais que o vec0 float32 (dois vetores de 1.536 bytes por
   página de 4 KB). A cifra quase não muda o tamanho (+0–2%).

---

## 4. T3 — Custo da UI Automation e do SetWinEventHook (adianta o S3, I-5)

### 4.1 Método

- **Programa:** `uiaprobe`, em Rust com o crate `windows` 0.62 (o mesmo do Aura). Assina
  `SetWinEventHook` fora do processo (`WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS`) para
  `EVENT_SYSTEM_FOREGROUND` e `EVENT_OBJECT_NAMECHANGE`, em **dois ganchos separados** (modo
  `sep`) ou num gancho único de 0x0003 a 0x800C (modo `wide`, o desenho que o Twinkle Tray usava).
  Do `NAMECHANGE` só aproveita a troca de título da janela em primeiro plano
  (`OBJID_WINDOW`, `CHILDID_SELF`). A cada evento aproveitado, outra thread (MTA) lê o texto da
  janela com **o mesmo algoritmo de `aura-win::uia::window_text`**: `TextPattern` na raiz; senão
  `FindAll(TreeScope_Descendants)` até 2.000 nós, `TextPattern` de documentos e edições, `Name` de
  textos. Sem agrupar eventos: uma leitura por evento.
- **Privacidade:** o texto lido é contado e descartado na hora; o arquivo de saída só tem
  contagens de eventos, tempos, número de caracteres e nome do executável.
- **Uso:** 10 minutos de uso **sintético** (não houve uso humano): um roteiro alterna ao acaso
  entre Chrome (4 abas públicas: Wikipédia, docs do Python, GitHub, G1), Edge (3 abas: Wikipédia,
  Microsoft Learn, BBC Brasil), VS Code (um arquivo de código aberto, digitando linhas), Word
  (documento em branco por COM, digitando frases, fechado sem salvar), Teams e WhatsApp (só vêm
  para a frente, sem teclado). Navegadores e VS Code com perfis temporários. Troca de abas com
  Ctrl+Tab e rolagem com PageDown.
- **CPU e RAM do programa:** `GetProcessTimes` e `GetProcessMemoryInfo` no fim; conjunto de
  trabalho amostrado a cada segundo.
- **Custo de disparar eventos para os outros processos:** `uiaprobe notify` chama
  `NotifyWinEvent(EVENT_OBJECT_NAMECHANGE)` 20.000 vezes × 10 amostras numa janela própria,
  sem nenhum gancho, com o `uiaprobe run sep` e com o `uiaprobe run wide` rodando (o método do
  relato do Twinkle Tray).
- **Efeito nos navegadores:** Chrome e Edge, perfil temporário, sem e com o `uiaprobe` rodando
  (ele lê a janela por UIA quando ela vem para a frente). Página local com 4.000 elementos e 40
  trocas de texto, cor e `aria-label` por quadro, e um campo de texto. Teclas enviadas pelo CDP
  (`Input.dispatchKeyEvent`, não depende de foco e não toca nas janelas do usuário), uma a cada
  100 ms, com limite de 15 s por tecla: mede o tempo até o navegador confirmar a tecla e o atraso
  do `keydown` dentro da página (`performance.now() - event.timeStamp`). CPU somada da árvore de
  processos do navegador e memória do maior processo. Estado de `chrome://accessibility` /
  `edge://accessibility` lido numa aba nova. Uma primeira versão com `SendInput` e o
  [Speedometer 3.1](https://browserbench.org/Speedometer3.1/) foi descartada: as teclas não
  chegavam à página (foco do teclado fora do conteúdo) e o processo da página trocava no meio da
  medição.

### 4.2 Resultados: 10 minutos de uso [medido, §1]

**Ressalva:** uso sintético por roteiro. Nos últimos ~3 minutos o usuário abriu o Firefox e
usou o PC, então há uso humano real misturado (6 leituras do Firefox). O Teams não abriu janela
durante o teste (o roteiro não o encontrou) e **não foi medido**. A janela do terminal deste
agente troca de título o tempo todo (indicador de progresso) e gerou a maior parte das leituras
por troca de título.

| Medida | Valor |
| --- | ---: |
| Duração | 600 s |
| Eventos `EVENT_SYSTEM_FOREGROUND` | 147 (0,25/s) |
| Eventos `EVENT_OBJECT_NAMECHANGE` recebidos (todos os objetos, todos os processos) | 10.665 (17,8/s) |
| … dos quais troca de título da janela em primeiro plano | 201 (0,34/s) |
| Leituras por UIA (uma por evento aproveitado) | 348 |
| Tempo por leitura, todas as janelas: p50 / p95 | 15 ms / 851 ms |
| Leituras com ≥ 200 caracteres de texto | 25% (ver por app) |
| CPU do programa | 10,5 s em 600 s = **1,75% de um núcleo** (0,11% dos 16 threads) |
| RAM do programa (conjunto de trabalho) | média 22 MB, pico **38 MB** |

Por aplicativo (tempo da leitura completa da janela com o algoritmo do Aura):

| App | Leituras | ms p50 | ms p95 | ms máx. | Caracteres (mediana) | Com ≥ 200 caracteres | Nós lidos (máx.) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Chrome 157 | 25 | 82 | 631 | 880 | 8.484 | 60% | 2.000 (teto) |
| Edge 157 | 27 | 80 | 389 | 389 | 20.000 (teto) | 96% | 2.000 (teto) |
| VS Code 1.123 | 14 | 90 | 124 | 222 | 1.971 | 86% | 241 |
| Word (Microsoft 365, 16.0.20527) | 26 | 144 | 1.426 | 1.429 | 311 | 54% | 180 |
| WhatsApp (WhatsApp.Root.exe) | 12 | **2.648** | **3.024** | 3.640 | 20.000 (teto) | 100% | 2.000 (teto) |
| Firefox (uso humano) | 6 | 240 | 331 | 331 | 20.000 (teto) | 100% | 2.000 (teto) |
| Explorer (área de trabalho, barra de tarefas) | 59 | 22 | 119 | 172 | 0 | 3% | 206 |
| Windows Terminal | 177 | 12 | 27 | 129 | 54 | 0% | 22 |
| Teams | — | — | — | — | — | — | não medido |

Nenhuma leitura usou o `TextPattern` da raiz: em todas as janelas o texto veio da varredura de
descendentes.

### 4.3 Resultados: custo de disparar eventos com o gancho ativo [medido, §1]

`NotifyWinEvent` disparado por um processo de teste, 20.000 chamadas × 10 amostras, com outro
processo segurando o gancho (sem leitura UIA). µs por chamada, p50 / p95 das amostras:

| Evento disparado | Sem o gancho | Ganchos separados (FOREGROUND + NAMECHANGE) | Gancho único 0x0003–0x800C |
| --- | ---: | ---: | ---: |
| `EVENT_OBJECT_NAMECHANGE` (0x800C) | 20,5 / 80 | **213 / 217** | 214 / 216 |
| `EVENT_OBJECT_LOCATIONCHANGE` (0x800B) | 326 / 332 | 323 / 326 | **430 / 434** |

- **Assinar um evento custa a quem o dispara ~0,19 ms por evento** (de 20 para 213 µs no
  `NAMECHANGE`), com ganchos separados ou não. No uso medido havia 17,8 `NAMECHANGE`/s no sistema:
  ~3,4 ms de CPU por segundo espalhados pelos processos que disparam (~0,3% de um núcleo),
  **não medido** diretamente, calculado.
- **Ganchos separados evitam o custo dos eventos não assinados** (o achado do Twinkle Tray): o
  `LOCATIONCHANGE`, que o gancho largo inclui, sobe +107 µs; com ganchos separados, nada.
- A linha de base do `LOCATIONCHANGE` já é 326 µs sem o meu programa: algum software aberto neste
  PC já assina esse evento. As linhas de base dependem do que está instalado.
- Diferença para o relato do Twinkle Tray (1,9 µs sem gancho, 27,6 µs com o largo): ordem de
  grandeza diferente, outra máquina e outro evento de teste; a conclusão qualitativa é a mesma.

### 4.4 Janela anônima vista de fora [medido, §1]

Chrome 157 e Edge 157 com perfil temporário, abrindo `example.com`:

| Navegador | Título da janela (`GetWindowText`) | Nome do elemento raiz na UIA | Outro elemento |
| --- | --- | --- | --- |
| Chrome, normal | "Example Domain - Google Chrome" | (sem marca) | — |
| Chrome, `--incognito` | **igual ao normal** | "Example Domain - Google Chrome (Modo anônimo)" | botão "Anônima" |
| Edge, normal | "Example Domain - Perfil 1 – Microsoft Edge" | (sem marca) | — |
| Edge, `--inprivate` | "Example Domain – [InPrivate] – Microsoft Edge" | "Example Domain – Microsoft Edge (InPrivate)" | "Navegação InPrivate" |

Classe (`Chrome_WidgetWin_1`) e estilo estendido são iguais nos dois modos. Ou seja: pelo
título, o Aura **não** reconhece a janela anônima do Chrome; pela UIA, reconhece, mas o texto
vem traduzido para o idioma do navegador ("Modo anônimo", "Incognito"…) e a consulta já é uma
chamada de UIA (que liga a acessibilidade do navegador, ver 4.5).

### 4.5 Efeito de um cliente UIA nos navegadores [medido, §1]

Página local com 4.000 elementos e 40 trocas de texto, cor e `aria-label` por quadro; 50 teclas
(Chrome) ou 20 (Edge) enviadas pelo CDP (`Input.dispatchKeyEvent`), uma a cada 100 ms, limite de
15 s por tecla. "Confirmação" é o tempo até o navegador devolver a tecla processada.

| Caso | Teclas confirmadas | Confirmação p50 / p95 / máx. | Atraso no JS (`keydown`) p50 / p95 | CPU dos processos do navegador na digitação | Maior processo (RSS) | `chrome://accessibility` |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Chrome, sem o `uiaprobe` | 50 / 50 | **16 / 21 / 23 ms** | 5,5 / 10 ms | 8 s em 5,8 s | 227 MB | só "Native accessibility API support" |
| Chrome, **com** o `uiaprobe` | 50 / 50 | **12.380 / 14.537 / 14.838 ms** | 8,6 / 17,9 ms | **1.392 s em 593 s (~2,3 núcleos)** | **598 MB** | + "Web accessibility" e "Extended properties" |
| Edge, sem o `uiaprobe` | 20 / 20 | 18 / 20 / 20 ms | 6,9 / 9,5 ms | 3,9 s em 2,4 s | 222 MB | não lido (erro na aba de diagnóstico) |
| Edge, com o `uiaprobe` | — | **não medido**: a aba da página não respondeu ao CDP | — | — | — | — |

- **Ser cliente de UIA de um navegador com página muito dinâmica trava a entrada.** Com o
  `uiaprobe` lendo a janela uma vez (quando ela veio para a frente), o Chrome ligou a árvore de
  acessibilidade completa e passou a levar ~12 s para confirmar cada tecla, ocupando ~2,3 núcleos
  e triplicando a memória do processo principal. O atraso medido dentro da página continuou
  baixo: a fila está antes do processo da página, no processamento dos eventos de
  acessibilidade. Reproduz, em ordem de grandeza, o relato citado em [C §7.2] (fila de entrada de
  43 s).
- **Mesmo sem o `uiaprobe`, o Chrome já estava com "Native accessibility API support" ligado**
  neste PC: algum programa aberto (não identificado) já age como cliente de acessibilidade.
- **Ressalvas:** página sintética e extrema (DOM trocando a cada quadro), um caso por condição,
  o usuário estava usando o PC. O teste foi interrompido a pedido do usuário pelo custo de CPU;
  as variantes com a página pesada (400 trocas por quadro), o Edge com o `uiaprobe` e o
  Speedometer com o `uiaprobe` não foram medidos. Numa primeira tentativa (Chrome, página pesada,
  com o `uiaprobe`), a primeira tecla não foi confirmada em 9,5 minutos; o processo da página
  acumulou 687 s de CPU e 1,19 GB, e o teste foi encerrado.
- Speedometer 3.1 no Chrome **sem** o `uiaprobe`: 39,4 e 31,3 em duas execuções (variação alta,
  com o PC em uso); sem par "com" para comparar.

---

## 5. T4 — Formato de imagem e OCR (adianta o S4, I-9)

### 5.1 Método

- **Páginas públicas:** artigo "Edgar von Wahl" da Wikipédia em português (texto do dump
  20231101, [CC BY-SA](https://huggingface.co/datasets/wikimedia/wikipedia)) em 16 px e em 12 px;
  [tutorial do Python em pt-BR](https://docs.python.org/pt-br/3/tutorial/introduction.html)
  (15 px); e as primeiras 120 linhas de `rusqlite/src/lib.rs` (MIT) num editor escuro estilo VS
  Code (Consolas 14 px, números de linha). Cada página é aberta no Chrome 157 (perfil temporário,
  quiosque 1920 × 1080, `--force-device-scale-factor=1`) e capturada por `PrintWindow` com
  `PW_RENDERFULLCONTENT`; o texto-fonte é o `innerText` do que coube na tela, lido por CDP.
- **Duas referências PNG por página, ambas com 1.280 px de largura:** `nat1280` (página com 1.280
  px de largura, sem escala) e `de1920` (janela de 1.920 px reduzida para 1.280 com Lanczos, como
  o plano faz com a janela ativa, "lado maior ≤ 1.280 px"). A redução diminui a letra em 1/3.
- **Formatos (Pillow 13.0.0):** PNG; JPEG q60, q80, q90 (4:2:0, padrão) e q80 4:4:4; WebP q75,
  q90 e sem perdas; AVIF q60 e q80 (velocidade 6, padrão, 1 thread).
- **OCR:** `Windows.Media.Ocr` em pt-BR, pelo mesmo caminho do `aura-win::ocr::recognize`
  (`SoftwareBitmap::CreateCopyWithAlphaFromBuffer`, BGRA pré-multiplicado, linhas juntadas com
  `\n`), sobre a imagem decodificada de cada formato.
- **CER:** distância de Levenshtein entre o texto do OCR e o texto-fonte, dividida pelo tamanho do
  texto-fonte, depois de reduzir todo espaço em branco a um espaço; também sem acento e caixa.
- **Tempo:** codificação e decodificação em memória, 1 aquecimento + 5 repetições, p50 e p95.

### 5.2 Resultados [medido, §1]

Recall de palavras do OCR (fração das palavras do texto-fonte reconhecidas, insensível à ordem),
por página e formato:

| Formato | código nativo | código 1920→1280 | docs nativo | docs 1920→1280 | wiki 12 px nativo | wiki 12 px 1920→1280 | wiki 16 px nativo | wiki 16 px 1920→1280 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| PNG (referência) | 90,8% | 23,7% | 99,8% | 87,9% | 95,7% | 16,4% | 96,7% | 88,5% |
| JPEG q60 | 78,6% | 22,5% | 99,4% | 68,8% | 93,3% | 12,4% | 97,7% | 86,3% |
| JPEG q80 | 86,1% | 27,7% | 99,8% | 83,3% | 95,0% | 14,9% | 96,0% | 89,4% |
| JPEG q90 | 85,0% | 22,5% | 99,4% | 85,8% | 95,7% | 15,6% | 96,3% | 89,1% |
| JPEG q80 4:4:4 | 82,7% | 29,5% | 99,8% | 83,0% | 96,3% | 15,3% | 95,8% | 89,6% |
| WebP q75 | 85,5% | 24,9% | 99,8% | 85,0% | 95,6% | 14,5% | 95,6% | 88,0% |
| WebP q90 | 85,5% | 20,8% | 99,6% | 86,4% | 95,8% | 15,3% | 96,3% | 88,7% |
| WebP sem perdas | 90,8% | 23,7% | 99,8% | 87,9% | 95,7% | 16,4% | 96,7% | 88,5% |
| AVIF q60 | 86,7% | 23,1% | 99,8% | 84,5% | 95,4% | 14,1% | 96,7% | 87,5% |
| AVIF q80 | 86,1% | 23,1% | 99,4% | 86,6% | 95,8% | 15,2% | 96,3% | 88,8% |

Médias por tipo de referência (4 páginas cada; CER só nas 3 de texto corrido, porque no código o
OCR lê a coluna de números de linha como um bloco à parte e o CER passa a medir a ordem, não o
reconhecimento):

| Formato | Nativo: KB médio (mín–máx) | Nativo: recall | Nativo: CER | 1920→1280: KB médio (mín–máx) | 1920→1280: recall | 1920→1280: CER | Codificar p50 (p95 máx) | Decodificar p50 (p95 máx) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| PNG | 109 (67–145) | 95,7% | 2,4% | 393 (150–558) | 54,2% | 32,7% | 16 ms (22) | 3,5 ms (4,1) |
| JPEG q60 | 159 (66–212) | 92,2% | 1,5% | 98 (37–133) | 47,5% | 41,0% | 1,7 ms (2,4) | 2,8 ms (4,2) |
| JPEG q80 | 223 (89–301) | 94,2% | 1,3% | 136 (48–186) | 53,9% | 34,5% | 1,8 ms (2,5) | 3,2 ms (5,0) |
| JPEG q90 | 305 (118–412) | 94,1% | 1,6% | 188 (63–256) | 53,3% | 33,6% | 1,9 ms (3,0) | 3,5 ms (5,2) |
| JPEG q80 4:4:4 | 345 (134–474) | 93,6% | 1,5% | 188 (68–259) | 54,3% | 34,5% | 2,9 ms (5,0) | 4,0 ms (6,3) |
| WebP q75 | 133 (44–184) | 94,1% | 1,6% | 81 (24–117) | 53,1% | 34,7% | 97 ms (136) | 13 ms (23) |
| WebP q90 | 195 (67–270) | 94,3% | 1,6% | 131 (38–191) | 52,8% | 34,0% | 103 ms (150) | 15 ms (23) |
| **WebP sem perdas** | **44 (18–60)** | **95,7%** | 2,4% | 255 (95–372) | 54,2% | 32,7% | 53 ms (63) | 6,6 ms (6,9) |
| AVIF q60 | 70 (23–99) | 94,6% | 2,0% | **75 (22–103)** | 52,3% | 35,3% | 313 ms (737) | 13 ms (18) |
| AVIF q80 | 104 (33–147) | 94,4% | 1,7% | 116 (35–158) | 53,4% | 33,6% | 284 ms (916) | 14 ms (19) |

Os tempos de codificação e decodificação das colunas finais são a mediana das 8 páginas (p50 de
cada uma) e o maior p95. OCR: ~100–125 ms por imagem, igual em todos os formatos.

**Licenças dos codificadores** (das versões embutidas no Pillow 13.0.0): libjpeg-turbo
([licença IJG + BSD-3 + zlib](https://github.com/libjpeg-turbo/libjpeg-turbo/blob/main/LICENSE.md)),
libwebp ([BSD-3](https://github.com/webmproject/libwebp/blob/main/COPYING)), libavif
([BSD-2](https://github.com/AOMediaCodec/libavif/blob/main/LICENSE)) com aom
([BSD-2 + licença de patentes AOM](https://aomedia.googlesource.com/aom/+/refs/heads/main/LICENSE))
e dav1d ([BSD-2](https://code.videolan.org/videolan/dav1d/-/blob/master/COPYING)); Pillow
([MIT-CMU/HPND](https://github.com/python-pillow/Pillow/blob/main/LICENSE)). [não verificado:
não abri cada arquivo de licença nesta sessão].

### 5.3 O que isso diz

1. **O que derruba o OCR é reduzir a imagem, não o formato.** Na resolução nativa, todos os
   formatos ficam entre 92% e 96% de palavras reconhecidas (JPEG q60 é o pior: 79% no código e
   69% nos docs reduzidos). Reduzir uma janela de 1.920 px para 1.280 px, como o plano faz,
   derruba o OCR do código de 91% para 24% e da Wikipédia em 12 px de 96% para 16%, já no PNG.
   A letra de 12 px vira ~8 px e o OCR do Windows deixa de ler.
2. **Para tela de texto na resolução nativa, WebP sem perdas é o menor e o fiel**: 44 KB em
   média (18–60 KB) contra 70 KB do AVIF q60, 133 KB do WebP q75 e 223 KB do JPEG q80. Código em
   ~53 ms, leitura em ~7 ms. Imagem reduzida (com tons intermediários do redimensionamento)
   inverte o quadro: aí o sem perdas sobe para 255 KB e o AVIF q60 (75 KB) e o WebP q75 (81 KB)
   ganham.
3. **AVIF custa caro para codificar**: ~0,3 s de p50 e até 0,9 s de p95 por imagem em 1 thread
   (WebP com perdas ~0,1 s; JPEG ~2 ms), sem ganho de OCR.
4. O "~80 KB por imagem" do plano §4.3 cabe com WebP sem perdas na resolução nativa (44 KB) ou
   com AVIF/WebP com perdas na imagem reduzida — mas a imagem reduzida não serve para OCR depois.

---

## 6. T5 — `DirtyRegionMode` do Windows.Graphics.Capture

### 6.1 Método

- Build 26200 (≥ 26100): testado. Programa `wgcprobe` em Rust (crate `windows` 0.62). Ele cria
  uma janela de teste própria (900 × 600, texto fixo desenhado por GDI) e a captura com
  `GraphicsCaptureItem` (`CreateForWindow`), `Direct3D11CaptureFramePool::CreateFreeThreaded`
  (2 buffers, BGRA), `IsCursorCaptureEnabled = false`, `IsBorderRequired = false`.
- **Cenários (30 s cada):** janela parada; janela com só o cursor de texto piscando
  (`CreateCaret`/`ShowCaret`, intervalo padrão do sistema); janela com mudança pequena (um
  contador redesenhado a cada 1 s numa área de 160 × 30 px).
- **Configurações:** `DirtyRegionMode` = `ReportOnly` (0) e `ReportAndRender` (1);
  `MinUpdateInterval` = 0, 100 ms e 1 s. São 18 execuções.
- **Registro:** cada quadro que chega no `FrameArrived`, com o instante, o número de retângulos
  em `DirtyRegions` e a área somada; CPU do processo (inclui a janela de teste).

### 6.2 Resultados [medido, §1]

Janela de captura 886 × 593 px (inclui a moldura). Quadros em 30 s; "após 1 s" exclui o quadro
inicial.

| Cenário | Modo | MinUpdateInterval | Quadros | Após 1 s | Regiões por quadro | Área típica | CPU do processo em 30 s |
| --- | --- | ---: | ---: | ---: | --- | ---: | ---: |
| Parada | ReportOnly | 0 / 100 ms / 1 s | 1 / 1 / 1 | 0 / 0 / 0 | 1 (janela inteira, só o primeiro) | — | 16–62 ms |
| Parada | ReportAndRender | 0 / 100 ms / 1 s | 2 / 1 / 1 | 1 / 0 / 0 | 1; o quadro extra (aos 24 s) trouxe 6 retângulos na moldura | — | 16–62 ms |
| Cursor piscando | ReportOnly | 0 / 100 ms / 1 s | 9 / 9 / 5 | 6 / 6 / 4 | 1 (4 × 24 px, o cursor) | 96 px² | 31–62 ms |
| Cursor piscando | ReportAndRender | 0 / 100 ms / 1 s | 9 / 9 / 5 | 6 / 6 / 4 | 1 (4 × 24 px) | 96 px² | 47–78 ms |
| Mudança pequena (1/s) | ReportOnly | 0 / 100 ms / 1 s | 34 / 32 / 30 | 32 / 29 / 29 | 1 (162 × 32 px; às vezes um 2º quadro de 70 × 18 logo depois) | 5.184 px² | 31–62 ms |
| Mudança pequena (1/s) | ReportAndRender | 0 / 100 ms / 1 s | 34 / 32 / 30 | 32 / 30 / 29 | idem | 5.184 px² | 0–94 ms |

- **Chega quadro quando nada muda? Não.** Com a janela parada, só chegou o quadro inicial em 30 s
  (5 de 6 casos). A exceção teve um quadro aos 24 s com retângulos só na moldura (barra de
  título e bordas), provavelmente por troca de janela ativa (o T6 abria janelas do Chrome
  nesse período).
- **Cursor piscando:** chega um quadro por piscada (~530 ms, `CursorBlinkRate` = 530) com um
  retângulo do tamanho do cursor, mas **só nos primeiros ~5 s**: o cursor do sistema para de
  piscar após `CaretTimeout` = 5.000 ms sem entrada (valores lidos em
  `HKCU\Control Panel\Desktop` neste PC; documentação pública do `CaretTimeout`: não encontrada
  na página de `SystemParametersInfo`).
- **`MinUpdateInterval` limita a taxa:** com 1 s, chega no máximo um quadro por segundo
  (piscadas e trocas agrupadas); 0 e 100 ms não mudaram nada a 1 troca por segundo.
- **`ReportOnly` × `ReportAndRender`:** nenhuma diferença na contagem de quadros nem em
  `DirtyRegions`; os dois modos preenchem as regiões. Não inspecionei os pixels para ver se
  `ReportAndRender` desenha algo no quadro.
- **`DirtyRegions` é preciso:** devolve só o retângulo alterado (cursor 4 × 24; contador
  162 × 32), em coordenadas da janela capturada.
- Custo: 0–94 ms de CPU em 30 s para o processo inteiro (janela de teste + captura), < 0,3% de
  um núcleo.
- Ressalvas: uma janela GDI própria, sem aceleração de vídeo; navegadores e apps com animação
  podem gerar quadros sem mudança visível. Não testei captura de monitor inteiro.

---

## 7. T6 — Reddit

**Não acessível também a partir deste PC; não usei a conta do usuário.** Tentativas desta vez
(12/11/2026, rede doméstica):

| Caminho | Resultado |
| --- | --- |
| `www.reddit.com/…/search.json` e `about.json` por `curl` | 403, página "You've been blocked by network security" |
| `old.reddit.com/…/.json` por `curl` | 302 para `/login/?reason=lor2` |
| `old.reddit.com` num Chrome real, perfil temporário, sem login | Página "Entre para usar o Reddit antigo… é necessário ter uma conta para acessar o Reddit antigo" |
| `www.reddit.com` (HTML) no mesmo Chrome, sem login | **Abriu** busca e listas de r/ObsidianMD em 3 páginas |
| `fetch()` do JSON público de dentro da página aberta (mesma origem, mesmo navegador) | 403 nas 6 buscas (uma por subreddit) |
| `www.reddit.com` (HTML) logo depois | "You've been blocked by network security" para o navegador também (bloqueio por IP) |
| [PullPush](https://api.pullpush.io) | 526 |
| [Arctic Shift](https://arctic-shift.photon-reddit.com) | 422 "Timeout. Maybe slow down a bit" em 5 consultas, com e sem filtro de título e de data |

Como o acesso que funcionou exigiria contornar o bloqueio de rede ou usar a conta pessoal do
usuário no navegador dele, parei aqui. As tabelas das seções 2 e 3 de `segundo-cerebro.md` não
mudaram. **Encerrado por decisão do usuário em 13/11/2026:** o T6 fica como "inacessível". O
fórum do Obsidian, o Tildes e o Hacker News já cobrem as mesmas dores [C §2], e usar a conta
pessoal acrescentaria pouco.

---

## 8. T7 — Pesquisa web que ficou sem resposta

### 8.1 Como o Recall detecta janela anônima

- **Respondido.** A Microsoft documenta o mecanismo: o navegador marca a própria janela com
  `SetInputScope(hwnd, IS_PRIVATE)` (valor 61) ao entrar em modo anônimo, e "must also have a
  `http` or `https` protocol handler registered before `SetInputScope` will support the behavior";
  também deve parar de publicar atividades do usuário nesse modo
  ([Recall guidance for developers of web browsers](https://learn.microsoft.com/en-us/windows/apps/develop/windows-integration/recall/recall-web-browsers),
  `ms.date` 01/12/2025, lido em 12/11/2026).
- O Chromium põe `IS_PRIVATE` no escopo de entrada quando o aprendizado está desligado, e o
  comentário do código diz que isso vale em modo "incognito" ou "guest"
  ([`tsf_input_scope.cc`](https://chromium.googlesource.com/chromium/src/+/HEAD/ui/base/ime/win/tsf_input_scope.cc),
  função `CreateInputScope`). O Brave marca todas as janelas, não só as privadas
  ([Brave](https://brave.com/privacy-updates/35-block-recall/)) [não verificado: lido por resumo de busca].
- **O que isso significa para o Aura:** o sinal existe, mas é um escopo de entrada do TSF
  definido pelo próprio app. API pública para **outro processo** ler o escopo de entrada de uma
  janela: não encontrada (procurei a documentação do Recall, de `SetInputScope` e o código do
  Chromium). Medição neste PC: ver T3 (o que um processo de fora enxerga numa janela anônima).
- **Teste com o Recall:** não feito. Este PC não é Copilot+: não há NPU (`Get-PnpDevice -Class
  ComputeAccelerator` vazio) nem componentes do Recall instalados.

### 8.2 Orientação da ANPD sobre uso doméstico, gravação de reunião ou memória de tela

- **Continua não encontrada.** Desta vez procurei: busca por "fins exclusivamente particulares e
  não econômicos" restrita a gov.br/anpd (achei só notas técnicas sobre outros temas e o texto da
  lei); busca por guia da ANPD sobre gravação de reuniões (achei o
  [guia para o Poder Público](https://www.gov.br/anpd/pt-br/centrais-de-conteudo/materiais-educativos-e-publicacoes/guia-poder-publico-anpd-versao-final.pdf/@@download/file),
  que não trata do tema, e textos de escritórios e blogs que citam "orientação da ANPD" sem
  indicar o documento). Nada muda na seção 9 do complemento: o S6 continua exigindo advogado.

### 8.3 Resultados independentes de Zep/Graphiti e Letta

- **Graphiti: achado.** Wolff e Bennati (KTH), [arXiv 2601.07978](https://arxiv.org/abs/2601.07978)
  (v1 de 12/01/2026, v5 de 03/09/2026), testbed reproduzível no LoCoMo com gpt-4o-mini como
  respondedor, juiz LLM da mesma família e embeddings `text-embedding-3-small`. Acurácia (média
  ± desvio em 10 conversas, tabela do [HTML](https://arxiv.org/html/2601.07978)): mem0
  81,08 ± 3,19%; RAG 78,31 ± 5,75%; contexto inteiro 77,16 ± 5,21%; **Graphiti 56,03 ± 5,56%**;
  cognee 55,27 ± 3,60%. Custo total: RAG US$ 0,65, mem0 US$ 5,43, Graphiti US$ 6,95. Os autores
  declaram independência ("independence from any commercial interest… central concerns"); não
  há declaração de financiamento, e o testbed adapta código dos repositórios do mem0 e do Zep.
  Leitura: um RAG simples empata com o mem0 e supera o grafo temporal, a uma fração do custo.
- **Letta:** resultado independente **não encontrado**. Achei só o número do próprio Letta
  (74,0% no LoCoMo com gpt-4o-mini usando ferramentas de arquivo) citado por terceiros
  [autodeclarado, não verificado na fonte primária]. Procurei: arXiv 2025–2026 com LoCoMo e
  LongMemEval, a análise crítica do MemPalace ([arXiv 2604.21284](https://arxiv.org/html/2604.21284v1)),
  que marca Letta como "not reported", e o GRAVITY ([arXiv 2605.01688](https://arxiv.org/abs/2605.01688)),
  cujo resumo não nomeia os sistemas comparados.

### 8.4 Medições publicadas do custo de UIA e de SetWinEventHook

- **SetWinEventHook: achado um relato com números.** No
  [issue #1352 do Twinkle Tray](https://github.com/xanderfrangos/twinkle-tray/issues/1352)
  (16/10/2026), um usuário mediu o custo de **outro** processo disparar um evento enquanto um
  processo mantém o gancho: `NotifyWinEvent` levou 1,9 µs sem gancho, 1,9 µs com dois ganchos
  separados (primeiro plano e troca de nome) e **27,6 µs com um gancho único de 0x0003 a
  0x800C**; `CB_ADDSTRING` foi de 72 µs para 145 µs. Windows 11, Intel Core de 8ª geração; método:
  "a small test app while another test process held the hook". O mantenedor fechou o issue com
  um commit três dias depois. Um relato, sem ferramenta de medição nomeada. Reproduzido neste PC
  no T3.
- **Custo de UIA nos navegadores:** medição publicada em porcentagem de CPU **não encontrada**.
  O Chrome diz que o código de acessibilidade, "when enabled, it can impact loading times,
  performance, and battery life" ([Chrome for Developers](https://developer.chrome.com/blog/chromium-accessibility-performance)),
  sem número; o código do Chromium espera um tempo antes de desligar o suporte quando o cliente
  some, para não reconstruir a árvore toda hora
  ([browser_accessibility_state_impl.cc](https://chromium.googlesource.com/chromium/src/+/5db31b51f232cfd6b7b99a8919e739c7c45d0f55/content/browser/accessibility/browser_accessibility_state_impl.cc))
  [não verificado: lido por resumo de busca]. Medição neste PC: T3.

---

## 9. Impacto no plano

Não alterei `docs/project/plano-segundo-cerebro.md` nem o complemento [C §10]. Abaixo, o que os
resultados fazem com I-1 a I-18 e as propostas novas, a partir de I-19.

### 9.1 I-1 a I-18

| # | Proposta do complemento | Veredito | Evidência |
| --- | --- | --- | --- |
| I-1 | Candidatos do S1: e5-small, granite-97m-r2, Bekko a8m/a25m; granite-311m como teto | **Confirma** | Os três candidatos superam o e5-small nas perguntas por paráfrase; o granite-311m acertou tudo (§2) |
| I-2 | Primeiro passo binário + reordenação; "int8 economiza disco, não tempo" | **Confirma o desenho; derruba a premissa** | Binário → float32 a 500 mil: 43 / 44 ms sem cifra (recall@10 0,88). Mas no build MSVC o int8 foi 2,2× mais rápido que o float32, porque o sqlite-vec roda escalar (§3.1, §3.5) |
| I-3 | Saída documentada: BLOB no `memory.db` + varredura em Rust | **Confirma, e sobe de saída a candidato principal** | Com SQLCipher e filtro de período por faixa de `rowid`, o BLOB binário → float32 leva 31 ms a 500 mil, contra 205 ms do vec0 (§3.4). Ver I-22 |
| I-4 | Dizer o que a cifra cobre e o que não cobre | Sem teste novo | — |
| I-5 | O S3 mede o efeito nos navegadores; chamadas pontuais em vez de assinaturas de eventos UIA | **Confirma e agrava** | Uma única leitura por UIA ligou a árvore completa do Chrome; numa página muito dinâmica, cada tecla passou a levar ~12 s e o navegador ocupou ~2,3 núcleos (§4.5). Chamadas pontuais não bastam: a primeira chamada já liga o modo. Ver I-31 |
| I-6 | Excluir janelas com afinidade de exibição | Sem teste novo | — |
| I-7 | Redação de segredos | Sem teste novo | — |
| I-8 | Em Reunião, gravar por mudança visual; piso de 1,5 s; batida de 30 s | **Confirma, com um meio mais barato** | O WGC só entrega quadro quando algo muda e informa o retângulo exato que mudou (§6). Isso serve de sinal de "mudou" sem comparar imagens |
| I-9 | O S4 mede o tamanho real a 1.280 px; AVIF só se couber no orçamento de CPU | **Confirma o AVIF fora; derruba a redução para 1.280 antes do OCR** | AVIF: 0,3–0,9 s por imagem sem ganho de OCR. Reduzir de 1.920 para 1.280 derruba o OCR de código de 91% para 24% (§5) |
| I-10 a I-17 | Agenda, `memory_count`/`memory_list`, confiança pela fonte, teto que não apaga o destilado, escrita no Cofre, tipos de propriedade, MCP antecipado, consentimento | Sem teste novo | O T7 trouxe mais um dado a favor de I-12: no LoCoMo, o Graphiti acertou 56% e um RAG simples 78% (§8.3) |
| I-18 | Busca boa em português como argumento | **Confirma em parte** | Modelos pequenos acertam 80–93% na 1ª posição em português, mas num conjunto sintético de 30 perguntas (§2) |

### 9.2 Propostas novas

| # | Decisão ou trecho do plano | O que a medição mostrou | Mudança proposta |
| --- | --- | --- | --- |
| I-19 | §5.3 (2): "Duas buscas, uma fusão… Fusão por posição recíproca" | RRF com pesos iguais **piorou** todos os modelos: acerto@1 cai 7–23 pontos no geral e 10–35 na paráfrase; com AND, o FTS5 quase não casa (§2.3) | No S1, comparar RRF com peso menor para o lexical e "lexical só quando a pergunta tem termo raro" (código, nome, número) contra vetor puro. Não adotar RRF de pesos iguais por padrão. O FTS5 continua necessário para termos exatos e para `memory_count`/`memory_list` (I-11) |
| I-20 | §9 S1: escolha do modelo | granite-97m-r2: 0,87 de acerto@1 e 17 ms por documento; bekko-a25m int8: 0,93 e 16 ms; bekko-a8m padrão: 0,80 e 6,5 ms; e5-small: 0,73 e 6,9 ms (§2) | Levar ao S1 oficial: granite-97m-r2, bekko-a25m (int8 completo) e bekko-a8m (padrão). Tirar o e5-small da posição de favorito. Medir o Bekko também com OpenVINO, que o cartão recomenda |
| I-21 | §5.2: "SQLCipher… Compatibilidade com sqlite-vec e com o build MSVC é o spike S2"; §8.1: `memory_search` ≤ 300 ms p95 com 500 mil | Compila e funciona. Mas, com a cifra e o cache padrão de 2 MB, toda busca que relê o índice fica 5–10× mais lenta; o binário do vec0 passa de 300 ms a ~1 milhão (§3.4) | O ADR 0012 deve fixar: a busca vetorial roda num processo com cache de páginas que comporte o índice binário (~80 MB por milhão de vetores; deixou o binário → float32 em 66 ms a ~1 milhão). Como isso estoura o orçamento de +40 MB do host, a busca fica no worker, não no host. Alternativa: filtrar o período por faixa de `rowid` antes (I-22) |
| I-22 | ADR 0012 proposto: "FTS5 e sqlite-vec"; I-3 como saída | O sqlite-vec é escalar no MSVC (o AVX não compila no `cl.exe`), o filtro por metadado do vec0 continua lendo tudo, e o BLOB ordenado no tempo com varredura em Rust (SIMD) é igual ou melhor em todos os casos com filtro (§3) | Tornar "BLOB em tabela comum, ordem cronológica, índice binário (48 B por vetor) + reordenação em int8 (384 B)" o desenho principal, sem extensão. O sqlite-vec fica como opção. Isso também elimina a dependência de um projeto pré-1.0 parado desde maio |
| I-23 | §5.2 e plano de build | A SQLCipher com OpenSSL embutido exige **Perl nativo do Windows** no build; o Perl do Git falha; o primeiro build leva +5 min (§3.1) | Registrar no ADR 0012 e na documentação de build/CI: Strawberry Perl (ou OpenSSL pré-compilado com `OPENSSL_DIR`) é pré-requisito; medir o tempo de CI |
| I-24 | §5.2: "Vetores quantizados em 8 bits… ~190 MB varridos por busca" | Reordenar em int8 em vez de float32 custa 0,01–0,02 de recall e ocupa 403 MB por milhão em vez de 2 GB (§3.4) | Guardar só binário + int8 por padrão (~480 MB por milhão de trechos); float32 não precisa ser guardado |
| I-25 | §4.2: "UI Automation devolve o texto exato da janela… É barato" | O algoritmo atual (`FindAll` de até 2.000 nós) levou **2,6 s** por leitura no WhatsApp e 1,4 s de p95 no Word; Chrome e Edge 80 ms de p50 e 390–630 ms de p95 (§4.2) | Ler com orçamento de tempo (por exemplo, abortar acima de 200 ms e cair para OCR), pedir só os tipos de controle de texto com `CacheRequest`, e nunca ler a cada evento: assentar ~1,5 s como o plano já diz (o protótipo leu a cada evento, o pior caso) |
| I-26 | §4.2: gatilhos por `SetWinEventHook` | Cada evento assinado custa ~0,19 ms a quem o dispara; a faixa larga ainda soma +0,1 ms aos eventos de posição; houve 17,8 `NAMECHANGE`/s no uso (§4.3) | Assinar só `EVENT_SYSTEM_FOREGROUND` e `EVENT_OBJECT_NAMECHANGE`, em ganchos separados, nunca uma faixa. Avaliar assinar o `NAMECHANGE` só do processo em primeiro plano (parâmetro `idProcess`, refeito a cada troca de janela); não medido |
| I-27 | §4.5: "Excluídos de fábrica… janelas anônimas de navegador" | O título da janela anônima do Chrome é igual ao normal; a UIA expõe "(Modo anônimo)" / "(InPrivate)" no nome da janela, traduzido; o `IS_PRIVATE` que o Recall usa não é legível de fora (§4.4, §8.1) | Detectar janela anônima pelo nome UIA da raiz, com uma tabela de traduções por idioma, e pelo título no Edge. Na dúvida, não capturar. Documentar que navegadores sem essa marca não são reconhecidos |
| I-28 | §4.3: imagem "lado maior ≤ 1.280 px, com perdas, ~80 KB" | Reduzir de 1.920 para 1.280 px antes do OCR derruba o reconhecimento de código e de texto de 12 px para 16–24%; na resolução nativa, o WebP sem perdas tem 44 KB em média e é fiel (§5) | Fazer o OCR sobre a captura na resolução nativa, antes de reduzir. Guardar telas de texto em WebP sem perdas na resolução nativa (cabe nos ~80 KB). A versão reduzida serve só de miniatura. Evitar JPEG q60 e AVIF |
| I-29 | §4.2: "Otimização posterior: retângulos alterados do DXGI ou do WGC… documentação rala"; ADR 0008 (GDI) | No build 26200, o WGC por janela não entrega quadro quando nada muda, informa o retângulo exato que mudou e respeita `MinUpdateInterval`; custo < 0,3% de um núcleo. O cursor de texto para de piscar após 5 s sem entrada (§6) | Reavaliar no ADR 0013 o WGC por janela como gatilho barato de "mudou algo visível": substitui o hash perceptual para "nada mudou" e dá a área alterada. Exige build ≥ 26100; manter GDI como reserva |
| I-30 | §8.1: "CPU do host com captura de Momentos ≤ 1% em média" | Ganchos + leitura UIA a cada evento: 1,75% de um núcleo (0,11% da CPU total de 16 threads) e 38 MB de pico (§4.2) | Deixar explícito no orçamento se "1%" é de um núcleo ou do total. Do jeito medido (pior caso, sem agrupar eventos), passa de 1% de um núcleo e fica abaixo de 1% do total |
| I-31 | §4.2: "Texto por UI Automation + app, título, endereço" para todas as janelas | Ler um navegador por UIA liga a acessibilidade completa dele e, com DOM dinâmico, trava a digitação por segundos (§4.5) | Por padrão, **não ler navegadores (Chrome, Edge, Electron) por UIA**: usar captura + OCR na resolução nativa (I-28) e o título/endereço da janela. Leitura por UIA de navegador só como opção, com aviso. O S3 oficial mede quanto tempo o Chrome leva para desligar o modo depois que o cliente para, e repete com páginas reais |

---

## 10. Não encontrado / não medido

| Item | Situação | Onde procurei ou por que não medi |
| --- | --- | --- |
| Threads do Reddit (P1, T6) | **Não acessível** | 8 caminhos, ver §7. Não usei a conta do usuário |
| Recall: teste da janela anônima neste PC | Não medido | O PC não é Copilot+ (sem NPU, sem componentes do Recall) |
| API para outro processo ler o `IS_PRIVATE` de uma janela | Não encontrado | Página de orientação do Recall para navegadores, `SetInputScope`, código do Chromium (§8.1) |
| Orientação da ANPD sobre uso doméstico, gravação de reunião ou memória de tela | Não encontrado | Busca restrita a gov.br/anpd e por guias/notas técnicas (§8.2) |
| Resultado independente do Letta | Não encontrado | arXiv 2025–2026 (LoCoMo, LongMemEval), análise do MemPalace, GRAVITY (§8.3) |
| Custo de UIA em navegadores, publicado em % de CPU | Não encontrado | Blog do Chrome sobre desempenho de acessibilidade, código do Chromium (§8.4); medido aqui (§4.5) |
| Documentação de `CaretTimeout` | Não encontrado | Página de `SystemParametersInfo`; valor lido do registro deste PC (§6.2) |
| Edge com o `uiaprobe`; página pesada (400 trocas); Speedometer com o `uiaprobe` | Não medido | Teste interrompido a pedido do usuário pelo custo de CPU (§4.5) |
| Quanto tempo o Chrome leva para desligar a acessibilidade depois que o cliente UIA para | Não medido | Fica para o S3 oficial |
| Teams no T3 | Não medido | O roteiro não achou a janela do Teams durante o teste |
| Uso humano real de 10 minutos no T3 | Não medido | Uso sintético por roteiro; o usuário usou o PC nos minutos finais |
| sqlite-vec compilado com clang-cl (com AVX) | Não medido | clang-cl não instalado |
| Bekko com OpenVINO | Não medido | Fora do protocolo; o cartão diz que é 5,5–6,1× mais rápido que ONNX Runtime [autodeclarado] |
| Mais de 100 candidatos na reordenação do binário | Não medido | Só o valor proposto em I-2 |
| Partição (`partition key`) do vec0 por mês | Não medido | Só o metadado `dia` e a faixa de `rowid` |
| Recall do binário com os vetores do modelo escolhido, em português | Não medido | Vetores públicos em inglês (MiniLM); refazer no S2 oficial |
| Notebook modesto (pedido em I-2) | Não medido | Só este desktop; números de notebook tendem a ser maiores |
| `ReportAndRender` desenhando algo nos pixels do quadro | Não medido | Só os metadados (`DirtyRegions`) foram comparados |
| Causa da variação de 1 a 33 ms do binário com cifra a 100 mil | Não identificada | Reproduzida duas vezes; não ocorre sem cifra (§3.4) |
| Licenças dos codificadores, arquivo por arquivo | [não verificado] | Links para os arquivos de licença de cada projeto, não abertos nesta sessão (§5.2) |

---

## 11. Como repetir

Tudo em `%USERPROFILE%\aura-medicoes` (fora do repositório): `scripts\` (Python), `t2-vec\`,
`t3-uia\`, `t4-ocr\ocrcli\`, `t5-wgc\` (projetos Cargo). Nada disso entra no git.

```text
# ambiente
python -m venv venv
venv\Scripts\pip install onnxruntime numpy tokenizers huggingface_hub psutil pyarrow pillow rapidfuzz websocket-client pywin32

# T1 — modelos de dl_onnx.py; um processo por modelo; depois a avaliação
venv\Scripts\python -I scripts\t1_embed.py <modelo> t1-embeddings\docs.json t1-embeddings\queries.json t1-embeddings\run
venv\Scripts\python -I scripts\t1_eval.py t1-embeddings\docs.json t1-embeddings\queries.json t1-embeddings\run e5-small granite-97m bekko-a8m bekko-a8m-q8 bekko-a25m bekko-a25m-q8 granite-311m
#   distratores: scripts\t1_distr_hf.py sobre wikimedia/wikipedia 20231101.pt, train-00002-of-00006.parquet, semente 42; T1_REPS=1

# T2 — vetores: scripts\t2_prep.py (tazarov/dbpedia-entities-all-MiniLM-L6-v2-1M, semente 42)
cargo build --release --no-default-features --features plain              # sem cifra
set PATH=<strawberry>\perl\bin;%PATH% & set CARGO_TARGET_DIR=target-cipher & cargo build --release   # SQLCipher
vecbench bench <N> <dir_db> <consultas> <saida.jsonl>     # VB_CFGS=lista, VB_CACHE_KB=n

# T3
uiaprobe run sep|wide <segundos> <saida.json> [noread]
uiaprobe notify 20000 800C|800B
uiaprobe anon "<parte do título>"
venv\Scripts\python -I scripts\t3_driver.py <uiaprobe.exe> <saida> 600
venv\Scripts\python -I scripts\t3_browser.py <uiaprobe.exe> <saida>

# T4
venv\Scripts\python -I scripts\t4_render.py t4-ocr\fonte\textos.json t4-ocr\fonte\codigo_rusqlite_lib.rs t4-ocr\paginas
venv\Scripts\python -I scripts\t4_codecs.py t4-ocr\paginas <ocrcli.exe> t4-ocr\res
venv\Scripts\python -I scripts\t4_palavras.py t4-ocr\res\t4.json t4-ocr\paginas

# T5
wgcprobe static|caret|small 0|1 0|100|1000 30
```
