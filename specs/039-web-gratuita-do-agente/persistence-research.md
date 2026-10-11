# Investigação de retenção — 10/10/2026

## Pergunta e resultado observado

Como entregar resultados web ao modelo sem gravar o conteúdo integral no rollout e sem substituir o histórico normal do Aura?

O protótipo `experiments/private_result_probe.py` executou o app-server **rust-v0.159.0** fixado, uma ferramenta MCP HTTP e um provedor Responses scripted, todos locais. A ferramenta devolveu uma referência aleatória, sem página. O gateway experimental substituiu a referência por um trecho sintético somente na requisição destinada ao provedor. O provedor verificou o trecho em memória e respondeu com um literal que não o reproduz. O turno terminou como `completed`.

Após encerrar o processo, a inspeção de todos os arquivos do CODEX_HOME isolado encontrou a referência e um rollout JSONL; não encontrou o literal integral da página. Um segundo processo abriu o mesmo threadId e encontrou a resposta. Foram duas requisições de inferência simuladas, uma chamada MCP e uma expansão. Os headers `thread-id`, `session-id` e `x-client-request-id` corresponderam ao threadId emitido pelo app-server. Não houve inferência paga, busca pública ou alteração de dados do usuário.

Resultado sanitizado em `experiments/private-result-result.json`. Procedimento executado:

```powershell
$env:AURA_CODEX_BIN = Join-Path $env:TEMP 'aura-038-model-picker-production/profile/bin/codex/rust-v0.159.0/bin/codex-app-server.exe'
python "$env:TEMP/aura-039-persistence-probe/private_result_probe.py"
```

O script preservado no esforço é cópia daquele executado. Seu diretório de trabalho e CODEX_HOME são criados com `tempfile.mkdtemp` sob TEMP. Não participa do runtime nem cria dependência Python para o Aura.

Tentativas anteriores: o primeiro parser não tratou a forma efetiva de `function_call_output.output` como lista; a tentativa seguinte mostrou recusa MCP por ausência de anotação read-only, sem chamada da ferramenta. O experimento foi corrigido para a ferramenta sintética declarar as mesmas anotações de leitura usadas no Aura. Essas tentativas não contam como red/green da aplicação nem como falha do gate normal já existente.

## Contrato da versão fixada

- [ThreadStartParams e ThreadResumeParams no tag fixado](https://github.com/openai/codex/blob/rust-v0.159.0/codex-rs/app-server-protocol/src/protocol/v2/thread.rs): start aceita ephemeral; resume não o expõe. A alternativa `thread/resume.history` está explicitamente marcada instável e destinada ao Codex Cloud.
- [Política de rollout no tag fixado](https://github.com/openai/codex/blob/rust-v0.159.0/codex-rs/rollout/src/policy.rs): `FunctionCallOutput` é persistido independentemente do historyMode. Logo, trocar o formato de histórico não resolve a retenção.
- [Config no tag fixado](https://github.com/openai/codex/blob/rust-v0.159.0/codex-rs/core/src/config/mod.rs): ephemeral é um override de runtime. Não há prova de que escrever uma chave arbitrária em config mude uma retomada.
- [Documentação oficial sobre limitações](https://developers.openai.com/siwc/token-sharing-open-source/preview-limitations): `store:false` da inferência é compatível com o histórico local. Portanto, não equivale a desativar gravação de rollout.

As fontes do tag foram baixadas de raw.githubusercontent.com e inspecionadas localmente. Context7 não está disponível. A documentação atual auxilia a distinção conceitual; a observação do executável fixado é a prova de formato usada nesta decisão.

## Decisão técnica e limites da prova

Preferir entrega transitória de resultados pelo gateway, preservando threads normais, rollouts, IDs e operações existentes. Proposta no ADR 0010; execução em TK-007. Não criar loop LLM, atualizar sidecar, apagar conteúdo depois da gravação nem substituir conversas normais por efêmeras.

O protótipo **não é a correção do Aura**. Seu parser recursivo serve somente ao experimento; a implementação deverá reconhecer um envelope exato, validar a origem da ferramenta e vincular a referência ao threadId autenticado. Não demonstrou isolamento entre Conversas, expiração, toggle, cancelamento, orçamento, Responses/Chat/Anthropic no gateway real, compactação, retomada de um novo turno ou privacidade de argumentos/URLs e de raciocínio emitido pelo modelo. A busca por bytes de um literal não prova ausência de toda informação derivada nem de conteúdo comprimido. O gate EV-032 permanece falho no código atual.

Não reduzir a promessa de retenção para fazer o protótipo passar: o teste final deve examinar páginas, snippets e URLs não citadas que sejam descobertas pelas ferramentas, além de preservar resposta e fontes citadas. Um argumento ou saída de raciocínio que reproduza conteúdo proibido continua sendo uma falha, ainda que o output MCP esteja protegido. Esse resultado deve voltar ao plano, não dispensar o oráculo.

## Implementação parcial e nova falha real — plan r8

Host/WebService/gateway já entregam outputs web por referências transitórias. As regressões atuais passaram: Rust304, fixado17, sandbox1, UI253 e typecheck. O teste do literal integral de página passou no thread normal. Nenhum desses resultados aprova a retenção integral.

O novo caso `web_fetched_but_uncited_source_url_is_not_persisted_in_normal_history` fez o app-server fixado pesquisar, ler W1 e W2 e responder citando somente W1. Os oráculos confirmaram os dois fatos recebidos pelo provedor, quatro chamadas de inferência e seis requests do transporte web, antes da inspeção. O teste falhou em0,75s: a URL de W2 aparece em dois arquivos, inclusive em um `response_item.function_call.arguments` e em `event_msg.item_completed` do rollout. O segundo arquivo é o WAL de `thread_history_1.sqlite`; a leitura SQLite read-only confirmou a URL na tabela `thread_items`, coluna `item_json`. Todos os dados são sintéticos e o perfil TEMP é exclusivo desse teste.

`experiments/audit_synthetic_storage.py` reproduz a inspeção de bytes, células SQLite (incluindo WAL visível por SQLite) e frames zstd/gzip que existam. O relatório sanitizado não contém corpo, argumentos nem nonce. No relatório final desta fixture:88 arquivos,128 linhas SQLite, nenhuma célula comprimida observada. Somente a URL não citada foi encontrada entre os quatro literais definidos; isso não prova ausência de dados arbitrários, raciocínio, compactação ou caminhos comprimidos não exercitados.

Conclusão técnica: proteger apenas `function_call_output` é insuficiente. A próxima solução precisa atuar também antes de o sidecar receber/persistir argumentos web devolvidos pelo provedor e avaliar raciocínio. Requer planejamento da transformação de retorno/SSE, preservando execução dos argumentos reais pelo Host, correlação de namespace/callId, orçamentos/cancelamento e todos os protocolos. Não implementar uma transformação ampla de strings, limpar os arquivos posteriormente, substituir threads por efêmeras ou aceitar persistência de W2. ADR0010 continua proposto e FD001 continua aberto. Conforme a sequência autorizada, TK007 fica bloqueado e o trabalho independente TK006 avança antes da resolução final.

Após normalizar rustfmt, o mesmo teste falhou novamente em0,71s (EV050), sem alterar expected. O perfil novo foi inspecionado pelo script preservado:86 arquivos,128 linhas SQLite, mesmos três locais de gravação e zero frames comprimidos observados (EV051; `experiments/uncited-storage-formatted-result.json`). O relatório anterior de88 arquivos fica histórico, separado desta execução. Regressões Rust304 e os17 casos originais passaram novamente; Clippy/fmt/diff check também passaram. A diferença de quantidade de arquivos auxiliares SQLite entre os perfis não altera o achado.

## Protótipos de argumentos e raciocínio sintético — retorno ao plano r9

Executado em dois perfis TEMP exclusivos com AURA_CODEX_BIN fixado: `python specs/039-web-gratuita-do-agente/experiments/private_arguments_probe.py --schema-conforming` e a mesma chamada com `--reasoning --schema-conforming`. Ambos completaram o Turno com duas requisições de inferência simulada e uma chamada MCP; o argumento opaco respeitou o schema original com url obrigatório e additionalProperties=false. MCP recebeu a referência, a memória resolveu os argumentos originais e o provedor recebeu os argumentos e o resultado externo originais. Após reiniciar o app-server, thread/read abriu o mesmo thread e a mesma resposta. A inspeção encontrou zero dos quatro literais sintéticos em 77 arquivos e 84/85 linhas SQLite, respectivamente. Referência opaca foi gravada, conforme esperado.

A variante com raciocínio restabeleceu um summary e campo opaco sintéticos somente para o provedor controlado. Não foi usado um blob criptográfico válido de provedor comercial, compactação ou adapter do Aura. Nenhum frame comprimido apareceu; orçamento/autorização de produção não foram exercitados. Resultado sanitizado: `experiments/private-arguments-schema-result.json`. Não houve inferência paga nem acesso ao perfil do usuário. O scanner de quatro literais é prova limitada de viabilidade, não aprovação geral de privacidade.

Preferir o envelope conforme o schema, em vez da primeira variante experimental sem url obrigatório. A decisão permite proteger a fronteira SSE mantendo o contrato MCP e a identidade/retomada do Codex. A solução será implementada no TK007 em etapas de argumentos e raciocínio; EV050/051 continuam descrevendo a falha real da aplicação até nova execução. Não alterar oráculo, limpar disco após gravação ou substituir threads normais por efêmeras.

## Implementação r9 — argumentos protegidos, retenção integral pendente

O retorno SSE do gateway já protege os argumentos de web_search/web_fetch antes do Codex; Host e gateway restituem os originais nas respectivas fronteiras autorizadas. Argumentos/resultados compartilham o mesmo armazenamento e limites, com kinds distintos, referências não recursivas e contexto capturado antes de upstream I/O. Um atraso após cancelamento/novo Turno não pode aproveitar a nova autoridade. Não houve alteração de schema MCP, versão do sidecar, spec1 ou expected do gate.

Red reexecutado antes da correção: W2 não citada em dois arquivos,0,81s. Green após integração:0,77s; após formatação e convergência,18/18 casos reais passaram, incluindo o mesmo gate W2. O perfil sintético pós-fmt foi preservado para auditoria opt-in, sem copiar corpos para o repositório. `experiments/uncited-storage-private-arguments-formatted-result.json`:86 arquivos/128 linhas SQLite,zero dos quatro literais; zero frames comprimidos. EV058/059 registram progresso parcial, não a privacidade integral.

316 regressões dos cinco crates e18 integrações reais passaram. Dez novos testes HTTP provam proteção/restauração, fragmentação UTF-8/CRLF, deltas/intercalação, escopo, origem, orçamento, expiração, cancelamento tardio e IDs. O parser existente dos adapters corrompia Unicode entre chunks; um red literal independente mostrou esse problema e a correção preserva os bytes até o frame completo. Repetição histórica de call_id após um output concluído permanece compatível; duplicação de chamada ainda pendente não autoriza expansão. Clippy/all-targets/fmt/diff passaram. O último ajuste de teste somente extraiu variáveis de comparação literal, após o qual os10casos eClippy foram reexecutados.

FD001 continua aberto: falta proteger e provar raciocínio/item opaco, compactação e os caminhos de retenção ainda não exercitados. A auditoria de quatro literais não é prova geral de ausência de toda informação derivada; não afirmar suporte a blob criptográfico comercial ou decompression branch sem executar. TK007 permanece in_progress/partial, TK004/TK005 dependentes.

## Implementação r9 — raciocínio protegido e compactação exercitada

O novo red real de raciocínio encontrou conteúdo de W2 em dois arquivos mesmo com outputs e argumentos já protegidos (EV063, histórico). O gateway agora retém o item original de raciocínio em memória e entrega ao sidecar somente uma referência, sem summary ou deltas privados. Na próxima requisição autorizada, o provedor recebe o item original completo. Essa entrega compartilha orçamento, escopo e expiração com os outros dados transitórios. Um teste de relógio revelou e corrigiu a perda de proteção quando o resultado antigo expira antes do raciocínio mais recente.

Na confirmação atual, passaram 14 testes HTTP de private_calls, nove de delivery e todos os 21 casos web com o app-server rust-v0.159.0 fixado. Os três novos casos reais exercitam raciocínio privado, novo Turno após reinício e compactação pela operação pública do Host. O caso de compactação confirmou o evento Compacted, nove requisições ao provedor controlado, seis ao transporte web e reabertura do mesmo histórico com a resposta e apenas W1 citada. O provedor verificou os campos originais de raciocínio; não se descartou todo o item para fazer o scan passar.

O perfil sintético exclusivo da compactação foi inspecionado pelo auditor: 88 arquivos, 139 registros SQLite e zero ocorrências dos quatro marcadores predefinidos, conforme experiments/reasoning-compaction-storage-result.json. Nenhum frame comprimido foi observado nesse perfil. Separadamente, check_storage_auditor.py passou com controles positivos independentes: dois arquivos gzip/zstd e duas células SQLite comprimidas, cada localização com os três marcadores esperados. Esse controle comprova a detecção pelo auditor, sem atribuir armazenamento comprimido ao Codex.

Os campos opacos desta fixture ainda são sintéticos, sem criptografia comercial. A matriz existente Responses/Chat/Anthropic passou, mas os novos casos de raciocínio exercitaram Responses; isso não comprova replay de raciocínio nativo nos outros adapters. Ainda faltam campos criptografados representativos, cobertura adicional do ciclo de vida do raciocínio, regressões finais e revisão antes de concluir TK007. ADR0010 continua proposto; FD001 continua aberto.

A [documentação oficial de raciocínio](https://developers.openai.com/api/docs/guides/reasoning#keeping-reasoning-items-in-context) recomenda preservar os itens necessários durante o ciclo de ferramentas. A implementação mantém o item original para o provedor enquanto a referência está vigente, sem atualizar o sidecar ou executar inferência paga.

## Provas finais representativas — TK007 r9

A fixture externa agora pode emitir ciphertext AES-256-GCM real, gerado e decifrado por .NET fora do Aura. Somente o ciphertext entra no provedor controlado; chave e plaintext sintético ficam no probe independente. O request seguinte observado pelo provedor contém os mesmos bytes; a descriptografia recupera o literal predefinido e uma tag adulterada é rejeitada. O fluxo inclui novo Turno, reinício, compactação pública e abertura do mesmo histórico. Resultado atual: experiments/encrypted-reasoning-compaction-final-result.json, caso nativo1/1 em1,74s, 88arquivos/139registros SQLite/zero quatro marcadores. O oráculo Rust verifica também ausência dos próprios bytes do ciphertext nos arquivos.

Procedimento da raiz: `pwsh -NoProfile -File specs/039-web-gratuita-do-agente/experiments/check_encrypted_reasoning.ps1 -CodexBin <executavel fixado> -Output <novo resultado sanitizado>`. O probe usa somente perfis criados sob seu próprio diretório TEMP, restaura suas variáveis de processo e preserva os perfis para inspeção. Duas primeiras tentativas passaram no caso nativo e na autenticação, mas falharam na coleta do perfil: saída de teste capturada e fixture ignorando AURA_E2E_DIR. Essas falhas de coleta foram corrigidas; não foram classificadas como reds do Aura. Nenhum perfil do usuário foi acessado.

Chat Completions emite reasoning_content e Anthropic emite thinking_delta nas variantes novas. Reinício e compactação passaram em ambos, com mesma resposta/fontes citadas e sem os literais privados no histórico. Eventos públicos ReasoningDelta e ToolCall também foram verificados sem texto privado/referências. Responses preserva o item original completo para o provedor; os adapters mantêm sua política anterior de normalização, sem introduzir replay de assinaturas proprietárias.

O mesmo probe habilita diagnóstico em diretório isolado: zero dumps durante a jornada nativa privada. Um processo separado executa private_web_diagnostics_are_suppressed_with_a_public_positive_control: a requisição pública cria exatamente um arquivo; requisições privadas, erro upstream com URL/conteúdo/nonces e erro de escopo não geram outros arquivos nem expõem os literais no erro. O controle prova que a instrumentação estava habilitada.

Regressões: 504 testes de workspace passaram; 34 dependentes de ambiente ficaram ignorados nessa execução. Separadamente,25/25 casos web reais em26,17s,16/16 HTTP private_calls e o caso de diagnóstico ignorado executado explicitamente passaram. Após o último ajuste da fixture externa de erro, os16casos afetados e o diagnóstico foram reexecutados; nenhum código produtivo mudou. Clippy de workspace/all-targets excluindo desktop, fmt e diff check passaram. Os controles comprimidos detectam dois arquivos e duas células gzip/zstd com três marcadores cada; nenhum frame comprimido apareceu nos perfis Codex. Essa cobertura permite concluir a entrega transitória, sem aprovar UI/qualidade live/Windows dos sucessores ou inferência comercial.
