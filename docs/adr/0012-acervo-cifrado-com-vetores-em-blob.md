---
status: accepted
---

# Acervo em banco próprio cifrado, com vetores em BLOB e busca no worker

O Acervo do Segundo Cérebro fica num banco próprio, `memory.db`, separado do `aura.db` e cifrado por inteiro com SQLCipher (`rusqlite` com `bundled-sqlcipher-vendored-openssl`).

- **Chave:** chave crua de 256 bits derivada pelo cofre existente (HKDF a partir da DEK protegida por DPAPI), passada como `PRAGMA key = "x'…'"`, sem senha nem PBKDF2.
- **Texto:** as Lembranças são indexadas em FTS5 com `unicode61 remove_diacritics 2`.
- **Vetores:** sem extensão. Ficam em tabelas comuns, em ordem de inserção (o `rowid` acompanha o tempo), em duas formas por trecho:
  - binária, 1 bit por dimensão (48 bytes para 384 dimensões);
  - int8 (384 bytes).

  Float32 não é guardado.
- **Busca:**
  1. o filtro de período limita a leitura a uma faixa de `rowid`, e a data e a fonte de cada linha são conferidas;
  2. o binário é varrido em Rust (POPCNT) para achar **200 candidatos**;
  3. esses candidatos são reordenados pelo int8 e saem os k melhores.
- **Processos:** o host é o único escritor (texto, FTS5 e vetores) e responde à parte lexical. A varredura vetorial roda no `aura-worker`, que abre o banco só para leitura, com cache de páginas dimensionado para o índice binário (N × 72 bytes + 4 MB, ~74 MB por milhão de trechos).

Evidência: spike S2 do esforço 046 (`specs/046-spikes-segundo-cerebro/s2.md`), com o mesmo `rusqlite`/SQLCipher, 100 mil vetores reais do bekko-a8m (ADR 0014) em português e bases de 500 mil e 1 milhão derivadas para tempo. Medições anteriores: `docs/research/segundo-cerebro-medicoes-2026-11.md` §3.

| Caso (cache dimensionado, 200 candidatos) | Desktop p50 / p95 | Aproximação modesta p50 / p95 |
| --- | ---: | ---: |
| 500 mil, base inteira | 37,6 / 38,9 ms | 58,1 / 58,8 ms |
| 500 mil, 30 dias | 6,8 / 7,0 ms | 11,6 / 12,1 ms |
| 1 milhão, base inteira | 66,8 / 69,9 ms | 102,9 / 103,8 ms |
| 1 milhão, 30 dias | 9,4 / 9,9 ms | 16,1 / 16,7 ms |

- **Recall@10 contra a busca exata,** nos 100 mil vetores reais: 0,84 com 100 candidatos; 0,90 com 200; 0,93 com 400.
- **Cache padrão de 2 MB:** a base inteira passa de 300 ms a partir de 500 mil (344 ms na aproximação; 660 ms com 1 milhão).
- **Abertura:** com senha, 126–205 ms; com chave crua, 0,3–0,5 ms.
- **Aproximação modesta:** 1 núcleo e processador limitado a 50%, ~1,6× mais lenta que o desktop. É um piso otimista para um notebook real.

## Considered options

- **sqlite-vec (`vec0`)**, a proposta original do plano: compila e funciona com SQLCipher no MSVC, mas tem três problemas. O crate compila o C sem SIMD, porque o caminho AVX usa sintaxe de GCC/Clang e não compila no `cl.exe`. O filtro por metadado continua lendo todos os blocos. E o projeto é pré-1.0 sem commits desde maio de 2026.
- **Float32 exato** (vec0 ou BLOB): 450 ms com 500 mil sem cifra e 4,6 s com cifra. Fora da meta.
- **Índice fora do SQLCipher** (LanceDB, USearch, arquivo próprio): segundo formato de armazenamento, fora da cifra. Um índice de tudo o que a pessoa viu não pode ficar em claro.
- **Busca no host com o cache padrão:** fora da meta a partir de 500 mil. Com cache grande, estoura o orçamento de +40 MB do host.
- **100 candidatos** (proposta original): perde 16% dos 10 vizinhos exatos com o modelo escolhido, contra 10% com 200, por 5 ms a menos.
- **Senha com PBKDF2:** +126–205 ms por abertura, sem ganho, porque a chave do cofre já é aleatória.

## Consequences

- O build do Windows e o CI passam a exigir **Perl nativo** (Strawberry Perl) ou um OpenSSL pré-compilado com `OPENSSL_DIR`. O Perl do Git for Windows não serve. O primeiro build a frio leva ~5 min a mais.
- O `libsqlite3-sys` é um só por binário. Com o `rusqlite` do workspace em SQLCipher, o `aura.db` também passa a usar o SQLite do SQLCipher (3.51.3 no lugar de 3.53.2), sem chave e com o mesmo formato. O FTS5 com `remove_diacritics 2` funciona nesse build (teste do S2).
- Disco: 469 bytes por trecho em vetores (469 MB por milhão), mais o texto e o FTS5.
- A primeira consulta numa conexão nova decifra o índice binário: 46 ms com 100 mil, 410 ms com 1 milhão no desktop.
  - Na Fase 1, dezenas de milhares de Lembranças, o worker sob demanda do ADR 0005 basta.
  - Com Momentos de tela (ADR 0013), a primeira busca depois de ociosidade passa de 300 ms. O esforço dos Momentos decide entre aquecer o worker quando o Overlay abre e mantê-lo vivo durante a Conversa.
- O código de varredura e de quantização é nosso (algumas centenas de linhas), com testes de equivalência contra a busca exata.
- "Esquecer" apaga linhas comuns; não há índice aproximado para reconstruir.
- O `memory.db` cifra o índice, não os originais: Notas e Reuniões continuam em claro no `aura.db` (CAND-086).
