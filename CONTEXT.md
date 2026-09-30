# Aura

Glossário do domínio. Define o que cada termo é; detalhes técnicos ficam em `docs/architecture/` e nos `plan.md`.

## Language

### Interface com o usuário

**Overlay**
Janela flutuante, translúcida e sempre no topo onde o usuário conversa com o agente. Tem dois estados: compacto (só a barra de entrada) e expandido (conversa visível).
_Avoid_: popup, janela de chat, HUD

**Minibar**
Faixa estreita sempre no topo que substitui o Overlay quando o usuário troca de janela durante uma resposta, mostrando status e prévia.
_Avoid_: barra mínima, mini-overlay

**Atalho de invocação**
Combinação de teclas global (ou duplo toque em Ctrl) que mostra ou esconde o Overlay.
_Avoid_: hotkey mágica, gatilho

**Chip de contexto**
Elemento visível na barra de entrada que representa algo que será enviado no próximo Turno (captura de tela, recorte de áudio, seleção, anexo). O usuário pode removê-lo antes de enviar.
_Avoid_: tag, anexo implícito

**Aplicativo anterior**
Janela que estava em primeiro plano imediatamente antes do Overlay aparecer; fonte da seleção, do título e do perfil de aplicativo.
_Avoid_: app ativo (ambíguo depois que o Overlay ganha foco)

### Conversa com o agente

**Conversa**
Sequência persistente de Turnos entre o usuário e o agente, correspondente a uma thread do Codex.
_Avoid_: sessão, chat, thread (na UI)

**Conversa efêmera**
Conversa que não é gravada em disco e desaparece ao ser fechada.
_Avoid_: modo anônimo, incógnito

**Turno**
Um pedido do usuário e todo o trabalho do agente que se segue até a resposta final, interrupção ou falha.
_Avoid_: mensagem, rodada

**Item**
Unidade dentro de um Turno: mensagem do usuário, resposta do agente, raciocínio, chamada de ferramenta, comando, alteração de arquivo, busca web.
_Avoid_: evento, bloco

**Modo Chat**
Modo padrão de Conversa: o agente responde e usa ferramentas de leitura (tela, áudio, anexos, web, MCP); não executa comandos nem altera arquivos.
_Avoid_: modo simples

**Modo Tarefa**
Modo em que o agente pode executar comandos e alterar arquivos no Workspace da conversa ou em pastas concedidas, sempre sob sandbox e Aprovações.
_Avoid_: modo agente (todo modo é agêntico), cowork

**Aprovação**
Pedido explícito do agente para executar uma ação com efeito (comando, alteração de arquivo, permissão, tool com efeito). O usuário aceita uma vez, aceita para a Conversa, recusa ou cancela.
_Avoid_: confirmação, permissão (reservado para Permissão do agente)

**Persona**
Instruções base que definem como o agente do Aura se comporta; substituem as instruções padrão de programação do Codex.
_Avoid_: system prompt (na UI), personalidade

**Workspace da conversa**
Pasta local de uma Conversa que guarda anexos, capturas usadas e arquivos gerados; é o diretório de trabalho do agente.
_Avoid_: pasta temporária, sandbox

### Provedores

**Provedor**
Origem dos modelos usados numa Conversa. É o Provedor ChatGPT ou um Provedor BYOK.
_Avoid_: backend, engine, API

**Provedor ChatGPT**
Acesso aos modelos pela conta ChatGPT do usuário via "Continue with ChatGPT" (Sign in with ChatGPT), consumindo o plano Plus/Pro dele.
_Avoid_: conta OpenAI, Codex login, assinatura (quando se refere ao provedor)

**Provedor BYOK**
Provedor configurado pelo usuário com endpoint, formato de API e Credencial próprios.
_Avoid_: custom provider, chave própria (quando se refere ao provedor inteiro)

**Credencial**
Segredo de um Provedor BYOK ou servidor MCP, guardado no cofre do Windows e nunca exibido depois de salvo.
_Avoid_: token, senha, API key (genérico)

**Gateway**
Tradutor local que permite ao agente usar Provedores BYOK que não falam a Responses API.
_Avoid_: proxy (ambíguo com proxies de rede)

### Captura e privacidade

**Fonte de captura**
Origem de dados sensoriais: Tela, Microfone ou Áudio do sistema. Cada fonte tem seu próprio Modo de captura.
_Avoid_: dispositivo, input

**Modo de captura**
Como uma Fonte de captura grava: Desligado, Sob demanda, Buffer recente, Manual ou Contínuo.
_Avoid_: modo de gravação (ambíguo)

**Sob demanda**
Modo em que nada é gravado; uma captura instantânea é feita apenas quando o usuário pede ou o agente pede com Permissão.
_Avoid_: manual (reservado para Gravação manual)

**Buffer recente**
Modo que mantém somente os últimos N minutos de uma Fonte de captura, descartando continuamente o mais antigo, até o usuário salvar um Recorte.
_Avoid_: histórico, replay buffer, rolling

**Gravação manual**
Modo que grava a partir do clique em "Gravar" até o clique em "Parar".
_Avoid_: captura manual

**Contínuo**
Modo que grava sempre que o Aura está ativo, sujeito à Retenção.
_Avoid_: always-on (na UI), 24/7

**Segmento**
Pedaço curto e cifrado de gravação de uma Fonte de captura; unidade de armazenamento e descarte.
_Avoid_: chunk, arquivo

**Recorte**
Intervalo de tempo de uma ou mais Fontes de captura promovido a item salvo ou anexado a uma Conversa.
_Avoid_: clip, trecho

**Retenção**
Regra que define por quanto tempo e até quanto espaço Segmentos e Gravações são mantidos.
_Avoid_: limpeza, expiração

**Permissão do agente**
Regra, por Fonte de captura, que define se o agente pode acessar capturas por conta própria: Nunca, Perguntar ou Sempre.
_Avoid_: aprovação (reservado para ações com efeito)

**Política de privacidade**
Conjunto de regras que decide, no momento da captura ou do acesso, se algo pode ser capturado, gravado ou entregue ao agente: Modos, Permissões, Janelas excluídas e Pausa.
_Avoid_: filtro, guard

**Janela excluída**
Aplicativo ou janela (por processo, título ou padrão) que nunca é capturado nem entregue ao agente.
_Avoid_: blacklist, lista negra

**Pausa de privacidade**
Estado global, acionado por atalho ou bandeja, que suspende todas as Fontes de captura até ser retomado.
_Avoid_: modo privado

**Registro de acesso**
Histórico local de cada captura entregue ao agente, com Fonte, momento, motivo e decisão da Política.
_Avoid_: log (genérico), auditoria (ok em texto técnico)

### Voz e anexos

**Push-to-talk**
Gravar a fala enquanto um atalho ou botão está pressionado e transcrevê-la para a barra de entrada.
_Avoid_: ditado (reservado para ditado em qualquer app, CAND-011)

**Modelo ASR**
Modelo de reconhecimento de fala escolhido pelo usuário, local (baixado) ou em nuvem (via Provedor BYOK).
_Avoid_: STT, whisper (é só uma família)

**Catálogo de modelos**
Lista curada de Modelos ASR disponíveis para download, com tamanho, idiomas, velocidade e requisitos.
_Avoid_: loja, hub

**Transcrição**
Texto produzido por um Modelo ASR a partir de fala, com idioma e marcações de tempo.
_Avoid_: legenda

**Anexo**
Arquivo que o usuário adiciona a um Turno (imagem, vídeo, áudio, PDF, planilha, documento).
_Avoid_: upload, arquivo enviado

**Ingestão**
Conversão local de um Anexo em conteúdo que o modelo entende (texto, tabelas, imagens de páginas, keyframes, Transcrição).
_Avoid_: parsing, processamento

### Extensões

**Ferramenta do Aura**
Capacidade do desktop (captura de tela, texto da janela, áudio recente) que o agente pode chamar, sempre sujeita à Política de privacidade.
_Avoid_: tool nativa, plugin

**Skill**
Pacote de instruções e recursos no padrão agentskills.io que o agente carrega quando a tarefa pede ou quando o usuário invoca por `/`.
_Avoid_: prompt salvo, macro

**Comando rápido**
Atalho de texto iniciado por `/` que aplica um prompt pronto ou uma ação do Aura (por exemplo `/screen`, `/tldr`).
_Avoid_: slash command (na UI)

**Servidor MCP**
Servidor externo configurado pelo usuário que oferece ferramentas e recursos ao agente pelo Model Context Protocol.
_Avoid_: conector, integração

**Memória**
Fato durável sobre o usuário ou suas preferências que o agente guarda para Conversas futuras, revisável e apagável.
_Avoid_: contexto persistente

**Perfil de aplicativo**
Instruções e preferências de contexto aplicadas automaticamente quando o Aplicativo anterior é um processo específico.
_Avoid_: modo por app
