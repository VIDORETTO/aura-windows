---
schema: hybrid/change
schema_version: 1.0
effort_id: 015-composer-e-conversa
revision: 1
status: closed
profile: compact
---

# Change: Composer (@ e /), fila, modo visível e ações nas mensagens

## Objetivo e limites

Pedido do usuário em 04/10/2026 após a análise end-to-end: "o `@` e o `/` no chat estão muito confusos"; corrigir e melhorar o que a análise encontrou.

Achados tratados (uso real em `%LOCALAPPDATA%\Aura` + leitura de `InputBar.tsx`):

1. Enter com `@palavra` no fim do texto executava a captura de tela (o menu `@` não filtrava e o primeiro item era escolhido).
2. Skills apareciam como `/nome` e inseriam `$nome` no texto.
3. `/` sozinho + Enter enviava "/" como mensagem; Esc no menu `/` apagava o texto todo.
4. Menus só reconheciam o gatilho no fim do texto (ignoravam o cursor); comandos embutidos só em português; `/tela` duplicava `@tela`.
5. Enter durante uma resposta não fazia nada, sem aviso.
6. O modo (Chat/Tarefa/Plano) não aparecia no Overlay compacto; no uso real o usuário repetiu o mesmo pedido em três conversas achando que estava em Tarefa.
7. Sem "tentar de novo", "editar" ou "refazer no Modo Tarefa" nas mensagens.
8. Clicar no medidor de contexto compactava sem confirmar.
9. A Persona não pedia busca na web para dados atuais (o agente disse não poder confirmar resultados ao vivo com `web_search = "live"` ligado).

Investigado e **sem defeito**: a pasta criada na Área de Trabalho em Modo Tarefa sem pasta concedida veio de um pedido de escalonamento (`sandbox_permissions: require_escalated`), que gera Aprovação; escrita direta fora do workspace é negada pela sandbox. Fica coberto por `real_app_server_task_mode_confines_writes`.

Fora do escopo: `thread/fork`/`thread/revert`/`thread/queue` do app-server (não documentados no contrato fixado); submenus de `@janela`, `@arquivo` com busca no workspace e `@conversa`; comandos que substituem a seleção; ditado global; integração com o Explorer (viram candidatos do roadmap).

## Contrato de comportamento

- Entradas: texto da barra de entrada com posição do cursor; teclas Enter/Tab/Esc/setas; Catálogo de skills; comandos rápidos; estado `running` da Conversa; modo da sessão.
- Saída:
  - `/` = fazer algo (comandos embutidos, Comandos rápidos, Skills), em seções "Comandos" e "Skills". `@` = contexto (tela, região, janela, seleção, arquivo, recente), em seção "Contexto". Os dois menus abrem quando o gatilho está no início de uma palavra **na posição do cursor** e filtram pelo que foi digitado, sem diferenciar acentos nem maiúsculas.
  - Escolher uma Skill cria um Chip de contexto de Skill (o host prefixa `$nome` no turno); nada de `$` aparece no texto.
  - Nomes embutidos no idioma da interface (`/plano`, `/compactar` em pt-BR; `/plan`, `/compact` em inglês); os dois idiomas continuam aceitos ao enviar. `/tela` sai do menu (continua aceito digitado).
  - Com um Comando rápido digitado, uma linha de dica mostra o argumento (`‹inglês›`) e de onde vem o texto (seleção ou texto digitado).
  - Enter durante uma resposta coloca a mensagem na fila (Chip "Na fila", cancelável) e ela é enviada quando o turno termina; Ctrl+Enter orienta o turno em andamento.
  - Overlay compacto mostra o modo atual; clicar abre a escolha do modo.
  - Última resposta do agente: "Tentar de novo" reenvia o último pedido; em Chat/Plano, "Refazer no Modo Tarefa" troca o modo e reenvia. Mensagem do usuário: "Editar" devolve o texto à barra de entrada.
  - O medidor de contexto abre uma confirmação antes de compactar.
- Erros/invariantes: Enter nunca executa um item `@` cujo filtro não casou; com o gatilho sozinho (`/` ou `@`) Enter só escolhe se o usuário navegou com as setas e nunca envia o gatilho como mensagem; Esc fecha o menu sem apagar o texto; Skill desativada ou inexistente → erro `skill`; a mesma Skill não duplica o Chip.
- Compatibilidade: Ctrl+Shift+Enter (inserir no app), comandos digitados por extenso, aliases `plan`/`plano`, `compact`/`compactar`, `screen`/`tela`, Comandos rápidos e o fluxo de steer continuam iguais.

## Requisitos e aceite

- **FR-001** — Os menus `/` e `@` MUST seguir o cursor, filtrar sem acento e não executar ações que o usuário não escolheu.
- **FR-002** — Escolher uma Skill MUST criar um Chip de Skill.
- **FR-003** — Enter durante uma resposta MUST enfileirar a mensagem.
- **FR-004** — O modo MUST ficar visível no Overlay compacto.
- **FR-005** — As mensagens MUST oferecer tentar de novo, editar e refazer no Modo Tarefa.
- **AC-001** — Dado o texto "fala com o @joao", quando o usuário pressiona Enter, então a mensagem "fala com o @joao" é enviada e nenhuma captura acontece; com "@regiao" o menu mostra só "@região".
- **AC-002** — Dado "/" sozinho, quando Enter, então nada é enviado; Esc com "/tl" fecha o menu e mantém "/tl"; com o cursor depois de "veja " em "veja  agora", digitar "@" abre o menu de contexto.
- **AC-003** — Dada a Skill `revisar-contrato`, quando escolhida em `/rev`, então o texto fica vazio e aparece o Chip "revisar-contrato"; o host devolve um Chip `skill` com payload `{name, path}`; escolher de novo não duplica; Skill desativada devolve erro `skill`.
- **AC-004** — Com a interface em inglês, o menu `/` mostra `/plan` e `/compact` e não mostra `/tela`/`/screen`; em pt-BR mostra `/plano` e `/compactar`.
- **AC-005** — Dado "/traduzir " digitado, então a dica mostra "‹inglês›" e "usa a seleção ou o texto digitado".
- **AC-006** — Dada uma resposta em andamento, quando o usuário escreve "e depois?" e pressiona Enter, então aparece "Na fila: e depois?"; ao terminar o turno a mensagem é enviada uma vez; cancelar a fila descarta.
- **AC-007** — No Overlay compacto aparece "Chat"; escolher "Tarefa" troca o modo da sessão.
- **AC-008** — Na última resposta de uma Conversa em Chat, "Refazer no Modo Tarefa" troca para Tarefa e reenvia o último pedido; "Tentar de novo" reenvia sem trocar; "Editar" numa mensagem do usuário põe o texto na barra.
- **AC-009** — Clicar no medidor de contexto mostra "Compactar a conversa?"; só "Compactar" chama a compactação. O botão de janela "Compactar" passa a "Recolher" ("Collapse") para não competir com a compactação da conversa.
- **AC-010** — A Persona (pt-BR e en) orienta usar a busca na web para dados que mudam (notícias, resultados, preços, placares).

## Leitura e mapa de alterações

- `apps/desktop/src/overlay/composer.ts` → `findTrigger`, `filterMenu`, `normalize`, `quickHint`; new (lógica pura testável).
- `apps/desktop/src/overlay/InputBar.tsx` → menus, fila, dica, combobox; existing.
- `apps/desktop/src/overlay/session.ts` → `queued`, `queue`, `flushQueue`, `retryLast`, `attachSkill`; existing.
- `apps/desktop/src/overlay/Header.tsx` → `ModeBadge` no compacto; existing.
- `apps/desktop/src/overlay/Messages.tsx` → ações nas mensagens; existing.
- `apps/desktop/src/overlay/StatusBar.tsx` → confirmação do medidor; existing.
- `crates/aura-app/src/host.rs` → `attach_skill`; existing.
- `apps/desktop/src-tauri/src/*` → comando `context_attach_skill`; existing.
- `apps/desktop/src/ipc/{commands,mock,types}.ts`, contrato dourado; existing.
- `crates/aura-codex/persona/{pt-BR,en}.md` → busca para dados atuais; existing.
- `crates/aura-app/tests/real_app_server.rs` → `real_app_server_task_mode_confines_writes`; new.

## Plano breve

Seam: funções puras de `composer.ts` e `InputBar`/`OverlayApp` no Vitest; `Host::attach_skill` no teste do host com o app-server falso. Abordagem: fila no cliente (estado da sessão), sem API experimental do app-server; "tentar de novo" reenvia o texto como novo turno (sem reverter o anterior). Dependências: none.

## Sequência e tarefas

- [x] C-001 Red/green de `composer.ts` (gatilho no cursor, filtro sem acento, dica).
- [x] C-002 Red/green do host `attach_skill` + IPC/contrato.
- [x] C-003 Red/green da UI: menus, Skill como Chip, fila, modo no compacto, ações nas mensagens, medidor.
- [x] C-004 Persona; regressão completa (Rust, UI, typecheck, clippy) e evidência.

## Validação e evidência

Comando/procedimento: `cargo test -p aura-app --test host attach_skill`, `pnpm -C apps/desktop test`, `pnpm -C apps/desktop typecheck`, `cargo test --workspace --exclude aura-desktop`, `cargo clippy --workspace --all-targets -- -D warnings`, `AURA_CODEX_BIN=… cargo test -p aura-app --test real_app_server real_app_server_task_mode_confines_writes -- --ignored`.

Resultado executado: EV-001 — `composer.ts` 11/11; host `attach_skill` verde; UI 169/169 (inclui 11 casos de `Composer.test.tsx`); Rust 313 passados + 3 ignorados; clippy e fmt limpos; typecheck limpo. Sandbox no app-server real: escrita direta fora do workspace negada (pasta fora do TEMP, Área de Trabalho local e do OneDrive) e escalonamento gera 1 Aprovação; recusada, nada é escrito. Nativo (WebView2 154): build demo — `composer`, `overlay`, `settings`; build de produção com o app-server real — `compact`, `mode-switch`, `skills`, `model-efforts`, `panels` (provedor loopback). Regressão encontrada no nativo e corrigida com teste: com a interface em inglês, Enter em `/compactar` trocava o texto por `/compact` em vez de compactar.

Limitações: "Tentar de novo" reenvia o pedido como novo turno (o turno anterior continua no histórico; `thread/revert` fica para CAND-021); a fila é do cliente (perde-se se o app fechar); o comportamento do modelo real diante da nova orientação de busca na Persona não foi avaliado; a pasta na Área de Trabalho do uso real veio de uma Aprovação que o registro do app-server não guarda (inferido pelo pedido `require_escalated` com justificativa).

## Condição de retorno

Retornar se o contrato, símbolo, dependência ou decisão material não puder ser satisfeito sem ampliar o escopo.

## Estado

`change.md` é a fonte canônica deste esforço compacto. Não crie `tasks.md` ou tickets paralelos para esta mudança.
