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
