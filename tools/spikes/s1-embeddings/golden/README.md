# Conjunto dourado do S1 (esforço 046)

Fixture sintética para avaliar a busca do Acervo em português. **Nenhum dado real**: pessoas, empresas, valores e eventos são inventados.

- `corpus.json`: 240 documentos em seis fontes que imitam o que o Aura guarda: 60 trechos de reunião, 50 turnos de conversa com o agente, 35 notas, 40 páginas lidas, 25 trechos de anexos e 30 textos de tela. Datas de 2026-08-01 a 2026-11-12. São cerca de 40 grupos de documentos parecidos (mesmo tema ou pessoa, com detalhes diferentes), 12 pessoas recorrentes e termos exatos (códigos, pedidos, siglas) em cerca de um terço dos documentos.
- `queries.json`: 100 perguntas com resposta única (`doc`), em três estilos: 30 por termo exato, 50 por paráfrase e 20 livres.

## Autoria e viés

- O corpus foi escrito por um subagente (Claude); as perguntas, por **outro** subagente, que só viu o corpus e nenhum resultado de busca. O dono do produto delegou a escrita das perguntas ao agente em 13/11/2026.
- Viés declarado: documentos e perguntas vêm da mesma família de modelo. Texto escrito por modelo tende a ser mais limpo e regular que fala e telas reais, e as perguntas por paráfrase foram pedidas de propósito, o que desfavorece a busca lexical. O conjunto mede **ordem relativa** entre modelos e fusões, não a qualidade absoluta no uso real.

## Uso

```text
cargo run --release --manifest-path tools/spikes/s1-embeddings/Cargo.toml -- <modelo> <dir_modelos> tools/spikes/s1-embeddings/golden/corpus.json tools/spikes/s1-embeddings/golden/queries.json <saida.json>
```

Licença: conteúdo sintético criado para este repositório, sob a mesma licença do projeto.
