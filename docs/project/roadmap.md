# Roadmap: Aura

Roadmap candidates are strategic outcomes, not executable tasks. Promote a candidate to an effort only after its next slice meets the readiness contract.

## Marcos

| Marco | Resultado demonstrável | Tickets (ordem detalhada em `execution-order.md`) |
| --- | --- | --- |
| M1 — Pergunte sobre a sua tela | Atalho → Overlay translúcido → "Continue with ChatGPT" → pergunta com print → resposta em streaming | 001 TK-001…006; 002 TK-001…005; 004 TK-001 (12) |
| M2 — Agente com contexto e voz | Histórico, modos Chat/Tarefa, aprovações, BYOK completo, push-to-talk com ASR local, região, política de privacidade, ferramentas de tela via MCP, PDF/planilhas/documentos | 002 TK-006…009; 003 TK-001…007; 004 TK-002…004; 005 TK-001; 006 TK-001…005; 007 TK-001…003 (26) |
| M3 — Memória de curto prazo | Buffers "últimos N minutos" de tela/áudio, gravação manual/contínua, Recortes transcritos, ferramentas `screen_recent`/`audio_recent` | 004 TK-005…007; 005 TK-002…004 (6) |
| M4 — Poder e extensões | Skills, MCP manager, comandos rápidos, modo plano/progresso, memórias; seleção como citação, Minibar, TTS, perfis por app, inserir no app; ASR em nuvem e parciais; áudio/vídeo como anexo | 006 TK-006…007; 007 TK-004…006; 008 TK-001…006; 009 TK-001…005 (18) |
| M5 — V1 pública | Instalador assinado, auto-update, onboarding, orçamentos de desempenho verificados, acessibilidade, i18n, diagnóstico | 010 TK-001…006 (6) |

## Candidatos

| Candidate | Outcome/hypothesis | Depends on | Priority | State | Promoted effort |
| --- | --- | --- | --- | --- | --- |
| CAND-001 | Fundação do app residente e overlay translúcido | — | P0 | promoted | 001-fundacao-overlay |
| CAND-002 | Conversa agêntica via Codex app-server com login ChatGPT | CAND-001 | P0 | promoted | 002-conversa-agente-codex |
| CAND-003 | BYOK com endpoint customizado via gateway Responses | CAND-002 | P1 | promoted | 003-byok-gateway |
| CAND-004 | Contexto de tela: modos de captura, ferramentas do agente e privacidade | CAND-001, CAND-002 | P0 | promoted | 004-contexto-de-tela |
| CAND-005 | Captura de áudio (microfone e sistema) com buffer e gravação | CAND-001 | P1 | promoted | 005-captura-de-audio |
| CAND-006 | Voz: catálogo de ASR, download e transcrição | CAND-005 | P0 | promoted | 006-voz-e-asr |
| CAND-007 | Anexos multimodais (imagem, vídeo, áudio, PDF, planilhas, documentos) | CAND-002, CAND-006 | P1 | promoted | 007-anexos-multimodais |
| CAND-008 | Extensões do agente: skills, MCP, memórias, modo plano, progresso | CAND-002 | P1 | promoted | 008-extensoes-do-agente |
| CAND-009 | Produtividade: seleção como citação, minibar, notificações, TTS, perfis por app | CAND-001, CAND-002 | P2 | promoted | 009-produtividade |
| CAND-010 | Distribuição e qualidade: instalador, update, onboarding, desempenho, a11y, i18n | CAND-001 | P1 | promoted | 010-distribuicao-e-qualidade |
| CAND-011 | Ditado em qualquer app (estilo Magic Echo) com modo bruto e "inteligente" | CAND-006 | P2 | candidate | — |
| CAND-012 | Modo voz em tempo real usando a assinatura (`thread/realtime/*` do Codex) | CAND-006, estabilização da API | P2 | candidate | — |
| CAND-013 | Controle do computador (`/do`) com política de aprovação forte | CAND-004, CAND-008 | P3 | candidate | — |
| CAND-014 | Apontar e anotar na tela (setas, círculos, destaque) | CAND-004 | P3 | candidate | — |
| CAND-015 | Timeline pesquisável de tela e áudio (FTS5 + OCR/UIA) | CAND-004, CAND-005 | P3 | candidate | — |
| CAND-016 | Assistente de reuniões (transcrição, diarização, resumo) | CAND-005, CAND-006 | P3 | candidate | — |
| CAND-017 | Agentes agendados e gatilhos por atividade | CAND-008 | P3 | candidate | — |
| CAND-018 | Porte para macOS | CAND-010 | P4 | candidate | — |
| CAND-019 | Contexto rico no `@`: escolher janela aberta, arquivo do workspace (`fuzzyFileSearch`), conversa anterior, área de transferência | CAND-004 | P2 | candidate | — |
| CAND-020 | Comandos rápidos com saída "substituir a seleção no app" (reaproveita Inserir no app) | CAND-009 | P2 | candidate | — |
| CAND-021 | Ramificar/reverter conversa e fila nativa (`thread/fork`, `thread/revert`, `thread/queue/*` do app-server, após validar a API fixada) | CAND-002 | P2 | candidate | — |
| CAND-022 | Limites do plano ChatGPT no Overlay (`account/rateLimits/updated`) | CAND-002 | P3 | candidate | — |
| CAND-023 | "Perguntar ao Aura" no menu de contexto do Explorer e janela companheira fixável | CAND-007 | P3 | candidate | — |
| CAND-024 | Palavra de ativação para o modo voz | CAND-012 | P3 | candidate | — |
| CAND-025 | Ditado inteligente em qualquer app: correções no meio da frase, formatação, modo comando ("reescreva mais formal") e snippets (Wispr Flow) | CAND-011 | P1 | candidate | — |
| CAND-026 | Modo gravação/reunião: transcrição ao vivo, resumo e tarefas em nota salva na conversa, briefing antes da próxima reunião (ChatGPT Record, Granola, Highlight) | CAND-016 | P1 | candidate | — |
| CAND-027 | Projetos: conversas agrupadas com arquivos e instruções próprias e memória restrita ao projeto (ChatGPT Projects) | CAND-008 | P2 | candidate | — |
| CAND-028 | Sugestões na tela vazia conforme o app em foco (ex.: "Resumir esta página" no navegador, "Explicar o erro" no VS Code) | CAND-009 | P2 | candidate | — |
| CAND-029 | Comparar modelos lado a lado na mesma pergunta (Msty Split Chat) | CAND-003 | P3 | candidate | — |
| CAND-030 | Base de conhecimento local (pastas indexadas com citações) consultada pelo agente (Msty Knowledge Stacks, Jan) | CAND-007 | P2 | candidate | — |
| CAND-031 | Conectores (Gmail, Calendário, Drive, Microsoft 365) como servidores MCP pré-configurados com OAuth | CAND-008 | P2 | candidate | — |
| CAND-032 | Exportar conversa (Markdown/PDF) e copiar resposta como texto formatado | CAND-002 | P3 | candidate | — |
| CAND-033 | Preparo em três velocidades (⚡ uma frase · 🧭 grill-me · ▶ sem preparo): cartões, "Entendi assim" e Skill `aura-preparo`, reutilizável por Reunião, Ensaio, Ditado, Perfis, Projetos, Agendados e Conectores | CAND-008 | P0 | candidate | — |
| CAND-034 | Configurar com IA em tudo: `settings_describe/propose/apply/undo`, cartão antes→depois com Desfazer, "Pedir à IA" na busca, `/configurar` e onboarding "conte como trabalha" | CAND-033 | P0 | candidate | — |
| CAND-035 | Motor de Reunião opt-in: sessão, transcrição contínua (Você/Eles), biblioteca com FTS5, "esqueci de iniciar" a partir do Buffer recente (absorve CAND-016 e o núcleo de CAND-026) | CAND-005, CAND-006 | P0 | candidate | — |
| CAND-036 | Painel ao vivo da Reunião: ações sob demanda (Perdi o fio, Resumo, O que respondo?, Termo, Marcar), notas do usuário, níveis de ajuda, detecção suave de reunião (Fireflies Live Assist, Cluely, Granola) | CAND-033, CAND-035 | P0 | candidate | — |
| CAND-037 | Pós-reunião: Receitas (Antes/Durante/Depois), notas melhoradas, ações com origem citada, e-mail, "sem resposta", promessas, agente executa em modo Tarefa (Granola, Otter, Fathom) | CAND-036 | P0 | candidate | — |
| CAND-038 | Consentimento e privacidade de reunião: aviso aos participantes, áudio apagado ao encerrar, modo só local, redação de dados pessoais, registro do que foi enviado | CAND-035 | P0 | candidate | — |
| CAND-039 | Pergunte sobre minhas reuniões (`meeting_search`) e Projetos com reuniões, arquivos e instruções (Fireflies AskFred, Read, Granola Spaces; une CAND-027/030) | CAND-037 | P1 | candidate | — |
| CAND-040 | Legendas e tradução ao vivo; sugerir resposta em outro idioma | CAND-036 | P1 | candidate | — |
| CAND-041 | Briefing antes da reunião e resumo diário a partir do calendário (Granola Briefs, Fireflies Meeting Prep) | CAND-031, CAND-037 | P1 | candidate | — |
| CAND-042 | Ensaio por voz: entrevista, vendas, apresentação e negociação com personas e feedback (Final Round, Sensei, Yoodli, Gong) | CAND-033, CAND-006 | P1 | candidate | — |
| CAND-043 | Quem falou: diarização e nomes dos participantes (spike de motor e licença antes de prometer) | CAND-035 | P1 | candidate | — |
| CAND-044 | Áudio por aplicativo (process loopback) e supressão de eco para transcrição limpa | CAND-005 | P1 | candidate | — |
| CAND-045 | Coaching de fala privado: ritmo, muletas, tempo de fala, perguntas (Yoodli, Read, Verve) | CAND-035 | P2 | candidate | — |
| CAND-046 | Modo Estudo (aula/vídeo): notas, glossário, flashcards e quiz | CAND-035 | P2 | candidate | — |
| CAND-047 | Compromissos e promessas entre reuniões, com envelhecimento e lembretes | CAND-037 | P2 | candidate | — |
| CAND-048 | Hábitos locais viram sugestões de configuração (sempre como proposta) | CAND-034 | P2 | candidate | — |
| CAND-049 | Aura como servidor MCP/API local somente leitura (notas e reuniões) para outras ferramentas (Granola API/MCP) | CAND-039 | P3 | candidate | — |

> Revisão 2026-10-05: CAND-033…049 vêm de `docs/project/plano-reuniao-e-configuracao-assistida.md` e `docs/research/competitors-ao-vivo.md`. CAND-016 passa a ser coberto por CAND-035/036/037; CAND-025, 028, 030 e 031 ganham prioridade como habilitadores (ditado inteligente, sugestões contextuais, base de conhecimento, conectores). Nada aqui é esforço até passar pelo fluxo Hybrid.
