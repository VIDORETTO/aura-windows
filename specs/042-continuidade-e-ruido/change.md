---
schema: hybrid/change
schema_version: 1.0
effort_id: 042-continuidade-e-ruido
revision: 1
status: accepted
profile: compact
---

# Change: Continuidade da conversa e ruído do runtime

## Objetivo e limites

Corrigir problemas observados em conversas reais (rollouts de 04/10 e 12/10/2026) e no `aura.log`:
o agente esquecia, no Turno seguinte, tudo o que pesquisou na web; conversas reabertas após
reinício recebiam um aviso falso de mudança de modo; o servidor MCP do Aura gerava ~4 mil avisos
`resources/list` no log; o log gravava códigos de cor ANSI do app-server.

Decisão do usuário (13/10/2026): privacidade só até onde não prejudica a experiência. O Aura roda
localmente; o conteúdo web lido pelo agente pode ficar no histórico local da Conversa como qualquer
outra mensagem. Isso **revoga**, para Conversas normais, a retenção apenas por Turno dos resultados
web (spec 039, plano TK-004 r7+/ADR 0010 proposto): resultados, argumentos e raciocínio deixam de
passar por envelopes opacos. Conversa efêmera continua sem rollout.

Exclui: atualização do sidecar, mudança de limites/orçamento da web, UI, IPC.

## Contrato de comportamento

- Entradas: Turnos sucessivos de uma Conversa; reinício do app; servidor MCP do Aura consultado pelo Codex.
- Saída: o modelo recebe, em Turnos seguintes e após reinício, as consultas, URLs e textos das
  ferramentas web dos Turnos anteriores; nenhum aviso de modo quando o modo não mudou.
- Erros/invariantes: limites, cancelamento, desativação da web e bloqueio de destinos privados
  permanecem; handles legados de conversas antigas continuam recebendo indisponibilidade.
- Compatibilidade: formatos IPC e banco preservados, exceto coluna nova `instructed_mode`
  (migração aditiva, nula em linhas antigas).

## Requisitos e aceite

- **FR-001** — Resultados e argumentos web de Turnos anteriores DEVEM continuar visíveis ao modelo.
- **AC-001** — Dado Turno 1 com `web_search` + `web_fetch` de página contendo "Sul: 30", quando o
  usuário envia uma pergunta de continuação, então a requisição do Turno 2 ao provedor contém
  "Sul: 30" e a URL lida, e não contém "no longer available".
- **FR-002** — O aviso de modo DEVE ser enviado só quando o modo mudou desde a última instrução dada
  ao modelo, inclusive após reinício.
- **AC-002** — Dada Conversa em Chat, quando o app reinicia e o usuário envia outra mensagem, então a
  mensagem não contém `<aura-mode>`. Dada troca Chat→Tarefa antes do reinício sem nova mensagem,
  quando o usuário envia após reiniciar, então a mensagem contém "Modo Tarefa".
- **FR-003** — O servidor MCP do Aura DEVE responder listas vazias a `resources/list`,
  `resources/templates/list` e `prompts/list`.
- **AC-003** — `resources/list` → `{"resources":[]}`; `resources/templates/list` →
  `{"resourceTemplates":[]}`; `prompts/list` → `{"prompts":[]}`; método desconhecido → -32601.
- **FR-004** — O app-server DEVE ser iniciado sem cores ANSI no stderr encaminhado ao log.
- **FR-005** — O Aura DEVE desligar o catálogo de plugins do Codex (`[features] plugins = false`):
  ele clonava ~94 MB em `codex-home/.tmp/plugins` e chamava chatgpt.com (401) a cada início.

Rodada 2 (testes novos por área, 13/10/2026):

- **FR-006** — Anexo de texto em UTF-16 com BOM (`>` do PowerShell 5, "Unicode" do Bloco de Notas)
  e arquivo vazio DEVEM ser lidos como texto, não "formato não suportado".
- **FR-007** — Lembretes DEVEM aceitar `09:00:00.000`, `09:00-03:00` e `12:00Z` como os modelos escrevem.
- **FR-008** — Se o app-server fecha durante o handshake, o Aura DEVE tentar mais uma vez antes de
  falhar o envio do usuário; falhas repetidas continuam erro.
- **FR-009** — Ferramentas de sub-agentes e "goals" do Codex DEVEM ficar desligadas
  (`multi_agent`/`goals = false`): a UI não as mostra e custavam ~13 KB por requisição (54→46 ferramentas).

## Leitura e mapa de alterações

- `crates/aura-app/src/web.rs` → `output`, `begin_response`, `PrivateCallScope` — devolver resultado web direto; existing.
- `crates/aura-codex/src/service.rs` → `start`, `context`, `send` — modo instruído persistido; existing.
- `crates/aura-store/src/conversations.rs` + `migrations/0016_instructed_mode.sql` — coluna e acesso; new/existing.
- `crates/aura-mcp/src/lib.rs` → `handle_one` — listas vazias; existing.
- `crates/aura-codex/src/launcher.rs` → `strip_ansi`, `NO_COLOR` — log legível; existing.
- `crates/aura-codex/src/home.rs` → `render` — `features.plugins = false`; existing.
- `crates/aura-gateway/src/server.rs` → filtro de stream só com protetor; existing.
- `crates/aura-app/tests/real_app_server.rs` — asserções do contrato revogado trocadas (histórico local mantém conteúdo; Efêmera continua sem gravar); existing.
- `crates/aura-app/tests/conversation_continuity.rs` — testes ponta a ponta; new.

## Plano breve

Seam: `Host` + app-server fixado + provedor Responses roteirizado (`conversation_continuity.rs`);
`router` MCP (`aura-mcp/tests/protocol.rs`); `ConversationsRepo`. Abordagem: remover a camada de
envelopes do caminho novo, mantendo o resolvedor para handles legados.

## Sequência e tarefas

- [x] C-001 AC-001 red (observado em 13/10) → green.
- [x] C-002 AC-002 red (observado em 13/10) → green.
- [x] C-003 AC-003 red → green.
- [x] C-004 FR-004 (teste `color_sequences_are_removed_from_forwarded_lines`).
- [x] C-005 FR-005 (sondagem do app-server fixado: sem `.tmp/plugins` nem chamada a chatgpt.com).
- [x] C-007 FR-006 red (`windows_text_files`: 2 falhas) → green.
- [x] C-008 FR-007 red (`reminder_times`: 2 falhas) → green.
- [x] C-009 FR-008 red (`startup_retry`: Handshake(Closed)) → green.
- [x] C-010 FR-009 (sonda: 54→46 ferramentas, 39→26 KB).
- [x] C-006 Regressão completa (`cargo test`, ignorados com app-server fixado, UI, clippy, fmt).

## Validação e evidência

Comando/procedimento: `cargo test --workspace --exclude aura-desktop`;
`AURA_CODEX_BIN=<fixado> cargo test -p aura-app --test conversation_continuity --test web_research --test real_app_server -- --ignored`.

Resultado executado (13/10/2026, após rodada 2): `cargo test --workspace --exclude aura-desktop` 519 passed;
fmt e `clippy -D warnings` limpos; ignorados com app-server fixado: `conversation_continuity` 6/6,
`web_research` 5/5, `real_app_server` 31/31 (`web_anthropic_thinking_compaction` intermitente uma
vez com "handshake failed" sob 2 threads, verde isolado 2×; `task_mode_confines_writes` exige
`AURA_SBX_OUTSIDE` fora de `%TEMP%` quando `AURA_E2E_DIR` está em `%TEMP%`, pois o sandbox de
Tarefa permite `%TEMP%` por desenho). UI 253/253 e typecheck sem mudança de UI.
E2E WebDriver da UI não executado: falta `msedgedriver` da versão do WebView2 nesta máquina.

Limitações: comportamento do modelo real (ChatGPT) não é exercitado; o provedor é roteirizado.

## Publicação 0.3.1

Pedido do usuário (13/10/2026): atualizar main e a release. Versão 0.3.1 em Cargo/package/
tauri.conf; ADR 0011 substitui 0010. Checks antes do commit: fmt, `clippy --workspace
--all-targets -D warnings` (inclui shell Tauri), 519 testes Rust, cargo-deny 0.20.2 `licenses ok,
sources ok`. `prepare-sidecars.ps1 -Release -DirectML` e `tauri build --ci --bundles nsis,msi`
exit 0; assinaturas NSIS/MSI aceitas pela pública permanente e rejeitadas pela aposentada e com
bytes adulterados (`updater-verify`); ProductVersion 0.3.1 em aura.exe, NSIS e MSI. Smoke do
aura.exe de produção em perfil isolado: inicia, app-server pronto, `features` desligadas, sem
`.tmp/plugins` nem ANSI no log.

| Asset | SHA-256 |
| --- | --- |
| Aura_0.3.1_x64-setup.exe | `def9e599324a9e0221dad7b75960ed302dcc840984fc2ee5ccb506efb2438197` |
| Aura_0.3.1_x64_en-US.msi | `b94fec113cb01e2a593091b82dfe5b6861ae92f755506ae0433518f835236980` |

Não executado: instalação/atualização em Windows limpo e modelo real do ChatGPT.

## Condição de retorno

Retornar se o app-server fixado rejeitar resultados web diretos ou se a migração não for aditiva.

## Estado

`change.md` é a fonte canônica deste esforço compacto. Não crie `tasks.md` ou tickets paralelos para esta mudança.
