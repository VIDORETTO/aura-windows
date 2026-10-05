# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — passed

- Ticket: `—`
- Acceptance: `AC-001`, `AC-002`, `AC-003`
- Procedure: `Rust red/green: catalog (gpt-6.1-sol), plan list append/enrich, discovery of known models, manual model efforts validation, effortPresets settings; React red/green (picker per-mode memory, plan GPT-6.1 Sol efforts, manual model effort form, effort table); native red on the previous release and green: loopback custom provider discovers gpt-6.1-sol, manual qa-reasoner with Low/High/Max default High via Settings, effort table, picker Low in Chat and Max in Task, restart, Task turn upstream reasoning.effort=max; regressions model-effort, profile-model, provider-manual-model.`
- Execution: `executed`
- Environment: Windows11 WebView2 154 production SHA256 549FE0815780BB94C7C51F661A42D4328208B75F6114C571620B09CA5E82BE3F
- Tested revision: `local:6d43a46ae2abdbc6fb251a2d65e64e7f7493b0a1138f9167f155f766e3d2213c`
- Timestamp: `2026-10-04T21:14:28+00:00`
- Observations: Discovered GPT-6.1 Sol with 1050000 context, 128000 output, images/tools, efforts low..max default medium; manual model saved efforts [low,high,max] default high; preset {chat: low, task: max} persisted after restart; upstream reasoning.effort max.
- Evidence refs: none
- Limitations: No ChatGPT account in QA: GPT-6.1 Sol in the plan verified by the catalog function and the mock, not by a real plan turn; the plan server still decides access.
