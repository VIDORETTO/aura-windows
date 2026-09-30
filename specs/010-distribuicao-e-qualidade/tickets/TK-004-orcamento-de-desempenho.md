---
schema: hybrid/ticket
schema_version: 1.0
id: TK-004
effort: 010-distribuicao-e-qualidade
type: delivery
status: implemented
ticket_revision: 3
requires: []
requirement_refs: ["FR-004"]
acceptance_refs: ["AC-007", "AC-008"]
spec_revision: 1
plan_revision: 1
owned_areas: ["tools/aura-bench/src/all.rs", "tools/aura-bench/src/baseline.rs", "bench", ".github/workflows/perf.yml"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-004 — Orçamento de desempenho verificado continuamente

## Objetivo e limites

Entrega `aura-bench all` (ocioso 5 min, 50 aberturas, abrir com Chip de tela, primeiro delta pintado com app-server falso), comparação com `bench/baseline.json`, workflow `perf.yml` no Windows e o relatório de aceitação na máquina de referência. Corrige regressões encontradas ou abre tickets de correção.

Não inclui: otimizações específicas não reveladas pelas medições.

## Leitura em ordem

1. `tools/aura-bench/` (001 TK-003; 004 TK-005 acrescentou `capture-buffer`).
2. `docs/architecture/overview.md` → "Orçamentos de desempenho".
3. Docs: PDH/`GetProcessMemoryInfo` (`PrivateUsage`), ETW opcional.

## Decisões já resolvidas

- Soma de working set privado de `aura.exe` + todos os `msedgewebview2.exe` cujo processo pai seja do Aura.
- Regressão = pior que baseline × 1,2 em qualquer métrica → exit code 1.
- Máquina de referência: Windows 11, 4 núcleos, 8 GB, SSD; runner self-hosted opcional; sem ela, só CI anti-regressão.
- Liberdade local: formato do relatório (JSON + Markdown).

## Mapa de alterações

- Novo: `tools/aura-bench/src/{all.rs,baseline.rs}`; `bench/baseline.json`; `.github/workflows/perf.yml`.

## Contrato técnico

- Entradas: binário de release; baseline.
- Saídas: `bench/latest.json`, `bench/report.md`; exit code.
- Invariantes: OT-003.

## Exemplos de aceite

- **AC-007**: máquina de referência → `idle.private_ws_mb ≤ 150` e `idle.cpu_avg_pct ≤ 0.5` no relatório.
- **AC-008**: `open.p95_ms ≤ 100`; baseline `{open.p95_ms: 80}` e medição 100 → exit 1 (100 > 96); medição 90 → exit 0 (unit do comparador).

## Dependências e sequência de execução

Depende de: 001-fundacao-overlay/TK-003 (outro esforço). Idealmente executado após o Marco 4.

- [ ] TK-004.1 Comparador com baseline (unit) red→green.
- [ ] TK-004.2 Cenários `all` e workflow.
- [ ] TK-004.3 Execução na máquina de referência; tickets de correção se necessário; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-bench baseline`; `cargo run -p aura-bench --release -- all --out bench/latest.json --baseline bench/baseline.json`.
- Estado esperado: dentro dos orçamentos na máquina de referência.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: runner compartilhado com ruído — repetir 3× e usar mediana.

## Condição de retorno à planejadora

Retornar se H-004 falhar por margem que exija mudança arquitetural (ex.: destruir WebView2 oculto após inatividade — trade-off com H-001).

## Relatório de saída

Relatar métricas, comparação, EV refs, tickets de correção abertos.
