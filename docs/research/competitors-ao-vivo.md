# Estudo de concorrentes 2 — copilotos ao vivo e memória de reuniões

Data: 2026-10-05. Complementa `competitors.md` (que cobre overlays e assistentes de desktop de código aberto). Este estudo cobre o outro lado do mercado: assistentes que **ouvem a conversa e ajudam durante ela** (reuniões, entrevistas, vendas) e os que **guardam memória de reuniões**.

> Método e limite: pesquisa na web (resenhas, páginas de ajuda e notas de lançamento indexadas). Os sites oficiais de vários produtos não puderam ser abertos nesta sessão (bloqueio de rede), então preços e recursos abaixo são os **relatados pelas fontes listadas no fim** e podem ter mudado. Confirmar antes de qualquer decisão comercial. Nada aqui é aconselhamento jurídico.

## 1. Classificação (a tabela do pedido, ampliada)

"Quão direto é" = proximidade com o que o Aura quer ser (assistente residente que vê a tela, ouve o áudio e responde na hora).

| Produto | Quão direto | Foco | O que faz bem (copiar a ideia) | Lacuna que o Aura explora |
| --- | --- | --- | --- | --- |
| **Cluely** | 🔴 Muito direto | Overlay que vê a tela e ouve mic + áudio do sistema; respostas e tópicos em tempo real | Atalho único (`Ctrl+Enter`) para "me ajude agora"; sem bot na chamada; base de conhecimento própria (roteiros, material de estudo); definição de termos ao vivo; transcrição + resumo + tarefas depois; modo Widget discreto | Assinatura própria (a partir de ~US$ 20/mês, camada "indetectável" mais cara); nuvem fechada; reputação ligada a "colar" em entrevistas; sem agente que **executa** o pós-reunião |
| **Fireflies.ai Live Assist** | 🔴 Muito direto | Sugestões, respostas, coaching e transcrição durante a reunião (web, Chrome, desktop, celular) | Botões prontos: "Perdi o fio (último minuto)", "Perguntas de follow-up", "Resumo até agora", "Ações"; pontos de pauta que aparecem sozinhos; respostas que consultam reuniões e documentos antigos; "Meeting Prep" (resumo antes de reuniões recorrentes) e resumo diário | Plataforma de equipe/nuvem; foco em vendas; pouca visão de tela |
| **Final Round AI** | 🔴 Direto (entrevistas) | Copiloto de entrevista + ensaio | Transcreve a pergunta do entrevistador e sugere resposta numa janela lateral; plataformas de código (LeetCode, HackerRank, CodeSignal); **entrevistas simuladas** (vídeo, áudio ou chat) com feedback no formato STAR e ritmo; banco de perguntas | Caro no mensal (relatado ~US$ 150 mensal vs ~US$ 25/mês no anual); plano grátis sem o copiloto ao vivo |
| **LockedIn AI** | 🔴 Direto (entrevistas) | Copiloto técnico (código, design de sistemas) | App desktop; ensaio com dificuldade adaptativa; opção de **especialista humano** em tempo real | Créditos/tempo limitado; recursos furtivos só nos planos altos (~US$ 50/mês) |
| **Sensei AI** | 🔴 Direto (entrevistas) | Respostas rápidas e contextualizadas; preparo comportamental/liderança | **Detecção automática da pergunta** (sem digitar nada no meio da entrevista); feedback de histórias no formato STAR | Só navegador (menos discreto, menos integrado ao SO) |
| **Verve AI** | 🟠 Direto/parcial | Coaching e respostas estruturadas | Coaching de **tom de voz e confiança** em tempo real; modo furtivo em todos os planos | Baseado em navegador; foco estreito em entrevista |
| **Granola** | 🟠 Parcial | Notas e memória de reuniões **sem bot** (captura o áudio do sistema) | Você anota rascunhos, a IA **melhora** as notas depois; *Templates* por tipo de reunião (1:1, discovery, stand-up, pitch, entrevista de usuário); *Recipes* (prompts salvos "antes / durante / depois"); *Briefs* (2–3 pontos antes de cada reunião, a partir de reuniões, e-mails e web); *Spaces* (pastas compartilhadas com consulta em conjunto); API pública e servidor MCP; app Windows | Não ajuda **durante** a conversa (é caderno, não copiloto); sem tela; sem ação (não executa tarefas); resenhas criticam notas "bonitas, mas difíceis de confiar" (falta de rastreio até a fala original) |
| **Fathom** | 🟡 Adjacente | Transcrição, resumos, ações, coaching de reuniões | Resumos com 15+ modelos (BANT, Sandler…); clipes e playlists de trechos; "Ask Fathom"; CRM; plano grátis generoso | Pós-reunião; sem copiloto ao vivo; limites de IA no grátis (5 reuniões/mês para recursos avançados) |
| **Otter.ai** | 🟡 Adjacente | Agente de reuniões | Transcrição ao vivo e chat durante a reunião; marca ações no meio da chamada; **executa** tarefas (agenda follow-up, envia e-mail); CRM | Nuvem; assinatura; sem tela |
| **Read AI** | 🟡 Adjacente | Inteligência de reuniões | Métricas de comunicação (ritmo, muletas, tempo de fala, engajamento); busca ("Ask Read") que cruza reuniões, e-mails e mensagens | Pós-reunião, muito orientado a equipe/empresa; métricas subjetivas ("carisma", "viés") são discutíveis e podem soar invasivas |
| **Gong** | 🟡 Concorrente no nicho de vendas | Inteligência de receita | Placar de chamadas, *deal boards* por risco; desde fev/2026 agentes que **avaliam chamadas ao vivo** e fazem **roleplay com personas de compradores** | Enterprise/vendas; fora do alcance de quem usa sozinho |

### Produtos de fora da tabela que valem como referência

| Produto | O que importa para o Aura |
| --- | --- |
| **Yoodli** | Roleplays por voz com personas (vendas, negociação, entrevista) e feedback em ritmo, muletas, concisão, tempo de fala × escuta. Modelo para o **Ensaio** do Aura. |
| **Wispr Flow** | Ditado com edição por IA: *Modo Comando* (selecione o texto e diga "deixe mais formal"), snippets por voz, dicionário pessoal que aprende com as suas correções, ajuste por app (e-mail × chat × código). Já mapeado como CAND-025. |
| **Krisp** | Captura sem bot, supressão de ruído **local**, transcrição/resumo e tradução de sotaque (experimental). Lição: qualidade do áudio decide a qualidade da nota. |
| **Rewind / Limitless** | A Meta comprou a Limitless (dez/2025) e o Rewind desligou a captura de tela e áudio em 19/12/2025: o espaço de "memória pessoal de tela" ficou **vazio** para quem quer controle local. screenpipe é o principal sobrevivente (licença *source-available*). |
| **Pickle Glass, Open-Cluely e similares** | Clones de código aberto do Cluely. Provam a demanda por alternativa gratuita e privada; **verificar a licença** antes de reaproveitar qualquer código (AGENTS.md: só permissivas). |
| **Meetily, Noota e outros "sem bot"** | Reforçam que "sem bot" virou requisito básico — e que **bot invisível não substitui consentimento** (ver §4). |

## 2. O que todos os bons produtos têm em comum (requisitos básicos)

1. **Captura sem bot** do áudio do sistema + microfone (o participante não vê ninguém entrar).
2. **Transcrição ao vivo** com rótulo de quem falou.
3. **Notas estruturadas** por tipo de reunião (modelos / receitas), não só um resumo genérico.
4. **Ações pós-reunião**: tarefas com dono e prazo, e-mail de follow-up.
5. **Conversar com as reuniões** (busca e perguntas entre várias reuniões).
6. **Preparo antes** (briefing a partir do histórico) e **ensaio** (prática com feedback).
7. **Integrações** (calendário, CRM, gerenciador de tarefas) — hoje cada vez mais via **MCP/API**.

Tudo isso é replicável no Aura sem inventar tecnologia nova; a diferença está em **como** (§3).

## 3. Onde o Aura pode ser claramente melhor

| Eixo | Concorrentes | Aura |
| --- | --- | --- |
| **Custo** | Assinatura própria por cima de tudo (US$ 15–50+/mês) | Usa o plano ChatGPT que o usuário já paga, ou a chave dele (BYOK); sem mensalidade do Aura |
| **Privacidade** | Nuvem do fornecedor guarda áudio e transcrição | Local-first: áudio cifrado no disco, ASR local, política de privacidade por fonte, registro de acesso do que foi enviado ao modelo, modo "só local" com modelo local |
| **Escopo** | Cada produto faz *uma* coisa (reunião **ou** entrevista **ou** ditado) | Um único assistente residente: reunião, ditado, tela, arquivos, agente — o contexto se soma |
| **Ação** | Resumem e sugerem | É um agente: pode **fazer** o trabalho depois (minuta, planilha, e-mail, tarefa) com aprovação e sandbox |
| **Visão** | Quase só áudio (Cluely vê a tela, mas sem histórico) | Tela e áudio juntos, **com histórico** (buffer recente): "o que ele quis dizer com *esse número*?" resolve com o slide daquele minuto |
| **Retroativo** | Precisa iniciar antes | "Esqueci de iniciar": transforma os últimos N minutos do buffer em reunião |
| **Controle** | Ligam sozinhos pelo calendário | Reunião é **opt-in**, com indicador visível, pausa e fim claros |
| **Configuração** | Telas de opções e modelos prontos | A própria IA entrevista o usuário e configura (ver `docs/project/plano-reuniao-e-configuracao-assistida.md`) |

## 4. Riscos de mercado e de reputação

- **Entrevistas "indetectáveis".** Cluely, Final Round, LockedIn, Sensei e Verve vendem, em maior ou menor grau, ajuda que o entrevistador/proctor não percebe. Isso viola regras de muitas empresas e plataformas e é o principal ponto de crítica da imprensa ao Cluely. O Aura **já** exclui o Overlay de capturas e compartilhamento por **privacidade** (decisão do esforço 001), mas não deve virar produto de evasão: ver decisão Q-006 no plano.
- **Consentimento de gravação.** Um bot visível **não** é consentimento, e "sem bot" não muda a lei: a licitude depende das regras da jurisdição (em vários países/estados todas as partes precisam concordar) e de a LGPD/GDPR exigirem base legal e transparência para dados pessoais. Por isso o plano trata o aviso aos participantes como recurso de primeira classe.
- **Confiança nas notas.** Resumos que parecem corretos mas não apontam de onde vieram são a crítica mais comum ao Granola. Toda nota/ação do Aura deve apontar para o trecho da transcrição.
- **Limites do plano ChatGPT.** Reunião longa + resumo contínuo pode consumir o plano rápido; o desenho precisa de modo econômico e medidor (ver plano §6).
- **Ecossistema móvel.** Cluely, Granola e Fireflies têm app de celular; o Aura é só Windows (não-objetivo declarado). Aceitável, mas comunicar.

## Fontes

- Cluely: [softwarefinder](https://softwarefinder.com/sales-tools/cluely), [tldv](https://tldv.io/blog/cluely-review/), [makerstack](https://makerstack.co/reviews/cluely-review/), [dupple](https://www.dupple.com/tools/cluely), [Wikipedia](https://en.wikipedia.org/wiki/Cluely)
- Fireflies: [Live Assist (blog)](https://fireflies.ai/blog/live-assist), [Guia Live Assist/Sales Assist](https://guide.fireflies.ai/articles/2679406774-live-assist-sales-assist-fireflies-desktop-app-real-time-notes-answers-suggestions), [Meeting Prep](https://guide.fireflies.ai/articles/3878280311-meeting-prep-get-briefed-before-every-meeting)
- Final Round AI: [tooldirectory](https://tooldirectory.ai/tools/final-round-ai), [3box](https://3box.ai/blog/final-round-ai-review-2026-is-it-worth-it), [shadecoder](https://www.shadecoder.com/blogs/final-round-ai-review-2026-features-pricing-honest-verdict)
- LockedIn / Sensei / Verve (as páginas de comparação são escritas por concorrentes entre si — tratar como indício, não como fato): [LockedIn × Verve](https://www.lockedinai.com/compare/lockedinai-vs-verve-copilot), [Sensei AI review](https://www.finalroundai.com/blog/sensei-ai-review), [InterviewCopilot × LockedIn](https://interviewcopilot.ai/blog/interviewcopilot-vs-lockedin-ai), [DEV: 10 copilotos](https://dev.to/finalroundai/the-10-best-interview-copilot-tools-for-2026-4a8j) (autoria de um concorrente — usar só para fatos verificáveis)
- Granola: [Recipes e templates](https://www.granola.ai/blog/meeting-recipes-repeatable-formats), [Briefs](https://www.granola.ai/blog/pre-meeting-brief-templates), [TechCrunch (Série C, mar/2026)](https://techcrunch.com/2026/03/25/granola-raises-125m-hits-1-5b-valuation-as-it-expands-from-meeting-notetaker-to-enterprise-ai-app/), [HappyScribe](https://www.happyscribe.com/blog/granola-ai-review)
- Fathom / Otter / Read AI: [Fathom (visão geral)](https://www.fathom.ai/overview), [Otter](https://otter.ai/), [Read AI](https://www.read.ai/meeting-reports), [Read AI: melhores assistentes](https://www.read.ai/articles/best-ai-meeting-assistants)
- Gong: [claap](https://www.claap.io/blog/what-is-gong-software), [growthcentr](https://www.growthcentr.com/how-gong-conversation-intelligence-ai-coaching-works/)
- Yoodli: [tooldirectory](https://tooldirectory.ai/tools/yoodli), [makerstack](https://makerstack.co/reviews/yoodli-review/)
- Wispr Flow: [Recursos](https://wisprflow.ai/features), [eesel](https://www.eesel.ai/blog/wispr-flow-overview)
- Krisp: [PR Newswire (accent conversion)](https://www.prnewswire.com/news-releases/krisp-ai-note-taker-raises-the-bar-for-meetings-by-expanding-its-ai-meeting-suite-with-accent-conversion-302689460.html)
- Rewind/Limitless: [Limitless](https://help.limitless.ai/en/articles/9135527-screen-recording), [screenpipe](https://screenpipe.com/blog/best-rewind-ai-alternative-2026)
- Código aberto estilo Cluely: [Pickle Glass (resenha)](https://www.aibase.com/news/19492), [Open-Cluely](https://github.com/shubhamshnd/Open-Cluely)
- Consentimento: [Circleback](https://circleback.ai/blog/recording-consent-for-ai-meeting-notes), [tldv](https://tldv.io/blog/is-bot-free-recording-legal/), [recordinglaw.com](https://www.recordinglaw.com/us-laws/ai-meeting-recording-laws/)
