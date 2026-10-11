# Avaliação backend de qualidade — TK-006

## Gate atual — plan r10

Execução em 10/10/2026 às 20:38:56 UTC, mesma sessão até saída terminal: **10/12 fontes relevantes confirmadas, 8/10 alvos extraídos, 58 HTTP do Aura em 193 s, exit 0/phase complete**. A relevância do Museu foi confirmada pelo corpo oficial às 20:44:59 UTC, depois das extrações e ainda dentro dos 30 minutos do experimento. Essa inspeção posterior apenas classifica a busca; o Museu não faz parte dos dez alvos de extração. Nenhuma consulta, alvo, literal ou falha foi substituída. IBGE permanece não verificado; Tauri CLI permanece fora do tema invoke. Tauri unsupported_content e SearXNG sem o literal continuam no denominador da extração.

Procedimento executado: `cargo run -p aura-web --example quality -- 'C:/Users/gabri/AppData/Local/Temp/aura-039-quality-plan10-1791664733872/report.json' 'C:/Users/gabri/AppData/Local/Temp/aura-039-quality-plan10-1791664733872/targets.json'`. Os dez HTML foram inspecionados e seus literais fixados antes das leituras; os metadados e a classificação estão em [quality-results-2026-10-10-r10.json](quality-results-2026-10-10-r10.json). Relatórios anteriores preservados. As requisições da inspeção independente não entram no contador do Aura. Sem conta, chave de busca ou inferência comercial. Este gate aprova somente o backend; síntese, E2E nativo e DPI continuam pendentes.

10/10/2026, Windows, spec r1 / plan r7, baseline `ed9a9654c6e76e48186fc010a12eba77023b00f5`. Harness `crates/aura-web/examples/quality.rs`, WebService e ReqwestTransport reais, sem chave/conta de busca. Uma sessão externa estável, consultas/metas de tdd.md mantidas. Avaliação concluída: **11/12 relevantes, 8/10 alvos de extração, 58 tentativas HTTP do Aura, 239 s incluindo preparação dos alvos**. Registro de metadados sem snippets/corpos: [quality-results-2026-10-10.json](quality-results-2026-10-10.json). Esse resultado aprova somente o gate backend, não o esforço inteiro.

Procedimento: `cargo run -p aura-web --example quality -- C:\Users\gabri\AppData\Local\Temp\aura-039-quality-20261010-r7-report.json C:\Users\gabri\AppData\Local\Temp\aura-039-quality-20261010-r7-targets.json`. JSON temporário contém resultados e metadados, sem corpos de páginas. 12 buscas completas em 33 s/48 tentativas HTTP pelo transporte do Aura; origem Parallel em todas, sem fallback. Clippy do example passou antes da execução.

## Relevância nos primeiros cinco

Domínio alvo e tema foram fixados em tdd.md antes da busca. Classificação pelo título/conteúdo da fonte; URL do domínio certo com tema errado não conta. Meta >=10/12, resultado observado 11/12; nenhuma consulta repetida/substituída.

| Caso | Consulta fixa | Fonte relevante/posição | Resultado |
| --- | --- | --- | --- |
| 1 | Rust ownership official book | [Understanding Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html), 5 | relevante, capítulo atual |
| 2 | Tauri v2 invoke commands | CLI nas posições 1/2, release CLI antiga na 5; código upstream na 3 está em GitHub, fora do domínio alvo | falha; CLI não explica invoke e não contém esse termo |
| 3 | React useEffect documentation | [useEffect](https://react.dev/reference/react/useEffect), 1 | referência do hook |
| 4 | Microsoft WebView2 introduction | [Introduction to Microsoft Edge WebView2](https://learn.microsoft.com/en-us/microsoft-edge/webview2/), 2 | visão geral do componente |
| 5 | Python pathlib documentation | [pathlib](https://docs.python.org/3/library/pathlib.html), 3 | referência oficial; versões antigas também aparecem |
| 6 | SQLite foreign keys documentation | [Foreign Key Support](https://sqlite.org/foreignkeys.html), 5; mesmo documento com // na 1 | suporte e ativação de foreign keys |
| 7 | IBGE Censo 2022 resultados | [primeiros resultados oficiais, PDF](https://www.ibge.gov.br/biblioteca/visualizacao/livros/liv102011.pdf), 2 | relevante para busca; leitura PDF está fora do escopo |
| 8 | INPE programa queimadas dados | [Dados Abertos](https://data.inpe.br/queimadas/portal/dados-abertos/index.html), 1 | portal oficial/dados CSV e KML |
| 9 | Museu do Amanhã horário visita | [Horários e Ingressos](https://museudoamanha.org.br/visite/horarios-e-ingressos), 2 | informação oficial de visita; sem provar extração do Aura |
| 10 | W3C WCAG 2.2 recommendation | [WCAG 2.2](https://www.w3.org/TR/WCAG22/), 1 | recomendação; rascunhos antigos também aparecem |
| 11 | OpenAI web search tools documentation | [Web search](https://developers.openai.com/api/docs/guides/tools-web-search), 2 | guia oficial; .md na posição 1 não selecionado como HTML |
| 12 | SearXNG search API JSON | [Search API](https://docs.searxng.org/dev/search_api.html), 4; mesma rota com // na 1 | API/formato; fonte .rst.txt não selecionada como HTML |

## Alvos fixados antes de extrair

Inspeção independente das fontes oficiais pelo browser/web tool, antes de entregar o manifesto ao processo de avaliação. Estes dez HTML e seus literais são fixos; qualquer falha de leitura estática permanece no denominador e não será substituída depois. Os literais são títulos/seções ou trechos curtos, não esperados calculados pelo extractor. CLI é válida para avaliar extração de HTML, embora tenha falhado a relevância da consulta invoke. IBGE não foi escolhido para extração: PDF fora do escopo e inspeção da rota HTML disponível retornou erro no browser; Museu não foi selecionado para esta amostra de dez. Nenhuma falha do extractor foi observada antes desta seleção.

| Caso de busca | HTML inspecionado | Literal alvo independente | Resultado de extração |
| --- | --- | --- | --- |
| 1 | [Rust capítulo 4](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html) | `without needing a garbage collector` | passou; dom_fallback, 430 caracteres, 128 ms |
| 2 | [Tauri CLI](https://v2.tauri.app/reference/cli/) | `List of Commands` | falhou; unsupported_content, 1.188 ms; fonte mantida no denominador |
| 3 | [React useEffect](https://react.dev/reference/react/useEffect) | `synchronize a component with an external system` | passou; readability, 20.000 caracteres, truncado, 665 ms |
| 4 | [WebView2](https://learn.microsoft.com/en-us/microsoft-edge/webview2/) | `HTML, CSS, and JavaScript` | passou; readability, 8.511 caracteres, 307 ms |
| 5 | [Python pathlib](https://docs.python.org/3/library/pathlib.html) | `Pure paths` | passou; readability, 20.000 caracteres, truncado, 800 ms |
| 6 | [SQLite](https://sqlite.org/foreignkeys.html) | `Enabling Foreign Key Support` | passou; readability, 20.000 caracteres, truncado, 1.226 ms |
| 8 | [INPE dados abertos](https://data.inpe.br/queimadas/portal/dados-abertos/index.html) | `Focos de Queimadas e Incêndios` | passou; readability, 2.052 caracteres, 350 ms |
| 10 | [WCAG 2.2](https://www.w3.org/TR/WCAG22/) | `Status of This Document` | passou; readability, 20.000 caracteres, truncado, 1.230 ms |
| 11 | [OpenAI guia HTML](https://developers.openai.com/api/docs/guides/tools-web-search) | `web_search` | passou; readability, 16.997 caracteres, 656 ms |
| 12 | [SearXNG](https://docs.searxng.org/dev/search_api.html) | `Search API` | falhou o literal no texto; readability, 2.811 caracteres, 656 ms. Título existe no DTO, porém esperado não foi mudado |

Meta de extração >=8/10. Contador do harness mede somente tentativas HTTP da avaliação pelo transporte real do Aura (handshakes/fallbacks/redirects inclusive), máximo 60. Inspeção independente preparatória pelo browser não é instrumentada por esse contador e não é execução do WebService. Prazo do experimento 30 minutos, sem rodar buscas extras/rotacionar identidade para contornar cota. Resultados não comprovam disponibilidade contínua, síntese de três jornadas, SIWC real, Windows/UX final ou privacidade do rollout normal.

## Limitações e próxima ação

O gate backend atingiu o limiar com duas falhas conservadas. A origem não é um índice próprio do Aura e não há SLA gratuito. Tauri precisará de diagnóstico no ticket dono da leitura; o erro foi classificado sem alegar leitura. Readability pode remover um cabeçalho da saída e enriquecer indevidamente o título com texto de navegação/acessibilidade (observado em React/SearXNG). Esses achados merecem correção posterior com fixture sintética e seam pública, sem repetir/trocar a amostra desta execução para inflar o resultado. O harness não salva páginas completas nem dá ao modelo credenciais/permissões extras.

TK-004 continua bloqueado por EV-032/FD-001: retenção web não citada no rollout normal. TK-005 continua pendente (cadeia determinística integrada, síntese de três jornadas, build/E2E Windows e convergência). Ao final, renovar QA compartilhado do esforço 038 e resolver DPI/resize/foreground. Nenhum commit, publicação ou atualização de sidecar.
## Renovação executada — plan r8

Nova execução em10/10/2026: **10/12 fontes confirmadas independentemente, 8/10 extrações, 58 HTTP em242s**. Limiares originais10/12 e8/10 atingidos. A consulta IBGE teve resultado indexado plausível, mas a inspeção independente do PDF falhou e a página HTML retornou403; ficou fora da aprovação, sem declarar o documento irrelevante. Tauri continua falhando a relevância do tema invoke. Esta contagem conservadora é separada dos11/12 históricos de r7.

Procedimento efetivamente executado:

```powershell
cargo run -p aura-web --example quality -- 'C:/Users/gabri/AppData/Local/Temp/aura-039-quality-r8-1791656334857-report.json' 'C:/Users/gabri/AppData/Local/Temp/aura-039-quality-r8-1791656334857-targets.json'
```

O processo terminou com exit0 e phase complete. Doze buscas consumiram48 requests/33s; uma única sessão foi mantida. Após as buscas, fontes oficiais foram inspecionadas pelo browser e dez HTML/literais fixados antes das leituras. O manifesto foi publicado completo, sem troca de alvo após resultados. Mais10 requests concluíram a leitura. O tempo total inclui inspeção independente, cujas requisições não entram no contador do transporte do Aura. Não houve inferência paga nem credencial de busca.

Metadados sanitizados e classificação em [quality-results-2026-10-10-r8.json](quality-results-2026-10-10-r8.json). Snippets/corpos foram removidos; títulos das leituras foram limitados a dez palavras, conservando a contagem original de caracteres para documentar ruído. A amostra anterior não foi sobrescrita.

| Caso | Posição | Fonte examinada | Relevância |
| --- | --- | --- | --- |
| 1 | 4 | [fonte](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html) | confirmada |
| 2 | — | CLI/releases do domínio alvo | tema incorreto |
| 3 | 1 | [fonte](https://react.dev/reference/react/useEffect) | confirmada |
| 4 | 2 | [fonte](https://learn.microsoft.com/en-us/microsoft-edge/webview2/) | confirmada |
| 5 | 3 | [fonte](https://docs.python.org/3/library/pathlib.html) | confirmada |
| 6 | 5 | [fonte](https://sqlite.org/foreignkeys.html) | confirmada |
| 7 | 1 | [fonte](https://www.ibge.gov.br/biblioteca/visualizacao/livros/liv102011.pdf) | não verificada; fora da aprovação |
| 8 | 1 | [fonte](https://data.inpe.br/queimadas/portal/dados-abertos/index.html) | confirmada |
| 9 | 2 | [fonte](https://museudoamanha.org.br/visite/horarios-e-ingressos) | confirmada |
| 10 | 1 | [fonte](https://www.w3.org/TR/WCAG22/) | confirmada |
| 11 | 2 | [fonte](https://developers.openai.com/api/docs/guides/tools-web-search) | confirmada |
| 12 | 5 | [fonte](https://docs.searxng.org/dev/search_api.html) | confirmada |

Os dez HTML e literais da tabela r7 acima reapareceram nos resultados e foram novamente inspecionados/fixados. Resultado atual, sem substituir as falhas:

| Caso | Resultado de extração | Extractor / tempo |
| --- | --- | --- |
| 1 | passou,430 caracteres | dom_fallback,122ms |
| 2 | falhou: unsupported_content |393ms |
| 3 | passou,20.000 caracteres,truncado | readability,572ms |
| 4 | passou,8.511 caracteres | readability,405ms |
| 5 | passou,20.000 caracteres,truncado | readability,385ms |
| 6 | passou,20.000 caracteres,truncado | readability,1.071ms |
| 8 | passou,2.052 caracteres | readability,133ms |
| 10 | passou,20.000 caracteres,truncado | readability,997ms |
| 11 | passou,16.997 caracteres | readability,461ms |
| 12 | falhou: literal Search API ausente no texto | readability,246ms |

Gate backend do TK006 renovado; não aprova síntese, histórico privado ou janela Windows. TK007 está bloqueado pela URL W2 não citada em arguments/rollout/SQLite (EV050/051), mesmo com outputs em memória; TK004/TK005 permanecem dependentes. Compactação/raciocínio/formatos comprimidos não exercitados continuam pendentes. Qualidade observada nesta amostra não garante disponibilidade contínua ou equivalência geral com produtos proprietários.

## Renovação atual — plan r9

Execução nova em 10/10/2026: **10/12 fontes relevantes confirmadas independentemente, 8/10 leituras com literal e origem, 58 HTTP do Aura em 422 s, exit 0 e phase complete**. Iniciada às 20:11:40 UTC. A retomada confirmou o mesmo handle 22833 vivo; nenhuma busca foi reiniciada. Dez alvos foram publicados atomicamente às 20:18:37 UTC, antes da extração. O tempo inclui a inspeção independente, cujas requisições não são instrumentadas pelo transporte do Aura.

Procedimento executado, sem chave/conta de busca ou inferência comercial:

```powershell
cargo run -p aura-web --example quality -- 'C:/Users/gabri/AppData/Local/Temp/aura-039-quality-r9-1791663097282/report.json' 'C:/Users/gabri/AppData/Local/Temp/aura-039-quality-r9-1791663097282/targets.json'
```

Relatório sanitizado em [quality-results-2026-10-10-r9.json](quality-results-2026-10-10-r9.json): sem snippets, corpos ou contexto de sessão; títulos das leituras limitados a dez palavras com tamanho original preservado. Amostras r7/r8 e manifesto temporário permanecem intactos. As consultas, domínios, limiares 10/12 e 8/10 e dez URLs HTML são os mesmos. O literal OpenAI desta inspeção é `up-to-date information`, escolhido no corpo da fonte antes de ler, sem troca após resultados.

Relevância confirmada nos casos 1/3/4/5/6/8/9/10/11/12, nas posições 4/1/2/3/5/1/2/1/2/4. A inspeção independente do [Museu](https://museudoamanha.org.br/visite/horarios-e-ingressos) agora retornou dias, faixa de horário e última entrada; isso confirma o tema pelo corpo, além do nome da página. A resposta HTTP direta continha apenas shell e não seria prova suficiente. O [IBGE PDF](https://www.ibge.gov.br/biblioteca/visualizacao/livros/liv102011.pdf) retornou 403 e a rota HTML não foi acessível: não verificado, fora da aprovação. Tauri continua falhando relevância porque CLI não explica invoke. Todas as doze consultas continuam no denominador.

| Caso | Extração atual | Tempo |
| --- | --- | --- |
| 1 | literal preservado; dom_fallback, 430 caracteres | 129 ms |
| 2 | unsupported_content; falha mantida | 127 ms |
| 3 | literal preservado; readability, 20.000 caracteres, truncado | 498 ms |
| 4 | literal preservado; readability, 8.511 caracteres | 316 ms |
| 5 | literal preservado; readability, 20.000 caracteres, truncado | 526 ms |
| 6 | literal preservado; readability, 20.000 caracteres, truncado | 1.071 ms |
| 8 | literal preservado; readability, 2.052 caracteres | 169 ms |
| 10 | literal preservado; readability, 20.000 caracteres, truncado, document_truncated | 1.022 ms |
| 11 | literal preservado; readability, 16.997 caracteres | 470 ms |
| 12 | Search API ausente no texto; falha mantida | 258 ms |

FD-002/FD-003 continuam achados menores, sem reduzir o denominador ou mudar o parser durante a avaliação. O resultado prova somente AC-001/AC-013 backend. Desde os registros históricos acima, TK-007 foi concluído com EV-068 e TK-004 com EV-072; seus bloqueios de retenção foram resolvidos. Próximo ticket TK-005 conserva cadeia determinística, três jornadas de síntese e build/E2E Windows. QA físico DPI/resize do esforço 038 e disponibilidade contínua da rota gratuita não são aprovados por esta amostra.
