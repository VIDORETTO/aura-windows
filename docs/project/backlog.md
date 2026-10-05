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
| `CAND-019` | Contexto rico no `@`: escolher janela aberta, arquivo do workspace (`fuzzyFileSearch`), conversa anterior, área de transferência | `roadmap` | `P2` | `roadmap` | `candidate` | — | Promover após prontidão |
| `CAND-020` | Comandos rápidos com saída "substituir a seleção no app" (reaproveita Inserir no app) | `roadmap` | `P2` | `roadmap` | `candidate` | — | Promover após prontidão |
| `CAND-021` | Ramificar/reverter conversa e fila nativa (`thread/fork`, `thread/revert`, `thread/queue/*` do app-server, após validar a API fixada) | `roadmap` | `P2` | `roadmap` | `candidate` | — | Promover após prontidão |
| `CAND-022` | Limites do plano ChatGPT no Overlay (`account/rateLimits/updated`) | `roadmap` | `P3` | `roadmap` | `candidate` | — | Promover após prontidão |
| `CAND-023` | "Perguntar ao Aura" no menu de contexto do Explorer e janela companheira fixável | `roadmap` | `P3` | `roadmap` | `candidate` | — | Promover após prontidão |
| `CAND-024` | Palavra de ativação para o modo voz | `roadmap` | `P3` | `roadmap` | `candidate` | — | Promover após prontidão |
| `CAND-025` | Ditado inteligente em qualquer app: correções no meio da frase, formatação, modo comando ("reescreva mais formal") e snippets (Wispr Flow) | `roadmap` | `P1` | `roadmap` | `candidate` | — | Promover após prontidão |
| `CAND-026` | Modo gravação/reunião: transcrição ao vivo, resumo e tarefas em nota salva na conversa, briefing antes da próxima reunião (ChatGPT Record, Granola, Highlight) | `roadmap` | `P1` | `roadmap` | `candidate` | — | Promover após prontidão |
| `CAND-027` | Projetos: conversas agrupadas com arquivos e instruções próprias e memória restrita ao projeto (ChatGPT Projects) | `roadmap` | `P2` | `roadmap` | `candidate` | — | Promover após prontidão |
| `CAND-028` | Sugestões na tela vazia conforme o app em foco (ex.: "Resumir esta página" no navegador, "Explicar o erro" no VS Code) | `roadmap` | `P2` | `roadmap` | `candidate` | — | Promover após prontidão |
| `CAND-029` | Comparar modelos lado a lado na mesma pergunta (Msty Split Chat) | `roadmap` | `P3` | `roadmap` | `candidate` | — | Promover após prontidão |
| `CAND-030` | Base de conhecimento local (pastas indexadas com citações) consultada pelo agente (Msty Knowledge Stacks, Jan) | `roadmap` | `P2` | `roadmap` | `candidate` | — | Promover após prontidão |
| `CAND-031` | Conectores (Gmail, Calendário, Drive, Microsoft 365) como servidores MCP pré-configurados com OAuth | `roadmap` | `P2` | `roadmap` | `candidate` | — | Promover após prontidão |
| `CAND-032` | Exportar conversa (Markdown/PDF) e copiar resposta como texto formatado | `roadmap` | `P3` | `roadmap` | `candidate` | — | Promover após prontidão |
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
| `011-correcao-auditoria-e2e` | Correção integral da auditoria E2E | `effort` | `—` | `implementation` | `active` | `39/41` | Pending outside the agent: TK-001 native drag at 125%/150% DPI (requires changing display scaling) and the width report; TK-038 maintainer updater signing key, secret and signed-update test (010 AC-004). Everything else verified on the final build. |
| `012-selecao-e-cor-de-destaque` | Change: Seleção ao voltar ao Overlay e cor de destaque | `effort` | `—` | `discovery` | `active` | `—` | Registrar o entendimento necessário para a próxima entrega |
| `013-modelos-e-esforco` | Change: GPT-6.1 Sol, esforço por modelo e modo, esforços em modelos personalizados | `effort` | `—` | `discovery` | `active` | `—` | Registrar o entendimento necessário para a próxima entrega |
| `014-modo-no-meio-da-conversa` | Change: Trocar entre Chat, Tarefa e Plano no meio da conversa | `effort` | `—` | `discovery` | `active` | `—` | Registrar o entendimento necessário para a próxima entrega |
| `015-composer-e-conversa` | Change: Composer (@ e /), fila, modo visível e ações nas mensagens | `effort` | `—` | `discovery` | `active` | `—` | Registrar o entendimento necessário para a próxima entrega |
| `016-app-desktop-e-auditoria` | Change: Menu nativo, diálogos do navegador, histórico e auditoria visual | `effort` | `—` | `discovery` | `active` | `—` | Registrar o entendimento necessário para a próxima entrega |
