---
schema: hybrid/plan
schema_version: "1.0"
effort_id: 038-restaurar-seletores-modelo-raciocinio
revision: 3
spec_revision: 1
status: ready
---

# Plano — restaurar modelo e raciocínio

## Resumo e contrato consumido

Spec r1, FR-001 a FR-007 / AC-001 a AC-009. Recuperar componente existente, separar estado do catálogo dos dados e validar seleção/envio na fronteira pública. Baseline/limitações em `discovery.md`.

## Contexto técnico

React 19, Zustand 5, TS estrito, Tailwind 4, Rust 2024 e app-server rust-v0.159.0 observados. Manter `models_list -> ModelInfo[]`; carregamento/erro inicialmente no estado UI, sem migração. Comandos abaixo identificados na configuração/instruções; não executados nesta sessão.

## Módulos, interfaces, consumidores e seams

- `Header`/`ModelPicker`: botão/diálogo/opções/foco; seam `OverlayApp` com bridge IPC controlada e janela Windows para geometria.
- `useSession.loadCatalog`: novo estado previsto `catalogStatus` e ação de repetir; resultados de provedores/modelos independentes, conserva dados na falha. Não mockar Zustand/componente/capacidades.
- `effectiveModel`, `modelCapabilities`, `presetEffort`, `setModel`, `setEffort`, `setMode`: reconciliar após sucesso e antes de `send`; observar UI e `conversation_send`, sem testar funções privadas.
- `Host::models` → `CodexService::codex_models` → `plan_catalog`: preservar erro classificável e catálogo do 017, sem ampliar acesso ou esconder indisponibilidade.
- `Floating`, `useAutoHeight`, `apply_minimum`: seam Windows de corte físico/DPI; alterar somente conforme reprodução.
- `StatusBadges` em `StatusBar.tsx`: na janela compacta abaixo de 600 px, preservar ícone, título e nome acessível da pausa, abreviando somente seu texto visual. Reprodução nativa a 480 px mostrou esse texto sobrepondo o botão Reunião. O teste de janela mede também sobreposição entre controles.

## Abordagem e alternativas

TK-001: diagnóstico, acesso, estados explícitos, últimas opções válidas e repetição. Preservar `catalogRequest`; sucesso vazio diferente de falha. TK-002: restaurar/revalidar esforço, escolha no próximo Turno e aviso de invalidação confirmada. TK-003: geometria/teclado/idiomas nativos, incluindo correção localizada caso necessária.

Descartados: seletor somente nas Configurações; hardcode extra; update do sidecar; jsdom como prova de largura/DPI. Sem dependências externas novas. Context7 indisponível nesta sessão; código/contratos locais e documentação oficial do Codex sustentam o plano, sem substituir schema fixado.

## Dados, compatibilidade e mapa de alterações

Sem migração; preservar `Settings.effortPresets` e defaultModel. Provedor fixo por Conversa; escolha durante execução afeta só próximo Turno.

Detalhe técnico reconciliado em r2 após TDD: para o Provedor ChatGPT, resolver o modelo efetivo ao ler/gravar presets, mesmo quando o seletor está no padrão. Continuar lendo o alias legado `::default` quando não houver preset específico; após escolha explícita de esforço, gravar pelo ID real e substituir somente o modo desse alias, preservando os demais. Não migrar banco nem apagar preferências de outros modelos/modos. Restaurar após catálogo confirmado e avisar ao invalidar esforço; erro de refresh conserva escolha válida anterior.

O Turno conserva a escolha de modelo capturada antes de operações assíncronas, mas revalida seu esforço contra as capacidades atuais imediatamente antes de `conversation_send`. TDD reproduziu catálogo que muda durante `conversation_start`. Teste adicional em `crates/aura-app/tests/host.rs`, `model_effort_preferences_survive_a_host_restart`, usa disco temporário real e APIs públicas de Settings para provar persistência; nenhum acesso lateral ao SQLite.

- Existentes: `apps/desktop/src/overlay/{Header.tsx,ModelPicker.tsx,session.ts,OverlayApp.tsx,Composer.test.tsx,OverlayApp.test.tsx}`, `apps/desktop/src/ui/{Popover.tsx,floating.tsx}`.
- Existentes se necessárias: `crates/aura-app/src/host.rs` (`Host::models`), `crates/aura-app/tests/host.rs`, `apps/desktop/src-tauri/src/overlay.rs` (`apply_minimum`).
- Existentes: `apps/desktop/src/i18n/{pt-BR,en}.ts`; `apps/desktop/e2e/specs/{composer,model-effort,model-efforts,profile-model,resize}.e2e.ts`.
- Novo previsto: `apps/desktop/e2e/specs/model-picker-recovery.e2e.ts`.
- Fora: login/gateway/sidecar, salvo diagnóstico de leitura. Nova mudança de IPC exige reconciliar plano/ADR 0009 antes de regenerar dourado.

## Verificação e oráculos

`tdd.md` mapeia cada AC. Um red por comportamento, green mínimo e próximo caso. Literais sintéticos `qa-reasoner`, `[low, high, max]`, padrão `high`, independentes da oferta comercial. Mock somente rede/processo/IPC/relógio quando varia. Persistência atualizada/relida via APIs públicas com store temporário real, sem consultar banco por fora.

Comandos futuros da raiz:

```powershell
pnpm -C apps/desktop test -- src/overlay/Composer.test.tsx src/overlay/OverlayApp.test.tsx
pnpm -C apps/desktop typecheck
cargo test -p aura-app --test host
pnpm -C apps/desktop tauri build --no-bundle --features demo
pnpm -C apps/desktop/e2e test -- --spec ./specs/model-picker-recovery.e2e.ts
pnpm -C apps/desktop/e2e test -- --spec ./specs/composer.e2e.ts --spec ./specs/model-effort.e2e.ts --spec ./specs/model-efforts.e2e.ts --spec ./specs/profile-model.e2e.ts --spec ./specs/resize.e2e.ts
```

Fechamento futuro: `pnpm -C apps/desktop test`, `cargo test --workspace --exclude aura-desktop`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`. MSVC/WebView2/WebDriver compatíveis necessários; falta de ambiente não é falha comportamental. Demo não prova acesso de conta real.

## Obrigações técnicas

- **OT-001** → FR-002/FR-003, AC-003/AC-004: erro não sobrescreve catálogo válido; estados independentes para provedor/modelo.
- **OT-002** → FR-004/FR-006, AC-005/AC-007: restaurar/revalidar esforço ao carregar e enviar; Turno em curso preservado.
- **OT-003** → FR-007, AC-008/AC-009: DOM não prova pixels/DPI/drag region/tamanho nativo.

## Riscos e gates

G1: comportamento solicitado registrado. G2: seams/oráculos definidos e adotados no pedido posterior de execução dos esforços/tickets pelo usuário. G3: TK-001 sem dependências; outros encadeados pela versão corrigida. Estado real no checkpoint/evidências, sem reutilizar aprovações desatualizadas. Executar `package` antes de cada implementação; evidência somente após execução real.
