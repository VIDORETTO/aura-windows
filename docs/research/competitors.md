# Estudo de concorrentes

Data: 2026-09-29. Leitura dos READMEs/documentação oficial de cada projeto. Licenças importam: só reaproveitamos código de projetos permissivos (MIT/Apache) e apenas ideias dos demais.

| Projeto | Stack | Licença | Posicionamento |
| --- | --- | --- | --- |
| [ThukiWin](https://github.com/ayzekhdawy/thukiwin) | Tauri 2, React 19, Rust, SQLite, Ollama | Apache-2.0 (fork de Thuki/macOS) | Overlay flutuante local, "secretária" com leitura de tela e controle do computador |
| [AI Overlay](https://github.com/crony-io/aioverlay) | Tauri 2, SvelteKit, Rust | MIT | Overlay glassmorphic multi-provedor, captura de região/texto |
| [Clicky for Windows](https://github.com/lefterisloukas/heyclicky-windowz) | Python, PyQt6 | MIT | Tutor por voz ao lado do cursor, aponta elementos na tela |
| [Jan](https://www.jan.ai/docs/desktop) | Tauri, llama.cpp | Apache-2.0 | Alternativa local ao ChatGPT; modo Cowork agêntico, MCP, skills, memórias |
| [AnythingLLM](https://anythingllm.com/) | Electron | MIT | IA on-device: documentos, Meeting Assistant, Magic Echo (ditado), agentes |
| [screenpipe](https://github.com/dp466/screenpipe) | Tauri, Rust | Source-available (uso comercial exige licença) | Memória contínua de tela/áudio, busca, "pipes" (agentes agendados), MCP |

## Matriz de funcionalidades e decisão para o Aura

Legenda da coluna Aura: **MVP** (marco 1–2), **V1** (primeira versão pública), **Depois** (roadmap), **Não** (fora de escopo).

| Funcionalidade | Onde existe | Aura | Observação de desenho |
| --- | --- | --- | --- |
| Atalho global para abrir overlay | Todos | **MVP** | Combinação configurável; opção de "duplo toque em Ctrl" (ThukiWin usa hook `WH_KEYBOARD_LL`, janela 400 ms) |
| Janela flutuante translúcida, sempre no topo, fora da taskbar | ThukiWin, AI Overlay | **MVP** | Acrylic/Mica no Windows 11, fallback blur/cor no 10; opacidade ajustável |
| Overlay invisível em capturas/compartilhamento | — (ninguém anuncia) | **MVP** | `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)`; diferencial de privacidade |
| Texto selecionado vira citação ao abrir | ThukiWin, AI Overlay (Alt+C) | **V1** | Preferir UI Automation `TextPattern`; fallback Ctrl+C simulado com restauração do clipboard |
| Captura de tela inteira / região | ThukiWin, AI Overlay, Clicky | **MVP** | Windows.Graphics.Capture; seletor de região congelando a tela |
| Contexto de tela ao vivo para o agente | Clicky (a cada pergunta), screenpipe (contínuo) | **MVP** | Modos: desligado, sob demanda, últimos N minutos, contínuo, manual |
| Buffer "últimos minutos" de áudio/tela | screenpipe (contínuo) | **V1** | Ring buffer em disco, segmentado, criptografado; nada persiste sem ação |
| Árvore de acessibilidade + OCR | screenpipe | **V1** | UI Automation primeiro, `Windows.Media.Ocr` como fallback — barato e sem modelo extra |
| Filtro de janelas sensíveis (senhas, bancos) | Clicky (Privacy Guard), screenpipe | **MVP** | Lista padrão + regras do usuário; aplicado na captura, não no prompt |
| Minibar ao trocar de janela | ThukiWin | **V1** | Faixa de 40 px com status e prévia |
| Notificação ao terminar resposta | ThukiWin | **V1** | Toast nativo |
| Slash commands (`/screen`, `/tldr`, `/translate`...) | ThukiWin, Jan | **MVP** | Mesma paleta lista skills do usuário |
| Push-to-talk com transcrição | Clicky, AnythingLLM | **MVP** | ASR local escolhido pelo usuário |
| Catálogo de modelos ASR para baixar | Handy (referência), AnythingLLM | **MVP** | Whisper (GGUF), Parakeet, Moonshine, SenseVoice via `transcribe-rs` (MIT) |
| Recomendação de modelo pelo hardware | AnythingLLM | **V1** | RAM/GPU/NPU detectados |
| Ditado em qualquer app (Magic Echo) | AnythingLLM | **Depois** | Reusa ASR + inserção por `SendInput`/clipboard |
| Leitura em voz alta (TTS) | ThukiWin, Clicky | **V1** | Voz do Windows (offline) padrão; Edge/OpenAI opcionais com aviso |
| Anexar imagens, PDFs, planilhas, áudio, vídeo | Clicky (docs), Jan, AnythingLLM | **MVP** (imagem, PDF) / **V1** (resto) | Ingestão local converte para texto + imagens antes do turno |
| Conversas efêmeras | AI Overlay | **MVP** | `thread/start.ephemeral = true` do Codex |
| Chaves nunca expostas ao webview | AI Overlay (proxy Rust) | **MVP** | Credenciais no Windows Credential Manager; UI só vê metadados |
| Histórico criptografado | AI Overlay (AES-GCM) | **MVP** | Chave protegida por DPAPI |
| Multi-provedor + BYOK | Todos | **MVP** | ChatGPT (assinatura) + BYOK via gateway local |
| MCP com aprovação inline por tool | Jan | **MVP** | Cartão de aprovação dentro do chat |
| Skills gerenciáveis e invocáveis por `/` | Jan, Codex | **V1** | Padrão agentskills.io, pasta do Aura |
| Modo Plano, painel de progresso, "Agent changes", artefatos | Jan Cowork | **V1** | Eventos `turn/plan/updated`, `turn/diff/updated`, `fileChange` |
| Subagentes/monitores em background | Jan Cowork | **Depois** | Codex multi-agent |
| Memórias do usuário | Jan, AnythingLLM, Codex | **V1** | `features.memories` do Codex + tela de revisão |
| Controle do computador (`/do`) | ThukiWin, Clicky | **Depois** | Exige política de aprovação forte; alto risco |
| Apontar/anotar na tela (setas, círculos) | Clicky | **Depois** | Overlay click-through separado |
| Memória por aplicativo | Clicky | **V1** | Perfil de contexto por processo ativo |
| Timeline pesquisável de tudo que foi visto/ouvido | screenpipe | **Depois** | Depende de modo contínuo + índice FTS5 |
| Assistente de reuniões | AnythingLLM, screenpipe | **Depois** | Reusa captura de áudio + ASR + diarização |
| Agentes agendados ("pipes") | screenpipe | **Depois** | Codex tem automações; avaliar |
| Journal/flashcards, tutor | Clicky | **Não** | Fora do núcleo |
| Busca web | Clicky, AnythingLLM | **MVP** | `web_search` do Codex |
| Auto-update | AI Overlay | **V1** | Tauri updater com assinatura |

## Lições de design

1. **Velocidade de invocação é o produto.** ThukiWin mede em ms a janela do duplo toque; o overlay precisa estar pré-aquecido e aparecer em < 100 ms.
2. **Transparência sobre o que a IA vê.** Nenhum concorrente mostra claramente o que vai no prompt. O Aura exibe "chips de contexto" (tela, áudio, seleção, arquivos) removíveis antes do envio.
3. **Privacidade aplicada na captura, não no prompt.** screenpipe aplica permissões em camadas determinísticas; o Aura faz o mesmo com um motor de política único.
4. **Local-first sem ser local-only.** Assinatura ChatGPT dá qualidade de fronteira; ASR/OCR locais evitam enviar áudio e tela brutos sem necessidade.
5. **Evitar dependências pesadas.** Clicky exige Python; Jan/AnythingLLM carregam runtimes grandes. O Aura mantém Rust + WebView2 e baixa modelos sob demanda.
