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

## EV-003 — passed

- Ticket: `TK-001`
- Acceptance: `AC-005`
- Procedure: `Validação em Python de golden/corpus.json e golden/queries.json: contagens, ids, unicidade, sobreposição lexical por estilo (palavras de 4+ letras)`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974
- Tested revision: `local:fd59c8eacb9e3d8abd3aa4538eab5b59791b202de7687964ef5d855fb2ba848a`
- Timestamp: `2026-11-14T01:09:38+00:00`
- Observations: 240 documentos (reuniao 60, conversa 50, pagina 40, nota 35, tela 30, anexo 25), ~40 grupos parecidos; 100 perguntas (exato 30, parafrase 50, livre 20), 100 documentos distintos, todos válidos; sobreposição média com o documento certo: exato 0,63, parafrase 0,32, livre 0,56. Autores distintos (subagentes); viés declarado no README.
- Evidence refs: none
- Limitations: none recorded

## EV-004 — passed

- Ticket: `TK-002`
- Acceptance: `AC-006`, `AC-007`, `AC-008`
- Procedure: `cargo test --release; s1-embeddings.exe <modelo> <dir_modelos> golden/corpus.json golden/queries.json <saida.json> para e5-small, granite-97m, bekko-a8m, bekko-a25m-q8 (3 rodadas nos empatados); paridade Rust x Python (onnxruntime 1.31.1)`
- Execution: `executed`
- Environment: Windows 11 Pro 25H2 build 26200.9974; Rust 1.98.1 MSVC; ort 2.0.0-rc.12; Ryzen 7 7800X3D
- Tested revision: `local:cd0b2d83f948fc32b8ee5b05d6cce530773b1faebc87c081e251becfcb82f879`
- Timestamp: `2026-11-14T01:15:10+00:00`
- Observations: MRR com RRF 1:1: bekko-a25m 0,897, e5 0,894, bekko-a8m 0,892, granite 0,838; só FTS5 0,867; só vetor 0,751-0,832. Empate (<0,02) desempatado por latência: bekko-a8m (16,1-17,0 ms/doc) sobre e5-small (17,1-17,5). Paridade cos 0,98-1,00. ADR 0014 aceito; I-19 revertida.
- Evidence refs: `C:/Users/gabri/AppData/Local/aura-spikes/s1`
- Limitations: none recorded
