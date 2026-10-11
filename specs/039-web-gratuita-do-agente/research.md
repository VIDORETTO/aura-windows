# Pesquisa — busca e leitura gratuitas

Data da consulta: 06/10/2026. Escopo: documentação pública primária, sem chamar APIs de busca dos fornecedores, criar contas, testar a rede do Aura ou copiar código externo. Baseline local `ed9a9654c6e76e48186fc010a12eba77023b00f5`. Context7 não oferece ferramentas nesta sessão; foram consultadas fontes oficiais diretamente. Disponibilidade/cotas/qualidade ao vivo permanecem não verificadas.

## O que fazem os projetos estudados

### Hermes Agent

Separa `web_search` de `web_extract` e permite backend diferente por capacidade. Inclui DDGS sem chave e SearXNG, além de fornecedores com API e acesso anônimo; documentação descreve rotação gratuita sujeita a limites. O suporte nativo OpenAI depende do transporte/assinatura. Contribuição para o Aura: separar busca/extração e expor degradação; não herdar promessa de disponibilidade nem copiar dependência Python. [Documentação Hermes](https://hermes-agent.nousresearch.com/docs/user-guide/features/web-search).

### OpenClaw

Expõe busca por provedores e leitura separada. Pesquisa gratuita é uma rota explicitamente selecionada. DDG é integração HTML experimental e não oficial, sensível a mudança de markup/CAPTCHA. Parallel oferece rota anônima separada da API autenticada. Contribuição: manter adapters e rota gratuita explícita, sem autodetectar credencial paga. [Busca](https://docs.openclaw.ai/tools/web), [DDG](https://docs.openclaw.ai/tools/duckduckgo-search), [Parallel](https://docs.openclaw.ai/tools/parallel-search).

Leitura faz HTTP sem executar JS, extração principal com Readability, limites/cache e validação de destinos/redirecionamentos. Páginas autenticadas ou dependentes de JS usam outra capacidade. Contribuição: extrair localmente em vez de simplesmente retirar tags; distinguir indisponibilidade de ausência de conteúdo. [Leitura do OpenClaw](https://docs.openclaw.ai/tools/web-fetch).

### ChatGPT/OpenAI e Claude/Anthropic

A ferramenta web da OpenAI documenta busca, abertura de página e localização de conteúdo, além de referências; chamadas de busca da API têm custo. Isso explica capacidades públicas, não revela toda a infraestrutura do ChatGPT. Adotar pesquisa iterativa e proveniência, sem depender da API paga. [Web search OpenAI](https://developers.openai.com/api/docs/guides/tools-web-search).

Claude documenta busca com citações e custo adicional à inferência. A leitura é capacidade separada; pode apoiar citações e o fluxo buscar → selecionar → ler → analisar. Adotar referências junto às afirmações e distinguir trecho de página lida. Não prometer paridade do mecanismo proprietário. [Busca Claude](https://platform.claude.com/docs/en/agents-and-tools/tool-use/web-search-tool), [Leitura Claude](https://platform.claude.com/docs/en/agents-and-tools/tool-use/web-fetch-tool).

## Alternativas de gratuidade

1. **Parallel Search MCP anônimo**: fornecedor documenta endpoint `https://search.parallel.ai/mcp`, acesso gratuito sem chave com limites menores, buscas anônimas em modo fast e parâmetros `objective`/`search_queries`. `session_id` deve ser estável na Conversa; não mudar para evadir rate limit. Configurações avançadas autenticadas não se aplicam ao acesso anônimo. Recomendação técnica: primário, com contrato de `tools/list` capturado/testado na futura integração; não usar `/mcp-oauth` ou API autenticada. [Fonte oficial](https://docs.parallel.ai/integrations/mcp/search-mcp).
2. **DDG HTML**: gratuito sem chave/conta, porém interface não oficial com bloqueios/markup variável. Alternativa limitada a falha do primário, sem burlar CAPTCHA ou trocar identidade/IP. Implementação Rust local; DDGS Python não é requisito do Aura. [Integração documentada pelo OpenClaw](https://docs.openclaw.ai/tools/duckduckgo-search).
3. **SearXNG**: metabusca com API HTTP/JSON e parâmetros de idioma/data. Muitas instâncias públicas desativam JSON. Instância própria não cobra por busca, mas exige infraestrutura, manutenção e cumprimento de licença do serviço. Não empacotar SearXNG nem exigir servidor neste esforço; uma integração opcional futura pode usar MCP já disponível. [API oficial, documentação 2026.10.4](https://docs.searxng.org/dev/search_api.html).
4. **Brave/Tavily/Firecrawl/Exa e APIs hospedadas de modelos**: gratuidade, quando existente, pode exigir chave, cota ou termos comerciais mutáveis. Não são base da promessa sem configuração. Hermes/OpenClaw os suportam; isso não torna todos gratuitos. O esforço não adiciona cobrança, crédito ou fallback autenticado.

## Extração e dependências

`aura-ingest::text::html_to_text` existente remove tags e retém estrutura básica; não isola conteúdo principal de site. Usá-lo sozinho não sustenta qualidade de leitura solicitada. Candidato escolhido: `dom_smoothie` 0.18.2, Rust, extração inspirada em Readability, licença MIT; manter parser de busca no mesmo ecossistema DOM. Confirmar árvore transitiva/MSRV/Windows com `cargo deny` e build na implementação antes de fixar lockfile. Não copiar código SearXNG (licença não permissiva) nem instalar Python/Node no runtime. [Repositório/licença](https://github.com/niklak/dom_smoothie), [API 0.18.2](https://docs.rs/dom_smoothie/0.18.2/dom_smoothie/).

HTTP já usa `reqwest` 0.13.5 no lockfile. API oferece redirect policy, resolução explícita, timeout e desativação de proxy; implementar DNS validado/pinado e redirecionamento manual. Client genérico padrão não basta para SSRF. [ClientBuilder da versão 0.13.5](https://docs.rs/reqwest/0.13.5/reqwest/struct.ClientBuilder.html).

## Recomendação e limites

Inferência de projeto: serviço Rust local, Parallel anônimo primário, DDG alternativo, leitura direta com extração local, fontes estruturadas e fluxo iterativo no agente atual. Essa escolha combina contrato gratuito documentado com independência para ler páginas; não é benchmark executado.

Riscos: mudança do acesso anônimo, rate limit, bloqueio do DDG, HTML difícil/JS e capacidade do modelo em escolher/citar fontes. Cache/orçamento/falhas explícitas reduzem efeitos, mas não oferecem SLA. Gate de qualidade futuro em `tdd.md`; fracasso volta ao planejamento, sem contratar serviço pago. Não é realista prometer web ilimitada com qualidade/disponibilidade idênticas a ChatGPT/Claude gratuitamente.

## Observações da implementação do TK-001

Endpoint anônimo respondeu a initialize, initialized, tools/list e web_search sem chave. Servidor observado 1.27.0; protocolo negociado 2025-03-26. Schema mínimo publicado exige objective e search_queries, admite session_id e model_name. Aura envia somente os dois primeiros e UUID de sessão estável; nenhum nome de modelo ou histórico. Resultado real inclui results com url/title/publish_date/excerpts, aceito via structuredContent ou JSON do bloco textual. O teste público usa fixture mínima do schema e resultados sintéticos, com oráculo independente.

Uma busca pública por documentação Rust retornou fontes ao vivo. É smoke de disponibilidade/contrato, não a avaliação fixa de 12 consultas do TK-005. DDG tem cobertura controlada de HTML, desafio, link intermediário e intercalamento; não se atribui disponibilidade contínua a essa cobertura.
