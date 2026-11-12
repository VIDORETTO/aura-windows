# Pesquisa: segundo cérebro, memória de tela e recuperação

Data: 2026-11-12. Base para `docs/project/plano-segundo-cerebro.md`.

**Como ler.** Cada afirmação traz a fonte. Onde a fonte é o próprio fornecedor, isso está dito. A
rede desta sessão bloqueou a leitura direta de `forum.obsidian.md`, `tildes.net`, Reddit e da
documentação do screenpipe; nesses casos o que está aqui vem de resumos de busca e de citações
de segunda mão, e está marcado como **[2ª mão]**. O gist do Karpathy foi lido na íntegra. Nenhum
número deste documento foi medido por nós.

## 1. O que o usuário pediu, em uma frase

Um modo opcional em que o Aura passa a lembrar de tudo o que ajuda o usuário (reuniões, artigos
lidos, conversas com pessoas e com o próprio agente, o que passou pela tela), aceita gastar mais
disco e processamento, vem pronto para usar e se conecta ao Obsidian sem exigir que a pessoa
monte a estrutura.

## 2. As dores de quem tenta montar um segundo cérebro

Ordenadas pela frequência com que apareceram nas fontes.

| # | Dor | Evidência | O que isso exige do Aura |
| --- | --- | --- | --- |
| D1 | **Manter o sistema custa mais do que ele devolve.** As pessoas descrevem cansaço de cuidar do sistema, não do trabalho. | "The organizing tax was higher than the value I got back" [2ª mão, Hacker News]; "the 'maintaining the system' part is what kills most setups" [2ª mão, Reddit]; Karpathy: "the maintenance burden grows faster than the value" ([gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)) | Manutenção zero: quem arquiva, liga e revisa é o agente. |
| D2 | **Capturar sem filtro vira gaveta de bagunça.** "Falácia do colecionador": salvar parece aprender. | "If it crossed my screen, I dumped it into Obsidian. That's how you end up with 10,000 zombie notes" ([dev.to](https://dev.to/dev_tips/i-rage-quit-my-second-brain-and-it-actually-made-me-a-better-developer-3c02)); "My Obsidian vault turned into a junk drawer once life got busy" [2ª mão, Reddit]; [guia Carly](https://www.usecarly.com/blog/what-is-a-second-brain/) | O bruto fica fora da vista; só o destilado aparece para a pessoa. |
| D3 | **O problema real é achar, não organizar.** Mesmo bem organizado, a pessoa não lembra que a nota existe. | Tópico "Every attempt at PKM has landed me in the same place: a huge mess" ([fórum Obsidian](https://forum.obsidian.md/t/every-attempt-at-pkm-has-landed-me-in-the-same-place-a-huge-mess/89221)) [2ª mão] | Busca por significado, tempo, pessoa e app; e o sistema traz a memória certa na hora, sem a pessoa pedir. |
| D4 | **Paralisia na montagem.** Pastas, PARA, Zettelkasten, dezenas de plugins. | Guias de iniciante repetem "comece com estrutura mínima", "2 ou 3 plugins" ([dsebastien](https://www.dsebastien.net/the-ultimate-beginners-guide-to-obsidian/), [iGeeksBlog](https://www.igeeksblog.com/obsidian-simple-beginner-setup/)); montar um app de segundo cérebro "takes a significant chunk of time" ([Android Authority](https://androidauthority.com/second-brain-apps-alternatives-3695946)) | Estrutura pronta, criada com um clique, sem decisão de método. |
| D5 | **A captura para quando a vida aperta.** Qualquer passo manual morre na semana cheia. | Mesmo conjunto de fontes de D1/D2 | Captura automática, com a pessoa só escolhendo o que nunca capturar. |
| D6 | **"Li isso em algum lugar."** Não lembrar qual era o artigo. | Existe uma categoria inteira de extensões só para isso: [Memberry](https://github.com/shemanaev/memberry), [Retrospective History](https://chromewebstore.google.com/detail/retrospective-history/imgneaikmibmmjghdemcdbakmkhgkfmi), [FindThatPage](https://chromewebstore.google.com/detail/findthatpage/ihcifceibehgdaknhoibnbehdaanbgod), [Hister](https://hn.jakechampion.name/reader/https://www.xda-developers.com/hister-indexes-every-page-visit-search-full-text-browsing-history/) | Guardar URL, título, texto lido e tempo de leitura de cada página. |
| D7 | **Agente dentro do cofre faz estrago.** Notas quase duplicadas, páginas que envelhecem, citação em que não se confia, mistura de texto de IA com texto da pessoa. | Comentários no gist: "the default failure is near-duplicate pages", "never silently overwrite", "it goes stale the second someone renames a thing", "the hard part … is trusting what it tells you later"; guias pedem arquivo de regras, backup antes da primeira escrita e não expor o cofre inteiro ([EvoMap](https://evomap.ai/blog/obsidian-claude-code-vault-to-code-workflow), [MindStudio](https://www.mindstudio.ai/blog/ai-second-brain-claude-code-obsidian-architecture)) | Pasta própria do Aura, nunca sobrescrever texto humano, toda afirmação com origem citada, revisão periódica com aprovação. |
| D8 | **Custo de tokens da ingestão.** | Um implementador do padrão LLM-wiki mediu ~72% dos tokens gastos em *decidir* onde escrever, não em escrever (autorrelato, um único acervo; comentário no gist) | Decisões baratas por regra; LLM só para destilar. |
| D9 | **Ferramentas de memória de tela são pesadas.** | screenpipe: 5–20% de CPU, 0,5–3 GB de RAM, 5–30 GB/mês conforme a fonte ([README](https://cdn.jsdelivr.net/gh/screenpipe/screenpipe@main/README.md), [docs](https://mintlify.com/screenpipe/screenpipe/introduction), [análise](https://www.blog.brightcoding.dev/2026/06/25/screenpipe-the-ai-memory-tool-every-developer-needs)); Recall reserva 25 GB num SSD de 256 GB, 75 GB em 512 GB, 150 GB em 1 TB ([Microsoft](https://support.microsoft.com/en-us/topic/2c35b596-5a96-4090-b791-c27fae75f660)) | Orçamento explícito de CPU, RAM e disco, com teto e descarte do mais antigo. |
| D10 | **Medo de ser gravado.** | Recall foi adiado, virou opt-in e ganhou filtro de dados sensíveis depois da reação ([PCWorld](https://www.pcworld.com/article/2360514/microsoft-retreats-on-recall-always-on-screenshots-will-be-opt-in.html), [The Register](https://www.theregister.com/2024/06/14/microsoft_recall_release_delayed/), [Microsoft](https://support.microsoft.com/windows/d404f672-7647-41e5-886c-a3c59680af15)) | Desligado por padrão, indicador sempre visível, banco cifrado, "esquecer" fácil. |
| D11 | **O fornecedor some e leva a memória.** | Rewind virou Limitless, foi comprado pela Meta em 05/12/2025 e parou de capturar em 19/12/2025 ([Hedy](https://hedy.ai/post/meta-acquires-limitless-ai-privacy/), fonte concorrente) | Dados locais, em formato aberto, exportáveis em Markdown. |
| D12 | **Pilhas de RAG de servidor não servem para usuário de desktop.** | RAGFlow pede Elasticsearch, MySQL, MinIO e Redis, cinco serviços e ~16 GB de RAM ([Railway](https://railway.com/deploy/ragflow-vs-anythingllm--deploy-ragflow-anythingllm-alternative), [PromptQuorum](https://www.promptquorum.com/ko/power-local-llm/local-llm-knowledge-retrieval-compared)) | Motor embutido no próprio Aura; servidores externos só como opcional. |

## 3. O que funcionou para as pessoas

| # | Solução | Fonte | Como o Aura aproveita |
| --- | --- | --- | --- |
| S1 | **O LLM mantém uma wiki; a pessoa só lê e pergunta.** Três camadas: fontes brutas imutáveis, wiki escrita pelo LLM, um arquivo de regras. Três operações: ingerir, consultar, revisar ("lint"). `index.md` como catálogo e `log.md` só de acréscimo. | [Gist do Karpathy](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f) (abril de 2026) | É o desenho do Cofre e da Consolidação (plano §6 e §7). |
| S2 | **Estrutura mínima; pasta nova só quando doer.** Diário primeiro, nota separada depois. Ligações e propriedades em vez de hierarquia. | Guias de iniciante da seção D4 | Cofre com poucas pastas fixas e diário como porta de entrada. |
| S3 | **Um laço que roda mesmo quando a motivação cai.** | [Nate's Newsletter](https://natesnewsletter.substack.com/p/grab-the-system-that-closes-open); relato de "sexta tentativa" que deu certo quando a IA passou a classificar ([dev.to](https://dev.to/huy_tieu/i-finally-built-a-second-brain-that-i-actually-use-6th-attempt-4075), um autor só) | Consolidação em segundo plano usando as Instruções agendadas que já existem (esforço 030). |
| S4 | **Separar o que é do agente do que é da pessoa**, com regras na raiz, backup e escopo limitado. | [MindStudio](https://www.mindstudio.ai/blog/ai-second-brain-claude-code-obsidian-architecture), [EvoMap](https://evomap.ai/blog/obsidian-claude-code-vault-to-code-workflow) | O Aura só escreve em `Aura/` e só em blocos marcados. |
| S5 | **Capturar por evento, texto de acessibilidade primeiro, OCR só como reserva.** Gatilhos: troca de app, clique, pausa de digitação. | screenpipe ([arquitetura](https://mintlify.com/screenpipe/screenpipe/architecture), [como funciona](https://screenpi.pe/blog/ai-screen-search-how-it-works)) [2ª mão] | É o desenho dos Momentos de tela (plano §4). |
| S6 | **Teto de disco proporcional, apagando o mais antigo; filtro de dados sensíveis ligado.** | [Microsoft Recall](https://support.microsoft.com/en-us/topic/2c35b596-5a96-4090-b791-c27fae75f660) | Retenção em camadas com teto (plano §4.5). |
| S7 | **Busca híbrida (palavra + significado) com contexto no trecho.** A Anthropic mediu queda de falhas de recuperação de 35% (contexto no embedding), 49% (com BM25) e 67% (com reordenação), contra uma base de embeddings simples. | [Anthropic, Contextual Retrieval](https://www.anthropic.com/engineering/contextual-retrieval) | Recuperação do Acervo (plano §5). Quem já tem busca híbrida ganha perto dos 35%, não dos 67%. |
| S8 | **Um índice só.** Vários plugins de embeddings no mesmo cofre atrapalham. Padrão de 2026: recuperação local + agente externo via MCP. | [Levantamento de abril de 2026](https://laoujin.github.io/Atlas/research/2026-04-30-upgrading-a-personal-obsidian-workflow-in-2026/ai-plugins-for-in-vault-retrieval-summarization/) | O Aura é o índice e o agente; não depende de plugin do Obsidian. |

## 4. Produtos e projetos de referência

| Projeto | O que faz | Aproveitamos | Não aproveitamos |
| --- | --- | --- | --- |
| **screenpipe** | Memória contínua de tela e áudio, busca, agentes, MCP | Ideias: captura por evento, acessibilidade antes de OCR | Código. A licença mudou de MIT para comercial/source-available (README e listagens recentes divergem das antigas). Já estava em "Evitar" no radar; continua. |
| **Microsoft Recall** | Fotos da tela a cada poucos segundos, busca semântica local | Teto de disco por tamanho do SSD, descarte do mais antigo, filtro de sensíveis, opt-in | Só existe em PCs Copilot+; fechado. |
| **Rewind / Limitless** | Foi a referência no Mac; encerrado | A lição D11 | — |
| **OpenRecall, Windrecorder** | Clones abertos do Recall | Confirmam a demanda; Windrecorder busca por OCR e por descrição da imagem | Python; não lidos em detalhe nesta sessão. |
| **Padrão LLM-wiki (Karpathy)** | Wiki mantida por LLM, Obsidian como leitor | Camadas, operações, index/log | O gist diz que `index.md` funciona até "~100 fontes, ~centenas de páginas"; acima disso é preciso busca de verdade. |
| **Smart Connections, Copilot, Khoj (plugins Obsidian)** | Embeddings e chat dentro do cofre | Mostram o que o usuário espera ver | Exigem o Obsidian aberto e configuração por plugin. |
| **RAGFlow** | RAG de servidor com ótimo parser de PDFs e tabelas | Só como base externa opcional via MCP | Como motor: Docker, cinco serviços, ~16 GB de RAM (D12). |
| **AnythingLLM** | App desktop com LanceDB embutido | Prova de que "um app, índice embutido" é o formato certo | Electron; 10+ avisos de segurança entre 2024 e 2026 segundo uma wiki de administração. |
| **Mem0, Zep/Graphiti, Letta** | Memória de conversa para agentes | Ideias: extrair fatos e consolidar (Mem0), fatos com janela de validade (Graphiti), memória em camadas (Letta) | Como dependência. São serviços de servidor, e os benchmarks não permitem escolher: Mem0 se atribui 93,4 no LongMemEval e uma avaliação independente mediu 49,0 numa versão anterior ([arXiv 2603.04814](https://arxiv.org/html/2603.04814), [comparativo](https://www.agenticwire.news/article/mem0-zep-letta-agent-memory)). |

## 5. Tecnologias para o motor

### 5.1 Índice

| Opção | A favor | Contra |
| --- | --- | --- |
| **SQLite FTS5 + sqlite-vec** | Mesmo arquivo e mesma biblioteca que o Aura já usa; FTS5 já está no radar como "Adotar"; sqlite-vec é C puro e pequeno, sucessor mantido do sqlite-vss; há um crate Rust que já junta os dois ([sapphire-retrieve](https://docs.rs/crate/sapphire-retrieve/0.2.0/source/src/sqlite_store.rs)) e um benchmark público pequeno em que FTS5 + sqlite-vec com fusão e reordenação fica perto do DuckDB ([sanityboard](https://sanityboard.pages.dev/evals/vecdb)) | Busca vetorial por força bruta; precisa de quantização e filtro prévio para escalar |
| LanceDB | Rust, colunar, busca vetorial e textual | Segundo armazenamento ao lado do SQLite; as fontes divergem sobre o texto completo vir pronto |
| Servidor (Qdrant, Elasticsearch) | Escala | Processo residente a mais; contraria a leveza |

Recomendação: **FTS5 + sqlite-vec**, com fusão por posição recíproca (RRF). Licença do
sqlite-vec a confirmar no ticket (regra do `deny.toml`).

### 5.2 Modelo de embeddings local (multilíngue, CPU)

| Modelo | Tamanho | Observação |
| --- | --- | --- |
| multilingual-e5-small | pequeno | Padrão histórico; conversões ONNX rodam só em CPU com pouca perda. 50,9 em recuperação na tabela dos autores do Bekko. |
| Bekko a8m / a25m (2026) | 7,7 M / 24,9 M parâmetros | Licença MIT, traz ONNX. Os autores reportam 56,2 (a8m) no MMTEB multilíngue, acima do e5 e do BGE-M3, e 364 documentos/s num Ryzen 9. **Autoavaliação.** ([modelo](https://huggingface.co/hotchpotch/bekko-embedding-v1-a8m), [artigo](https://arxiv.org/pdf/2607.25180)) |
| EmbeddingGemma-300M | <200 MB de RAM quantizado | 100+ idiomas, trechos até 512 tokens ([Google](https://developers.googleblog.com/en/introducing-embeddinggemma/)). Usa os termos do Gemma, que não são uma licença permissiva comum: **checar antes de considerar.** |
| BGE-M3 | ~568 M parâmetros | Forte e pesado; 54,6 na mesma tabela. |

Nenhuma fonte trouxe números para **português**. A escolha tem de sair de um teste nosso
(plano §9, spike S1).

### 5.3 Imagens

- Para capturas de tela, que são quase sempre texto, OCR + embedding de texto é mais barato e
  competitivo. Um estudo de 2025 encontrou pipelines com OCR à frente do ColPali em todos os
  níveis de qualidade de documento ([arXiv 2505.05666](https://arxiv.org/abs/2505.05666)).
- ColPali/ColQwen guardam muitos vetores por página; custo de disco e latência altos para um
  acervo pessoal de centenas de milhares de imagens.
- Embedding visual de vetor único (família CLIP/SigLIP) faz sentido para busca por aparência
  ("aquele gráfico azul"). A busca desta sessão não confirmou disponibilidade em ONNX nem
  licenças; fica como spike.

### 5.4 Captura no Windows

- **Desktop Duplication (DXGI)** entrega, por quadro, a lista de retângulos alterados; quadro
  sem retângulos é sinal gratuito de "nada mudou" ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_2/nf-dxgi1_2-idxgioutputduplication-getframedirtyrects)). A duplicação cai em troca de sessão e precisa ser recriada.
- **Windows.Graphics.Capture** ganhou `DirtyRegionMode` e `MinUpdateInterval` no Windows 11
  24H2 (build 26100). Há "essentially no documentation" segundo um
  [issue do exemplo oficial](https://github.com/robmikh/Win32CaptureSample/issues/82); o libwebrtc
  testa a presença da API antes de usar.
- O Recall tira uma foto "a cada poucos segundos". A regra "só se a tela mudou" que circula não
  foi confirmada nas páginas da Microsoft encontradas.

### 5.5 Obsidian

- Um cofre é uma pasta de arquivos Markdown. Escrever nessa pasta funciona com o Obsidian
  fechado e sem plugin.
- **CLI oficial** desde a versão 1.12.0 (fevereiro de 2026): `obsidian search query="…"`,
  `create`, `append`, `prepend`, saída em JSON. Ainda em acesso antecipado segundo uma fonte, e
  **precisa do aplicativo aberto** ([Obsidian](https://obsidian.md/cli), [dev.to](https://dev.to/shimo4228/obsidians-official-cli-is-here-no-more-hacking-your-vault-from-the-back-door-3123)). Serve como atalho opcional, não como dependência.
- Propriedades no cabeçalho YAML alimentam Bases e Dataview. A sintaxe dos comandos de Bases na
  CLI não foi confirmada.

## 6. O que a pesquisa não respondeu

1. Opiniões diretas do Reddit e do fórum do Obsidian (bloqueados; só citações de segunda mão).
2. Qualidade dos modelos de embedding em português.
3. Custo real de CPU da captura por evento com UI Automation no Windows.
4. Formato de disco e deduplicação internos do screenpipe e do Recall.
5. Consentimento de terceiros em gravação de conversas: nenhuma fonte encontrada. É questão
   jurídica (LGPD) que precisa de orientação própria antes de qualquer promessa na interface.
