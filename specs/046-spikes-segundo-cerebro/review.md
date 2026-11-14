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
