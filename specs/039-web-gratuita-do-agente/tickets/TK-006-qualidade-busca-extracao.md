---
schema: hybrid/ticket
schema_version: 1.0
id: TK-006
effort: 039-web-gratuita-do-agente
type: delivery
status: done
ticket_revision: 32
requires: ["TK-003"]
requirement_refs: ["FR-001", "FR-003", "FR-008"]
acceptance_refs: ["AC-001", "AC-013"]
spec_revision: 1
plan_revision: 11
owned_areas: ["crates/aura-web/examples/quality.rs", "specs/039-web-gratuita-do-agente/quality.md", "specs/039-web-gratuita-do-agente/quality-results-2026-10-10.json", "specs/039-web-gratuita-do-agente/quality-results-2026-10-10-r8.json", "specs/039-web-gratuita-do-agente/quality-results-2026-10-10-r9.json", "specs/039-web-gratuita-do-agente/quality-results-2026-10-10-r10.json"]
verification_status: passed
last_update: Git LF normalization explicitly revalidated; exact original fingerprints recoverable without behavioral edits. Canonical evidence/current0.3 release documented in041.
---































# TK-006 — Avaliar relevância e extração na internet real

## Objetivo e limites

Executar a avaliação backend de AC-013 anteriormente incluída no TK-005, independente da retenção normal do Codex. Demonstrar WebService gratuito com transporte seguro real, relevância e extração fiel. Não aprova UI Windows, síntese de três jornadas, privacidade do rollout nem entrega final; esses gates continuam nos TK-004/TK-005. Não alterar consultas/metas após obter resultados.

Autorização: executar tickets um por vez e avançar trabalho independente quando houver bloqueios, deixando sua resolução para o final.

## Exclusões

Não implementar UI, síntese LLM, persistência normal ou upgrade do sidecar. Não criar conta/chave/pagamento, executar JS, contornar CAPTCHA, trocar consultas/metas ou publicar instalador. Não aprovar os demais tickets com esta avaliação.

## Leitura em ordem

1. AGENTS.md, CONTEXT.md e docs/HANDOFF.md.
2. spec.md r1 AC-001/AC-013, plan.md r9 Fatias/avaliação, contracts.md e tdd.md consultas fixas.
3. `crates/aura-web/src/lib.rs`: WebService::new/begin_turn/search, SearchRequest e WebSource.
4. `crates/aura-web/src/fetch.rs`: FetchRequest/FetchOutput e limites de paginação.
5. `crates/aura-web/src/transport.rs`: Transport/ReqwestTransport, DNS/pin/stream.
6. `crates/aura-web/tests/search.rs` e `tests/fetch.rs`: smokes já existentes, sem confundir com avaliação de qualidade.

## Decisões já resolvidas

Usar o serviço e transporte de produção sem chave, conta, proxy ou bypass. Wrapper do adapter externo apenas conta requests; não substitui parser, cache ou registry. Limite 60 HTTP/30 minutos, uma sessão por avaliação, 12 buscas e 10 leituras principais. Harness e resultados são novos; nenhuma dependência de runtime adicional. Oráculos e metas fixos em tdd.md.

Liberdade: formato local do harness e timestamps. Divergência de serviço/qualidade volta ao planejamento com evidência, sem reduzir meta.

## Mapa de alterações

Novo example `quality.rs` executável por cargo; relatório `quality.md` contém metadados e avaliação, sem páginas inteiras. Não editar produção para fazer avaliação passar. Problema de adapter/extractor retorna ao ticket dono e exige regressão/novo pacote.

## Contrato técnico

Entrada: 12 consultas públicas literais de tdd.md; dez URLs HTML estáticas selecionadas dos resultados com alvos inspecionados e fixados antes de extrair. Saída: posição/domínio/URL relevante, resultado de leitura/alvo/extractor, contagem HTTP, latência e erros estáveis. HTTP público com validação DNS/pin/redirect, sem credenciais. Não rotacionar sessão ao limitar. Parar nos tetos; nenhuma retenção de páginas no repositório.

## Exemplos de aceite

- AC-001: serviços gratuitos recebem consultas públicas sem chave/Authorization/conta de busca.
- AC-013: Rust ownership retorna rust-lang.org/capítulo Ownership entre os primeiros cinco. A lista completa e seus domínios não mudam; meta pelo menos 10/12. Dez HTML estáticos têm título/seção/fato literal fixado por inspeção independente; pelo menos 8/10 leituras preservam alvo e origem. Erros são falha/partial conforme observado, não sucesso por título parecido.

Oráculo independente: lista de domínios/temas e alvos da fonte; nenhum esperado gerado pelo extractor/modelo. Casos já verdes são cobertura, sem red artificial.

## Dependências e sequência de execução

TK-003 done fornece serviços/integração. TK-004 e TK-007 concluíram a retenção normal com EV-068/EV-072. TK-005 exige TK-004 e este gate backend; esforço não será concluído sem síntese e Windows.

- [x] TK-006.1 Package ready:true e graph antes de criar o harness.
- [x] TK-006.2 Inspecionar seams e oráculos; execução de qualidade não usa internet para fabricar red unitário.
- [x] TK-006.3 Executar 12 buscas; classificar top5 por domínio e conteúdo que responde ao tema.
- [x] TK-006.4 Inspecionar/fixar dez alvos HTML estáticos antes de extrair; executar até dez leituras e registrar os resultados.
- [x] TK-006.5 Evidence add após execução, revisão Standards/Spec, estados pelo runner e projeções por render.

## Validação

Comandos da raiz: `cargo run -p aura-web --example quality -- <report.json> <targets.json>` usa uma sessão para busca e leitura, orçamento compartilhado e arquivos TEMP explícitos. Argumentos exatos e alvos da execução r7 estão em quality.md e EV-033. `cargo test -p aura-web --test search --test fetch`, `cargo clippy -p aura-web --all-targets -- -D warnings`, `cargo fmt --all -- --check`.

Procedimento de renovação: escolher dois caminhos novos sob TEMP, sem sobrescrever o relatório anterior; iniciar o harness com esses argumentos; após as doze buscas, inspecionar e fixar dez alvos HTML presentes nos resultados e gravar o manifest no segundo caminho, conforme a estrutura documentada no example; aguardar as dez extrações e saída terminal. Registrar comandos/argumentos exatos, classificação independente, datas e limitações. O limite permanece 60 requests/30 minutos; não trocar consultas ou remover falhas de HTML estático para cumprir a meta.

Execução histórica r7: 11/12 relevantes e 8/10 extrações, 58 HTTP do Aura em 239 s. Experimento e casos já verdes não produziram red artificial. A mudança para plan r8 invalidou EV-033; esta sessão não reexecutou a avaliação ao vivo, e o estado atual é implemented/stale aguardando a renovação final. Sem internet disponível registrar impedimento concreto.

Renovação r8 executada após bloquear TK007:10/12 fontes confirmadas independentemente,8/10 extrações,58 HTTP/242s,exit0/phase complete. IBGE não entrou na aprovação por falha da inspeção independente; Tauri teve tema incorreto. Os dez alvos fixados antes da leitura e as duas falhas de extração foram preservados. Argumentos exatos, classificação e limites em quality.md; metadata sem snippets/corpos em quality-results-2026-10-10-r8.json. A declaração anterior implemented/stale descreve somente o período anterior a esta execução, não a situação renovada.

## Condição de retorno à planejadora

Renovação r9 executada: 10/12 fontes confirmadas por conteúdo independente, 8/10 extrações, 58 HTTP em 422 s, exit 0/phase complete. Mantidos as doze consultas e dez HTML; alvos fixados antes de ler. Literal OpenAI escolhido nesta inspeção: `up-to-date information`, sem mudança após observar extração. Tauri continua sem relevância e unsupported_content na leitura; IBGE não verificado por 403/erro de acesso; Search API do SearXNG ausente no texto continua falha. Relatório e manifesto prévios preservados. Metadados em quality-results-2026-10-10-r9.json; limitações e revisão atuais em quality.md/review.md. O processo 22833 foi observado vivo antes da retomada e aguardado até saída terminal, sem reiniciar consultas.

Meta falha, opção gratuita muda, destino inseguro, necessidade de API paga/JS/contorno de cota, erro de parser ou oráculo sem inspeção prévia. Preservar resultados e não substituir queries/sites após falha para inflar métrica.

## Relatório de saída

quality.md e quality-results-2026-10-10.json registram a amostra r7 e suas limitações. EV-033 e revisão Standards/Spec aprovaram aquela execução somente no escopo backend; após invalidação r8, nova evidência é necessária para voltar a done. Síntese/Windows/privacidade continuam pendentes nos tickets anteriores.
