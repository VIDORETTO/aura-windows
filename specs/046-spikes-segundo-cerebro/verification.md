# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — passed

- Ticket: `TK-004`
- Acceptance: `AC-009`, `AC-010`
- Procedure: `cargo test --release e cargo build --release em tools/spikes/s3-momentos; s3-momentos.exe rodar 0.0167 (1 min de uso real)`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D; Chrome 157.0.8103.51
- Tested revision: `local:49a4b8fc911abc43d352bc66f3be6e6ebadbea84680edf2a7feac51112b481e7`
- Timestamp: `2026-11-14T01:00:08+00:00`
- Observations: 3 testes de unidade (resumo sem valores de texto); 1 min de uso real: 0,22 s de CPU, 57 MB de pico, resumo só com números. Firefox passou pela UIA e a regra foi corrigida para OCR.
- Evidence refs: `C:/Users/gabri/AppData/Local/aura-spikes/s3`
- Limitations: none recorded

## EV-002 — passed

- Ticket: `TK-004`
- Acceptance: `AC-011`
- Procedure: `python -I tools/spikes/s3-momentos/chrome_check.py <s3-momentos.exe> <saida.json> (perfil temporário, example.com)`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; Ryzen 7 7800X3D; Chrome 157.0.8103.51
- Tested revision: `local:ffb7d36a282514f4109a5aeca6b8fc6b106694b195a3c2127d6b976899216ccd`
- Timestamp: `2026-11-14T01:00:09+00:00`
- Observations: Ler só o nome UIA da raiz não ligou Web accessibility; uma leitura completa ligou Web accessibility, Text metrics e Extended properties, e o modo continuou ligado 10 min depois do cliente encerrar.
- Evidence refs: `C:/Users/gabri/AppData/Local/aura-spikes/s3/chrome-check.json`
- Limitations: none recorded
