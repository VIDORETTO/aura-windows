# Descoberta — web do agente

Pedido e autorização em 06/10/2026: pesquisa + specs/planos/tickets/TDD, somente documentação. Baseline `ed9a9654c6e76e48186fc010a12eba77023b00f5`. Nenhum teste da aplicação/requisição aos fornecedores de pesquisa foi executado.

## Estado inspecionado

- `crates/aura-codex/src/home.rs`, `BaseConfig.web_search`: padrão true; render gera `web_search=live`. É ferramenta hospedada do provedor, não acesso gratuito universal de busca/leitura.
- ADR 0003: ferramentas Responses hospedadas não existem nos provedores traduzidos. ADR 0007: credenciais SIWC do Aura não podem ser substituídas por login privado/backend do Codex.
- `crates/aura-mcp/src/tools.rs`, `all`: catálogo próprio de ferramentas; sem `web_search`/`web_fetch` do Aura. `HostTools::call` roteia desktop/extensões; é seam de integração, sem APIs Windows novas.
- `crates/aura-codex/src/mapping.rs`: identifica item webSearch; `Messages.tsx` mostra ferramentas/markdown. Fontes estruturadas por busca/leitura ainda não têm contrato próprio na UI.
- `aura-mcp::ToolHandler`, `CallContext`: contexto lê Conversa do header opcional `x-aura-conversation`. Não foi encontrada configuração que forneça esse header por Conversa na rota atual. Portanto não se pode pressupor correlação de Turno; acrescentar resolução confiável no Host, sem argumento remoto ou seleção do Overlay como autoridade.
- `crates/aura-ingest/src/text.rs`, `html_to_text`: extrai estrutura de anexo HTML, sem pontuar conteúdo principal de páginas.
- HANDOFF alerta para `tool_search` em BYOK Chat e namespace MCP; a suíte real do sidecar fixado é obrigatória antes de afirmar compatibilidade.

## Decisões e hipóteses

Gratuito significa sem serviço pago de busca/extração, sem mudar custos de inferência. Interface uniforme em todos os provedores capazes. Web pública sem login, scripts, ações com efeitos ou acesso privado. Sem depender de instâncias públicas aleatórias de SearXNG. Pesquisa de alternativas e recomendação em `research.md`; técnica reversível no plano, comportamento na spec.

Hipóteses a testar: contrato/qualidade atual do MCP anônimo, extração HTML principal em Rust, normalização da saída MCP em provedores traduzidos, mapeamento de cancelamento ao Turno real. Nenhuma é declarada resolvida por navegar documentação.

## Gate de descoberta

Perfil standard por múltiplos módulos, integração externa, UI e controles de acesso. Tickets começam por provar busca gratuita; dependentes só executam após predecessor done. Não abrir o sandbox de comandos para liberar leitura web. Questões futuras de JS/PDF/infra ficam fora do esforço e não bloqueiam o contrato atual.
