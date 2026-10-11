# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-004`
- Procedure: `pnpm -C apps/desktop test -- src/overlay/Composer.test.tsx src/overlay/OverlayApp.test.tsx; pnpm -C apps/desktop typecheck; pnpm -C apps/desktop test -- src/i18n; cargo test -p aura-app --test host`
- Execution: `executed`
- Environment: Windows; Vitest 5.0.2/jsdom; Rust fixado do projeto; IPC mock publico.
- Tested revision: `local:aca019d64f8b4885a0caa563318b23ca74360867865d646f18a6d57f78489ec1`
- Timestamp: `2026-10-07T00:45:44+00:00`
- Observations: 82 testes UI, typecheck, 3 i18n e 73 Host passaram. Red real observado para falha sem recuperacao e carregamento sem estado; green apos correcao. Catalogo preservado em erro, retry no menu aberto, resposta antiga descartada, BYOK preservado, seletores antes e apos iniciar Conversa com provedor fixo.
- Evidence refs: none
- Limitations: Geometria nativa e DPI pertencem ao TK-003; esta evidencia de UI usa jsdom e nao prova pixels ou acesso comercial ChatGPT. Evidence invalidated because an input changed.

## EV-002 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-004`
- Procedure: `pnpm -C apps/desktop test -- src/overlay/Composer.test.tsx src/overlay/OverlayApp.test.tsx src/i18n; pnpm -C apps/desktop typecheck; cargo test -p aura-app --test host; cargo fmt --all -- --check`
- Execution: `executed`
- Environment: Windows; Vitest/jsdom; Host com store real temporario, plataforma/app-server falsos.
- Tested revision: `local:2af655a192208a8d5ce528c262bd50a6662727905e7a552391d1ad3560be851c`
- Timestamp: `2026-10-07T01:00:17+00:00`
- Observations: Evidencia renovada apos TK-002/plan r2: 91 testes UI/i18n, typecheck, 74 Host e rustfmt check passaram; estados, retry, preservacao, corrida de catalogos e provider fixo continuam corretos.
- Evidence refs: none
- Limitations: Geometria/DPI nativos em TK-003; nao prova login comercial ChatGPT. Sem alteracao de IPC ou sidecar. Evidence invalidated because an input changed.

## EV-003 — stale

- Ticket: `TK-002`
- Acceptance: `AC-005`, `AC-006`, `AC-007`
- Procedure: `pnpm -C apps/desktop test -- src/overlay/Composer.test.tsx src/overlay/OverlayApp.test.tsx src/i18n; pnpm -C apps/desktop typecheck; cargo test -p aura-app --test host; cargo fmt --all -- --check`
- Execution: `executed`
- Environment: Windows; Vitest/jsdom; Host com store real temporario, plataforma/app-server falsos.
- Tested revision: `local:2af655a192208a8d5ce528c262bd50a6662727905e7a552391d1ad3560be851c`
- Timestamp: `2026-10-07T01:00:18+00:00`
- Observations: 91 testes UI/i18n, typecheck, 74 Host e rustfmt check passaram. Literais qa-reasoner low/high/max, esforco per-model/modo e catalogo tardio; alias legado preserva outros modos; reinicio real Host/disco preserva low/max; invalidacao avisada; red reproduziu max enviado apos catalogo mudar durante startup, green agora envia null.
- Evidence refs: none
- Limitations: Geometria/DPI nativos em TK-003; nao prova login comercial ChatGPT. Sem alteracao de IPC ou sidecar. Evidence invalidated because an input changed.

## EV-004 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-004`
- Procedure: `pnpm -C apps/desktop test -- src/overlay/Composer.test.tsx src/overlay/OverlayApp.test.tsx src/i18n; pnpm -C apps/desktop typecheck; cargo test -p aura-app --test host; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: Windows 11; Vitest/jsdom IPC controlada, Host com store temporario real
- Tested revision: `local:c4fba55b62f4eea01e2410aab83723a8a0e9c4736b0c66ad71f8f87627c0b9f2`
- Timestamp: `2026-10-08T00:01:06+00:00`
- Observations: 91 testes UI/i18n, 74 Host, typecheck, rustfmt e diff check passaram novamente depois do ajuste de largura. Log UI em TEMP/aura-038-ui-tests.log. Review Standards/Spec renovada sem achados bloqueantes em AC001..004.
- Evidence refs: none
- Limitations: Geometria, DPI e teclado nativo pertencem ao TK-003; nao prova login de conta comercial. Evidence invalidated because an input changed.

## EV-005 — stale

- Ticket: `TK-002`
- Acceptance: `AC-005`, `AC-006`, `AC-007`
- Procedure: `pnpm -C apps/desktop test -- src/overlay/Composer.test.tsx src/overlay/OverlayApp.test.tsx src/i18n; pnpm -C apps/desktop typecheck; cargo test -p aura-app --test host; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: Windows 11; Vitest/jsdom IPC controlada, Host com store temporario real
- Tested revision: `local:c4fba55b62f4eea01e2410aab83723a8a0e9c4736b0c66ad71f8f87627c0b9f2`
- Timestamp: `2026-10-08T00:01:07+00:00`
- Observations: 91 UI/i18n,74 Host,typecheck,rustfmt,diff check passaram novamente. Persistencia real no reinicio Host, preferencias por modelo/modo, alias legado e corrida com catalogo tardio verdes. Review Standards/Spec renovada sem achados bloqueantes em AC005..007.
- Evidence refs: none
- Limitations: Geometria/DPI nativos em TK-003; nao prova login comercial. Evidence invalidated because an input changed.

## EV-006 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-004`
- Procedure: `pnpm -C apps/desktop test; pnpm -C apps/desktop typecheck; cargo test -p aura-app --test host; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:3f755830062e871f5337cf5f063b35943bf434821832c24e9ac4447bf8481720`
- Timestamp: `2026-10-08T00:10:55+00:00`
- Observations: 244 UI/33 arquivos,74 Host,typecheck,rustfmt,diff check verdes no codigo e plan r3. Estados de catalogo, retry, corrida e visibilidade por OverlayApp. Logs TEMP/aura-038-ui-final.log.
- Evidence refs: none
- Limitations: Matriz nativa completa em TK-003; nao comprova conta ChatGPT comercial. Evidence invalidated because an input changed.

## EV-007 — stale

- Ticket: `TK-002`
- Acceptance: `AC-005`, `AC-006`, `AC-007`
- Procedure: `pnpm -C apps/desktop test; pnpm -C apps/desktop typecheck; cargo test -p aura-app --test host; cargo fmt --all -- --check; git diff --check`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:3f755830062e871f5337cf5f063b35943bf434821832c24e9ac4447bf8481720`
- Timestamp: `2026-10-08T00:10:55+00:00`
- Observations: 244 UI,74 Host e verificacoes de tipo/formato passaram. Casos literais low/max, reinicio real Host, alias, remocao de suporte e corrida antes de send. Plan r3/current code.
- Evidence refs: none
- Limitations: Matriz nativa/escala completa em TK-003. Evidence invalidated because an input changed.

## EV-008 — stale

- Ticket: `TK-003`
- Acceptance: `AC-009`
- Procedure: `AURA_HOME=TEMP/aura-038-model-picker-production/geometry-profile; AURA_E2E_APP=TEMP/aura-038-model-picker-production/aura-qa.exe; PATH inclui msedgedriver compativel; pnpm -C apps/desktop/e2e test -- --spec ./specs/model-picker-recovery.e2e.ts --logLevel error`
- Execution: `executed`
- Environment: Windows 11; WebView2 154.0.4258.62; msedgedriver 154.0.4258.53; build normal atual, perfil isolado sem chaves
- Tested revision: `local:28d47960dd6f83283773fe8b656e736da698e90b021a427a967281d1d950b827`
- Timestamp: `2026-10-08T00:10:56+00:00`
- Observations: 2/2 E2E passaram: Tab/Enter seleciona Low/Baixo; Escape fecha e devolve foco ao botao, Overlay continua aberto, ptBr/en. Log TEMP/aura-038-native-final.log.
- Evidence refs: none
- Limitations: Escalas150/200 e regressao de handles nativos ainda pendentes. Evidence invalidated because an input changed.

## EV-009 — partial

- Ticket: `TK-003`
- Acceptance: `AC-008`
- Procedure: `pnpm -C apps/desktop tauri build --no-bundle; E2E model-picker-recovery no build normal/perfil isolado; regressao composer,model-effort,model-efforts,profile-model,resize no build normal; composer previamente no demo`
- Execution: `executed`
- Environment: Windows 11; dois monitores 1920x1080 e1366x768, ambos scale1.0; perfil QA isolado; WebView2 154.0.4258.62
- Tested revision: `local:8bd56b5ad5277cfb00fc4a2ef8f5f6d825041ec2bfa41115ef99010c7a0ebcba`
- Timestamp: `2026-10-08T00:10:56+00:00`
- Observations: Build normal passou. Native480 compact/expandido passou com label108.5/91.45px, seta visivel, overlap[], clique e menu dentro da area util do monitor menor. Inspecao screenshots confirms. Model-effort/efforts reais passaram high/max/reinicio. Resize falhou por foreground refusal; perfil tinha fixture de inputs deslocados pela busca Settings, correcao e rerun em andamento. Composer necessita demo, onde passou.
- Evidence refs: none
- Limitations: AC008 parcial: faltam DPI150/200 reais. Nao simular layout nem reduzir aceite. Regressao total ainda nao aprovada. Logs TEMP/aura-038-native-final.log e aura-038-production-regression.log; screenshots TEMP/aura-038-model-picker-production.

## EV-010 — partial

- Ticket: `TK-003`
- Acceptance: `AC-008`
- Procedure: `AURA_CODEX_BIN=TEMP/aura-038-model-picker-production/profile/bin/codex/rust-v0.159.0/bin/codex-app-server.exe; build normal atual/perfil QA isolado; pnpm -C apps/desktop/e2e test -- --spec ./specs/profile-model.e2e.ts --logLevel error`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:233c3a453d98f7ab4c01c539416e57e4069f9797372c773fc961d6902c5b55b8`
- Timestamp: `2026-10-08T00:12:14+00:00`
- Observations: Profile-model agora passed: processo powershell.exe/title QA Insert Target,profile_active QA target,botao qa-other e turno upstream model qa-other. 1/1 em2.9s; log TEMP/aura-038-profile-cached-bin.log.
- Evidence refs: none
- Limitations: Resultado parcial do gate total AC008: ainda faltam DPI150/200 e resize/foreground. Este procedimento isolado nao aprova geometria toda.

## EV-011 — passed

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-002`, `AC-003`, `AC-004`
- Procedure: `pnpm -C apps/desktop test; pnpm -C apps/desktop typecheck; cargo test -p aura-app --test host; cargo fmt --all -- --check; git diff --check; fixedbaseline Standards/Spec review including shared web changes`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:c9b58afb26f568237e808d5a0dfdbb3839847f6be4530f718a7759a2fa28926b`
- Timestamp: `2026-10-10T21:16:28+00:00`
- Observations: UI253/33files26.66s,typecheckpassed;Host74/74in1.82s including diskrestart preferences. Original selectors/catalog/presets/startup/send oracles passed;web config metadata independent. Standards/Spec no blocker for selectedACs;no productchange inrenewal.
- Evidence refs: none
- Limitations: Does not approve AC008/009 nativewindow,150/200DPI,resize,SIWC or commercialmodel availability. Nativebuild38814 still running; prior100percent QA separate.

## EV-012 — passed

- Ticket: `TK-002`
- Acceptance: `AC-005`, `AC-006`, `AC-007`
- Procedure: `pnpm -C apps/desktop test; pnpm -C apps/desktop typecheck; cargo test -p aura-app --test host; cargo fmt --all -- --check; git diff --check; fixedbaseline Standards/Spec review including shared web changes`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:c9b58afb26f568237e808d5a0dfdbb3839847f6be4530f718a7759a2fa28926b`
- Timestamp: `2026-10-10T21:16:30+00:00`
- Observations: UI253/33files26.66s,typecheckpassed;Host74/74in1.82s including diskrestart preferences. Original selectors/catalog/presets/startup/send oracles passed;web config metadata independent. Standards/Spec no blocker for selectedACs;no productchange inrenewal.
- Evidence refs: none
- Limitations: Does not approve AC008/009 nativewindow,150/200DPI,resize,SIWC or commercialmodel availability. Nativebuild38814 still running; prior100percent QA separate.

## EV-013 — stale

- Ticket: `TK-003`
- Acceptance: `AC-009`
- Procedure: `Native build e2e without demo exit0 2m57s; pnpm -C apps/desktop/e2e test -- --spec ./specs/model-picker-recovery.e2e.ts; inspect native compact/expanded screenshots; fresh UI253/typecheck parity already passed.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:72d7c15774f2749d0be1eff0c83590dfb6567db5eb0a455a2d4d2c33384218be`
- Timestamp: `2026-10-10T21:31:48+00:00`
- Observations: Native2/2 in5.2s. Keyboard Tab/Enter opens and selects low; Escape closes and returns model focus, Overlay/input retained in ptBR/en. Actual OS scale1; images inspected.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-038-final-native-20261010-181914/model-picker.log`
- Limitations: Only AC009 approved; physical150/200 and complete resize regression not approved. Long previousApp context was synthetic after actual monitor move; no assertion of screen capture or real LLM by these tests. Evidence invalidated because an input changed.

## EV-014 — stale

- Ticket: `TK-003`
- Acceptance: `AC-008`
- Procedure: `Native build e2e no demo exit0 2m57s. model-picker-recovery2/2; composer/model-effort/model-efforts/profile-model4/4; resize8/9 then isolated new-profile resize6/9; native images inspected.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:0aeaeea345513c51357ba7901ffdb244430df312f4271b0826511b7454f62718`
- Timestamp: `2026-10-10T21:32:06+00:00`
- Observations: 480logicalpx/native100percent: compact/expanded accessible full name, positive label/arrow, hit test, no overlap, menu within monitor; keyboard passed EV013. Resize firstEast-120 width900 unchanged expectedbelow850; subsequent8 casespassed. Freshprofile6passed/3 refusedforeground. No guard/oracle weakened.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-038-final-native-20261010-181914`
- Limitations: Actual150/200DPI unavailable; complete resize not approved. First drag failure remains unclassified; fresh mouse cases stopped before input because target lacked foreground. Public model/protocol regression scripted is not commercial LLM proof. Evidence invalidated because an input changed.

## EV-015 — stale

- Ticket: `TK-003`
- Acceptance: `AC-008`
- Procedure: `Current normal OS native e2e build (no demo), isolated fresh native-resize-profile, existing resize.e2e.ts with unchanged guarded qa_resize helper; full nine-case native suite. Previous model-picker2/2 at480px/100percent remains separate EV013/014.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:edf1a0685dd4bd5052979ef1dde9c5a234ce6b8b0312bbc32f340eee3d6ec17c`
- Timestamp: `2026-10-10T22:10:43+00:00`
- Observations: Native resize9/9 passed44sec: compact both sides, multiple widths, height refit, expanded min/every edge+corner, menu fit, width restore, available monitors and legacy clamp. Previous failures did not recur in current fresh profile with current build. Foreground refusal guard and oracles unchanged; no resize/product/helper code edited to make it pass.
- Evidence refs: `EV-013`, `EV-014`
- Limitations: Actual available monitors100percent;150/200native scales still unexecuted. Earlier initial unchanged width and foreground refusals retained; origin not conclusively assigned. Current pass approves resizing gate, not every DPI geometry requirement. Evidence invalidated because an input changed.

## EV-016 — partial

- Ticket: `TK-001`
- Acceptance: `AC-001`
- Procedure: `Read-only real-picker.mjs disposable native harness; default existing ChatGPT profile, normal complete pin (no binary override), public models_list and UI ModelPicker; select model locally without effort selection, compare settings_get before/after. No inference, capture or browser operation.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:a254f34daee16bc271561ac3949cd200dbeb1cae3ed868ac63f032b595138082`
- Timestamp: `2026-10-10T22:16:32+00:00`
- Observations: Real signed-in ChatGPT catalog3models, native picker visible; actual GPT-6 Luna selected locally, sixenabled effort radios including defaultmedium andlow/medium/high/xhigh/max. Settings identical before/after. Public existing catalog recovery used if needed; no hidden state injection or synthetic metadata.
- Evidence refs: none
- Limitations: Additional actual account proof at100percent only; does not replace full compact/expanded/loading/error/keyboard/DPI cases. First harness lookup attempted provider item before recovery; classified harness error and preserved.

## EV-017 — passed

- Ticket: `TK-003`
- Acceptance: `AC-009`
- Procedure: `pnpm -C apps/desktop/e2e test -- --spec ./specs/model-picker-recovery.e2e.ts --spec ./specs/resize.e2e.ts; ambiente nativo atual descrito em040/qa.md.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:0b0f446a909cf8cb0aeaaec304c9ffe2ff3a6a51358b7da60264fe02fa918733`
- Timestamp: `2026-10-11T02:20:10+00:00`
- Observations: 2/2 seletor/teclado e9/9resize em build nativa040 atual; teclado abre/seleciona esforço/Escape devolve foco em pt-BR/en. Renova prova invalidada por overlay/helper.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-040-monitor-20261010/regression-final.log`
- Limitations: Não aprova AC008150/200; DPI nativo real100.

## EV-018 — partial

- Ticket: `TK-003`
- Acceptance: `AC-008`
- Procedure: `Mesma execução nativa regression-final.log: model-picker-recovery2/2 e resize9/9, geometria480logicalpx compacto/expandido.`
- Execution: `executed`
- Environment: OS=Windows-11-10.0.26200-SP0; Python=3.14.2
- Tested revision: `local:0b0f446a909cf8cb0aeaaec304c9ffe2ff3a6a51358b7da60264fe02fa918733`
- Timestamp: `2026-10-11T02:20:10+00:00`
- Observations: 480logicalpx e100% reais: nome acessível completo, seta/rótulo positivos, hit test acionável, zerooverlaps e menu dentro da área útil; resize9/9. Prova nova sobre040, substitui geometria anterior stale.
- Evidence refs: `C:/Users/gabri/AppData/Local/Temp/aura-040-monitor-20261010/regression-final.log`
- Limitations: Ambos os monitores100%; faltam150/200 nativos. Nenhum aceite reduzido ou DPI do navegador usado como substituto.
