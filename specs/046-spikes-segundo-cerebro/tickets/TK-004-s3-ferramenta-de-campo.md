---
schema: hybrid/ticket
schema_version: 1.0
id: TK-004
effort: 046-spikes-segundo-cerebro
type: spike
status: done
ticket_revision: 5
requires: []
requirement_refs: ["FR-001", "FR-002"]
acceptance_refs: ["AC-009", "AC-010", "AC-011"]
spec_revision: 1
plan_revision: 1
owned_areas: ["tools/spikes/s3-momentos", "specs/046-spikes-segundo-cerebro/s3-s4.md"]
verification_status: passed
last_update: package verificado depois da implementação (desvio registrado na revisão)
---





# TK-004 — S3/S4 — ferramenta de campo e verificações no Chrome

## Objetivo e limites

Construir a ferramenta que o usuário roda num dia real, com o desenho do ADR 0013 e só números na saída. Testá-la por alguns minutos e fazer as duas verificações curtas no Chrome.

Não inclui: o dia real (TK-005); código de produto.

## Leitura em ordem

1. `specs/046-spikes-segundo-cerebro/plan.md` → S3/S4.
2. `docs/adr/0013-momentos-de-tela-por-evento.md`.
3. `crates/aura-win/src/uia.rs` e `crates/aura-win/src/ocr.rs` (algoritmos a imitar, só leitura).

## Decisões já resolvidas

- Classe `Chrome_WidgetWin_*` → captura + OCR; o resto → UIA com orçamento de 200 ms e OCR de reserva.
- Saída só numérica; nenhum texto ou imagem gravado; limite de 10 h; parada por Ctrl+C, pelo fechamento da janela ou pelo arquivo `parar`.
- Verificações no Chrome com perfil temporário e página leve (nada de DOM pesado).

## Mapa de alterações

- Novo: projeto `tools/spikes/s3-momentos`, roteiro `specs/046-spikes-segundo-cerebro/s3-s4.md`.

## Contrato técnico

- `s3 rodar [--horas N]` → `%LOCALAPPDATA%\aura-spikes\s3\resumo-*.json`.
- `s3 chrome-raiz` e `s3 chrome-desliga` → resultado das verificações.

## Exemplos de aceite

- **AC-009**: o binário implementa o desenho; o JSON não tem nenhum campo de texto.
- **AC-010**: execução de alguns minutos pelo agente, com resumo válido.
- **AC-011**: (a) estado do modo depois de ler só a raiz; (b) tempo até desligar.

## Dependências e sequência de execução

Depende de: nenhum.

- [ ] TK-004.1 Testes de unidade da agregação (p50/p95, sem texto).
- [ ] TK-004.2 Binário; execução curta.
- [ ] TK-004.3 Verificações no Chrome.
- [ ] TK-004.4 Roteiro e evidência.

## Validação

- `cargo test --manifest-path tools/spikes/s3-momentos/Cargo.toml`; execuções registradas.

## Condição de retorno à planejadora

Pare e devolva se uma medição exigir mudar código de produto, se um modelo ou biblioteca não tiver licença permissiva, ou se o resultado contradisser um ADR de forma que exija decisão do dono do produto (registre a proposta).

## Relatório de saída

Arquivos, comandos, números, `EV`, limitações, decisão proposta e próxima ação.
