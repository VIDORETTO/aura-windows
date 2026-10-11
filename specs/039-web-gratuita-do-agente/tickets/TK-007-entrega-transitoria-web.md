---
schema: hybrid/ticket
schema_version: 1.0
id: TK-007
effort: 039-web-gratuita-do-agente
type: delivery
status: done
ticket_revision: 40
requires: ["TK-003"]
requirement_refs: ["FR-005", "FR-006", "FR-007", "FR-008"]
acceptance_refs: ["AC-008", "AC-009", "AC-010", "AC-011", "AC-012", "AC-014"]
spec_revision: 1
plan_revision: 11
owned_areas: ["crates/aura-gateway/src/server.rs", "crates/aura-gateway/src/lib.rs", "crates/aura-gateway/src/tool_results.rs", "crates/aura-gateway/src/private_calls.rs", "crates/aura-gateway/src/sse.rs", "crates/aura-gateway/tests/private_calls.rs", "crates/aura-gateway/tests/tool_results.rs", "crates/aura-web", "crates/aura-app/src/web.rs", "crates/aura-app/src/host.rs", "crates/aura-app/tests/web_tools.rs", "crates/aura-app/tests/real_app_server.rs", "docs/adr/0010-entrega-transitoria-de-resultados-web.md"]
verification_status: passed
last_update: Git LF normalization explicitly revalidated; exact original fingerprints recoverable without behavioral edits. Canonical evidence/current0.3 release documented in041.
---







































# TK-007 — Entregar resultados web em memória pelo gateway

## Objetivo e limites

Entregar ao provedor o resultado web completo permitido por tool, mantendo somente um envelope opaco no output que o sidecar pode gravar. Preservar threads normais, mensagens, IDs e gestão de histórico. Executar como fatia vertical antes de retornar ao TK-004 bloqueado.

Não inclui upgrade, loop LLM, novo provedor, limpeza posterior de rollouts, redução do gate de retenção nem alterações de UI. O protótipo não constitui implementação nem aprovação do ticket.

## Leitura em ordem

1. AGENTS.md, docs/HANDOFF.md e CONTEXT.md — instruções e vocabulário.
2. `../spec.md` r1, `../plan.md` r9, `../tdd.md` — comportamento e oráculos.
3. `../persistence-research.md`, `docs/adr/0010-entrega-transitoria-de-resultados-web.md` — prova e limitações.
4. `crates/aura-gateway/src/server.rs` — auth, responses, Routes e start; transformação precede Upstream::responses.
5. `crates/aura-app/src/web.rs` — WebTurns/call_web e autoridade confiável; `host.rs` — composição e observer.
6. `crates/aura-web/src/lib.rs`, `src/cache.rs` — WebContext, orçamento e cancelamento.
7. `crates/aura-app/tests/real_app_server.rs` — protocolo real, histórico e oráculo de retenção; `crates/aura-gateway/tests` — fronteira HTTP existente.

## Decisões já resolvidas

Entrega transitória restrita a resultados web. Resolver na fronteira autenticada do gateway, antes de qualquer adapter. Header `thread-id` observado no fixado, sem correlação por conversa selecionada, único turno ativo ou ID aceito do modelo. Envelope exato versionado, referência aleatória de 256 bits emitida pelo Host, sem URL, título, snippet ou página. Validar função/namespace/callId pelo input da requisição antes de resolver.

Liberdade local: nomes privados, organização e mensagens estáveis sem conteúdo. Não escolher nova autenticação, alterar limites ou presumir que todo string JSON é envelope. Referência expirada no histórico resulta em erro de dado anterior indisponível, permitindo novo turno sem falhar toda a conversa; referência de outra Conversa bloqueia antes do upstream.

## Mapa de alterações

- Novo `aura-gateway/src/tool_results.rs`: interface de resolução e transformação limitada aos outputs reconhecidos; exportar em lib.rs e testar pela fronteira HTTP.
- Existente Routes/responses: resolver opcional compartilhado em todas as rotas; nenhum resolver significa passagem atual. Solicitações com envelope privado não geram dumps de diagnóstico, inclusive antes da expansão.
- WebService/cache: nova entrada de entrega transitória incluída nos mesmos tetos de entradas/8 MiB/32 MiB. Não criar HashMap de conteúdo sem contabilização fora desse Module.
- Host/WebTurns: reter output antes de devolvê-lo pelo MCP, vincular thread/turno/sessão e compor resolver pelo mesmo serviço. Em falha de admissão, devolver erro estável e nunca conteúdo bruto como fallback.
- Real_app_server/web_tools: adaptar o consumidor conforme fronteira pública e preservar oráculos do provedor e disco. Fora: shell, UI, pin de sidecar, dados do usuário.

## Contrato técnico

Entradas: requisição Responses autenticada, thread-id confiável, input com function_call identificado e output correspondente; resultado web emitido pelo Host para WebContext vigente. Saída: mesma requisição com somente o envelope autorizado substituído por texto estruturado externo, ou erro explícito sem conteúdo. Todas as rotas recebem expansão antes da tradução e usam o mesmo resolver. Complemento r9: retorno SSE normalizado protege argumentos das chamadas web antes do sidecar; Host e gateway resolvem somente o envelope próprio de argumentos conforme schema. O envelope de resultado nunca autoriza expansão em arguments. Scope de retorno é capturado antes do await upstream e não é renovado por chegada tardia.

Autorização não vem do corpo, namespace isolado ou texto de página. Não expandir em user/developer/system, instruções, arguments, resultados de terceiros ou texto contendo um marcador como substring. Uma expansão não percorre recursivamente o conteúdo substituído. Referência vinculada à Conversa/thread emissor, ao Turno e à sessão; nonce desconhecido, sessão inválida ou escopo divergente nunca lê outro conteúdo.

Entradas de entrega compartilham limites com buscas/documentos/registry, expiram em no máximo 15 min e são removidas na conclusão/cancelamento do Turno, desativação, fechamento, exclusão e shutdown. Cache normal pode continuar no prazo da spec; envelope antigo não restaura conteúdo após reinício e não dispara rede automática. Toggle false é verificado na resolução; após true, envelopes cancelados continuam inertes. Retries do mesmo pedido ativo são idempotentes: leitura não consome a referência.

Headers/body/envelopes não são logados; erro de resolução não inclui nonce, URL, query ou conteúdo. Não criar arquivo temporário de página. O gateway mantém auth de loopback e rejeição de Origin. Normalidade e identidade do thread não dependem do envelope.

## Exemplos de aceite

- **AC-008/AC-014**: literal `PRIVATE_PAGE_039` emitido por web_fetch → provedor recebe literal e marca externalContent; sidecar recebe referência. Mesmo envelope em mensagem do usuário ou resultado de outro servidor permanece texto, sem acesso ao conteúdo.
- **AC-009/AC-012**: duas Conversas A/B, mesma forma de callId, nonce de A na requisição de B → nenhum upstream e nenhum conteúdo de A. Reinício mantém resposta/fontes citadas e threadId, mas nunca conteúdo bruto do envelope anterior.
- **AC-010/AC-011**: desativar/cancelar/fechar/remove e atraso de I/O → referência indisponível; reativar não a revive. Novo pedido pode usar cache permitido e emitir outra referência, sem contornar orçamento.
- **AC-012**: entrega mais entradas recentes excederia 8/32 MiB → limit_exceeded antes de devolver envelope ou raw fallback. Dois retries autorizados entregam o mesmo literal, sem duplicar armazenamento ou cruzar sessão.
- **AC-014**: Responses, Chat e Anthropic recebendo conteúdo expandido no gateway real, com app-server fixado; histórico normal permanece legível e compactação não reintroduz página não citada.

Oráculos literais independentes e scans do CODEX_HOME isolado; não recompor transformação no teste. O scan abrange JSONL e storage comprimido/SQLite que existam, não somente um literal em um arquivo.

## Dependências e sequência de execução

Depende de TK-003 done (ferramentas e gateway integrados). TK-004 fica bloqueado até esta entrega; executar uma fatia por vez, sem trabalho paralelo nas áreas compartilhadas.

- [x] TK-007.1 Package ready:true, graph e inputs atuais; primeiro teste na fronteira HTTP com red comportamental.
- [x] TK-007.2 Implementar resolução exata e escopo; casos de mensagem/terceiro/fake nonce/cross-thread um por vez.
- [x] TK-007.3 Retenção em orçamento real e ciclo de vida pelo Host/WebService; red → green por caso.
- [x] TK-007.4 Teste normal de retenção, reinício/novo turno e compactação reais; nove combinações de protocolo/modo e regressões existentes.
- [x] TK-007.5 Evidências reais, revisão Standards/Spec, atualização do ADR e checkpoint; retorno ao TK-004 apenas após done.

## Validação

Raiz: `cargo test -p aura-gateway --test tool_results`; `cargo test -p aura-web --test search --test fetch`; `cargo test -p aura-app --test web_tools --test host --test ipc_contract`; `cargo test -p aura-app --test real_app_server web:: -- --ignored --test-threads=1`; `cargo test --workspace --exclude aura-desktop`; `cargo clippy --workspace --all-targets --exclude aura-desktop -- -D warnings`; `cargo fmt --all -- --check`; `git diff --check`.

AURA_CODEX_BIN fixado e verificado; AURA_E2E_DIR sob TEMP isolado. A implementação já passou no caso real `web::web_normal_research` que verifica o literal de página; esse caso não aprova o gate integral de retenção. A suíte HTTP `tool_results` passou em 9/9 casos no checkpoint atual. Reinício, compactação, matriz completa de protocolos/modos e inspeção de URLs não citadas, argumentos, raciocínio e armazenamento comprimido/SQLite continuam pendentes. Protótipo Python externo já executado, apenas prova de viabilidade. Falha de ambiente é impedimento; literal não entregue ou página persistida é red real.

## Condição de retorno à planejadora

Header ausente/divergente no protocolo real, envelope não distinguível de dado externo, exigência de reidratação instável, orçamento incompatível, argumento/raciocínio persistindo conteúdo proibido ou falha de gestão do histórico. Manter expected do gate; não trocar normal por efêmero nem aceitar a falha.

## Retomada técnica — plan r9

Condição de retorno satisfeita: planejamento aprovado de argumentos privados antes do sidecar, baseado em EV050/051 e protótipos conforme schema em persistence-research.md. Ler a seção r9 antes de executar. Novos arquivos previstos: `crates/aura-gateway/src/private_calls.rs` e `crates/aura-gateway/tests/private_calls.rs`. Existentes: `tool_results.rs`, `server.rs`, `aura-web/src/delivery.rs`, `aura-web/tests/delivery.rs`, `aura-app/src/web.rs`. Ownership restrito à entrega privada deste ticket; nenhuma mudança nas permissões, schema MCP, spec ou versão do sidecar.

Primeiro comportamento: esconder argumentos da chamada web em SSE, restituir ao Host e ao provedor, preservar thread/resposta e fazer o gate W2 passar sem mudar expected. Depois: fragmentação/intercalação/origem/ciclo de vida/erro sem conteúdo. Raciocínio e compactação seguem obrigatórios, com nova evidência; resultados de argumentos não aprovam a totalidade de AC009. Executar `cargo test -p aura-gateway --test private_calls`, `cargo test -p aura-web --test delivery`, `cargo test -p aura-app --test web_tools` e o gate real W2, além da matriz/regrissões previstas. Não concluir o ticket na passagem de uma camada parcial.

## Progresso r9 — argumentos integrados

O gate W2 passou com app-server fixado sem alterar expected, após red0,81s. Outputs e argumentos têm kinds distintos no mesmo mapa/budget de memória; Host resolve antes do DTO e gateway antes dos adapters. Scope é capturado antes de I/O; a prova HTTP troca o contexto ativo enquanto upstream aguarda e confirma recusa da emissão tardia. Dez casos HTTP novos cobrem schema, UTF-8/CRLF, deltas/intercalação, namespace/user, escopo, orçamento, expiração, snapshots e call_id histórico. O parser existente `aura-gateway/src/sse.rs` também foi corrigido após red de Unicode (detalhe reversível necessário aos três adapters, sem nova dependência/contrato).

O fixture `aura-app/tests/web_tools.rs` usa requests observados pelo provedor externo real da fronteira HTTP e resposta SSE vazia, em vez de JSON sem formato Responses; todos os14 oráculos existentes foram preservados. Guardar perfil sintético de teste pode ser optado com AURA_WEB_KEEP_SYNTHETIC_PROFILE=1 para auditoria; nenhum perfil do usuário participa. Auditoria sanitizada de86arquivos/128linhasSQLite: zero dos quatro literais; sem frames comprimidos nesta fixture. Raciocínio, blob de provedor real, compactação e gate integral continuam pendentes. TK007 permanece in_progress/partial; não desbloquear TK004 ainda.

## Relatório de saída

Provas atuais:504regressões workspace,25integrações fixadas,16HTTP private_calls,9delivery e diagnóstico isolado com controle positivo. AES-GCM externo foi preservado/decifrado independentemente após reinício/compactação; auditoria final88arquivos/139registros sem quatro marcadores e scan nativo sem ciphertext. Raciocínio/compactação também exercitados em Chat/Anthropic; eventos públicos sem texto privado/referências. Clippy/all-targets/fmt/diff passaram. Comandos/resultados/limitações em persistence-research.md. A prova não executa modelo comercial nem adiciona replay de assinaturas proprietárias; frames comprimidos foram exercitados em controles do auditor, pois não apareceram no perfil Codex. Gate do ticket refere-se à entrega transitória; UI/live/Windows permanecem nos sucessores.

Arquivos, comandos, revisões, EVs e limitações; resultados separados para viabilidade, implementação e gate integral de TK-004. Done exige verificação e revisão, não somente passagem do protótipo.
