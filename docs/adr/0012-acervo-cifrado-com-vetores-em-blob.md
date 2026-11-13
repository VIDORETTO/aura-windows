---
status: proposed
---

# Acervo em banco próprio cifrado, com vetores em BLOB e busca no worker

O Acervo do Segundo Cérebro fica num banco próprio, `memory.db`, separado do `aura.db` e cifrado por inteiro com SQLCipher (`rusqlite` com `bundled-sqlcipher-vendored-openssl`; chave guardada pelo cofre existente, DPAPI). O texto das Lembranças é indexado em FTS5 com `unicode61 remove_diacritics 2`. Os vetores **não** usam extensão: ficam numa tabela comum, em ordem de inserção (o `rowid` acompanha o tempo), em duas formas por trecho — binária (1 bit por dimensão: 48 bytes para 384 dimensões) e int8 (384 bytes). Float32 não é guardado. A busca filtra o período primeiro, como faixa de `rowid`; varre o binário em Rust (POPCNT) para achar 100 candidatos; reordena esses candidatos pelo int8; e devolve os k melhores. A busca vetorial roda no `aura-worker`, com cache de páginas do SQLite dimensionado para o índice binário (~80 MB por milhão de trechos), não no host.

Evidência: `docs/research/segundo-cerebro-medicoes-2026-11.md` §3 (Ryzen 7 7800X3D, build MSVC, vetores reais de 384 dimensões). Com cifra e o cache padrão de 2 MB, toda varredura completa fica 5–10× mais lenta (o binário do sqlite-vec leva 331 ms a ~1 milhão). Com a forma proposta, binário → reordenação dos últimos 30 dias leva 31 ms a 500 mil e 47 ms a ~1 milhão; binário → reordenação sem filtro, com cache de 120 MB, leva 66 ms a ~1 milhão. Recall@10 de 0,86–0,90 contra a busca exata.

## Considered options

- **sqlite-vec (`vec0`)**, a proposta original do plano: compila e funciona com SQLCipher no MSVC, mas o crate compila o C sem SIMD (o caminho AVX usa sintaxe de GCC/Clang e não compila no `cl.exe`), o filtro por metadado continua lendo todos os blocos, e o projeto é pré-1.0 sem commits desde maio de 2026. Fica como opção se o S2 mostrar ganho, não como dependência.
- **Float32 exato** (vec0 ou BLOB): 450 ms a 500 mil sem cifra e 4,6 s com cifra. Fora da meta.
- **Índice fora do SQLCipher** (LanceDB, USearch, arquivo próprio): segundo formato de armazenamento, fora da cifra. Um índice de tudo o que a pessoa viu não pode ficar em claro.
- **Busca no host com cache grande**: estoura o orçamento de +40 MB do host.

## Consequences

- O build do Windows e o CI passam a exigir **Perl nativo** (Strawberry Perl) ou um OpenSSL pré-compilado com `OPENSSL_DIR`. O Perl do Git for Windows não serve. O primeiro build a frio leva ~5 min a mais.
- Disco: ~480 MB por milhão de trechos de vetores (binário + int8), mais o texto e o FTS5.
- O worker precisa estar vivo para a busca vetorial. Isso muda o "sob demanda" do ADR 0005 enquanto o Segundo Cérebro estiver ligado; o S2 decide entre worker residente e partida a frio por consulta.
- O código de varredura e de quantização é nosso (algumas centenas de linhas), com testes de equivalência contra a busca exata.
- "Esquecer" apaga linhas comuns; não há índice aproximado para reconstruir.
- Recall: com 100 candidatos, o binário perdeu ~10% dos 10 vizinhos exatos num modelo em inglês. O S2 mede com o modelo escolhido (ADR 0014) e ajusta o número de candidatos.
- Tudo foi medido num desktop rápido. O S2 oficial repete num notebook modesto antes de este ADR ser aceito.
