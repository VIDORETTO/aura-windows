---
schema: hybrid/ticket
schema_version: 1.0
id: TK-001
effort: 001-fundacao-overlay
type: delivery
status: implemented
ticket_revision: 5
requires: []
requirement_refs: ["FR-001"]
acceptance_refs: ["AC-001", "AC-002", "AC-003"]
spec_revision: 2
plan_revision: 1
owned_areas: ["Cargo.toml", "rust-toolchain.toml", ".github/workflows", "apps/desktop", "crates/aura-core"]
verification_status: stale
last_update: Evidence invalidated after an input changed.
---





# TK-001 — Esqueleto do monorepo e app residente na bandeja

## Objetivo e limites

Entrega o walking skeleton: workspace Cargo + app Tauri 2/React que inicia oculto, mostra ícone e menu na bandeja, garante instância única, encerra filhos ao sair, e CI rodando testes em Linux e build em Windows.

Não inclui: Overlay translúcido (TK-002), atalho (TK-003), configurações (TK-004), store (TK-004/TK-006). Os itens do menu "Abrir", "Nova conversa" e "Configurações" podem apenas emitir eventos ainda sem janela correspondente.

## Leitura em ordem

1. `AGENTS.md` → convenções, comandos canônicos e limites.
2. `docs/architecture/overview.md` → seção "Módulos" (layout do monorepo) e "Processos".
3. `docs/adr/0002-tauri-rust-react.md` → stack decidido.
4. `specs/001-fundacao-overlay/plan.md` → "Modules, interfaces…" e OT-004.
5. Documentação atual do Tauri 2 (Context7: `tauri-apps/tauri`) → `tauri-plugin-single-instance`, tray icon API, sidecar/Job Object.

## Decisões já resolvidas

- Monorepo: `apps/desktop` (Tauri + React/Vite/TS/Tailwind 4), `crates/*`, `tools/*`; pnpm workspace para o front.
- Identificador do app `com.aura.desktop`, nome "Aura", instalador por usuário (config bundler preparada, sem publicar).
- `ChildRegistry` usa um Job Object do Windows com `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`; todos os filhos futuros são adicionados a ele.
- Liberdade local: nomes de helpers, organização interna de `lib.rs`.
- Alternativa descartada: `tauri-plugin-store` para config (será SQLite no TK-004).

## Mapa de alterações

- Novo: `Cargo.toml` (workspace), `rust-toolchain.toml`, `.cargo/config.toml` (target MSVC), `.github/workflows/ci.yml` (jobs `test-linux`: nextest nos crates puros; `build-windows`: `pnpm tauri build --debug` + nextest completo).
- Novo: `apps/desktop/src-tauri/src/main.rs` → `main`; `lib.rs` → `run()`; `tray.rs` → `build_tray(app)`; `children.rs` → `ChildRegistry::{new, adopt(child), shutdown()}`.
- Novo: `apps/desktop/src/main.tsx`, `apps/desktop/src/App.tsx` (placeholder), `apps/desktop/index.html`, `vite.config.ts`, `tailwind` setup, `vitest.config.ts`.
- Novo: `crates/aura-core/src/lib.rs` com `AppEvent` (enum de eventos de bandeja: `OpenOverlay`, `NewConversation`, `OpenSettings`, `Quit`).
- Novo: `apps/desktop/e2e/` com WebdriverIO + `tauri-driver` e o primeiro spec.
- Fora da fatia: qualquer UI além do placeholder.

## Contrato técnico

- Entradas: execução de `aura.exe` (com ou sem `--background`); cliques no menu da bandeja.
- Saídas: processo residente; ícone com menu "Abrir", "Nova conversa", "Configurações", "Sair"; eventos `AppEvent` publicados no barramento interno (`tokio::sync::broadcast`).
- Invariantes: no máximo uma instância por usuário do Windows; nenhuma janela visível na inicialização.
- Erros: falha ao criar o ícone da bandeja → log `error` e sair com código 2.
- Efeitos: segunda execução encaminha argumentos à primeira e dispara `OpenOverlay`; `Quit` chama `ChildRegistry::shutdown()` antes de `app.exit(0)`.
- Compatibilidade/concorrência: `ChildRegistry` é `Send + Sync`; `adopt` pode ser chamado de qualquer thread.

## Exemplos de aceite

- **AC-001**: app não rodando + iniciar → 1 processo `aura.exe`, janela `main`/`overlay` com `isVisible=false`, ícone presente (roteiro manual com print); efeito proibido: qualquer janela visível.
- **AC-002**: app rodando + iniciar de novo → após 2 s `Get-Process aura` retorna 1 processo; evento `OpenOverlay` recebido pela instância original (verificado por log `event=OpenOverlay source=second_instance`).
- **AC-003**: app com um filho de teste (`ping -t 127.0.0.1` adotado pelo `ChildRegistry` via comando de debug `debug_spawn_child`) + Sair → após 3 s nem `aura.exe` nem `ping.exe` do teste existem. Oráculo: spec (3 s).

## Dependências e sequência de execução

Depende de: nenhum.

- [ ] TK-001.1 Criar workspace, toolchain e app Tauri mínimo; confirmar `pnpm tauri dev` abre sem janela visível (red: E2E AC-001 falha antes da config `visible:false`).
- [ ] TK-001.2 Escrever E2E de AC-001 e fazê-lo passar.
- [ ] TK-001.3 Teste de integração Rust `child_registry_kills_adopted_children_on_shutdown` (red) → implementar Job Object (green).
- [ ] TK-001.4 Bandeja + `Quit` → E2E/roteiro AC-003.
- [ ] TK-001.5 Single instance → E2E AC-002.
- [ ] TK-001.6 CI Linux + Windows verde; registrar evidência.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run --workspace` (Linux e Windows); `pnpm -C apps/desktop e2e -- --spec e2e/residente.spec.ts` (Windows); roteiro manual AC-001/AC-003 com prints.
- Estado esperado: todos verdes; prints mostram só o ícone na bandeja.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: falta de WebView2 ou de `tauri-driver`/`msedgedriver` é ambiente, não red de comportamento.

## Condição de retorno à planejadora

Retornar se o Tauri não permitir iniciar sem janela visível, se Job Objects conflitarem com o modelo de processos do Tauri/WebView2, ou se `tauri-driver` não funcionar no runner Windows (decidir alternativa de E2E).

## Relatório de saída

Relatar versões fixadas (Rust, Tauri, Node, pnpm, plugins), arquivos criados, AC-001–AC-003 com EV refs, limitações (por exemplo, bandeja validada só manualmente) e próxima ação (TK-002).
