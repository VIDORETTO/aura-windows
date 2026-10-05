# Plano de produto — Reunião ao vivo e configuração assistida por IA

Data: 2026-10-05 · Status: **proposta** (não é spec; vira candidatos CAND-033…049 em `roadmap.md` e só vira esforço depois de passar pelo fluxo Hybrid) · Base: `docs/research/competitors.md`, `docs/research/competitors-ao-vivo.md`, `docs/HANDOFF.md`, `CONTEXT.md`, ADRs 0001–0009.

## 0. Resumo

**O que o Aura é:** assistente residente para Windows. Um atalho abre o Overlay sobre o app em uso; a conversa é com um agente de verdade (Codex app-server, plano ChatGPT ou chave própria) que vê tela e ouve áudio **somente** como a política de privacidade permite, entende voz (ASR local), lê anexos, usa Skills/MCP e pede aprovação antes de agir.

**O que o mercado mostra:** os concorrentes diretos (Cluely, Fireflies Live Assist, Final Round, LockedIn, Sensei, Verve) e os adjacentes (Granola, Fathom, Otter, Read, Gong) convergiram em: captura sem bot, transcrição ao vivo, notas por tipo de reunião, ações pós-reunião, busca entre reuniões, preparo/briefing e ensaio. Nenhum deles é, ao mesmo tempo, **local-first, sem mensalidade própria, com visão de tela e agente que executa**. É essa a vaga do Aura.

**Três apostas deste plano**

1. **Um padrão único: "Preparo em três velocidades".** Toda função que precisa de contexto do usuário abre com a mesma tela: **⚡ Explicar em uma frase** · **🧭 Grill-me completo** · **▶ Começar sem preparo**. A IA primeiro tenta descobrir sozinha (janela, calendário, anexos, memórias), só pergunta o que muda o resultado, nunca bloqueia (§2).
2. **Configurar com IA, em tudo.** O usuário descreve o que quer em linguagem natural e a IA propõe a mudança como um cartão "antes → depois", com **Aplicar** e **Desfazer**. O app deixa de depender de o usuário caçar opções (§3). Isto é a Fase 0 e entrega valor sem esperar o ASR.
3. **Reunião é um modo à parte, opt-in.** Nunca fica ligada sozinha; tem tela de preparo, painel ao vivo discreto, pós-reunião que **executa** o trabalho (agente) e uma biblioteca local. Diferenciais: "esqueci de iniciar" (retroativo), tela + áudio juntos, só-local, custo zero extra (§4).

**Ordem recomendada:** Fase 0 (Preparo + Configurar com IA + motor de Reunião) → Fase 1 (Reunião MVP, pós-reunião, consentimento) → Fase 2 (tradução ao vivo, calendário, Ensaio, ditado inteligente, Projetos) → Fase 3 (agendados, coaching, estudo) (§8).

**Revisão 2 (2026-10-05):** §12 trata de **ocultar o Aura em qualquer transmissão** (já implementado como opção, §12.1) e §13 reúne **funções dos concorrentes fora de reuniões** e §14 o **pacote "Dia a Dia"**: funções simples, sem código nem reunião, para o uso comum.

**Decisões que dependem de você:** §10 (Q-006 a Q-010), todas com recomendação.

---

## 1. Onde o Aura está e o que falta

### 1.1 Peças que já existem e que este plano reaproveita

| Já existe (onde) | Serve para |
| --- | --- |
| Buffer recente e gravação manual de microfone e áudio do sistema, cifrados, rotulados "Você"/"Sistema" (`aura-app/src/recorder.rs`, ADR 0004) | Base da transcrição de reunião e do **"esqueci de iniciar"** (salvar os últimos N min só reclassifica segmentos) |
| ASR local e em nuvem, VAD, parciais ao vivo, vocabulário (`aura-asr`, `voice.rs`) | Transcrição da reunião (precisa de um laço contínuo, hoje só há push-to-talk) |
| Formulários do agente: perguntas com opções e `form` (`overlay/userInput.ts`, `UserInputCard.tsx`) | Os cartões de pergunta do Preparo |
| Skills internas invisíveis + ferramentas MCP de escrita com aprovação (`core-skills/`, `aura-app/src/tools.rs`, esforço 017) | O molde de **"Configurar com IA"**: já funciona para Skills, Comandos rápidos e servidores MCP |
| Perfis de aplicativo, Memórias, Skills, Comandos rápidos, modos Chat/Tarefa/Plano | Receitas, ajuda por app, pós-reunião com agente em modo Tarefa |
| Política de privacidade, Janelas excluídas, Pausa, Registro de acesso, Overlay fora de capturas | Consentimento, "o que foi enviado ao modelo", slides sob controle |
| Minibar, notificações, TTS, "Inserir no app" | HUD compacto, lembretes, falar respostas, colar e-mail pronto |
| Anexos multimodais com `attachment_read` | Material de apoio da reunião, base de conhecimento |

### 1.2 O que não existe (e precisa ser construído)

Reunião como entidade (início, fim, briefing, notas) · transcrição **contínua** de longa duração · "quem falou" além de Você/Sistema · índice de busca das transcrições (FTS5) · Receitas (modelos de nota) · calendário · agendador · assistente de configurações (`settings_*`) · tela de Preparo · áudio **por aplicativo** (hoje o loopback pega tudo o que toca) e supressão de eco.

### 1.3 Observações que condicionam o plano

- **Os motores de voz locais nunca foram compilados no Windows** (HANDOFF §7, item 4). Reunião com ASR local depende disso; a nuvem (BYOK) é o plano B e tem custo de privacidade. Por isso a **Fase 0 começa por Preparo + Configurar com IA**, que não dependem de ASR.
- `docs/discovery.md` e `vision.md` listam "assistente de reuniões" como **não-objetivo da V1**; o roadmap já tem CAND-016/026. Este plano propõe promover reunião a objetivo **pós-V1 (0.3)** — requer atualizar `vision.md` quando a spec for aceita.
- O orçamento "ocioso ≤ 150 MB / 0,5% CPU" continua valendo. Reunião ativa terá **orçamento próprio** (hipóteses H-019/H-020), porque o worker de ASR fica quente (ADR 0005).

---

## 2. O padrão central: Preparo em três velocidades

### 2.1 Definição

**Preparo**: momento curto, antes de uma função que depende de contexto, em que o Aura entende o que o usuário quer. Três caminhos, sempre visíveis:

- **⚡ Explicar em uma frase** — um campo (digitar ou segurar `Ctrl+Space`). A IA devolve o cartão **"Entendi assim"**.
- **🧭 Grill-me completo** — entrevista guiada. Percorre a árvore de decisões do tema, **uma pergunta por vez, sempre com a resposta recomendada**, e antes de perguntar tenta descobrir sozinha o que puder. É o mesmo método que o `hybrid-discover` usa com o desenvolvedor, agora voltado ao usuário final.
- **▶ Começar sem preparo** — usa padrões sensatos e começa já. O preparo pode ser completado depois, sem pressa.

### 2.2 Tela (esboço)

```
╭──────────────────────────────────────────────────────────────╮
│ ◎  Nova reunião                                          ✕   │
│ Vi: Microsoft Teams · "Revisão Q3" · começou há 1 min          │
│                                                              │
│  ⚡ Explicar em uma frase      🧭 Grill-me completo (~3 min)   │
│     digite ou segure Ctrl+Space   eu pergunto, você responde  │
│                                                              │
│  ▶ Começar agora, sem preparo          Usar "1:1 semanal" ▾  │
╰──────────────────────────────────────────────────────────────╯
```

```
╭ Pergunta 3 de ~6 ──────────────────────────── Pular o resto ╮
│ O que você precisa levar desta reunião para dizer que valeu? │
│  ◉ Uma decisão sobre o corte de 10%        ← recomendada     │
│  ○ Alinhar expectativas, sem decisão                         │
│  ○ Descobrir as objeções do Bruno                            │
│  ○ Outro… (digite ou fale)                                   │
╰──────────────────────────────────────────────────────────────╯
```

```
╭ Entendi assim ───────────────────────────────────────────────╮
│ Objetivo       Fechar o orçamento de mídia do Q3           ✎ │
│ Sair com       Decisão sobre o corte de 10% e um dono      ✎ │
│ Pontos         1 Resultados · 2 Orçamento · 3 Próximos     ✎ │
│ Cuidado        Não prometer prazo sem falar com o Bruno    ✎ │
│ Inferido       Participantes: Ana, Bruno (título da janela)✎ │
│ Ajuda ao vivo  Sob demanda · Receita: Reunião de decisão   ▾ │
│ [Começar]   [Ajustar com a IA]   [Salvar como Receita]       │
╰──────────────────────────────────────────────────────────────╯
```

### 2.3 Política de perguntas (quando a IA conversa e quando não)

A IA conversa **caso ache necessário e possa**. Cada pergunta passa por quatro filtros, nesta ordem:

1. **Dá para descobrir sozinha?** Título/processo da janela, calendário (quando houver conector), anexos, memórias, reuniões anteriores com o mesmo título, Perfil do app. Se sim, **não pergunta**: mostra "Inferido: …" com ✎ para corrigir.
2. **A resposta muda o resultado?** Objetivo, o que precisa sair da conversa, papéis, riscos, tom, idioma. Se não muda, assume o padrão e registra.
3. **Dá tempo?** Reunião começa em < 2 min (calendário), usuário clicou "Começar agora" ou está falando por voz → **não bloqueia**; no máximo uma pergunta depois, dentro do painel, descartável ("não perguntar de novo para este tipo").
4. **É a melhor forma?** Pergunta curta, com opções e a recomendada; "Outro…" aceita voz. Máximo **2** perguntas no caminho Rápido, ~**8** no Completo. Nunca repete o que já foi respondido em outro Preparo: "Da última vez você disse *X*. Mantém?".

### 2.4 Regras de ouro

1. Inferir antes de perguntar. 2. Uma pergunta por vez, com resposta recomendada. 3. **Nunca bloquear**: "Começar agora" sempre visível. 4. Dizer o que entendeu e deixar editar. 5. Lembrar (Receita/Perfil/Projeto) e oferecer reaproveitar. 6. Tempo limitado e visível ("Pergunta 3 de ~6", "Pular o resto"). 7. Tudo reversível e auditável. 8. Linguagem do usuário, sem jargão. 9. Teclado e voz de primeira classe. 10. Nada sai da máquina sem Chip/aprovação (princípio do `ui-ux.md`).

### 2.5 Onde o padrão se aplica

| Função | ⚡ Rápido | 🧭 Completo (grill-me) | O que fica salvo |
| --- | --- | --- | --- |
| **Reunião** | Uma frase; tipo e participantes inferidos | Objetivo, o que precisa sair, papéis, pontos obrigatórios, pontos sensíveis, material, estilo de ajuda, o que fazer depois | Briefing (+ Receita opcional) |
| **Ensaio** (entrevista, vendas, apresentação) | Cole a vaga/diga o cenário | Seu histórico, pontos fracos, dificuldade, tipo de entrevistador, o que treinar | Cenário + rubrica de feedback |
| **Ditado inteligente** | "Como você escreve?" (Formal / Direto / Amigável) ou cole um texto seu | Contextos (e-mail, chat, código), assinatura, idiomas, vocabulário | Estilo, dicionário, snippets, ajustes por app |
| **Perfil de aplicativo** | "Criar perfil para este app" (infere do app e do uso) | O que você faz nele, formato de resposta | Perfil |
| **Projeto** | Nome; a IA sugere reuniões e arquivos | Objetivo, pessoas, glossário, instruções | Projeto |
| **Agente agendado** | "Todo dia às 8h, me diga…" → proposta + prévia da 1ª execução | Fontes, formato, destino, quando não rodar | Agendamento |
| **Conector** (calendário, e-mail…) | "Conectar meu calendário" → fluxo guiado | Permissões mínimas, o que ler e o que escrever | Servidor MCP (segredo só no cofre) |
| **Primeiros passos / Privacidade** | 3 perfis: Cautelosa · Equilibrada · Máxima ajuda | Apps sensíveis, dados de terceiros, compartilha tela?, reuniões? | Política, Janelas excluídas, atalhos, modelo de voz |
| **Provedor com chave própria** | "Tenho uma chave do Groq" → predefinição + teste | Custo × privacidade × velocidade → recomenda modelos | Provedor + modelo padrão (chave só no campo seguro) |

### 2.6 Como implementar (reuso)

- **Cartões:** reutilizam `questions`/`form` do Overlay; novo apenas o cartão "Entendi assim" (editável) e o seletor das três velocidades.
- **Cérebro:** uma Skill interna `aura-preparo` (invisível, como as de `core-skills/`) com o método e o formato de saída; Skills filhas por tema (`aura-preparo-reuniao`, `-ensaio`, …). Sem código novo de "entrevistador": é instrução.
- **Gravação:** ferramentas MCP `*_save` do domínio (ex.: `meeting_brief_save`) com aprovação quando alteram algo persistente fora da sessão.
- **Modo da conversa:** o Preparo roda em Chat (só lê e pergunta); só entra em Tarefa quando o resultado exigir gravar arquivos.
- **Memória:** respostas úteis e duráveis do Preparo viram propostas de Memória (revisáveis), nunca gravadas em silêncio.

---

## 3. Configurar com IA, em tudo

### 3.1 Por onde o usuário chega

- **Primeiros passos:** "Conte como você trabalha" (⚡ uma frase ou 🧭 grill-me) → a IA propõe privacidade, modelo de voz (já recomenda pelo hardware), atalhos, perfis e Comandos rápidos de uma vez.
- **Busca das Configurações** (`Ctrl+K`): quando não acha, o resultado final é **"Pedir à IA"**.
- **`/configurar`** no Overlay e um botão ✨ no topo de cada página das Configurações.
- **Sugestões contextuais** (CAND-028): "Você abriu o Teams: criar um perfil de reunião?".

### 3.2 O que a IA pode configurar

| Área | Exemplo de pedido | O que a IA propõe |
| --- | --- | --- |
| Privacidade | "Não grave nada quando eu abrir o app do banco" | Nova Janela excluída (processo e título, a partir das janelas abertas) |
| Captura | "O microfone só quando eu segurar o atalho" | Microfone em *Sob demanda* |
| Voz | "Dito em português e inglês" | Modelo recomendado ao hardware + idiomas + vocabulário |
| Perfis | "No VS Code, respostas curtas em TypeScript" | Perfil de aplicativo |
| Provedores | "Quero usar o Groq" | Predefinição, modelo e teste de conexão; **a chave é digitada num campo seguro, a IA nunca a vê** |
| Atalhos | "Conflita com o PowerToys" | Alternativa livre e conferida |
| Aparência | "Mais discreto" | Opacidade, modo compacto, cor neutra |
| Memórias | "Organize minhas memórias" | Mesclar duplicadas e remover contraditórias (diff) |
| Reunião | "No Teams, ajuda só sob demanda e apague o áudio no fim" | Perfil/Receita de reunião |
| Retenção | "Guarde gravações por 3 dias" | Retenção por dias e espaço |
| Skills, Comandos, MCP | (já existe "Criar com IA") | Passa a usar o mesmo cartão de diff |

### 3.3 Mecânica e proteções

- Ferramentas MCP do Aura: `settings_describe` (somente leitura: valores atuais, opções válidas e descrição de cada item — derivadas do tipo `Settings` para não ficar defasadas; teste de paridade como o do i18n), `settings_propose` (valida e devolve o **diff**, não grava), `settings_apply` (aprovação), `settings_undo` (instantâneos).
- Cartão **antes → depois** com **[Aplicar] [Ajustar] [Cancelar]**; depois de aplicar, **[Desfazer]** por 30 s e "Desfazer última mudança da IA" nas Configurações.
- Nunca: tocar segredos/credenciais; ligar o modo YOLO (continua exigindo digitar ACEITO); apagar dados; mexer em atalhos sem checar conflito.
- Mudança que **amplia** captura ou permissão do agente (ex.: tela *Contínua*) pede confirmação reforçada e explica o efeito; mudanças que **reduzem** aplicam com um clique.
- Tudo entra no Registro de acesso; a IA vê só metadados das Configurações.

Exemplo:

> **Você:** não quero que o Aura veja nada quando eu mexer no Nubank nem no KeePass.
> **Aura:** Achei 2 janelas abertas que combinam: *Nubank — Chrome* e *KeePass*. Proponho excluí-las pelo **processo** (assim vale para qualquer janela do KeePass) e pelo **título** no navegador.
> `Janelas excluídas  + KeePass.exe   + título "*Nubank*" (chrome.exe)`  **[Aplicar] [Ajustar]**

### 3.4 Aprender com o uso (P2)

Padrões locais e opt-in viram sugestões ("Você anexa a tela toda vez que abre o Excel — criar um perfil?", "Você recusa o microfone em reuniões do banco — excluir essa janela?"). Sempre como proposta, nunca silencioso.

---

## 4. Reunião (modo à parte)

### 4.1 Princípios

1. **Opt-in e visível.** Só começa por ação do usuário; indicador REC no HUD, na bandeja e no Overlay; fim claro. Nunca liga captura por conta própria (AC-M01).
2. **Separado do Overlay.** O Overlay continua leve para perguntas rápidas; a Reunião tem painel próprio.
3. **Silencioso por padrão** (princípio 4 do `ui-ux.md`): transcreve e anota sem avisos; ajuda quando pedido.
4. **Cada afirmação aponta de onde veio** (trecho da transcrição). Resposta ao "Granola: notas bonitas, difíceis de confiar".
5. **Local-first**: áudio cifrado, ASR local quando possível, só texto vai ao modelo.

### 4.2 Jornada

```
Entrada ──► Preparo ──► Briefing ──► Captura ──► Ao vivo ──► Fim ──► Pós-reunião ──► Biblioteca
 │            3 caminhos   "Entendi    mostra o     painel      confirma   resumo, ações,   pergunte
 │                         assim"      que ouve     discreto    ou sugere  e-mail, agente   depois
 └ retroativo: "Esqueci de iniciar" cria a Reunião a partir do buffer recente
```

**Entradas (todas explícitas):** botão "Reunião" no Overlay · `Ctrl+Shift+Alt+M` (atalho próprio, configurável; `Ctrl+Shift+M` muta o Teams) · `/reuniao` · bandeja › Iniciar reunião · **sugestão discreta** quando o app anterior é Teams/Zoom/Meet e há áudio (um aviso, descartável, "nunca para este app"; respeita o modo Foco; **não inicia nada**).

**Início sem preparo:** "Começar agora" inicia na hora. Depois de ~3 min sem Briefing, o painel mostra **uma única** linha "Quer me contar o objetivo? Ajudo melhor." (descartável, lembra a escolha).

**Retroativo:** com Buffer recente de áudio ligado, "Esqueci de iniciar → salvar os últimos N min como reunião" cria a Reunião com transcrição completa; o Preparo pode ser feito **depois** para refinar o resumo.

**Fim:** botão ■, ou a IA sugere ("Parece que acabou — encerrar?") por fim de evento do calendário ou silêncio prolongado; nunca encerra sozinha.

### 4.3 Painel ao vivo (HUD)

Janela própria (`meeting`): encaixável na lateral (≤ 360 px), excluída de capturas, opacidade ajustável, opção click-through; no Overlay aparece só a linha "● Reunião 23:41".

```
╭ Reunião · Revisão Q3 ·  ● 23:41 ───────────────── ⏸  ■ ╮
│ Pauta   ✔ Resultados   ▶ Orçamento   ○ Próximos passos     │
│ ─ Ao vivo ───────────────────────────────────────────────  │
│ Eles:  …e se cortarmos 10% do orçamento de mídia?          │
│ Você:  ▍                                                   │
│ ┌ Perguntaram a você ───────────────────────────────────┐  │
│ │ "Qual o impacto no CAC?"        [Rascunhar resposta]  │  │
│ └───────────────────────────────────────────────────────┘  │
│ [Perdi o fio] [Resumo] [O que respondo?] [Termo] [★ Marcar] │
│ Suas notas  ________________________________________       │
╰──────────────────────────────────────────────────────────╯
```

**Ações ao vivo** (todas sob demanda; no painel ao vivo, `Ctrl+Enter` = "me ajude agora", como no Cluely):

| Ação | Faz | Inspiração |
| --- | --- | --- |
| Perdi o fio | Resume o último 1–2 min | Fireflies "Catch up" |
| Resumo até agora | Pontos, decisões e pendências por enquanto | Fireflies |
| O que respondo? / Rascunhar resposta | Usa a pergunta detectada, o Briefing e a base de conhecimento | Cluely, Final Round, Sensei |
| Boas perguntas agora | Follow-ups coerentes com a pauta | Fireflies |
| Termo / sigla | Explica a seleção ou a última fala | Cluely "Define" |
| ★ Marcar momento (e nota por voz) | Destaca o trecho para o pós-reunião | Granola, Fathom |
| O que falta na pauta | Mostra itens não cobertos e o tempo | Fireflies talking points |
| O que está na tela? | Captura o slide (política) e lê com OCR | Cluely |
| Traduzir / legendas | Última fala ou legendas contínuas | novo (Fase 2) |
| Checar fato | Busca web do agente | novo |
| "Ação: …" | Registra tarefa por voz ou texto | Otter |

**Níveis de ajuda:** *Silencioso* (só transcreve e anota) · **Sob demanda** (padrão) · *Equilibrado* (avisa só quando fazem uma pergunta a você ou quando um item da pauta ficou de fora faltando 5 min) · *Ativo* (sugestões contínuas). Muda por IA, pela Receita ou no painel.

### 4.4 Pós-reunião

```
╭ Revisão Q3 · 42 min · 3 pessoas ───────────────────────────╮
│ Resumo        cada frase cita o trecho  ⟶ 12:31             │
│ Decisões · Ações (dono · prazo) · Sem resposta · Suas promessas │
│ [Rascunhar e-mail]  [Criar tarefas]  [Gerar minuta (agente)]  │
│ [Perguntar sobre esta reunião]  [Exportar ▾]  [Apagar áudio]  │
╰────────────────────────────────────────────────────────────╯
```

- **Notas melhoradas** (Granola): a IA combina o que o usuário anotou com a transcrição, no formato da **Receita** (§4.5).
- **Ações com dono e prazo**, cada uma ligada ao trecho de origem.
- **"Sem resposta":** perguntas feitas e não respondidas. **"Suas promessas":** o que *você* se comprometeu a fazer (e o que prometeram a você).
- **O agente executa** (diferencial): em modo Tarefa, com aprovação, produz ata, minuta/proposta em DOCX, planilha, apresentação, issue ou rascunho de e-mail. Concorrentes param no resumo.
- **Debrief de 30 s:** 3 perguntas rápidas ("o que foi bem? o que mudar?") alimentam Memória e coaching.

### 4.5 Receitas (modelos por tipo de reunião)

Receita = Skill + modelo de notas + ações padrão + estilo de ajuda ao vivo, com fases **Antes / Durante / Depois** (como no Granola). Nascem de 8 embutidas — 1:1, Stand-up/Daily, Cliente/Discovery, Chamada de vendas, Entrevista (como entrevistado), Entrevista (como entrevistador), Reunião de decisão, Aula/Treinamento — e a IA **cria e ajusta** novas ("Receita para minhas reuniões com fornecedores"). Formato aberto (agentskills.io), exportável.

### 4.6 Biblioteca e "Pergunte"

Reuniões guardadas localmente com busca (FTS5) e filtros por pessoa/projeto/Receita. O agente ganha `meeting_search`: "o que combinamos com a Ana em agosto?" responde com chips de origem (reunião, minuto). Projetos (CAND-027) agrupam reuniões + arquivos + instruções.

### 4.7 Privacidade e consentimento

| Tema | Decisão proposta |
| --- | --- |
| Aviso aos participantes | Botão **Copiar aviso** (pt-BR/en, curto, editável) e opção "perguntar antes de gravar reuniões com pessoas de fora" por Receita. Primeiro uso explica, em linguagem simples, que a licitude depende do país/empresa e da LGPD, e que "sem bot" não dispensa aviso |
| Áudio | Por padrão **apagado ao encerrar** (fica texto + notas); "manter N dias" é opt-in |
| Só local | ASR local + modelo local (Ollama via BYOK): nada sai da máquina; indicador "Só local" |
| Dados pessoais | Opt-in: redigir CPF, cartão, e-mail, telefone antes de enviar ao provedor |
| Registro de acesso | Cada chamada ao modelo na reunião lista **o que foi enviado** (trechos) |
| Terceiros | A fala "Eles" não vira Memória nem perfil sem confirmação |
| Pausa | A Pausa de privacidade interrompe a reunião e marca a lacuna na transcrição |
| Slides | Captura de tela respeita Janelas excluídas e Permissão do agente |

### 4.8 Dados e ferramentas (rascunho)

- **Tabelas:** `meetings` (título, tipo, início/fim, briefing JSON, receita, projeto, retenção) · `meeting_utterances` (reunião, t0, t1, falante `you|them|p1…`, texto, idioma) · `meeting_notes` (do usuário, melhoradas, versões) · `meeting_moments` · `meeting_actions` (texto, dono, prazo, estado, trecho de origem) · índice FTS5. Cifrados como o resto do `aura.db`.
- **Ferramentas MCP:** `meeting_context` (Briefing + resumo rolante + últimas falas), `meeting_search`, `meeting_note_append`, `meeting_action_save`, `meeting_brief_save` — leitura livre sob a Política; escrita com aprovação quando persistente.
- **Resumo rolante:** a cada ~60–90 s (ou mudança de assunto/silêncio) um modelo **rápido** atualiza as notas por item da pauta; um modelo **melhor** faz a passada final. Modo **Econômico** = só sob demanda + passada final. Medidor "Esta reunião usou ~X% do seu plano".

### 4.9 Arquitetura e reuso

Segue o AGENTS.md: lógica em crates testáveis em qualquer SO; shell só encaminha; APIs do Windows só em `aura-win`.

- `aura-meeting` (novo, puro): Reunião, Briefing, Utterance, Receita, estado; repositório em `aura-store` (migração).
- `aura-asr`: `ContinuousTranscriber` (VAD por segmentos, duas fontes, parciais, retoma após falha); worker permanece quente durante a reunião.
- `aura-app/src/meeting.rs`: orquestração (iniciar/pausar/encerrar, laço de transcrição, resumo rolante, comandos IPC, `HostEvent`).
- `aura-win`: depois, **áudio por aplicativo** (process loopback) e supressão de eco (CAND-044).
- UI: `apps/desktop/src/meeting/` (Preparo, HUD, Pós), janela Tauri `meeting`, chaves i18n pt-BR/en, tipos IPC + contrato dourado (ADR 0009).
- ADRs a propor ao promover: **0010** Reunião como sessão opt-in com transcrição contínua; **0011** Assistente de configurações (diff/undo, escopo proibido).

### 4.10 Critérios de aceite candidatos (a refinar na spec)

- **AC-M01** Dado o Aura aberto por um dia sem o usuário iniciar Reunião, então nenhuma Fonte de áudio é capturada além do que a Política já permitia; Reunião nunca liga captura sozinha.
- **AC-M02** Dado "1:1 com a Ana sobre promoção; quero sair com data para falar com o RH", ao enviar no ⚡, surge o cartão "Entendi assim" com ≤ 2 perguntas de esclarecimento; "Começar" está disponível o tempo todo.
- **AC-M03** No 🧭, a IA faz ≤ 8 perguntas, uma por vez, cada uma com resposta recomendada e "Outro…"; não pergunta o que inferiu (janela, anexos, memórias); "Pular o resto" gera o Briefing com o que existe.
- **AC-M04** Com reunião começando em < 2 min (calendário), só o ⚡ é oferecido.
- **AC-M05** Fala do microfone aparece como "Você" e a do sistema como "Eles", com atraso p95 dentro do orçamento medido (H-019).
- **AC-M06** "Perdi o fio" responde o último minuto em ≤ 5 s, com trechos citáveis.
- **AC-M07** Toda nota e ação cita o minuto de origem; clicar abre o trecho.
- **AC-M08** Com 15 min de buffer de áudio, "Salvar como reunião" cria a Reunião com transcrição completa.
- **AC-M09** A Pausa de privacidade interrompe a Reunião e marca a lacuna.
- **AC-M10** Ao encerrar, em ≤ 60 s o pós-reunião mostra resumo, decisões e ações; o áudio é apagado conforme a retenção.
- **AC-M11** (Configurar com IA) "Não grave nada quando eu abrir o app do banco" gera um diff de Janelas excluídas; **Aplicar** grava e registra; **Desfazer** restaura.

---

## 5. Catálogo de funções (priorizado)

Esforço: **S** ≤ 1 semana · **M** 2–3 semanas · **L** > 1 mês (estimativa de ordem de grandeza, a confirmar nos planos). Todas usam o Preparo (§2) quando precisam de contexto.

### P0 — fundação e Reunião MVP

| # | Função | Origem | Valor ao usuário | Esforço | Candidato |
| --- | --- | --- | --- | --- | --- |
| F01 | Preparo em três velocidades (padrão, cartões, Skill `aura-preparo`) | grill-me; Granola Recipes (parcial) | Nunca encara um campo em branco; a IA entrevista | M | CAND-033 |
| F02 | Configurar com IA (`settings_*`, diff, desfazer, "Pedir à IA", onboarding "conte como trabalha") | pedido do usuário; Wispr (perfis por app) | Configura tudo falando; menos tempo em menus | M | CAND-034 |
| F03 | Motor de Reunião: sessão opt-in, transcrição contínua, biblioteca FTS5, "esqueci de iniciar" | Granola, Otter, Fathom | Reunião sem bot, local, retroativa | L | CAND-035 (absorve 016 e núcleo de 026) |
| F04 | Painel ao vivo: ações sob demanda, notas do usuário, marcadores | Fireflies, Cluely, Granola | Ajuda no momento certo sem poluir | M | CAND-036 |
| F05 | Pós-reunião: Receitas, notas melhoradas, ações, e-mail, "sem resposta", agente executa | Granola, Otter, Fathom | O trabalho **fica pronto**, não só resumido | M | CAND-037 |
| F06 | Consentimento e privacidade de reunião (aviso, só local, redação) | exigência legal/reputação | Confiança e conformidade | S | CAND-038 |

### P1 — diferenciais de uso

| # | Função | Origem | Valor | Esforço | Candidato |
| --- | --- | --- | --- | --- | --- |
| F07 | Pergunte sobre minhas reuniões; Projetos | Fireflies AskFred, Read, Granola Spaces | Memória útil entre reuniões | M | CAND-039 (+ 027, 030) |
| F08 | Legendas e tradução ao vivo; "responder em inglês" | novo; Krisp | Reuniões em outro idioma; acessibilidade | M | CAND-040 |
| F09 | Briefing antes da reunião e resumo diário (calendário via conector) | Granola Briefs, Fireflies Meeting Prep | Chega preparado sem esforço | M | CAND-041 (+ 031) |
| F10 | Ensaio por voz: entrevista, vendas, apresentação, negociação, com feedback | Final Round, Sensei, Yoodli, Gong | Pratica antes, com rubrica e evolução | M | CAND-042 |
| F11 | Quem falou: diarização e nomes dos participantes | Fathom, Read | Notas e ações com dono certo | M | CAND-043 |
| F12 | Áudio por aplicativo e supressão de eco | Krisp | Transcrição limpa e mais privada | M | CAND-044 |
| F13 | Ditado inteligente: modo comando, snippets, dicionário que aprende | Wispr Flow | Escreve falando em qualquer app | M | CAND-025 (existente) |
| F14 | Sugestões na tela vazia e detecção suave de reunião | já no roadmap | Descobre as funções sem procurar | S | CAND-028 (existente) |
| F15 | Conectores com "Conectar com IA" (Calendário, Gmail, Drive, Slack, Notion, Linear, CRM) | Otter, Fathom, Granola MCP | Ações fecham o ciclo | M | CAND-031 (existente) |

### P2/P3 — profundidade

| # | Função | Origem | Esforço | Candidato |
| --- | --- | --- | --- | --- |
| F16 | Coaching de fala privado (ritmo, muletas, tempo de fala) — só você vê | Yoodli, Read, Verve | S | CAND-045 |
| F17 | Modo Estudo (aula/vídeo): notas, glossário, flashcards, quiz | novo | M | CAND-046 |
| F18 | Compromissos e promessas entre reuniões (painel com envelhecimento) | Gong deal boards (para uso pessoal) | S | CAND-047 |
| F19 | Hábitos → sugestões de configuração | novo | S | CAND-048 |
| F20 | Agentes agendados e gatilhos ("resumo da manhã", "briefing 10 min antes") | screenpipe pipes, Otter, Gong | M | CAND-017 (existente) |
| F21 | Aura como servidor MCP/API local, somente leitura | Granola API/MCP | S | CAND-049 |
| F22 | Timeline pesquisável; apontar/anotar na tela; guia passo a passo; controle do computador | Rewind/screenpipe, Clicky | L | CAND-015, 014, 013 (existentes) |

---

## 6. Ideias próprias (além do que os concorrentes fazem)

1. **Retroativo.** O Buffer recente já existe; "esqueci de iniciar" transforma os últimos minutos em reunião. Ninguém que depende de iniciar antes consegue isso.
2. **Tela + áudio no mesmo minuto.** "O que ele quis dizer com *esse número*?" resolve com o quadro do buffer de tela daquele instante. Concorrentes são quase só áudio.
3. **Da reunião ao trabalho pronto.** O agente em modo Tarefa gera o artefato (minuta, planilha, issue) com aprovação e sandbox — o produto não termina no resumo.
4. **Preparo por voz.** Segurar `Ctrl+Space` e falar o contexto: a reunião começa sem mexer no teclado.
5. **Receita por app/janela.** Perfis de aplicativo + Receitas: abrir o Teams sugere a Receita certa.
6. **Só local.** ASR local + modelo local + redação de dados pessoais: o que um SaaS não oferece.
7. **Custo zero extra.** Usa o plano ChatGPT ou a chave do usuário; medidor de uso por reunião; modo Econômico.
8. **Confiança por citação.** Todo trecho gerado aponta para a fala original; "de onde veio isso?" em um clique.
9. **Promessas.** O que **eu** prometi e o que prometeram a **mim**, com lembrete — útil para quem trabalha sozinho.
10. **Debrief de 30 s.** Três perguntas após a reunião alimentam Memória e coaching (um mini grill-me pós-evento).
11. **Acessibilidade como recurso.** Legendas ao vivo, "Perdi o fio" (atenção/TDAH) e leitura em voz alta fazem do modo Reunião uma ferramenta de inclusão.
12. **Receitas abertas e compartilháveis** (agentskills.io): trocar "Receita de 1:1" ou "Discovery" como arquivo.

---

## 7. Riscos e tratamento

| Risco | Tratamento |
| --- | --- |
| **Entrevistas "invisíveis"**: violam regras de empresas/plataformas e geram dano de reputação | Posicionar em **preparo, ensaio e retrospectiva**; nada de recursos de evasão de proctoring; aviso de responsabilidade; manter a exclusão de captura como **privacidade** (decisão do esforço 001). Ver Q-006 |
| **Consentimento/LGPD**: gravar sem avisar | Aviso pronto, indicador visível, áudio apagado por padrão, só local, nada automático (§4.7). Revisar com jurídico antes de lançar |
| **ASR local nunca compilado no Windows** | Fase 0 começa por funções sem ASR; spike de ASR contínuo com nuvem como plano B; benchmark H-019 |
| **Eco**: alto-falante faz o microfone repetir o "Eles" | Detectar e deduplicar; recomendar fones; CAND-044 |
| **Quem falou** acima de 2 pessoas | Começar com Você/Eles; spike de diarização (avaliar licença e custo, ex.: sherpa-onnx) antes de prometer |
| **Custo/limites do plano ChatGPT** | Resumo rolante com modelo rápido, modo Econômico, medidor; H-020 |
| **Alucinação em notas e ações** | Citação obrigatória, estado "sem fonte" visível, edição fácil |
| **Excesso de funções** | Reunião é modo à parte; funções novas aparecem por sugestão contextual, não por menus |
| **"Configurar com IA" aplicar errado** | Diff obrigatório, Desfazer, escopo proibido (§3.3); bateria de testes de frases (H-024) |
| **Escopo** (V1 declarou reunião como não-objetivo) | Entregar como 0.3 depois da V1; atualizar `vision.md` com a spec |

---

## 8. Fases e esforços sugeridos

Numeração dos esforços é indicativa; o Hybrid atribui ao promover. Cada esforço entra pelo fluxo `discover → specify → plan → slice → implement → verify`.

**Fase 0 — Fundações (valor imediato, risco baixo)**

| Esforço | Entrega | Depende de |
| --- | --- | --- |
| **019 — Preparo e Configurar com IA** (CAND-033, 034) | Seletor das 3 velocidades + cartão "Entendi assim"; `aura-preparo`; `settings_*`; "Pedir à IA" na busca; `/configurar`; onboarding "conte como trabalha"; Perfil de app por IA | 017 (core-skills, tools) |
| **020 — Motor de Reunião** (CAND-035, base de 038) | `aura-meeting`; transcrição contínua (Você/Eles) com ASR local ou nuvem; armazenamento + FTS5; "esqueci de iniciar"; spike de diarização e de eco; benchmarks H-019/H-020 | 004, 005, 006 (motores locais validados no Windows) |

**Fase 1 — Reunião MVP (lançamento "0.3 — Reuniões")**

| Esforço | Entrega | Depende de |
| --- | --- | --- |
| **021 — Reunião: Preparo, HUD e ações sob demanda** (CAND-036) | Janela `meeting`, Preparo (3 caminhos), Briefing, ações ao vivo, notas do usuário, marcadores, detecção suave | 019, 020 |
| **022 — Pós-reunião, Receitas e consentimento** (CAND-037, 038) | Receitas embutidas + criadas por IA; notas melhoradas; ações/e-mail; agente executa; aviso, só local, redação | 021 |
| **023 — Biblioteca e Pergunte** (CAND-039) | Biblioteca, busca, `meeting_search`, Projetos básicos | 022 |

*Primeira fatia vertical sugerida (valida o risco cedo):* iniciar/parar manual → transcrição Você/Eles (nuvem se o local não estiver pronto) → ⚡ Preparo → resumo final com ações **citando o minuto**. Só depois HUD e Receitas.

**Fase 2 — Diferenciais** (números = candidatos do roadmap): tradução/legendas (040) · calendário e briefing (041, com 031) · Ensaio (042) · diarização (043) · áudio por app (044) · ditado inteligente (025) · sugestões contextuais (028) · Projetos e base de conhecimento (027, 030).

**Fase 3 — Profundidade** (idem): agendados (017) · coaching (045) · Modo Estudo (046) · compromissos (047) · hábitos (048) · MCP/API local (049).

## 9. Métricas de sucesso

| Métrica | Meta inicial |
| --- | --- |
| Tempo até iniciar uma reunião preparada (⚡) | ≤ 20 s |
| Reuniões iniciadas com Briefing | ≥ 60% (⚡ + 🧭) |
| Propostas de configuração aceitas sem edição | ≥ 70% |
| Notas/ações com citação de origem | 100% |
| Captura de áudio sem ação do usuário | 0 (auditável no Registro de acesso) |
| Orçamento em reunião (CPU, memória, tokens) | Definir após H-019/H-020 e fixar no esforço 020 |

**Hipóteses novas:** **H-019** ASR contínuo local sustenta uma reunião de 60 min em tempo real dentro de um teto de CPU/memória a fixar (palpite inicial: ≤ 25% de 4 núcleos, ≤ 700 MB no worker). **H-020** Resumo rolante com modelo rápido mantém uma reunião de 60 min em uma fração aceitável do plano. **H-021** A maioria prefere ⚡ a 🧭, e 🧭 melhora a utilidade percebida. **H-022** Você/Eles basta para a maioria das reuniões 1:1. **H-023** Deduplicação reduz o eco a < 5% de linhas duplicadas. **H-024** A IA aplica corretamente ≥ 90% de uma bateria de 30 pedidos de configuração.

## 10. Decisões que dependem de você

| ID | Pergunta | Recomendação |
| --- | --- | --- |
| **Q-006** (decidida em 05/10) | Posicionamento em **entrevistas**/ocultação | Você pediu a opção de ocultar como os concorrentes: **mantida e configurável** (§12). Continuam de fora: esconder o processo do Gerenciador de Tarefas, enganar softwares de prova/proctoring, e marketing de "indetectável" |
| **Q-007** | Por onde começar: Fase 0 completa ou direto na Reunião? | Fase 0 (Preparo + Configurar com IA) **em paralelo** ao spike de ASR contínuo: entrega valor já e destrava a Reunião |
| **Q-008** | Calendário na 0.3? | Não; detectar pelo app/janela e preparo manual. Calendário entra na Fase 2 via conectores MCP |
| **Q-009** | Guardar o áudio das reuniões? | Apagar ao encerrar por padrão (fica texto e notas); manter é opt-in |
| **Q-010** | Promover "reunião" de não-objetivo a objetivo pós-V1 (atualizar `vision.md`)? | Sim, como 0.3, quando a spec do esforço 020 for aceita |
| Q-001 (aberta) | Open-source × pago (termos do plano ChatGPT) | Segue valendo; uso intenso de reunião aumenta a relevância dos limites do plano |

## 11. Vocabulário proposto (para `CONTEXT.md` quando a spec for aceita)

- **Reunião** — sessão ao vivo iniciada pelo usuário, com captura explícita, transcrição, notas e fim. _Evitar_: gravação (ambíguo com Gravação manual), chamada.
- **Preparo** — momento curto, antes de uma função, em que o Aura entende o contexto do usuário (⚡ Rápido, 🧭 Completo, ▶ sem preparo). _Evitar_: wizard, onboarding (reservado aos Primeiros passos).
- **Grill-me** — o caminho Completo do Preparo. _Evitar_: interrogatório.
- **Briefing** — resultado editável do Preparo de uma Reunião (objetivo, resultado esperado, pontos, cuidados, ajuda).
- **Receita** — modelo reutilizável por tipo de reunião (Antes/Durante/Depois). _Evitar_: template (na UI).
- **Painel ao vivo** — janela da Reunião em andamento. _Evitar_: HUD (na UI).
- **Marcador** — momento destacado pelo usuário durante a Reunião.
- **Nível de ajuda** — Silencioso, Sob demanda, Equilibrado, Ativo.

## 12. Ocultar o Aura em qualquer transmissão

### 12.1 Estado (implementado em 05/10)

Opção **Configurações › Geral › "Ocultar em transmissões e gravações"** (`hideFromCapture`, padrão **ligada**) e o mesmo interruptor no menu Aparência do Overlay. Usa `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` em **todas** as janelas do Aura (Overlay, Configurações, seletor de região), aplicado na hora ao mudar a opção e na criação de cada janela. Ligada: só o usuário vê o Aura. Desligada: aparece como qualquer janela (útil para demonstrações e tutoriais).

O que isso cobre: tudo que captura pela pilha gráfica do Windows (Windows Graphics Capture, duplicação DXGI, GDI/BitBlt) — compartilhamento de tela do Meet, Teams, Zoom, Discord e Slack, OBS, Gravador de Tela, Print Screen, e controle remoto como AnyDesk e semelhantes quando capturam pelo Windows. O Overlay já ficava fora do Alt+Tab e da barra de tarefas.

**Limites que o app informa na própria opção:** não cobre câmera apontada para a tela nem placa de captura de vídeo (HDMI); ferramentas de acesso remoto com driver próprio de espelhamento, ou sessões RDP, **precisam ser testadas** (roteiro abaixo) — não prometemos o que não validamos; o Windows 10 anterior à versão 2004 não suporta esta API; notificações do Windows (toasts) e o ícone da bandeja não são janelas do Aura e podem aparecer na transmissão.

### 12.2 Próximos passos

- **Modo Transmissão:** quando o usuário estiver compartilhando (detecção por app de transmissão em primeiro plano ou botão), silenciar notificações nativas e prévias da Minibar fora do Aura.
- **Testar ocultação:** botão que captura a tela pelos três caminhos (WGC, DXGI, GDI) e confirma, com a miniatura, que o Aura não aparece.
- **Roteiro manual** de validação em: Meet (Chrome), Teams, Zoom, Discord (tela e janela), OBS (tela e janela), AnyDesk, RDP e Print Screen. Registrar evidência no Hybrid.
- **Fora do escopo, por decisão:** ocultar o processo, mascarar o nome do executável ou enganar proctoring. Primeiro uso mostra aviso curto: respeite as regras da empresa, da prova e dos participantes.

## 13. Outras funções dos concorrentes (fora de reuniões)

| # | Função | Origem | O que é no Aura | Candidato |
| --- | --- | --- | --- | --- |
| G01 | **"Me ajude agora"**: uma tecla responde sobre tela + último trecho de áudio, sem digitar | Cluely (`Ctrl+Enter`) | Atalho global que anexa tela e buffer recente e envia uma pergunta padrão; resposta no Overlay ou Minibar | CAND-050 |
| G02 | **Modo Código/Prova técnica**: lê o enunciado na tela e explica passo a passo (modo tutor: dica, depois solução) | Final Round, LockedIn (LeetCode, HackerRank) | Skill de tutor de código + OCR/UIA; níveis de ajuda "dica → abordagem → solução" | CAND-051 |
| G03 | **Carreira**: currículo sob medida para a vaga, carta de apresentação, banco de perguntas, histórias STAR | Final Round | Skills + Projeto "Busca de emprego"; usa Preparo 🧭 | CAND-052 |
| G04 | **Detecção automática de pergunta** (na tela ou no áudio) com rascunho de resposta | Sensei | Gatilho opcional do painel ao vivo e do Overlay | em CAND-036 |
| G05 | **Widget compacto fixo** (sempre visível, minúsculo) | Cluely Desktop Widget | Modo do Overlay/Minibar fixável, com opacidade | CAND-053 |
| G06 | **Tutor guiado na tela**: aponta onde clicar, passo a passo | Clicky | Anotação sobre a tela (CAND-014) + modo "guia" | CAND-014 (existente) |
| G07 | **Tradução de tela e de vídeo** (legenda do que toca/aparece) | Krisp, Cluely (12+ idiomas) | OCR + áudio do sistema → tradução no Overlay | CAND-040 |
| G08 | **Clipes e playlists** de trechos (áudio/tela) para compartilhar | Fathom | Recortes exportáveis com legenda | CAND-054 |
| G09 | **Base de conhecimento viva** (roteiros, FAQs, material de estudo) consultada em qualquer resposta | Cluely, Fireflies AskFred | Pastas indexadas com citações | CAND-030 (existente) |
| G10 | **Rastreadores de tópicos e palavras** (alertar quando citarem preço, prazo, concorrente) | Fireflies, Gong | Regras simples no painel ao vivo, criadas por IA | CAND-055 |
| G11 | **Roleplay** com personas (cliente difícil, entrevistador) | Gong, Yoodli | Ensaio | CAND-042 |
| G12 | **Voz limpa**: supressão de ruído do microfone local | Krisp | Avaliar (fora do núcleo; só se houver motor permissivo) | CAND-056 |
| G13 | **Agente que age**: agenda follow-up, envia e-mail, atualiza CRM com aprovação | Otter, Gong | Modo Tarefa + conectores MCP | CAND-031 (existente) |
| G14 | **Integração via MCP/API** com outras ferramentas | Granola | Aura como servidor MCP somente leitura | CAND-049 |

Não adotar: coach humano em tempo real (LockedIn), métricas de "carisma/viés" (Read) e qualquer recurso que dependa de enganar terceiros.

## 14. Pacote "Dia a Dia": funções simples para todo mundo

Pesquisa 3 (assistentes de uso geral): Windows Click to Do e PowerToys Advanced Paste, ChatGPT desktop (Appshots, Quick Chat, tarefas agendadas), Raycast AI (comandos de IA com substituição no lugar, histórico da área de transferência, snippets, tradutor), Apple Intelligence (Writing Tools: revisar, reescrever, resumir) e Highlight (assistente que vê a tela). O que eles têm em comum: **ação em um gesto, no app onde a pessoa já está, com resultado que volta para o lugar certo**.

Critérios de escolha: usada todo dia · zero configuração · resultado em poucos segundos · aproveita o que o Aura já tem (Chip de seleção, "Inserir no app", região da tela, OCR, buffer recente, TTS, Comandos rápidos). Quase tudo é Skill + Comando rápido + um pouco de interface.

### 14.1 Prioridade alta (a base do dia a dia)

| # | Função | Como o usuário vê | Já existe e é reaproveitado | Origem | Candidato |
| --- | --- | --- | --- | --- | --- |
| D01 | **Menu na seleção**: selecione texto em qualquer app, aperte o atalho e escolha Corrigir · Mais formal · Mais curto · Mais simpático · Resumir · Traduzir · Explicar; o resultado **substitui a seleção** (ou copia) com um clique | Mini-menu flutuante perto do texto; sem abrir conversa | Chip de seleção (UI Automation), `/corrigir`, `/traduzir`, "Inserir no app" | Apple Writing Tools, Raycast, Click to Do | CAND-058 (absorve CAND-020) |
| D02 | **Copiar texto da tela ou de uma imagem**: arrastar uma região e o texto já está copiado (com tradução opcional) | Atalho + retângulo; aviso "texto copiado" | Seleção de região, Windows OCR | Click to Do, PowerToys | CAND-059 |
| D03 | **"Isso é golpe?"**: selecione uma mensagem, e-mail, link ou boleto e o Aura diz, em linguagem simples, se tem sinais de golpe e o que fazer | Botão no menu da seleção e comando `/golpe` | Seleção, tela, busca web do agente; Skill interna | novo; alto valor para o público brasileiro | CAND-060 |
| D04 | **Responder isto**: com um e-mail ou conversa na tela, o Aura escreve um rascunho de resposta no tom da pessoa e cola no campo | Atalho; opções "curta / completa / educada" | Tela, Perfis de app, "Inserir no app" | Cluely, Wispr, ChatGPT | CAND-061 |
| D05 | **Lembretes por conversa**: "me lembra às 15h de ligar para o João" ou "toda segunda às 9h…" vira notificação nativa | Escreve ou fala; cartão de confirmação com horário | Notificações nativas, agendador mínimo (primeira fatia de CAND-017) | ChatGPT Tasks, Raycast | CAND-062 |
| D06 | **Anotação rápida**: falar ou digitar "anota: …" guarda numa caixa de entrada pesquisável; a IA organiza por assunto quando pedido | Atalho de voz → "Anotado" na Minibar | Ditado global, memórias, armazenamento local | Raycast, Otter | CAND-063 |
| D07 | **Onde eu parei?**: depois de uma interrupção, o Aura resume o que você estava fazendo nos últimos minutos | Botão e `/parei` | Buffer recente de tela (`screen_recent`) | novo (aproveita o diferencial do buffer) | CAND-064 |

### 14.2 Prioridade média

| # | Função | Descrição | Origem | Candidato |
| --- | --- | --- | --- | --- |
| D08 | **Colar como…** | Colar o que está na área de transferência em outro formato: texto limpo, lista, tabela, resumo, tradução, e-mail formal | PowerToys Advanced Paste | CAND-065 |
| D09 | **Entender documentos** | Contrato, boleto, bula, termos de uso, extrato: resumo em linguagem simples, riscos e prazos destacados, perguntas para fazer | novo; usa anexos e leitura por trechos | CAND-066 |
| D10 | **Ler a seleção em voz alta** | Mesma voz offline do "Ouvir", mas para qualquer texto selecionado (acessibilidade e estudo) | Apple, Windows Narrator | CAND-067 |
| D11 | **Copiar para o destino certo** | Copiar resposta já formatada para WhatsApp (sem Markdown), e-mail ou documento | Exportar (CAND-032) | CAND-032 (existente, sobe para P1) |
| D12 | **Salvos** | Estrela numa resposta guarda numa lista pesquisável (receitas, textos prontos, instruções) | Raycast, ChatGPT | CAND-068 |

### 14.3 Depois

| # | Função | Descrição | Candidato |
| --- | --- | --- | --- |
| D13 | **Histórico da área de transferência e snippets** (`;email`, `;endereço`) — opt-in, nunca guarda o que vem de gerenciadores de senha | CAND-069 |
| D14 | **Organizar arquivos**: "arrume minha pasta Downloads por tipo", "ache o contrato de aluguel de 2024" (modo Tarefa, com prévia e aprovação) | CAND-070 |
| D15 | **Como faço isso aqui?**: tutor passo a passo do app em foco (une CAND-014) | CAND-071 |
| D16 | **Respostas instantâneas locais**: contas, porcentagens, conversões de moeda e unidade sem esperar o modelo | CAND-072 |

### 14.4 Princípios para o pacote

1. **Um gesto, um resultado.** Nenhuma etapa de configuração antes da primeira vez; o Preparo (§2) só aparece quando realmente muda o resultado (ex.: tom de voz uma única vez).
2. **O resultado volta para onde a pessoa está** (substituir, colar ou copiar), nunca só "fica no chat".
3. **Mostrar o que será trocado** antes de substituir (antes → depois, com Desfazer), como em §3.
4. **Privacidade igual ao resto:** tudo passa pelo Chip e pela Política; nada de monitorar a área de transferência por padrão.
5. **Descoberta por sugestão:** o menu na seleção e as sugestões contextuais (CAND-028) ensinam as funções; não exigem decorar comandos.

### 14.5 Ordem sugerida

**Pacote 0.2.x "Dia a Dia"** (rápido, valor imediato, sem depender de ASR): D01 → D03 → D02 → D04 → D10. Em seguida D05/D06/D07 e o resto. D01 e D03 são principalmente Skills e Comandos rápidos; o trabalho de interface é o mini-menu e o "Substituir" com antes → depois.

Fontes: [Click to Do e PowerToys](https://www.windowscentral.com/software-apps/microsoft-powertoys-will-soon-take-full-advantage-of-the-same-ai-that-powers-copilot-pcs) · [ChatGPT desktop 2026](https://www.theneuron.ai/explainer-articles/gpt-5-6-and-the-new-chatgpt-desktop-app-complete-guide/) · [Raycast AI](https://www.raycast.com/core-features/ai) · [Apple Writing Tools](https://support.apple.com/guide/iphone/find-the-right-words-with-writing-tools-iph6f08da1d2/ios) · [Highlight](https://www.theai.tw/en/tools/highlight-ai)

## Fontes

Resumo das fontes e limites do levantamento em `docs/research/competitors-ao-vivo.md`.
