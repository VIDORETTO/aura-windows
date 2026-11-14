# Review — 046-spikes-segundo-cerebro

Revisão local feita pelo agente implementador, ticket a ticket; não se atribui aprovação a outro revisor. Todo o código revisado é **protótipo** em `tools/spikes/`, fora do workspace do produto.

## TK-004 — Ferramenta de campo do S3/S4 e verificações no Chrome

Escopo: `tools/spikes/s3-momentos/{Cargo.toml, src/lib.rs, src/main.rs, chrome_check.py}` e `specs/046-spikes-segundo-cerebro/s3-s4.md`.

### Standards

- O projeto tem `[workspace]` próprio: o build da raiz não o inclui (FR-004). As dependências novas têm licença permissiva: `image-webp` (MIT/Apache-2.0) e `windows` 0.62, a mesma do Aura.
- Privacidade (FR-002):
  - o texto lido por UIA e OCR só é contado, e a imagem só é medida;
  - o resumo é testado contra valores de texto;
  - a saída fica fora do repositório.
- Desvio de processo: o `package` deste ticket foi verificado depois da implementação. Ele estava pronto (`ready: true`, sem bloqueios) e nada do contrato mudou, mas a ordem prevista no `AGENTS.md` não foi seguida. Registrado aqui, sem consequência no resultado.

### Spec

- **AC-009:** o desenho do ADR 0013 está implementado:
  - ganchos separados, assentamento de 1,5 s e batida por WGC;
  - UIA com orçamento de 200 ms e OCR de reserva;
  - navegadores e Firefox por OCR na resolução nativa;
  - WebP sem perdas só para medir o tamanho.
- **AC-010:** execução de 1 minuto com resumo válido. Ela achou um defeito (o Firefox ia para a UIA), corrigido.
- **AC-011:** (a) o nome da raiz não liga a árvore web; (b) uma leitura completa liga e mantém o modo por pelo menos 10 min.
- Limite: o desligamento depois de 10 min não foi observado, porque o teste parou no limite.

Disposição: TK-004 liberado para `verified` e `done`. O TK-005 (dia real) segue bloqueado, aguardando o usuário.

## TK-001 — Conjunto dourado

Escopo: `tools/spikes/s1-embeddings/golden/{corpus.json, queries.json, README.md}`.

### Standards

Fixture sintética sem dado real, com README de origem, autoria e viés. Entra no git porque vai virar o conjunto permanente do `aura-bench` (ADR 0014); a regra "não commitar conjuntos de dados" das medições valia para dados baixados de terceiros. Nenhum achado bloqueante.

### Spec

**AC-005:** 240 documentos em 6 fontes e 100 perguntas com resposta única, de autores distintos; as perguntas não viram resultados. A sobreposição lexical por estilo foi medida (paráfrase 0,32) e o viés está declarado.

Disposição: TK-001 liberado para `verified` e `done`.

## TK-002 — S1 em Rust

Escopo: `tools/spikes/s1-embeddings/src/{lib.rs, main.rs}`, `specs/046-spikes-segundo-cerebro/s1.md`, ADR 0014 e a revisão 3 do plano (§5.3).

### Standards

- Mesmo `ort` do lockfile do Aura; `tokenizers` e `rusqlite` com licença permissiva.
- Funções puras testadas com literais calculados à parte: fusão, métricas, critério, termo raro.
- A paridade com o Python foi verificada antes de concluir, porque o resultado do granite contrariava a evidência anterior.
- Nenhum achado bloqueante.

### Spec

- **AC-006:** 4 modelos com qualidade, latência, RAM e carga.
- **AC-007:** 3 fusões contra o vetor puro, por estilo.
- **AC-008:** recomendação pelo critério definido antes de medir, com desempate e reserva registrados.
- A reversão da I-19 está documentada no plano (revisão 3) e no documento de medições, sem apagar o registro anterior.

Disposição: TK-002 liberado para `verified` e `done`. ADR 0014 aceito.
