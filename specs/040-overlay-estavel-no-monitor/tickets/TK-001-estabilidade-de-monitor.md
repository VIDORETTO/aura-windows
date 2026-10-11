---
schema: hybrid/ticket
schema_version: 1.0
id: TK-001
effort: 040-overlay-estavel-no-monitor
type: delivery
status: done
ticket_revision: 8
requires: []
requirement_refs: ["FR-001", "FR-002", "FR-003"]
acceptance_refs: ["AC-001", "AC-002", "AC-003"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-core/src/placement.rs", "apps/desktop/src-tauri/src/overlay.rs", "crates/aura-win/examples/qa_foreground.rs", "crates/aura-win/examples/qa_resize.rs", "apps/desktop/e2e/specs/monitor-stability.e2e.ts", "specs/040-overlay-estavel-no-monitor"]
verification_status: passed
last_update: "Todos os aceites040 comprovados e revisão registrada; executável disponível, instância antiga preservada."
---








# TK-001 — Estabilidade e memória no monitor atual

## Objetivo e limites

Corrigir salto nas transições e memória atribuída ao monitor anterior; pedido explícito autoriza. Sem IPC/migração/dependência/release. Um ticket por vez; preservar outros esforços/conta/arquivos preexistentes.

## Leitura em ordem

1. AGENTS.md, spec r1, plan r1, tdd.md deste esforço.
2. `apps/desktop/src-tauri/src/overlay.rs`: set_mode, remember_placement, show/target_monitor.
3. `crates/aura-core/src/placement.rs`: place_overlay/clamp_into/testes.
4. `apps/desktop/src/overlay/session.ts`: setOverlayMode/setMinibar/send, sem edição prevista.
5. `apps/desktop/e2e/specs/resize.e2e.ts` e `crates/aura-win/examples/qa_resize.rs`: arraste guardado.

## Exclusões

Sem IPC, migração, atualização de sidecar, instalação ou publicação.

## Decisões já resolvidas

Geometria atual seleciona monitor; novo modo conserva âncora/usa dimensões adequadas. show oculto contextual. Novo monitor_for_rect no core; set_mode/remember_placement no shell; teste monitor-stability novo. Detalhes privados reversíveis livres; UI/provedor/store fora.

## Mapa de alterações

Existentes placement.rs e overlay.rs; novos monitor-stability.e2e.ts e qa_foreground.rs (fixture SO vazia, sem entrada em outros apps). Caminhos completos no frontmatter e plano. Nenhuma edição em Session/store.

## Contrato técnico e exemplos de aceite

Rect físico/Monitors físicos → maior interseção positiva/None. Nenhuma interseção usa fallback; zero monitores None. Ler posição/tamanho antes de mudança nativa. Host::save_placement preserva esquema e ID/DPI do monitor geométrico. Operações seriais no shell; nenhum novo worker/estado de cursor.

- AC-001: anterior A, janela B → compacto/expandido mantém B e x/y se cabe; saved do outro modo não reancora.
- AC-002: salvar A/B separadas; abertura em cada monitor recupera sua largura/posição.
- AC-003: exemplos negativos/fronteira do TDD e mínimos/fallback placement existente.

## Dependências e sequência de execução

Nenhum predecessor; tools/monitores já identificados.

- [x] TK-001.1 Package ready, red nativo por monitor incorreto.
- [x] TK-001.2 Caso puro por vez, green mínimo, integrar shell.
- [x] TK-001.3 Native green, regressões placement/model-picker/resize, produção compila.
- [x] TK-001.4 Evidência, revisão em dois eixos, checkpoint/render.

## Validação

Diretório raiz; comandos/ambiente no plano. Windows11/MSVC/WebView2/EdgeWebDriver154, TEMP/processo/identidade QA isolados. Terminal real/foreground guard; ambiente ausente não é red, menos de dois monitores é not_run. Sem simular DPI nativo.

## Condição de retorno à planejadora

Retornar à planejadora com símbolo/resultado se exigir identidade/configuração de produção, IPC, persistência ou ambiente indisponível. Preservar diffs. Relatar caminhos, AC, comandos/EV/limitações; done somente com evidência e revisão atuais.

## Relatório de saída

EV-001 red comportamental real; EV-002 native3/3+picker2/2+resize9/9; EV-003 core56/desktop1/UI253/types/clippy/fmt/diff; EV-004 artefato normal `target/release/aura-monitor-fix.exe`. Relatório `qa.md`, revisão local separada Standards/Spec em `review.md`; caminhos/hashes/limites registrados. DPI real100%; 150/200 de038 não aprovado. Instância do usuário preservada; abrir versão nova após Sair pela bandeja. SC-002 permanece observação posterior.
