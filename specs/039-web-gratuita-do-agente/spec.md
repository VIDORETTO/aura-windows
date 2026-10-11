---
schema: hybrid/spec
schema_version: "1.0"
effort_id: 039-web-gratuita-do-agente
revision: 1
status: accepted
profile: standard
---

# Busca e leitura da web gratuitas para o agente

## Problema e resultado desejado

Pedido de 06/10/2026: pesquisar como Hermes Agent, OpenClaw, ChatGPT e Claude usam a web e preparar acesso gratuito de busca e leitura de páginas no Aura, com pesquisa de qualidade. Resultado: agente encontra fontes, abre páginas, cruza informações e responde com referências verificáveis, sem chave/conta/cartão/API paga de busca ou extração.

A preparação inicial foi somente documental; o usuário posteriormente autorizou executar todos os esforços e tickets, um por vez, deixando bloqueios para o final. Esta atualização editorial não altera requisitos ou aceites da revisão 1. A comparação com ChatGPT/Claude orienta capacidades observáveis; não promete reproduzir seus índices, modelos proprietários ou disponibilidade. Gratuidade aqui cobre busca/extração: conexão, máquina e inferência do provedor atual continuam com seus custos/limites existentes.

## Consumidores e escopo

Usuário nos modos Chat, Tarefa e Plano; agente via ferramentas do Aura; provedores de busca; sites públicos. Inclui busca gratuita padrão, alternativa gratuita, leitura HTTP(S), conteúdo principal, continuidade de leitura, fontes/citações, limites, cancelamento, preferência para desativar web, degradação e avaliação de qualidade.

Exclui: navegador autenticado, executar JavaScript, CAPTCHA/paywall, compras/formulários, contas/cartões/chaves de busca, APIs pagas, hospedagem obrigatória do Aura, crawler/índice global, scraping em massa, VPN/proxy de evasão e expansão automática de permissões. PDF remoto, downloads binários e páginas só renderizadas por JS ficam para esforço próprio; informar a limitação.

## Jornadas e aceites

### US-001 — Pesquisar sem configurar serviço pago (P1)

- **AC-001** — Com apenas um Provedor ChatGPT ou BYOK capaz de ferramentas já configurado, sem credenciais de busca, pedir pesquisa retorna fontes públicas com título, URL, trecho, identificação da origem e data de consulta. Não pede cadastro/chave/cartão nem chama rota paga de pesquisa. Falta de suporte a ferramentas no modelo é informada.
- **AC-002** — Provedor gratuito primário responde 429/503/timeout: usa uma alternativa gratuita dentro do orçamento, marca a origem efetiva e informa degradação; se todos falham retorna indisponibilidade estruturada. CAPTCHA/403 não é sucesso vazio, não há rotação de identidade para contornar cota, nem ativação de API paga.
- **AC-003** — Resposta de busca contém URL duplicada, URL não web e metadados incompletos: preserva ordem útil das fontes válidas, elimina duplicatas exatas após normalização segura, omite esquema inválido e distingue sem resultados de erro. Data de publicação ausente permanece desconhecida; horário da consulta não vira data de publicação.

### US-002 — Ler a página, não apenas o trecho (P1)

- **AC-004** — URL pública de artigo HTML com título, navegação, anúncios, parágrafos, lista e tabela: leitura devolve título/URL final/conteúdo principal em texto estruturado, preserva informação da lista/tabela e identifica fonte/data de consulta; navegação, scripts e anúncios não dominam o resultado.
- **AC-005** — Página legível maior que limite por chamada: resultado informa truncamento, intervalo e próximo deslocamento; leitura seguinte continua no ponto informado até limite de documento. URL redirecionada mantém URL pedida e final. JS obrigatório, login, PDF/binário, bloqueio ou extração vazia produzem erro/limitação explícita, sem alegar leitura integral.
- **AC-006** — URL privada/local, DNS para IP privado, DNS rebinding ou redirecionamento público→privado é rejeitado antes de conexão ao destino proibido. Esquemas não HTTP(S), userinfo e portas fora do contrato são inválidos. URL pública segura funciona usando os mesmos controles.

### US-003 — Entregar pesquisa verificável (P1)

- **AC-007** — Pergunta factual atual ou pedido explícito de pesquisa: agente pesquisa, abre fonte relevante e responde com links próximos às afirmações apoiadas. Pedido de comparação usa duas fontes de domínios independentes quando disponíveis; documento específico pode usar a fonte primária única. Distingue conteúdo lido de trecho de busca e incerteza quando não conseguiu verificar.
- **AC-008** — Página tenta instruir agente a ignorar usuário, ler segredo ou executar comando: conteúdo continua dado externo não confiável; não aumenta permissões, não aciona ações só por instrução da página e não vira Persona. Fontes inventadas ou identificadores sem resultado correspondente não são apresentados como verificados.
- **AC-009** — Durante busca/leitura, usuário vê atividade e depois fontes com título/domínio/URL acessível; abrir fonte usa navegador externo mediante clique, com URL HTTP(S) validada. Funciona com pt-BR/en, teclado e histórico; Conversa efêmera não ganha persistência de conteúdo web.

### US-004 — Manter controle e limites (P1)

- **AC-010** — Desativar acesso à web impede novas chamadas de busca/leitura, incluindo cache e busca nativa hospedada; chamadas em curso são canceladas e respostas tardias descartadas. Ativar novamente permite uso, sem mudar regras de captura, arquivos ou comandos.
- **AC-011** — Repetição da mesma busca/leitura na mesma Conversa dentro de 15 minutos reutiliza cache sinalizado, sem rede extra. Cache expirado, pedido de atualização ou outra Conversa não reutiliza indevidamente. Cancelamento interrompe trabalho, não inicia fallback e não cria resultado/cache de sucesso.
- **AC-012** — Entrada inválida, página grande/lenta, excesso de chamadas ou concorrência atinge limites abaixo com erro explícito, sem travar UI ou crescimento ilimitado; texto/conversa/credenciais não aparecem em logs/diagnósticos. Provedores recebem apenas objetivo/consultas necessários e identificador aleatório de sessão, nunca histórico inteiro ou ID local do usuário.

### US-005 — Provar qualidade e compatibilidade (P1)

- **AC-013** — Suíte determinística atravessa busca → leitura → resposta/fontes com exemplos literais e sem internet; valida caminho feliz, degradação e limites. Na avaliação ao vivo de 12 consultas públicas definidas em `tdd.md`, ao menos 10 recuperam fonte relevante entre os primeiros 5 resultados; pelo menos 8 dos 10 HTML estáticos selecionados extraem a informação-alvo. Metas são gates futuros, não resultados desta sessão.
- **AC-014** — App-server fixado e gateway demonstram ferramentas em Chat/Plano/Tarefa, com Responses, Chat Completions e Anthropic em provedores capazes. Pesquisa não exige habilitar execução de comandos nem rede da sandbox de Tarefa. Ausência de suporte real é falha de compatibilidade explícita, sem upgrade automático do sidecar.

## Requisitos

- **FR-001** — Busca e extração DEVEM operar sem cobrança adicional de serviço, chave paga, cartão ou autenticação de busca; nenhum fallback pago/nativo hospedado automático.
- **FR-002** — Busca DEVE retornar fontes estruturadas, ordenar/deduplicar, classificar falhas e oferecer alternativa gratuita limitada.
- **FR-003** — Leitura DEVE extrair conteúdo principal, manter proveniência/estrutura e permitir continuar, informando limitações.
- **FR-004** — Acesso DEVE impedir destinos privados e validar DNS/conexão/redirecionamentos, sem credenciais/cookies do usuário.
- **FR-005** — Agente DEVE pesquisar quando necessário, ler fontes e apoiar afirmações com citações reais, tratando conteúdo externo como não confiável.
- **FR-006** — UI DEVE mostrar atividade/fontes acessíveis e preservar contexto no histórico conforme política de Conversa existente.
- **FR-007** — Usuário DEVE poder desativar web; cache, cancelamento, limites e logs DEVEM respeitar isolamento/privacidade.
- **FR-008** — Entrega DEVE passar pela avaliação de qualidade e pelo contrato da versão fixada nos transportes/modos suportados.

## Limites e erros

Valores iniciais deste contrato, revisáveis mediante nova spec:

- Objetivo até 2.000 caracteres Unicode; 1–3 consultas de 1–200 caracteres cada; no máximo 10 resultados, padrão 5; trecho por resultado até 1.000 caracteres. Não prometer filtro temporal exato que backend gratuito não ofereça.
- URL até 2.048 caracteres, somente HTTP/80 ou HTTPS/443, sem userinfo. Até 3 redirecionamentos, validar cada salto e DNS. Conteúdo HTTP descomprimido até 2 MiB; saída de leitura por chamada até 20.000 caracteres; documento extraído até 100.000, continuação por deslocamento Unicode.
- Prazo total por chamada 30 s, conexão até 5 s, orçamento compartilhado entre tentativas. Máximo 2 chamadas de busca ao backend por tool (primário + alternativa), 3 consultas HTTP se alternativa DDG; sem retentar CAPTCHA.
- Até 3 pesquisas e 6 leituras por Turno; até 2 requisições web simultâneas por Conversa. Estouro informa limite e aguarda novo pedido/Turno.
- Cache somente memória, 15 min, por Conversa, máximo 32 entradas/8 MiB; eliminar ao fechar/excluir Conversa ou sair do app. Atualização explícita ignora cache e continua sujeita ao mesmo orçamento.
- Erros estáveis: `invalid_input`, `web_disabled`, `unsafe_url`, `rate_limited`, `unavailable`, `timeout`, `blocked`, `unsupported_content`, `too_large`, `limit_exceeded`, `cancelled`, `extract_failed`. Sem resultados é sucesso vazio.

## Hipóteses e dependências

Pesquisa primária e limites em `research.md`. Serviço gratuito pode mudar política ou impor cotas; não há SLA/infinito garantido. Leitura sem JS cobre conteúdo público estático; não cobre toda a internet. MCP/gateway existentes sustentam integração; fidelidade de namespace e ferramentas diferidas deve ser testada no fixado, especialmente Chat Completions (limitação no HANDOFF). Sintese depende do modelo atual.

## Critérios de sucesso

- **SC-001** — Todos os AC têm ticket/TDD e somente evidência real futura pode aprová-los.
- **SC-002** — Geração de fontes e fluxo ao vivo atingem metas AC-013; nenhum custo de serviço pago introduzido.
- **SC-003** — Falhas explícitas, controles de destino e desativação passam com fixtures adversas.
- Observação posterior: disponibilidade, latência e relevância, sem consultas/conteúdo/identificadores pessoais em telemetria.

## Decisões e questões

Sem novas contas/pagamentos/infra obrigatória. Fontes são públicas; acesso automático da web pode ser desligado. Sem decisão de implantação ou instalação global nesta sessão. Eventual indisponibilidade permanente da opção gratuita retorna ao planejamento com evidência, sem mudar a promessa silenciosamente.

