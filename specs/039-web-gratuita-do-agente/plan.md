---
schema: hybrid/plan
schema_version: "1.0"
effort_id: 039-web-gratuita-do-agente
revision: 11
spec_revision: 1
status: ready
---

# Plano — web gratuita do agente

## Resumo e contrato consumido

Spec r1, FR-001 a FR-008, AC-001 a AC-014. Novo Module `aura-web` concentra busca, leitura, proveniência, transporte seguro, cache e orçamento. `aura-app` compõe e publica pelo MCP local existente. Runtime permanece Rust/Tauri, sem Python/Node/browser extra instalado. Fontes e pesquisa: `research.md`; estado atual: `state.json` e evidências do runner. A revisão 3 registra busca e leitura efetivas dos TK-001/TK-002. Integração Host, cache de busca e sua agregação com documentos, UI e qualidade seguem nos sucessores.

## Contexto e dependências

Rust 2024, Tokio, reqwest 0.13.5 e url já no workspace. Busca usa dom_query 0.28 para DDG, chrono 0.4 para timestamps UTC e tokio-util 0.7 para cancelamento, com árvore de licenças permissiva verificada por cargo-deny. Extração prevista com dom_smoothie 0.18.2 MIT no TK-002, sem copiar código externo. Verificar licença transitiva/MSRV/Windows antes de adicionar dependência; se não for permissiva, retornar ao plano com alternativa. Context7 indisponível; fontes primárias e código publicado das bibliotecas consultados.

Primário: `https://search.parallel.ai/mcp`, acesso anônimo documentado. Adapter consulta `tools/list` e chama ferramenta de busca publicada, normalizando conteúdo/erros JSON ou SSE. Não enviar Authorization, model_name, chave de provedor ou cabeçalhos herdados. Session ID aleatório estável por Conversa; não rotacionar ao atingir cota. [Contrato oficial](https://docs.parallel.ai/integrations/mcp/search-mcp).

Alternativa: DDG HTML em endpoint fixo HTTPS, até 3 consultas do mesmo objetivo, markup sintético em testes públicos. Sem DDGS/Python, spoof de identidade, CAPTCHA/proxy. SearXNG/fornecedores pagos não entram no runtime desta fatia. Parallel anônimo foi observado ao vivo em initialize/tools/list/tools/call; isso prova disponibilidade pontual, não o gate de qualidade do TK-005 nem disponibilidade contínua.

Seam efetiva do TK-001: `WebService::new(Arc<dyn Transport>, Arc<dyn Clock>)`, `live`, `begin_turn`, `search`, `cancel_turn` e `cancel_conversation`. `WebContext` tem campos privados e capability não desserializável, criada pelo consumidor confiável; o Host fará essa composição no TK-003. Transporte retorna `HttpBody` em stream; a leitura SSE conclui na primeira resposta JSON-RPC correspondente, sem aguardar EOF. Primário recebe até 15 s dentro do teto total de 30 s, reservando tempo para uma alternativa. Cancelamento é verificado antes dela e antes de registrar fontes; sessão anônima da Conversa permanece estável entre Turnos. `refresh` não acrescenta cache nesta fatia: todas as buscas atuais usam rede. Cache e seus limites serão implementados no TK-004.

## Módulos, interfaces e seams

- Novo `crates/aura-web`: interface pública prevista `WebService::{search,fetch,cancel_conversation}` com DTOs de `contracts.md`; adapters `ParallelAnonymousSearch`, `DuckDuckGoHtmlSearch` e transporte HTTP seguro. Sem dependência de Windows/Tauri/LLM.
- Seam rede/DNS/relógio/cancelamento: transporte real com reqwest + adapter de teste com respostas/DNS virtuais/clock controlado; não mockar parser, cache, orçamento ou registry. Teste público `WebService` atravessa essas regras.
- Existente `crates/aura-app/src/host.rs`, `Host`, `HostConfig`: compor serviço, resolver Conversa/Turno corrente com dados confiáveis e sinais de interrupt/close/shutdown. Novo `crates/aura-app/src/web.rs`: integração de ferramentas/fontes, sem lógica de rede duplicada.
- Existentes `aura-mcp::tools::all`, `ToolHandler`, `HostTools::call`: ferramentas `web_search`/`web_fetch` do Aura com schemas fechados, `readOnlyHint=true`, `openWorldHint=true`, `destructiveHint=false`. Não reutilizar helper read_only atual com openWorldHint=false. Contexto existente só tem Conversa; Host resolve Turno ativo, nunca confia no argumento remoto para orçamento/identidade.
- `aura-codex::home::BaseConfig`, persona pt-BR/en, `mapping`, `aura-gateway`: busca hospedada nativa desativada para rota gratuita; manter MCP e aprovações/sandbox atuais. Validar namespace e tool_search no fixado; não depender de atualização.
- `HostEvent`, IPC, `Messages`, redutor de Conversa, Configurações: novo evento/fontes/pref. `WebSource` e `webEnabled` previstos, serialização camelCase e dourado ADR 0009. UI usa fontes de registry, não HTML remoto.

Correlação escolhida para esta versão: novo registro de Turnos ativos do Host alimentado por eventos confiáveis do app-server. Header opcional só é aceito se corresponder a Turno ativo registrado. Sem header, usar contexto somente quando há exatamente um Turno ativo no Host; zero ou múltiplos retornam erro de contexto antes de cache/rede. Não inferir contexto da Conversa selecionada na UI. Isolamento falha fechado; futuras pesquisas simultâneas de Conversas diferentes exigem correlação por chamada comprovada e revisão do plano. Duas requisições do mesmo Turno continuam permitidas. AC-012 e contracts.md já exigem contexto confiável e não autorizam chamada sem orçamento.

Detalhe de integração TK-003 (r4): `CodexService` oferece observador síncrono de eventos antes do broadcast; Host atualiza registro por thread/UUID local/turnId e cria WebContext ali, evitando atraso ou perda da fila UI. Fim remove somente Turno correspondente; interrupt/close/shutdown cancelam antes de aguardar processo. Header existente X-Aura-Conversation corresponde a UUID local registrado, não a argumento do modelo. Não usar active_turns fallback do Codex para orçamento: sua resposta turn/start pode chegar após eventos de conclusão, enquanto o registro do Host segue a sequência de eventos confiáveis.

`WebAccess`/slot Weak seguem o padrão existente de HostTools, com Host implementando a chamada e serviço `Arc<WebService>`. `HostConfig.web` opcional injeta serviço real construído sobre adapters DNS/HTTP/clock externos controlados; None compõe live. Dependência aura-web no Cargo de aura-app e campo None no shell Tauri são encaminhamento necessário, sem comportamento de SO. Seam determinística combina ToolHandler/Host real e HTTP do router MCP; prova integrada de composição/namespace continua com app-server fixado e upstream scripted. Não mockar WebAccess do Host, registry ou orçamento nos testes de comportamento.

## Transporte, extração e orçamento

DNS resolve antes de conexão, rejeita respostas mistas com endereço não público; pin de IP validado mantendo hostname TLS/SNI. Checar IPv4/IPv6 incluindo mapped IPv4, loopback, link-local, privados, multicast, unspecified, reservados/documentação/metadata. Rejeitar nome local/IP literal proibido antes de rede. Redirect automático desativado, cada Location e resolução validadas, no máximo 3. Sem proxy ambiente/sistema, cookies, Referer automático, credenciais ou TLS inseguro; User-Agent do Aura. HTTP apenas porta 80, HTTPS 443.

Para testes positivos da política, DNS/transporte virtuais representam hosts públicos sem internet. Servidor loopback para protocolo MCP do próprio Aura não equivale a permitir URL privada em web_fetch. Nenhuma feature de teste enfraquece validação da build de produção.

Streaming conta bytes descomprimidos até 2 MiB, timeout compartilhado entre handshake/busca/fallback/redirect/leitura. Pedido cancelado aborta/corta resultado tardio, não faz fallback/cache. Extração HTML em tarefa de CPU com limite DOM e entrada; saída texto estruturado, sem HTML executável. Páginas técnicas sem artigo usam fallback DOM conservador com heading/list/table, marcando extractor; evitar depender só de strip tags. Não aceitar aviso CAPTCHA como conteúdo útil.

Detalhes efetivos do TK-002: reqwest decodifica gzip/brotli/deflate antes do stream contado; codificação desconhecida é unsupported_content. `encoding_rs` 0.8.35 interpreta BOM ou charset HTTP declarado, default UTF-8, rejeitando erros/controles binários. `dom_smoothie` fixado 0.18.2, modo Markdown e teto 20.000 elementos; conteúdo curto usa primeiro main/article/body sanitizado, sem duplicar main e article aninhados. Markdown é texto estruturado externo, não HTML a executar. Fixture preserva o fato literal com pontuação escapada em Markdown. Login sem conteúdo principal, shell JS vazio, PDF/binário e desafio identificável têm erro explícito; essas detecções não prometem reconhecer todos os bloqueios de sites.

Continuação exige versão opaca do documento; versão ausente/divergente/expirada/evictada não baixa outra página para juntar trechos. Cache de documentos já é necessário à paginação no TK-002: 15 min, LRU 32 documentos/8 MiB por Conversa, 32 MiB agregados de documentos, só memória e cancel_conversation limpa. TK-004 agrega buscas e registry ao orçamento e comprova desativação/isolamento/histórico. Limites de ferramentas de leitura (6) e concorrência (2 compartilhadas com busca) são contados antes de cache. Testes usam byte budget independente: 23 documentos de 400.000 bytes excedem 8 MiB antes de 32 entradas; 5 Conversas com 18 documentos cada excedem 32 MiB sem exceder teto individual.

Deslocamentos Unicode e paginação sobre versão de documento no cache; hash/token de versão evita misturar páginas mudadas. 20.000 caracteres por chamada/100.000 por documento; memória LRU por Conversa 32 entradas/8 MiB/15 min; limite global do serviço 32 MiB com evicção para várias Conversas. Budget confiável por Turno, sem expor turnId aceito do modelo. Limites da spec retornam erro estável.

## Rota do agente e qualidade

O agente atual escolhe consultas, abre fontes primárias, refina quando faltam dados e cruza domínios em comparações. Persona e descrições indicam quando pesquisar, ler e citar; sem loop LLM paralelo ou segundo modelo pago para resumir páginas. Cache/snippet nunca tratado como leitura integral. Erros/bloqueios são explicados, não escondidos por resposta confiante.

Registry por Conversa emite W1, W2 etc., reutiliza ID para mesma URL segura e mantém metadados de leitura. Citações validadas com marcador `[[aura-source:W1]]` convertido em link/texto seguro somente se ID existe; desconhecido vira referência não verificada, sem badge. Markdown comum continua sanitizado; painel de fontes oferece auditoria, não garante veracidade de toda síntese LLM.

## Dados e compatibilidade

`Settings.webEnabled` novo booleano, default true, compatibilidade de configurações antigas via serde default, nenhuma migração SQL nova se storage existente JSON suportar. UI copy deixa claro que consulta/objetivo e IP chegam ao serviço; nunca envia conversa inteira. Desativação cancela tudo e impede cache/tools/busca nativa. Pref. separada das regras de captura e do network de Tarefa.

Conversas normais persistem somente fontes citadas/URL/título e trechos efetivamente incluídos na resposta, usando storage/transcript já existente, não páginas inteiras em banco/logs. Efêmeras não persistem fontes/conteúdo. Cache some ao fechar, apagar, shutdown. Histórico precisa restaurar fontes da resposta sem nova rede. Expandir contrato público IPC exige dourado Rust/TS.

Detalhe efetivo do TK-004 (r5): novo `aura-app/src/web_history.rs` guarda somente metadados das fontes citadas em `web_citations`, via Store existente e tabela idempotente do Module. Chave thread/turn/item e FK com exclusão em cascata; INSERT condicionado à existência da Conversa normal impede gravação em efêmera. Snippets ficam vazios na metadata persistida; o trecho efetivamente usado já está no texto da resposta. `CodexService::open` fornece IDs internos não serializados para correlação; o DTO do Host acrescenta `sources` opcional, preservando a forma role/text quando vazio. Shell somente troca o tipo encaminhado. Red/green atravessa Host e app-server fixado real, banco em disco e reinício completo do Host; UI abre histórico pelo fluxo público e ativa link pelo teclado sem enviar nova mensagem.

Detalhe efetivo do TK-004 (r6): `web_source_sequence` guarda somente o maior número de fonte emitido por Conversa normal, inclusive para fontes não citadas, sem guardar suas URLs/títulos/trechos. A restauração aplica uma linha de citações por vez e libera o lock do banco antes de chamar o registry; falha de leitura ou de admissão bloqueia ferramentas daquele Turno. IDs citados são restaurados e números anteriores nunca são atribuídos a outra URL após reinício. A UI repõe as fontes citadas no registry da Conversa ao carregar o histórico. A consulta de metadata para citações é separada da autoridade de ferramentas: desligar web preserva a possibilidade de citar o que já foi consultado, enquanto rede e cache continuam bloqueados.

Registry, metadados de sessão/Turno e entradas de busca/documento compartilham os tetos de 8 MiB por Conversa e 32 MiB no serviço. A contagem inclui capacidade das strings, vetores e margem conservadora das estruturas; é orçamento de dados retidos pelo serviço, não medição do RSS do processo. Antes de admitir fontes, simular as alterações e exigir espaço para metadata mais a entrada mais recente. Evictar somente entradas dispensáveis do cache; rejeitar crescimento de metadata com `limit_exceeded`, sem consumir novo ID ou reatribuir IDs antigos. A criação/crescimento do contexto também verifica o orçamento antes de reter estado. Fechar/excluir remove todo o estado e cancela I/O; slots antigos só liberam concorrência se a sessão da Conversa ainda é a mesma. Verificações atravessam WebService e, para persistência, Host/app-server fixado e Store real.

Pendência comprovada, sem dispensa de contrato: no fixado rust-v0.159.0, os rollouts de Conversas normais persistem outputs das ferramentas, incluindo conteúdo web não citado. O teste público `web_normal_research_does_not_persist_uncited_page_content_in_codex_rollouts` falhou por esse comportamento; não é falha de ambiente. Ephemeral não gera rollout nem grava URL da fonte nos arquivos locais. Guardar somente citações no banco do Aura não resolve a retenção pelo sidecar. Revisar solução de persistência/runtime antes de concluir TK-004; não alterar somente o flag de histórico estendido ou sanitizar depois da escrita e alegar ausência de persistência. Não atualizar o sidecar sem ticket próprio. As novas provas de identidade, citação após toggle e memória não dispensam esse gate nem o de isolamento/exclusão da metadata.

### Revisão 8 — entrega transitória na fronteira do gateway

`persistence-research.md` registra prova executada com o fixado: resultado MCP opaco, expansão em memória somente para o provedor scripted, turno completo e mesmo histórico após reinício, sem o literal integral da página nos arquivos examinados. Não é implementação nem aprovação da retenção integral. ADR 0010 proposto complementa a passagem do gateway para envelopes privados; mantém threads normais e operações existentes.

TK-007 implementará a fatia de entrega antes de TK-004. Novo `aura-gateway/src/tool_results.rs` define interface de resolução; Routes e responses recebem resolver opcional. Header `thread-id` da requisição autenticada vincula a leitura à Conversa; `function_call` anterior, namespace `mcp__aura`, nome web_search/web_fetch e callId identificam o único output elegível. Envelope JSON exato/versionado com nonce de 256 bits, sem página/metadados, substituído uma vez em blocks de texto de function_call_output; nenhuma busca por substring, expansão em mensagens/arguments ou recursão no conteúdo externo. Requisições sem envelope preservam passagem. Expansão acontece antes dos adapters Responses/Chat/Anthropic e nunca entra em dumps.

Host/WebTurns emite o envelope só após admissão do resultado serializado no cache contabilizado de WebService. Entregas transitórias contam junto com entradas e registry nos tetos atuais, sem storage paralelo não contabilizado. Vínculo inclui thread, Turno e sessão; expiração <=15 min, conclusão/cancelamento do Turno, toggle false, fechar/excluir/shutdown removem os resultados. Retries ativos lêem sem consumir. Conteúdo anterior indisponível recebe erro de dado antigo e permite nova pesquisa autorizada; escopo divergente bloqueia antes de upstream. Falha de admissão nunca devolve conteúdo bruto como fallback. Histórico de mensagens e citações continua sem rede; cache de pesquisa permanece sujeito à política original.

Validação: HTTP real do gateway + Host/WebService, dados literais e inspeção completa do CODEX_HOME isolado; reinício, novo Turno, compactação, isolamento, toggle, orçamento e três protocolos. Examinar também URLs/snippets descobertos não citados em argumentos/raciocínio: se ainda forem gravados, a solução permanece parcial e retorna ao plano. Nenhum oráculo ou aceite foi reduzido. Python existe somente no protótipo descartável, fora do runtime. A proposta não altera código existente nem reaprova evidência anterior; renovar evidências afetadas pelo plano antes de implementação.

### Revisão 9 — argumentos privados antes do sidecar

EV050/051 demonstram que outputs protegidos deixam a URL não citada nos argumentos da chamada devolvida pelo provedor. O protótipo `experiments/private_arguments_probe.py --schema-conforming`, com o executável fixado, demonstrou chamada MCP, argumentos originais restituídos ao provedor e mesmo thread/resposta após reinício sem os quatro literais sintéticos em arquivos/SQLite. A variante `--reasoning --schema-conforming` demonstrou somente encaminhamento de raciocínio sintético e campo opaco; não prova blob criptográfico real, compactação ou orçamento/autorização de produção. Relatório sanitizado em `experiments/private-arguments-schema-result.json`.

A próxima fatia do TK007 protege argumentos `web_search`/`web_fetch` no retorno SSE normalizado dos três adaptadores, antes de entregá-los ao Codex. Novo `aura-gateway/src/private_calls.rs`: parser de frames com UTF-8 preservado entre chunks, limite explícito de 8 MiB por frame e 32 chamadas registradas; correlacionar item_id/output_index/call_id/namespace, reter somente argumentos completos, ocultar deltas privados e emitir o mesmo envelope nos snapshots added/done/completed. Não procurar substrings nem substituir mensagens do usuário, argumentos de outras ferramentas ou respostas finais. Repetição inconsistente de snapshot/ID falha com erro estático, sem conteúdo bruto. Erro de stream nunca repassa mensagem do provedor contendo dados privados. Falha de admissão/escopo também interrompe sem fallback bruto.

Envelope de argumentos respeita o schema MCP existente: objeto com somente `url` (web_fetch) ou `objective` e `queries` (web_search), string `aura-tool-arguments:v1:` mais nonce de 256 bits; no search, objective contém a referência e queries tem exatamente a mesma string como único elemento. O Host resolve a referência antes de desserializar o DTO e antes de rede/cache, usando o contexto confiável do Turno. Na próxima requisição Responses, somente function_call do namespace do Aura permite restituir os argumentos ao provedor. Referência antiga recebe argumento de indisponibilidade, sem rede automática; outra Conversa/ferramenta/kind é rejeitada. Conteúdo de página continua externo e as permissões/sandbox permanecem intactas.

`ToolResultResolver` ganha resolução de argumentos e captura de uma interface `ToolCallProtector` antes do await de upstream. O adapter WebTurns guarda WebContext/thread emissor naquele instante; emissão tardia após cancelamento/novo Turno/toggle nunca busca a autoridade de um Turno mais recente. `WebService` retém argumentos no mesmo mapa/budget de entregas, diferenciando kind de resultado/argumentos; retenção/cancelamento/TTL/limites atuais permanecem. Nenhuma dependência nova. A proteção suprime dumps para requisições com referências ou Turno privado ativo; respostas HTTP/stream têm erros estáticos.

Seams/oráculos: gateway HTTP com upstream SSE externo controlado e WebService real, preservação literal de argumentos no provedor e ausência literal no stream entregue ao Codex; fragmentação em cada byte, chamadas intercaladas, snapshots repetidos, falso nonce, namespace alheio, escopo divergente e cancelamento enquanto upstream aguarda. Host/MCP real + app-server fixado: o teste W2 já falho mantém todos os oráculos e deve passar após a integração. Executar a matriz original de 17 casos e a nova regressão, depois inspecionar bytes e SQLite do perfil sintético. Nenhum protótipo substitui essa prova.

Raciocínio continua obrigação integral do TK007: a fatia seguinte retém o item original/summary/campo opaco em memória no mesmo budget, oculta seus deltas e reidrata somente o item autorizado antes do provedor. Campos criptográficos reais, expiração/continuação, compactação, novo Turno/reinício e todos os caminhos de retenção precisam de evidência representativa. Não concluir TK007/TK004 apenas com argumentos protegidos. A implementação segue um comportamento por vez; não reduz spec r1, gates, ferramentas públicas, identidade do thread nem capacidades dos provedores.

## Fatias e mapa de alterações futuras

1. TK-001 busca gratuita e alternativa: novos `crates/aura-web/{Cargo.toml,src/lib.rs,src/search.rs,src/transport.rs,tests/search.rs}`; workspace Cargo.toml/Cargo.lock. `contracts.md` é contrato previsto, não API existente.
2. TK-002 ler páginas públicas: novos `src/fetch.rs`, `tests/fetch.rs`, fixtures HTML sintéticas; reforça transporte real seguro e continuação.
3. TK-003 agente/modos: existentes `crates/aura-app/src/{host.rs,tools.rs,lib.rs}`, novos `web.rs`, `tests/web_tools.rs`; `crates/aura-mcp/src/tools.rs`, `crates/aura-mcp/tests/protocol.rs`; `crates/aura-codex/src/{home.rs,mapping.rs,service.rs}`, `persona/{pt-BR,en}.md`; testes gateway/real_app_server conforme compatibilidade observada.
4. TK-004 UX/fontes/desativação: existentes `aura-core/src/settings.rs`, `aura-app/src/events.rs`, `aura-app/tests/ipc_contract.rs`, `apps/desktop/src/{ipc,state,overlay,settings,i18n}`; novo `overlay/WebSources.tsx`, seam UI via OverlayApp. Preferência em `settings/General.tsx`, `GeneralSection`, com esclarecimento de privacidade; não duplicar editor de MCP ou regras de captura.
5. TK-005 prova integrada: novos `aura-app/tests/web_research.rs`, `apps/desktop/e2e/specs/web-research.e2e.ts`; complementa `tests/real_app_server.rs`. Auditoria/diagnóstico só metadados redigidos. Serializar dependências para evitar sobreposição de writes; nenhum trabalho paralelo requerido.
6. TK-006 avaliação independente de busca/extração: novo `crates/aura-web/examples/quality.rs`, lista/oráculos em `tdd.md`, relatório de relevância/extração em `quality.md`. Depende de TK-003 para consumir os serviços e limites já integrados. É extração da avaliação backend antes prevista em TK-005, não um novo requisito. TK-005 continua exigindo TK-004 e passa a exigir também TK-006; conserva jornadas de síntese, prova determinística integrada, build/E2E Windows e convergência final. A retenção normal pendente no TK-004 não impede observar relevância/extração do WebService isolado, mas continua impedindo concluir TK-004/TK-005/esforço.
7. TK-007 entrega transitória de outputs web: gateway/Host/WebService e testes públicos, conforme seção r8 e ticket. Depende de TK-003; TK-004 passa a requerer TK-007. É solução da retenção já exigida, sem novo comportamento de produto, provedor ou redução de gate. Áreas compartilhadas são executadas em série.

Revisão 7: somente decomposição de execução para cumprir a orientação de avançar trabalho independente deixando bloqueios para o final. Nenhum limite, aceite ou oráculo foi reduzido. O harness usa WebService e transporte de produção, sem bypass de destinos e sem inferência paga. Instrumenta somente a variação externa de transporte para contar no máximo 60 requests HTTP; mantém uma sessão externa e prazo de 30 minutos. Uma busca por cada uma das 12 consultas fixas, resultado avaliado nos primeiros cinco; selecionar/inspecionar dez HTML estáticos e fixar seus alvos antes da leitura pelo extractor, sem descartar falhas estáticas. Artefatos gravam somente URLs, títulos, avaliação, contadores, tempos, extractor e limitações; corpos completos permanecem fora do git. A síntese de três jornadas permanece em TK-005.

## Estratégia de verificação

### Revisão 11 — entrega de ferramentas ao ChatGPT com code mode

EV086 registrou a falha de síntese real: páginas foram lidas, mas o modelo recebeu referências opacas. Um observador loopback descartável, que encaminha bytes sem modificá-los e registra somente tipos/chaves/comprimentos, confirmou `custom_tool_call`/`custom_tool_call_output` do code mode. O gateway protege/restaura chamadas MCP diretas, não JavaScript arbitrário nem resultados aninhados de exec. A disponibilidade do serviço e a conclusão do Turno não aprovam a síntese.

Decisão técnica reversível do TK-003: `CodexService::start` e `ensure_loaded` impõem `features.code_mode.direct_only_tool_namespaces=["mcp__aura"]` nas configurações de `thread/start` e `thread/resume`. É a mesma fronteira pública já usada para desativar busca hospedada e correlacionar a Conversa. A lista deste namespace interno é controlada pelo Aura; override do chamador não pode removê-lo. Não desativar code mode das outras ferramentas, substituir modelo/provedor, atualizar pin, alterar schemas MCP/IPC ou ampliar resolução de referências no gateway. Modos e permissões continuam com seu contrato atual. Retomada reaplica a configuração também a threads anteriores.

Compatibilidade verificada no código oficial de **rust-v0.159.0**: [CodeModeConfigToml](https://github.com/openai/codex/blob/rust-v0.159.0/codex-rs/features/src/feature_configs.rs) declara a lista; [apply_direct_model_only_namespace_overrides](https://github.com/openai/codex/blob/rust-v0.159.0/codex-rs/core/src/tools/spec_plan.rs) expõe esses namespaces diretamente e os exclui da superfície aninhada. Context7 não está disponível nesta sessão; a verificação usa a fonte primária da versão fixada. Um override temporário por thread com essa única diretiva produziu, no ChatGPT conectado/gpt-6-luna, chamada MCP direta e resposta correta/citada à WCAG em 7,766 s. Relatório `experiments/code-mode-delivery-probe-2026-10-10.json`. Isso é prova do protótipo, não da implementação nem das três jornadas.

Alternativas descartadas: desligar flags globais não prevalece sobre `tool_mode=code_mode_only` do catálogo nativo; trocar catálogo via config por thread não alterou o transporte no experimento; interpretar JavaScript/results aninhados ampliaria a autoridade da expansão e não protegeria argumentos privados no mesmo seam. Nenhuma dependência nova ou alteração do catálogo global.

TDD na seam pública `CodexService`: observar requests do processo externo controlado, exigir a lista literal no início mesmo com override vazio e na retomada após parada ociosa. Primeiro red por chave ausente, depois implementação mínima; testar um comportamento por vez. Regressão `cargo test -p aura-codex --test service`, Host/MCP/gateway e matriz real fixada; privacidade de threads normais/reinício/compactação mantém os oráculos anteriores. Build nativa e repetir as mesmas três perguntas reais de `tdd.md` sem o override experimental. Usar instalação normal completa do pin, incluindo code-mode-host; aguardar prontidão, sem `AURA_CODEX_BIN` incompleto. Não gravar credenciais/corpos de páginas nos relatórios.

G2: contrato/spec r1 preservado, consumidor/configuração suportados comprovados; TK-003 é dono da correção, TK-005 conserva o gate de síntese, fonte/opener e convergência. Invalidar evidências afetadas pela revisão e renovar após execução; resultado backend não substitui síntese, nem prova a UI/DPI. FD004 permanece aberto até as três jornadas passarem com o produto corrigido.

Revisão 10 — composição da prova nativa TK-005. A feature `e2e` existente do shell é o limite de compilação: novo `src-tauri/src/e2e_web.rs` contém somente o adapter DNS/HTTP externo sintético e aplicação à `HostConfig`. `setup` chama essa composição apenas com feature e2e e `AURA_E2E_WEB=1`; demo sem esse opt-in e produção sem e2e preservam o comportamento existente. Usar `HostConfig.web` já aprovado, WebService/validação/cache/registry reais, respostas do protocolo Parallel e HTML sintético42/12/30 em news.example e independent.example. Nenhuma conexão pública real é necessária. Rota pública `/pending` retorna stream que não termina para provar cancel/toggle; `/blocked` retorna403. URLs privadas continuam recusadas pelo mesmo serviço antes do adapter. Não criar comando IPC de injeção, nem eventos UI artificiais, nem bypass de DNS/porta na produção.

No opt-in e2e, `CodexRuntime::Fake` vira Binary com `AURA_CODEX_BIN` explícito do pin verificado; configuração Binary existente mantém seu SpawnHook. A fixture não instala/baixa versões nem substitui supervisor/gateway/MCP. `aura-web` e `futures` do workspace são dependências opcionais ativadas apenas por e2e, sem biblioteca nova. Shell somente compõe adaptadores; orquestração continua aura-app. Log de fixture, se usado, é somente contador/URL sintética em perfil QA próprio, nunca corpo/chave de provedor.

E2E novo usa upstream Responses loopback de `e2e/responses.ts` e decisões scripted apenas na variação do provedor. Pedidos pela UI; atividade e fontes observadas no DOM renderizado; respostas/transcript pela API pública; abrir fonte por ação de usuário e cancelar/desativar durante uma leitura realmente pendente. Perfis em TEMP isolados, WebDriver existente compatível, build `pnpm -C apps/desktop tauri build --no-bundle --features demo,e2e`; manter também compilação e2e com OS nativo quando necessário. Fixtures determinísticas não aprovam três jornadas de síntese ao vivo, SIWC real nem DPI. Esta revisão só detalha seam de QA já exigida, sem mudar spec r1/contrato/runtime produtivo; classificar impacto e renovar evidências afetadas, preservando as anteriores.

Cada AC → casos em `tdd.md`, oráculos literais independentes, red/green um comportamento por vez. O TK-001 já tem testes na seam efetiva; resultados e limites de cobertura ficam nas evidências do runner. Rede/DNS/clock/processo falso apenas nas variações reais; parsing/proveniência/budget reais. Síntese determinística usa respostas scripted do processo externo com marcador literal, não avaliação LLM usada como único oráculo.

Comandos da raiz, executados conforme o target de cada ticket existir:

```powershell
cargo test -p aura-web --test search
cargo test -p aura-web --test fetch
cargo test -p aura-app --test web_tools
cargo test -p aura-mcp --test protocol
cargo test -p aura-app --test web_research
cargo test -p aura-app --test ipc_contract
pnpm -C apps/desktop test
pnpm -C apps/desktop typecheck
cargo test --workspace --exclude aura-desktop
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo deny check licenses
pnpm -C apps/desktop tauri build --no-bundle --features demo
pnpm -C apps/desktop/e2e test -- --spec ./specs/web-research.e2e.ts
```

Dourado após formato IPC: definir `$env:UPDATE_GOLDEN='1'`, executar `cargo test -p aura-app --test ipc_contract`, remover env pelo PowerShell e rodar teste normal/TS. App-server real: definir AURA_CODEX_BIN para o executável fixado já verificado, executar `cargo test -p aura-app --test real_app_server -- --ignored --nocapture` com provedores upstream scripted. Não imprimir dumps brutos/segredos. Live smoke conforme tdd com consultas públicas e limite; não parte de CI determinístico. Se cargo-deny/WebDriver/conta faltarem, registrar not_run específico, não aprovação.

## Obrigações técnicas e gates

- **OT-001** → FR-001/FR-002, AC-001/AC-002: separar endpoint gratuito de API autenticada; nenhuma credencial herdada ou troca de sessão para escapar de limite.
- **OT-002** → FR-004, AC-006: validar destino até conexão real, redirects e DNS pin; teste positivo e adverso, sem bypass de produção.
- **OT-003** → FR-005/FR-006, AC-007/AC-008/AC-009: fonte real identificável, marker validado, conteúdo externo não vira instrução; UI não exibe HTML recebido.
- **OT-004** → FR-007, AC-010/AC-011/AC-012: autorização atual a cada chamada, cancelamento, isolamento e budget por Turno confiável.
- **OT-005** → FR-008, AC-013/AC-014: fixtures não provam qualidade ao vivo nem interoperabilidade do fixado; exigir ambas.

G1/G2 documental completo, sem garantia do serviço gratuito. G3 só primeiro ticket sem blockers; demais exigem predecessor done e pacote atual. Mudança de acesso anônimo/contrato público, necessidade de JS ou aprovação de custo volta à planejadora; não reduzir metas para aprovar entrega. Qualidade depende do modelo, mas fontes/rota/limites devem permanecer verificáveis.
