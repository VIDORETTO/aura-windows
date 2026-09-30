# Ordem de execução entre esforços

O runner do Hybrid Kit valida dependências (`requires`) **dentro** de cada esforço. Dependências **entre** esforços estão declaradas na seção "Dependências e sequência de execução" de cada ticket ("Depende de: 00X-…/TK-00Y (outro esforço)") e são consolidadas aqui. Grafo global verificado em 2026-09-29: 63 tickets, sem ciclos, nenhuma dependência apontando para um marco posterior.

## Regras

1. Um ticket só começa quando `package --json` retorna `ready: true` **e** todos os tickets de outros esforços listados em "Depende de" estão `done`.
2. Tickets da mesma onda podem rodar em paralelo, desde que não compartilhem `owned_areas` (o `graph` avisa sobreposições; as sobreposições atuais já são serializadas por dependências diretas ou transitivas).
3. "Onda" = profundidade no grafo global (1 = sem dependências). Ondas repetem entre marcos porque a numeração é global.
4. `010/TK-004` (desempenho) e `010/TK-005` (acessibilidade/i18n) não têm dependência formal além do 001, mas só fazem sentido com a interface do Marco 4 pronta; o CI de desempenho pode ser ligado cedo como anti-regressão.

## Tickets por marco e onda
## M1
onda 1: 001/TK-001 Esqueleto do monorepo e app residente na bandeja
onda 2: 001/TK-002 Janela do Overlay translúcida, sempre no topo e invisível em capturas; 002/TK-001 Spike de validação + cliente JSON-RPC e supervisor do app-server
onda 3: 001/TK-003 Atalho de invocação, duplo toque, foco e medição de abertura; 001/TK-004 Configurações persistentes e janela de Configurações
onda 4: 001/TK-005 Estados compacto/expandido, posição por monitor e perda de foco; 001/TK-006 Cofre local cifrado e logs com redação
onda 5: 002/TK-002 "Continue with ChatGPT" (Sign in with ChatGPT com uso do plano) e Gateway do plano
onda 6: 002/TK-003 Conversa em streaming com a Persona do Aura
onda 7: 002/TK-004 Seletor de modelo/esforço e indicador "Usando plano ChatGPT"; 002/TK-005 Chips de contexto e envio de imagens
onda 8: 004/TK-001 Captura de tela sob demanda como Chip (Marco 1)

## M2
onda 1: 006/TK-001 Catálogo de modelos, detecção de hardware e recomendação
onda 2: 006/TK-002 Gerenciador de downloads de modelos
onda 3: 006/TK-003 `aura-worker` e motor de transcrição local
onda 4: 005/TK-001 Fontes de áudio (microfone e sistema), hub e medidor
onda 5: 006/TK-004 Push-to-talk no Overlay
onda 6: 003/TK-001 Cadastro de Provedores, Credenciais no cofre e teste de conexão; 006/TK-005 Atalho global de voz, idioma e vocabulário personalizado
onda 7: 002/TK-006 Histórico, retomada e Conversas efêmeras; 002/TK-008 Direcionar turno, compactar e uso de contexto; 002/TK-009 Modos Chat e Tarefa com Workspace da conversa; 003/TK-002 Provedores BYOK no Gateway (modo passagem) e registro no Codex
onda 8: 002/TK-007 Aprovações, perguntas do agente e elicitation MCP; 003/TK-003 Tradução Chat Completions: texto em streaming e erros; 003/TK-007 Descoberta de modelos, capacidades e troca de Provedor; 007/TK-001 Pipeline de Ingestão, orçamento e PDF
onda 9: 003/TK-004 Tradução Chat Completions: tool calls em streaming; 003/TK-005 Tradução Chat Completions: imagens e uso de tokens; 004/TK-002 Seleção de região com tela congelada; 004/TK-003 Motor de Política de privacidade, exclusões, Pausa e Registro de acesso; 007/TK-002 Planilhas: esquema, amostra e estatísticas; 007/TK-003 Documentos, apresentações, texto e código
onda 10: 003/TK-006 Adaptador Anthropic Messages; 004/TK-004 Servidor MCP do Aura com ferramentas de tela e Permissão do agente

## M3
onda 10: 004/TK-005 Buffer recente de tela (últimos N minutos)
onda 11: 004/TK-006 Gravação manual, modo Contínuo, Retenção e gerenciador de Gravações; 004/TK-007 Anexar "últimos X minutos" e ferramenta de frames recentes
onda 12: 005/TK-002 Buffer recente, Gravação manual e Contínuo para áudio
onda 13: 005/TK-003 Indicadores e Pausa de privacidade para áudio
onda 14: 005/TK-004 Anexar áudio recente transcrito e ferramenta `audio_recent`

## M4
onda 6: 006/TK-007 Parciais em streaming
onda 7: 006/TK-006 ASR em nuvem via Provedor BYOK com fallback local; 008/TK-001 Skills: listar, importar com revisão, criar e invocar por `/`; 009/TK-003 Leitura em voz alta (TTS)
onda 8: 008/TK-002 Gerenciador de Servidores MCP; 008/TK-003 Comandos rápidos embutidos e personalizados; 008/TK-004 Modo plano e painel de Progresso; 008/TK-006 Memórias e instruções pessoais
onda 9: 007/TK-004 Áudio como anexo (Transcrição com tempos); 008/TK-005 Painéis de Alterações e Arquivos gerados com prévia segura; 009/TK-002 Minibar e notificações; 009/TK-004 Perfis de aplicativo
onda 10: 007/TK-006 Originais no workspace, `attachment_read` e cache por hash; 009/TK-001 Texto selecionado vira Chip de citação
onda 11: 009/TK-005 Inserir a resposta no Aplicativo anterior
onda 12: 007/TK-005 Vídeo como anexo (keyframes + Transcrição)

## M5
onda 1: 010/TK-005 Acessibilidade e idiomas (pt-BR/en)
onda 4: 010/TK-004 Orçamento de desempenho verificado continuamente
onda 5: 010/TK-006 Painel de diagnóstico e exportação redigida
onda 6: 010/TK-001 Instalador por usuário assinado e desinstalação limpa
onda 7: 010/TK-002 Atualização automática assinada com canais
onda 10: 010/TK-003 Onboarding de primeira execução
