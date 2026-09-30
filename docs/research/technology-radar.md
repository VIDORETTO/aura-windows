# Radar de tecnologia

Estado observado em 2026-09-29 (estrelas/licença via GitHub API). Versões exatas são fixadas no momento da implementação de cada ticket e registradas no `plan.md` do esforço; esta página orienta a escolha.

## Adotar

| Área | Escolha | Licença | Por quê |
| --- | --- | --- | --- |
| Shell desktop | Tauri 2 (`tauri-apps/tauri`, ~111k★) | Apache-2.0/MIT | Rust + WebView2, binário pequeno, tray, multi-janela, updater, sidecars |
| UI | React 19 + TypeScript + Vite + Tailwind CSS 4 | MIT | Ecossistema maduro para chat (markdown em streaming, highlight), mesmo stack de ThukiWin |
| Tipos Rust→TS | `tauri-specta` / `specta` | MIT | Comandos e eventos tipados ponta a ponta (padrão usado pelo Handy) |
| Harness agêntico | `codex-app-server` fixado por release | Apache-2.0 | Ver `harness-evaluation.md` |
| Captura de tela | `windows-capture` (Windows.Graphics.Capture, ~0.5k★) | MIT | Rápido, suporta encoder de vídeo por Media Foundation |
| Áudio | `wasapi` (loopback de sistema) + `cpal` (microfone) | MIT/Apache | Loopback nativo do WASAPI e captura de mic multiplataforma |
| Reamostragem | `rubato` | MIT | 16 kHz mono para ASR |
| VAD | Silero via `vad-rs`/ONNX | MIT | Corte de silêncio para ditado e buffer |
| ASR local | `transcribe-rs` (ONNX: Parakeet, Moonshine, SenseVoice, Canary) + `transcribe-cpp` (Whisper GGUF) | MIT | Mesmo motor do Handy (~32k★); cobre modelos rápidos em CPU e Whisper com GPU |
| Inferência ONNX | `ort` | Apache-2.0 | DirectML no Windows |
| Download de modelos | `reqwest` com retomada + SHA-256 (`sha2`) | MIT/Apache | Downloads verificáveis e canceláveis |
| Banco local | SQLite (`rusqlite` bundled) + FTS5 | MIT | Histórico, índice, metadados; sem servidor |
| Criptografia em repouso | AES-256-GCM (`aes-gcm`) com chave protegida por DPAPI (`CryptProtectData`) | MIT/Apache | Chave atrelada ao usuário do Windows |
| Segredos | Windows Credential Manager (`keyring` crate) | MIT/Apache | Chaves BYOK fora de arquivos e fora do webview |
| OCR | `Windows.Media.Ocr` (WinRT, crate `windows`) | MIT | Sem modelo extra, rápido, idiomas do sistema |
| Acessibilidade | UI Automation (crate `windows`/`uiautomation`) | MIT | Texto estruturado da janela ativa e seleção |
| Hotkeys | `tauri-plugin-global-shortcut` + hook `WH_KEYBOARD_LL` para duplo toque | MIT/Apache | Combinações comuns e gesto de duplo toque |
| PDF | `pdfium-render` (binário PDFium) | MIT/Apache | Texto + renderização de páginas em imagem |
| Planilhas | `calamine` (xlsx/xls/ods) | MIT | Leitura rápida sem Office |
| DOCX/PPTX | `docx-rs`/extração XML direta | MIT | Texto estruturado |
| Vídeo | Media Foundation (decodificação de keyframes) | — | Evita embutir FFmpeg (~80 MB); FFmpeg opcional baixável |
| MCP server do Aura | `rmcp` (SDK Rust oficial do MCP) | MIT/Apache | Expor ferramentas de tela/áudio ao Codex |
| HTTP gateway | `axum` + `tokio` | MIT | Gateway Responses local |
| Testes Rust | `cargo nextest`, `insta` (snapshots de contrato), `wiremock` | MIT/Apache | Contratos de protocolo e gateway |
| Testes UI | Vitest + Testing Library + `@tauri-apps/api/mocks` | MIT | Seam de IPC controlado |
| E2E | WebdriverIO + `tauri-driver` em runner Windows | MIT | Jornadas críticas no binário real |
| Instalador | NSIS (Tauri bundler), assinatura Authenticode | — | Instalação por usuário, sem admin |

## Avaliar

- **Silero/earshot** para VAD de baixa latência em streaming.
- **Moonshine streaming** para parciais em tempo real no push-to-talk.
- **Foundry Local / Windows ML** para ASR e modelos em NPU (AnythingLLM já integra).
- **Realtime do Codex** (`thread/realtime/*`, experimental) para modo voz em tempo real usando a assinatura.

## Evitar

- Electron (memória ociosa alta, contradiz "leve").
- Python embutido (Clicky) — distribuição frágil.
- FFmpeg obrigatório no instalador.
- Código do screenpipe (licença não permissiva para uso comercial).
