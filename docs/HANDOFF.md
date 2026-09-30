# HANDOFF — do Linux para o Windows

Documento para o agente de IA (ou pessoa) que continua o Aura **no Windows**:
compilar o app de verdade, validar o que só foi verificado por tipo e produzir
evidências. Leia inteiro antes de mudar código.

> Onde foi feito: VPS Linux x86_64 (2 vCPU), sem WebView/GTK, sem MSVC, sem
> conta ChatGPT. Tudo que é independente de SO foi **executado e testado**,
> inclusive **com o Codex app-server real** (build Linux da mesma versão
> fixada). Os adaptadores Windows foram **verificados por tipo e clippy**
> (`--target x86_64-pc-windows-msvc`); o shell Tauri (`apps/desktop/src-tauri`)
> **nunca foi compilado** (exige WebView2/MSVC).

---

## 1. Resumo do estado

| Camada | Onde | Estado | Como foi verificado |
| --- | --- | --- | --- |
| Domínio, store, cofre, política | `crates/aura-{core,store,policy}` | ✅ | `cargo test` |
| Codex app-server (cliente, supervisor, serviço) | `crates/aura-codex` | ✅ | Contrato com app-server falso **e teste ponta a ponta com o app-server real v0.159.0** (§2.4) |
| Sign in with ChatGPT | `crates/aura-auth` | ✅ lógica | IdP simulado; **login real não executado** (sem conta) |
| Gateway (passagem, Chat Completions, Anthropic; ferramentas com namespace) | `crates/aura-gateway` | ✅ | Fixtures + **turno real** Codex → gateway (Chat) → provedor simulado |
| Servidor MCP do Aura (6 ferramentas) | `crates/aura-mcp`, `aura-app/src/tools.rs` | ✅ | **Codex real conecta, lista e chama** `active_window_info` |
| Captura: redação, segmentos cifrados, retenção, **gravador em segundo plano, gravações manuais, `screen_recent`/`audio_recent`** | `aura-capture`, `aura-app/src/recorder.rs` | ✅ lógica | Testes com fonte/áudio sintéticos |
| Seleção de região (tela congelada) | `aura-app` + janela `region` | ✅ lógica / 🟡 janela | Teste do host; UI testada; janela Tauri não compilada |
| Áudio, ASR, push-to-talk, **parciais ao vivo**, **ASR em nuvem com fallback**, **vocabulário** | `aura-audio`, `aura-asr`, `aura-app/src/voice.rs` | ✅ lógica | Testes; motores locais reais (`--features engines`) **não compilados** |
| Anexos: texto, código, HTML, planilhas, DOCX/PPTX/ODF, RTF, **PDF (camada de texto)**, **WAV**; **MP3/M4A/vídeo** via Media Foundation | `aura-ingest`, `aura-app/src/media.rs`, `aura-win/src/media.rs` | ✅ / 🟡 MF | PDF/WAV testados; decodificação MF só por tipo |
| Extensões: Skills, comandos rápidos, MCP (+ status/OAuth), importadores | `aura-extensions`, `aura-app` | ✅ | Testes |
| Perfis de aplicativo, TTS (ouvir resposta), Minibar + notificação, diagnóstico redigido, onboarding, updater | `aura-app`, UI, shell | ✅ lógica / 🟡 shell | Testes (TTS fake, zip redigido sem segredos) |
| Adaptadores Windows (GDI, UIA, OCR, WASAPI, Cred. Manager, atalhos, entrada, DXGI, MF H.264 enc/dec, TTS, Job Object) | `crates/aura-win` | 🟡 só tipos | `check` + `clippy -D warnings` no alvo MSVC |
| Shell Tauri | `apps/desktop/src-tauri` | 🟡 nunca compilado | — |
| UI React | `apps/desktop/src` | ✅ | `tsc`, **53 testes** Vitest (inclui auditoria axe em todas as páginas), `vite build` |
| Contrato IPC Rust↔TS | `aura-app/tests/ipc_contract.rs` + `src/ipc/contract.test.ts` | ✅ | Arquivo dourado compartilhado (ADR 0009) |
| CI, perf, release, E2E Windows, licenças (`deny.toml`) | `.github/`, `apps/desktop/e2e`, `deny.toml` | 🟡 escritos | Não executados |

Números: **246 testes Rust** + 2 ignorados (app-server real, sob demanda),
**53 testes de UI**, `clippy -D warnings` e `rustfmt --check` limpos (inclusive
`aura-win`/`aura-bench` no alvo MSVC).

### Bugs reais encontrados rodando o app-server de verdade (já corrigidos)

1. O binário fixado `codex-app-server` **não aceita o subcomando `app-server`** — o app nunca subiria. `launcher::app_server_args` agora só o usa para o CLI `codex`.
2. `thread/start` rejeitava `approvalPolicy: "onRequest"`: o protocolo usa **kebab-case** (`on-request`, `read-only`, `workspace-write`).
3. Ferramentas MCP chegam como **namespace** (`{"type":"namespace","name":"mcp__aura",...}`) e o Codex exige `namespace` no `function_call` — a tradução Chat/Anthropic agora achata para `mcp__aura__<tool>` e devolve `namespace` na saída.

---

## 2. Primeiros passos no Windows

### 2.1 Pré-requisitos

- Windows 11 (ou 10 22H2), WebView2 Runtime.
- Visual Studio 2022 Build Tools, "Desktop development with C++" (MSVC + Windows SDK).
- Rust 1.98.1 (fixado em `rust-toolchain.toml`), Node 22 + pnpm (`corepack enable`).
- Motores de voz locais: CMake + LLVM/Clang (`transcribe-rs`).

### 2.2 Ordem sugerida

1. `cargo test --workspace --exclude aura-desktop` → verde (compila `aura-win` e o DPAPI de verdade).
2. `./scripts/prepare-sidecars.ps1`, depois `pnpm -C apps/desktop install && pnpm -C apps/desktop tauri build --no-bundle`. Corrija erros de API do Tauri (§4.1) **sem mover lógica para o shell**.
3. Demo sem conta: `pnpm -C apps/desktop tauri dev --features demo` (agente falso + SO sintético).
4. Real: `pnpm -C apps/desktop tauri dev`. Primeira execução baixa e verifica o app-server fixado (progresso no Overlay).
5. Roteiro manual do §6, E2E (`apps/desktop/e2e`) e evidências no Hybrid (§8).

### 2.3 Comandos

```powershell
cargo test --workspace --exclude aura-desktop
cargo clippy --workspace --all-targets -- -D warnings
pnpm -C apps/desktop test ; pnpm -C apps/desktop typecheck
pnpm -C apps/desktop tauri dev [--features demo]
pnpm -C apps/desktop tauri build                          # NSIS/MSI + artefatos do updater
pnpm -C apps/desktop/e2e install ; pnpm -C apps/desktop/e2e test   # após build --features demo
cargo run -p aura-bench --release -- all --pid <pid> --out bench/latest.json --baseline bench/baseline.json
$env:UPDATE_GOLDEN=1; cargo test -p aura-app --test ipc_contract   # após mudar formatos de IPC
```

Variáveis: `AURA_HOME` (pasta de dados), `AURA_CODEX_BIN` (app-server local, sem download),
`AURA_GATEWAY_DUMP_DIR` (grava cada requisição bruta do Codex ao gateway — **só para
diagnóstico de protocolo**, contém conteúdo de conversa), `AURA_FAKE_CODEX_*`.

### 2.4 Teste ponta a ponta com o app-server real (qualquer SO)

```bash
# binário da mesma release fixada (Linux: codex-app-server-x86_64-unknown-linux-musl.zst;
# Windows: o pacote fixado em crates/aura-codex/codex-version.toml)
AURA_CODEX_BIN=/caminho/codex-app-server cargo test -p aura-app --test real_app_server -- --ignored --nocapture
```

Cobre: `initialize`, `thread/start` com provedor BYOK do gateway, turno em streaming,
chave do cofre injetada (`Authorization: Bearer …`), ferramenta MCP `aura.*` chamada
pelo modelo e executada pelo host. Há também `probe_raw_responses_request` (imprime as
ferramentas que o Codex envia) para investigar mudanças de protocolo.

---

## 3. Mapa do código

```
crates/
  aura-core        domínio, Settings, atalhos, gestos, posicionamento, chips, JSON-RPC, logs com redação
  aura-store       SQLite + migrações, cofre AES-GCM (DEK protegida por DPAPI), repositórios
  aura-policy      decide(): permitir / redigir / perguntar / negar
  aura-codex       app-server: binário fixado, supervisor, serviço, app-server falso
  aura-auth        Sign in with ChatGPT (ADR 0007)
  aura-gateway     /p/{id}/v1/responses|models; tradução Chat/Anthropic; presets
  aura-mcp         servidor MCP (/mcp) + ferramentas
  aura-capture     Frame, redação, segmentos cifrados, retenção; seams VideoCodec, MediaFileDecoder
  aura-audio       hub, DSP, gravação
  aura-asr         catálogo, downloads, worker, push-to-talk (parciais), nuvem
  aura-ingest      anexos leves + PDF; HeavyIngestor
  aura-extensions  Skills, comandos rápidos, MCP, importadores
  aura-app         Host: composição + 1 método por comando + HostEvent; recorder, media, voice,
                   profiles, speech, diagnostics, privacy, tools
  aura-win         adaptadores Windows
  aura-worker      processo sob demanda (ASR)
apps/desktop/
  src-tauri        shell fino: comandos → Host, overlay, região, tray, atalhos, plataforma Windows
  src              React: overlay/, settings/, state/, ipc/, i18n/, ui/, lib/
  e2e              WebdriverIO + tauri-driver (Windows)
tools/aura-bench   orçamentos de desempenho
```

Regra de ouro: lógica nova vai para `aura-app` (ou crates de domínio) **com
teste**; `src-tauri` só encaminha; APIs do Windows só em `aura-win`.

---

## 4. O que ainda não rodou

### 4.1 Shell Tauri — nunca compilado

Pontos prováveis de ajuste: identificadores em `capabilities/default.json`
(`updater:default`, `process:allow-restart`, `dialog:allow-save`, `opener:allow-open-path`…);
`TrayIconBuilder::show_menu_on_left_click`; `WebviewWindow::hwnd()` (convertido com
`hwnd.0 as usize as u64`); `WebviewWindowBuilder` da janela `region`; plugins
(`single-instance`, `autostart`, `dialog`, `opener`, `notification`, `updater`, `process`);
`bundle.externalBin` exige `binaries/aura-worker-<triple>.exe`; escopo do
`assetProtocol` para miniaturas.

### 4.2 `aura-win` — só verificado por tipo

| Módulo | Suposição | Como validar |
| --- | --- | --- |
| `capture.rs` (GDI) | `BitBlt`+`CAPTUREBLT` respeita `WDA_EXCLUDEFROMCAPTURE`; `PrintWindow(PW_RENDERFULLCONTENT)` em Chrome/Electron | Capturar com o Overlay aberto; janela do Chrome |
| `windows_info.rs` | Z-order, DPI por monitor, `restore_focus` | Exclusão sobreposta em 100%/150% |
| `ocr.rs` / `uia.rs` | `join()` do WinRT; pacote pt-BR; TextPattern | `screen_text` com `ocr` e `uia` |
| `credman.rs` | Fragmentação > 2560 bytes | Login real; `cmdkey /list` |
| `hotkeys.rs` | PTT só armado com Overlay visível | Ctrl+Space no Word com o Overlay fechado |
| `audio.rs` (cpal 0.16) | Loopback em dispositivo de saída | Buffer de áudio do sistema |
| `encoder.rs` | fMP4 em memória; stride RGB32; decodificação de keyframes | Buffer de tela → `screen_recent` |
| `media.rs` | Source reader para MP3/M4A/MP4; seek por `MF_PD_DURATION` | Anexar .mp3 e .mp4 |
| `speech.rs` | Voz pt-BR instalada | "Ouvir" numa resposta |
| `job.rs` | Filhos morrem com o Aura | Matar `aura.exe`; app-server/worker somem |
| `input.rs` | Colar e restaurar clipboard | "Inserir no app" no Notepad/Chrome |

### 4.3 Contratos externos ainda assumidos

| Item | Onde | Verificar |
| --- | --- | --- |
| Chaves MCP de usuário (`env_vars`, `bearer_token_env_var`, `http_headers`, `disabled_tools`…) e `mcp_oauth_credentials_store = "keyring"` | `aura-extensions/src/mcp_config.rs` | O servidor `aura` (url + bearer) **já foi validado** com o app-server real; validar um servidor stdio de terceiros |
| `mcpServerStatus/list`, `mcpServer/oauth/login`, `config/mcpServer/reload` | `aura-codex/src/service.rs` | Chamar com o app-server real; ajustar o parser `parse_mcp_status` |
| `tool_search` (MCP diferido) | gateway | Com provedores Responses (plano ChatGPT/OpenAI) é nativo. Se um provedor Chat receber `tool_search` (visto com metadados de modelo conhecidos), as ferramentas MCP ficam indisponíveis nele — investigar com `AURA_GATEWAY_DUMP_DIR` |
| Revogação SIWC, fixtures SSE reais, SHA dos modelos ASR, marca "Continue with ChatGPT" | vários | Login real; gravar SSE reais; baixar modelos; ativo oficial no botão |
| Updater | `tauri.conf.json` | Gerar chaves (`pnpm tauri signer generate`), pôr a pública em `plugins.updater.pubkey`, conferir o endpoint (`VIDORETTO/aura-windows`) |

---

## 5. Desvios em relação ao plano

1. Captura via **GDI** (ADR 0008).
2. IPC por **contrato dourado** sem tauri-specta (ADR 0009).
3. i18n em módulos TS com teste de paridade.
4. Crate **`aura-app`** concentra a orquestração.
5. Crate **`aura-win`** concentra as APIs do Windows; `aura-capture`/`aura-audio` têm a feature `store`.
6. Onboarding no Overlay (login → privacidade → atalho → voz), não em janela própria.
7. Seleção capturada por UI Automation ao abrir o Overlay (sem Ctrl+C sintético).
8. Áudio gravado em PCM + zstd (sem perdas) em vez de Opus.
9. PDF (camada de texto) e WAV processados no próprio host; o worker fica só para ASR local.
10. TTS pelo `Windows.Media.SpeechSynthesis` (offline) em vez de provedor de nuvem.

---

## 6. Roteiro de validação manual no Windows

Roteiros detalhados: `docs/qa/acessibilidade.md`, `docs/qa/instalacao.md`.

**Marco 1 — Overlay e captura**
- [ ] Atalho abre em ≤ 100 ms no monitor do app anterior; fora do Alt+Tab; invisível em Print Screen/Teams.
- [ ] Ctrl+Shift+S → Chip com miniatura; KeePass coberto; `@região` → seletor sobre tela congelada.
- [ ] Buffer de tela "últimos 10 min" ligado → perguntar "o que eu fiz nos últimos 2 minutos?" (`screen_recent`).
- [ ] Gravação manual → Exportar (WAV + MP4 abrem).

**Marco 2 — Conversa**
- [ ] "Continuar com ChatGPT" → modal do plano → resposta em streaming; "Gerenciar uso".
- [ ] Modo Tarefa + pasta concedida: aprovação de comando (A/R); painel Arquivos/Alterações com prévia HTML isolada.
- [ ] Pergunta do agente (formulário) e elicitation MCP; medidor de contexto → compactar.
- [ ] Perda de foco durante resposta → Minibar → notificação ao terminar.

**Marco 3 — BYOK e extensões**
- [ ] Groq/OpenRouter/Anthropic/Ollama com tool call; MCP stdio de terceiros; importar do Claude Desktop.
- [ ] Perfil para `code.exe` → selo "Perfil: VS Code", Chip de tela automático.

**Marco 4 — Voz e anexos**
- [ ] Parakeet v3 baixado; segurar Ctrl+Space → parciais ao vivo → texto no cursor; vocabulário aplicado.
- [ ] Ditado global (Ctrl+Alt+Space) em outro app; "Ouvir" uma resposta.
- [ ] Anexar PDF, XLSX, DOCX, MP3, MP4 → Chips; `attachment_read` por páginas/linhas/tempo.

**Marco 5 — Distribuição**
- [ ] Instalador por usuário; iniciar com o Windows; atualização assinada; diagnóstico exportado sem segredos.
- [ ] `aura-bench` na máquina de referência; E2E verde; auditoria com Narrador.

---

## 7. Fila de trabalho restante

| # | Tarefa | Ticket |
| --- | --- | --- |
| 1 | Compilar o shell Tauri; rodar demo e real | 001 TK-001..005 |
| 2 | Validar `aura-win` (§4.2) | 004, 005, 006, 007, 009 |
| 3 | Login ChatGPT real + turno com o plano; gravar transcripts do app-server | 002 TK-001/002 |
| 4 | Compilar `aura-worker --features engines` (Parakeet/Whisper) | 006 TK-003 |
| 5 | Suporte a `tool_search` na tradução Chat/Anthropic, se aparecer com provedores reais | 003 |
| 6 | Assinatura de código, chaves do updater, primeira release | 010 TK-001/002 |
| 7 | Baseline de desempenho na máquina de referência | 010 TK-004 |

---

## 8. Registro de progresso (Hybrid)

Tickets e evidências foram atualizados nesta sessão (`python .hybrid/hybrid.py render
--project . --effort <id>` para ver). Ao validar no Windows:

```powershell
python .hybrid/hybrid.py evidence add --project . --effort 001-fundacao-overlay --ticket TK-002 `
  --acceptance-refs AC-003 --procedure "Print Screen com Overlay aberto" --result passed --executed `
  --environment "Windows 11 23H2, 150% DPI" --observations "Overlay ausente da captura" --path docs/qa/evidencias/tk-002.png
python .hybrid/hybrid.py ticket update --project . --effort 001-fundacao-overlay --ticket TK-002 --status verified --verification-status passed
python .hybrid/hybrid.py render --project . --effort 001-fundacao-overlay
```

---

## 9. Regras que continuam valendo

- Segredos só no Credential Manager; nunca em logs, `config.toml`, webview, fixtures ou diagnóstico.
- Toda captura passa por `aura_policy::decide` **antes** de acontecer.
- Mudou formato de IPC → `src/ipc/types.ts` + dourado + testes TS.
- Mudou comportamento → spec primeiro; código depois.
- Licenças permissivas (`deny.toml`); modelos, sidecars e capturas fora do git.
