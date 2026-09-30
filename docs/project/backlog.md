# Backlog gerado

<!-- GENERATED: roadmap candidates and effort checkpoints are canonical sources. -->

| ID | Título/resultado | Origem | Prioridade | Fase | Estado | Tickets | Próxima ação |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `CAND-011` | Ditado em qualquer app (estilo Magic Echo) com modo bruto e "inteligente" | `roadmap` | `P2` | `roadmap` | `candidate` | — | Promover após prontidão |
| `CAND-012` | Modo voz em tempo real usando a assinatura (`thread/realtime/*` do Codex) | `roadmap` | `P2` | `roadmap` | `candidate` | — | Promover após prontidão |
| `CAND-013` | Controle do computador (`/do`) com política de aprovação forte | `roadmap` | `P3` | `roadmap` | `candidate` | — | Promover após prontidão |
| `CAND-014` | Apontar e anotar na tela (setas, círculos, destaque) | `roadmap` | `P3` | `roadmap` | `candidate` | — | Promover após prontidão |
| `CAND-015` | Timeline pesquisável de tela e áudio (FTS5 + OCR/UIA) | `roadmap` | `P3` | `roadmap` | `candidate` | — | Promover após prontidão |
| `CAND-016` | Assistente de reuniões (transcrição, diarização, resumo) | `roadmap` | `P3` | `roadmap` | `candidate` | — | Promover após prontidão |
| `CAND-017` | Agentes agendados e gatilhos por atividade | `roadmap` | `P3` | `roadmap` | `candidate` | — | Promover após prontidão |
| `CAND-018` | Porte para macOS | `roadmap` | `P4` | `roadmap` | `candidate` | — | Promover após prontidão |
| `001-fundacao-overlay` | Specification: Fundação do app residente e Overlay | `effort` | `—` | `implementation` | `active` | `0/6` | No Windows: compilar o shell Tauri e validar TK-001..006 (docs/HANDOFF.md §2, §4.1, §6 Marco 1) |
| `002-conversa-agente-codex` | Specification: Conversa agêntica via Codex app-server | `effort` | `—` | `implementation` | `active` | `0/9` | Spike com o app-server real (TK-001); UI de perguntas do agente (TK-007), uso de contexto (TK-008), pastas concedidas (TK-009) |
| `003-byok-gateway` | Specification: BYOK com endpoint customizado | `effort` | `—` | `implementation` | `active` | `0/7` | Validar com provedores reais e gravar fixtures SSE reais (docs/HANDOFF.md §4.3) |
| `004-contexto-de-tela` | Specification: Contexto de tela e privacidade | `effort` | `—` | `implementation` | `active` | `0/7` | Validar captura GDI/exclusão no Windows; ligar gravador em segundo plano e RecentMedia (TK-005..007); seleção de região (TK-002) |
| `005-captura-de-audio` | Specification: Captura de áudio | `effort` | `—` | `implementation` | `active` | `0/4` | Validar WASAPI/loopback; medidor e indicadores; ligar AudioSegmentWriter e audio_recent |
| `006-voz-e-asr` | Specification: Voz e ASR | `effort` | `—` | `implementation` | `active` | `0/7` | Compilar aura-worker com --features engines e validar PTT; vocabulário e ASR em nuvem no host (TK-005/006); parciais (TK-007) |
| `007-anexos-multimodais` | Specification: Anexos multimodais | `effort` | `—` | `implementation` | `active` | `0/6` | PDF/áudio/vídeo no aura-worker (HeavyIngestor) — TK-001/004/005 |
| `008-extensoes-do-agente` | Specification: Extensões do agente | `effort` | `—` | `implementation` | `active` | `0/6` | Status/OAuth de servidores MCP (TK-002); painéis Alterações/Arquivos (TK-005); Memórias (TK-006) |
| `009-produtividade` | Specification: Produtividade no desktop | `effort` | `—` | `implementation` | `active` | `0/5` | Validar seleção UIA e inserir no app; Minibar (TK-002), TTS (TK-003), Perfis (TK-004) |
| `010-distribuicao-e-qualidade` | Specification: Distribuição e qualidade | `effort` | `—` | `implementation` | `active` | `0/6` | Instalador NSIS assinado, updater, exportação de diagnóstico, auditoria de acessibilidade; aura-bench na máquina de referência |
