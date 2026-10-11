# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — failed

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-003`
- Procedure: `Antes da correção: cargo test -p aura-core current_window_on_negative_origin_monitor_does_not_follow_previous_app; pnpm -C apps/desktop/e2e test -- --spec ./specs/monitor-stability.e2e.ts (native-red-3.log).`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:3207cee696fe870f5ef84cb38d8f747e8d968b080089bb26a523bc609c03e176`
- Timestamp: `2026-10-11T02:18:54+00:00`
- Observations: Reds reais em versões anteriores: stub None vs B; shell anterior devolveu x2000 de B para x1280 em A ao expandir. Relatório distingue os fontes então executados dos fingerprints green atuais.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-040-monitor-20261010/core-red.log`, `C:/Users/gabri/AppData/Local/Temp/aura-040-monitor-20261010/native-red-3.log`
- Limitations: Erros de origin/foreground anteriores foram ambientais e não contam como red. Esta evidência registra falhas anteriores, não o código corrigido.

## EV-002 — passed

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`
- Procedure: `pnpm -C apps/desktop/e2e test -- --spec ./specs/monitor-stability.e2e.ts; pnpm -C apps/desktop/e2e test -- --spec ./specs/model-picker-recovery.e2e.ts --spec ./specs/resize.e2e.ts (env QA e procedimento em qa.md).`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:e3867b8b37b5f33ebcbeb043227cc8eea8804b6944eb33a988708bdc78ede126`
- Timestamp: `2026-10-11T02:18:54+00:00`
- Observations: 3/3 estabilidade: saved expandido B2000/80 altura500; quatro modos conservam compacto2080/60; A80/80 largura700 e B2000/80 largura820 restauradas; conversa/Minibar real conserva2000/80. Regressão2picker+9resize passou. Guard/oráculos preservados.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-040-monitor-20261010/native-final-fit-anchor.log`, `C:/Users/gabri/AppData/Local/Temp/aura-040-monitor-20261010/regression-final.log`
- Limitations: DPI real100%; 150/200 de038 pendente. Modelo scripted varia somente rede externa. Ativação assistida só de fixtures vazias próprias; não foi encerrada a instância produtiva do usuário.

## EV-003 — passed

- Ticket: `TK-001`
- Acceptance: `AC-003`
- Procedure: `cargo test -p aura-core; cargo test -p aura-desktop; pnpm -C apps/desktop test; pnpm -C apps/desktop typecheck; cargo clippy -p aura-core -p aura-win -p aura-desktop --all-targets -- -D warnings; cargo fmt --all -- --check; git diff --check.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:bb0d75e80b206a08a564ea6322e0c1fcfae512679e6b9b11a194bd86729aaa97`
- Timestamp: `2026-10-11T02:20:10+00:00`
- Observations: Core56/56 incluindo11placement; desktop1/1; UI253/253em33arquivos; types/clippy/fmt/diff verdes. Oráculos literais de origem negativa, DPI distintos, maior área, desempate, ausência e regressão de DPI/mínimos/fallback; integração mantém fallback somente sem interseção.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-040-monitor-20261010`
- Limitations: Desconexão física e WebView2 nativo150/200 não executados; modelos/performance fora desta mudança de geometria.

## EV-004 — passed

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`
- Procedure: `Sem TAURI_CONFIG: cargo rustc -p aura-desktop --release --features tauri/custom-protocol --bin aura -- -C extra-filename=-monitor-fix; copiar target/release/deps/aura-monitor-fix.exe para target/release/aura-monitor-fix.exe; Get-FileHash SHA256.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:8135c6a0e8cc93b85ba0233480d03abd60dd1857654bc290a914f824ebe272fb`
- Timestamp: `2026-10-11T02:20:10+00:00`
- Observations: Build normal exit0 2m56; SHA256 D440E061E0986416E46991AFBED332376D7978FB32C708FCD47A529D3D9698AA. Sem e2e/demo, sem instalação; pronto para abertura após sair da versão anterior.
- Evidence refs: none
- Limitations: Compilar não prova comportamento; greens são EV002/003 no shell equivalente, identidade e perfil QA. Instância antiga PID32200 preservada; não foi atualizada em memória.
