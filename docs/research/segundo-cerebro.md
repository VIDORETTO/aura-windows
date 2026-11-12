# Pesquisa: segundo cérebro, memória de tela e recuperação

Data: 2026-11-12, revisado no mesmo dia. Base para `docs/project/plano-segundo-cerebro.md`.
Detalhes, métodos e a lista de mudanças propostas ao plano estão em
`docs/research/segundo-cerebro-complemento-2026-11.md` (citado abaixo como **[C §n]**).

**Como ler.** Cada afirmação traz a fonte. Onde a fonte é o próprio fornecedor, isso está dito
(**[autodeclarado]**). A primeira versão foi escrita com a rede restrita e marcava como
**[2ª mão]** o que vinha de resumos de busca; na revisão, o fórum do Obsidian, o Tildes, o
Hacker News, o código do screenpipe e as páginas da Microsoft foram lidos direto, e as marcas
foram substituídas. O Reddit continua inacessível (bloqueio à rede e ao buscador, [C §1]); o que
dependia dele está marcado **[não verificado]**. O gist do Karpathy foi lido na íntegra. Nenhum
número deste documento foi medido por nós.

## 1. O que o usuário pediu, em uma frase

Um modo opcional em que o Aura passa a lembrar de tudo o que ajuda o usuário (reuniões, artigos
lidos, conversas com pessoas e com o próprio agente, o que passou pela tela), aceita gastar mais
disco e processamento, vem pronto para usar e se conecta ao Obsidian sem exigir que a pessoa
monte a estrutura.

## 2. As dores de quem tenta montar um segundo cérebro

D1–D12 ordenadas pela frequência com que apareceram nas fontes da primeira versão. D13–D18
foram acrescentadas na revisão, com a recorrência indicada em cada linha; método de contagem em
[C §2.1].

| # | Dor | Evidência | O que isso exige do Aura |
| --- | --- | --- | --- |
| D1 | **Manter o sistema custa mais do que ele devolve.** As pessoas descrevem cansaço de cuidar do sistema, não do trabalho. Recorrente: 4 sites, 5 discussões [C §2.2]. | "you have to spend SO MUCH time writing notes" ([ErrorNoBrain, HN](https://news.ycombinator.com/item?id=44402670)); "a Second Job instead of a Second Brain" ([nivertech, HN](https://news.ycombinator.com/item?id=44440855)); "I'd become a professional organizer, not a better thinker" ([sajjadkhavari, fórum Obsidian](https://forum.obsidian.md/t/115302)); Karpathy: "the maintenance burden grows faster than the value" ([gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)). As duas frases de segunda mão da versão anterior ("The organizing tax…", HN; "the 'maintaining the system' part…", Reddit) não foram localizadas nas discussões lidas [não verificado]. | Manutenção zero: quem arquiva, liga e revisa é o agente. |
| D2 | **Capturar sem filtro vira gaveta de bagunça**, inclusive transcrições. "Falácia do colecionador": salvar parece aprender. Recorrente: 3 sites, 4 discussões [C §2.2]. | "hundreds or even thousands of transcriptions that are effectively meaningless… they just pile up, collecting, not connecting" ([romebot, fórum Obsidian](https://forum.obsidian.md/t/every-attempt-at-pkm-has-landed-me-in-the-same-place-a-huge-mess/89221)); "Bigger graph = smarter me, or so I thought" ([M0r13n, HN](https://news.ycombinator.com/item?id=44403262)); "If it crossed my screen, I dumped it into Obsidian… 10,000 zombie notes" ([dev.to](https://dev.to/dev_tips/i-rage-quit-my-second-brain-and-it-actually-made-me-a-better-developer-3c02)); [guia Carly](https://www.usecarly.com/blog/what-is-a-second-brain/). A frase "junk drawer" atribuída ao Reddit: [não verificado]. | O bruto fica fora da vista; só o destilado aparece para a pessoa. Reuniões precisam virar resumo, não só trechos. |
| D3 | **O problema real é achar, não organizar.** Mesmo bem organizado, a pessoa não lembra que a nota existe. Recorrente: 2 sites, 4 discussões [C §2.2]. | "the awareness required for retrieval is challenging no matter how well everything is organized" e "200 notes on a topic become clutter, and there's no way to distill it when I need it" ([fórum Obsidian 89221](https://forum.obsidian.md/t/every-attempt-at-pkm-has-landed-me-in-the-same-place-a-huge-mess/89221), lido direto); "Search and tags still rules over everything" ([on_the_train, HN](https://news.ycombinator.com/item?id=47757377)) | Busca por significado, tempo, pessoa e app; e o sistema traz a memória certa na hora, sem a pessoa pedir. |
| D4 | **Paralisia na montagem.** Pastas, PARA, Zettelkasten, dezenas de plugins. | Guias de iniciante repetem "comece com estrutura mínima", "2 ou 3 plugins" ([dsebastien](https://www.dsebastien.net/the-ultimate-beginners-guide-to-obsidian/), [iGeeksBlog](https://www.igeeksblog.com/obsidian-simple-beginner-setup/)); montar um app de segundo cérebro "takes a significant chunk of time" ([Android Authority](https://androidauthority.com/second-brain-apps-alternatives-3695946)) | Estrutura pronta, criada com um clique, sem decisão de método. |
| D5 | **A captura para quando a vida aperta.** Qualquer passo manual morre na semana cheia. | Mesmo conjunto de fontes de D1/D2 | Captura automática, com a pessoa só escolhendo o que nunca capturar. |
| D6 | **"Li isso em algum lugar."** Não lembrar qual era o artigo. | Existe uma categoria inteira de extensões só para isso: [Memberry](https://github.com/shemanaev/memberry), [Retrospective History](https://chromewebstore.google.com/detail/retrospective-history/imgneaikmibmmjghdemcdbakmkhgkfmi), [FindThatPage](https://chromewebstore.google.com/detail/findthatpage/ihcifceibehgdaknhoibnbehdaanbgod), [Hister](https://hn.jakechampion.name/reader/https://www.xda-developers.com/hister-indexes-every-page-visit-search-full-text-browsing-history/) | Guardar URL, título, texto lido e tempo de leitura de cada página. |
| D7 | **Agente dentro do cofre faz estrago.** Notas quase duplicadas, páginas que envelhecem, citação em que não se confia, mistura de texto de IA com texto da pessoa. | Comentários no gist: "the default failure is near-duplicate pages", "never silently overwrite", "it goes stale the second someone renames a thing", "the hard part … is trusting what it tells you later"; guias pedem arquivo de regras, backup antes da primeira escrita e não expor o cofre inteiro ([EvoMap](https://evomap.ai/blog/obsidian-claude-code-vault-to-code-workflow), [MindStudio](https://www.mindstudio.ai/blog/ai-second-brain-claude-code-obsidian-architecture)) | Pasta própria do Aura, nunca sobrescrever texto humano, toda afirmação com origem citada, revisão periódica com aprovação. |
| D8 | **Custo de tokens da ingestão.** | Um implementador do padrão LLM-wiki mediu ~72% dos tokens gastos em *decidir* onde escrever, não em escrever (autorrelato, um único acervo; comentário no gist) | Decisões baratas por regra; LLM só para destilar. |
| D9 | **Ferramentas de memória de tela são pesadas.** | screenpipe: CPU "approximately 5-20%", RAM "approximately 0.5-3 GB", disco "varies… Measure a representative day" ([README atual](https://github.com/screenpipe/screenpipe/blob/main/README.md)) [autodeclarado]; os 5–30 GB/mês de listagens antigas saíram do README. Um usuário desligou o screenpipe porque "made my CPU melt" num M1 ([kerv, HN](https://news.ycombinator.com/item?id=49031269)). Recall reserva 25 GB num SSD de 256 GB, 75 GB em 512 GB, 150 GB em 1 TB ([Microsoft](https://support.microsoft.com/en-us/topic/2c35b596-5a96-4090-b791-c27fae75f660)), exige 50 GB livres e pausa abaixo de 25 GB ([Microsoft](https://support.microsoft.com/en-us/windows/retrace-your-steps-with-recall-aa03f8a0-a78b-4b3e-b0a1-2eb8ac48701c)) | Orçamento explícito de CPU, RAM e disco, com teto e descarte do mais antigo. |
| D10 | **Medo de ser gravado.** | Recall foi adiado, virou opt-in e ganhou filtro de dados sensíveis depois da reação ([PCWorld](https://www.pcworld.com/article/2360514/microsoft-retreats-on-recall-always-on-screenshots-will-be-opt-in.html), [The Register](https://www.theregister.com/2024/06/14/microsoft_recall_release_delayed/), [Microsoft](https://support.microsoft.com/windows/d404f672-7647-41e5-886c-a3c59680af15)) | Desligado por padrão, indicador sempre visível, banco cifrado, "esquecer" fácil. |
| D11 | **O fornecedor some e leva a memória.** | Rewind virou Limitless, foi comprado pela Meta em 05/12/2025 e parou de capturar em 19/12/2025 ([Hedy](https://hedy.ai/post/meta-acquires-limitless-ai-privacy/), fonte concorrente) | Dados locais, em formato aberto, exportáveis em Markdown. |
| D12 | **Pilhas de RAG de servidor não servem para usuário de desktop.** | RAGFlow pede Elasticsearch, MySQL, MinIO e Redis, cinco serviços e ~16 GB de RAM ([Railway](https://railway.com/deploy/ragflow-vs-anythingllm--deploy-ragflow-anythingllm-alternative), [PromptQuorum](https://www.promptquorum.com/ko/power-local-llm/local-llm-knowledge-retrieval-compared)); no Obsidian, o Khoj tem 55 mil downloads contra 3,5 milhões do Copilot ([estatística oficial](https://github.com/obsidianmd/obsidian-releases/blob/master/community-plugin-stats.json), [C §7.4]) | Motor embutido no próprio Aura; servidores externos só como opcional. |
| D13 | **Sincronização corrompe ou duplica notas.** Recorrente: 2 sites, 5 discussões. | Tópico de 105 posts sobre seções duplicadas pelo Obsidian Sync ([fórum](https://forum.obsidian.md/t/94732)); troca silenciosa do conteúdo de uma nota ([hadik, HN](https://news.ycombinator.com/item?id=47759111)); dois sincronizadores no mesmo cofre geram "duplicated notes and silently reverted edits" ([fórum](https://forum.obsidian.md/t/117819)); o Sync descarta a nota criada localmente se a remota chegar em poucos minutos ([ajuda do Obsidian](https://obsidian.md/help/sync/troubleshoot)) | O Aura escreve de um jeito que não briga com o sincronizador: arquivos novos ou acréscimo, nada de reescrever a mesma nota, pasta fora do OneDrive [C §10, I-14]. |
| D14 | **Pessoal e trabalho misturados; empresa que proíbe gravador de tela.** Recorrente: 1 site, 2 discussões. | "I can't sync my sensitive work notes" ([HN](https://news.ycombinator.com/item?id=47757377)); "How are you planing to segregate between professional use and personal use" ([HN](https://news.ycombinator.com/item?id=49024761)); empresas que "blocks unaudited desktop recorders by policy" ([HN](https://news.ycombinator.com/item?id=49033472)) | Agenda de captura e aviso sobre política corporativa; perfis como candidato [C §10, I-10]. |
| D15 | **A memória de tela aumenta o estrago de um agente ou de um vazamento.** 1 discussão, 3 autores. | "a much bigger blast radius" ([HN](https://news.ycombinator.com/item?id=49031437)); automação que enviou chaves de API locais a um endpoint ([HN](https://news.ycombinator.com/item?id=49027906)); o TotalRecall extraiu dados do Recall sem quebrar a cifra, rodando como o mesmo usuário ([TotalRecall](https://github.com/xaitax/TotalRecall)) | Escopo por consulta, redação de segredos e honestidade sobre o que a cifra cobre [C §10, I-4 e I-7]. |
| D16 | **IA no cofre erra contagem e listagem.** Opinião de um autor, mas é limite conhecido de busca top-k. | "show me all my notes on a topic" e "how many notes do I have?" "give partial results" ([fórum Obsidian](https://forum.obsidian.md/t/every-attempt-at-pkm-has-landed-me-in-the-same-place-a-huge-mess/89221)) | Ferramentas de contagem e listagem por filtro exato [C §10, I-11]. |
| D17 | **Apagar tudo alivia, mas muita gente se arrepende.** Recorrente numa discussão, opinião dividida. | "keep all notes, archived" ([HN](https://news.ycombinator.com/item?id=44403363)); "One of the few decisions I absolutely regret…" ([HN](https://news.ycombinator.com/item?id=44406901)) | O teto de disco não apaga o destilado e avisa antes [C §10, I-13]. |
| D18 | **Escrever é o que vale; quase ninguém relê.** Recorrente: 3 sites. | "the act of writing is the useful part" ([Tildes](https://tildes.net/~tech/1oth/i_deleted_my_second_brain)); "The writing is the important bit, not the resulting output" ([HN](https://news.ycombinator.com/item?id=44403917)) | O Aura não substitui as notas da pessoa nem as enche de texto gerado; a captura é complemento. |

## 3. O que funcionou para as pessoas

| # | Solução | Fonte | Como o Aura aproveita |
| --- | --- | --- | --- |
| S1 | **O LLM mantém uma wiki; a pessoa só lê e pergunta.** Três camadas: fontes brutas imutáveis, wiki escrita pelo LLM, um arquivo de regras. Três operações: ingerir, consultar, revisar ("lint"). `index.md` como catálogo e `log.md` só de acréscimo. | [Gist do Karpathy](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f) (abril de 2026) | É o desenho do Cofre e da Consolidação (plano §6 e §7). |
| S2 | **Estrutura mínima; pasta nova só quando doer.** Diário primeiro, nota separada depois. Ligações e propriedades em vez de hierarquia. | Guias de iniciante da seção D4 | Cofre com poucas pastas fixas e diário como porta de entrada. |
| S3 | **Um laço que roda mesmo quando a motivação cai.** | [Nate's Newsletter](https://natesnewsletter.substack.com/p/grab-the-system-that-closes-open); relato de "sexta tentativa" que deu certo quando a IA passou a classificar ([dev.to](https://dev.to/huy_tieu/i-finally-built-a-second-brain-that-i-actually-use-6th-attempt-4075), um autor só) | Consolidação em segundo plano usando as Instruções agendadas que já existem (esforço 030). |
| S4 | **Separar o que é do agente do que é da pessoa**, com regras na raiz, backup e escopo limitado. | [MindStudio](https://www.mindstudio.ai/blog/ai-second-brain-claude-code-obsidian-architecture), [EvoMap](https://evomap.ai/blog/obsidian-claude-code-vault-to-code-workflow) | O Aura só escreve em `Aura/` e só em blocos marcados. |
| S5 | **Capturar por evento, texto de acessibilidade primeiro, OCR só como reserva.** Gatilhos: troca de app e de foco, clique, pausa de digitação, fim de rolagem, cópia; verificação visual a cada 3 s; reserva a cada 30 s; piso de 1,5 s para gatilhos de teclado; repetição descartada por hash do texto, exceto em reunião. | screenpipe, lido no código e na especificação ([EVENT_DRIVEN_CAPTURE_SPEC.md](https://github.com/screenpipe/screenpipe/blob/main/docs/EVENT_DRIVEN_CAPTURE_SPEC.md), [event_driven_capture.rs](https://github.com/screenpipe/screenpipe/blob/main/crates/screenpipe-engine/src/event_driven_capture.rs)) [C §4.1] | É o desenho dos Momentos de tela (plano §4), com os ajustes I-8 [C §10]. |
| S6 | **Teto de disco proporcional, apagando o mais antigo; filtro de dados sensíveis ligado.** | [Microsoft Recall](https://support.microsoft.com/en-us/topic/2c35b596-5a96-4090-b791-c27fae75f660) | Retenção em camadas com teto (plano §4.5). |
| S7 | **Busca híbrida (palavra + significado) com contexto no trecho.** A Anthropic mediu queda de falhas de recuperação de 35% (contexto no embedding), 49% (com BM25) e 67% (com reordenação), contra uma base de embeddings simples. | [Anthropic, Contextual Retrieval](https://www.anthropic.com/engineering/contextual-retrieval) | Recuperação do Acervo (plano §5). Quem já tem busca híbrida ganha perto dos 35%, não dos 67%. |
| S8 | **Um índice só.** Vários plugins de embeddings no mesmo cofre atrapalham. Padrão de 2026: recuperação local + agente externo via MCP. | [Levantamento de abril de 2026](https://laoujin.github.io/Atlas/research/2026-04-30-upgrading-a-personal-obsidian-workflow-in-2026/ai-plugins-for-in-vault-retrieval-summarization/); downloads: Copilot (agentes no cofre) 3,5 M, Claudian 2,7 M, Omnisearch 2,1 M, Smart Connections 1,3 M ([estatística oficial](https://github.com/obsidianmd/obsidian-releases/blob/master/community-plugin-stats.json)) | O Aura é o índice e o agente; não depende de plugin do Obsidian. Quem já usa Claude Code ou Codex no cofre consulta o Acervo por MCP [C §10, I-16]. |
| S9 | **Uma entrada cronológica única, sem processar** (nota diária ou caixa de entrada). Recorrente: 3 sites, 4 discussões. | "daily notes, which i barely ever 'process'" ([fórum Obsidian](https://forum.obsidian.md/t/every-attempt-at-pkm-has-landed-me-in-the-same-place-a-huge-mess/89221)); "Just do it chronologically. It's fine." ([HN](https://news.ycombinator.com/item?id=47778549)); "inbox folder… Then daily notes. That's it." ([HN](https://news.ycombinator.com/item?id=47759154)) | Confirma `Caixa de entrada.md` e `Diário/` como portas de entrada (plano §7.2). |
| S10 | **Separar o registro bruto da memória derivada e apagar pela linhagem.** | "derived, revisable claims" que dizem se estão "current, disputed, or superseded", e apagar um intervalo invalida embeddings, entidades e resumos ([HN](https://news.ycombinator.com/item?id=49093110)); "treating deletion as part of the data model" ([HN](https://news.ycombinator.com/item?id=49026609)) | Confirma bruto imutável, origem obrigatória e Esquecer que apaga o derivado (plano §4.5 e §6). |
| S11 | **Guardar o bruto e buscar nele; fato extraído é índice, não substituto.** | Avaliação independente: Mem0 49,0% contra 82,4% do contexto longo no LongMemEval ([arXiv 2603.04814](https://arxiv.org/abs/2603.04814)); proposições curtas com turno de origem igualam ou superam bases fortes ([arXiv 2511.17208](https://arxiv.org/abs/2511.17208)) [C §8] | Turnos brutos continuam indexados; Fichas com origem e confiança pela fonte [C §10, I-12]. |
| S12 | **Agenda de captura, filtros por app e site, e modo só texto.** | screenpipe: "stop recording on weekend" e "only capture accessibility data, it still captures most of the meaningful data" ([HN](https://news.ycombinator.com/item?id=49024847), [HN](https://news.ycombinator.com/item?id=49035249)) [autodeclarado] | Predefinição Econômico (só texto) já existe; falta a agenda [C §10, I-10]. |
| S13 | **Respeitar quem pede para não ser capturado.** | Signal liga `WDA_EXCLUDEFROMCAPTURE` por padrão no Windows 11 por causa do Recall ([Signal](https://signal.org/blog/signal-doesnt-recall/)) | Excluir essas janelas também da leitura por UIA [C §10, I-6]. |

## 4. Produtos e projetos de referência

| Projeto | O que faz | Aproveitamos | Não aproveitamos |
| --- | --- | --- | --- |
| **screenpipe** | Memória contínua de tela e áudio, busca, agentes, MCP | Ideias: captura por evento, acessibilidade antes de OCR, exceção de reunião na deduplicação, compactação de imagens antigas em vídeo, redação de segredos [C §4.1] | Código. Desde 10/06/2026 o repositório inteiro está sob a "Screenpipe Commercial License": uso livre só pessoal não comercial, educacional ou de pesquisa; proíbe usar o código para "build… a competing product"; versões antigas continuam MIT ([LICENSE.md](https://github.com/screenpipe/screenpipe/blob/main/LICENSE.md)). Cifra em repouso só quando o usuário tranca o cofre. Continua em "Evitar". |
| **Microsoft Recall** | Captura "every few seconds and when the content of your active window changes" ([Microsoft](https://support.microsoft.com/en-us/windows/retrace-your-steps-with-recall-aa03f8a0-a78b-4b3e-b0a1-2eb8ac48701c)), busca semântica local | Teto de disco por tamanho do SSD, descarte do mais antigo, filtro de sensíveis ligado por padrão, opt-in, filtro de navegação privada, DRM e área de trabalho remota [C §4.2] | Só existe em PCs Copilot+; fechado. Não é otimizado para português. Cifra forte, mas extraível por processo do mesmo usuário ([TotalRecall Reloaded](https://github.com/xaitax/TotalRecall), 04/2026). |
| **Rewind / Limitless** | Foi a referência no Mac; encerrado | A lição D11 | — |
| **OpenRecall** | Clone aberto do Recall (AGPL-3.0) | Descarte por similaridade estrutural (MSSIM ≥ 0,9) a cada 3 s [C §4.3] | Código (AGPL); WebP sem perdas é caro em disco. |
| **Windrecorder** | Gravador de tela com busca (GPL-2.0) | Três filtros de repetição (imagem 0,7, OCR 0,7, tabela 0,94); busca por imagem com uform3 multilíngue; 10–20 GB/mês de vídeo e ~160 MB/mês de banco [autodeclarado] [C §4.4] | Código (GPL); vídeo contínuo. |
| **Padrão LLM-wiki (Karpathy)** | Wiki mantida por LLM, Obsidian como leitor | Camadas, operações, index/log | O gist diz que `index.md` funciona até "~100 fontes, ~centenas de páginas"; acima disso é preciso busca de verdade. |
| **Smart Connections, Copilot, Claudian, Khoj (plugins Obsidian)** | Embeddings, chat e agentes de código dentro do cofre | Mostram o que o usuário espera ver; o que mais se instala é agente de código no cofre (Copilot 3,5 M e Claudian 2,7 M de downloads) e busca lexical (Omnisearch 2,1 M); Khoj tem 55 mil [C §7.4] | Exigem o Obsidian aberto e configuração por plugin. |
| **RAGFlow** | RAG de servidor com ótimo parser de PDFs e tabelas | Só como base externa opcional via MCP | Como motor: Docker, cinco serviços, ~16 GB de RAM (D12). |
| **AnythingLLM** | App desktop com LanceDB embutido | Prova de que "um app, índice embutido" é o formato certo | Electron; 10+ avisos de segurança entre 2024 e 2026 segundo uma wiki de administração. |
| **Mem0, Zep/Graphiti, Letta** | Memória de conversa para agentes | Ideias: extrair fatos e consolidar (Mem0), fatos com janela de validade (Graphiti), memória em camadas (Letta) | Como dependência. São serviços de servidor, e os números dos fornecedores não se sustentam: Mem0 se atribui mais de 90 no LongMemEval ([comparativo](https://www.agenticwire.news/article/mem0-zep-letta-agent-memory)) [autodeclarado]; a avaliação independente ([arXiv 2603.04814](https://arxiv.org/abs/2603.04814), lida) mediu 49,0% contra 82,4% de passar o histórico inteiro ao mesmo modelo [C §8]. Zep e Letta: resultado independente não encontrado. |

## 5. Tecnologias para o motor

### 5.1 Índice

| Opção | A favor | Contra |
| --- | --- | --- |
| **SQLite FTS5 + sqlite-vec** | Mesmo arquivo e mesma biblioteca que o Aura já usa; FTS5 já está no radar como "Adotar"; sqlite-vec é C puro e pequeno, licença MIT ou Apache-2.0; o screenpipe usa em produção com Rust; combina com SQLCipher (projetos públicos fazem isso, com a sintaxe `k = ?` quando o SQLite do SQLCipher é antigo) [C §5.1]; há um crate que já junta FTS5 e sqlite-vec ([sapphire-retrieve](https://docs.rs/crate/sapphire-retrieve/0.2.0/source/src/sqlite_store.rs)) e um benchmark pequeno em que a fusão com reordenação fica perto do DuckDB ([sanityboard](https://sanityboard.pages.dev/evals/vecdb)) | Busca vetorial por força bruta; **int8 não acelera** (39 ms contra 38,7 ms de float32 com 100 mil × 384 num i9; binário: 2 ms) ([SimpleVecDB](https://coderdayton.github.io/SimpleVecDB/benchmarks)) [autodeclarado]; pré-1.0 e sem commits desde 18/05/2026 ([repositório](https://github.com/asg017/sqlite-vec)) |
| LanceDB | Rust, colunar, busca vetorial e textual, Apache-2.0 | Segundo armazenamento ao lado do SQLite, fora da cifra do SQLCipher |
| BLOB no próprio SQLite + varredura em Rust | Sem extensão, cifrado junto com o resto | Mesmo custo de força bruta; código nosso para manter |
| Servidor (Qdrant, Elasticsearch) | Escala | Processo residente a mais; contraria a leveza |

Recomendação: **FTS5 + sqlite-vec**, com fusão por posição recíproca (RRF), primeiro passo
binário e reordenação dos melhores, e saída documentada para BLOB + varredura em Rust se o
sqlite-vec parar [C §10, I-2 e I-3]. A licença do sqlite-vec passa no `deny.toml`.

### 5.2 Modelo de embeddings local (multilíngue, CPU)

Recuperação em português do Brasil: nDCG@10 médio em seis tarefas nativas do
[MTEB-BR](https://arxiv.org/abs/2607.04581) (julho de 2026), média calculada por nós a partir dos
[resultados publicados](https://huggingface.co/datasets/mteb-pt/mteb-pt-results) [C §3.1].

| Modelo | Tamanho | Licença | MTEB-BR | Observação |
| --- | --- | --- | ---: | --- |
| multilingual-e5-small | 118 M parâm. | MIT | 0,507 | Padrão histórico; ONNX int8 oficial de 118 MB. 50,9 em recuperação na tabela dos autores do Bekko. |
| **granite-embedding-97m-multilingual-r2** (2026) | 97 M | Apache-2.0 | **0,557** | 384 dimensões, 8.192 tokens, ONNX int8 oficial de 98 MB ([modelo](https://huggingface.co/ibm-granite/granite-embedding-97m-multilingual-r2)). Não estava na primeira versão. |
| Bekko a8m / a25m (2026) | 7,7 M / 24,9 M parâmetros **ativos**; 106 M / 123 M no total | MIT | não avaliado | Traz ONNX (130 MB / 199 MB). Os autores reportam 56,2 (a8m) no MMTEB multilíngue e 364 documentos/s num Ryzen 9, mas nas tarefas multilíngues de pergunta e resposta ficam abaixo do gte-multilingual-base e do BGE-M3. **Autoavaliação.** ([modelo](https://huggingface.co/hotchpotch/bekko-embedding-v1-a8m), [artigo](https://arxiv.org/pdf/2607.25180), [C §3.4]) |
| EmbeddingGemma-300M | 308 M | **Termos do Gemma** | 0,653 | O melhor pequeno em português, mas os [termos](https://ai.google.dev/gemma/terms) obrigam a repassar restrições de uso, anexar aviso e aceitar restrição remota pelo Google: **não é licença permissiva** [C §3.3]. |
| granite-embedding-311m-multilingual-r2 | 312 M | Apache-2.0 | 0,607 | ONNX int8 oficial de 313 MB. |
| BGE-M3 | ~568 M | MIT | 0,635 | Forte e pesado; 54,6 no MMTEB na tabela dos autores do Bekko. |

Um segundo benchmark, o [MTEB-PT](https://arxiv.org/abs/2607.04071), conclui que rankings
multilíngues não preveem bem o desempenho em português. A escolha final continua saindo do
spike S1 com o conjunto dourado; o teste local ficou para o computador do usuário [C §3.5].

### 5.3 Imagens

- Para capturas de tela, que são quase sempre texto, OCR + embedding de texto é mais barato e
  competitivo. Um estudo de 2025 encontrou pipelines com OCR à frente do ColPali em todos os
  níveis de qualidade de documento ([arXiv 2505.05666](https://arxiv.org/abs/2505.05666)).
- ColPali/ColQwen guardam muitos vetores por página; custo de disco e latência altos para um
  acervo pessoal de centenas de milhares de imagens.
- Embedding visual de vetor único (família CLIP/SigLIP) faz sentido para busca por aparência
  ("aquele gráfico azul"). Com ONNX e licença permissiva: SigLIP 2 base (Apache-2.0, conversão
  da comunidade de 378 MB em int8), CLIP ViT-B/32 (MIT, 154 MB em int8) e uform3 multilíngue
  (Apache-2.0, 87 MB o codificador de imagem; usado pelo Windrecorder). Evidência de que servem
  para capturas de tela é fraca: o CLIP sem ajuste fino teve recall@10 pouco acima de 0,2 em
  busca de telas de app ([GUing](https://arxiv.org/abs/2405.00145)); SigLIP em telas: não
  encontrado. Continua como spike da Fase 4 [C §6.1].
- Formato: AVIF ficou ~34% menor que JPEG num teste com quadros de TV, mas custou ~13× mais CPU
  para codificar que WebP ([GDELT](https://blog.gdeltproject.org/at-scale-ocr-of-television-news-experiments-optimizing-the-still-frame-file-storage-format/));
  estudo do efeito do formato na qualidade do OCR de telas: não encontrado [C §6.2].

### 5.4 Captura no Windows

- **Desktop Duplication (DXGI)** entrega, por quadro, a lista de retângulos alterados; quadro
  sem retângulos é sinal gratuito de "nada mudou" ([Microsoft](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_2/nf-dxgi1_2-idxgioutputduplication-getframedirtyrects)). A duplicação cai em troca de sessão e precisa ser recriada.
- **Windows.Graphics.Capture** ganhou `DirtyRegionMode` e `MinUpdateInterval` no Windows 11
  24H2 (build 26100). Confirmado na referência: os valores `ReportOnly` e `ReportAndRender` e o
  `MinUpdateInterval` não têm descrição nenhuma
  ([enum](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture.graphicscapturedirtyregionmode)).
  Há "essentially no documentation" segundo um
  [issue do exemplo oficial](https://github.com/robmikh/Win32CaptureSample/issues/82); o libwebrtc
  calcula a própria região alterada em vez de usar a API [C §7.1].
- O Recall salva uma captura "every few seconds and when the content of your active window
  changes" ([Microsoft](https://support.microsoft.com/en-us/windows/retrace-your-steps-with-recall-aa03f8a0-a78b-4b3e-b0a1-2eb8ac48701c)):
  a regra de mudança está confirmada.
- **UI Automation tem custo fora do Aura:** um processo cliente de UIA faz Chrome e Firefox
  manterem a árvore de acessibilidade completa; um relato com rastreamento mediu 43 s de entrada
  travada no Chrome ([issue](https://github.com/anthropics/claude-code/issues/44739)). Apps
  Electron só expõem texto quando detectam tecnologia assistiva. Janelas com
  `WDA_EXCLUDEFROMCAPTURE` (como o Signal) escondem os pixels, não a árvore de UIA [C §7.2].

### 5.5 Obsidian

- Um cofre é uma pasta de arquivos Markdown. Escrever nessa pasta funciona com o Obsidian
  fechado e sem plugin.
- **CLI oficial** desde a versão 1.12.0 (fevereiro de 2026): `obsidian search query="…"`,
  `create`, `append`, `prepend`, saída em JSON. Liga-se em Configurações → Geral. Confirmado na
  ajuda: "requires the Obsidian app to be running"; o primeiro comando abre o app
  ([ajuda](https://obsidian.md/help/cli)). Sem o app, só o Obsidian Headless (beta), que
  sincroniza e publica mas exige conta do Obsidian Sync. Há defeitos relatados com agentes:
  `property:set` sai com sucesso sem mudar arquivos escritos por fora, e códigos de saída 0 em
  falhas ([fórum](https://forum.obsidian.md/t/113166)). Serve como atalho opcional, não como
  dependência.
- Propriedades no cabeçalho YAML alimentam Bases e Dataview; o tipo de uma propriedade vale para
  o cofre inteiro ([ajuda](https://obsidian.md/help/properties)). Sintaxe confirmada na CLI:
  `property:set name=… value=… type=… path=…`, `bases`, `base:views`,
  `base:query path=… view=… format=json`, `base:create` [C §7.3].
- **Obsidian Sync** mescla conflitos com diff-match-patch e pode duplicar texto; nota criada
  localmente pode ser trocada pela remota sem mesclar; cofre em `Desktop` ou `Documents` costuma
  estar no OneDrive ([ajuda](https://obsidian.md/help/sync/troubleshoot)). Isso muda como o Aura
  escreve no Cofre [C §10, I-14].

## 6. O que a pesquisa não respondeu (revisado)

1. **Opiniões diretas.** Fórum do Obsidian, Tildes e Hacker News: respondido, lidos direto
   ([C §1–2]). Reddit: **continua sem resposta**. Tentativas: `reddit.com` e `old.reddit.com`
   (403 para esta rede), leitor Jina (403), PullPush (526), buscador da Anthropic (domínio
   recusado), agent-reach (exige login), Internet Archive (fora do ar; cópias sem comentários).
2. **Embeddings em português.** Respondido em parte: o MTEB-BR e o MTEB-PT (julho de 2026) têm
   números independentes (seção 5.2). Bekko não foi avaliado em português; o teste local com 50
   documentos e 30 perguntas **não rodou** e ficou para o computador do usuário [C §3.5].
3. **Custo de CPU da captura por evento com UIA no Windows.** Não encontrado como medição. Há
   só a estimativa da especificação do screenpipe (200–500 ms para percorrer a árvore no
   Windows contra 10–50 ms no macOS) [autodeclarado] e um risco novo: ser cliente de UIA deixa
   os navegadores mais lentos (seção 5.4). Continua no spike S3, que passa a medir também o
   efeito nos navegadores.
4. **Formato de disco e deduplicação do screenpipe e do Recall.** screenpipe: respondido no
   código (JPEG q80 por captura, compactado em MP4 depois de 10 min; deduplicação por hash do
   texto de acessibilidade com exceções; SQLite com FTS5 e sqlite-vec) [C §4.1]. Recall:
   respondido em parte (captura a cada poucos segundos e quando a janela muda; cifra por captura
   num enclave; AES-256-GCM por página segundo o TotalRecall); o formato do banco e a regra de
   deduplicação não são públicos [C §4.2].
5. **Consentimento de terceiros.** Respondido em parte: a LGPD não se aplica a tratamento "por
   pessoa natural para fins exclusivamente particulares e não econômicos" (art. 4º, I); uso com
   clientes é econômico e exige base legal (art. 7º) e transparência. Granola, Fathom e
   Fireflies oferecem aviso no chat, marca d'água ou e-mail; o Otter é processado por não pedir
   a todos [C §9]. Orientação da ANPD específica: não encontrada. Continua precisando de
   advogado antes de qualquer promessa na interface (spike S6). **Isto é pesquisa, não
   aconselhamento jurídico.**
