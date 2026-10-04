# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`, `AC-042`
- Procedure: `pnpm -C apps/desktop/e2e test --spec ./specs/resize.e2e.ts; executável original preservado`
- Execution: `executed`
- Environment: Windows 11; WebView2/EdgeDriver 154.0.4258.53; DPI 100%; perfil isolado; original SHA256 592C2010FE11AC9D6B8B36BF04AFAC1D5E012AF01595CE82490C8251A15CF055
- Tested revision: `local:40796feb94970ff86c3fc697a779be2f0969785c882f46c28fe7db76ec52f3f9`
- Timestamp: `2026-10-03T12:59:10+00:00`
- Observations: Red comportamental: compacto janela 400 shell 80; expandido altura interna 64 entrada corta; largura acompanha nas laterais
- Evidence refs: none
- Limitations: Relato de largura e DPI125/150 não reproduzidos; red original não comprova correção atual Evidence invalidated because an input changed.

## EV-002 — stale

- Ticket: `TK-001`
- Acceptance: `AC-042`
- Procedure: `pnpm -C apps/desktop/e2e test --spec ./specs/resize.e2e.ts; cargo test --workspace --exclude aura-desktop; cargo clippy -p aura-desktop -- -D warnings`
- Execution: `executed`
- Environment: Windows 11; WebView2/EdgeDriver 154.0.4258.53; produção sem demo; duas telas DPI100; SHA256 D6B3A7950E7DA3A2539A4FB42D134ECC3D14EF3101A4BF0311FA6503F56A5384
- Tested revision: `local:19751baa43c5c6d96b9a3d0d502b81b9495ece3ce87dce9d5be1280e25fee394`
- Timestamp: `2026-10-03T13:16:40+00:00`
- Observations: 9 E2E passaram; mínimo interno expandido 360 entrada324; compacto80; legado64 restaurado360; placement literal DPI125/150 passou
- Evidence refs: none
- Limitations: DPI125/150 somente lógica pura; inferência/captura/ASR não exercitados Evidence invalidated because an input changed.

## EV-003 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`
- Procedure: `pnpm -C apps/desktop/e2e test --spec ./specs/resize.e2e.ts`
- Execution: `executed`
- Environment: Produção Windows11; WebView2 154.0.4258.53; duas telas100%
- Tested revision: `local:e731dc65abc40cf6068f200d0233bb994b3fb73487cb3ddc8b7117a7e92e0dd2`
- Timestamp: `2026-10-03T13:16:40+00:00`
- Observations: Altura corrigida; 9/9 nativos; larguras acompanham; menus e reabertura corretos
- Evidence refs: none
- Limitations: Relato específico largura não reproduzido; falta execução nativa125/150 Evidence invalidated because an input changed.

## EV-004 — stale

- Ticket: `TK-002`
- Acceptance: `AC-002`
- Procedure: `Vitest OverlayApp: 2 casos red reais de exclusao; green 19/19; suite UI completa e typecheck`
- Execution: `executed`
- Environment: Windows11, Vitest5, Node local
- Tested revision: `local:6a80c1e62f9b81dcfae2ef67de5fdf28a1e0a9e09a4140012c586ea4a02f676e`
- Timestamp: `2026-10-03T13:34:09+00:00`
- Observations: Suite UI 60/60 passou e typecheck passou. Baseline nativo demo red: ambos asides visiveis e conversa0px a480; logs target/qa-tools/panels-native-red-demo.log e panels-width-native-red.log. Novo build nativo ainda em curso.
- Evidence refs: none
- Limitations: Layout novo ainda nao executado no WebView2; nao valida inferencia de provider. Evidence invalidated because an input changed.

## EV-005 — stale

- Ticket: `TK-002`
- Acceptance: `AC-002`
- Procedure: `Native WebDriver panels.e2e e resize.e2e, pnpm test60/60, typecheck, build frontend/release demo, local review Standards/Spec`
- Execution: `executed`
- Environment: Windows11 WebView2/driver154.0.4258.53, packaged demo SHA2561EC08AF482205443909207F195E063CE541B7D01A671B57035E522453C1A9860, isolated profile, 100percent
- Tested revision: `local:f3d276cd4923f5fdea301186e6a684e51073a803abffbb0d4e90b908e249a1c2`
- Timestamp: `2026-10-03T13:37:47+00:00`
- Observations: Conversation480/640 and704(Files)/768(History) at1024, both-direction exclusion, draft preserved, Escape. Native panels1/1 and resize9/9. Logs target/qa-tools/panels-native-green.log, resize-after-panels-demo.log, panels-ui-all.log, panels-typecheck.log. Local review specs/011-correcao-auditoria-e2e/review.md.
- Evidence refs: none
- Limitations: Demo backend controlled; only frontend layout and native window covered, not provider inference. TK001 DPI pending separately. Evidence invalidated because an input changed.

## EV-006 — stale

- Ticket: `TK-003`
- Acceptance: `AC-003`
- Procedure: `Host TDD3 public cases and UI TDD6, UI66/66, typecheck, cargo workspace, golden update/retest and clippy -D warnings`
- Execution: `executed`
- Environment: Windows11 UI Vitest5, real Rust host with test OS and loopback discovery
- Tested revision: `local:f019579396ebb28ec9763c7574affcf5d88a106a94a48c5f090693552f209915`
- Timestamp: `2026-10-03T13:54:32+00:00`
- Observations: Save/remove/discovery invalidates; event literal providersChanged/event{} no secrets; mounted Overlay refreshes model/provider and keeps draft; out-of-order old response rejected; first local selected; error status rendered. Native baseline failed mounted textarea after save. Logs target/qa-tools/providers-*.
- Evidence refs: none
- Limitations: New native production executable validation pending; no paid inference or personal credentials. Evidence invalidated because an input changed.

## EV-007 — stale

- Ticket: `TK-003`
- Acceptance: `AC-003`
- Procedure: `Production WebDriver providers-refresh and panels with real pinned engine and loopback upstream; ignored real_app_server_byok_turn_and_mcp_tool warm thread before registering second provider; UI66/66, typecheck, workspace, golden, clippy aura-app/gateway -D warnings, fmt, diff`
- Execution: `executed`
- Environment: Windows11 WebView2/driver154.0.4258.53 production SHA2567CEF37E8E8BD4B9629B3CCC84FF4D84F1E40562932B93EE550F4F81C24B25772, isolated profile, pinnedrust-v0.159.0
- Tested revision: `local:19be09acbe7c0ed17dc5d37e20f4947ec310dd5b395544f4d1167cb6de2aef9a`
- Timestamp: `2026-10-03T14:11:48+00:00`
- Observations: Native mounted first save unlocked, selected local, discovered literal model,503 error visible draft preserved,last remove restoredLoginCard. New provider after engine running now creates real thread; ignored real test1/1 exercised BYOK turn and MCP. Logs providers-runtime-native-green.log, providers-real-server-warm-thread.log, panels-production-runtime-green.log, providers-runtime-workspace.log/providers-runtime-clippy.log; local review no blocker.
- Evidence refs: none
- Limitations: Loopback upstream response controlled, no paid provider or personal credentials; discarded stalled warm-models preparation is not a passed check. Evidence invalidated because an input changed.

## EV-008 — stale

- Ticket: `TK-002`
- Acceptance: `AC-002`
- Procedure: `Production panels.e2e with AURA_E2E_REAL_PROVIDER=1 and resize regression9/9 after TK003`
- Execution: `executed`
- Environment: Windows11, WebView2/driver154.0.4258.53, production7CEF37E8E8BD4B9629B3CCC84FF4D84F1E40562932B93EE550F4F81C24B25772,100percent isolated profile
- Tested revision: `local:619b5f71acfd6e6ab6eba66d69eec5b5e3035a3d858286d966e041ceb00a44d3`
- Timestamp: `2026-10-03T14:12:39+00:00`
- Observations: Both-direction exclusion, widths480/640/704/768, draft/Escape passed native production1/1; resize9/9. Logs panels-production-runtime-green.log and resize-after-providers-production.log. Local review renewed.
- Evidence refs: none
- Limitations: Upstream model response controlled; layout real. No DPI125/150 claim. Evidence invalidated because an input changed.

## EV-009 — stale

- Ticket: `TK-001`
- Acceptance: `AC-042`
- Procedure: `Production resize.e2e9/9 after TK003, native min drag and legacy placement; existing pure DPI placement suite rerun workspace`
- Execution: `executed`
- Environment: Windows11 WebView2/driver154.0.4258.53 production7CEF37E8E8BD4B9629B3CCC84FF4D84F1E40562932B93EE550F4F81C24B25772, two screens100percent
- Tested revision: `local:8c204c8435fe41fc8be26c6e4766da1a58516760856ae491b64cbdeba86eb19d`
- Timestamp: `2026-10-03T14:12:40+00:00`
- Observations: Expanded inner min360, input324,compact80,legacy64 corrected360. Native9/9 log resize-after-providers-production.log; workspace pure DPI examples passed. Revalidated current.
- Evidence refs: none
- Limitations: Native DPI125/150 not executed; QA001 original width variant remains unconfirmed. Evidence invalidated because an input changed.

## EV-010 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`
- Procedure: `Production resize.e2e9/9 after TK003; both screens100percent,480/640/900, all expanded borders/corners, compact East/West, menus/reopen`
- Execution: `executed`
- Environment: Windows11 WebView2/driver154.0.4258.53 production7CEF37E8E8BD4B9629B3CCC84FF4D84F1E40562932B93EE550F4F81C24B25772, two screens100percent
- Tested revision: `local:8c204c8435fe41fc8be26c6e4766da1a58516760856ae491b64cbdeba86eb19d`
- Timestamp: `2026-10-03T14:12:41+00:00`
- Observations: Height variant corrected and all9 native cases passed current frontend; log resize-after-providers-production.log.
- Evidence refs: none
- Limitations: Original width report not reproduced; native DPI125/150 unavailable on current screens, pure DPI does not replace native test. TK001 notdone. Evidence invalidated because an input changed.

## EV-011 — stale

- Ticket: `TK-004`
- Acceptance: `AC-004`
- Procedure: `pnpm -C apps/desktop test (72); typecheck; cargo test --workspace --exclude aura-desktop; ipc_contract; clippy app/desktop/core all-targets and win qa_tray -D warnings; fmt/diff; pnpm build; cargo build -p aura-desktop --release --features tauri/custom-protocol; WebDriver localization.e2e production (3)`
- Execution: `executed`
- Environment: Windows11 WebView2/driver154.0.4258.53 release production ECC7FD46E67E4410404E7A0A29F4F39D60847FBEFCA564FA0BFCC9A6617C8869 isolated profile
- Tested revision: `local:d1ab1ce18e0e1cf1a5b56b55161489de3bfdb58866658d91ad0713594f6e9ac9`
- Timestamp: `2026-10-03T14:59:59+00:00`
- Observations: 72 UI passed, workspace/types/golden/clippy/fmt/diff passed. Native3/3 title/DiagnosticsStopped/MCP/quick/custom/chip/aria/tray EN PT EN; logs localization-native-complete, localization-ui-all, localization-workspace, localization-typecheck-final, localization-golden-final, localization-clippy/tray-clippy. Local Standards+Spec review no blocker.
- Evidence refs: none
- Limitations: Ready(mock) and plural2 were React tested; native actual stateStopped and1line. Tray callback opens real HMENU; not physical icon click. Discarded failed clear-draft preparations final/final2 are not passed. Evidence invalidated because an input changed.

## EV-012 — stale

- Ticket: `TK-003`
- Acceptance: `AC-003`
- Procedure: `WebDriver providers-refresh production; cargo test -p aura-app --test real_app_server real_app_server_byok_turn_and_mcp_tool -- --ignored --nocapture; UI72/types/workspace/golden/clippy`
- Execution: `executed`
- Environment: Windows11 production ECC7FD46E67E4410404E7A0A29F4F39D60847FBEFCA564FA0BFCC9A6617C8869 pinnedrust-v0.159.0 loopback isolated
- Tested revision: `local:1d2f9b6cfe89daadc7bea51cd783b253c7737a53f68729b5501f63106500bfba`
- Timestamp: `2026-10-03T15:00:39+00:00`
- Observations: providers-after-localization1/1 and localization-real-server1/1 passed, hot-engine second provider real turn+MCP; 72UI/types/workspace/golden/clippy passed; local review no blocker.
- Evidence refs: none
- Limitations: No paid inference/personalcredentials, loopback controlled. Evidence invalidated because an input changed.

## EV-013 — stale

- Ticket: `TK-002`
- Acceptance: `AC-002`
- Procedure: `WebDriver panels.e2e with AURA_E2E_REAL_PROVIDER=1 production and loopback upstream; UI72/types`
- Execution: `executed`
- Environment: Windows11 production ECC7FD46E67E4410404E7A0A29F4F39D60847FBEFCA564FA0BFCC9A6617C8869 isolated
- Tested revision: `local:f00cc1546af1b5d7fe57b8768e14429a47dac39f277cd2b72ecb81090e1ea00b`
- Timestamp: `2026-10-03T15:00:39+00:00`
- Observations: panels-after-localization1/1 actual conversation480/640/1024 panel exclusion/draft/Escape passed; UI/types passed. Local review no blocker.
- Evidence refs: none
- Limitations: Loopback response controlled, actual pinned appserver. Evidence invalidated because an input changed.

## EV-014 — stale

- Ticket: `TK-001`
- Acceptance: `AC-042`
- Procedure: `WebDriver resize.e2e production actual mouse probe qa_resize, 9 cases; workspace/UI72/types/fmt`
- Execution: `executed`
- Environment: Windows11 production ECC7FD46E67E4410404E7A0A29F4F39D60847FBEFCA564FA0BFCC9A6617C8869 two monitors100percent
- Tested revision: `local:6b68f6d4befad587f6be31f2ec3a39cd2dd515660e6d6b823553230760e0beb0`
- Timestamp: `2026-10-03T15:00:40+00:00`
- Observations: resize-after-localization9/9 min internal360 input324 legacyrestored, border/corners/menu/reopen passed.
- Evidence refs: none
- Limitations: Native100percent only; 125/150puretests not native proof. Evidence invalidated because an input changed.

## EV-015 — stale

- Ticket: `TK-001`
- Acceptance: `AC-001`
- Procedure: `WebDriver resize.e2e production actual mouse probe qa_resize, 9 cases; workspace/UI72/types/fmt`
- Execution: `executed`
- Environment: Windows11 production ECC7FD46E67E4410404E7A0A29F4F39D60847FBEFCA564FA0BFCC9A6617C8869 two monitors100percent
- Tested revision: `local:6b68f6d4befad587f6be31f2ec3a39cd2dd515660e6d6b823553230760e0beb0`
- Timestamp: `2026-10-03T15:00:40+00:00`
- Observations: resize-after-localization9/9 widths480640900 and height refit passed, borders/corners bothmonitors.
- Evidence refs: none
- Limitations: Specific width report not reproduced, native125/150not executed; do notmarkcomplete. Evidence invalidated because an input changed.

## EV-016 — stale

- Ticket: `TK-005`
- Acceptance: `AC-005`
- Procedure: `Vitest bridge.test public concurrent identity/login/reset/injection/retry; pnpm -C apps/desktop test76/typecheck/build; real Vite1420 browser coldreload/click/welcome/input; cargo release custom-protocol and WebDriver localization3/3`
- Execution: `executed`
- Environment: Windows11 Vite8.3.1 IAB real preview; native production9886E59523207542E02E2363ADF88B0B9B0495B325853A3D5228E49E16C0D5E5 WebView2/driver154
- Tested revision: `local:3501f7d92df57adf80132ffdb7343f155d77592fac8aee91e33240b1065cca54`
- Timestamp: `2026-10-03T15:10:44+00:00`
- Observations: Concurrent red lostallloginevents; sharedPromisegreen uniqueinstance, signedIn/events; injection/reset/retry pass.76UI/typecheck/build/diff pass. Realbrowserloginadvancedwelcome/inputedit/Enviarenabled; proof target/qa-tools/bridge-preview-green.jpg. Nativebridge smoke3/3; logs bridge-concurrent-red/green, bridge-ui-final/typecheck-final, bridge-native-smoke. Local Standards/Spec review no blocker.
- Evidence refs: none
- Limitations: Previewloginissynthetic, noOAuthaccount authorization; adapterfailuretestedcontrolledexternalboundary. Evidence invalidated because an input changed.

## EV-017 — stale

- Ticket: `TK-006`
- Acceptance: `AC-006`
- Procedure: `cargo test -p aura-app --test host (27); workspaceRust; ipc_contract; UI78/typecheck/build; clippyapp/desktop alltargets com/sem e2e -Dwarnings; fmt/diff; cargo release e2e,tauri/custom-protocol; AURA_E2E_LOCAL_AUTH=1 WebDriverauth-cancel1/1; Vite real cancel/delayeddeadline/retry`
- Execution: `executed`
- Environment: Windows11 WebView2/driver154.0.4258.53 reale2e release A793A3CB23C907F51CC31CF95F5AAF63F43D858035D50102BB2A19E0F51716FE isolated profile; loopbackauthorize only; ViteIAB
- Tested revision: `local:238a8a82fc9e91737bb31a50c87d876a89a235f4d9865501a6ecebea40a78405`
- Timestamp: `2026-10-03T18:34:57+00:00`
- Observations: Hostred cancelled+failed; greenexactCancelled twice and genuinefailureonce.27Host/78UI passed, workspace/types/golden/build/clippy/fmt/diffpass. NativePT/EN waitingBrowser,cancelled,waitingBrowser,cancelled zeroFailed/accountnull; auth-cancel-native-final.log1/1/photoauth-cancel-native-green.png. Previewcancel persisted pastdeadline, retrysucceededWelcome/Input, photoauth-cancel-preview-green.jpg. Local Standards/Spec review no blocker.
- Evidence refs: none
- Limitations: OAuthfull authorization not executed; nativeexternalauthorize dependency replaced by loopback page via explicit e2e compilefeature excluded production. InvalidfakeTimer harness discarded, notred. UI/mockcurrenttests/Vite; nativecurrentfinalartifact retested. Evidence invalidated because an input changed.

## EV-018 — stale

- Ticket: `TK-005`
- Acceptance: `AC-005`
- Procedure: `UI78 inclbridge4/typecheck/build; Vite real retryaftercancel welcome/input; nativeWebDriverlocalization3/3 currentrelease`
- Execution: `executed`
- Environment: Windows11 Vite8.3.1 realIAB; nativee2erelease A793A3CB23C907F51CC31CF95F5AAF63F43D858035D50102BB2A19E0F51716FE WebView2154 noauthoverride in smoke
- Tested revision: `local:56ae822ddf0c349f44f22f91ac3f9f3afedac3156a324a91cd3c86e4b53729eb`
- Timestamp: `2026-10-03T18:34:58+00:00`
- Observations: Bridge4 regressionpassed, previewfreshlogin/cancel/retryreachedwelcome/input, nativeIPC/eventslocalization3/3 auth-cancel-localization-regression.log. No newblockingreviewfinding.
- Evidence refs: none
- Limitations: MockloginnotrealOAuth; nativeartifacte2efeature but loopbackoverrideflag absent forsmoke, actualhostengine no demo. Evidence invalidated because an input changed.

## EV-019 — stale

- Ticket: `TK-007`
- Acceptance: `AC-007`
- Procedure: `TDD core JSON/Host/UI; Store reopen; UI79/workspace/golden/typecheck/build/clippy/fmt; native microphone initial and separate-process restart.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 release normal no demo/e2e SHA256 697DBDE547A950E6380312ECF9A3BD6737E1990269E40A9E3E88B76854BDE5DD
- Tested revision: `local:b310df2d547ac191855f231f1b3a7970c43b86a290be39e54fd0d06498d916a3`
- Timestamp: `2026-10-03T19:00:28+00:00`
- Observations: Native two devices Fifine/CABLE selected, listening/cancelled; restart persisted CABLE, default cleared. Both logs 1/1; independent Host device-id assertions; review no blocking finding.
- Evidence refs: none
- Limitations: No ASR, retained PCM, or acoustic identification; device IDs remain adapter friendly names. Other audio findings pending. Evidence invalidated because an input changed.

## EV-020 — stale

- Ticket: `TK-008`
- Acceptance: `AC-008`
- Procedure: `React red/green, native red on TK007 and green on current; UI82/typecheck/build/diff; real microphone initial/restart regression.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production custom-protocol no demo/e2e SHA256 244AE84A21082999168805CA8089AA4EB8D93C8D10B92BFAB72ADB51C03D7E03
- Tested revision: `local:fcf9078439a6727e983e8cd23185d885fa0218bb6dcd172350b174e5eca609a8`
- Timestamp: `2026-10-03T19:09:38+00:00`
- Observations: Repeat across cycles and after clear passed native1/1; duplicate Done ignored; React batching and two auto-send IPC requests passed; review no blocker.
- Evidence refs: none
- Limitations: ASR outputs controlled at event seam; no acoustic ASR model or latency approval. Native UI/IPC/WebView real; microphone regression real WASAPI without ASR. Evidence invalidated because an input changed.

## EV-021 — stale

- Ticket: `TK-009`
- Acceptance: `AC-009`
- Procedure: `Core JSON red/green; UI PT/EN release language null and selector; native fixed->auto red then hold green and separate-process restart; repeat regression; core/app/golden/UI85/types/build/clippy/fmt/diff.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 normal production no demo/e2e SHA256 8F5C9AC15A31D805DFA6D5FA9E72699C537FF4CE24176ECA18C888B74A6B6B99
- Tested revision: `local:715486b6b520158e8145d8c70434ec672f308c6ffaeb1b0822516c1460c8caa2`
- Timestamp: `2026-10-03T19:23:57+00:00`
- Observations: Native selected Spanish then auto null, restarted preserved; PT/EN real hold Listening/Empty. React external bridge proves null requests. Native repeat1/1. Standards/Spec review no blocker in language slice.
- Evidence refs: none
- Limitations: No acoustic model/autodetection quality, latency or physical hotkey approval. Native invoke immutable, rejected observer harness not product red. Fast click race confirmed and assigned TK010, unresolved here. Evidence invalidated because an input changed.

## EV-022 — stale

- Ticket: `TK-010`
- Acceptance: `AC-010`
- Procedure: `React click/keyboard/pending/error/cancel red-green; native Enter red, focus loss red and final toggle green; UI92/types/build/diff; language/repeat/microphone native regressions and resize9.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 normal production no demo/e2e SHA256 805B30DFE31F825661E95584F495F69EA6D64037784DF9CC413F8DEECC2D01F5
- Tested revision: `local:88165cce17a49e3fd1f71c66d0defb405bf045f1b1d1046ff73f833993b8d7ab`
- Timestamp: `2026-10-03T19:41:09+00:00`
- Observations: Native Enter->Space focus BUTTON, mouse toggle/leave/Esc1/1. Language initial/restart, repeat and microphone initial/restart1/1 each; resize9/9. UI pending/retry/Partial cancel covered. Standards/Spec review no final blocker.
- Evidence refs: none
- Limitations: No acoustic ASR/model/latency/physical hotkey or DPI125/150 approval. Prior disabled release failed focus; CtrlSpace filter skipped and full92 actually ran. No retained audio. Evidence invalidated because an input changed.

## EV-023 — passed

- Ticket: `TK-011`
- Acceptance: `AC-011`
- Procedure: `Host/config/UI red-green (model_missing Failed, no Fake in production, card offer/cancel/retry/complete/language); Rust red-green recommendation requires worker GPU inference; worker --capabilities test; native red on TK-010 then model-install E2E 3/3 on release with real Parakeet V3 download, hashes, restart and SAPI->VB-Cable speech transcribed through UI; voice/mic/language native regressions; resize 9/9; card fit; UI96/workspace/desktop/clippy/fmt/golden/typecheck/build.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production custom-protocol no demo/e2e SHA256 E593D5B07783B3D6087394F2986BCC1CC1EBFBCC5136385179F991803F91CA7F; aura-worker engines CPU-only SHA256 B2A273F8968324109C209CD56A4060A614AB7E71617AB09FA62EB40D99FAD68F; Ryzen 7 7800X3D; VB-Cable
- Tested revision: `local:9d0942aa2059ec04a4ce14014b65f726a3d1d6f908e9111da8d443f1ae071bc1`
- Timestamp: `2026-10-03T21:03:29+00:00`
- Observations: Card Parakeet V3 640 MB; cancel shows neutral status; five SHA-256 match models.toml; selected and persisted after restart, microphone opens; real transcript 'The quick brown fox jumps over the lazy dog.' with partial states. Earlier run exposed Whisper Turbo CPU 100-160s and timeout; fixed via gpu_inference. Regressions toggle/repeat/microphone x2/language x2 1/1, resize 9/9.
- Evidence refs: none
- Limitations: GPU (vulkan) worker not built/tested; Whisper Turbo still selectable but slow on CPU-only worker. Worker built with GGML_NATIVE (AVX-512 here) - distribution risk outside this ticket. Synthetic speech only, no WER measurement. Harness-only failed attempts not counted.

## EV-024 — passed

- Ticket: `TK-012`
- Acceptance: `AC-012`
- Procedure: `React red/green catalog metadata and localized descriptions; native red on TK-011 release, green voice-catalog E2E EN/PT on release; UI98/typecheck/build.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production custom-protocol SHA256 E6243060C5E493688B60EA00B6330C047F95DC36DBE1D9957C1FA087708D8FBF
- Tested revision: `local:cd77a2e790d927222d5ea010e2f50580744843b46ea19d4faf256884412e59ab`
- Timestamp: `2026-10-03T21:12:13+00:00`
- Observations: Parakeet V3 85/85, 25 languages incl pt, CPU min 2 GB RAM, English description; Whisper Turbo 40/82 multilingual GPU recommended min 4 GB; PT labels after switch.
- Evidence refs: none
- Limitations: Bars reflect catalog ratings, not measured on this machine.

## EV-025 — passed

- Ticket: `TK-013`
- Acceptance: `AC-013`
- Procedure: `Core/Host/UI/golden red-green; native red (no test command) then audio-test E2E with VB-Cable + SAPI on release; regressions microphone x2, toggle, catalog, real ASR; workspace/clippy/fmt/UI99/typecheck/build.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production custom-protocol SHA256 1330E10C452BA4C0E0C815FBC4EEFADF0520E410284865DE8436001FAB357450; VB-Cable; SAPI
- Tested revision: `local:1f6e0c4c70565a82508ec86d1c64ca74b52fe31f3bd3ffd16c0920120818ed9a`
- Timestamp: `2026-10-03T21:38:08+00:00`
- Observations: Mic CABLE Output 28 Hz peak -16.6 dBFS; system loopback CABLE Input silence -63.7 to speech -14.7; missing output and missing mic fall back with localized warning; stop no longer hangs on silent loopback; preference persisted.
- Evidence refs: none
- Limitations: Test tone plays on default output only. Rate measured from events, not screen frames.

## EV-026 — passed

- Ticket: `TK-014`
- Acceptance: `AC-014`
- Procedure: `React red/green effort section, capabilities, wire-aware provider efforts; UI102/typecheck/build; native model-effort E2E with loopback upstream capturing request body; providers-refresh and panels regressions.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production custom-protocol SHA256 0C9BA80208F4CFA417FB214A0BD4BFD5D0E71ED0BC1169E6E1875DA7B1A8EBE1; app-server rust-v0.159.0; loopback upstream
- Tested revision: `local:24b20053346e86a7523e9d92c21e72893f877dab018e991b6e526c7b8f52467d`
- Timestamp: `2026-10-03T22:50:18+00:00`
- Observations: qa-reasoner Images/Tools/Reasoning with Low/Medium/High; qa-plain Text only, no effort; upstream received reasoning.effort=high for qa-reasoner.
- Evidence refs: none
- Limitations: Native red not run (previous binary overwritten). Settings defaultEffort not validated per model by Host. First native attempt timed out on cold app-server start.

## EV-027 — passed

- Ticket: `TK-015`
- Acceptance: `AC-015`
- Procedure: `React rename red/green; aura-codex history provider red/green; native red on TK-014 release and green history E2E with real app-server and loopback upstream including restart; UI suite, clippy.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 4852618E9AFD744B6B35E62841D0603F43B056D7BB08DE9FF0661B204A5CA88E; app-server rust-v0.159.0
- Tested revision: `local:6c624dd47aff8b08775ee495e82c60ea6bcf357d7affd0b1714cc60a43e9adb5`
- Timestamp: `2026-10-03T22:59:16+00:00`
- Observations: BYOK conversation now listed; renamed via UI to 'QA renamed via UI', persisted after restart.
- Evidence refs: none
- Limitations: Upstream answers errors (no inference); rename of ChatGPT-plan conversations exercised only via fake app-server.

## EV-028 — passed

- Ticket: `TK-016`
- Acceptance: `AC-016`
- Procedure: `React paging red/green with 120 mock conversations; native red on TK-015 release; native green with 55+ real app-server conversations; UI suite.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 D00052803DFBACAA9FB5A27C034F343E5C8FAFAF770DFB6C8E8CC5401EB1D357; app-server rust-v0.159.0
- Tested revision: `local:0b1620029d384db47c151ed72a0ba1d7958356f26ab649324c077ccef6257d1b`
- Timestamp: `2026-10-03T23:07:35+00:00`
- Observations: First page 50 without oldest; Load more reached 100 rows and showed QA page 001.
- Evidence refs: none
- Limitations: Scroll-triggered loading not implemented; explicit button only.

## EV-029 — passed

- Ticket: `TK-017`
- Acceptance: `AC-017`
- Procedure: `React red/green /compactar; native red on TK-016 release; native green with real app-server and deterministic Responses loopback including post-compaction turn; UI suite/typecheck.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 4D9E19649079A8FA3723DABADE9B238D390060E1EC39BD6188004DD4F959FDE7; app-server rust-v0.159.0
- Tested revision: `local:02875039ade68f79a536e154af3db1fb940e8cdfa76c97e13f10ad4cdca56cd9`
- Timestamp: `2026-10-03T23:23:28+00:00`
- Observations: Compaction request with app-server summary prompt; separator shown; next turn carries summary and not the old answer; command never sent as prompt.
- Evidence refs: none
- Limitations: Deterministic loopback model; real summary quality not evaluated. Fresh-profile cold start can exceed 150 s.

## EV-030 — passed

- Ticket: `TK-018`
- Acceptance: `AC-018`
- Procedure: `Host red/green clipboard image; React red/green paste; native red on TK-017 release; native green with real Windows clipboard PNG, Ctrl+V, real app-server turn to loopback upstream.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 A93227EE1CE9F85CBCF92B36C7B00BEE461DF44BBB25A67864FCDDE5418A1022; app-server rust-v0.159.0
- Tested revision: `local:2c830d3331c0f8f578e2a4fd37848fb53fb709e66f66c87589a76bb415773c07`
- Timestamp: `2026-10-03T23:29:56+00:00`
- Observations: Chip clipboard-<stamp>.png created; model request contained input_image data:image/png;base64.
- Evidence refs: none
- Limitations: Native clipboard exercised with PNG only (WebView2 exposes clipboard bitmaps as PNG); JPEG/WebP/GIF covered by Host type checks.

## EV-031 — passed

- Ticket: `TK-019`
- Acceptance: `AC-019`
- Procedure: `React red (original file)/green provider edit; native red on TK-018 release; native green editing keyless provider name/URL with two loopback upstreams and existing conversation turn.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 CAA738B245738123A50B10819AF701BBC5BE8B6E29818132848A166E355EAF02; app-server rust-v0.159.0
- Tested revision: `local:fd92ffc58cc1e245370281d558c42fde04be2493ed53798daf322526c366dc4f`
- Timestamp: `2026-10-03T23:35:12+00:00`
- Observations: Same provider id after edit; models from new URL; existing conversation reached the new URL only.
- Evidence refs: none
- Limitations: Key replacement verified in React/mock only; native run used a keyless provider to avoid writing to the personal Windows Credential Manager.

## EV-032 — passed

- Ticket: `TK-020`
- Acceptance: `AC-020`
- Procedure: `Registry red/green auth+header validation; React red/green; native red on TK-019 release; native green creating Custom provider via UI, chat then anthropic after edit, loopback recording paths and headers.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 7234891D2C2E816C2F95198AEC533926B6BBC40E4519376A963C779EA78DC0D8; app-server rust-v0.159.0
- Tested revision: `local:35d8c93186e9f1cb89477b5d44babeb87fabe41577e0a9dea075e5f7d9d56e5e`
- Timestamp: `2026-10-03T23:41:03+00:00`
- Observations: /v1/models and /v1/chat/completions carried X-QA-Tenant; after edit /v1/messages carried it plus anthropic-version.
- Evidence refs: none
- Limitations: Keyless provider: x-api-key auth for Anthropic custom verified by unit test only (avoids writing to the Windows Credential Manager).

## EV-033 — passed

- Ticket: `TK-021`
- Acceptance: `AC-021`
- Procedure: `Registry tests; Host red/green manual model and 404 status; React red/green; native red on TK-020 release; native green adding model via UI, re-test, picker and real turn to loopback without /models.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 F20F3BBAD47450F2BE2E25CB89A583946589F2329B491B098A4646AE6BBFC4D4; app-server rust-v0.159.0
- Tested revision: `local:788ce5daefe008e734374b93b5edf04c29475b9f2d013b08f45d4bdd404f6c57`
- Timestamp: `2026-10-03T23:51:46+00:00`
- Observations: Manual model persisted through re-test (status unverified, no error), shown with Tools/Reasoning in picker, turn sent with model qa-manual-model.
- Evidence refs: none
- Limitations: Registry unit test was written together with the implementation (no separate red). Max output not editable in UI.

## EV-034 — passed

- Ticket: `TK-022`
- Acceptance: `AC-022`
- Procedure: `Extensions/home/Host/React red-green; service tests with protocol-faithful fake; JSON-RPC probe of pinned app-server; native red on TK-021 release; native green create/list/disable/new-conversation/edit/restart.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 D4908B227F4E473FFC59B9CD955FA52B85868759AD0DD3C02C3736F228CD0DAD; app-server rust-v0.159.0
- Tested revision: `local:16d5af359b0c1d1c55b830bf4e44da9049a235a942ff1ea3f843112b9dcc699b`
- Timestamp: `2026-10-04T00:11:26+00:00`
- Observations: Origins aura/user/system; disabled skill absent from new conversation request; SKILL.md updated; disabled state persisted after restart.
- Evidence refs: none
- Limitations: Service and slash-menu tests written together with code. Only Aura skills are editable; user/system skills are toggle-only by design.

## EV-035 — passed

- Ticket: `TK-023`
- Acceptance: `AC-023`
- Procedure: `Unit red/green parser; React red/green; native red on TK-022 release; native green starting a real MCP server from a folder with spaces via app-server and checking its argv.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 F263175C4757EC133CF7975E88E81FD7549C915A923C716DE70E36C0EA335B03; app-server rust-v0.159.0; Node
- Tested revision: `local:1dfddb68ef0d995aab17090d08ce8c541d28a03b6a8c3a3267d400fdc2926f63`
- Timestamp: `2026-10-04T00:17:34+00:00`
- Observations: Saved args and server argv exactly as intended, including empty argument; tools listed.
- Evidence refs: none
- Limitations: A quoted value ending in a backslash before the closing quote is read as an escaped quote (documented semantics).

## EV-036 — passed

- Ticket: `TK-024`
- Acceptance: `AC-024`
- Procedure: `React red/green tool toggles; native red on TK-023 release; native green with real MCP server, UI toggle, agent restart and model request tools inspection.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 1546F528946335BB15041DEC1220AB67064D2FC2F97A5CDC87A2791B7F894CC9; app-server rust-v0.159.0; Node MCP server
- Tested revision: `local:611d6312af7f47738a4f0a46f69139654cfb2b2f2ea9a2557bb3009d5c6a0a8e`
- Timestamp: `2026-10-04T00:22:41+00:00`
- Observations: Before: qa_echo and qa_write offered; after disabling qa_write: only qa_echo offered to the model in a new conversation.
- Evidence refs: none
- Limitations: Change applies after the agent restarts (existing notice), as before for MCP edits.

## EV-037 — passed

- Ticket: `TK-025`
- Acceptance: `AC-025`
- Procedure: `Diag unit tests; Host red/green status error and diagnose; React red/green; native red on TK-024 release; native green with real ok/failing MCP servers and log view.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 2F08AC78D08497588AF9F6254AE575400DAFF9654F4D075D05650C2030E714FB; app-server rust-v0.159.0; Node MCP servers
- Tested revision: `local:2fdc6e12541a9c55a999c349c2c4d682f400bacda6f0dfb36f43cfa88d4e0e9a`
- Timestamp: `2026-10-04T00:38:01+00:00`
- Observations: Connected and error states shown; diagnosis log showed exit code 3 and the server's stderr line.
- Evidence refs: none
- Limitations: Log lines come from an on-demand diagnostic run (stdio only), not from the app-server's own process; HTTP servers show the error reason only.

## EV-038 — passed

- Ticket: `TK-026`
- Acceptance: `AC-026`
- Procedure: `Host red/green memories; React red/green; JSON-RPC probe of memory methods; native red on TK-025 release; native green with seeded memory files, real app-server and loopback request inspection.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 7BD6A4181777F8A91547536515C5FC8967091F62D45376FB459ECCBB3ADE39DC; app-server rust-v0.159.0
- Tested revision: `local:e6737ec3bda7956529b3127598112fb404874f1d48c9a4b0092b60003e867bcc`
- Timestamp: `2026-10-04T00:46:28+00:00`
- Observations: Request carried alpha+beta, then only alpha after forgetting beta, then neither after forgetting everything; consolidation note written.
- Evidence refs: none
- Limitations: Memory files were seeded (real generation needs model calls). Consolidation applying the notes was not exercised (requires real model).

## EV-039 — passed

- Ticket: `TK-027`
- Acceptance: `AC-027`
- Procedure: `Native E2E on packaged release: real conversation, HTML file with script/fetch/remote image in workspace, open preview, inspect iframe, count loopback requests; screenshot.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 7BD6A4181777F8A91547536515C5FC8967091F62D45376FB459ECCBB3ADE39DC; app-server rust-v0.159.0
- Tested revision: `local:808e43f03e4b3e40b54ca96765cd9098e3e98c4b3c9b0bc1516fd4279b284035`
- Timestamp: `2026-10-04T00:47:43+00:00`
- Observations: Preview rendered heading and inline style; script did not run; remote image blocked; zero network requests.
- Evidence refs: none
- Limitations: No product change; relies on WebView2 not applying frame-src to about:srcdoc (checked on 154).

## EV-040 — passed

- Ticket: `TK-028`
- Acceptance: `AC-028`
- Procedure: `Host red/green PDF text preview; React red/green; native red on TK-026 release; native green with generated PDFs in a real conversation workspace.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 392B583E890AF43936A0730831FEDE11010D7157B057DF98CA7A63C8296771D8; app-server rust-v0.159.0
- Tested revision: `local:3cf97861f078ef6d6723dd4c5fed5bce0505282c7782b9395cd9301dafd9c5f4`
- Timestamp: `2026-10-04T00:54:24+00:00`
- Observations: Text preview 5 of 7 pages with page headings; scanned-PDF explanation shown.
- Evidence refs: none
- Limitations: Text-layer preview, not a rendered page image; layout requires Open (default viewer).

## EV-041 — passed

- Ticket: `TK-029`
- Acceptance: `AC-029`
- Procedure: `Host red/green range validation; React red/green minutes field; native red on TK-028 release; native green setting/clamping minutes and restart.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 D982A0A352F1F8A0712C2CB795714352B659E7153A71992D309F796870DB8AE6
- Tested revision: `local:c55da2c5ab4122c95cb003593450fc51d5506aa43fd7121fcb4d716fb96c904e`
- Timestamp: `2026-10-04T01:00:42+00:00`
- Observations: 2 min saved, 45 clamped to 30, persisted after restart.
- Evidence refs: none
- Limitations: Buffer pruning by the chosen minutes relies on existing retention tests; not timed natively. Test screen segments deleted.

## EV-042 — passed

- Ticket: `TK-030`
- Acceptance: `AC-030`
- Procedure: `Host red/green retention settings and recorder wiring; React red/green; native red on TK-029 release; native green via Privacy UI with restart.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 C7CFF8986A0D742914FBF1E90AEF558DD00533FC30D02FFE37C82E4F8FC6805B
- Tested revision: `local:66264bd45bce3995899762c7e35cf4c8d11ca7482ab804105c74353b039c2213`
- Timestamp: `2026-10-04T01:08:25+00:00`
- Observations: UI saved 3 days/5 GB/manual; recorder received the limits; persisted after restart.
- Evidence refs: none
- Limitations: Deletion by age/size relies on the existing pure retention-plan tests; not timed natively with real old segments.

## EV-043 — passed

- Ticket: `TK-031`
- Acceptance: `AC-031`
- Procedure: `Host red/green for consent outcome and sealed thumbnail; Host test for thread link, thumbnail data URL and openConversation event; React red/green; native red on TK-030 release; native green with loopback model calling mcp__aura.screen_capture through the real app-server, consent in UI, Settings row, thumbnail and Open conversation.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 D45ADD2E507EDC1F48EEA0878DAD49EB9C1E6C6E7B4E20E34A055CCCDE71E634
- Tested revision: `local:13ed17d02d88f3ebdad72cfbb63424fc0f9780ba876ae97556df904231cea636`
- Timestamp: `2026-10-04T02:29:00+00:00`
- Observations: Entry allow/consent with 240x126 sealed thumbnail and threadId; row shows exact date/time with seconds; conversation reopened in Overlay. Log cleared after the run.
- Evidence refs: none
- Limitations: Only screen_capture stores a thumbnail (screen_text/recent tools log outcome without image). Timeout path covered by code review, not timed natively (consent timeout is minutes).

## EV-044 — passed

- Ticket: `TK-032`
- Acceptance: `AC-032`
- Procedure: `Host test for duration, playback copies, attach and cleanup; React red/green; native red on TK-031 release; native green with VB-Cable/SAPI microphone and screen manual recording, WebView2 player metadata and playback, attach to Overlay with Parakeet V3 profile.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 72D500AB43BBCA4FA617727F71538A3572D9B54EE20D41A0403127B9F26D7A0D
- Tested revision: `local:7f8441a5ce110f04e6bec59550780c4676d748b22c2a9e5012645ae2209b8012`
- Timestamp: `2026-10-04T02:46:09+00:00`
- Observations: 10.05 s wall recording reported 10207 ms and 0:10; audio 10.19 s and video 10 s loaded without media error; audio advanced 1.42 s; mic.wav attached and transcribed. Recordings deleted and playback cache empty after the run.
- Evidence refs: none
- Limitations: Attaching audio needs an installed speech model (transcription, 007); without it the error is shown. 60 s system-audio case of 005 AC-004 not timed; duration precision observed ±0.2 s on 10 s. Multi-segment screen video plays per segment, not as one stitched file.

## EV-045 — passed

- Ticket: `TK-033`
- Acceptance: `AC-033`
- Procedure: `Host red/green for clip, validation, open-segment flush and workspace adoption; gateway body-limit red/green; React red/green; native red on TK-032 release; native green with screen + VB-Cable mic/system buffers, Parakeet V3 transcription and loopback model turn.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 30D1CB1319C94EBCA7B20BCBA04919D41033A056092BB5426E6B90A07F6E4CC6
- Tested revision: `local:3d513b04b43c5a69aaf41484ce2e86ae3f4ed4386e532374f550e1dbe74d6043`
- Timestamp: `2026-10-04T15:44:20+00:00`
- Observations: Clip chip in 2.4 s; turn had 8 images, 8 frame labels, Você and Sistema transcript lines of the spoken sentence; 8 frames and mic/system WAV in the conversation workspace. Screen frames and segments removed after the run.
- Evidence refs: none
- Limitations: Clip chip has no duration badge/thumbnail rendering beyond the existing chip preview; the 2-minute case was exercised with 1 minute. ASR quality varies (system line misheard 'quarterly').

## EV-046 — passed

- Ticket: `TK-034`
- Acceptance: `AC-034`
- Procedure: `Host red/green for voices, cloud consent gate, cloud request and cleanup on provider removal; React red/green; native red on TK-033 release; native green with real Windows voices, loopback OpenAI-compatible /audio/speech and loopback model for auto-read.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 2C3BDB8D5FFA494E991B310968B11B786466BB5FA81244B158EC769C1EF7E672
- Tested revision: `local:bdbfce66d43ef22601a3ed0e248d8530d2598b51c31698b4efe825afa64f5b73`
- Timestamp: `2026-10-04T16:10:54+00:00`
- Observations: Two installed voices produced different WAV; consent dialog before any cloud request; cloud request body model tts-1/voice alloy/response_format wav; auto-read sent the finished answer once without a second consent.
- Evidence refs: none
- Limitations: Audible output quality not judged (no speaker capture); OpenAI real endpoint not called (loopback only). Ctrl+Shift+L belongs to TK-035.

## EV-047 — passed

- Ticket: `TK-035`
- Acceptance: `AC-035`
- Procedure: `React red/green for both shortcuts and selection; native red on TK-034 release; native green with loopback model, consented loopback voice and a QA WinForms target window brought to the foreground.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 1AD8A59D81331BB5EBFCDB57FA330422FAEFC230E9A95184AAFCA27585F07480
- Tested revision: `local:4714fc91a8dfd42dea66885d7e2a20a19e8e68ecb7c351b84c0f331cb0e4bd0a`
- Timestamp: `2026-10-04T16:18:49+00:00`
- Observations: Voice received the last answer on Ctrl+Shift+L; target text box received exactly the answer on Ctrl+Shift+Enter; clipboard restored to its previous QA value.
- Evidence refs: none
- Limitations: Overlay reopened through the access-log link path (overlay::show) instead of the global hotkey: synthesized Ctrl+Shift+Space did not reach the hotkey in the test session. Selected-code insertion verified in React only.

## EV-048 — passed

- Ticket: `TK-036`
- Acceptance: `AC-036`
- Procedure: `React red/green for the profile model field and provider+model application; native red on TK-035 release; native green creating the profile in Settings and opening the Overlay with the global shortcut over a QA WinForms window, turn sent to the loopback provider.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 00A3382345E8C1D376A9535B712CFA7DB94003848369B02B7C5EC6B7C50C6E1B
- Tested revision: `local:a79a7d842d2ed5487323b35e87d1d25036b522cc783243c9487731578044d5a2`
- Timestamp: `2026-10-04T16:25:02+00:00`
- Observations: Profile saved aura-<provider>::qa-other; active profile QA target; model button qa-other; upstream request model qa-other.
- Evidence refs: none
- Limitations: Profile model applies to new conversations only (existing thread keeps its model), as before.

## EV-049 — passed

- Ticket: `TK-037`
- Acceptance: `AC-037`
- Procedure: `Host red/green with independent size oracle; mask unit test; React green; native red on TK-036 release; native green comparing disk free with PowerShell Get-PSDrive and checking the Diagnostics screen.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 A7C6F27088162222FADB2750D51CAA57891C2F0BE940B98467533006BB1C7E49
- Tested revision: `local:b973790b505d22eba96bce149abbd69232922f02d05c7bb42b4d6c173425b3ba`
- Timestamp: `2026-10-04T16:33:19+00:00`
- Observations: Gateway reachable, aura MCP 6 tools, worker installed/idle CPU with Parakeet V3, mic active, free space within 0.01% of PowerShell, Aura folder 1.1 GB.
- Evidence refs: none
- Limitations: No ChatGPT account signed in on the QA profile, so the masked account line was verified in Host/React tests only. Windows 10 clean VM of 010 SC-001 not run.

## EV-050 — partial

- Ticket: `TK-038`
- Acceptance: `AC-038`
- Procedure: `aura-core unit tests for key detection; React red/green; guard script run against the real config (exit 1), a synthetic valid config (exit 0) and CI without secret (exit 1); native red on TK-037 release and green on TK-038 release.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 487BAE28C1E687B5F24A595A4407F05F862542DC3DB84EE7239BB50442DDF25D
- Tested revision: `local:0f26d86b0146c3f5dc35d3a3c6de08e6b82ab89fc8bb3cce7988527bb77a1a43`
- Timestamp: `2026-10-04T16:39:19+00:00`
- Observations: Placeholder key detected; UI explains updates are not configured; release guard blocks unsigned/placeholder releases.
- Evidence refs: none
- Limitations: No real signing key generated (maintainer decision) and no signed update downloaded/applied; 010 AC-004 remains open.

## EV-051 — passed

- Ticket: `TK-039`
- Acceptance: `AC-039`
- Procedure: `React red/green; native red on TK-038 release; native green clicking Resume getting started in Settings and observing the guide in the Overlay.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 46BBFEE8F455CEC6E363F0D0822106E2B620EF5910BF642F0E21247E467418B4
- Tested revision: `local:f17b33ba898d4fb0516d00734826b4df8400ae932bb1fae384926ea972646e1a`
- Timestamp: `2026-10-04T16:43:58+00:00`
- Observations: Guide shown again at step 1 of 3 after the click; onboarded=false persisted.
- Evidence refs: none
- Limitations: The guide still has 3 steps (privacy, shortcut, voice); sign-in and the guided first request of 010 AC-006 happen outside it (login card / empty state).

## EV-052 — passed

- Ticket: `TK-040`
- Acceptance: `AC-040`
- Procedure: `React red/green on the guide text; native red on TK-039 release; native green reading the shortcut step and pressing Escape.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 8B25626FA95AC56E19DFA626FAC8B3C7A0D74AD49AC6A2EC89B4D0E6929DB882
- Tested revision: `local:2995f975ef1f6ec06f92a05531454558dea225fb258dd8b414a3535606347806`
- Timestamp: `2026-10-04T16:47:54+00:00`
- Observations: New text shown; guide still displayed after Escape.
- Evidence refs: none
- Limitations: Overlay visibility after Esc checked through the guide element staying displayed, not through the OS window state.

## EV-053 — passed

- Ticket: `TK-041`
- Acceptance: `AC-041`
- Procedure: `Native red on TK-040 release and green on TK-041 release reading visible top-level windows of the Aura process with EnumWindows/GetWindowRect/DWM cloak, then launching the executable again; regression specs on the same build.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 E2693A146AADB157DD0BDC26B482AC137BEC80CF4EACDD9488ED4F7C33BC99A3
- Tested revision: `local:82b4326bd420c58bff52a813838c3f193ed9eec86f57c037eae8265439b17a6e`
- Timestamp: `2026-10-04T16:53:39+00:00`
- Observations: No user-visible window at start; Overlay 656x89 after relaunch; three existing E2E specs still pass.
- Evidence refs: none
- Limitations: Toast content not captured from the Action Center; logon autostart (001 AC-015) not executed; tray menu items not clicked in this run.

## EV-054 — partial

- Ticket: `TK-001`
- Acceptance: `AC-001`
- Procedure: `WebDriver resize.e2e on the final release with the qa_resize SendInput probe (9 cases: compact side handles, widths, external resize, expanded minimum, every edge/corner, context menu, hide/reopen width, two monitors, legacy placement); spec now opens the Overlay by relaunching (Aura starts in the tray since TK-041). Final regression: cargo test --workspace --exclude aura-desktop (45 suites), clippy -D warnings (workspace and aura-desktop with e2e), fmt, ipc_contract golden, UI 140/140, typecheck.`
- Execution: `executed`
- Environment: Windows11 WebView2/driver 154.0.4258.53 production custom-protocol SHA256 E2693A146AADB157DD0BDC26B482AC137BEC80CF4EACDD9488ED4F7C33BC99A3; fresh isolated profiles; DPI 100%
- Tested revision: `local:75bce8e97b44212e17d3c656cd624cd1ac00eb0041ca3074afbdbce34f98992f`
- Timestamp: `2026-10-04T17:09:07+00:00`
- Observations: 9/9 passing on the final build.
- Evidence refs: none
- Limitations: QA-001 still partial: native drag at 125%/150% DPI not executed (needs changing the user's display scaling) and the specific width report was not reproduced.

## EV-055 — passed

- Ticket: `TK-002`
- Acceptance: `AC-002`
- Procedure: `WebDriver panels.e2e with AURA_E2E_REAL_PROVIDER=1 (real pinned app-server, loopback upstream) on the final release; selector updated to the model item that now also shows capabilities (TK-014). Final regression: cargo test --workspace --exclude aura-desktop (45 suites), clippy -D warnings (workspace and aura-desktop with e2e), fmt, ipc_contract golden, UI 140/140, typecheck.`
- Execution: `executed`
- Environment: Windows11 WebView2/driver 154.0.4258.53 production custom-protocol SHA256 E2693A146AADB157DD0BDC26B482AC137BEC80CF4EACDD9488ED4F7C33BC99A3; fresh isolated profiles; DPI 100%
- Tested revision: `local:3aba24e65802e07023d5f515e290d99e49b7101a936552dbb908b33a86dd8c60`
- Timestamp: `2026-10-04T17:09:07+00:00`
- Observations: History/Files exclusive panels and conversation preservation passed (1/1).
- Evidence refs: none
- Limitations: Loopback provider, no inference.

## EV-056 — passed

- Ticket: `TK-003`
- Acceptance: `AC-003`
- Procedure: `WebDriver providers-refresh.e2e on the final release with a fresh profile and the pinned app-server. Final regression: cargo test --workspace --exclude aura-desktop (45 suites), clippy -D warnings (workspace and aura-desktop with e2e), fmt, ipc_contract golden, UI 140/140, typecheck.`
- Execution: `executed`
- Environment: Windows11 WebView2/driver 154.0.4258.53 production custom-protocol SHA256 E2693A146AADB157DD0BDC26B482AC137BEC80CF4EACDD9488ED4F7C33BC99A3; fresh isolated profiles; DPI 100%
- Tested revision: `local:3adb376dda5cebc6282a5e8379e6aca047782131b16556812739ebf5b299b32f`
- Timestamp: `2026-10-04T17:09:08+00:00`
- Observations: Provider catalog refresh without remounting the Overlay passed (1/1).
- Evidence refs: none
- Limitations: Loopback upstreams only.

## EV-057 — passed

- Ticket: `TK-004`
- Acceptance: `AC-004`
- Procedure: `WebDriver localization.e2e (EN/PT/EN, window titles, tray texts via qa_tray probe, Diagnostics/MCP) on the final release. Final regression: cargo test --workspace --exclude aura-desktop (45 suites), clippy -D warnings (workspace and aura-desktop with e2e), fmt, ipc_contract golden, UI 140/140, typecheck.`
- Execution: `executed`
- Environment: Windows11 WebView2/driver 154.0.4258.53 production custom-protocol SHA256 E2693A146AADB157DD0BDC26B482AC137BEC80CF4EACDD9488ED4F7C33BC99A3; fresh isolated profiles; DPI 100%
- Tested revision: `local:747877d8aec6992606221bd4eebac5fb68efc9579bfd732aaedb392641befea6`
- Timestamp: `2026-10-04T17:09:08+00:00`
- Observations: 3/3 passing.
- Evidence refs: none
- Limitations: Tray verified by probe, not a physical click.

## EV-058 — passed

- Ticket: `TK-005`
- Acceptance: `AC-005`
- Procedure: `Vitest bridge tests (concurrent init, reset/injection, retry) inside UI 140/140 and native localization.e2e 3/3 on the final release. Final regression: cargo test --workspace --exclude aura-desktop (45 suites), clippy -D warnings (workspace and aura-desktop with e2e), fmt, ipc_contract golden, UI 140/140, typecheck.`
- Execution: `executed`
- Environment: Windows11 WebView2/driver 154.0.4258.53 production custom-protocol SHA256 E2693A146AADB157DD0BDC26B482AC137BEC80CF4EACDD9488ED4F7C33BC99A3; fresh isolated profiles; DPI 100%
- Tested revision: `local:c00853b407525a9d0d76d31c5144898a5a3d0e5abfab795f8a1efcafd8bba126`
- Timestamp: `2026-10-04T17:09:08+00:00`
- Observations: Bridge initialization and native IPC/events passed.
- Evidence refs: none
- Limitations: Does not approve external OAuth.

## EV-059 — passed

- Ticket: `TK-006`
- Acceptance: `AC-006`
- Procedure: `WebDriver auth-cancel.e2e with AURA_E2E_LOCAL_AUTH=1 on the final e2e,tauri/custom-protocol release SHA256 846E10EB2B1CBAED70F38F370D0042E4B0C4C183F7BC5D696EFCA2A90619D89C (named aura-auth-qa.exe), fresh profile. Final regression: cargo test --workspace --exclude aura-desktop (45 suites), clippy -D warnings (workspace and aura-desktop with e2e), fmt, ipc_contract golden, UI 140/140, typecheck.`
- Execution: `executed`
- Environment: Windows11 WebView2/driver 154.0.4258.53 production custom-protocol SHA256 E2693A146AADB157DD0BDC26B482AC137BEC80CF4EACDD9488ED4F7C33BC99A3; fresh isolated profiles; DPI 100%
- Tested revision: `local:c24950fa6a774f8fb5d0a615dc81a65b1b9b7fee7685e79f4468ed137aa19709`
- Timestamp: `2026-10-04T17:09:08+00:00`
- Observations: Two cancellations PT/EN: states waitingBrowser, cancelled, waitingBrowser, cancelled; no technical failure text; no active account.
- Evidence refs: none
- Limitations: Loopback authorize page only; complete external OAuth not approved.

## EV-060 — passed

- Ticket: `TK-007`
- Acceptance: `AC-007`
- Procedure: `WebDriver microphone.e2e on the final release (saved device preference, explicit > saved > default). Final regression: cargo test --workspace --exclude aura-desktop (45 suites), clippy -D warnings (workspace and aura-desktop with e2e), fmt, ipc_contract golden, UI 140/140, typecheck.`
- Execution: `executed`
- Environment: Windows11 WebView2/driver 154.0.4258.53 production custom-protocol SHA256 E2693A146AADB157DD0BDC26B482AC137BEC80CF4EACDD9488ED4F7C33BC99A3; fresh isolated profiles; DPI 100%
- Tested revision: `local:4689b56ad56deb008237b3b6595160d2bb4a22a7db29f951aa68725f3daa0798`
- Timestamp: `2026-10-04T17:09:09+00:00`
- Observations: 1/1 passing.
- Evidence refs: none
- Limitations: Device list of this machine (VB-Cable available).

## EV-061 — passed

- Ticket: `TK-008`
- Acceptance: `AC-008`
- Procedure: `WebDriver voice-repeat.e2e on the final release. Final regression: cargo test --workspace --exclude aura-desktop (45 suites), clippy -D warnings (workspace and aura-desktop with e2e), fmt, ipc_contract golden, UI 140/140, typecheck.`
- Execution: `executed`
- Environment: Windows11 WebView2/driver 154.0.4258.53 production custom-protocol SHA256 E2693A146AADB157DD0BDC26B482AC137BEC80CF4EACDD9488ED4F7C33BC99A3; fresh isolated profiles; DPI 100%
- Tested revision: `local:5387d6c9157ff63955021708371f4c4566d3217db10af6da9757d312f3bd0d95`
- Timestamp: `2026-10-04T17:09:09+00:00`
- Observations: Equal phrases of different cycles accepted, terminal duplicate ignored (1/1).
- Evidence refs: none
- Limitations: Synthetic voice events in the real WebView; not an acoustic recognition test.

## EV-062 — passed

- Ticket: `TK-009`
- Acceptance: `AC-009`
- Procedure: `WebDriver asr-language.e2e on the final release. Final regression: cargo test --workspace --exclude aura-desktop (45 suites), clippy -D warnings (workspace and aura-desktop with e2e), fmt, ipc_contract golden, UI 140/140, typecheck.`
- Execution: `executed`
- Environment: Windows11 WebView2/driver 154.0.4258.53 production custom-protocol SHA256 E2693A146AADB157DD0BDC26B482AC137BEC80CF4EACDD9488ED4F7C33BC99A3; fresh isolated profiles; DPI 100%
- Tested revision: `local:26d053b0cba30bc1a7d11ea47d1bdab35e8127d14ba7e943b9933eb36d3e2647`
- Timestamp: `2026-10-04T17:09:09+00:00`
- Observations: null clears the ASR language, absent patch keeps it, UI does not derive it from the interface language (1/1).
- Evidence refs: none
- Limitations: No acoustic or physical shortcut approval.

## EV-063 — passed

- Ticket: `TK-010`
- Acceptance: `AC-010`
- Procedure: `WebDriver voice-toggle.e2e on the final release (click/Enter/Space toggle, focus loss, pending guard, Escape on partial). Final regression: cargo test --workspace --exclude aura-desktop (45 suites), clippy -D warnings (workspace and aura-desktop with e2e), fmt, ipc_contract golden, UI 140/140, typecheck.`
- Execution: `executed`
- Environment: Windows11 WebView2/driver 154.0.4258.53 production custom-protocol SHA256 E2693A146AADB157DD0BDC26B482AC137BEC80CF4EACDD9488ED4F7C33BC99A3; fresh isolated profiles; DPI 100%
- Tested revision: `local:1aea69442cf58bba67b6a978c8f9f1dd09c43b0e73ed515c4f7168e91bd79197`
- Timestamp: `2026-10-04T17:09:09+00:00`
- Observations: 1/1 passing.
- Evidence refs: none
- Limitations: Keyboard/mouse through WebDriver; ASR result path covered by TK-011.
