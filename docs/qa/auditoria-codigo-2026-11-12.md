# Auditoria de código: captura, memória e busca

Data: 2026-11-12. Fonte: commit `bd972c7`. Nenhum código foi alterado.

**Escopo.** Lidos por inteiro: `aura-capture` (segmentos, retenção, codificador), `aura-store`
(núcleo e migrações), `aura-app` (`recorder.rs`, `notes.rs`, `memories.rs`, busca de
`meeting.rs`), a lista de ferramentas em `aura-mcp` e a documentação de projeto. **Não** foram
revisados linha a linha: gateway, autenticação, UI React, shell Tauri e `aura-win`. A suíte de
testes não foi executada nesta sessão. O foco foi o que sustenta (ou impede) o segundo cérebro.

**Como cada achado foi verificado.** *Executado*: rodei o código do repositório e observei o
resultado. *Leitura*: conclusão tirada do código, sem execução.

## Resumo

| ID | Gravidade | Achado | Verificação |
| --- | --- | --- | --- |
| A-01 | Alta | Modo Contínuo nunca apaga nada: a Retenção (dias e GB) não se aplica | Executado |
| A-02 | Alta | O gravador de tela pode morrer em silêncio e o app continua dizendo que grava | Leitura |
| A-03 | Média | Mudar o tamanho do Buffer recente não vale para o áudio até reiniciar, e o valor antigo apaga Segmentos de tela | Leitura |
| A-04 | Média | Cada Segmento fechado refaz a retenção inteira: custo cresce com o acervo e bloqueia o runtime | Leitura |
| A-05 | Média | Busca de notas e de reuniões carrega a tabela toda; não existe FTS5 no código | Leitura (confirmada por `grep`) |
| A-06 | Média | `audio_recent` retranscreve o áudio a cada chamada e carrega tudo na memória | Leitura |
| A-07 | Média | O gravador de tela fica preso ao monitor em que começou | Leitura |
| A-08 | Baixa | Tela a 1 fps sem detecção de mudança, com um quadro-chave a cada 10 quadros | Leitura |
| A-09 | Baixa | Áudio sem perdas (PCM + zstd) é caro para gravação longa | Leitura |
| A-10 | Média | Não há busca entre Conversas; o histórico fica em JSONL sem cifra | Leitura |
| A-11 | Processo | 17 de 44 esforços têm `change.md` com os campos do modelo por preencher | Executado (`grep`) |
| A-12 | Processo | `architecture/overview.md` descreve escolhas que os ADRs e o HANDOFF já mudaram | Leitura |
| A-13 | Manutenção | `host.rs` tem 4.195 linhas | Executado (`wc`) |

## A-01 — Modo Contínuo ignora a Retenção (Alta, executado)

`crates/aura-app/src/recorder.rs:45` grava os Segmentos do modo Contínuo com `kind =
"continuous"`. `crates/aura-capture/src/retention.rs:45` e `:50` só tratam `"buffer"` e
`"recording"`. Um Segmento `"continuous"` não entra em nenhum dos dois filtros e nunca é
candidato a exclusão.

Sonda feita fora do repositório, chamando `aura_capture::retention::plan` com 30 dias de
Segmentos de 3 GB (90 GB) e a política padrão (7 dias, 20 GB):

```
kind=recording:  deleted 24
kind=continuous: deleted 0
```

Efeito: com uma Fonte em Contínuo o disco cresce sem limite, e os controles "dias" e "GB
máximos" das Configurações não fazem nada.

Por que os testes não pegaram: `continuous_keeps_seven_days_and_twenty_gigabytes` monta
Segmentos `kind = "recording"` com `manual = false`. Essa combinação não existe em produção:
toda linha de `recordings` nasce com `manual = 1` (`recorder.rs:367`), e o modo Contínuo usa
outro `kind`. O teste cobre um caso que o app nunca produz.

Correção sugerida: red com um Segmento `"continuous"` além do limite; depois `plan` trata
`"continuous"` pelas regras de idade e tamanho. Conferir no mesmo ticket os outros filtros por
`kind` (`recorder.rs:424`).

Este é pré-requisito do segundo cérebro: o modo que mais grava é o único sem teto.

## A-02 — Gravador de tela morre em silêncio (Alta, leitura)

Em `crates/aura-capture/src/segments.rs`, o laço do `SegmentRecorder` usa `?` em
`encoder.push`, `encoder.flush` e `store_seg` (que grava arquivo, insere no SQLite e roda a
retenção). Qualquer erro encerra a tarefa. Ninguém observa o `JoinHandle`: o único `await` está
em `stop()`.

Enquanto isso, `CaptureService` mantém `screen = Some(ScreenRun)`. `active()` continua
devolvendo `"screen"`, e `apply()` compara só `kind` e `recording_id`, conclui que nada mudou e
não reinicia. A pessoa vê o indicador de gravação e nada está sendo gravado, até mudar a
Política ou reiniciar o app.

Correção sugerida: erro por Segmento vira aviso e o laço segue; a tarefa encerrada é detectada
(`is_finished`) e vira `HostEvent` com reinício e recuo exponencial. O indicador deve refletir o
estado real.

## A-03 — Retenção do áudio fica velha e apaga tela (Média, leitura)

`start_audio` copia a `RetentionPolicy` no momento em que o áudio começa
(`recorder.rs:296`) e a entrega ao `AudioSegmentWriter`. O gravador de tela usa o
`Arc<Mutex<RetentionPolicy>>` compartilhado. `apply()` não reinicia o áudio quando só os
minutos do Buffer recente mudam.

Os dois `SegmentStore` (tela e áudio) leem a mesma tabela `capture_segments`, e
`apply_retention` planeja sobre **todas** as Fontes (`self.list(None)`). Resultado: se o usuário
aumenta o Buffer recente de 5 para 30 minutos, o áudio continua com 5 e, a cada Segmento de
áudio fechado, apaga também os Segmentos de **tela** com mais de 5 minutos. O aumento não tem
efeito até reiniciar.

Correção sugerida: o áudio recebe o mesmo `Arc` compartilhado; `apply_retention` filtra pela
Fonte do próprio store.

## A-04 — Retenção refeita a cada Segmento (Média, leitura)

A cada 10 s por Fonte, `apply_retention` faz: duas leituras completas de `capture_segments`
com `JOIN`, um `read_dir` de todas as pastas, um `SELECT id, path` completo e um
`Path::exists` por linha, tudo sob o mutex `publication` e o mutex único da conexão SQLite.
`delete` faz duas consultas por id. `range` lê a Fonte inteira e filtra em Rust, sem usar o
índice `(source, end_ms)`.

Em Contínuo, sete dias de Segmentos de 10 s são ~60 mil linhas por Fonte. Com três Fontes
gravando, isso é uma varredura completa a cada ~3 s. Além disso, o gravador de tela é uma tarefa
`tokio::spawn` que faz captura GDI (49 ms medidos no HANDOFF), codificação, E/S de arquivo e
SQLite direto nas threads do runtime, sem `spawn_blocking`.

Correção sugerida: retenção por consulta (`DELETE … WHERE end_ms < ?`), varredura de órfãos só
na inicialização e de hora em hora, `range` com `WHERE`, gravador em thread própria.

## A-05 — Busca sem índice (Média, leitura)

`NotesRepo::search` (`notes.rs:103`) e `MeetingRepo::search` (`meeting.rs:352`) carregam todas
as linhas e filtram em Rust com `fold`, que só conhece uma lista fixa de acentos. A ordem é só
cronológica; não há relevância. `search_project` roda uma busca por reunião.

O roadmap (CAND-035: "biblioteca com FTS5") e o radar ("SQLite + FTS5": Adotar) dão o FTS5 como
escolhido, mas não há nenhuma tabela FTS5 nas 16 migrações nem no código.

Correção sugerida: tabelas FTS5 com `unicode61 remove_diacritics 2` e ordenação por `bm25`. É o
primeiro tijolo do Acervo (plano §5).

## A-06 — `audio_recent` retranscreve tudo (Média, leitura)

`audio_transcript` (`recorder.rs:711`) lê o intervalo pedido inteiro, descomprime para `f32` e
transcreve de uma vez, a cada chamada. Trinta minutos a 16 kHz são ~115 MB por Fonte na
memória, e a mesma fala é transcrita de novo na pergunta seguinte. A Reunião já faz diferente:
transcreve em passos com cursor e guarda o texto.

Correção sugerida: transcrever uma vez por Segmento, guardar a Transcrição e servir o texto.

## A-07 — Gravador preso a um monitor (Média, leitura)

`CaptureService::target()` escolhe o monitor do app em primeiro plano no instante em que
`apply()` roda, e o `SegmentRecorder` guarda esse alvo até ser reiniciado. Com dois monitores, o
trabalho feito no outro não é gravado.

## A-08 e A-09 — Custo de disco do desenho atual (Baixa, leitura)

- Tela: 1 quadro por segundo fixo, sem comparar com o anterior, em Segmentos de 10 quadros
  (`FRAMES_PER_SEGMENT`). Cada Segmento é independente, então um em cada dez quadros é
  quadro-chave completo, mesmo com a tela parada.
- Áudio: PCM de 16 bits + zstd (desvio 8 do HANDOFF; o ADR 0004 previa Opus). São ~115 MB por
  hora e por Fonte antes do zstd.

Para o Buffer recente isso é aceitável. Para memória de longo prazo, não: o plano troca vídeo
contínuo por Momentos de tela e guarda texto no lugar de áudio.

## A-10 — Conversas sem busca e sem cifra (Média, leitura)

O histórico vem de `thread/list` do Codex (`host.rs:2133`); não há busca pelo conteúdo entre
Conversas. Os rollouts são JSONL sem cifra (limitação já registrada em
`architecture/overview.md`). A Memória do Codex vem desligada por padrão
(`aura-codex/src/home.rs`, `memories: false`). Hoje o agente não tem como lembrar de uma
Conversa anterior.

## A-11 e A-12 — Documentos de processo defasados

- `grep` por `[exact command]` encontra 17 de 44 `change.md` com o modelo por preencher
  ("Leitura e mapa de alterações", "Plano breve", "Resultado executado: [pending]"). O
  `backlog.md` lista os esforços 012–035 em `discovery`, enquanto o HANDOFF e o CHANGELOG dizem
  que foram implementados. O AGENTS.md exige evidência registrada; ela não está lá.
- `architecture/overview.md` ainda fala em captura por WGC, PDF e vídeo no worker e
  `cargo nextest`. Os ADRs 0008 e o HANDOFF (desvios 1 e 9) dizem GDI e PDF no host.

Sugestão: uma passada de `evidence add`/`ticket update` nos esforços entregues e uma revisão do
`overview.md` antes de abrir os esforços do segundo cérebro, que vão depender dele.

## A-13 — `host.rs` com 4.195 linhas

O arquivo concentra um método por comando de todas as áreas. As funções de memória somariam
dezenas de comandos. Vale extrair por domínio (captura, reunião, extensões) antes, mantendo o
contrato IPC dourado como rede de segurança.

## Pendências já conhecidas que continuam valendo

Do HANDOFF, sem mudança: CI, Performance e Release desativados no Actions; DPI 150%/200% e
Windows limpo não aprovados; UIA/OCR, atalhos e "Inserir no app" sem validação por pessoa. A
validação de UIA/OCR passa a ser crítica: é a base dos Momentos de tela.
