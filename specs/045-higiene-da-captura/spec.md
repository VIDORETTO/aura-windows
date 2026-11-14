---
schema: hybrid/spec
schema_version: 1.0
effort_id: 045-higiene-da-captura
revision: 1
status: accepted
profile: standard
---

# Higiene da captura

## Problem and desired result

A auditoria de 12/11/2026 (`docs/qa/auditoria-codigo-2026-11-12.md`) achou quatro defeitos na captura em segundo plano.

- **A-01:** no modo Contínuo, a Retenção nunca apaga nada. Os Segmentos `continuous` não entram em nenhuma regra, e o disco cresce sem limite.
- **A-02:** o gravador de tela pode morrer em silêncio enquanto o app continua dizendo que grava.
- **A-03:** uma mudança no tamanho do Buffer recente não chega ao áudio até reiniciar o app, e o valor antigo apaga Segmentos de tela.
- **A-04:** cada Segmento fechado refaz a retenção inteira (leituras completas da tabela, varredura de pastas e de arquivos), com custo que cresce com o acervo.

Resultado desejado: os limites de dias e de GB valem para o Contínuo, o indicador de gravação reflete o estado real e o gravador se recupera sozinho. Mudanças de política valem na hora para tela e áudio, e o custo da retenção por Segmento não depende do tamanho do acervo. É o plano do Segundo Cérebro, Fase 0 (CAND-083); o modo que mais grava precisa de teto antes da memória de longo prazo.

## Consumers and actors

Usuário que liga Buffer recente, Contínuo ou Gravação manual nas Configurações de privacidade. Consumidores técnicos: `aura-app::recorder::CaptureService`, `aura-capture::{retention, segments}`, `aura-audio::recorder::AudioSegmentWriter` e o laço periódico do host.

## Scope

### Included

- Regras de retenção para Segmentos `continuous` (A-01).
- Áudio usando a política de retenção viva, compartilhada com a tela (A-03).
- Erro de Segmento sem derrubar o gravador; gravador morto detectado, informado e reiniciado com recuo (A-02).
- Retenção por consulta após cada Segmento, varredura de órfãos só na inicialização e de hora em hora, e `range` com filtro no SQL (A-04).

### Excluded

- A-05 (FTS5), que é da Fase 1 (Acervo).
- A-06 (Transcrição guardada por Segmento) e A-07 (gravador preso a um monitor): ficam no CAND-083 para um esforço seguinte.
- Mover a captura e a codificação da tela para uma thread própria (parte da sugestão de A-04): fica registrado como pendência; esta entrega muda só o custo da retenção.
- A-08 e A-09 (formato de disco), mudanças de UI, de IPC, de esquema SQLite, de formato de Segmento e de identificadores.

## User journeys and scenarios

### US-001 — O Contínuo respeita os limites de dias e de GB (P1)

Como usuário com uma Fonte em Contínuo, quero que os limites "dias" e "GB máximos" apaguem o mais antigo, para o disco não crescer sem limite.

Demonstração independente: planejar a retenção sobre Segmentos `continuous` além dos limites e ver os mais antigos marcados para apagar.

#### Acceptance scenarios

- **AC-001** — Dados 10 Segmentos `continuous` de 1 dia e 2 GB cada (dias 0 a 9) e a política padrão (7 dias, 20 GB), em `now` = fim do dia 10: os dias 0, 1 e 2 são apagados (idade). Com 3 GB cada: os dias 0, 1, 2 e 3 são apagados (idade e tamanho, até ≤ 20 GB).
- **AC-002** — Segmentos `recording` com `manual = true` não são apagados por idade ou tamanho enquanto `apply_to_manual = false`; com `apply_to_manual = true`, seguem as mesmas regras. Segmentos `buffer` continuam seguindo só o Buffer recente.
- **AC-003** — `continuous` e gravações sujeitas a limite dividem um único teto de GB, somado sobre todas as Fontes (tela, microfone, sistema), apagando primeiro o de início mais antigo.

### US-002 — Mudar o Buffer recente vale na hora para o áudio (P1)

Como usuário, quero que aumentar o Buffer recente valha logo, sem apagar o que eu pedi para guardar.

Demonstração independente: um gravador de áudio criado com 5 minutos e uma política alterada para 30 minutos não apagam Segmentos dentro dos 30 minutos.

#### Acceptance scenarios

- **AC-004** — Dado um gravador de áudio iniciado com Buffer de 5 minutos e Segmentos de tela `buffer` de 0 a 20 minutos, quando a política compartilhada passa a 30 minutos e o áudio fecha um Segmento em 20 minutos, então nenhum Segmento de tela que termina depois de 20 − 30 minutos é apagado; com a política de volta a 5 minutos, os que terminam antes de 15 minutos são apagados.

### US-003 — O gravador não morre em silêncio (P1)

Como usuário, quero que o indicador de gravação diga a verdade e que o gravador volte sozinho.

Demonstração independente: provocar falhas de armazenamento no gravador de tela e observar que ele continua, ou, se parar, que deixa de ser informado como ativo e é reiniciado.

#### Acceptance scenarios

- **AC-005** — Quando a gravação de um Segmento falha uma vez, o erro é registrado como aviso e os Segmentos seguintes continuam sendo gravados.
- **AC-006** — Depois de 5 falhas seguidas, o gravador encerra. Daí em diante, `CaptureService::active()` não informa mais `screen`, e a supervisão periódica o reinicia com recuo exponencial (espera de 15 s, 30 s, 60 s… até 15 min entre tentativas; volta a 15 s depois de um reinício que grava um Segmento). Cada mudança do estado real gera o evento de privacidade com a lista de Fontes ativas.

### US-004 — Retenção sem custo crescente (P2)

Como usuário de Contínuo, quero que fechar um Segmento não fique mais caro quanto mais eu gravo.

Demonstração independente: com dezenas de milhares de Segmentos, aplicar a retenção depois de um Segmento devolve as mesmas exclusões do planejamento puro, sem varrer pastas.

#### Acceptance scenarios

- **AC-007** — Para qualquer conjunto de Segmentos, a retenção aplicada pelo armazenamento apaga exatamente os ids que `retention::plan` devolve para o mesmo conjunto, política e instante.
- **AC-008** — A retenção após um Segmento não remove arquivos órfãos; a varredura de órfãos remove arquivos `.seg` sem linha, `.seg.tmp` e linhas sem arquivo, e roda ao iniciar o gravador e no máximo uma vez por hora.
- **AC-009** — `range(source, from, to)` devolve os mesmos Segmentos que antes, filtrados no SQL.

## Requirements

- **FR-001** — `retention::plan` trata `continuous` com as regras de idade e tamanho, junto com `recording` sujeito a limite (não manual, ou manual com `apply_to_manual`).
- **FR-002** — O áudio usa a mesma política de retenção viva da tela.
- **FR-003** — Erro por Segmento não encerra o gravador; 5 falhas seguidas encerram; o estado informado reflete o gravador vivo; a supervisão do host reinicia com recuo exponencial.
- **FR-004** — A retenção após cada Segmento consulta só candidatos à exclusão; a varredura de órfãos fica separada e com cadência limitada; `range` filtra no SQL.

## Limits, errors, and compatibility

- Sem mudança de esquema, de formato de arquivo, de IPC ou de eventos: o evento de privacidade existente transporta a lista de Fontes ativas.
- A soma de GB é global (todas as Fontes), como hoje; o Buffer recente usa o maior valor entre as Fontes, como hoje.
- Falha de captura de quadro (sem Segmento) continua sendo ignorada como hoje; contam como falha do gravador só os erros de codificação e de armazenamento.
- Ordem de exclusão estável: início mais antigo primeiro.

## Hypotheses and dependencies

- Hipótese confirmada por sonda da auditoria: `plan` ignora `continuous`.
- Os demais itens (A-02 a A-04) vêm de leitura; cada ticket confirma com um teste vermelho antes de corrigir.
- Sem dependência nova.

## Success criteria

### Delivery-verifiable

- **SC-001** — Testes de AC-001 a AC-009 passam; `cargo test --workspace --exclude aura-desktop`, `cargo clippy -D warnings` e `cargo fmt --check` passam no Windows.
- **SC-002** — Com 60.000 Segmentos `continuous`, a retenção após um Segmento leva menos que antes, medido no mesmo PC e registrado como evidência.

### Post-delivery observation

- **SC-003** — Num dia real de Contínuo, o uso de disco estabiliza no teto configurado.

## Decisions and open questions

- Teto de GB global e Buffer pelo maior valor: mantidos como hoje para não mudar o significado dos controles.
- Recuo da supervisão alinhado ao laço de 15 s do host: 15 s, 30 s, 60 s, … até 15 min.
- Nenhuma decisão material pendente.
