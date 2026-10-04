---
schema: hybrid/spec
schema_version: "1.0"
effort_id: 011-correcao-auditoria-e2e
revision: 1
status: accepted
profile: standard
---

# Correção integral da auditoria E2E

## Problem and desired result

Corrigir os 42 achados da auditoria de 02/10/2026, preservando dados pessoais, privacidade e contratos existentes. Autorizado pelo pedido /goal de 03/10/2026. Relatos e riscos são hipóteses até reprodução; inspeção não é execução real.

## Consumers and actors

Usuário do Aura no Windows; executor e verificador local.

## Scope

Inclui QA-001–QA-042 e atualização rastreável do relatório. Exclui migração de versão do app-server e mudanças em contas/configurações pessoais sem necessidade do teste. Um ticket por vez, primeiro QA-001/042, depois QA-002/003 e demais P1.

## Requirements

- **FR-001** — Resolver QA-001 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-002** — Resolver QA-002 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-003** — Resolver QA-003 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-004** — Resolver QA-004 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-005** — Resolver QA-005 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-006** — Resolver QA-006 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-007** — Resolver QA-007 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-008** — Resolver QA-008 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-009** — Resolver QA-009 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-010** — Resolver QA-010 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-011** — Resolver QA-011 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-012** — Resolver QA-012 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-013** — Resolver QA-013 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-014** — Resolver QA-014 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-015** — Resolver QA-015 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-016** — Resolver QA-016 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-017** — Resolver QA-017 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-018** — Resolver QA-018 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-019** — Resolver QA-019 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-020** — Resolver QA-020 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-021** — Resolver QA-021 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-022** — Resolver QA-022 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-023** — Resolver QA-023 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-024** — Resolver QA-024 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-025** — Resolver QA-025 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-026** — Resolver QA-026 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-027** — Resolver QA-027 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-028** — Resolver QA-028 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-029** — Resolver QA-029 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-030** — Resolver QA-030 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-031** — Resolver QA-031 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-032** — Resolver QA-032 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-033** — Resolver QA-033 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-034** — Resolver QA-034 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-035** — Resolver QA-035 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-036** — Resolver QA-036 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-037** — Resolver QA-037 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-038** — Resolver QA-038 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-039** — Resolver QA-039 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-040** — Resolver QA-040 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-041** — Resolver QA-041 conforme o comportamento esperado da auditoria e o contrato do esforço original.
- **FR-042** — Resolver QA-042 conforme o comportamento esperado da auditoria e o contrato do esforço original.

## User journeys and scenarios

### QA-001 — Janela compacta aumenta, mas conteúdo não acompanha

- **AC-001** — Aumentar e reduzir pelas bordas laterais no compacto: largura interna, viewport e conteúdo acompanham sem espaço vazio; altura segue conteúdo e menus. Expandido responde em todas as bordas. Preservar tamanho ao reabrir e validar DPI 100/125/150% e duas telas.

### QA-002 — Histórico e Arquivos juntos tornam a conversa ilegível

- **AC-002** — **P1 · Reproduzido no app nativo de demonstração.**

1. Enviar uma mensagem para expandir o Overlay na largura padrão de cerca de 640 px.
2. Abrir Histórico (`Ctrl+H`).
3. Abrir Arquivos e alterações no cabeçalho.

Histórico ocupa 256 px e Arquivos 320 px, ambos sem encolhimento. Sobra cerca de 64 px para a conversa; com os recuos, mensagens ficam quebradas em uma ou duas palavras por linha. A interface permite essa combinação sem ajustar a largura, alternar painéis ou manter um mínimo útil para a conversa.

Esperado: conversa legível em qualquer combinação permitida; painéis sobrepostos, mutuamente exclusivos ou largura mínima apropriada. Fonte: `HistoryPanel.tsx` (`w-64 shrink-0`), `WorkPanel.tsx:73` (`w-80 shrink-0`), `OverlayApp.tsx:258`.

![Conversa comprimida entre os dois painéis](evidencias-2026-10-02/25-demo-dois-paineis-0.jpg)

### QA-003 — Primeiro provedor cadastrado não desbloqueia o Overlay até reiniciar

- **AC-003** — **P1 · Reproduzido no perfil nativo isolado; ligação ausente também confirmada no código.**

1. Começar sem conta/provedores, clicar em “Usar minha chave”.
2. Cadastrar um provedor local sem credencial, nome `QA local isolado`.
3. Voltar ao Overlay.

O cadastro persistiu e apareceu nas Configurações, mas o Overlay continuou em “Bem-vindo / Continuar com ChatGPT / Usar minha chave”. Após reiniciar, passou a exibir primeiros passos e entrada de conversa. O endpoint local não estava disponível: o erro de conexão era esperado e **não é o achado**. A inconsistência é reconhecer o mesmo cadastro somente depois de reiniciar.

`OverlayApp.tsx:101` recarrega catálogo na montagem/mudança de `auth.active.clientId`; salvar provedor em outra janela não invalida o catálogo do Overlay. Esperado: refletir cadastro, remoção e atualização de modelos sem reinício, com o estado de erro do provedor claramente indicado.

Evidência de cadastro: [15-demo-provedor-salvo](evidencias-2026-10-02/15-demo-provedor-salvo-0.jpg). Depois do reinício, o fluxo de conversa ficou disponível.

### QA-004 — Interface continua parcialmente em português após escolher inglês

- **AC-004** — **P2 · Reproduzido no perfil nativo isolado e confirmado no código.**

Selecionar English em General e abrir Diagnostics, Extensions → Add server e anexar um TXT. Observações:

- Diagnostics: estado `pronto` e título nativo `Aura — Configurações`.
- Formulário MCP: `Nome`, `Transporte`, `Variável secreta (opcional)`, `Valor secreto`.
- Chips: `2 linhas`, `1 linhas`, nome acessível da lista `Contexto`.
- Menu de sugestões: nome acessível `Sugestões` literal.
- Comandos rápidos embutidos continuam com instruções em português. O catálogo de comandos persistidos precisa de uma decisão de localização, além da tradução da interface.

Esperado: interface e nomes acessíveis coerentes com o idioma (`010 AC-010`). O teste de paridade de chaves não detecta literais fora dos dicionários.

Evidências: [Diagnostics](evidencias-2026-10-02/30-demo-diagnostico-en-0.jpg), [MCP](evidencias-2026-10-02/32-demo-mcp-en-0.jpg), [anexo](evidencias-2026-10-02/33-demo-anexo-txt-0.jpg). Fontes: `settings/Diagnostics.tsx`, `settings/Extensions.tsx:211`, `overlay/ChipList.tsx:20`, `overlay/InputBar.tsx:191`, `src-tauri/src/tray.rs`.

### QA-005 — Login simulado da prévia de desenvolvimento não avança

- **AC-005** — **P2 · Reproduzido somente na prévia de navegador.**

Abrir a prévia Vite sem Tauri e clicar “Continuar com ChatGPT”: a tela permanece no login. Expandir/compactar responde normalmente; não houve erro de console capturado que explicasse o bloqueio.

`ipc/bridge.ts` inicializa um singleton após `await`, sem compartilhar uma promessa de inicialização. Chamadas concorrentes podem criar bridges mock diferentes; assinaturas e invocações deixam de usar a mesma instância. **Essa é uma hipótese causal forte por inspeção, não uma causa isolada por teste nesta sessão.**

Esperado: login mock transicionar para o estado conectado e permitir testar a interface. Este achado não significa que o login ChatGPT real falhou; o perfil real já estava conectado e enviou um turno com sucesso.

### QA-006 — Cancelar login aparece como falha técnica

- **AC-006** — **P3 · Reproduzido no app nativo isolado.**

Iniciar login e clicar Cancelar no Aura: volta ao cartão inicial, porém apresenta `Não foi possível entrar: sign-in cancelled` em vermelho. Esperado: cancelamento voluntário voltar ao estado inicial sem erro técnico, ou mensagem neutra localizada. O retorno ao estado inicial funcionou; uma nova autorização completa não foi realizada.

### QA-007 — Seletor de microfone em

- **AC-007** — Seletor de microfone em `settings/Voice.tsx:86` não tem `value` nem `onChange`; não salva dispositivo. `InputBar.tsx:228` chama `pttPress()` sem dispositivo. A seleção visual não controla a captura.

Esperado: Escolher um microfone e usar esse dispositivo, inclusive após reinício; `005 AC-001/002`. Validar com dois dispositivos de nomes distintos.

### QA-008 — 

- **AC-008** — `InputBar.tsx:73` descarta uma transcrição cujo texto é igual a `lastVoice.current`, sem reiniciar a referência em uma nova gravação. Duas falas consecutivas idênticas perdem a segunda inserção/envio.

Esperado: Deduplicar evento, não conteúdo entre gravações. Regressão: ditar a mesma frase duas vezes, inclusive após limpar o campo; `006 AC-007/008`. Não foi reproduzido com ASR real nesta sessão.

### QA-009 — Com idioma da interface pt-BR,

- **AC-009** — Com idioma da interface pt-BR, `pttRelease` passa `pt`. O host prioriza `asrLanguage` quando fixo, mas, quando a opção é Automático (`null`), usa esse fallback português (`host.rs:1577`).

Esperado: Automático realmente permitir autodetecção, independentemente do idioma da interface; `006 FR-007`. **Idioma fixo Espanhol/Inglês não está sendo declarado quebrado:** ele tem precedência no host.

### QA-010 — Botão de microfone só implementa

- **AC-010** — Botão de microfone só implementa `onPointerDown/Up/Leave`; opera por pressão, sem alternância de clique e sem ação de teclado no próprio botão.

Esperado: Clique alternar escuta conforme `006 AC-007`; Enter/Space no controle também funcionar. O atalho global é um caminho separado.

### QA-011 — Não há cartão de instalação recomendada no fluxo de tentativa de ditado sem modelo. O erro de

- **AC-011** — Não há cartão de instalação recomendada no fluxo de tentativa de ditado sem modelo. O erro de `pttPress` vira apenas aviso genérico.

Esperado: Cartão com modelo recomendado, tamanho e Baixar; `006 AC-013`. O perfil real estava sem modelos instalados; download/transcrição real não foram executados.

### QA-012 — Catálogo visual mostra nome, descrição, tamanho, licença e recomendação, mas não barras de velocidade/precisão nem requisitos CPU/GPU/RAM (

- **AC-012** — Catálogo visual mostra nome, descrição, tamanho, licença e recomendação, mas não barras de velocidade/precisão nem requisitos CPU/GPU/RAM (`Voice.tsx`).

Esperado: Metadados comparáveis para escolher o modelo; `006 AC-001`.

### QA-013 — Configurações não oferecem teste/medidor de microfone nem seleção e teste de saída de áudio do sistema. O seletor de microfone é o único controle de dispositivo observado.

- **AC-013** — Configurações não oferecem teste/medidor de microfone nem seleção e teste de saída de áudio do sistema. O seletor de microfone é o único controle de dispositivo observado.

Esperado: Medidores em tempo real e diagnóstico de dispositivo; `005 AC-001/002`. Não confundir a listagem de dispositivos do backend com um fluxo utilizável na UI.

### QA-014 — 

- **AC-014** — `overlay/ModelPicker.tsx` permite provedor/modelo/modo, mas não esforço de raciocínio. Também não exibe capacidades de imagem, ferramentas e raciocínio.

Esperado: Escolher somente esforços suportados e identificar capacidades por modelo; `002 AC-011`, `003 AC-012`.

### QA-015 — Histórico possui fixar, arquivar e excluir, mas nenhuma ação de renomear (

- **AC-015** — Histórico possui fixar, arquivar e excluir, mas nenhuma ação de renomear (`HistoryPanel.tsx`).

Esperado: Renomear conversa pela UI; `002 AC-016`.

### QA-016 — Histórico requisita

- **AC-016** — Histórico requisita `limit: 50` e utiliza somente `.items`, sem cursor/paginação ou “carregar mais” (`HistoryPanel.tsx:23`).

Esperado: Navegar todo o histórico. A busca pode recuperar conversas fora das primeiras 50, mas não substitui paginação para navegar sem conhecer seu texto. Regressão com >50 conversas; `002 FR-006`.

### QA-017 — Menu

- **AC-017** — Menu `/` contém `/plano`, `/tela`, comandos e Skills. `/compactar` não tem tratamento local em `session.send`. O texto pode ir ao modelo como prompt em vez de acionar compactação.

Esperado: Comando produzir Item de compactação e reduzir contexto; `002 AC-023`. O botão no indicador de contexto é um caminho diferente.

### QA-018 — Entrada não registra

- **AC-018** — Entrada não registra `onPaste`/tratamento de imagem na área de transferência; há seletor de arquivo e drop de caminhos Tauri.

Esperado: `Ctrl+V` com PNG/JPEG/WebP/GIF criar Chip e enviar a imagem; `002 AC-013`, `007 AC-001`. O teste de anexo por diálogo passou para TXT; ele não cobre colagem de imagem.

### QA-019 — Provedor salvo só tem Testar e Excluir (

- **AC-019** — Provedor salvo só tem Testar e Excluir (`settings/Providers.tsx`); não há editar URL, nome ou substituir credencial pela UI.

Esperado: Corrigir configuração sem excluir/recriar o provedor e romper suas referências. É um problema de operação da configuração, mesmo com backend capaz de salvar.

### QA-020 — “Personalizado” oferece URL e chave, mas não formato Responses/Chat Completions/Anthropic nem cabeçalhos extras (

- **AC-020** — “Personalizado” oferece URL e chave, mas não formato Responses/Chat Completions/Anthropic nem cabeçalhos extras (`Providers.tsx`).

Esperado: Configurar o formato correto e cabeçalhos; `003 AC-002`. Um endpoint customizado que exige outro formato fica sem configuração pela interface.

### QA-021 — Não há entrada manual de id de modelo/capacidades para provedor sem

- **AC-021** — Não há entrada manual de id de modelo/capacidades para provedor sem `/models`; seletor usa apenas listas descobertas.

Esperado: Informar id e modalidades manualmente; `003 AC-013`.

### QA-022 — Lista de Skills tem nome, descrição e Excluir; não mostra origem, ativar/desativar ou edição (

- **AC-022** — Lista de Skills tem nome, descrição e Excluir; não mostra origem, ativar/desativar ou edição (`settings/Extensions.tsx`, função `Skills`).

Esperado: Gerenciar origem, ativação e edição; `008 AC-001`, `FR-001`.

### QA-023 — Argumentos MCP são transformados com

- **AC-023** — Argumentos MCP são transformados com `args.split(/\s+/)` (`Extensions.tsx:159`). Aspas não protegem caminhos com espaços: `"C:\QA Folder\server.js"` vira dois argumentos com aspas remanescentes.

Esperado: Editor de argumentos estruturado ou parsing com semântica definida. Validar com caminho com espaços, argumento vazio e aspas; `008 AC-005`. Nenhum servidor externo foi iniciado por esse formulário.

### QA-024 — 

- **AC-024** — `disabledTools` é sempre `[]` no cadastro; lista mostra número de ferramentas, sem controles por ferramenta.

Esperado: Desabilitar ferramentas individualmente nas novas conversas; `008 AC-007`.

### QA-025 — Status MCP visual não apresenta claramente conectado/erro e últimas linhas de log; mostra contagem quando disponível e Conectar quando não autenticado.

- **AC-025** — Status MCP visual não apresenta claramente conectado/erro e últimas linhas de log; mostra contagem quando disponível e Conectar quando não autenticado.

Esperado: Diagnóstico operacional de falha e lista utilizável de ferramentas; `008 AC-005`.

### QA-026 — Memórias têm um switch geral, mas não tela de revisão, edição ou exclusão dos fatos guardados.

- **AC-026** — Memórias têm um switch geral, mas não tela de revisão, edição ou exclusão dos fatos guardados.

Esperado: Usuário controlar os itens memorizados; `008 AC-015`. Não foi necessário ativar memórias para verificar a ausência da interface.

### QA-027 — Prévia HTML usa

- **AC-027** — Prévia HTML usa `<iframe srcDoc>` (`WorkPanel.tsx:55`), enquanto CSP do pacote declara `frame-src 'none'` (`tauri.conf.json:36`). Há conflito de configuração com a prévia prevista.

Esperado: Prévia HTML em sandbox funcionar no **pacote de release** sem executar scripts/rede; `008 AC-014`. Confirmar no WebView2 empacotado; a execução em Vite não valida a CSP de produção.

### QA-028 — Prévia aceita imagens, texto/Markdown e HTML; PDF não entra em nenhum ramo e retorna

- **AC-028** — Prévia aceita imagens, texto/Markdown e HTML; PDF não entra em nenhum ramo e retorna `null` (`WorkPanel.tsx:16–55`).

Esperado: Mostrar prévia PDF ou uma ação/explicação explícita; `008 AC-014`. Abrir arquivo externamente não é prévia no Aura.

### QA-029 — 

- **AC-029** — `Privacy.tsx:19` reutiliza os minutos existentes ou 10; o usuário pode selecionar Buffer recente, mas não editar a duração.

Esperado: Escolher N entre 1–30 minutos; `004 AC-013`, buffer de áudio em `005`.

### QA-030 — Tela não oferece edição de Retenção em dias/espaço.

- **AC-030** — Tela não oferece edição de Retenção em dias/espaço.

Esperado: Configurar os limites e saber quando serão aplicados; `004 AC-016`, `005 AC-006`. Isso não afirma que a rotina interna de limpeza não existe.

### QA-031 — Registro visual mostra tempo relativo, fonte, ferramenta/requester e decisão crua. Não mostra ligação com Conversa, data/hora exata, miniatura ou indicação detalhada de cobertura (

- **AC-031** — Registro visual mostra tempo relativo, fonte, ferramenta/requester e decisão crua. Não mostra ligação com Conversa, data/hora exata, miniatura ou indicação detalhada de cobertura (`Privacy.tsx:112`).

Esperado: Auditoria rastreável das capturas entregues; `004 AC-008`.

### QA-032 — Gravações oferecem iniciar/parar/exportar/excluir, mas não player, duração explícita ou anexar à conversa (

- **AC-032** — Gravações oferecem iniciar/parar/exportar/excluir, mas não player, duração explícita ou anexar à conversa (`Recordings.tsx`).

Esperado: Reproduzir e anexar no Aura com duração; `004 AC-015`, `005 AC-004`. Gravações reais não foram criadas nesta sessão.

### QA-033 — Menu de contexto inspecionado tem tela, região, janela, seleção e arquivo; não há “últimos N minutos” de tela/áudio, escolha Microfone/Sistema/Ambos ou recorte combinado.

- **AC-033** — Menu de contexto inspecionado tem tela, região, janela, seleção e arquivo; não há “últimos N minutos” de tela/áudio, escolha Microfone/Sistema/Ambos ou recorte combinado.

Esperado: Usuário anexar o Buffer recente pela interface; `004 AC-014`, `005 AC-007/009`. Ferramentas que o agente pode chamar não substituem o comando explícito do usuário. Evidência: [menu compacto](evidencias-2026-10-02/26-demo-contexto-compacto-0.jpg).

### QA-034 — Configurações de Voz não incluem escolha de voz TTS, leitura automática ou escolha de provedor TTS. Botão Ouvir existe nas mensagens.

- **AC-034** — Configurações de Voz não incluem escolha de voz TTS, leitura automática ou escolha de provedor TTS. Botão Ouvir existe nas mensagens.

Esperado: Escolher voz offline/nuvem e leitura automática, com consentimento no caso de nuvem; `009 AC-005/006`. Não foi comprovada qualidade/saída de áudio TTS real.

### QA-035 — Handler de teclado do Overlay não implementa

- **AC-035** — Handler de teclado do Overlay não implementa `Ctrl+Shift+L` nem `Ctrl+Shift+Enter`. A entrada trata Ctrl+Enter como direcionamento de turno, sem distinguir a combinação de inserção prevista.

Esperado: Atalhos para ouvir e inserir no Aplicativo anterior; `009 AC-005/009`. O botão Inserir é outro caminho e não foi acionado contra um app pessoal.

### QA-036 — 

- **AC-036** — `Profiles.tsx` tem `defaultModel` no objeto, mas nenhum controle para escolher o modelo.

Esperado: Configurar todos os campos do Perfil de aplicativo disponíveis no domínio, especialmente modelo padrão. Edição de nome/instruções/modo existe e **não** foi classificada como ausente.

### QA-037 — Diagnostics exibe versão, engine, porta, pasta e contagens. Não apresenta estado detalhado de Gateway/MCP/worker, fontes ativas, conta e espaço em disco.

- **AC-037** — Diagnostics exibe versão, engine, porta, pasta e contagens. Não apresenta estado detalhado de Gateway/MCP/worker, fontes ativas, conta e espaço em disco.

Esperado: Diagnóstico completo do pipeline; `010 AC-011`. A exportação ZIP funcionou; o achado é a tela resumida.

### QA-038 — Chave pública do updater ainda é

- **AC-038** — Chave pública do updater ainda é `REPLACE_WITH_THE_PUBLIC_KEY_FROM_...` em `tauri.conf.json:83`.

Esperado: Publicação com chave válida e teste de atualização assinada; `010 AC-004`. Não foi realizado download/aplicação de atualização nem atribuído um erro de rede ao botão Verificar atualizações.

### QA-039 — Onboarding pode ser pulado, mas não há controle em Configurações para retomá-lo.

- **AC-039** — Onboarding pode ser pulado, mas não há controle em Configurações para retomá-lo.

Esperado: Retomar primeiros passos nas Configurações; `010 AC-006`, `FR-003`.

### QA-040 — Texto

- **AC-040** — Texto `onboarding.shortcut.body` ensina “Esc para fechar” (`i18n/pt-BR.ts:80`, usado por `FirstRun.tsx`). Handler atual e revisão 2 dizem que Esc nunca esconde Overlay.

Esperado: Tutorial coerente com o comportamento: atalho/botão para recolher, Esc para menus/voz; `001 AC-005`.

### QA-041 — Iniciar o executável normalmente mostrou Overlay.

- **AC-041** — Iniciar o executável normalmente mostrou Overlay. `001 AC-001` exige iniciar somente na bandeja; há caminho `--background` no shell.

Esperado: Decidir e atualizar contrato/onboarding ou ajustar inicialização. Não confundir abertura manual com o teste de logon/autostart, que não foi executado.

### QA-042 — Configuração nativa tem

- **AC-042** — Expandido tem mínimo próprio de 480 × 360 pixels lógicos na área interna durante arraste pelas bordas, suficiente para cabeçalho, conversa, entrada e rodapé; compacto mantém mínimo 480 × 64 e altura automática. Alternar os modos não deixa controles cortados.

## Limits, errors, and compatibility

Compatibilidade com os esforços 001–010 e ADRs 0001–0009. Nunca substituir um segredo nem registrar conteúdo pessoal em evidência. Divergências exigem decisão explícita no contrato antes da implementação. Evidência parcial não fecha aceite.

## Hypotheses and dependencies

QA-001: possível concorrência entre autoaltura e resize, ainda não comprovada. QA-027/042 exigem WebView2 empacotado. QA-038 requer atualização realmente assinada; não simular aprovação de publicação. Dispositivos/ASR/serviços externos exigem ambiente real e ausência é registrada.

## Success criteria

- **SC-001** — Todos os aceites com testes executados e execução real pertinente; relatório registra evidências e limites sem falsos concluídos.

## Decisions and open questions

Mínimo expandido 360 px lógicos é decisão reversível para preservar controles. Demais decisões materiais serão resolvidas na revisão do contrato do ticket antes de liberá-lo; autorização existente cobre as correções contratadas.

Clarificação editorial de 03/10: o mínimo refere-se ao arraste pelo usuário e à área interna, não à chamada administrativa Tauri set_size que pode ultrapassar restrições nativas. Não muda o comportamento contratado; mantém revisão 1 e reexecuta evidência afetada. QA-001 largura continua hipótese distinta da variante comprovada de altura.
