---
schema: hybrid/change
schema_version: 1.0
effort_id: 016-app-desktop-e-auditoria
revision: 1
status: closed
profile: compact
---

# Change: Menu nativo, diálogos do navegador, histórico e auditoria visual

## Objetivo e limites

Pedido do usuário em 04/10/2026: "quando eu clico com o botão direito no aplicativo, aparece para salvar e umas opções muito esquisitas, como se estivesse em uma página web"; e um novo estudo end-to-end do app inteiro, corrigindo o que for encontrado.

Achados (tour com capturas no build demo nativo, leitura de código e jornadas E2E com o app-server real):

1. Clique direito mostrava o menu de página do WebView2 (Voltar, Recarregar, Salvar como, Imprimir, Inspecionar…); atalhos de navegador (F5, Ctrl+P, Ctrl+S, Alt+←, Ctrl+F) também ativos.
2. Excluir conversa, apagar todos os dados e esquecer memórias usavam `window.confirm` (diálogo do navegador "tauri.localhost diz…").
3. Histórico perdia conversas no "Carregar mais": o cursor do app-server tem resolução de segundos e é exclusivo, então conversas no mesmo segundo da última linha nunca apareciam (reproduzido: 2ª página com 1 de 5 conversas).
4. "Arquivar" escondia a conversa sem caminho de volta.
5. O "Modelo padrão" das Configurações (modelo do plano ChatGPT) era aplicado a qualquer provedor e vencia o modelo do Perfil de aplicativo; o perfil BYOK (`aura-<id>::<modelo>`) seria repassado cru.
6. Visual: rótulos do `@` com espaço/reticências ("@janela ativa", "@arquivo…", "@últimos minutos") impossíveis de digitar; dicas do `/` e lista de Comandos rápidos com marcadores crus (`{selecao}`, `{tela}`, "Traduza para :"); Configurações mantinham a rolagem ao trocar de página; tabela de esforço vazava do cartão; "Servidores MCP" duas vezes no Diagnóstico; contagem de modelos duplicada em Provedores; "Memórias" em dois lugares; Sobre sem versão; aprovação respondida dizia só "Respondido" e mostrava "em ."; seletor mostrava "Modelo padrão" sem dizer qual.

Fora do escopo (candidatos no roadmap): recursos novos de concorrentes (ver CAND-025…CAND-032).

## Contrato de comportamento

- Clique direito: em campos de texto, só ações de edição (emoji, desfazer/refazer, recortar, copiar, colar, colar sem formatação, selecionar tudo); em texto selecionado, copiar; no resto, nenhum menu. Atalhos de navegador desligados. Builds de debug mantêm tudo (DevTools).
- Ações destrutivas confirmam dentro do app (pergunta + Confirmar/Cancelar); nada usa `window.confirm`.
- Histórico: "Carregar mais" alcança todas as conversas; "Arquivadas" lista as arquivadas, com "Desarquivar".
- Modelo de nova conversa: escolha do seletor > padrão do Perfil (só para o seu provedor) > padrão das Configurações (só no plano ChatGPT). O seletor mostra o nome do modelo efetivo.
- Correções visuais listadas no item 6.

## Requisitos e aceite

- **AC-001** — Clique direito real (mouse_event) no Overlay nativo: no cabeçalho nenhum menu; no campo de texto só Emoji, Desfazer, Recortar, Copiar, Colar, Colar sem formatação e Selecionar tudo.
- **AC-002** — Excluir conversa, apagar todos os dados e esquecer memórias mostram a pergunta no app; Cancelar não executa; Confirmar executa; `window.confirm` nunca é chamado.
- **AC-003** — Com 55 conversas criadas em sequência, após "Carregar mais" todas as "QA page 001…055" aparecem (app-server real).
- **AC-004** — Arquivar tira da lista; "Arquivadas" mostra; "Desarquivar" devolve (app-server real e UI).
- **AC-005** — `start_model`: BYOK sem escolha não recebe o padrão do plano; Perfil vence o padrão global; perfil BYOK só vale para o seu provedor.
- **AC-006** — Itens do `@`: @tela, @região, @janela, @seleção, @arquivo, @recente; "Traduza para ‹inglês›:" no `/` e em Extensões; cada página de Configurações abre no topo; aprovação mostra "Aceito"/"Recusado"; seletor mostra "GPT-5.5" (padrão do plano) ou o padrão das Configurações.

## Leitura e mapa de alterações

- `crates/aura-app/src/webview.rs` → `context_menu_keep`; new. `apps/desktop/src-tauri/src/webview_guard.rs` → plugin `on_webview_ready`; new.
- `apps/desktop/src/ui/ConfirmButton.tsx`; new. `settings/{Diagnostics,General,Extensions,Providers,EffortPresets,SettingsApp}.tsx`, `overlay/{HistoryPanel,InputBar,Messages,ModelPicker,OverlayApp,composer}.ts(x)`; existing.
- `crates/aura-codex/src/service.rs` → `page_cursor`, `unarchive`; `fake.rs`; `crates/aura-app/src/host.rs` → `start_model`, `unarchive`; shell `conversation_unarchive`.
- E2E: `history.e2e.ts` (todas as 55), `memories.e2e.ts` (Confirmar), `recent-clip.e2e.ts` (seletor `@recent`).

## Plano breve

Seam: funções puras (`context_menu_keep`, `start_model`, `page_cursor`, `templatePreview`), serviço contra o app-server falso, Vitest na UI, app-server real (testes ignorados) e jornadas nativas. Dependências: `webview2-com` 0.39 e `windows` 0.62 (mesmas versões do Tauri/wry; MIT/Apache).

## Sequência e tarefas

- [x] C-001 Menu nativo e atalhos de navegador (regra pura + shell + clique real).
- [x] C-002 Confirmação no app, arquivadas/desarquivar, paginação do Histórico.
- [x] C-003 Modelo padrão por provedor; correções visuais.
- [x] C-004 Regressão completa, tour nativo, evidência.

## Validação e evidência

Comando/procedimento: `cargo test --workspace --exclude aura-desktop`, `cargo clippy --workspace --all-targets -- -D warnings`, `pnpm -C apps/desktop test`, `pnpm -C apps/desktop typecheck`, `AURA_CODEX_BIN=… cargo test -p aura-app --test real_app_server -- --ignored`, jornadas nativas (demo: overlay, settings, composer; produção com o app-server real: compact, mode-switch, skills, model-efforts, memories, history, panels), clique direito real com captura, tour de capturas.

Resultado executado: EV-001 — Rust 319 passados + 4 ignorados (os ignorados com app-server real executados à parte: sandbox, arquivar/desarquivar); clippy limpo; UI 177/177; typecheck limpo; jornadas nativas verdes; History "Carregar mais" com 56 linhas (55 + a renomeada), antes 51; clique direito: campo de texto só com edição, cabeçalho sem menu; tour confirma as correções visuais.

Limitações: o "antes" do clique direito não pôde ser fotografado (o Overlay de produção é excluído das capturas); `recent-clip.e2e.ts` não foi concluída (exige áudio real tocando e transcrição; o seletor corrigido abre o cartão); a correção de paginação depende do formato de cursor do app-server fixado (cursor desconhecido passa intacto); a trava de "Carregar mais" sem novidades encerra a paginação se mais de 50 conversas compartilharem o mesmo segundo.

## Condição de retorno

Retornar se o contrato, símbolo, dependência ou decisão material não puder ser satisfeito sem ampliar o escopo.

## Estado

`change.md` é a fonte canônica deste esforço compacto. Não crie `tasks.md` ou tickets paralelos para esta mudança.
