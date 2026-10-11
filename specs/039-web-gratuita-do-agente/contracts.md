# Contratos previstos — web do Aura

Consumido por plan r3/spec r1. Busca, leitura, contexto privado e cache de documentos já existem em `aura-web`; ferramentas MCP, cache de busca, UI e IPC seguem nos tickets sucessores. Nomes finais podem ajustar-se localmente, preservando semântica/limites e IPC camelCase. Tool schemas fechados: rejeitar campos desconhecidos que tentem injetar contexto, headers ou URLs de backend.

## Identidade e autoridade

Conversação/Turno/identidade vêm de Host e contexto autenticado MCP. Não aceitar `conversationId`, `turnId`, `sessionId`, cookies, Authorization ou backendUrl como argumento da ferramenta. Serviço mapeia Conversa a UUID externo aleatório estável; modelo nunca escolhe sua rotação. Se não houver Turno confiável, erro de contexto, sem efetuar web anônima fora do orçamento.

Resolver pelo registro de Turnos ativos confiável do Host: header opcional deve corresponder a registro ativo; ausente só resolve quando existe exatamente um Turno ativo. Zero ou múltiplos sem correlação produzem `invalid_input` com mensagem de contexto indisponível, antes de cache/rede. Nunca usar thread selecionada no Overlay. Testar duas Conversas ativas sem header e garantir zero fonte/cache/rede atribuído à Conversa errada.

## Busca

`web_search({objective, queries, maxResults?, refresh?})`.

- objective: texto 1–2.000 caracteres; queries: 1–3 textos de 1–200; maxResults: inteiro 1–10, default 5; refresh: boolean default false.
- Resultado: `{provider, queryContext, results, searchedAt, cached, degraded, warnings}`.
- Fonte: `{sourceId, title, url, snippet, publishedAt: string|null, retrievedAt, kind: "searchSnippet"}`. `publishedAt` só com metadado verificável, não inferido de idade/consulta. `url` validada HTTP(S), sem userinfo/porta proibida.
- URL dedupe: normalizar hostname/case/porta padrão/fragmento e remover parâmetros `utm_*`; preservar outros parâmetros/caminho. Ranking mantém primeira ocorrência e ordem do backend; múltiplas consultas alternativas intercalam deterministamente com dedupe, sem inventar score.
- `results=[]` é sucesso vazio, não fallback obrigatório. Erro de primário passa ao alternativo somente para disponibilidade/bloqueio/rate limit; erro de input/disabled/cancel não tenta.
- Adapter Parallel observa schema `tools/list` e encaminha somente objetivo, consultas e sessão aleatória. Adapter DDG usa mesmas consultas, normaliza resultado sem executar markup. Schema mínimo observado em `tests/fixtures/parallel-tools.json`; não presumir JSON da API paga.

## Leitura

`web_fetch({url, startChar?, maxChars?, documentVersion?, refresh?})`.

- URL validada conforme spec; startChar inteiro >=0 default 0; maxChars inteiro 1–20.000 default 20.000; documentVersion opcional para continuação; refresh boolean.
- Resultado: `{sourceId, requestedUrl, finalUrl, title, contentType, extractor, text, retrievedAt, publishedAt: string|null, startChar, endChar, nextStartChar: number|null, documentVersion, truncated, documentTruncated, cached, warnings, externalContent: true}`.
- Índices por caracteres Unicode, endChar exclusivo. `nextStartChar=endChar` se há mais dentro de versão cacheada; no fim null. `documentTruncated` distingue teto 100.000 de paginação 20.000. Versão diferente/expirada durante continuação retorna erro explícito para reiniciar, não mistura documento.
- Somente HTML/texto simples suportados; conteúdo principal em texto estruturado com headings/listas/tabelas, sem HTML/script ativo. MIME ausente permite sniff conservador; MIME binário/PDF rejeitado.

## Erros e lifecycle

`ToolOutput::error(code, message)` existente com códigos da spec; mensagem redigida, nunca body bruto/cookies. Warning não transforma conteúdo de CAPTCHA/erro em sucesso. Orçamentos/cancelamento/disabled checados antes de cache e após espera; late result descartado. Timeout total compartilhado e sem retentativa invisível.

`WebService::cancel_conversation` fecha cache e aborta chamadas; interrupt cancela Turno corrente. Cache de fontes/conteúdo é isolado, limitado e somente memória. UI/history guardam proveniência citada pelas APIs existentes, respeitando efemeridade.

## Eventos e fontes na UI

Novo evento previsto `webSource` com threadId/turnId confiáveis e fonte estruturada, sem conteúdo integral. UI mostra atividade com itens de ferramenta existentes e painel por resposta. Marcador `[[aura-source:W1]]` só resolve para URL de fonte conhecida naquela Conversa; desconhecido nunca recebe badge de verificação. ID estável dentro da Conversa; fonte final lida atualiza registro do snippet sem perder origem.

URL de clique revalidada antes de opener; título/texto renderizados como texto. IPC cobre DTO do evento, preferências e estado das fontes pelo dourado, não schema interno do adapter externo. `Settings.webEnabled=true` como default para arquivos antigos; false cancela e bloqueia todos os caminhos web gerenciados por esta feature.
# Entrega transitória interna (plan r8)

Os DTOs de busca/leitura descritos neste arquivo continuam sendo os dados que o modelo recebe. Na integração proposta em TK-007, o output MCP visto pelo sidecar usa um envelope opaco versionado, sem copiar esses DTOs: `{"auraToolResult":{"version":1,"handle":"<nonce aleatório de 256 bits>"}}`. Não é argumento aceito do modelo nem credencial. O gateway resolve somente um output de ferramenta web do Aura associado ao threadId autenticado e à emissão vigente no Host; o provedor recebe o DTO original como dado externo. A forma final e os dois formatos de output (lista/string) devem ser fixados pelo teste com o sidecar real antes da implementação concluir.

Referência de conteúdo anterior expirado/cancelado recebe erro estável sem conteúdo e sem nova rede automática; referência de outra Conversa é bloqueada antes do provedor. Nenhuma expansão em mensagens, argumentos ou output de terceiros. A proposta ainda não está implementada; os gates de TK-004 permanecem pendentes.

