---
schema: hybrid/plan
schema_version: "1.0"
effort_id: 004-contexto-de-tela
revision: 1
spec_revision: 1
status: ready
---

# Plan: Contexto de tela e privacidade

## Summary

Três Modules profundos: `aura-policy` (puro: decide cada acesso), `aura-capture` (captura WGC, inventário de janelas, redação, região, buffer segmentado, OCR/UIA) e `aura-mcp` (servidor MCP local que expõe as Ferramentas do Aura e consulta a Política). O host liga tudo: Chips via `ContextTray` (002), cartões de permissão no Overlay, indicadores na bandeja e Registro de acesso no `aura.db`.

## Technical context

- Language/runtime: Rust; WinRT/Win32 via crate `windows`.
- Dependencies (fixar e consultar docs atuais no ticket): `windows-capture` (WGC + `VideoEncoder` Media Foundation), `windows` (`EnumWindows`, `DwmGetWindowAttribute(DWMWA_EXTENDED_FRAME_BOUNDS)`, `IsWindowVisible`, `Windows.Media.Ocr`, UI Automation `IUIAutomation`, `TextPattern`), `image`/`fast_image_resize` (miniaturas, redução), `webp`, `rmcp` (servidor MCP streamable HTTP), `axum` (montado no mesmo servidor loopback do Gateway), `aes-gcm` via `Vault::seal_bytes` (001 TK-006).
- Storage/data: `captures\screen\<yyyy-mm-dd>\<segment-id>.seg` (fMP4 H.264 cifrado); tabelas `capture_segments`, `recordings`, `access_log`, `exclusion_rules`, `agent_grants` (migração `0004_capture.sql`).
- Test command: `cargo nextest run -p aura-policy -p aura-capture -p aura-mcp`; testes Windows `--features win-integration`; `pnpm -C apps/desktop test -- capture privacy`.
- Target/platform: Windows 10 2004+/11.

## Consumed contract

- Spec: `spec.md`, revision 1.
- Requirements and acceptance refs: FR-001–FR-007; AC-001–AC-016.

## Modules, interfaces, consumers, and seams

| Module | Interface | Consumidores | Seam de teste |
| --- | --- | --- | --- |
| `aura-policy` | `Policy`, `SourcePolicy{mode: CaptureMode, agent: AgentPermission}`, `ExclusionRule{process?, title_glob?, class?}`, `fn decide(&Policy, &AccessRequest, &Grants) -> Decision{Allow, AllowRedacted(Vec<Rect>), Ask, Deny(DenyReason)}`, `default_exclusions()` | host, `aura-mcp`, 005 | Puro: tabelas de casos |
| `aura-capture::source` | `trait FrameSource { capture_monitor(MonitorId) -> Frame; capture_window(WindowId) -> Frame; stream(Target, Fps) -> FrameStream }` | host, buffer | Adapters reais: `WgcSource` (produção) e `SyntheticSource` (testes, pixels conhecidos) |
| `aura-capture::windows` | `trait WindowInventory { visible_windows(MonitorId) -> Vec<WindowInfo{id, pid, process, title, class, rect, z}> }` | redação, política | `Win32Inventory` e `StaticInventory` |
| `aura-capture::redact` | `fn redact(frame, windows, decision_rects) -> Frame` (pinta blocos opacos com cadeado) | host | Puro sobre `Frame` sintético |
| `aura-capture::region` | Janela seletora (Tauri, transparente, congelada) → `Rect` físico | host | Manual + Vitest do componente |
| `aura-capture::segments` | `SegmentRecorder::{start(target, fps, kind), stop}`; `SegmentStore::{index, range(from,to), delete}`; `retention::plan(&[SegmentMeta], &RetentionPolicy, now) -> Vec<SegmentId>` (puro) | buffer, gravação, 005 | `plan` puro; recorder com `SyntheticSource` + encoder real no Windows |
| `aura-capture::text` | `trait ScreenText { uia_text(WindowId, max_chars) -> Option<String>; ocr(Frame, lang) -> String }` | `aura-mcp` | Windows integração com janela de teste conhecida |
| `aura-mcp` | Servidor MCP em `/mcp`; tools `screen_capture{target, reason}`, `active_window_info{}`, `screen_text{source}`, `screen_recent{minutes, max_frames}`; `trait ToolHost { async fn request_permission(...) -> PermissionAnswer; async fn capture(...); … }` | Codex app-server | Cliente MCP de teste (`rmcp` client) contra o servidor com `ToolHost` falso |
| `apps/desktop/src-tauri/src/capture.rs` | comandos `capture_screen_chip`, `capture_region_chip`, `privacy_pause`, `access_log_list`, `recordings_*` | UI | E2E/Vitest |
| `apps/desktop/src/privacy/`, `src/recordings/`, `src/conversation/chips/` | UI de privacidade, registro, gravações/player, Chips de tela/região/Recorte | usuário | Vitest + mockIPC |

Identificação da Conversa em chamadas MCP: o Aura gera `conversation_uuid` antes do `thread/start` e passa `config: {"mcp_servers.aura.http_headers": {"X-Aura-Conversation": "<uuid>"}}` no `thread/start` (override por thread). O servidor MCP usa o header para concessões "nesta Conversa" e para o Registro de acesso. Confirmar no TK-004 que overrides por thread aplicam-se a headers MCP; se não, usar a Conversa com turno ativo (única) e pedir escolha quando houver mais de uma.

Formato de segmento: encoder Media Foundation H.264 escreve fMP4 de 10 s num `IMFByteStream` em memória; ao fechar, `Vault::seal_bytes` com chave derivada `"capture-segment"` e nonce aleatório → arquivo `.seg`. Nenhum texto claro vai ao disco. Leitura/decodificação para keyframes ocorre no `aura-worker` (ADR 0005, `media.keyframes`), que recebe bytes decifrados por pipe.

Player de Gravações: protocolo customizado `aura-media://recording/<id>/<n>` servindo segmentos decifrados; `<video>` com Media Source Extensions concatenando fMP4.

## Chosen approach and alternatives

- **Política pura e única** em crate próprio: facilita provar regras e reutilizar em áudio.
- **Redação por cobertura** (bloco opaco) em vez de recusar a captura inteira quando só parte da tela é sensível.
- **MCP (ADR 0006)** com `default_tools_approval_mode = "auto"` no Codex para as tools do Aura, porque a Permissão do agente do Aura é quem decide (evita dupla aprovação). Ações sem efeito colateral fora do Aura.
- **Buffer em disco cifrado** (ADR 0004).
- Alternativas descartadas: GDI `BitBlt` (ThukiWin) — lento e não respeita `WDA_EXCLUDEFROMCAPTURE` de forma confiável; screenshots JPEG por evento para o buffer (ver ADR 0004).

## Data, compatibility, and external dependencies

`0004_capture.sql`: `capture_segments(id, source, kind, recording_id, start_ms, end_ms, path, bytes)`, `recordings(id, source, started_at, ended_at, title)`, `access_log(id, at, source, requester, tool, conversation, decision, reason, artifact_path)`, `exclusion_rules(id, process, title_glob, class, enabled, builtin)`, `agent_grants(conversation, source, scope, expires_at)`. Varredura de inicialização remove `.seg` sem índice e índices sem arquivo.

Lista padrão de exclusão (processos): `KeePass.exe`, `KeePassXC.exe`, `Bitwarden.exe`, `1Password.exe`, `LastPass*.exe`, `Dashlane*.exe`, `NordPass*.exe`, `CredentialUIBroker.exe`, `consent.exe`, `SecHealthUI.exe`; títulos: `*InPrivate*`, `*Incognito*`, `*Navegação anônima*`, `*Private Browsing*`.

## Verification strategy

| AC | Nível | Oráculo | Procedimento |
| --- | --- | --- | --- |
| AC-001 | Integração Windows + medição | Spec (300 ms, Overlay ausente) | Overlay visível com magenta (001) + `capture_screen_chip` → imagem sem magenta; tempo do comando → Chip ≤ 300 ms p95 em 20 execuções |
| AC-002 | Integração Windows | Spec | Janela de teste com cor sólida atrás de outra → captura de janela contém só a cor |
| AC-003 | Unit política + Vitest | Spec | Aplicativo anterior excluído → Chip bloqueado com motivo |
| AC-004 | Manual + Vitest do seletor | Spec | Roteiro 2 monitores/DPI; `Esc` cancela |
| AC-005 | Unit `decide` + unit `redact` + integração com processos dublês | Lista padrão | Janela dublê `KeePassXC.exe` sobre metade da tela → retângulo coberto; em tela cheia → Deny |
| AC-006 | Unit + integração | Spec | Pausa → `decide` = Deny(Paused) para qualquer requisição; buffer para de gravar (nenhum segmento novo em 30 s) |
| AC-007 | Unit + cliente MCP de teste + Vitest | Spec | Nunca → erro MCP com motivo; Perguntar → `ToolHost::request_permission` chamado; Sempre → imagem |
| AC-008 | Integração repo + Vitest | Spec | Cada chamada gera linha no `access_log` com campos esperados |
| AC-009 | Vitest + manual | Spec | Indicadores por estado |
| AC-010 | `codex-e2e`/manual real | Spec | Pergunta sem anexo → Item `mcpToolCall aura.screen_capture` e resposta coerente |
| AC-011 | Integração Windows | Janela de teste com texto conhecido | UIA retorna "Texto de teste Aura 123"; janela desenhada sem UIA → OCR retorna o texto |
| AC-012, AC-014 | Integração (buffer sintético) | Spec (amostragem uniforme, máx. 8) | Buffer com frames numerados 0..119 (2 min a 1 fps) → 8 keyframes ≈ {0,17,34,…,119} |
| AC-013, AC-016 | Unit `retention::plan` + integração | Spec | 20 min simulados → só últimos 5 min; limites 7 dias/20 GB |
| AC-015 | Integração Windows + manual player | Spec | Gravar 30 s → `recordings` com duração 30 ± 1 s; player reproduz |

## Change map

| Path | Existing/new | Symbol or section | Purpose | Reference revision |
| --- | --- | --- | --- | --- |
| `crates/aura-policy/src/{lib.rs,rules.rs,decide.rs,defaults.rs}` | new | `decide`, `default_exclusions` | Política | 2026-09-29 |
| `crates/aura-capture/src/{source.rs,wgc.rs,windows.rs,redact.rs,segments.rs,retention.rs,text.rs}` | new | ver tabela | Captura | 2026-09-29 |
| `crates/aura-mcp/src/{lib.rs,tools/screen.rs,host.rs}` | new | servidor/tools | MCP | 2026-09-29 |
| `crates/aura-store/src/migrations/0004_capture.sql`, `access_log_repo.rs` | new | tabelas | Dados | 2026-09-29 |
| `apps/desktop/src-tauri/src/{capture.rs,region_window.rs,media_protocol.rs}` | new | comandos/janela/protocolo | Host | 2026-09-29 |
| `apps/desktop/src/{privacy,recordings}/**`, `conversation/chips/*` | new/existing | UI | Interface | 2026-09-29 |
| `crates/aura-gateway/src/server.rs` | existing (002 TK-002) | montar `/mcp` | Servidor loopback compartilhado | 002 TK-002 |

## Derived technical obligations

- **OT-001** → FR-003: nenhum pixel sai de `aura-capture` sem passar por `decide` + `redact` (API pública de captura exige `Decision`).
- **OT-002** → FR-003: `decide` é determinística e sem E/S; concessões e pausa entram como dados.
- **OT-003** → FR-005: segmentos só tocam o disco cifrados.
- **OT-004** → FR-004: tools MCP retornam erro estruturado `{code: "denied"|"paused"|"excluded"|"unavailable", message}` e nunca imagem parcial sem redação.
- **OT-005** → 005: `aura-policy` e `segments` são genéricos por `Source` (Screen, Mic, SystemAudio).

## Risks and gates

- Borda amarela da WGC no Windows 10 durante buffer contínuo. Mitigação: documentar; no Windows 11 usar `IsBorderRequired=false`.
- `WDA_EXCLUDEFROMCAPTURE` pode deixar áreas pretas em vez de "transparentes" dependendo do compositor — aceitável (Overlay não aparece).
- Encoder de hardware ausente (VMs) → fallback software a 0,5 fps com aviso.
- G2: satisfeito. G3: TK-001 depende de 001 TK-002/TK-003 e 002 TK-005 concluídos; TK-004 depende de 002 TK-002 (servidor loopback) e do resultado de H-003 (002 TK-001); TK-007 depende do `aura-worker` (006 TK-003).
