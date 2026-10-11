---
status: accepted
---

# Entrega transitória de dados web pelo gateway

## Contexto

O esforço 039 exige que páginas e snippets não citados permaneçam transitórios. O app-server fixado persiste outputs de ferramentas em threads normais, mesmo sem histórico estendido. Trocar todas as threads por efêmeras comprometeria retomada, gestão de histórico e identidade do esforço 002. Sanitizar rollouts após a escrita não impede a gravação proibida.

O protótipo registrado em `specs/039-web-gratuita-do-agente/persistence-research.md` demonstrou que o mesmo thread normal pode receber uma referência opaca de ferramenta, enquanto o gateway entrega o conteúdo ao provedor e o histórico continua abrindo após reinício.

## Decisão

O Host retém resultados de `web_search`/`web_fetch` somente em memória, no orçamento do serviço web. MCP devolve um envelope versionado com referência aleatória. Na fronteira autenticada do gateway, somente outputs dessas ferramentas e da Conversa emissora são resolvidos, antes da passagem ou tradução para o provedor. Conteúdo de usuário, instruções, resultados de outras ferramentas e referências inválidas nunca autorizam expansão.

O retorno SSE protege também os argumentos web e os itens de raciocínio emitidos quando há contexto privado ativo. Argumentos mantêm o schema MCP; raciocínio mantém o ID e uma referência opaca, sem summary ou deltas privados. O item original completo é restituído somente na requisição autorizada ao provedor, antes dos adapters. Os três kinds compartilham limites, expiração e cancelamento no WebService; o contexto de retorno é capturado antes do I/O. Referência de raciocínio expirada é omitida sem enviar ciphertext inválido nem bloquear as mensagens públicas da continuação. A compatibilidade dos adapters com raciocínio nativo mantém o contrato existente; esta decisão não acrescenta replay de assinaturas proprietárias.

Isso complementa ADR 0003: requisições sem envelopes privados mantêm passagem existente; as que contêm envelopes emitidos pelo Aura recebem transformação local restrita. Codex continua responsável pelos threads normais e pelo restante do histórico, conforme ADR 0001. Não introduz loop de inferência, autenticação de busca, novo provedor nem upgrade.

## Consequências e gate

Referências no histórico serão inertes quando o conteúdo não estiver mais retido. Retomada recebe um erro estruturado sem conteúdo e pode pesquisar novamente sob a autorização atual. Histórico de mensagens e fontes citadas continua disponível sem rede. Os envelopes não são credenciais do gateway e nunca recebem autoridade fora da Conversa emissora.

Aceitação baseada na implementação e revisão de TK-007: 504 regressões de workspace, 25 integrações fixadas, fronteira HTTP/escopo/origem/orçamento/ciclo de vida, reinício e compactação nos três protocolos. O gate W2 mantém os oráculos originais e examina também raciocínio e ciphertext. Prova AES-256-GCM externa confirma os bytes recebidos pelo provedor e a descriptografia autenticada independente; controles gzip/zstd exercitam o auditor separadamente. O perfil real dessa fixture foi inspecionado em arquivos e SQLite. TK-004 continua responsável pelos seus aceites de fontes e controles na interface.

Limites observados: fixtures sintéticas controladas, sem inferência comercial ou formato criptográfico proprietário; nenhum frame comprimido foi produzido pelo Codex nesses perfis. A auditoria de marcadores é uma prova representativa, não uma garantia matemática sobre toda informação derivada. Relatório e comandos em `specs/039-web-gratuita-do-agente/persistence-research.md`; não se substituiu histórico normal por efêmero nem se apagou conteúdo após gravação.

Alternativas descartadas nesta etapa: historyMode (política persiste outputs), chave arbitrária ephemeral em config (sem contrato), reidratação por history instável (substitui identidade e amplia a migração), limpeza posterior de rollout (grava antes de remover).
