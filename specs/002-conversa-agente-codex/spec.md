---
schema: hybrid/spec
schema_version: "1.0"
effort_id: 002-conversa-agente-codex
revision: 2
status: accepted
profile: standard
---

# Specification: Conversa agêntica via Codex app-server

## Problem and desired result

O usuário quer conversar com uma IA de fronteira dentro do Overlay usando a assinatura do ChatGPT que já paga, com respostas em streaming, histórico, controle sobre ações do agente e sem instalar nada além do Aura. O resultado é uma Conversa completa no Overlay: "Continue with ChatGPT" (Sign in with ChatGPT com uso do plano), envio de texto e imagens, resposta em streaming com persona do Aura, escolha de modelo, medidor de limite, histórico, Conversas efêmeras, Aprovações e modos Chat/Tarefa — tudo sobre o Codex app-server gerenciado pelo Aura.

## Consumers and actors

- Usuário no Overlay.
- Codex app-server (processo filho gerenciado), OpenAI (autorização em `auth.openai.com`, inferência em `api.openai.com/v1/responses` cobrada do plano ChatGPT).
- Esforços 003 (BYOK), 004/005 (Chips de contexto de captura), 007 (anexos), 008 (skills/MCP), 009 (Minibar), que consomem a sessão e o modelo de Chips.

## Scope

### Included

- Obtenção, verificação e supervisão do app-server fixado; `CODEX_HOME` isolado.
- Login "Continue with ChatGPT" (SIWC: registro dinâmico, consentimento de uso do plano), renovação de tokens, várias contas, sair com revogação, exibição da conta.
- Conversa com streaming, persona do Aura, markdown, interrupção e mapeamento de erros.
- Seletor de modelo/esforço de raciocínio e indicador "Usando plano ChatGPT" com "Gerenciar uso".
- Chips de contexto e envio de imagens.
- Histórico (listar, buscar, retomar, renomear, fixar, arquivar, excluir) e Conversa efêmera.
- Aprovações de comando, alteração de arquivo, permissões, perguntas do agente e elicitation MCP.
- Direcionar turno em andamento (steer), compactação manual, uso de tokens.
- Modos Chat e Tarefa com Workspace da conversa.

### Excluded

- Provedores BYOK (003).
- Captura de tela/áudio e ferramentas do Aura (004/005).
- Anexos não-imagem (007).
- Gerenciadores de skills/MCP e modo plano (008).
- Minibar e notificações (009).

## User journeys and scenarios

### US-001 — Continuar com ChatGPT (Priority: P1)

Como usuário Plus/Pro, quero entrar com minha conta ChatGPT e autorizar o Aura a usar meu plano, sem criar chave de API.

Independent demonstration: primeira abertura → "Continue with ChatGPT" → navegador → consentimento → Overlay mostra a conta e "Você está usando seu plano ChatGPT".

#### Acceptance scenarios

- **AC-001** — Dado nenhuma conta registrada, quando o usuário escolhe "Continue with ChatGPT", então o navegador padrão abre a autorização da OpenAI com registro dinâmico do Aura; após o consentimento com uso do plano, em até 5 s o Overlay mostra e-mail e conta ativa e, apenas nesse primeiro login, o aviso "Você está usando seu plano ChatGPT" com "Entendi" e link "Gerenciar uso".
- **AC-002** — Dado que o usuário recusa o consentimento ou a resposta não inclui a permissão de uso do plano, quando o login termina, então o Aura não faz inferência pelo plano e oferece "Ativar uso do plano ChatGPT" (nova autorização com consentimento) ou "Configurar outro provedor".
- **AC-003** — Dado um login em andamento, quando o usuário cancela ou fecha o navegador e cancela no Aura, então o estado volta a "não conectado" e uma nova tentativa funciona imediatamente.
- **AC-004** — Dado um usuário conectado, quando escolhe "Sair", então o Aura revoga a sessão renovável na OpenAI (ou avisa que a revogação não foi confirmada), apaga os tokens locais, mantém o registro do cliente para um próximo login, e nenhum token aparece em arquivos sob `%LOCALAPPDATA%\Aura`.
- **AC-026** — Dado uma conversa em andamento com o access token perto de expirar, quando o prazo chega, então o Aura renova o token antes do vencimento sem interromper o turno; dado um refresh inválido (`invalid_grant` ou equivalente), então pede novo login preservando a Conversa.
- **AC-027** — Dado duas contas ChatGPT, quando o usuário adiciona a segunda e alterna entre elas no menu da conta, então cada uma mantém seu próprio registro e credenciais, e as novas Conversas usam a conta ativa.

### US-002 — Conversar com respostas em streaming (Priority: P1)

Como usuário, quero enviar uma pergunta e ver a resposta surgir imediatamente, formatada, podendo interromper.

Independent demonstration: perguntar "liste 3 atalhos do Windows em tabela" e ver a tabela sendo escrita.

#### Acceptance scenarios

- **AC-005** — Dado um usuário conectado, quando envia um texto, então o Overlay expande, a resposta aparece progressivamente e, ao terminar, markdown (títulos, listas, tabelas, código com botão copiar, links) está renderizado; cada trecho recebido é pintado em até 50 ms após chegar ao host.
- **AC-006** — Dado uma resposta em andamento, quando o usuário pressiona `Ctrl+.` ou o botão parar, então o turno termina como "interrompido" e o texto parcial permanece visível.
- **AC-007** — Dado qualquer Conversa nova, quando ela é iniciada, então as instruções base enviadas ao agente são a Persona do Aura (e não as instruções padrão de programação do Codex), no idioma da interface.
- **AC-008** — Dado um erro, quando o turno falha, então o Overlay mostra mensagem específica: limite de uso do plano atingido (`subscription_sharing_usage_limit_exceeded`: modal com "Gerenciar uso" como ação principal e "Usar outro provedor" como secundária, sem inventar horário de reset), plano não elegível (`subscription_sharing_user_not_eligible`: explica que exige Plus/Pro ou política do workspace), recurso não suportado (`subscription_sharing_unsupported_capability`: indica o parâmetro), sem conexão (com "tentar novamente"), sessão expirada (com "entrar novamente"), contexto excedido (com "compactar ou nova conversa"), ou genérica com código e request id para diagnóstico.
- **AC-009** — Dado o app-server parado, quando o usuário envia a primeira mensagem, então o Aura o inicia de forma transparente (indicador "preparando…") e a mensagem é enviada; dado 15 min sem turno ativo, então o processo é encerrado; a próxima mensagem numa Conversa existente a retoma sem perda de histórico.
- **AC-010** — Dado um turno em andamento, quando o processo do app-server termina inesperadamente, então o turno aparece como falho com "reconectar", o supervisor reinicia o processo (backoff 1 s, 2 s, 4 s; no máximo 3 tentativas em 60 s) e a Conversa pode continuar.

### US-003 — Escolher modelo e acompanhar limites (Priority: P2)

#### Acceptance scenarios

- **AC-011** — Dado o seletor de modelo, quando aberto, então lista os modelos com visibilidade `list` retornados por `GET /v1/models` para a conta ativa (na ordem do servidor), marca o padrão e oferece apenas os esforços de raciocínio suportados pelo modelo escolhido; a escolha vale para novas Conversas e persiste entre reinícios.
- **AC-012** — Dado Conversas usando o plano ChatGPT, quando o Overlay está expandido, então aparece "Usando plano ChatGPT" junto ao seletor de modelo com link "Gerenciar uso" para `https://chatgpt.com/settings/usage`; com um Provedor BYOK, o indicador mostra o Provedor em uso.

### US-004 — Chips de contexto e imagens (Priority: P1)

#### Acceptance scenarios

- **AC-013** — Dado o Overlay aberto, quando o usuário cola (`Ctrl+V`) ou arrasta uma imagem PNG/JPEG/WebP/GIF, então surge um Chip com miniatura; ao enviar, a imagem vai no turno; se o Chip for removido antes, a imagem não vai.
- **AC-014** — Dado um modelo sem entrada de imagem, quando há Chip de imagem, então o Chip mostra aviso e o envio é bloqueado com a opção de trocar para um modelo compatível.

### US-005 — Histórico e Conversas efêmeras (Priority: P2)

#### Acceptance scenarios

- **AC-015** — Dado Conversas anteriores, quando o usuário pressiona `Ctrl+H`, então vê a lista (nome ou prévia, data, fixadas no topo), pode buscar por texto e, ao abrir uma, vê os turnos anteriores e pode continuar.
- **AC-016** — Dado uma Conversa no histórico, quando o usuário renomeia, fixa, arquiva ou exclui (com confirmação), então a lista reflete a mudança; a excluída não volta a aparecer e seu Workspace da conversa é removido.
- **AC-017** — Dado `Ctrl+Shift+E`, quando o usuário conversa e fecha a Conversa efêmera, então ela não aparece no histórico, nada dela fica no disco do Aura, e um indicador "efêmera" fica visível enquanto aberta.

### US-006 — Aprovações e perguntas do agente (Priority: P1)

#### Acceptance scenarios

- **AC-018** — Dado o agente pedindo para executar um comando, quando o cartão de Aprovação aparece com comando, pasta e motivo, então "Aceitar" executa, "Recusar" marca o item como recusado e o agente continua, e "Aceitar nesta conversa" aprova pedidos equivalentes seguintes apenas nessa Conversa.
- **AC-019** — Dado o agente propondo alterar arquivos, quando o cartão aparece, então mostra os arquivos e o diff resumido, com as mesmas decisões.
- **AC-020** — Dado o agente ou um servidor MCP pedindo informações ao usuário, quando o formulário aparece, então o usuário responde (opções ou texto livre) e a resposta é entregue; se houver tempo de auto-resolução, ele é mostrado e respeitado.
- **AC-021** — Dado uma Aprovação pendente com o Overlay oculto, quando o usuário o reabre, então o cartão pendente está visível e focado; nada é aprovado automaticamente, e o ícone da bandeja indica pendência.

### US-007 — Controle do turno (Priority: P2)

#### Acceptance scenarios

- **AC-022** — Dado uma resposta em andamento, quando o usuário envia texto com `Ctrl+Enter`, então o texto é acrescentado ao turno atual (sem iniciar outro) e aparece na conversa.
- **AC-023** — Dado uma Conversa longa, quando o usuário executa `/compactar`, então aparece um Item de compactação e o indicador de uso de contexto diminui.

### US-008 — Modos Chat e Tarefa (Priority: P1)

#### Acceptance scenarios

- **AC-024** — Dado o Modo Chat (padrão), quando o agente tenta executar comandos ou alterar arquivos, então isso não acontece sem sandbox somente-leitura e Aprovação; dado o Modo Tarefa com uma pasta concedida, então o agente pode alterar arquivos apenas no Workspace da conversa e nessa pasta, sempre com Aprovação conforme a política.
- **AC-025** — Dado uma Conversa nova, quando o primeiro turno é enviado, então existe `%LOCALAPPDATA%\Aura\workspaces\<id-da-conversa>` e ele é o diretório de trabalho do agente.

## Requirements

- **FR-001** — O sistema MUST obter, verificar (SHA-256 fixado) e supervisionar o app-server com `CODEX_HOME` isolado, início sob demanda, parada por inatividade e reinício após falha.
- **FR-002** — O sistema MUST implementar "Continue with ChatGPT" (Sign in with ChatGPT com uso do plano: PKCE, registro dinâmico, validação do ID token e do escopo `chatgpt.tokens.use.direct`), renovar tokens, suportar várias contas, sair com revogação e guardar credenciais só no cofre do Windows.
- **FR-003** — O sistema MUST conduzir Conversas com a Persona do Aura, streaming renderizado em markdown, interrupção e mensagens de erro específicas.
- **FR-004** — O sistema MUST oferecer seleção de modelo/esforço a partir do catálogo da conta e indicar quando o plano ChatGPT está em uso, com acesso a "Gerenciar uso".
- **FR-005** — O sistema MUST representar contexto a enviar como Chips removíveis e enviar imagens no turno respeitando as modalidades do modelo.
- **FR-006** — O sistema MUST oferecer histórico pesquisável com retomar, renomear, fixar, arquivar e excluir, além de Conversas efêmeras.
- **FR-007** — O sistema MUST apresentar Aprovações e pedidos de informação do agente/MCP e nunca aprovar ações com efeito sem decisão do usuário ou regra de sessão explícita.
- **FR-008** — O sistema MUST permitir direcionar um turno em andamento, compactar a Conversa e mostrar uso de contexto.
- **FR-009** — O sistema MUST oferecer Modo Chat (somente leitura) e Modo Tarefa (escrita restrita ao Workspace da conversa e pastas concedidas).

## Limits, errors, and compatibility

- Uma versão fixada do app-server por release do Aura; divergência de SHA-256 impede a execução e mostra "reinstale/atualize o Aura".
- Sem internet no primeiro uso, o download do app-server falha com mensagem e "tentar novamente".
- Mensagem vazia não é enviada; texto até 100 000 caracteres por turno (acima disso, sugerir anexar como arquivo).
- Imagens: até 10 por turno, 20 MB cada; maiores são recusadas no Chip com motivo.
- Conversas efêmeras não podem ser retomadas após fechar.
- Os rollouts do Codex não são cifrados pelo Aura (limitação documentada na arquitetura).
- Limitações do uso do plano ChatGPT (preview da OpenAI): sem entrada de áudio/vídeo (voz passa por ASR local), sem ferramentas hospedadas de geração de imagem, file search, computer use e `tool_search`; para Plus, a janela de 5 h é compartilhada com outros apps.
- Uso do plano ChatGPT em app pago/fechado depende de aprovação da OpenAI (Q-001 em `docs/discovery.md`); o fluxo técnico é o mesmo.

## Hypotheses and dependencies

- Hipótese H-008: `base_instructions` substitui a persona de programação. Check: TK-001.
- Hipótese H-003: imagens em resultados MCP chegam ao modelo. Check: TK-001 (resultado orienta o esforço 004).
- Hipótese H-004 (parcial): app-server ocioso ≤ 80 MB. Check: TK-001.
- Hipótese H-002: parcialmente confirmada pela doc SIWC (liberado para open-source/local). Check restante: aprovação para app pago, se Q-001 decidir assim.
- Hipótese H-016: o fluxo SIWC com registro dinâmico funciona para o Aura e um turno do app-server via Gateway completa com o access token. Check: TK-001 (spike) e TK-002.
- Dependência: esforço 001 (Overlay, store, `ChildRegistry`, `ForegroundTracker`, cofre). Estado: planejado.
- Dependência: servidor loopback do Gateway — criado neste esforço (TK-002) e estendido pelo 003.

## Success criteria

### Delivery-verifiable

- **SC-001** — AC-001–AC-025 com evidência atual (contrato automatizado com app-server falso + execução real ponta a ponta para login, streaming e aprovação).
- **SC-002** — Marco 1 demonstrado em vídeo (com o TK-001 do esforço 004).

### Post-delivery observation

- **SC-003** — Taxa de turnos falhos por erro do Aura (não do provedor) < 1% nas primeiras 2 semanas de beta (log local exportado voluntariamente).

## Decisions and open questions

- Login: SIWC é o único caminho de uso do plano ChatGPT (ADR 0007); o login interno do Codex não é usado.
- Marca e textos seguem as diretrizes de UI da OpenAI ("Continue with ChatGPT", "Usando plano ChatGPT", "Gerenciar uso").
- Persona em pt-BR/en segue o idioma da interface; o usuário pode acrescentar instruções próprias (008).
- Modo Chat é o padrão; Modo Tarefa exige escolha explícita por Conversa.
- Nenhuma pergunta de produto pendente.
