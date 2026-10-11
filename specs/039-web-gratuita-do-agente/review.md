# Revisão — web gratuita do agente

## TK-005 — síntese real SIWC executada, gate falhou

O usuário confirmou nesta sessão o ChatGPT já conectado ao Aura. Auth_status público confirmou conta ativa/plano habilitado; settings_get webEnabled=true; models_list/diagnostics públicos confirmaram o pin0.159.0 pronto e Aura MCP39 ferramentas sem erro. Nenhum e-mail, identificador de conta, token ou credencial foi exportado. Build Windows e2e sem demo concluído exit0 em2m57s, sem opt-in de transporte sintético.

Três pedidos fixados antes da execução, no modelo gpt-6-luna real e em Conversas efêmeras próprias, atravessaram Host IPC → app-server fixado → SIWC/gateway → MCP → busca/leitura públicas. Não se alteraram settings/provedores/conta. A execução final usou a instalação completa padrão em LOCALAPPDATA/Aura, com AURA_CODEX_BIN ausente; o app verifica o executável instalado. Os três Turnos completaram em40,2/27,9/22,9s e as Conversas efêmeras foram encerradas. Resultado sanitizado: experiments/synthesis-siwc-2026-10-10.json; harness e resultados anteriores permanecem em TEMP/aura-039-live-chatgpt-20261010-182133.

Fato atual (Museu): duas pesquisas, duas leituras recusadas; resposta informou limitação e não afirmou horários como verificados. Comparação Python/Rust: duas páginas oficiais tiveram pageContent registrado, mas o modelo informou receber identificadores opacos sem texto/sourceId citável. URL WCAG22: leitura bem-sucedida registrada; modelo informou a mesma limitação, sem fornecer os princípios como leitura confirmada. Portanto, o gate de síntese/compatibilidade falhou, apesar dos Turnos completed; ressalvas honestas não aprovam entrega quando a leitura teve sucesso e o conteúdo não chegou ao consumidor.

A causa precisa de investigação na forma de outputs consumida pelo modelo real, possivelmente pelo caminho code-mode; essa hipótese não é diagnóstico confirmado por captura de payload. Métodos/predecessores com upstream scripted continuam com evidências válidas no seu escopo, sem extrapolá-las para SIWC. Devolver à planejadora antes de alterar retenção/rehidratação ou desativar capacidades; preservar texto externo fora do sidecar. Não reduzir gate, trocar perguntas ou liberar raw fallback.

Erros do harness foram preservados: resolução inicial de módulo/comando, nome de provedor e leitura includeTurns de Conversa efêmera foram corrigidos somente em TEMP. O override inicial apontava para cópia antiga do app-server sem code-mode-host; três respostas sem ferramenta dessa tentativa não avaliam o produto. Nova execução removeu o override, esperou prontidão pública e observou ferramentas/páginas reais. As respostas finais foram coletadas por messageCompleted público, sem pedir includeTurns de thread efêmera. Nenhum red produtivo artificial, nova instalação, inferência paga separada, publicação ou mudança de pin.

Standards: logs do harness limitados a fase/status/contagem; metadata sanitizada e respostas QA sem páginas completas/segredos. Spec: backend liveEV082 e E2E scriptedEV084 permanecem aprovados no escopo; licenças passaram. TK-005 permanece incompleto: síntese falhou; destino externo do opener e DPI/resize038 também não foram aprovados. Nenhum processo QA restante ao fim da execução.

## TK-001 — renovação após dependências opcionais do shell

Cargo.lock mudou apenas para declarar aura-web/futures já presentes como dependências opcionais do desktop e2e. A implementação de busca, transporte, cache, orçamento e parsing não mudou; a evidência EV-076 ficou stale pelo runner. Renovação: search/fetch54 testes passaram (2 live ignorados, não aprovados por esta execução), cargo check do desktop de produção passou e clippy aura-web/all-targets -Dwarnings/fmt/diffcheck passaram. Instalação CLI cargo-deny0.20.2 em target/qa-tools concluída exit0; `cargo deny check licenses` com PATH somente do processo retornou licenses ok. Nenhuma instalação global, versão de app-server ou biblioteca adicional. Standards sem achado bloqueante.

Spec: AC-001/002/003/012 preservados pelos testes públicos originais, sem alteração de oráculo. Gate ao vivo EV-082 continua atual e não foi reexecutado desnecessariamente: nenhum input do backend ou plano mudou. A mudança de Cargo.lock é composição exclusiva de QA, não alteração de qualidade do provedor. Após nova evidência, TK-001 pode voltar a done; TK-005 permanece parcial EV-084 e os demais gates pendentes não são aprovados por licenças/testes backend.

## TK-005 — composição e jornada nativa, evidência parcial atual

Feature e2e + opt-in explícito compõem somente a variação externa DNS/HTTP; main.rs encaminha HostConfig existente, preserva SpawnHook e exige binário explícito. Sem biblioteca externa nova; aura-web/futures já existentes são opcionais no shell. Produção sem e2e passou em cargo check. Feature demo,e2e passou em check/clippy -Dwarnings e build release2m54s. Pin não alterado. Perfis QA em TEMP isolados, trace contém exclusivamente URLs sintéticas permitidas; sem corpo/header/credencial. Código novo incluído nesta revisão, sem commit/staging. Standards sem achado bloqueante na fatia.

E2E real em WebView2 154.0.4258.62, TEMP/aura-039-native-web-r10-run12-20261010:3/3 em4,2s. Pedido por textarea; fatos42/12/30 nos inputs reais do provedor; HTML ativo/menu removidos; dois domínios/citações; histórico por teclado/metadata sem nova rede; stream pendente interrompido por Parar sem leitura/fonte/próxima chamada; desligamento real em Settings cancela leitura, nova proposta de fetch não chega ao adapter, reativação permite nova leitura. A proposta desabilitada é recusada pela cadeia nativa antes do mesmo DTO usado no RPC direto; não se fabricou code web_disabled em output de outro formato. Erros diretos continuam cobertos pelas suítes existentes. Fonte recebe clique real; destino no navegador ainda não foi confirmado. Computer Use interrompeu inspeção pela ausência de suporte à política de URLs no navegador Windows atual; nenhuma entrada adicional foi enviada por esse plugin.

web_research5/5 atual em3,43s, scoped E2E TypeScript/fmt/diffcheck passaram. Erros iniciais de ambiente (pasta TEMP) e seletores/DTO/sincronização do harness foram corrigidos sem alteração produtiva nem red artificial. Histórico via mouse apresentou alvo de renomear após hover; teclado foi verificado com foco real. Fechar/reabrir WebView via WebDriver não recompôs target Settings; o teste conserva a mesma janela para verificar o controle global, sem aprovar reabertura. Esses limites não são substitutos de síntese LLM real, SIWC ou DPI. Global E2E tsc encontrou erros preexistentes fora da fatia. Cargo-deny inicialmente ausente; instalação CLI em target/qa-tools/cargo-deny em andamento, sem instalação global. Cargo.lock mudou apenas pela composição opcional do shell; EV076 ficou stale e precisa renovação. EV084 parcial não autoriza done do ticket/esforço.

## TK-006 — renovação concluída no plan r10

Mesma baseline fixa e trabalho local sem commit/staging. Standards: código e harness não alterados; uma sessão de 12 buscas/10 leituras, 58 HTTP/193 s, processo 7974 observado terminal exit 0. Manifesto fixado antes de extrair, resultados anteriores intactos, relatório atual sanitizado sem snippets/corpos/queryContext. Dez fontes HTML inspecionadas antes das leituras; a confirmação adicional do corpo do Museu às 20:44:59 UTC classifica apenas a busca, ainda dentro do prazo e sem mudar alvos de extração. Nenhuma conta, dependência, pin ou permissão introduzida. Sem achado Standards bloqueante.

Spec: 10/12 relevantes e 8/10 extrações satisfazem AC-001/AC-013. IBGE não verificado, Tauri fora do tema e unsupported_content, SearXNG sem literal mantidos nos denominadores. FD-002/003 continuam menores. Regressões plan10 EV-076/077/080/081/079 atuais. TK-006 pode concluir após registrar a nova evidência; síntese de três jornadas, E2E nativo e DPI não foram aprovados por esta avaliação.

Baseline fixo: `ed9a9654c6e76e48186fc010a12eba77023b00f5`, branch `release/0.2.0`. Inspecionados diff committed/cached/unstaged e novos arquivos; committed/cached sem mudança. Esta revisão inclui somente TK-001: workspace/lockfile, `crates/aura-web` e artefatos do esforço 039. Alterações do 038 e arquivos preexistentes permanecem fora deste escopo. Spec r1 / plan r2.

## TK-001 — Standards

Rust 2024, biblioteca independente de Windows/Tauri, erros thiserror estáticos e DTO camelCase com entrada fechada. Não há segredo, histórico, logs de consulta, API paga, dependência não permissiva ou mudança no sidecar fixado. DNS/HTTP/clock são as variações externas; parser, registry e orçamento reais no teste. Oráculos literais e casos de corrida/cancelamento não duplicam o algoritmo. Árvore de licenças passou em cargo-deny; Clippy -D warnings e rustfmt passaram. Fixture tools/list reduz o schema observado a campos/tipos, sem executar descrições remotas. Nenhum achado bloqueante neste ticket.

## TK-001 — Spec

Seam pública comprova busca anônima, fontes W1 com campos literais, sessão aleatória estável, ausência de Authorization/model_name/histórico, alternativa gratuita limitada e origem degradada. Dedupe conserva ordem e query semântica; data desconhecida continua null e sucesso vazio não tenta fallback. Limites de entrada, concorrência e três pesquisas por Turno, deadline compartilhado e cancelamento antes de alternativa/registro estão cobertos. SSE conclui com resposta correspondente mesmo sem EOF, inclusive UTF-8 fragmentado/heartbeats/mensagem de outro ID. Revisão encontrou URL de fonte acima de 2.048 caracteres aceita; caso público reproduziu duas fontes em vez de uma e a correção passou. Stream acima de 2 MiB é rejeitado, erro remoto bruto não é devolvido. Nenhum achado bloqueante remanescente no escopo TK-001.

Limites de cobertura: AC-001 aqui prova o serviço sem chave, não o caminho do modelo/Host (TK-003). AC-012 aqui prova entrada, orçamento de busca, concorrência e tamanho de resposta; documentos no TK-002 e isolamento global/cache/UI no TK-004 continuam pendentes. Disponibilidade real observada com uma consulta pública não aprova AC-013 ou compatibilidade AC-014. Não há web integrada no aplicativo ainda. Os AC compartilhados só estarão completos ao convergir as evidências dos demais tickets. TK-001 liberado para done somente no escopo definido pelo seu ticket, após registrar as execuções atuais.

## Plan r3 — renovação TK-001 e leitura TK-002

Baseline preservado. Inspecionados novos fetch.rs/fetch.rs de teste/report.html, mudanças no transporte/lib/search e árvore Cargo. Evidências r2 anteriores foram explicitamente invalidadas antes de renovação; spec r1 mantida. A precisão no roteiro de rebinding conserva a proibição de conexão privada: o endereço público pinado não é trocado por nova resolução. Nenhum aceite foi reduzido para obter passagem.

### Standards

Extração em spawn_blocking com entrada limitada, DOM limitado, dom_smoothie MIT fixado, encoding_rs para BOM/charset declarado; nenhuma execução JS/browser/SO. Reqwest mantém hostname TLS/SNI, ausência de proxy/cookies/creds, pin e peer validado, redirects manuais e decodificação antes do contador de bytes. Caminho de captura/ingestão e sidecar intactos. DTOs fechados, erros redigidos e entrada/contexto checados antes de rede/cache. Mocks somente DNS/HTTP/clock; HTML/normalização/extração/versão/evicção reais. Fixture Markdown expressa o mesmo fato com pontuação escapada, sem mudar o valor 42/12/30 ou remover tabela. Licenças, Clippy e formato passaram. Nenhum achado bloqueante remanescente nesta fatia.

### Spec

TK-001 foi reexecutado após mudar lib/transporte/plan; 18 casos e smoke anônimo passaram. TK-002: 29 casos comprovam artigo estruturado, charset ISO-8859-1, paginação ABC/DE e emoji, versão divergente/expirada/evictada sem misturar conteúdo, URL pedida/final, três redirects, destinos privados/userinfo/portas/reservados/DNS misto rejeitados, pin sem segunda resolução divergente, PDF/login/JS/CAPTCHA/vazio distintos. Cancelamento não registra fonte (próxima fonte continua W1), timeout é 30 s virtual, stream acima de 2 MiB é rejeitado e documento termina em 100.000 caracteres. Cache de documentos tem provas por entradas/bytes/agregado e é memória; sete fetches no Turno são limitados mesmo cacheados. Revisão também corrigiu duplicação de main/article aninhados, com red duas ocorrências/green uma.

Leitura real do capítulo Ownership do Rust retornou extractor=readability e 20.000 caracteres com próxima página/truncamento, via transporte de produção/TLS/pin, sem bypass. Prova pontual, não benchmark de extração de dez sites. O teste de rebinding da seam comprova endereços entregues à conexão; o adapter real usa resolve_to_addrs/peer, e o smoke positivo passa por ele. Não se alega montar infraestrutura DNS maliciosa no CI. Limites restantes: ferramentas/modos/privacidade do Host no TK-003, cache de busca/registry/UI/desativação/histórico no TK-004 e gates de qualidade/fixado/Windows no TK-005. TK-001 renovado e TK-002 liberados para done no escopo dos tickets, após evidências atuais; esforço permanece ativo.

Adendo plan r4: somente seam de composição/observação do TK-003 foi detalhada. Standards/Spec dos serviços TK-001/TK-002 não têm mudança de implementação/oráculo; 47 casos offline foram executados novamente e registrados EV-006/EV-007, sem reutilizar evidência stale. Liberados para done no mesmo escopo. Integração TK-003 continua sem prova neste adendo.

Renovação de 08/10: a composição acrescentou aura-web às dependências de aura-app e futures aos testes externos de rede. O diff do lockfile não muda os serviços, seus contratos ou os oráculos revisados acima. Os 47 casos foram novamente executados (EV-008/EV-009); TK-001/TK-002 liberados no escopo anterior. Esta revisão não aprova a integração ainda em implementação.

## TK-003 — Standards (10/10/2026)

Baseline SHA preservado; committed e staged sem mudança. Escopo: composição Host/HostTools, novo web.rs e web_tools.rs, CodexService/personas, schemas MCP, encaminhamento None no shell, manifesto aura-app e aviso localizado no estado/i18n da UI. Novos arquivos foram lidos, não omitidos do diff. Nenhum segredo vai para webview, config ou rede pública; slots Weak evitam ciclo do Host. Identidade vem de eventos confiáveis síncronos anteriores ao broadcast, e o token opaco do serviço impede reutilizar autoridade de um Turno encerrado. Nenhuma API Windows, novo modelo, sidecar ou dependência não permissiva foi introduzida nesta fatia. Entrada fechada e outputs somente texto, com externalContent; nenhum markup recebido é executado.

Mocks somente em DNS/HTTP/clock/OS/processo; Host, MCP, serviços e tradução reais. O app-server fixado é processo real, com provedores de inferência scripted e páginas públicas representadas pelo adapter externo. Oráculos verificam os resultados efetivos recebidos pelo provedor (42, 30, W1, W2, URLs), além da resposta literal: não basta o modelo scripted devolver uma resposta pronta. Códigos de aviso são traduzidos nas tabelas pt-BR/en; contrato IPC preservado. Clippy de workspace sem shell/all-targets, rustfmt e diff check passaram; testes públicos e types passaram. Sem achado bloqueante no escopo.

## TK-003 — Spec

Dez casos Host/MCP comprovam autoridade ausente/ambígua/desconhecida antes de rede, isolamento de duas Conversas, DTO fechado, budget de seis leituras mesmo cacheadas e reset somente por novo Turno confiável. Interrupt, fechamento efêmero, exclusão e shutdown cancelam/limpam antes de aguardar processo; shutdown não depende de turn/completed. Modelos com metadata conhecida sem ferramentas recebem aviso sem impedir Chat, traduzido nas duas línguas.

Doze testes atuais com rust-v0.159.0 comprovaram as nove combinações Responses/Chat/Anthropic × Chat/Plano/Tarefa, leitura depois de reiniciar/resumir e comparação com dois domínios. Namespace e resultados W1/W2/fatos 42/30 atravessam Host → Codex → gateway → provedor. Config gerada e override start/resume mantêm hosted web_search disabled mesmo diante de override live do chamador. Pesquisa não altera network=false ou pastas concedidas da Tarefa. Tentativa scripted de gravar após página maliciosa mantém a aprovação existente; recusa não salva o comando nem altera Settings/Persona. Regressão real separada de sandbox também negou escrita fora do workspace e pediu aprovação para escalada, recusada pelo teste.

Limites: prova de transporte/autoridade/aprovação não promete imunidade geral de LLM a prompt injection nem qualidade comercial ChatGPT. Badge/link do marcador e rejeição visual de W999, fontes/histórico/efemeridade, desativação e cache agregado pertencem ao TK-004; relevância/extrator ao vivo e Windows ao TK-005. AC compartilhados aqui são aprovados somente na fatia declarada, não no esforço inteiro. O probe de dumps brutos não é gate de comportamento e não foi executado. TK-003 liberado para done após registrar evidências destas execuções; esforço permanece ativo.

## Renovação dos predecessores e TK-004 parcial — 10/10/2026

Baseline fixo preservado. Esta renovação considera os arquivos novos, alterações unstaged e a integração do cache/controle/eventos; committed e staged continuam sem mudanças. Não é a revisão final do TK-004.

### Standards

Default retrocompatível e IPC dourado Rust/TS foram atualizados juntos. Toggle invalida capacidades e cancela operações sem reiniciar os budgets do Turno; fontes publicadas passam pela autoridade confiável. Nenhuma dependência nova foi introduzida nesta fatia. Seams públicas e oráculos literais foram preservados. A retomada real passou de nove para cinco requisições HTTP porque agora busca e leitura vêm do cache; os testes também exigem `cached:true` no resultado efetivamente recebido pelo provedor. Não houve redução do aceite para obter passagem.

Os testes de UI verificam texto, pt-BR/en, teclado, IDs desconhecidos, isolamento e rejeição de URL local. Citações são criadas a partir de fontes conhecidas após sanitização; HTML recebido não fornece botões acionáveis. Typecheck, Clippy, rustfmt e diff check passaram. Sem achado bloqueante para renovar os predecessores em seu escopo anterior. A revisão integral de privacidade/memória/histórico do TK-004 permanece pendente.

### Spec

EV-014/EV-018 renovam TK-001/TK-002 com 50 casos offline. EV-015 comprova 90 testes da aplicação e 90 de Codex/gateway/MCP. EV-016 comprova 12 casos com app-server fixado real; EV-017 renova a regressão real de sandbox. TK-001/TK-002/TK-003 liberados novamente para done somente no escopo declarado das revisões anteriores, após atualizar seus estados pelo runner.

EV-019 é parcial: 251 testes de UI, serviço/Host/IPC e checks comprovam controle da web, cache de buscas e UX de fontes. Ainda faltam persistência seletiva/restauração/efemeridade, retenção dos outputs MCP nos rollouts e limite incluindo o registry. AC-008/AC-009/AC-011/AC-012 continuam parciais no TK-004. A evidência não aprova qualidade live ou Windows da versão atual; esses gates continuam no TK-005. Nenhum aceite ou gate foi dispensado.

## Plan r5 — histórico e privacidade parcial (10/10/2026)

Baseline preservado. Diff inclui novo web_history.rs, observer do Host, IDs internos do transcript Codex, DTO e encaminhamento Tauri, UI/IPC/dourado/i18n e casos novos real_app_server/OverlayApp. Nenhum commit ou staged novo; arquivos preexistentes de QA permanecem fora do escopo.

### Standards

Persistência fica no Module aura-app sobre Store existente, com tabela idempotente, chave thread/turn/item e FK em cascata. INSERT exige metadata de Conversa normal e remove snippet; ausência de metadata efêmera impede escrita inclusive ao fechar. IDs de protocolo são internos e não serializados. Abertura mantém role/text legado e publica sources opcional. Erro de escrita resulta em aviso localizado sem conteúdo bruto; não é ocultado em logs. Shell apenas encaminha o DTO. Sem dependência/upgrade de sidecar/Windows API novos. Red/green do consumidor comprovado, com banco real em disco, dois Hosts e processo fixado; UI usa bridge externo e fluxo público. Oráculo literal W1/URL/42, exclusão de W2 e zero HTTP adicional.

As verificações atuais passaram: 50 offline do serviço, 90 app, 90 Codex/gateway/MCP, 12 integrações reais anteriores, sandbox real, 253 UI, types/Clippy/fmt/diff. EV-020/EV-021/EV-022 renovam os predecessores; alterações do transcript não modificam autoridade/aprovações/serviço de busca/leitura. Liberados para done novamente nos escopos já declarados, sem aprovar as pendências do TK-004.

### Spec

EV-024 comprova restauração seletiva após reinício completo do Host e UX sem rede; efêmera não deixa URL, trecho não citado ou rollout em disco. Resultado é parcial: preservação de IDs entre reinícios/Turnos, citação após desativar, isolamento/exclusão de metadata e orçamento de registry ainda pendentes.

Achado bloqueante para done de TK-004: `real_app_server::web::web_normal_research_does_not_persist_uncited_page_content_in_codex_rollouts`, plan r5 Dados/FR-007 e AC-012 — EV-023 falhou porque o fixado persiste página não citada no rollout normal. Estado: aberto, sem redução de oráculo ou aceite. A tabela de citações não resolve esse achado; arquitetura de persistência/runtime precisa ser revisada. Não declarar AC-009/AC-012 concluídos com os testes verdes parciais. TK-004 continua in_progress; TK-005/Windows/qualidade ainda pendentes.

## Plan r6 — renovação após identidade e orçamento (10/10/2026)

Baseline SHA preservado; committed e staged continuam vazios, alterações unstaged e arquivos novos dos esforços 038/039 foram inventariados. QA preexistente permanece fora do escopo. Revisão dos predecessores considera os arquivos de serviço, composição/observer, transcript, schemas e personas já revisados, mais as mudanças de admissão/slot/metadata. Não aprova o TK-004 inteiro.

### Standards

WebContext mantém campos privados e não aceita identidade do modelo. Admissão usa metadata mais entrada mais recente antes de consumir IDs, sem eviction de proveniência. Fechamento remove estado; Drop de slot exige a mesma sessão para não liberar concorrência de outra vida da Conversa. Persistência de números não retém dados de fontes não citadas, e restauração libera lock do Store antes da aplicação no registry. Consulta para citação após desativar não permite rede/cache/ferramentas. Sem dependência, API Windows ou versão de sidecar nova. Testes exercitam seams públicas e literais independentes; casos já verdes são cobertura. Workspace sem shell, 253 UI, typecheck, Clippy sem shell, rustfmt e diff check passaram. Nenhum achado novo bloqueante no escopo dos TK-001/TK-002/TK-003.

### Spec

54 casos offline de serviço passaram: 25 busca e 29 leitura. Os limites de 8/32 MiB agora incluem registry, metadata e crescimento de contextos. Rejeição antes de rede quando metadata não cabe, ausência de fonte/consumo de ID na rejeição e proteção de slots após fechamento estão comprovadas. 16 integrações no app-server fixado passaram em 12,87 s: nove protocolos/modos, comparação, recusa de ação, retomada com cache e quatro casos de histórico/identidade/toggle/efemeridade. A cobertura dos predecessores permanece equivalente à declarada nas revisões anteriores. Liberados para renovar done nesses escopos após evidência atual e regressão sandbox; não é aprovação global dos AC compartilhados.

TK-004 continua parcial: falta comprovar isolamento/exclusão de sua metadata e concluir a arquitetura que evita conteúdo não citado nos rollouts normais. TK-005 permanece sem prova live/Windows atual. Não reduzir metas ou mudar o esperado do teste de retenção para liberar sucessores.

## Plan r7 — metadata comprovada e avaliação independente

Decomposição conserva requisitos/aceites: TK-006 avalia somente busca/extração do WebService; TK-005 mantém dependências TK-004/TK-006, síntese, cadeia determinística e Windows. A retenção normal ainda impede a conclusão do esforço.

Standards: três casos na Interface WebHistory usam SQLite em disco e métodos de ConversationsRepo, sem ler tabelas por fora do Module. Reabertura comprova isolamento de Conversas com o mesmo turn/item/W1. Remoção apaga citações e sequência, recusa gravações tardias e conserva outra Conversa. Restauração pode reentrar no Store e para após lote rejeitado. Já passaram antes de qualquer nova correção; são cobertura, sem red fabricado. Sem alteração produtiva nesta rodada, além dos comportamentos r6 já revisados.

Spec: no r7, 54 casos de serviço, 144 casos de aplicação e 90 de Codex/gateway/MCP passaram; 16 integrações fixadas passaram em 12,37 s. O esperado do gate normal foi preservado e voltou a falhar 0/1 em 0,69 s por conteúdo não citado no rollout. Sandbox real reexecutado; precedentes liberados novamente somente no escopo original após evidências atuais. TK-004 tem provas de metadata/identidade/limites e UX, porém não pode ir para done. O achado de retenção continua aberto e a avaliação independente não o dispensa.

## TK-006 — avaliação backend (10/10/2026)

Baseline fixo preservado. Escopo desta revisão: example quality.rs, quality.md e quality-results-2026-10-10.json novos; os serviços produtivos não foram alterados para passar o gate. Committed/staged continuam vazios; diff e inventário anteriores incluem os arquivos preexistentes sem interferência.

Standards: harness chama WebService/ReqwestTransport reais, sem DNS/HTML mockados, proxy/credencial/contorno de cota. Uma sessão é mantida nas doze buscas. Wrapper externo conta no máximo 60 tentativas, sem modificar parser/budget/registry. Objetivos são consultas públicas fixas, resultado no git é metadata sem snippets/corpos. Alvos inspecionados/fixados antes do primeiro fetch; as duas falhas permanecem no denominador. Execução real, sem red fabricado para experimento de qualidade. Clippy example, 54 regressões offline, rustfmt e diff check passaram. Limite de medição: contador do Aura não mede requests internos do browser usado na preparação independente.

Spec: lista original de doze consultas e domínios manteve-se igual. 11/12 recuperam fonte relevante nos cinco primeiros; consulta Tauri não ganhou aprovação só por conter o domínio certo. Dez HTML foram selecionados antes da extração; 8/10 preservam o literal e origem, acima ou no limiar original. Execução terminou com 58 HTTP/239 s, sem repetir queries/URLs para inflar o resultado. Tauri unsupported_content e ausência do cabeçalho SearXNG no texto são limitações reais, junto com ruído de título em React/SearXNG; não reduzem o limiar aprovado, porém merecem diagnóstico no dono da leitura. TK-006 liberado para done somente no seu escopo backend depois da evidência atual.

Não aprovados: síntese de três jornadas, cadeia determinística/Windows do TK-005, privacidade de rollout normal do TK-004/FD-001 e DPI/resize do esforço 038. Qualidade observada numa amostra não garante SLA ou equivalência geral com ChatGPT/Claude. Esforço permanece ativo.
# Plan r8 — investigação de retenção e fronteira seguinte

Standards: somente documentos e protótipo experimental foram acrescentados nesta rodada, sem alterar código de produção. Prova usa executável fixado, CODEX_HOME/workspace aleatórios isolados, MCP/provedor loopback scripted e literal sintético, sem dumps de dados do usuário ou inferência paga. A cópia preservada do script possui o mesmo SHA-256 do executado. Parser recursivo e scan de bytes são limitações explícitas do protótipo; o ticket exige reconhecimento exato, escopo e inspeção do storage real. ADR 0010 permanece proposto, não aceito pelo sucesso do experimento.

Spec: o modelo recebeu o literal no experimento e o histórico normal abriu com o mesmo ID após reinício, sem o literal integral nos arquivos examinados. Isso justifica testar a fronteira do gateway e não trocar conversas normais por efêmeras. Não demonstra o gate completo de privacidade, URLs descobertas, raciocínio, compactação, isolamento, budget ou adapters reais. O gate existente foi reexecutado sem alterar o esperado e continua falhando. TK-004 permanece bloqueado e agora requer TK-007; TK-005 conserva seus gates. Specs r1 e metas live não mudaram.

Renovação dos predecessores no escopo original: 288 testes Rust dos cinco crates passaram, incluindo WebService54, app144 e Codex/gateway/MCP90. Fixado real16/16 em22,17s e sandbox1/1 em23,02s. Sem mudanças no código de produção após essas execuções. As revisões anteriores do escopo original permanecem aplicáveis; a fronteira nova não é aprovada por essas suítes. O primeiro comando de sandbox usou um filtro inexistente e rodou zero testes: foi corrigido e não constitui evidência de aprovação.

TK-006: a amostra r7 de qualidade continua histórica em quality.md; sua evidência foi invalidada pela revisão do plano e precisa ser renovada antes da convergência final. Não simular uma nova avaliação ao vivo a partir do relatório anterior. Windows/DPI/UI atual ainda pendentes.

## Plan r8 — renovação dos predecessores com entrega transitória parcial

Baseline `ed9a9654c6e76e48186fc010a12eba77023b00f5` preservado; committed/staged vazios, diff unstaged e arquivos novos considerados. Diretórios de QA preexistentes permanecem fora do escopo. Revisão limitada aos escopos originais de TK-001/TK-002/TK-003, sem aprovar TK-007 ou a retenção integral de TK-004.

### Standards

A composição mantém gateway autenticado e MCP como fronteiras externas reais. O Host emite referências somente após admissão no WebService; o provedor recebe os mesmos fatos e proveniência através do gateway, sem reimplementar a transformação no teste. O armazenamento transitório compartilha orçamento e cancelamento com o serviço existente. Nenhum pin, API Windows ou licença foi alterado; getrandom já existe no workspace e o novo dev-dependency do gateway é o crate interno aura-web. As novas funções de entrega, sua formatação e o gate integral permanecem no dono TK-007. Nenhum achado bloqueante novo nos escopos originais dos predecessores.

### Spec

Executados nesta rodada: 304 testes Rust nos cinco crates (app144, WebService61, Codex/gateway/MCP99), 17/17 integrações reais do app-server fixado em 14,59 s, sandbox1/1 em22,42 s, UI253/253 e typecheck. A matriz cobre os nove pares protocolo/modo, recusa de ação, retomada com cache, histórico normal, identidade, toggle e efemeridade. O literal não citado da página agora fica fora do rollout no caso existente. Esse resultado não prova ausência de URLs descobertas e não citadas, argumentos ou raciocínio em storage; o próximo caso deve ler W2 e responder citando somente W1, conservando os fatos recebidos pelo provedor e os oráculos existentes.

TK-001/TK-002/TK-003 podem renovar done somente em seus escopos originais, com evidências atuais vinculadas ao código. TK-007 continua in_progress/partial; TK-004 continua dependente do gate integral. Compactação, inspeção de armazenamento comprimido/SQLite, qualidade live e janela nativa atual ainda pendentes. Nenhuma redução de aceite foi autorizada.

### Oráculo de W2 e fechamento parcial antes de TK-006

Standards: rustfmt foi aplicado e Clippy dos cinco crates/all-targets passou com `-D warnings`; fmt/diff check passaram. O fixture de inferência recebeu resposta final independente do número de páginas lidas, permitindo ler W2 sem citá-la e mantendo todos os casos anteriores. A auditoria Python usa apenas perfil sintético explícito e SQLite mode=ro/query_only; não participa do runtime nem imprime dados privados. Nenhuma correção produtiva foi aplicada para esconder o novo red.

Spec: após a formatação,304 regressões Rust e17 casos originais fixados passaram em13,24s. O novo caso falhou em0,71s (EV050) porque W2 não citada aparece em arguments do rollout e SQLite/thread_items/item_json (EV051). O caso foi excluído somente da renovação do escopo original de TK-003, com sua falha executada e registrada separadamente; permanece obrigatório no gate integral. FD001 continua aberto, agora com causa localizada antes da persistência dos argumentos. TK007 blocked/failed retorna ao desenho de retorno/SSE; TK004/TK005 continuam dependentes. Seguir TK006 independente, conforme autorização, e resolver esse bloqueio ao final. Decompressão não foi exercitada (zero frames), raciocínio e compactação seguem pendentes.

## Plan r9 — predecessores renovados antes da próxima fatia

Standards: baseline e código produtivo preservados durante renovação. Plano9 acrescenta somente desenho da fatia sucessora TK007; schemas MCP foram conferidos no código (search exige objective/queries, fetch exige url). Nenhuma dependência, pin, política de SO ou comportamento dos predecessores mudou nesta rodada. Testes usam serviços internos reais e adapters externos controlados. Nenhum achado bloqueante novo nos escopos originais TK001/002/003; os gates conhecidos de retenção/raciocínio/compactação permanecem no sucessor.

Spec:304 regressões dos cinco crates passaram novamente. Os17casos originais com app-server fixado passaram em13,10s, mantendo todos os oráculos. O caso adicional W2 não foi usado para aprovar o predecessor e permanece falho/obrigatório emTK007. EV055/056/057 permitem renovar done somente nos escopos originais. A qualidade r8 permanece histórica após alteração do plano; sua renovação final não pode ser fabricada a partir do relatório anterior.

## TK007 r9 — revisão da fatia de argumentos e renovação dos predecessores

Baseline continua ed9a9654c6e76e48186fc010a12eba77023b00f5. Committed e staged vazios; mudanças unstaged e arquivos novos private_calls/tool_results/delivery/web/provas foram incluídos. Diretórios de QA anteriores preservados. A revisão desta fatia não aprova o ticket inteiro.

Standards: runtime segue Rust/Tauri,sem dependência nova/upgrade/APIWindows. SchemaMCP fechado preservado; nonce com256bits,kinds separados,admissão/cancelamento no WebService existente. WebTurns captura autoridade antes do await; não resolve contexto atual quando chega SSE tardio. O gateway transforma campos definidos do protocolo,sem procurar substrings de páginas ou expandir usuário/terceiros. Erros/admissão não têm fallback bruto. O fixture observa inputs do provedor externo pela rede e serviço real,sem reproduzir transformação. Testes de W2,Unicode,call_id ambiguo/histórico tiveram reds comportamentais antes da correção. Clippy dos cinco crates/all-targets,fmtdiff passaram; variáveis de comparação no teste corrigem lint sem mudar o literal. Nenhum achado Standards bloqueante novo nos escopos originais TK001/002/003.

Spec:316regressõesRust,10HTTP novos atuais,18/18 integrações fixadas em14,46s. Matriz9 pares,mensagens/fontes/identidade/restart/toggle/cancelamento epermissões existentes preservados. GateW2 ficou verde sem reduzir expected; auditoria externa do perfil pós-fmt examinou86arquivos/128linhasSQLite,sem quatro literais. EV058/059 são parciais porque raciocínio,blob de provedor,compactação ecompressão representativa permanecem sem prova. FD001 continua aberto. EV060/061/062 permitem renovar somente os escopos originais TK001/002/003. TK007 permanece in_progress/partial,sem liberarTK004/TK005; TK006 live r8 fica histórico apósplan9 até renovação final.

## TK-006 — renovação ao vivo r8

Standards: mesma baseline e harness/serviços de produção,sem ajuste de parser/consulta/limiar para passar. Caminhos TEMP novos preservam r7; uma sessão,60 requests/30min,sem conta/chave/rotação para escapar de cota. As fontes foram inspecionadas antes de liberar o manifesto de dez HTML; ele foi publicado atomicamente antes da leitura. Report do repositório contém somente metadata,classificação e literais curtos,sem snippets/corpos; títulos de leitura são limitados com tamanho original preservado. Clippy/fmt/diff e regressões já renovados com código produtivo inalterado desde então.

Spec: execução terminal exit0/complete,58 HTTP/242s. Dez dos doze casos têm fonte do domínio/tema confirmado independentemente; Tauri CLI não responde invoke, e IBGE ficou não verificado por bloqueio da inspeção,sem contar só título/snippet como leitura. O gate10/12 foi atingido mesmo com essa exclusão conservadora. Oito dos dez HTML preservam o literal/origem; Tauri unsupported_content e ausência do cabeçalho SearXNG continuam no denominador. Aprova somente AC001/AC013 do escopo backend de TK006 após nova evidência. Não aprova os gates de síntese,privacidade normal,compactação/Windows eDPI038; FD001 continua aberto. Nenhum aceite reduzido.

## Plan r9 — renovação após proteção de raciocínio

Baseline ed9a9654c6e76e48186fc010a12eba77023b00f5 e alterações locais preservadas. Esta renovação abrange os escopos originais de TK001/TK002/TK003; não aprova o gate integral de TK007. Standards: o novo kind de raciocínio usa a mesma memória limitada do WebService, com autoridade capturada antes do await, expiração, cancelamento e restauração exata na fronteira autorizada. Não há nova dependência, mudança de pin, schema MCP ou API Windows. O ajuste de frescor foi motivado por um red com relógio controlado, sem reconstruir o algoritmo no oráculo. Clippy dos cinco crates/all-targets passou sem avisos.

Spec: 321 testes Rust dos cinco crates passaram; 29 casos dependentes de ambiente permaneceram ignorados nessa execução e não contam como aprovação. A suíte web real separada passou 21/21 em19,07s, incluindo a matriz original de nove protocolos/modos e os novos casos Responses de raciocínio, reinício e compactação. As evidências atuais podem renovar somente os predecessores nos escopos já revisados. EV064 continua partial: campo criptográfico representativo, cobertura adicional de ciclo de vida, regressões finais e revisão integral ainda pendentes. TK004/TK005, live TK006 e QA DPI038 não são aprovados por esta renovação.

## TK007 — revisão final da entrega transitória r9

Baseline fixa ed9a9654c6e76e48186fc010a12eba77023b00f5 conferida; diff baseline...HEAD e staged vazios. Diff unstaged e inventário de arquivos novos incluídos, especialmente tool_results/private_calls/delivery/web e suas provas. Perfis e diretórios QA preexistentes foram preservados. Spec1/plan9 não foram reduzidos; esta revisão não aprova os tickets sucessores.

### Standards

O gateway expande somente envelopes exatos em campos autorizados, sem recursão no conteúdo externo. Threads são vinculados pelo header autenticado e mapa do Host; duplicação de header/nonce/call_id não cria autoridade. O contexto de retorno é capturado antes do I/O. Os três kinds compartilham armazenamento limitado e cancelamento do WebService. SSE mantém UTF-8 entre chunks, snapshots consistentes e erros estáticos sem fallback bruto. O raciocínio original é restituído na fronteira autorizada; summary e deltas privados não chegam ao sidecar/UI. Cancelamento tardio e falta de orçamento foram exercitados no HTTP real do gateway.

Provas usam os módulos internos reais, com variação somente nos processos/provedores, rede e relógio externos. AES-GCM é gerado e decifrado fora do Aura, sem nova dependência/runtime/chave no aplicativo; autenticação adulterada é rejeitada. O scan nativo verifica o ciphertext exato. A auditoria é read-only e limitada a perfis sintéticos; seus ramos comprimidos têm controles positivos independentes. Diagnóstico privado foi verificado com instrumentação realmente habilitada e controle público, inclusive erro upstream contendo conteúdo/nonces. Clippy workspace/all-targets excluindo desktop, fmt/diff passaram. Nenhum achado Standards bloqueante aberto para TK007.

### Spec

AC008/AC014: tool outputs e argumentos originais chegam ao provider pelos três adapters; mensagens/terceiros não autorizam expansão. Matriz original de nove protocolo/modo e recusa de ação continuam verdes. AC009/AC012 no escopo deste ticket: W2 não citada fica fora de argumentos/raciocínio/storage, com resposta W1 e identidade/histórico preservados. O gate original não foi alterado. Novo Turno, reinício e compactação pública foram exercitados nos três formatos, observando o evento Compacted e a abertura do mesmo histórico sem nova rede. O probe criptográfico preserva os bytes e comprova descriptografia independente; 88arquivos/139registros SQLite sem quatro marcadores, além do scan de ciphertext. AC010/AC011/AC012: fechamento/exclusão/shutdown/toggle/cancelamento/TTL e limites continuam cobertos pelas seams Host/WebService; novos casos HTTP de raciocínio impedem autoridade tardia e fallback bruto.

504 regressões workspace passaram (34 ambientais ignoradas, registradas separadamente),25web reais em26,17s,16HTTP private_calls e9delivery. Após a última alteração exclusivamente na fixture de erro do provider,16HTTP afetados e o diagnóstico isolado foram reexecutados com os mesmos oráculos. O probe final também reexecutou a jornada cifrada e esse diagnóstico. Campos criptografados representativos são prova de transporte opaco; não houve inferência comercial nem se afirmou formato proprietário/replay de assinaturas nativas. Nenhum frame comprimido apareceu no Codex; controles gzip/zstd comprovam o auditor separadamente. Essas limitações não dispensam nem contradizem os gates representativos executados.

Gate da entrega transitória satisfeito; ADR0010 pode ser aceito e FD001 resolvido com a evidência atual. TK004 conserva UI/fontes/controle, TK005 integração/síntese/Windows, TK006 renovação live e038 QA DPI. Não concluir o esforço ou a meta global pela passagem de TK007.

## TK004 — revisão após desbloqueio por TK007

Baseline fixa preservada; código de UI/controle/histórico da fatia inspecionado junto aos arquivos novos WebSources/webSources/web_history. Nenhuma alteração produtiva nesta renovação: apenas os documentos históricos foram atualizados para refletir o gate resolvido.

Standards: fontes vêm do DTO/registry confiável e são correlacionadas por Conversa/Turno. Texto/HTML externo não produz elementos de fonte; markdown sanitizado e React escapam títulos. URL do clique permanece HTTP(S), sem credenciais/portas alternativas, e a abertura requer ação do usuário. IPC tipado/dourado/paridade i18n são mantidos. Histórico persiste apenas metadata citada sem snippet, na seam Store/ConversationsRepo real; IDs não são reutilizados após reinício e exclusão recusa gravações tardias. A UI usa a seam de renderização/IPC externa, sem mockar módulos internos. Nenhum achado Standards bloqueante da fatia.

Spec:253testes UI/33arquivos e typecheck atuais passaram. Casos OverlayApp verificam atividade, fontes, distinção snippet/página, pt-BR/en, teclado, histórico sem nova pesquisa e rejeição deHTML/W999/URLfile/fontes de outra Conversa. SettingsApp verifica toggle por teclado, persistência da escolha e texto em inglês. Backend em504workspace/25fixados cobre cancelamento,limites/cache,metadata em SQLite real,identidade,toggle/efemeridade e entrega transitória EV068. Aceites AC008/009/010/011/012 da fatia satisfeitos com os oráculos originais; não houve red fabricado nesta renovação. Build/E2E de pesquisa,síntese/live e QA DPI conservados nos sucessores. Warnings de ownership refletem áreas compartilhadas e execução serial autorizada; nenhuma escrita paralela ocorreu.

TK004 pode concluir com evidência atual. Próxima fatia é renovar TK006 e depois TK005; o esforço e a meta global permanecem ativos.

## TK-006 — gate atual r9

Baseline fixa ed9a9654c6e76e48186fc010a12eba77023b00f5; committed/staged vazios, mudanças unstaged e novos harness/metadados incluídos. Código produtivo e harness inalterados desde as regressões EV-068/069/070/071. Esta rodada altera somente documentação da avaliação e seu relatório sanitizado; nenhuma correção de parser foi usada para mudar o resultado. Package ready foi confirmado antes de iniciar a execução; o retorno ready:false durante in_progress é regra de estado do runner, sem blockers/errors após reconciliar o checkpoint.

Standards: transporte real WebService, uma sessão, doze consultas originais, teto 60 HTTP/30 minutos. Processo confirmado vivo e aguardado até saída terminal, sem repetir buscas quando a observação foi perdida. Dez HTML/literais inspecionados independentemente e fixados antes das leituras. Relatório contém metadados, classificação e literais curtos, sem snippets/corpos/queryContext; caminho TEMP próprio e amostras anteriores preservados. Review inclui os arquivos novos. Nenhuma dependência, conta, licença, pin, API Windows ou segredo foi acrescentado; edição editorial dispensa teste novo conforme testing.md. Nenhum achado Standards bloqueante.

Spec: 10/12 fontes têm domínio e conteúdo relevante confirmados; Museu foi confirmado pelo corpo com horário/dias/última entrada. IBGE continua não verificado após erro/403, sem inferir conteúdo pelo título. Tauri CLI continua tema incorreto. Dez alvos foram fixados antes da extração; 8/10 preservaram literal/origem, mantendo Tauri unsupported_content e cabeçalho SearXNG ausente no denominador. 58 HTTP/422 s, exit0/complete. Gates backend AC-001/AC-013 de TK-006 atingidos. FD-002/FD-003 permanecem menores. Não aprova síntese, E2E Windows, SIWC/comercial, DPI ou equivalência geral com aplicações proprietárias. Próximo TK-005; meta global ativa.

## TK-005 — cadeia determinística, cobertura parcial

Mesma baseline, sem committed/staged; novo web_research.rs incluído integralmente. Produto/predecessores não modificados. Standards: fixtures apenas em variações reais (provedor remoto, HTTP/DNS, SO), processo fixado e Host/gateway/MCP/extração/cache/registry/history reais. DNS virtual não desabilita política de URL. Oráculos independentes literais42/12/30 e URLs, erro blocked, limite da sétima chamada e status Interrupted. Asserções observam eventos/transcript e inputs efetivamente recebidos pelo provedor, sem spies de colaboradores internos. Perfil TEMP próprio e teardown de Host/servidor. Nenhuma dependência, pin, segredo, conta ou execução de internet introduzida. Clippy do target/fmt/diffcheck passaram. Casos já verdes são cobertura, não reds inventados.

Spec:5/5casos atuais após formatação,3,57s. Prova integração de busca/deduplicação/HTML/doisdomínios/fatos/fontes/transcript,fallback429,erro403 com ressalva,cancelamento pendente e limites mesmo com cache. Provedor de síntese é scripted: não prova qualidade LLM comercial nem três jornadas ao vivo. OS é fake: não prova janela Windows. Build demo23928 em execução, sem aprovação antecipada. TK005 permanece parcial até composição da fixture nativa/E2E e jornadas de síntese; gatebackendEV073 e matrizpreexistenteEV068 não substituem essas obrigações. Nenhum achado bloqueante da nova fatia determinística.

## Plano r10 — revisão e renovação antes da composição E2E

Baseline ed9a9654c6e76e48186fc010a12eba77023b00f5 preservada, staged/committed vazios, novos arquivos de testes/experimentos incluídos. A revisão10 detalha somente a composição de QA na feature e2e já existente; spec1, APIs produtivas, pin, dados e política de destino permanecem iguais. Runner marcou os registros anteriores stale; nenhuma evidência foi reaprovada silenciosamente pelo hash. Código dos predecessores não foi alterado durante a renovação.

Standards: workspace sem desktop passou novamente; Clippy workspace/all-targets sem desktop -Dwarnings, fmt/diffcheck passaram. UI253/33arquivos em23,14s e typecheck passaram. Matriz25fixados passou em25,48s. Probe AES256GCM independente executado novamente:preservação/decrypt/tamper e controle diagnóstico públicos passaram; perfil sintético90arquivos/139linhasSQLite sem quatro marcadores. Controle de auditor com gzip/zstd/SQLite foi executado separadamente; ausência de frames comprimidos no perfil continua limitação explícita. Sem novos segredos/dependências/pin/OS APIs. Sem achado Standards bloqueante nos escopos predecessores.

Spec: busca/fallback/normalização/limites, leitura segura/paginação, MCP/9paresprotocolo-modo, identidade/histórico/privacidade/raciocínio/compactação/cancelamento/desativação e UI/fontes continuam passando com oráculos originais. Renovação admite done dos TK001/002/003/007/004 somente após novos EVs, na ordem de dependências. Gate liveTK006 precisa de nova execução no plano10; síntese/Windows/DPI não são aprovados por estes resultados. Composição E2E ainda não implementada, e packageTK005 precisa ready:true antes disso.


## Correção SIWC — plan11 / EV089

Baseline fixa ed9a9654c6e76e48186fc010a12eba77023b00f5; committed/staged vazios, unstaged e arquivos novos do esforço incluídos. Standards: correção restrita a `CodexService::start`/`ensure_loaded`, configuração publicada no pin0.159.0, sem dependência/licença/pin/IPC/SO novo. Oráculos literais na seam pública/processo externo; red vazio/ausente seguido de green real, sem enfraquecer teste. Não interpretar JavaScript arbitrário nem expandir escopo do gateway. Configurações/credenciais do usuário não foram exportadas. Nenhum bloqueante Standards nessa correção.

Spec: AC001/007/008/011/014 continuam cobertos pela regressão, matriz real e três sínteses SIWC do produto. 505workspace/253UI,types/clippy/fmt/diff,5integradas; matriz24passed inicialmente e restante passou na repetição isolada após handshake fechado antes do comportamento. Falha inicial de diretório QA também preservada como ambiente, não red. WCAG e comparação têm leitura/citações correspondentes; Museu tem snippet oficial e ressalva explícita de leitura unsupported_content. O novo resultado resolve FD004, não aprova outras metas de qualidade/UI/DPI/opener. Evidência EV089, relatório synthesis-siwc-plan11-2026-10-10.json; escopo de TK003 pode concluir.

Privacidade renovada no produto corrigido: probe AES-GCM externo, descriptografia independente, tag adulterada rejeitada, reinício/compactação,88arquivos/139linhasSQLite/0marcadores. Controle de diagnóstico público positivo, privado ausente; auditor gzip/zstd separado verde. Nenhum formato proprietário/comercial ou armazenamento comprimido nativo presumido. TK007 necessita EV atual antes de done.

## TK004 renovado plan11 — EV091

Standards: nenhum código UI alterado nesta correção; testes atuais253/typecheck, golden/Host/MCP e privacidade permanecem verdes. Build OS nativa e e2e3/3 (4,5s), perfil próprio e pin0.159.0 real; adapters externos scripted somente, validação/budget/registry reais. Inspeção de native-research.png: seletor visível, resposta42/12/30, W1/W2 clicáveis e metadados de páginas lidas sem sobreposição. Sem novo IPC/SO/dependência e sem achado bloqueante.

Spec: escopo TK004 de fontes/citações/restauração/controle global/cancelamento passa; script comprova nenhum request adicional ao restaurar histórico e após desligar. Ação de opener executada, mas destino no navegador externo ainda é gate específico TK005, não aprovado nesta prova. SIWC real é EV089 separado. Revisão permite done de TK004 com EV091 atual; restante não reduzido.

## TK006 revalidação semântica plan11

Standards: backend/harness fingerprint ef0449ffa2db99720e01f0a64e47ad6e9861a7c9b3e8ae7fb0125c9be58ebc2d é exatamente o da execução EV082; relatórios/lista de doze consultas/dez alvos da spec1 preservados. Plan11 muda somente exposição de ferramentas no consumidor Codex, sem alterar rede/extração/avaliação. A invalidação automática por hash de plan não representa nova falha backend. Nova evidência registra uma revalidação explícita do relatório real da mesma data, sem converter EV082 silenciosamente ou afirmar nova rodada de rede. Não repetir22tools sem mudança relevante, conforme política de testes. Sem achado Standards bloqueante.

Spec: foram recontados12buscas/10leituras,10relevantes/8alvos,58HTTP/193s e thresholds originais. Resultado representa o mesmo backend efetivamente executado em2026-10-10T20:46:18Z. Relatório quality-revalidation-plan11.json com hashes/procedência/limites; síntese é nova execução EV089 separada. A disponibilidade contínua não é garantida. Revisão permite renovar TK006 com esta prova explicitamente classificada; native opener/DPI não aprovados.

## Gate integrado atual — EV094/095

Standards: código de consumidor corrigido sob o pin0.159.0, build OS nativa (sem demo)3m47,505workspace/253UI/typecheck/clippy/fmt/diff atuais; E2E só adapters externos scripted em perfil QA e opt-in existentes. Credenciais não exportadas. Relatório SIWC contém apenas perguntas públicas/answer/metadados; sem corpos de páginas/nonces/log de auth. Nenhuma nova dependência/licença/pin ou API Windows. Sem achado bloqueante Standards no escopo implementado.

Spec: EV094 comprova três jornadas reais ChatGPT gpt-6-luna com leitura útil/citação (WCAG, Python/Rust) ou ressalva de leitura impossível (Museu). EV092/093 revalidam expressamente a medição backend inalterada12/10 (10relevantes/8alvos58HTTP193s), sem alegar nova rede. EV091 native3/3 mostra fontes/histórico/stop/toggle. FD004 está resolved, a falha de entrega opaca desapareceu. EV095 é partial: a ação de abertura foi executada, mas o endereço final no navegador externo não foi observado. AC009 inteiro ainda não aprovado; TK005blocked somente por esse gate. Pergunta manual enviada ao usuário, sem contornar restrição de ferramenta. Esforço permanece ativo.

Fila global atual: 038TK003 continua parcial somente por geometria nativa150/200%; nova resize9/9 é EV015 de038. Total8/10ticketsdone, sem publicação/commit. Processos próprios de QA encerrados após cada teste.

## Auditoria de convergência e produção — 10-10-2026

Check consistency passou nos dois esforços; overlap warnings são áreas serializadas já previstas, não novos tickets. Check convergence detecta AC008sem pass em038, mas sua agregação de passes de AC009em039não cobre a observação final do navegador externo. FD005registra esse limite com ownerTK005/EV095 para impedir conclusão por soma de seams parciais. Nenhum aceite foi reduzido.

Default production cargo check -p aura-desktop e cargo clippy --workspace --all-targets -- -D warnings (incluindo desktop, semdemo/e2e) passaram, EV096. Complementam a execução nativa/real, não substituem navegador/DPI. Build normal sem flags iniciada, sessão77227; frontend passou, aguardar terminal Rust antes de aprovar o executável.

Build normal sessão77227observado terminalexit0,release3m08: frontend/typecheck/Vite e shell Rust completos. Semfeaturesdemo/e2e e sembundle; executável target/release/aura.exe com hash/tamanho em production-binary-2026-10-10.json. EV097registra artefato real. Não publicar/instalar/commitar nem usar a compilação como prova dos gates pendentes.

## Destino externo confirmado pelo usuário — EV098

Standards: procedimento manual permitido, sem contornar o bloqueio do Computer Use nem modificar navegador/configuração. Executável em uso corresponde ao SHA256 de EV097, confirmado nesta rodada. Registro somente de URL pública e confirmação explícita, sem dados de perfil.

Spec: após o roteiro de leitura WCAG e clique W1, usuário enviou URL e respondeu expressamente que clicou na fonte e o navegador abriu esse endereço. Destino https://www.w3.org/WAI/standards-guidelines/wcag/ coincide com fonte. EV098 complementa EV091/095 e resolve FD005; AC009 integral aprovada pelas provas combinadas, não pelo href isolado. Todos os gates TK005 têm evidência atual. Revisão libera TK005 para done. Novo relato de salto de monitor tratado separadamente no esforço040; DPI150/200 de038 permanece pendente.
# Publicação 0.3.0 — reavaliação de metadados em 11/10/2026

Baseline original preservado. A revisão de distribuição está no [esforço 041](../041-publicacao-github-030/review.md), com [comparação exata dos inputs](../041-publicacao-github-030/input-revalidation.json). As EV-087/088/089/090/092/093/096/097 foram invalidadas pelo runner após incrementar a versão. EV-099 a EV-106 registram reavaliação explícita e checks atuais; não convertem evidência histórica em nova execução ao vivo.

Standards: os únicos ajustes em aura-web são User-Agent/clientInfo.version; a reversão desses identificadores em memória reproduz exatamente o SHA anterior de todo o diretório. Algoritmos, consumidores, gateway, política, oráculos e resultados externos permanecem iguais. Cargo/lockfile mudam metadados do workspace, sem atualização do app-server. Checks locais completos e build de produção 0.3.0 passaram.

Spec: o comportamento contratado não mudou. Reavaliação de qualidade/síntese/privacidade usa as medições históricas identificadas e a prova de equivalência, sem afirmar novas consultas ou E2E nativo. AC-014 tem novo build normal, instaladores e verificação de assinatura reais. Disponibilidade contínua da web e DPI150/200 não são aprovados. Sem novo achado bloqueante nos ajustes de publicação.

