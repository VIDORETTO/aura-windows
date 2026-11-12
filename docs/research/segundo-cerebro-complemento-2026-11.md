# Complemento de pesquisa: Segundo Cérebro (novembro de 2026)

Data: 2026-11-12. Complementa `docs/research/segundo-cerebro.md` e alimenta
`docs/project/plano-segundo-cerebro.md`. Não altera o plano: as mudanças propostas estão na
seção 10, "Impacto no plano".

**Como ler.** Cada afirmação traz o link da fonte lida. **[autodeclarado]** marca número ou
afirmação do próprio fornecedor ou autor do projeto. **[não verificado]** marca o que não
consegui confirmar na fonte primária. "Não encontrado" diz onde procurei. Conteúdo de páginas
foi tratado como dado; instruções embutidas em código de terceiros foram ignoradas.

**O que nós medimos: nada.** O teste local de embeddings (P3) ficou pendente por decisão do
usuário (vai rodar no computador pessoal; protocolo na seção 3.5). As médias da tabela de
português (seção 3.1) foram calculadas por mim a partir dos resultados brutos publicados pelo
MTEB-BR; não são medição nossa.

---

## 0. Resumo

**Respondido:** opiniões diretas do fórum do Obsidian, Tildes e Hacker News (P1); desenho real
de captura, deduplicação, disco e cifra do screenpipe, Recall, OpenRecall e Windrecorder (P2);
qualidade de embeddings em português por benchmark independente (P3); licença, manutenção e
combinação sqlite-vec + SQLCipher + rusqlite (P4); licenças de modelos visuais (P5); semântica
documentada das APIs do Windows (P6); sintaxe da CLI do Obsidian e o que os usuários instalam
(P7); avaliação independente de Mem0 (P8); texto da LGPD e práticas de aviso (P9).

**Sem resposta:** Reddit (bloqueado por todos os caminhos tentados, seção 1); custo de CPU da
UI Automation no Windows medido por alguém; impacto de WebP/AVIF/JPEG na qualidade do OCR;
semântica de `DirtyRegionMode`; mecanismo do Recall para detectar janela anônima; orientação
da ANPD sobre uso doméstico; desempenho de Bekko em português.

**As três descobertas que mais mudam o plano**

1. **Para embeddings em português, o candidato permissivo mais forte do tamanho certo é o
   `granite-embedding-97m-multilingual-r2` (Apache-2.0), que não estava no plano.** No
   benchmark nativo MTEB-BR ele supera o `multilingual-e5-small` em recuperação (0,557 contra
   0,507 de nDCG@10 médio em seis tarefas). O EmbeddingGemma é o melhor modelo pequeno (0,653),
   mas os termos do Gemma não são licença permissiva. Bekko não foi avaliado em português.
2. **Ser cliente de UI Automation tem custo fora do Aura.** Um processo que carrega
   `UIAutomationCore.dll` faz Chrome e Firefox ligarem a árvore de acessibilidade completa; um
   relato com rastreamento ETW mediu congelamento de até 43 s de entrada no Chrome. E a cifra em
   repouso (SQLCipher + DPAPI) não protege contra código rodando como o mesmo usuário: foi assim
   que o TotalRecall extraiu dados do Recall mesmo com enclave VBS. O spike S3 e o ADR 0012
   precisam tratar dos dois.
3. **No sqlite-vec, vetor de 8 bits economiza disco, não tempo.** Num benchmark publicado com
   100 mil vetores de 384 dimensões, int8 levou o mesmo tempo que float32 (39 ms) e binário
   levou 2 ms. A meta de p95 ≤ 300 ms com 500 mil trechos pede primeiro passo binário com
   reordenação, não int8 puro. E o sqlite-vec está sem commits desde maio de 2026, ainda pré-1.0.

---

## 1. Método e limites

| Fonte pedida | Resultado | Como |
| --- | --- | --- |
| Fórum do Obsidian, tópico 89221 | Lido inteiro (10 posts) | API JSON do Discourse |
| Tildes "I deleted my second brain" | Lido inteiro (33 comentários) | HTML direto |
| Hacker News 47729694 | Lido inteiro (154 comentários). **É "Introduction to Obsidian"** (11/04/2026), não "I deleted my second brain" | API do Algolia HN |
| HN "I deleted my second brain" | Achado e lido: item 44402470 (28/06/2025, 598 pontos, 344 comentários com texto) | API do Algolia HN |
| Reddit (r/ObsidianMD, r/PKMS, r/Zettelkasten, r/selfhosted, r/LocalLLaMA, r/ClaudeAI) | **Não acessível.** `reddit.com` e `old.reddit.com` devolvem 403 ("blocked by network security") a esta VPS; o leitor Jina recebe o mesmo 403; o espelho PullPush devolveu 526; a busca web da Anthropic recusa o domínio; o canal Reddit do agent-reach exige login; o Internet Archive listou URLs de 2025 mas ficou fora do ar nas consultas seguintes, e as cópias arquivadas não trazem comentários | Seis caminhos tentados |
| Substitutos do Reddit | Fórum do Obsidian (mais 6 tópicos de 2025–2026), HN (mais 4 discussões) | Busca do Discourse e do Algolia |

Discussões lidas para P1 e P7, com data do primeiro post:

| Discussão | Data | Tamanho |
| --- | --- | --- |
| [Fórum Obsidian 89221](https://forum.obsidian.md/t/every-attempt-at-pkm-has-landed-me-in-the-same-place-a-huge-mess/89221), "Every attempt at PKM… a huge mess" (autor: romebot) | 01/10/2024 | 10 posts, 10 autores |
| [Fórum Obsidian 115302](https://forum.obsidian.md/t/115302), "My Obsidian PKM system became a substitute for real thinking" (sajjadkhavari) | 14/06/2026 | 10 posts, 9 autores |
| [Fórum Obsidian 117819](https://forum.obsidian.md/t/117819), cofre entre dispositivos e acessível ao Claude (froisjea) | 30/08/2026 | 3 posts |
| [Fórum Obsidian 94732](https://forum.obsidian.md/t/94732), "Obsidian Sync incorrectly duplicates sections of files" | 12/01/2025 | 105 posts |
| [Fórum Obsidian 108029](https://forum.obsidian.md/t/108029), arquivo de conflito sem edição em dois aparelhos | 15/11/2025 | 43 posts |
| [Fórum Obsidian 113166](https://forum.obsidian.md/t/113166) e [114998](https://forum.obsidian.md/t/114998), defeitos da CLI com agentes | 04 e 06/2026 | 3 e 1 posts |
| [Tildes 1oth](https://tildes.net/~tech/1oth/i_deleted_my_second_brain), sobre o texto de Joan Westenberg | 28/06/2025 | 33 comentários, 27 autores |
| [HN 44402470](https://news.ycombinator.com/item?id=44402470), "I deleted my second brain" | 28/06/2025 | 344 comentários, 227 autores |
| [HN 47729694](https://news.ycombinator.com/item?id=47729694), "Introduction to Obsidian" | 11/04/2026 | 154 comentários, 104 autores |
| [HN 49024620](https://news.ycombinator.com/item?id=49024620), "Launch HN: Screenpipe (YC S26)" | 23/07/2026 | 61 comentários, 37 autores |
| [HN 44839965](https://news.ycombinator.com/item?id=44839965), "How would you build second brain in the AI era?" | 08/08/2025 | 5 comentários |

---

## 2. P1 — Dores reais de quem monta segundo cérebro

### 2.1 Contagem por tema

Método: li todos os comentários e, para dar uma ordem de grandeza reprodutível, contei os
comentários que casam com uma expressão regular por tema (script no scratchpad da sessão). A
contagem por palavra-chave tem falso positivo e falso negativo; a coluna "Fontes independentes"
vem da leitura, contando sites distintos (e discussões distintas no mesmo site).

| Tema | HN 44402470 | HN 47729694 | HN 49024620 | Tildes | Fórum 89221 | Fórum 115302 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Manter custa mais do que devolve | 20 | 7 | 0 | 0 | 2 | 1 |
| Captura em excesso, acúmulo | 51 | 6 | 1 | 2 | 4 | 1 |
| Achar importa mais que organizar | 4 | 1 | 2 | 1 | 2 | 0 |
| IA/LLM como saída | 39 | 14 | 25 | 7 | 1 | 0 |
| Espiral de configuração e plugins | 4 | 40 | 4 | 4 | 4 | 1 |
| Sincronização, conflito, duplicata | 2 | 46 | 0 | 1 | 1 | 0 |
| "Escrever é o valor, não reler" | 4 | 0 | 0 | 4 | 0 | 0 |
| Arquivar em vez de apagar | 40 | 0 | 1 | 2 | 0 | 0 |
| Privacidade, confiança | 3 | 7 | 11 | 4 | 0 | 0 |

### 2.2 Dores, separando recorrência de opinião

| # | Dor | Recorrência | Evidência direta |
| --- | --- | --- | --- |
| P1.1 | **Manter o sistema vira trabalho.** | Recorrente: 4 sites, 5 discussões | "you have to spend SO MUCH time writing notes" ([ErrorNoBrain, HN](https://news.ycombinator.com/item?id=44402670)); "a Second Job instead of a Second Brain" ([nivertech, HN](https://news.ycombinator.com/item?id=44440855)); "spending more time with the bureaucracy of it instead of actually doing work" ([tombert, HN](https://news.ycombinator.com/item?id=47757801)); "I'd become a professional organizer, not a better thinker" ([sajjadkhavari, fórum](https://forum.obsidian.md/t/115302)); decidir o lugar certo de cada nota "is a waste of time" (JCAPER, [Tildes](https://tildes.net/~tech/1oth/i_deleted_my_second_brain)). A frase "The organizing tax was higher than the value I got back", citada de segunda mão na pesquisa anterior, **não foi localizada** nas discussões lidas. |
| P1.2 | **Acúmulo sem uso, inclusive de transcrições.** | Recorrente: 3 sites, 4 discussões | "I now have hundreds or even thousands of transcriptions that are effectively meaningless… they just pile up, collecting, not connecting" ([romebot, fórum 89221](https://forum.obsidian.md/t/every-attempt-at-pkm-has-landed-me-in-the-same-place-a-huge-mess/89221)); "Bigger graph = smarter me, or so I thought" ([M0r13n, HN](https://news.ycombinator.com/item?id=44403262)); "I literally don't access 90% of my notes" ([scop, HN](https://news.ycombinator.com/item?id=44406041)); "What actually happens to these notes after three months? … nothing" ([fórum 115302](https://forum.obsidian.md/t/115302)). |
| P1.3 | **Na hora de achar, a pessoa não lembra que existe.** | Recorrente: 2 sites, 4 discussões | "the awareness required for retrieval is challenging no matter how well everything is organized" ([Stray-Sticky-Note, fórum 89221](https://forum.obsidian.md/t/every-attempt-at-pkm-has-landed-me-in-the-same-place-a-huge-mess/89221)); "Search and tags still rules over everything" ([on_the_train, HN](https://news.ycombinator.com/item?id=47757377)); "Ripgrep search was always enough" ([nextos, HN](https://news.ycombinator.com/item?id=44840072)); velocidade para abrir notas antigas decide se a pessoa volta a elas ([emadda, HN](https://news.ycombinator.com/item?id=44404334)). |
| P1.4 | **IA no cofre erra perguntas de contagem e listagem.** | **Opinião de um autor**, mas descreve limite conhecido de busca por top-k | "show me all my notes on a topic" e "how many notes do I have?" "give partial results" ([romebot, fórum 89221](https://forum.obsidian.md/t/every-attempt-at-pkm-has-landed-me-in-the-same-place-a-huge-mess/89221)). |
| P1.5 | **Espiral de configuração.** | Recorrente: 3 sites, 4 discussões | "Workflow Optimization Spiral" ([tech_ken, HN](https://news.ycombinator.com/item?id=47757229)); "When I reduced to the absolute basics, I just managed to write and retrieve notes consistently" ([freehorse, HN](https://news.ycombinator.com/item?id=47759280)); "knowledge management porn" (mxuribe, [Tildes](https://tildes.net/~tech/1oth/i_deleted_my_second_brain)); "Reduce your plugins and deactivate all of them" ([Jopp, fórum 89221](https://forum.obsidian.md/t/every-attempt-at-pkm-has-landed-me-in-the-same-place-a-huge-mess/89221)). |
| P1.6 | **Sincronização corrompe ou duplica notas.** | Recorrente: 2 sites, 5 discussões | Obsidian Sync que "makes an absolute mess of our shopping list" ([philips, HN](https://news.ycombinator.com/item?id=47758885)); troca silenciosa do conteúdo de uma nota pelo de outra ([hadik, HN](https://news.ycombinator.com/item?id=47759111)); plugin de git "will overwrite changes from other devices" ([echelon, HN](https://news.ycombinator.com/item?id=47757569)); tópico de 105 posts sobre seções duplicadas ([fórum 94732](https://forum.obsidian.md/t/94732)); dois sincronizadores no mesmo cofre criam "ping-pong" com "duplicated notes and silently reverted edits" ([inventdev, fórum 117819](https://forum.obsidian.md/t/117819)). |
| P1.7 | **Pessoal e trabalho misturados.** | Recorrente: 1 site, 2 discussões | "I can't sync my sensitive work notes" ([on_the_train, HN](https://news.ycombinator.com/item?id=47757377)); "First-class multi-vault support" ([echelon, HN](https://news.ycombinator.com/item?id=47757569)); na memória de tela: "How are you planing to segregate between professional use and personal use" ([subhajeet2107, HN](https://news.ycombinator.com/item?id=49024761)); empresas que bloqueiam gravadores de tela por política ([russelmelroy, HN](https://news.ycombinator.com/item?id=49033472)). |
| P1.8 | **Memória de tela amplia o estrago de um agente.** | 1 discussão, 3 autores | "turns 'give the agent access to my second brain' into a much bigger blast radius" ([rahulladumor, HN](https://news.ycombinator.com/item?id=49031437)); automação do screenpipe que "started sending my local api keys to an endpoint" ([basketbla, HN](https://news.ycombinator.com/item?id=49027906)); "How do you secure the database on my device?" ([Fraaaank, HN](https://news.ycombinator.com/item?id=49026624)). |
| P1.9 | **"Escrever é o que vale; quase nunca releio."** | Recorrente: 3 sites | "the act of writing is the useful part" (xavdid, [Tildes](https://tildes.net/~tech/1oth/i_deleted_my_second_brain)); "The writing is the important bit, not the resulting output" ([alabhyajindal, HN](https://news.ycombinator.com/item?id=44403917)); "There was value in writing it down at the time" ([rigmarole, fórum 89221](https://forum.obsidian.md/t/every-attempt-at-pkm-has-landed-me-in-the-same-place-a-huge-mess/89221)). |
| P1.10 | **Apagar tudo alivia, mas muitos se arrependem; a maioria prefere arquivar.** | Recorrente numa discussão; opinião dividida | "keep all notes, archived" ([barrnell3, HN](https://news.ycombinator.com/item?id=44403363)); "One of the few decisions I absolutely regret… was to throw away my old notebook" ([sedatk, HN](https://news.ycombinator.com/item?id=44406901)); do outro lado, "the most freeing thing I did" ([blindriver, HN](https://news.ycombinator.com/item?id=44404902)). |
| P1.11 | **Gravação contínua esquenta o computador.** | Um relato | screenpipe num M1 "made my CPU melt" e foi desligado ([kerv, HN](https://news.ycombinator.com/item?id=49031269)); o fundador diz que hoje mede em notebook de US$ 200 [autodeclarado]. |

### 2.3 Soluções relatadas

| # | Solução | Recorrência | Evidência |
| --- | --- | --- | --- |
| P1.S1 | **Uma entrada cronológica única (nota diária ou caixa de entrada), sem processar.** | Recorrente: 3 sites, 4 discussões | notas rápidas "are well suited for daily notes, which i barely ever 'process'" ([PaulHof, fórum 89221](https://forum.obsidian.md/t/every-attempt-at-pkm-has-landed-me-in-the-same-place-a-huge-mess/89221)); atalho global que joga tudo na nota do dia ([growthpilgrim, idem](https://forum.obsidian.md/t/every-attempt-at-pkm-has-landed-me-in-the-same-place-a-huge-mess/89221)); "Just do it chronologically. It's fine." ([bccdee, HN](https://news.ycombinator.com/item?id=47778549)); "inbox folder… Then daily notes. That's it." ([ChadMoran, HN](https://news.ycombinator.com/item?id=47759154)); uma nota única com o novo no topo (first-must-burn, [Tildes](https://tildes.net/~tech/1oth/i_deleted_my_second_brain)). |
| P1.S2 | **Estrutura nasce do uso, não da captura.** | Recorrente: 2 sites | marcar e ligar notas no momento de usar, não no de guardar ([ton, fórum 89221](https://forum.obsidian.md/t/every-attempt-at-pkm-has-landed-me-in-the-same-place-a-huge-mess/89221)); "structure is earned" ([growthpilgrim, idem](https://forum.obsidian.md/t/every-attempt-at-pkm-has-landed-me-in-the-same-place-a-huge-mess/89221)); "You can build a structure in your notes later" ([nout, HN](https://news.ycombinator.com/item?id=47757843)). |
| P1.S3 | **Agente de código (Claude Code, Codex) no cofre como memória.** | Recorrente: 2 sites, 3 discussões | "That's the killer feature of Obsidian imo" ([harmoni-pet, HN](https://news.ycombinator.com/item?id=47760202)); um MCP "with combined FTS and vector based semantic search using sqlite-vec" ([bitexploder, HN](https://news.ycombinator.com/item?id=47760229)); "low-friction way to have it remember things about me" ([tjarjoura, HN](https://news.ycombinator.com/item?id=47771955)). Ver também os números de instalação na seção 7.4. |
| P1.S4 | **Separar o registro bruto da memória derivada, e apagar pela linhagem.** | 1 discussão, 2 autores | o registro de captura é "chronological evidence"; a memória são "derived, revisable claims" que apontam para os trechos de origem e dizem se estão "current, disputed, or superseded"; apagar um intervalo "should invalidate embeddings, entity records, summaries" ([makeyouragent, HN](https://news.ycombinator.com/item?id=49093110)); "treating deletion as part of the data model" ([AmazingTurtle, HN](https://news.ycombinator.com/item?id=49026609)). |
| P1.S5 | **Trazer a memória antes de pedir, e mostrar de onde veio.** | Opinião de um autor | o acesso por MCP é "pull-based"; o que funciona "push relevant history… and show their work" ([jpardilla, HN](https://news.ycombinator.com/item?id=49039177)). |
| P1.S6 | **Captura sem imagem, só texto de acessibilidade.** | Afirmação do fornecedor | o screenpipe permite desligar as capturas de tela "and only capture accessibility data, it still captures most of the meaningful data" ([fundador, HN](https://news.ycombinator.com/item?id=49035249)) [autodeclarado]. |
| P1.S7 | **Agenda de captura e filtros por app e site.** | Afirmação do fornecedor | "define a recording schedule, for example stop recording on weekend" ([fundador, HN](https://news.ycombinator.com/item?id=49024847)) [autodeclarado]. |
| P1.S8 | **Git como rede de segurança, agente lendo uma cópia.** | 1 discussão | "push the vault to a private git remote and have Claude's side pull from a plain clone" ([ki-produktiv-de, fórum 117819](https://forum.obsidian.md/t/117819)). |

### 2.4 O que o plano ainda não cobre

1. Escrita do Aura num cofre sincronizado (P1.6): o plano trata o Cofre como pasta local e avisa
   sobre sincronização, mas não desenha como escrever para não gerar conflito (seção 7.3).
2. Separar trabalho e vida pessoal, e agenda de captura (P1.7, P1.S7).
3. Escopo por consulta para o agente e redação de segredos (chaves de API, tokens), não só de
   dados pessoais (P1.8).
4. Perguntas de contagem e listagem ("quantas reuniões com a Ana em outubro?"), que busca
   top-k não responde (P1.4).
5. Arrependimento de apagar (P1.10): o teto de disco apaga sozinho.
6. Transcrição acumulada sem destilação (P1.2): o plano destila na Consolidação, mas só na Fase
   3; até lá, Reuniões viram trechos brutos.

---

## 3. P3 — Embeddings em português

### 3.1 Benchmark nativo: MTEB-BR

O [MTEB-BR](https://arxiv.org/abs/2607.04581) (Tardelli Stekel, 06/07/2026) reúne 22 tarefas
criadas em português do Brasil, sem tradução, e avalia 93 modelos. Achados do artigo: o
ranking multilíngue global prevê o ranking em português só moderadamente (Spearman 0,75 em 55
modelos; um modelo é 3º lá e 49º aqui), e um modelo aberto alcança o grupo de topo. Resultados
brutos publicados em [mteb-pt/mteb-pt-results](https://huggingface.co/datasets/mteb-pt/mteb-pt-results)
(CC-BY 4.0). O artigo fixa 512 tokens de contexto para todos, o que penaliza modelos de
contexto longo; com 2.048 tokens o bge-m3 ganhou quase um quarto em BR-TaxQA-R (mesmo artigo).

Tabela abaixo: **média calculada por mim** sobre as seis tarefas de recuperação (Quati,
FaQuAD-IR, FAQ Bacen, MedPT, JurisTCU, BR-TaxQA-R), a partir dos arquivos JSON de cada modelo.
Conferi contra a tabela do artigo para o EmbeddingGemma (mesmos valores por tarefa).

| Modelo | Parâm. totais | Licença | nDCG@10 médio | recall@10 médio | Quati nDCG@10 |
| --- | ---: | --- | ---: | ---: | ---: |
| embeddinggemma-300m | 308 M | **Gemma (não permissiva)** | **0,653** | **0,656** | 0,607 |
| jina-embeddings-v5-text-small | 596 M | **CC-BY-NC-4.0** | 0,640 | 0,646 | 0,594 |
| bge-m3 | 568 M | MIT | 0,635 | 0,644 | 0,594 |
| granite-embedding-311m-multilingual-r2 | 312 M | Apache-2.0 | 0,607 | 0,622 | 0,572 |
| Qwen3-Embedding-0.6B | 596 M | Apache-2.0 | 0,604 | 0,618 | 0,562 |
| harrier-oss-v1-270m | 268 M | MIT | 0,571 | 0,588 | 0,547 |
| **granite-embedding-97m-multilingual-r2** | **97 M** | **Apache-2.0** | **0,557** | **0,581** | 0,495 |
| multilingual-e5-base | 278 M | MIT | 0,550 | 0,563 | 0,553 |
| F2LLM-v2-160M | 159 M | não informada | 0,531 | 0,546 | 0,476 |
| serafim-100m-portuguese-ir | 109 M | MIT | 0,519 | 0,544 | 0,450 |
| granite-embedding-107m-multilingual | 107 M | Apache-2.0 | 0,510 | 0,532 | 0,483 |
| **multilingual-e5-small** (linha de base do plano) | 118 M | MIT | **0,507** | **0,525** | 0,493 |
| F2LLM-v2-80M | 80 M | não informada | 0,498 | 0,515 | 0,434 |
| paraphrase-multilingual-MiniLM-L12-v2 | 118 M | Apache-2.0 | 0,023 | 0,038 | 0,014 |

O resultado do MiniLM (quase zero) parece falha de execução no benchmark, não desempenho real;
não usar como evidência. **Bekko a8m/a25m não está entre os 93 modelos avaliados** (lançados em
julho de 2026); desempenho em português: não encontrado.

### 3.2 Segundo benchmark: MTEB-PT

O [MTEB-PT](https://arxiv.org/abs/2607.04071) (Okamura, Alcoforado e Costa, 05/07/2026) usa 14
conjuntos existentes do MMTEB e avalia 17 modelos. Coluna de recuperação: Qwen3-Embedding-0.6B
78,9; gte-multilingual-base 76,9; multilingual-e5-large 74,5; multilingual-e5-base 71,7;
multilingual-e5-small 70,2. Conclusão dos autores: desempenho em português depende da tarefa,
"multilingual rankings do not reliably predict Portuguese-specific performance", e nenhum
modelo domina todas as famílias. Não inclui granite, bge-m3, EmbeddingGemma nem Bekko.

### 3.3 Licença e ONNX

Dados da API do Hugging Face, consultada em 12/11/2026.

| Modelo | Licença | Dim. | Contexto | ONNX quantizado publicado |
| --- | --- | ---: | ---: | --- |
| [granite-embedding-97m-multilingual-r2](https://huggingface.co/ibm-granite/granite-embedding-97m-multilingual-r2) | Apache-2.0 | 384 | 8.192 | Sim, oficial: `model_quint8_avx2.onnx`, 98 MB |
| [granite-embedding-311m-multilingual-r2](https://huggingface.co/ibm-granite/granite-embedding-311m-multilingual-r2) | Apache-2.0 | 768 | 8.192 | Sim, oficial: 313 MB |
| [multilingual-e5-small](https://huggingface.co/intfloat/multilingual-e5-small) | MIT | 384 | 512 | Sim, oficial: `model_qint8_avx512_vnni.onnx`, 118 MB |
| [bekko-embedding-v1-a8m](https://huggingface.co/hotchpotch/bekko-embedding-v1-a8m) | MIT | 384 | 8k [autodeclarado] | Sim, oficial: `model.onnx` 130 MB (padrão), int8 106 MB |
| [bekko-embedding-v1-a25m](https://huggingface.co/hotchpotch/bekko-embedding-v1-a25m) | MIT | 384 | 8k [autodeclarado] | Sim, oficial: 199 MB (padrão), int8 124 MB |
| [bge-m3](https://huggingface.co/BAAI/bge-m3) | MIT | 1.024 | 8.192 | Só fp32 no repositório oficial |
| [Qwen3-Embedding-0.6B](https://huggingface.co/Qwen/Qwen3-Embedding-0.6B) | Apache-2.0 | 1.024 | 32k | Não no repositório oficial |
| [embeddinggemma-300m](https://huggingface.co/google/embeddinggemma-300m) | Gemma | 768 | 2.048 | Conversão da comunidade ([onnx-community](https://huggingface.co/onnx-community/embeddinggemma-300m-ONNX), mesma licença) |
| [harrier-oss-v1-270m](https://huggingface.co/microsoft/harrier-oss-v1-270m) | MIT | 640 | 32k | [não verificado] |

**Os termos do Gemma não são licença permissiva** no sentido do `AGENTS.md`. Pelo texto
([Gemma Terms of Use](https://ai.google.dev/gemma/terms), modificado em 01/04/2026): quem
redistribui precisa incluir as restrições de uso como cláusula exigível no próprio contrato,
entregar cópia dos termos e um arquivo de aviso (seção 3.1); o uso fica sujeito à Prohibited Use
Policy, incorporada por referência (seção 3.2); e "Google reserves the right to restrict
(remotely or otherwise) usage". Usar o EmbeddingGemma exigiria decisão jurídica própria.

### 3.4 Ressalvas sobre o Bekko

- "a8m" é parâmetro **ativo**: 7,7 M sem a tabela de embeddings. O total é 106 M (a8m) e 123 M
  (a25m), segundo os metadados do repositório; o arquivo ONNX padrão tem 130 MB e 199 MB. O
  ganho de tamanho em disco e RAM contra o e5-small (118 M) é pequeno; o ganho de velocidade
  vem do menor número de camadas ativas [autodeclarado: 364 documentos/s num Ryzen 9 7950X com
  OpenVINO].
- Os 56,2 de recuperação MMTEB são média de 18 tarefas, várias em inglês. Na tabela por tarefa
  do próprio [cartão do modelo](https://huggingface.co/hotchpotch/bekko-embedding-v1-a8m), nas
  tarefas multilíngues de pergunta e resposta o a8m fica abaixo do gte-multilingual-base e do
  bge-m3: Belebele 69,7 contra 89,2 e 78,2; MLQA 67,5 contra 72,2 e 74,8; MIRACL com negativos
  difíceis 53,5 contra 64,2 e 69,6 [autodeclarado]. A vantagem média vem de tarefas como
  LEMBPasskey e WinoGrande.

### 3.5 Teste local: não rodou

Não rodou nesta sessão: a VPS tinha ~1,1 GiB de RAM livre, compartilhada com produção, e o
usuário preferiu rodar no computador pessoal. Protocolo sugerido, para o spike S1:

1. Conjunto: 50 documentos curtos em português (trechos de reunião, telas, notas, páginas lidas)
   e 30 perguntas escritas por outra pessoa, cada uma apontando o documento certo.
2. Modelos: multilingual-e5-small (base), granite-embedding-97m-multilingual-r2, Bekko a8m e
   a25m; opcional granite-311m-r2. Todos em ONNX int8, CPU, 1 thread.
3. Medir: acerto nos 10 primeiros, posição média, latência por documento e por consulta, pico de
   RAM do processo, tamanho do arquivo. Repetir com FTS5 + vetor + fusão (RRF) para ver quanto a
   busca híbrida compensa o modelo menor.

---

## 4. P2 — Memória de tela: como as ferramentas reais fazem

### 4.1 screenpipe

**Licença atual.** Desde 10/06/2026 o repositório inteiro está sob a "Screenpipe Commercial
License" ([LICENSE.md](https://github.com/screenpipe/screenpipe/blob/main/LICENSE.md),
[histórico](https://github.com/screenpipe/screenpipe/commits/main/LICENSE.md)). Uso livre só
pessoal não comercial, sem fins lucrativos, educacional ou pesquisa, e avaliação de 7 dias; uso
comercial, inclusive de código modificado, exige plano Enterprise; é proibido "use the Licensed
Work to build, offer, or operate a competing product". Versões publicadas antes sob MIT
continuam MIT. O fundador reconheceu a troca de MIT para source-available no
[Launch HN](https://news.ycombinator.com/item?id=49028293). **Confirma a decisão do plano: não
usar código.** Abaixo, só o desenho, lido no código e na documentação em 12/11/2026.

**Gatilhos.** A especificação de captura por evento
([EVENT_DRIVEN_CAPTURE_SPEC.md](https://github.com/screenpipe/screenpipe/blob/main/docs/EVENT_DRIVEN_CAPTURE_SPEC.md),
02/2026, marcada como histórica) define: troca de app e de foco de janela (assentar 300 ms),
clique (200 ms), pausa de digitação (500 ms), fim de rolagem (400 ms), cópia (200 ms), intervalo
mínimo de 200 ms por monitor e captura "de reserva" se nada acontecer. O código atual
([event_driven_capture.rs](https://github.com/screenpipe/screenpipe/blob/main/crates/screenpipe-engine/src/event_driven_capture.rs))
mudou os padrões: reserva a cada **30 s** (não 5 s), verificação visual a cada 3 s que dispara
captura com ~5% de diferença de pixels, qualidade JPEG 80, e um piso de **1,5 s** para gatilhos
de teclado e área de transferência, criado porque digitação contínua gerava "a back-to-back
capture storm" (issue #4844 citada no código). Em reunião, a reserva cai para 5 s.

**Deduplicação.** Por hash do texto de acessibilidade: se o texto não mudou, não grava. Três
exceções no código: gatilhos "duros" (troca de app, foco, fim de rolagem, reserva, manual)
sempre gravam; durante reunião, mudança visual grava mesmo com o mesmo texto, porque slides e
tela compartilhada mudam a imagem sem mudar a árvore de acessibilidade; e passados 30 s sem
gravar, grava de qualquer jeito.

**Texto.** Acessibilidade primeiro, OCR só se vier vazio; tempo-limite da árvore de 200 ms no
macOS e 350 ms no Windows, porque "UIA tree walk is slower on Windows (200-500ms vs 10-50ms on
macOS)" (especificação) [autodeclarado, sem método]. OCR no Windows: motor nativo do Windows,
com Tesseract e nuvem como alternativas
([screenpipe-screen/src/core.rs](https://github.com/screenpipe/screenpipe/blob/main/crates/screenpipe-screen/src/core.rs)).

**Disco.** Cada captura vira um JPEG em `~/.screenpipe/data/AAAA-MM-DD/`. Um trabalho de fundo
([snapshot_compaction.rs](https://github.com/screenpipe/screenpipe/blob/main/crates/screenpipe-engine/src/snapshot_compaction.rs))
junta JPEGs com mais de 10 minutos em MP4 de 25 a 100 quadros (HEVC por hardware ou H.264 por
software), alegando "10-30x compression on mostly-static screen content" [autodeclarado]; o
comentário do mesmo arquivo estima "~400KB each" por JPEG, contra ~80 KB na especificação de
fevereiro [autodeclarado, divergente]. Retenção local opcional, padrão de 14 dias
([retention.rs](https://github.com/screenpipe/screenpipe/blob/main/crates/screenpipe-engine/src/retention.rs)).

**Banco.** SQLite com FTS5 em quase tudo (`frames_fts`, `accessibility_fts`, `ocr_text_fts`,
`audio_transcriptions_fts`, `memories_fts`, entre outras), ~120 arquivos de migração, e
`sqlite-vec = "0.1.3"` carregado como extensão automática
([screenpipe-db/Cargo.toml](https://github.com/screenpipe/screenpipe/blob/main/crates/screenpipe-db/Cargo.toml)).
Embeddings de OCR guardados como BLOB numa tabela própria.

**Dados sensíveis e cifra.** Redação de dados pessoais com expressões regulares e, opcionalmente,
um modelo ONNX local (DirectML no Windows) e um detector de PII em imagem
([screenpipe-redact](https://github.com/screenpipe/screenpipe/blob/main/crates/screenpipe-redact/src/lib.rs)).
A cifra em repouso é um "cofre" que cifra banco, capturas e áudio **quando o usuário tranca**
(Argon2id + ChaCha20-Poly1305 por arquivo,
[screenpipe-vault](https://github.com/screenpipe/screenpipe/blob/main/crates/screenpipe-vault/src/lib.rs));
enquanto grava, os dados ficam em claro. No HN, o fundador descreveu "early support for
encryption at rest through our CLI" ([comentário](https://news.ycombinator.com/item?id=49026790)).

**Consumo.** README atual: CPU "approximately 5-20%", RAM "approximately 0.5-3 GB", e o disco
"varies… Measure a representative day on your device"
([README](https://github.com/screenpipe/screenpipe/blob/main/README.md)) [autodeclarado,
"planning estimates"]. Os 5–30 GB/mês citados na pesquisa anterior **não estão mais no README**.
Metas da especificação: tela parada < 0,5% de CPU, uso ativo < 5%, ~300 MB por dia de 8 h,
3.000–5.000 capturas por dia [autodeclarado, metas de projeto, não medições publicadas].

### 4.2 Microsoft Recall

- **Frequência e regra de mudança, confirmadas:** "a snapshot of your active screen will be
  saved every few seconds and when the content of your active window changes"
  ([Microsoft Support](https://support.microsoft.com/en-us/windows/retrace-your-steps-with-recall-aa03f8a0-a78b-4b3e-b0a1-2eb8ac48701c)).
- **Disco:** precisa de 50 GB livres para ligar; pausa abaixo de 25 GB livres (mesma página);
  teto ajustável, o mais antigo sai primeiro
  ([privacidade e controle](https://support.microsoft.com/en-us/windows/privacy-and-control-over-your-recall-experience-d404f672-7647-41e5-886c-a3c59680af15)).
- **Idiomas:** otimizado para inglês, chinês simplificado, francês, alemão, japonês e espanhol
  (página de suporte). **Português não está na lista.**
- **Filtros:** apps e sites escolhidos pelo usuário; navegação privada no Edge, Firefox, Opera,
  Chrome e Chromium 124+; conteúdo com DRM; clientes de área de trabalho remota suportados;
  "Sensitive information filtering", ligado por padrão, para senhas, documentos de identidade e
  cartões, com as bibliotecas do Microsoft Purview (página de privacidade e
  [blog de arquitetura](https://blogs.windows.com/windowsexperience/2024/09/27/update-on-recall-security-and-privacy-architecture/)).
  Partes de sites filtrados ainda podem aparecer (conteúdo embutido, abas abertas). O mecanismo
  de detecção de janela anônima não é descrito: não encontrado.
- **Cifra (fornecedor):** capturas e índice vetorial "always encrypted", chaves por captura
  dentro de um enclave VBS, presas ao TPM e ao Windows Hello ESS; consultas rodam no enclave
  (blog de arquitetura, 27/09/2024).
- **Análise independente:** [TotalRecall Reloaded](https://github.com/xaitax/TotalRecall)
  (Alexander Hagenah, 08/04/2026) confirma a criptografia ("per-page AES-256-GCM… The
  cryptography is genuinely solid"), mas mostra que o processo que desenha a linha do tempo
  (`AIXHost.exe`) não tem proteção: um processo comum do mesmo usuário injeta código, espera o
  usuário passar pelo Windows Hello e extrai capturas e texto; também achou uma interface COM
  (`IResponse4.get_UnfilledItems`) sem a checagem de acesso que as outras têm. "No admin
  required. Standard user."

### 4.3 OpenRecall

AGPL-3.0, último push em 24/09/2025 ([repositório](https://github.com/openrecall/openrecall)).
A cada 3 s captura os monitores configurados e compara cada um com a captura anterior por MSSIM; se a
similaridade for ≥ 0,9, descarta. Salva **WebP sem perdas**, roda OCR (docTR) e calcula o
embedding como média dos embeddings de cada linha (SentenceTransformer), guardado como BLOB no
SQLite ([screenshot.py](https://github.com/openrecall/openrecall/blob/main/openrecall/screenshot.py),
[database.py](https://github.com/openrecall/openrecall/blob/main/openrecall/database.py)).
Números de disco: não encontrados no repositório.

### 4.4 Windrecorder

GPL-2.0, ativo (push em 25/09/2026) ([repositório](https://github.com/yuka-friends/Windrecorder)).
Grava vídeo com ffmpeg em segmentos de 15 minutos e indexa depois; no modo de capturas, tira
uma a cada 3 s. Descarta se a imagem tiver similaridade ≥ 0,7 com a anterior ou se o OCR tiver
≥ 0,7 de coincidência, e evita repetir conteúdo já presente na tabela com ≥ 0,94
([config_default.json](https://github.com/yuka-friends/Windrecorder/blob/main/windrecorder/config_src/config_default.json)).
Pausa com tela parada, janela excluída ou tela bloqueada. Busca por OCR (Windows.Media.Ocr por
padrão) e, como extensão, por embedding de imagem com `uform3-image-text-multilingual-base`
(256 dimensões) e FAISS
([img_embed_manager.py](https://github.com/yuka-friends/Windrecorder/blob/main/windrecorder/img_embed_manager.py)).
Consumo [autodeclarado, README]: vídeo de 2 a 100 MB por hora, 10 a 20 GB por mês, compressível
para 0,1–0,7× com predefinições; banco SQLite de ~160 MB por mês.

### 4.5 Números lado a lado

| Ferramenta | Gatilho | Repetição descartada por | Imagem | Disco | Fonte e método |
| --- | --- | --- | --- | --- | --- |
| screenpipe | Evento + verificação visual a cada 3 s + reserva a cada 30 s | Hash do texto de acessibilidade, com exceções | JPEG q80, depois MP4 | Meta ~300 MB/dia de 8 h | Código e especificação [autodeclarado] |
| Recall | "A cada poucos segundos" e quando a janela ativa muda | Não documentado | Não documentado | Teto ajustável; exige 50 GB livres | Suporte Microsoft |
| OpenRecall | A cada 3 s | MSSIM ≥ 0,9 | WebP sem perdas | Não encontrado | Código |
| Windrecorder | Vídeo contínuo + índice a cada 3 s | Imagem ≥ 0,7, OCR ≥ 0,7, tabela ≥ 0,94 | Vídeo ffmpeg | 10–20 GB/mês de vídeo + 160 MB/mês de banco | README [autodeclarado] |
| Plano do Aura | Evento + ~1,5 s + batida 30–60 s | Impressão digital do texto + hash perceptual | Janela ativa ≤ 1.280 px | 100–150 MB/mês de texto; ~1,4 GB/mês de imagem | Estimativa do plano, não medida |

---

## 5. P4 — Armazenamento

### 5.1 sqlite-vec

- **Licença:** dupla, MIT ou Apache-2.0 (arquivos `LICENSE-MIT` e `LICENSE-APACHE` no
  [repositório](https://github.com/asg017/sqlite-vec)). Compatível com o `deny.toml`.
- **Manutenção:** pré-1.0 ("expect breaking changes!", README). Última versão estável v0.1.9 em
  31/03/2026; última pré-versão v0.1.10-alpha.4 e último commit em 18/05/2026 — quase seis
  meses sem commits até hoje, com 250 issues abertas
  ([releases](https://github.com/asg017/sqlite-vec/releases)). O código já traz índices
  aproximados (IVF, DiskANN, "rescore") ainda em pré-versão.
- **Rust:** crate `sqlite-vec` com 4,0 milhões de downloads
  ([crates.io](https://crates.io/crates/sqlite-vec)); o screenpipe usa em produção com
  rusqlite/sqlx (seção 4.1).
- **Com SQLCipher:** funciona. Na [issue #12](https://github.com/asg017/sqlite-vec/issues/12)
  (2024), `rusqlite` com `bundled-sqlcipher` dava "SQL logic error" porque o SQLite embutido no
  SQLCipher era mais antigo (3.39.4) e não aceitava `MATCH ? … LIMIT 10`; resolvido com a
  restrição `AND k = 10`. Hoje o `libsqlite3-sys` 0.40 (11/11/2026) empacota SQLCipher 4.20.0
  (`sqlcipher-src = "=420.0.0"`), com opção de OpenSSL embutido
  (`bundled-sqlcipher-vendored-openssl`)
  ([Cargo.toml](https://github.com/rusqlite/rusqlite/blob/master/libsqlite3-sys/Cargo.toml));
  SQLCipher é BSD-3-Clause ([repositório](https://github.com/sqlcipher/sqlcipher)). Compilar o
  OpenSSL embutido no MSVC: [não verificado].
- **Quem combina os três:** a busca de código do GitHub acha 29 `Cargo.toml` com
  `bundled-sqlcipher` e `sqlite-vec` juntos, por exemplo [brain0](https://github.com/Brain0-ai/brain0)
  (Apache-2.0) e [mnema](https://github.com/shaik-zeeshan/mnema) (memória de tela para Mac,
  AGPL). São projetos pequenos; nenhum publica medição.

### 5.2 Força bruta com 100 mil a 1 milhão de vetores

| Medição | Hardware | Resultado | Fonte |
| --- | --- | --- | --- |
| sqlite-vec 0.1.6, 100 mil × 384 dim., k=10 | i9-13900K | float32 38,7 ms; **int8 39,1 ms**; bit 1,96 ms. Disco: 151,8 / 41,4 / 9,3 MB | [SimpleVecDB](https://coderdayton.github.io/SimpleVecDB/benchmarks) [autodeclarado] |
| sqlite-vec, varredura exata, 1 M × 512 dim. | 32 núcleos, 64 GB | 395 ms por consulta (2,5 consultas/s); 1 M × 128: 10 consultas/s | [horosvec-bench, fórum Go](https://forum.golangbridge.org/t/42173) [autodeclarado] |

Extrapolação linear minha, **não medida**: 500 mil × 384 dimensões no mesmo i9 daria ~195 ms
para float32 ou int8 e ~10 ms para binário. Num notebook comum, o tempo de float/int8 tende a
passar da meta de 300 ms. Resultados publicados de int8 + reordenação no sqlite-vec com
500 mil a 1 milhão de itens: não encontrados (procurei no repositório, que tem scripts mas não
resultados, e em buscas na web).

### 5.3 Alternativas embutidas com licença permissiva

| Opção | Licença | Limitação principal |
| --- | --- | --- |
| [LanceDB](https://github.com/lancedb/lancedb) | Apache-2.0 | Segundo formato de armazenamento ao lado do SQLite; fora da cifra do SQLCipher |
| [USearch](https://github.com/unum-cloud/usearch) | Apache-2.0 | Núcleo C++ com binding Rust; índice HNSW em arquivo próprio, fora do SQLCipher |
| [hnsw_rs](https://github.com/jean-pierreBoth/hnswlib-rs) | MIT ou Apache-2.0 | Índice só em memória/arquivo próprio |
| [arroy](https://github.com/meilisearch/arroy) | MIT | Exige LMDB |
| [sqlite-vector](https://github.com/sqliteai/sqlite-vector) | Apache-2.0 (arquivo de licença alterado em 2025 e 2026) | Extensão concorrente do sqlite-vec; compatibilidade com SQLCipher [não verificado] |
| [libSQL](https://github.com/tursodatabase/libsql) | MIT | Busca vetorial nativa, mas é um fork do SQLite: não combina com SQLCipher |
| BLOB no SQLite + varredura em Rust | — | Sem extensão; trivial de escrever; mesma complexidade da força bruta do sqlite-vec |

---

## 6. P5 — Imagens

### 6.1 Embedding visual de vetor único

| Modelo | Licença | Tamanho | ONNX |
| --- | --- | --- | --- |
| [SigLIP 2 base patch16-224](https://huggingface.co/google/siglip2-base-patch16-224) | Apache-2.0 | 375 M parâm. | Conversão da comunidade, int8 378 MB ([onnx-community](https://huggingface.co/onnx-community/siglip2-base-patch16-224-ONNX), licença não declarada no repositório) |
| [SigLIP base patch16-224](https://huggingface.co/google/siglip-base-patch16-224) | Apache-2.0 | 203 M | Não oficial |
| [CLIP ViT-B/32](https://huggingface.co/Xenova/clip-vit-base-patch32) | MIT | — | Sim, int8 154 MB |
| [uform3 image-text multilingual base](https://huggingface.co/unum-cloud/uform3-image-text-multilingual-base) | Apache-2.0 | — | Sim: codificador de imagem 87 MB + texto 111 MB; usado pelo Windrecorder |
| [jina-clip-v2](https://huggingface.co/jinaai/jina-clip-v2) | CC-BY-NC-4.0 | 865 M | Fora: licença não comercial |

**Servem para "o gráfico azul"?** Evidência fraca. No [GUing](https://arxiv.org/abs/2405.00145),
o CLIP sem ajuste fino alcançou recall@10 acima de 0,2 em busca de telas de app por texto, e o
modelo ajustado com pares tela-legenda superou o CLIP em quase todas as métricas; os autores do
[UIClip](https://arxiv.org/abs/2404.12500) dizem que o CLIP "is not well-suited" para julgar
interfaces. Avaliação de SigLIP em capturas de tela: não encontrada. Conclusão provisória: um
codificador genérico deve achar cor e tipo de gráfico, mas não há medição; fica como spike da
Fase 4, com o uform3 (Apache-2.0, ONNX pequeno, multilíngue) como primeiro candidato.

### 6.2 Formato das imagens

- **Tamanho e custo:** no experimento do [GDELT](https://blog.gdeltproject.org/at-scale-ocr-of-television-news-experiments-optimizing-the-still-frame-file-storage-format/)
  com uma hora de TV em resolução SD a 1 quadro/s, os JPEGs somaram 97 MB, WebP 90–92 MB e AVIF
  64 MB, mas o AVIF levou 41 min 58 s contra 3 min 15 s do WebP num único núcleo, ~13× mais
  CPU. É vídeo de TV, não tela de computador.
- **Efeito no OCR:** estudo controlado comparando JPEG, WebP e AVIF em capturas de tela não
  encontrado (busquei artigos, blogs de compressão e o relatório NIST citado por terceiros, que
  não consegui abrir). Os guias concordam só em termos qualitativos: artefatos de JPEG se
  concentram em bordas de letras.
- **O que os projetos escolhem:** screenpipe usa JPEG q80 e compacta em vídeo (seção 4.1);
  OpenRecall usa WebP sem perdas; Windrecorder usa vídeo.

---

## 7. P6 e P7 — APIs do Windows e Obsidian

### 7.1 Windows.Graphics.Capture (build 26100)

- `GraphicsCaptureDirtyRegionMode` tem dois valores, `ReportOnly` (0) e `ReportAndRender` (1),
  **sem descrição** na referência; `GraphicsCaptureSession.MinUpdateInterval` é um `TimeSpan`
  sem observações; `Direct3D11CaptureFrame.DirtyRegions` devolve uma lista de `RectInt32`. As
  três se aplicam aos builds 26100 e 28000
  ([enum](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture.graphicscapturedirtyregionmode),
  [MinUpdateInterval](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture.graphicscapturesession.minupdateinterval),
  [DirtyRegions](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture.direct3d11captureframe.dirtyregions)).
  As páginas não estão no repositório público de fontes da documentação.
- Um desenvolvedor relatou que `MinUpdateInterval` abaixo de 1 ms, inclusive o padrão 0, limita
  a captura a ~50 quadros/s; o mantenedor do exemplo oficial não reproduziu
  ([issue #82](https://github.com/robmikh/Win32CaptureSample/issues/82)).
- O libwebrtc não usa essas APIs para decidir nada: calcula a própria "região danificada", em
  que vazia significa "no change in content since last frame"
  ([wgc_capture_session.cc](https://webrtc.googlesource.com/src/+/HEAD/modules/desktop_capture/win/wgc_capture_session.cc));
  um commit só registra se a propriedade existe ([commit](https://webrtc.googlesource.com/src/+/dfd8f570cd069d502c2ea923256ef835971e97b5))
  [não verificado: lido por resumo de busca].
- **O que acontece com quadros sem mudança:** não encontrado em documentação oficial.

### 7.2 SetWinEventHook e UI Automation

- Ganchos fora do processo são "noticeably slower than in-context hook functions due to
  marshaling" e os eventos chegam enfileirados
  ([out-of-context hook functions](https://learn.microsoft.com/en-us/windows/win32/winauto/out-of-context-hook-functions)).
  Medição publicada do custo de `EVENT_SYSTEM_FOREGROUND` ou `EVENT_OBJECT_NAMECHANGE`: não
  encontrada.
- **Ser cliente de UIA liga a acessibilidade dos navegadores.** Um relato com WPR/xperf num
  [issue do Claude Code](https://github.com/anthropics/claude-code/issues/44739) mostra que o
  Claude Desktop, ao carregar `UIAutomationCore.dll` e `OLEACC.dll`, fazia Chrome e Firefox
  manterem a árvore de acessibilidade completa; em páginas com DOM muito dinâmico, a fila de
  entrada do Chrome cresceu de 234 ms para 43.000 ms. Um relato, com rastreamento.
- **Chrome:** UIA nativa ligada por padrão a partir da versão 138
  ([Chrome for Developers](https://developer.chrome.com/blog/windows-uia-support-update)).
- **Electron:** "Electron applications will automatically enable accessibility features in the
  presence of assistive technology"; o app também pode ligar por
  `app.setAccessibilitySupportEnabled`, e as ferramentas assistivas do sistema têm prioridade
  ([documentação do Electron](https://github.com/electron/electron/blob/main/docs/tutorial/accessibility.md)).
  Ou seja, o Aura ver texto em apps Electron depende de o Aura parecer tecnologia assistiva, o
  que também liga o custo acima.
- **Custo da UIA no Windows:** "200-500ms vs 10-50ms on macOS" para percorrer a árvore, segundo
  a especificação do screenpipe [autodeclarado, sem método]. Medição independente: não
  encontrada.
- **`IsPassword`:** "indicates whether the automation element contains protected content or a
  password"; "Attempting to access the Value property of the protected element… may cause an
  error to occur" ([propriedades UIA](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-automation-element-propids)).
- **Janela anônima:** API pública de detecção não encontrada; o Recall filtra, mas não diz como
  (seção 4.2).
- **Apps que pedem para não ser capturados:** `SetWindowDisplayAffinity` com
  `WDA_EXCLUDEFROMCAPTURE` (Windows 10 2004+) exclui a janela de capturas
  ([documentação](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowdisplayaffinity)).
  O Signal liga isso por padrão no Windows 11 para se proteger do Recall
  ([blog do Signal](https://signal.org/blog/signal-doesnt-recall/)). A propriedade bloqueia
  pixels, não a árvore de UIA: um leitor de texto por UIA continuaria lendo a conversa.

### 7.3 Obsidian: CLI, Sync e escrita por agente

- **CLI oficial:** "Obsidian CLI requires the Obsidian app to be running. If Obsidian is not
  running, the first command you run launches Obsidian." Liga em Configurações → Geral
  ([ajuda](https://obsidian.md/help/cli)). Sintaxe real:
  - propriedades: `obsidian property:set name=<nome> value=<valor> type=text|list|number|checkbox|date|datetime path=<caminho>`,
    `property:read`, `property:remove`, `properties format=json counts`;
  - Bases: `obsidian bases`, `base:views`, `base:query path=<arquivo.base> view=<visão> format=json|csv|tsv|md|paths`,
    `base:create path=<arquivo.base> name=<nota> content=<texto>`;
  - busca: `search query=<texto>` e `search:context query=<texto>`.
- **Sem o app:** só o [Obsidian Headless](https://obsidian.md/help/headless) (beta aberto, Node
  22+, `ob login`), que sincroniza e publica sem o app, mas exige conta do Obsidian Sync. Não
  serve para editar propriedades.
- **Defeitos relatados com agentes:** `property:set` sai com código 0 e não muda nada num arquivo
  escrito por fora (instalador 1.12.7, Windows 11) ([fórum 113166](https://forum.obsidian.md/t/113166));
  `move` sai com 0 mesmo falhando (mesmo tópico); a CLI não imprime nada com stdin fechado, que é
  como agentes chamam comandos ([fórum 114998](https://forum.obsidian.md/t/114998)).
- **Obsidian Sync e conflitos** ([solução de problemas](https://obsidian.md/help/sync/troubleshoot)):
  - Markdown em conflito é mesclado com diff-match-patch; pode "create duplicate text". Desde a
    1.9.7 dá para escolher "Create conflict file", por aparelho.
  - **Notas criadas automaticamente na abertura (como notas diárias) podem sumir:** se a mesma
    nota for criada localmente e, em poucos minutos, chegar uma versão remota, "Sync will keep
    the remote version without merging".
  - Cofre dentro de `Desktop` ou `Documents` no Windows costuma estar no OneDrive; a recomendação
    é `D:\`, `C:\Vaults` ou `C:\Users\<nome>\Vaults`. O Windows Defender pode pôr em quarentena
    notas com blocos de código.
- **Propriedades** ([ajuda](https://obsidian.md/help/properties)): tipos texto, lista, número,
  caixa, data (`AAAA-MM-DD`), data e hora, e `tags`; "Once a property type is assigned to a
  property name, all properties with that name across your vault will use the same type";
  links em propriedades de lista vão entre aspas.
- **Bases:** arquivo `.base` em YAML com filtros como `file.hasTag("x")`, `file.inFolder("y")`,
  `file.hasLink("z")` e fórmulas ([sintaxe](https://obsidian.md/help/bases/syntax)).

### 7.4 O que o usuário realmente instala

Downloads acumulados por plugin, publicados pela Obsidian em
[community-plugin-stats.json](https://github.com/obsidianmd/obsidian-releases/blob/master/community-plugin-stats.json)
(atualizado em 12/11/2026). Conta downloads, não usuários ativos.

| Plugin | Downloads | O que é |
| --- | ---: | --- |
| Copilot | 3.469.124 | Hoje se descreve como "Run AI agents such as Claude Code, Codex, and OpenCode inside your vault" |
| Claudian | 2.696.822 | Claude Code e Codex embutidos no cofre |
| Omnisearch | 2.133.148 | Busca lexical, PDFs e OCR |
| Smart Connections | 1.268.667 | Notas relacionadas por embedding local |
| Local REST API with MCP | 850.579 | API e MCP para o cofre |
| Text Generator | 594.273 | Geração de texto |
| Agent Client | 332.006 | Claude Code, Codex e Gemini CLI pelo Agent Client Protocol |
| Karpathy LLM Wiki | 70.481 | Padrão LLM-wiki |
| MCP Connector | 62.857 | MCP com busca semântica |
| Khoj | 55.352 | Assistente com índice próprio |
| Nexus | 14.660 | Chat agêntico com memória semântica |

Leitura: o usuário comum instala **agente de código dentro do cofre** e **busca lexical**;
servidores de RAG com índice próprio (Khoj) ficam duas ordens de grandeza atrás.

---

## 8. P8 — Memória de conversas para agentes

- **Avaliação independente de Mem0** ([arXiv 2603.04814](https://arxiv.org/abs/2603.04814),
  05/03/2026): Mem0 com extração por GPT-5-nano e leitura por GPT-5-mini contra passar o
  histórico inteiro ao GPT-5-mini. Acurácia: LongMemEval 49,0% contra 82,4%; LoCoMo 57,7% contra
  92,9%; PersonaMem v2 62,5% contra 69,8%. A memória por fatos fica mais barata que contexto
  longo depois de ~10 turnos a 100 mil tokens de contexto. Os 93,4–94,4 que o Mem0 se atribui no
  LongMemEval são do fornecedor.
- **Base simples e forte** ([arXiv 2511.17208](https://arxiv.org/abs/2511.17208), 21/11/2025):
  quebrar cada sessão em proposições curtas e autocontidas, com entidades normalizadas e o turno
  de origem, sem comprimir, mais busca densa e filtro por LLM, "match or surpass strong
  baselines" em LoCoMo e LongMemEval.
- **Atualizar crença quase não ajuda nos benchmarks atuais** ([arXiv 2606.22030](https://arxiv.org/abs/2606.22030),
  20/06/2026): atualização bayesiana dá pouco ganho sobre "a última escrita vence" porque os
  benchmarks raramente têm contradição; com observações de confiabilidade diferente, ganha muito;
  e limitar a confiança pela **origem** do dado resiste a envenenamento da memória. O mesmo
  artigo mede 27,5 pontos de diferença entre F1 por token e juiz LLM nas mesmas respostas.
- **Zep/Graphiti e Letta:** resultados independentes não encontrados (procurei LongMemEval,
  LoCoMo e comparativos de 2025–2026; só achei números dos fornecedores).

Para um app local de uso pessoal, a evidência favorece: guardar o bruto e buscar nele; tratar
fatos extraídos como índice, nunca como substituto; fato com origem e data de validade; e
confiança limitada pela origem (o que o usuário disse vale mais do que o que passou na tela).

---

## 9. P9 — Legal e privacidade

> **Isto é pesquisa, não aconselhamento jurídico.** O spike S6 continua precisando de um
> advogado.

**Texto da LGPD** (Lei 13.709/2018, versão atualizada da
[Câmara dos Deputados](https://www2.camara.leg.br/legin/fed/lei/2018/lei-13709-14-agosto-2018-787077-normaatualizada-pl.html);
o site do Planalto recusou o acesso por excesso de requisições):

- Art. 4º, I: a lei "não se aplica ao tratamento de dados pessoais… realizado por pessoa natural
  para fins exclusivamente particulares e não econômicos".
- Art. 5º, X: tratamento é "toda operação realizada com dados pessoais, como as que se referem a
  coleta, produção, recepção, classificação, utilização, acesso, reprodução… arquivamento,
  armazenamento, eliminação…". Gravar, transcrever, guardar capturas e indexar entram.
- Art. 5º, II: dado sensível inclui saúde, vida sexual, religião, opinião política, biometria.
  Telas e reuniões podem conter isso.
- Art. 7º: bases legais, entre elas consentimento (I), execução de contrato a pedido do titular
  (V), exercício regular de direitos (VI) e legítimo interesse (IX).
- Art. 10: legítimo interesse só "para finalidades legítimas, consideradas a partir de situações
  concretas", respeitando "as legítimas expectativas" do titular.

**ANPD:** a Resolução CD/ANPD nº 2/2022 (agentes de pequeno porte) repete a exclusão: o
regulamento "não se aplica ao tratamento de dados pessoais realizado por pessoa natural para fins
exclusivamente particulares e não econômicos" (art. 1º, parágrafo único; lido na
[cópia publicada no gov.br](https://www.gov.br/palmares/pt-br/midias/arquivos-menu-acesso-a-informacao/legislacao/resolucao-cd-anpd-n2-2022.pdf)).
Orientação da ANPD sobre o que conta como uso "particular e não econômico", sobre gravação de
reuniões ou sobre memória de tela: não encontrada (procurei guias e notas técnicas da ANPD por
busca).

**Gravação por um dos participantes** (Lei 9.296/1996, art. 8º-A, § 4º, texto da
[Câmara](https://www2.camara.leg.br/legin/fed/lei/1996/lei-9296-24-julho-1996-349049-normaatualizada-pl.html)):
"A captação ambiental feita por um dos interlocutores sem o prévio conhecimento da autoridade
policial ou do Ministério Público poderá ser utilizada, em matéria de defesa, quando
demonstrada a integridade da gravação". Trata de prova em processo penal, não de proteção de
dados. O entendimento do STF de que é lícita a gravação feita por um dos interlocutores (Tema
237) aparece em textos jurídicos, mas não li a decisão [não verificado].

**Leitura para o Aura (hipótese a validar com advogado):** uso estritamente pessoal tende a cair
no art. 4º, I; "lembrar das conversas com clientes" é uso econômico, e aí o usuário vira
controlador e precisa de base legal (provavelmente legítimo interesse ou execução de contrato) e
de transparência com os participantes. Se a Consolidação mandar Lembranças a um provedor de
nuvem, esse provedor trata dados de terceiros a mando do usuário.

**Como os produtos avisam os participantes**

| Produto | Aviso | Fonte |
| --- | --- | --- |
| Recall | Nenhum aviso a terceiros documentado; os controles são todos de quem grava | Páginas de suporte (seção 4.2) |
| Granola | Sem robô; "Always get consent when transcribing others"; mensagem automática no chat e marca d'água no vídeo como opções; "It is not legal advice" | [Getting consent](https://docs.granola.ai/help-center/consent-security-privacy/getting-consent) |
| Fathom | E-mail de pedido de consentimento a convidados externos antes da reunião; se alguém recusa, desliga a gravação automática daquela reunião; no Zoom, só se o usuário for anfitrião | [Automatically Requesting Recording Consent](https://help.fathom.video/en/articles/294272) |
| Fireflies | No app de desktop: mensagem no chat, mensagem de áudio e marca d'água; se o usuário desligar, a responsabilidade é dele | [Consent & Notifications](https://guide.fireflies.ai/articles/3917896272-how-consent-notifications-work-on-the-fireflies-desktop-app) |
| Otter | Pede permissão ao anfitrião, não a todos; processado em ação coletiva (Brewer, 15/08/2025, N.D. Cal.) por gravar sem consentimento dos participantes e treinar IA com isso | [NPR](https://www.npr.org/2025/08/15/g-s1-83087/otter-ai-transcription-class-action-lawsuit) |

---

## 10. Impacto no plano

Não alterei `docs/project/plano-segundo-cerebro.md`. Mudanças propostas:

| # | Decisão ou trecho do plano | O que a fonte diz | Mudança proposta |
| --- | --- | --- | --- |
| I-1 | §9 S1: "multilingual-e5-small, Bekko a8m/a25m e mais um" | MTEB-BR: granite-97m-r2 (Apache-2.0, ONNX int8 oficial de 98 MB, 8.192 tokens) tem recuperação 0,557 contra 0,507 do e5-small; Bekko não foi avaliado em português; EmbeddingGemma lidera os pequenos mas tem licença Gemma (§3) | Candidatos do S1: e5-small (base), **granite-embedding-97m-multilingual-r2**, Bekko a8m/a25m; granite-311m-r2 como teto de qualidade. EmbeddingGemma fora, salvo parecer sobre os termos do Gemma. Registrar no S1 que o MTEB-BR fixa 512 tokens. |
| I-2 | §5.2: "Vetores quantizados em 8 bits… ~190 MB varridos por busca" e "se o p95 passar da meta, entra um primeiro passo binário" | No sqlite-vec, int8 tem o mesmo tempo de float32 (39 ms com 100 mil × 384 num i9); binário é ~20× mais rápido (§5.2) | Desenhar desde o início: primeiro passo binário, reordenação em int8 ou float dos ~100 melhores, filtro de período antes. O S2 mede num notebook modesto, não só no desktop. |
| I-3 | §5.2 e ADR 0012 proposto: "FTS5 e sqlite-vec" | sqlite-vec pré-1.0, sem commits desde 18/05/2026; combinação com SQLCipher funciona, com sintaxe `k = ?` se o SQLite do SQLCipher for antigo (§5.1) | O ADR 0012 registra a saída: vetores em BLOB no próprio `memory.db` e varredura em Rust se o sqlite-vec parar. O S2 compara as duas opções e verifica a compilação do SQLCipher com OpenSSL no MSVC. |
| I-4 | §5.2 e Q-012: cifra com SQLCipher + DPAPI, "foi a crítica central ao Recall" | O TotalRecall extraiu dados do Recall sem quebrar a cifra, injetando código como o mesmo usuário (§4.2). DPAPI do usuário também é acessível a qualquer processo dele | O ADR 0012 e a tela de ativação dizem o que a cifra cobre (disco roubado, outra conta, backup vazado) e o que não cobre (malware rodando como o usuário). Avaliar trancar o Acervo após inatividade e pedir Windows Hello para abrir a página Memória. |
| I-5 | §4.2: "UI Automation devolve o texto exato da janela… É barato" | Ser cliente de UIA liga a árvore completa do Chrome e do Firefox; um relato mediu 43 s de entrada travada (§7.2). Electron só expõe texto quando detecta tecnologia assistiva | O S3 mede também o efeito nos navegadores (latência de digitação numa página com DOM pesado, com e sem o Aura ligado), não só a CPU do Aura. Preferir chamadas pontuais por evento a assinaturas permanentes de eventos UIA. |
| I-6 | §4.5: lista de excluídos de fábrica | Signal e outros usam `WDA_EXCLUDEFROMCAPTURE`; a proteção cobre pixels, não a UIA (§7.2) | Excluir automaticamente janelas com afinidade de exibição diferente de `WDA_NONE`, também para leitura por UIA. |
| I-7 | §1.1 e §4.2: redação de dados pessoais (`pii.rs`: CPF, CNPJ, cartão, telefone) | O screenpipe filtra chaves de API por padrão; um usuário relatou chaves enviadas a um endpoint por uma automação (P1.8) | Acrescentar segredos à redação: chaves de API, tokens, cabeçalhos `Authorization`, cadeias de conexão. |
| I-8 | §4.2: filtro de repetição por impressão digital do texto | O screenpipe descobriu que só o hash do texto perde slides e tela compartilhada em reunião, e que digitação gera tempestade de capturas (§4.1) | Durante Reunião, gravar por mudança visual mesmo com o mesmo texto; piso de ~1,5 s entre gatilhos de teclado; forçar uma gravação após 30 s parado no mesmo texto (alimenta a duração). |
| I-9 | §4.3: imagem "~80 KB cada" | O screenpipe estima ~400 KB por JPEG de tela cheia no código e compacta em vídeo com ganho alegado de 10–30× (§4.1); AVIF custa ~13× mais CPU que WebP (§6.2) | O S4 mede o tamanho real a 1.280 px em telas de texto e testa compactar imagens antigas em Segmentos H.264, que o Aura já sabe gravar e cifrar. AVIF só se o S4 mostrar que o custo cabe no orçamento de CPU. |
| I-10 | §4.5: Pausa, Janelas excluídas, Esquecer | Separar pessoal e trabalho é pedido recorrente; empresas bloqueiam gravadores de tela (P1.7) | Acrescentar agenda de captura (dias e horários) e um aviso sobre política corporativa na tela de ativação. Perfis pessoal/trabalho ficam como candidato. |
| I-11 | §5.3: ferramentas `memory_*` | Busca top-k responde mal "quantas" e "todas as" (P1.4) | Acrescentar `memory_count`/`memory_list` por filtros exatos (período, fonte, pessoa, app), respondidas por SQL, sem ranqueamento. |
| I-12 | §5.5 e §6: Memórias e Fichas | Fatos extraídos perdem muito contra o bruto (Mem0: 49,0% contra 82,4%); proposições com origem funcionam; confiança limitada pela origem resiste a envenenamento (§8) | Manter os Turnos brutos indexados como fonte principal (o plano já faz). Fatos com origem, validade e **nível de confiança pela fonte**: o que o usuário afirmou não é substituído pelo que apareceu na tela ou numa página. |
| I-13 | §4.4: "Atingido o teto, o Acervo apaga… e avisa" | Muita gente se arrepende de apagar e prefere arquivar (P1.10) | O teto nunca apaga Fichas, Diário, Memórias nem o que está no Cofre de notas; avisa com antecedência e oferece exportar antes de apagar texto. |
| I-14 | §7.1(4) e §7.2: escrita direta na pasta, `Diário/AAAA/AAAA-MM-DD.md` | Obsidian Sync mescla e duplica; notas criadas na abertura podem ser descartadas; Desktop e Documents costumam estar no OneDrive; a CLI falha em silêncio com arquivos escritos por fora (§7.3) | Ao criar o Cofre, sugerir pasta fora de Desktop/Documents. Detectar Obsidian Sync, OneDrive e git e avisar. Preferir arquivos novos e acréscimo no fim a reescrever o mesmo arquivo; nunca usar o nome da nota diária do usuário; recomendar "Create conflict file". Não depender da CLI para gravar propriedades. |
| I-15 | §7.2: propriedades `tipo`, `data`, `pessoas`… | O tipo de uma propriedade vale para o cofre inteiro (§7.3) | Ler `.obsidian/types.json` antes de criar propriedades; não reutilizar nome que o usuário já usa com outro tipo; datas em `AAAA-MM-DD`; links em listas entre aspas. |
| I-16 | §3 e §8, Fase 4: "Aura como servidor MCP só de leitura" (CAND-049) | O usuário comum instala agente de código dentro do cofre (Copilot, Claudian), não RAG (§7.4) | Antecipar o MCP só de leitura do Acervo para logo depois da Fase 2: é o caminho para quem já usa Claude Code ou Codex no cofre. |
| I-17 | §4.5 e §9 S6: consentimento | LGPD não se aplica a uso particular e não econômico (art. 4º, I); uso com clientes é econômico. Granola, Fathom e Fireflies oferecem aviso no chat, marca d'água ou e-mail; o Otter é processado por não pedir a todos (§9) | Separar na interface "uso pessoal" de "uso com clientes". Na Reunião, lembrar o usuário de avisar e oferecer um texto de aviso pronto para colar no chat. Nada capturado vai para treino de modelo. O S6 continua exigindo advogado. |
| I-18 | §0 e posicionamento | O Recall não é otimizado para português (§4.2) | Usar como argumento: memória de tela com busca boa em português, medida no conjunto dourado. |

---

## 11. Não encontrado (resumo)

| Pergunta | Onde procurei |
| --- | --- |
| Threads do Reddit | Seis caminhos (seção 1) |
| Custo de CPU de `SetWinEventHook` e UIA no Windows, medido | Documentação da Microsoft, busca web, código do screenpipe |
| Efeito de JPEG/WebP/AVIF na qualidade do OCR de telas | Busca web, GDELT, guias de compressão |
| Semântica de `DirtyRegionMode` e quadros sem mudança no WGC | Referência da Microsoft, repositório de fontes da documentação, issue do exemplo oficial, libwebrtc |
| Como o Recall detecta janela anônima | Páginas de suporte e de desenvolvedor do Recall, busca web |
| Desempenho de Bekko em português | MTEB-BR, MTEB-PT, cartão do modelo |
| Resultados independentes de Zep/Graphiti e Letta | Busca web e arXiv |
| Orientação da ANPD sobre uso doméstico, gravação de reuniões ou memória de tela | Busca por guias e notas técnicas da ANPD |
| Benchmark de int8 + reordenação no sqlite-vec com 500 mil a 1 milhão de itens | Repositório do sqlite-vec, busca web |
