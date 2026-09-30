# Discovery: Aura — assistente de IA flutuante para Windows

Nome de trabalho: **Aura** (derivado da pasta do projeto; reversível). Data: 2026-09-29.

## Demand and desired outcome

Um app Windows residente, leve e sempre disponível. Uma combinação de teclas abre uma janela flutuante levemente transparente onde o usuário conversa com uma IA agêntica. A IA pode ver a tela em tempo real, ouvir áudio do microfone/sistema conforme o que o usuário permitir, usar tools, skills e servidores MCP, e receber anexos (imagens, vídeos, áudios, PDFs, planilhas, documentos). O usuário entra com a conta do ChatGPT e usa a própria assinatura, ou traz a própria chave (BYOK) com endpoint customizado. Voz é transcrita por um modelo ASR escolhido e baixado pelo usuário. A experiência deve ser extremamente fluida, moderna e profissional.

Resultado desejado: o usuário resolve uma dúvida ou tarefa sobre o que está fazendo **sem sair do app em que está**, em segundos, com controle explícito do que a IA vê e ouve.

## Actors, constraints, scope, and non-goals

**Atores**

- Usuário final no Windows (profissional do conhecimento, dev, estudante).
- OpenAI (ChatGPT/Codex) como provedor principal via assinatura.
- Provedores BYOK (OpenAI API, Azure, OpenRouter, Anthropic, Gemini, Groq, DeepSeek, Ollama, LM Studio, vLLM, endpoints corporativos).
- Servidores MCP de terceiros configurados pelo usuário.

**Restrições**

- Windows 10 1809+ x64 e Windows 11 (recomendado). ARM64 depois.
- Máquina do usuário tem recursos limitados e outras aplicações abertas: o Aura precisa ser leve em memória/CPU ociosos.
- Dados de tela e áudio são sensíveis: nada sai da máquina sem estar num turno enviado pelo usuário ou numa ferramenta aprovada pela política.
- Desenvolvimento acontece a partir de um servidor Linux; build, E2E e medições de desempenho exigem runner Windows (CI `windows-latest` + máquina de referência).

**Escopo (produto completo planejado)**: ver `docs/project/vision.md` e `docs/project/roadmap.md`.

**Não-objetivos da V1**: controle autônomo do mouse/teclado, timeline pesquisável contínua estilo screenpipe, assistente de reuniões, versão macOS/Linux, sincronização em nuvem, app mobile.

## Facts observed

- O Codex app-server é a interface oficial que a OpenAI usa para clientes ricos e oferece login ChatGPT por browser e device code, API key, threads, turnos com streaming, aprovações, MCP, skills, sandbox Windows, uso/limites da assinatura (`docs/research/harness-evaluation.md`).
- O Codex removeu `wire_api = "chat"`; provedores customizados precisam falar Responses API (código `model-provider-info/src/lib.rs`, constante `CHAT_WIRE_API_REMOVED_ERROR`).
- `thread/start` aceita `model_provider`, `model`, `config`, `base_instructions`, `developer_instructions`, `ephemeral` e `dynamic_tools` (código `app-server-protocol/src/protocol/v2/thread.rs`).
- `UserInput` aceita `text`, `image`, `localImage`, `audio`, `localAudio`, `skill`, `mention` (`v2/turn.rs`). Os modelos listados hoje declaram `inputModalities: ["text", "image"]`.
- Usar o token do Codex CLI fora do Codex não tem contrato documentado (issue `openai/codex#36886`); porém a OpenAI publica o programa **Sign in with ChatGPT — ChatGPT plan usage** (`developers.openai.com/siwc`) com OAuth/PKCE próprio do app, escopo `chatgpt.tokens.use.direct`, chamadas a `api.openai.com/v1/responses` e uma receita oficial para o Codex app-server. Liberado para apps open-source/locais; pagos/hospedados via formulário de interesse.
- O repositório está vazio exceto pelo Hybrid Development Kit instalado; não há Git inicializado (baseline por fingerprint de inventário).

## Decisions

| ID | Decision | Origin | Consequence |
| --- | --- | --- | --- |
| D-001 | Plataforma alvo: Windows 10 1809+ x64 e Windows 11 | Usuário | APIs Win32/WinRT livres; WGC exige 1903+ para captura de janela e 2004+ para `WDA_EXCLUDEFROMCAPTURE` → mínimo efetivo **Windows 10 2004** |
| D-002 | Harness: Codex app-server como sidecar | Pesquisa (usuário sugeriu Codex) | ADR 0001; BYOK depende do gateway (D-006) |
| D-003 | Login nativo ChatGPT consumindo a assinatura do usuário | Usuário | Via Sign in with ChatGPT (ADR 0007): OAuth/PKCE do próprio Aura, token injetado pelo Gateway no provedor Responses do app-server |
| D-004 | Stack: Tauri 2 + Rust (host) + React 19/TS (UI) | Recomendação | ADR 0002; build/E2E em Windows |
| D-005 | BYOK com endpoint customizado, chaves no Credential Manager | Usuário | Chaves nunca em arquivo de config nem no webview |
| D-006 | Gateway Responses local atende todos os provedores: passagem (plano ChatGPT, provedores Responses) ou tradução (Chat Completions, Anthropic) | Recomendação | ADR 0003, ADR 0007 |
| D-007 | Modos de captura independentes para tela, microfone e áudio do sistema: desligado, sob demanda, últimos N minutos (buffer), contínuo, manual (gravar a partir do clique) | Usuário | ADR 0004 (buffer em disco criptografado) |
| D-008 | Permissão do agente para acessar captura é separada da gravação | Recomendação | Um motor de política decide cada acesso; UI mostra o que foi usado |
| D-009 | ASR escolhido pelo usuário, com catálogo para download | Usuário | Motor `transcribe-rs`/`transcribe-cpp`; ASR em nuvem opcional via BYOK |
| D-010 | Anexos: imagem, vídeo, áudio, PDF, planilhas, documentos, texto/código | Usuário | Ingestão local converte para texto + imagens antes do turno |
| D-011 | Ferramentas do Aura expostas ao Codex por um servidor MCP local | Recomendação | Estável entre versões; `dynamicTools` só como fallback |
| D-012 | Idioma da UI: pt-BR primeiro, en em seguida | Suposição reversível | Strings externalizadas desde o início |
| D-013 | Atalho padrão `Ctrl+Shift+Space`; "duplo toque em Ctrl" opcional; tudo configurável | Suposição reversível | Evita conflito com `Alt+Space` (PowerToys) e `Ctrl+Space` (IME) |
| D-014 | Local-first: capturas, histórico e índices ficam na máquina, criptografados | Recomendação | Só o conteúdo de um turno vai ao provedor |

## Hypotheses and evidence

| ID | Hypothesis | Impact | Check | State |
| --- | --- | --- | --- | --- |
| H-001 | Com a janela pré-criada e oculta, o overlay aparece em ≤ 100 ms (p95) após o atalho | Define a sensação de fluidez; se falhar, rever arquitetura de janela | Harness de medição no TK de hotkey (esforço 001) em máquina de referência | pending |
| H-002 | Os termos da OpenAI permitem distribuir um app de terceiros que usa o plano ChatGPT via Codex app-server | Bloqueia lançamento comercial, não o desenvolvimento | Respondida em parte pela doc SIWC: permitido para apps open-source/locais; app pago/fechado requer aprovação via formulário de interesse | partially_confirmed |
| H-003 | Imagens retornadas por tools MCP chegam ao modelo como imagem | Define se `screen_capture` via MCP funciona sem `dynamicTools` | Spike TK-001 do esforço 002 | pending |
| H-004 | Consumo ocioso: host + WebView2 oculto ≤ 150 MB; `codex-app-server` ocioso ≤ 80 MB | Promessa de "leve" | Medição de working set privado em máquina de referência | pending |
| H-005 | Buffer de tela a 1 fps com H.264 por hardware usa ≤ 3% de CPU e ≤ 300 MB/h de disco | Viabilidade do modo "últimos N minutos" e contínuo | Benchmark no esforço 004 | pending |
| H-006 | Parakeet V3 int8 em CPU transcreve 10 s de fala em < 1 s num 4 núcleos | Latência do push-to-talk | Benchmark no esforço 006 | pending |
| H-007 | A tradução Responses→Chat Completions preserva tool calls em streaming | BYOK agêntico funcional | Testes de contrato com fixtures gravadas (esforço 003) | pending |
| H-008 | `base_instructions` substitui o prompt de programação do Codex | Persona de assistente de desktop | Spike TK-001 do esforço 002 | pending |
| H-009 | `WDA_EXCLUDEFROMCAPTURE` oculta o overlay das capturas do próprio Aura e de Teams/Zoom/OBS | Privacidade e evitar auto-captura | Teste manual e E2E de captura no esforço 001/004 | pending |

## Questions that block the next result

Nenhuma bloqueia o marco 1. Perguntas abertas com padrão adotado (podem mudar sem refazer o planejamento):

- Q-001 **(decisão material do usuário antes do lançamento)** O Aura será open-source (uso do plano ChatGPT liberado pelo SIWC) ou pago/fechado (exige enviar o formulário de interesse da OpenAI e aguardar aprovação)? Recomendação: publicar o cliente desktop como open-source (ou pedir aprovação já no início). Não bloqueia a implementação: a arquitetura é a mesma nos dois casos.
- Q-002 Nome e marca finais? Padrão: "Aura" como nome de trabalho; identificador do app `com.aura.desktop` configurável.
- Q-003 Telemetria? Padrão: nenhuma telemetria remota; métricas locais opcionais e exportáveis.
- Q-004 macOS no futuro? Padrão: crates de domínio sem dependência de Windows; adapters de SO isolados.

## Next deliverable and continue/stop criterion

Próximo resultado: **Marco 1** — overlay invocado por atalho, login ChatGPT, conversa em streaming com o Codex app-server e captura de tela sob demanda anexada ao turno (esforços 001 e 002 + TK de captura sob demanda do 004).

Continuar se: H-001, H-004 e H-008 se confirmarem e o login ChatGPT funcionar no app GUI. Parar e reavaliar o harness se o app-server não puder ter o prompt substituído, exceder o orçamento de memória de forma irremediável ou se H-002 for negada — nesse caso, avaliar loop próprio + Agents SDK apenas para BYOK.
