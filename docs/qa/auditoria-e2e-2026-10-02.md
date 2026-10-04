# Aura — auditoria funcional e visual

Data da sessão: 02/10/2026, Windows, horário de Brasília. Revisão do código: `5b130b69ac3ede8dc5a8fdc6b0e1ceccac7a8393`.

## Resultado e grau de certeza

Foram registrados **42 achados**, incluindo o problema de redimensionamento informado pelo usuário. Este documento reúne testes pela interface nativa, inspeção visual das dez páginas de Configurações, uma conversa com o backend real, testes em um perfil isolado de demonstração, testes automatizados e comparação da interface com os contratos em `specs/`.

**Não equivale a uma certificação de que todas as funções foram executadas ponta a ponta.** Os fluxos que não puderam ser executados estão identificados na matriz de cobertura. Uma ausência confirmada na interface é registrada como lacuna de implementação; não é apresentada como falha de um serviço real que não foi exercitado.

Estados usados:

- **Reproduzido:** observado durante esta sessão, com o ambiente indicado.
- **Inspeção:** ligação ausente, implementação incompatível ou recurso previsto que não existe na interface examinada; ainda precisa de uma regressão executável específica.
- **Relato:** informado pelo usuário, sem reprodução conclusiva nesta sessão.
- **Risco:** evidência técnica que exige validação no aplicativo empacotado.
- **Divergência:** comportamento observado e contrato/documentação discordam; requer decisão de produto.

Prioridade: **P1** bloqueia um fluxo ou causa perda funcional relevante; **P2** prejudica uso, configuração, acessibilidade ou requisito contratado; **P3** problema menor de comunicação. Não foi comprovado incidente P0.

## Ambientes e cuidados com a interpretação

1. **App real:** `target/release/aura.exe`, versão exibida 0.1.0, executável existente de 01/10/2026. Conta ChatGPT já conectada. Conversa real de QA respondeu `AURA QA OK`.
2. **App nativo de demonstração:** compilado com `cargo build -p aura-desktop --release --features demo`, usando o perfil isolado `target/qa-profile-2026-10-02`, backend falso e frontend Vite. Capturável pelo gancho existente `AURA_QA_CAPTURABLE`. Este build não valida o empacotamento final, ASR real, captura real ou respostas de provedores externos.
3. **Prévia no navegador:** `pnpm -C apps/desktop dev --host 127.0.0.1`, bridge em memória. Usada para verificar o login simulado.

O Overlay real é excluído das capturas do Windows: a ferramenta via o fundo em seu lugar. Isso corresponde ao requisito de privacidade, **não é um defeito visual**. As imagens do Overlay neste documento são do build de demonstração. As imagens de Configurações 01–14 são do app real.

O executável original foi restaurado ao final e sua integridade foi conferida: SHA-256 `592C2010FE11AC9D6B8B36BF04AFAC1D5E012AF01595CE82490C8251A15CF055`. O app real voltou a ser iniciado. O servidor Vite e a aba temporária foram encerrados. Nenhum código de produto foi alterado, nenhuma credencial foi inserida e nenhuma política de privacidade foi modificada. A conversa sintética real de QA ficou no histórico; as demais alterações de teste pertencem ao perfil isolado.

## Achados observados pela interface

### QA-001 — Janela compacta aumenta, mas conteúdo não acompanha

**P1 · Relato do usuário · redimensionamento ainda sem reprodução conclusiva.**

- Passos informados: abrir o Overlay compacto e aumentar a janela pelas bordas; o contêiner nativo cresce, mas o conteúdo compacto fica no tamanho anterior.
- Esperado: a largura do conteúdo acompanhar a largura interna, sem região vazia ou controles presos ao tamanho anterior. No compacto, a altura automática é intencional; o contrato permite redimensionar apenas a largura (`001 FR-004`).
- Nesta sessão: compacto com entrada mediu aproximadamente 642 × 82 px; abrir `@` fez a altura crescer para acomodar o menu. Enviar uma mensagem e alternar os modos produziu o layout expandido de aproximadamente 642 × 724 px.
- Os arrastes das bordas não alteraram confiavelmente a dimensão; aumentar além do retângulo capturado foi rejeitado pela ferramenta, e controles nativos de maximização apresentaram coordenadas fora da janela. **Não é possível concluir que o arraste expandido funciona nem que o defeito ocorre somente no compacto.**
- Pontos a investigar: `apps/desktop/src/overlay/ResizeHandles.tsx`, `OverlayApp.tsx:34` (`useAutoHeight`), WebView2 durante `startResizeDragging`, interação entre altura automática e evento nativo de resize. O cálculo automático reaplica `window.innerWidth` quando muda a altura; a causa do relato ainda não foi comprovada.
- Regressão necessária: aumentar/reduzir manualmente ambas as bordas laterais no compacto, todas as bordas no expandido, com DPI 100/125/150%, menus abertos, duas telas e reabertura. Comparar dimensão da janela, viewport e `.overlay-shell` após cada movimento.

### QA-002 — Histórico e Arquivos juntos tornam a conversa ilegível

**P1 · Reproduzido no app nativo de demonstração.**

1. Enviar uma mensagem para expandir o Overlay na largura padrão de cerca de 640 px.
2. Abrir Histórico (`Ctrl+H`).
3. Abrir Arquivos e alterações no cabeçalho.

Histórico ocupa 256 px e Arquivos 320 px, ambos sem encolhimento. Sobra cerca de 64 px para a conversa; com os recuos, mensagens ficam quebradas em uma ou duas palavras por linha. A interface permite essa combinação sem ajustar a largura, alternar painéis ou manter um mínimo útil para a conversa.

Esperado: conversa legível em qualquer combinação permitida; painéis sobrepostos, mutuamente exclusivos ou largura mínima apropriada. Fonte: `HistoryPanel.tsx` (`w-64 shrink-0`), `WorkPanel.tsx:73` (`w-80 shrink-0`), `OverlayApp.tsx:258`.

![Conversa comprimida entre os dois painéis](evidencias-2026-10-02/25-demo-dois-paineis-0.jpg)

### QA-003 — Primeiro provedor cadastrado não desbloqueia o Overlay até reiniciar

**P1 · Reproduzido no perfil nativo isolado; ligação ausente também confirmada no código.**

1. Começar sem conta/provedores, clicar em “Usar minha chave”.
2. Cadastrar um provedor local sem credencial, nome `QA local isolado`.
3. Voltar ao Overlay.

O cadastro persistiu e apareceu nas Configurações, mas o Overlay continuou em “Bem-vindo / Continuar com ChatGPT / Usar minha chave”. Após reiniciar, passou a exibir primeiros passos e entrada de conversa. O endpoint local não estava disponível: o erro de conexão era esperado e **não é o achado**. A inconsistência é reconhecer o mesmo cadastro somente depois de reiniciar.

`OverlayApp.tsx:101` recarrega catálogo na montagem/mudança de `auth.active.clientId`; salvar provedor em outra janela não invalida o catálogo do Overlay. Esperado: refletir cadastro, remoção e atualização de modelos sem reinício, com o estado de erro do provedor claramente indicado.

Evidência de cadastro: [15-demo-provedor-salvo](evidencias-2026-10-02/15-demo-provedor-salvo-0.jpg). Depois do reinício, o fluxo de conversa ficou disponível.

### QA-004 — Interface continua parcialmente em português após escolher inglês

**P2 · Reproduzido no perfil nativo isolado e confirmado no código.**

Selecionar English em General e abrir Diagnostics, Extensions → Add server e anexar um TXT. Observações:

- Diagnostics: estado `pronto` e título nativo `Aura — Configurações`.
- Formulário MCP: `Nome`, `Transporte`, `Variável secreta (opcional)`, `Valor secreto`.
- Chips: `2 linhas`, `1 linhas`, nome acessível da lista `Contexto`.
- Menu de sugestões: nome acessível `Sugestões` literal.
- Comandos rápidos embutidos continuam com instruções em português. O catálogo de comandos persistidos precisa de uma decisão de localização, além da tradução da interface.

Esperado: interface e nomes acessíveis coerentes com o idioma (`010 AC-010`). O teste de paridade de chaves não detecta literais fora dos dicionários.

Evidências: [Diagnostics](evidencias-2026-10-02/30-demo-diagnostico-en-0.jpg), [MCP](evidencias-2026-10-02/32-demo-mcp-en-0.jpg), [anexo](evidencias-2026-10-02/33-demo-anexo-txt-0.jpg). Fontes: `settings/Diagnostics.tsx`, `settings/Extensions.tsx:211`, `overlay/ChipList.tsx:20`, `overlay/InputBar.tsx:191`, `src-tauri/src/tray.rs`.

### QA-005 — Login simulado da prévia de desenvolvimento não avança

**P2 · Reproduzido somente na prévia de navegador.**

Abrir a prévia Vite sem Tauri e clicar “Continuar com ChatGPT”: a tela permanece no login. Expandir/compactar responde normalmente; não houve erro de console capturado que explicasse o bloqueio.

`ipc/bridge.ts` inicializa um singleton após `await`, sem compartilhar uma promessa de inicialização. Chamadas concorrentes podem criar bridges mock diferentes; assinaturas e invocações deixam de usar a mesma instância. **Essa é uma hipótese causal forte por inspeção, não uma causa isolada por teste nesta sessão.**

Esperado: login mock transicionar para o estado conectado e permitir testar a interface. Este achado não significa que o login ChatGPT real falhou; o perfil real já estava conectado e enviou um turno com sucesso.

### QA-006 — Cancelar login aparece como falha técnica

**P3 · Reproduzido no app nativo isolado.**

Iniciar login e clicar Cancelar no Aura: volta ao cartão inicial, porém apresenta `Não foi possível entrar: sign-in cancelled` em vermelho. Esperado: cancelamento voluntário voltar ao estado inicial sem erro técnico, ou mensagem neutra localizada. O retorno ao estado inicial funcionou; uma nova autorização completa não foi realizada.

## Voz e áudio — lacunas confirmadas por inspeção

| ID / prioridade | Problema, impacto e evidência | Comportamento esperado / contrato |
| --- | --- | --- |
| **QA-007 / P1** | Seletor de microfone em `settings/Voice.tsx:86` não tem `value` nem `onChange`; não salva dispositivo. `InputBar.tsx:228` chama `pttPress()` sem dispositivo. A seleção visual não controla a captura. | Escolher um microfone e usar esse dispositivo, inclusive após reinício; `005 AC-001/002`. Validar com dois dispositivos de nomes distintos. |
| **QA-008 / P1** | `InputBar.tsx:73` descarta uma transcrição cujo texto é igual a `lastVoice.current`, sem reiniciar a referência em uma nova gravação. Duas falas consecutivas idênticas perdem a segunda inserção/envio. | Deduplicar evento, não conteúdo entre gravações. Regressão: ditar a mesma frase duas vezes, inclusive após limpar o campo; `006 AC-007/008`. Não foi reproduzido com ASR real nesta sessão. |
| **QA-009 / P2** | Com idioma da interface pt-BR, `pttRelease` passa `pt`. O host prioriza `asrLanguage` quando fixo, mas, quando a opção é Automático (`null`), usa esse fallback português (`host.rs:1577`). | Automático realmente permitir autodetecção, independentemente do idioma da interface; `006 FR-007`. **Idioma fixo Espanhol/Inglês não está sendo declarado quebrado:** ele tem precedência no host. |
| **QA-010 / P2** | Botão de microfone só implementa `onPointerDown/Up/Leave`; opera por pressão, sem alternância de clique e sem ação de teclado no próprio botão. | Clique alternar escuta conforme `006 AC-007`; Enter/Space no controle também funcionar. O atalho global é um caminho separado. |
| **QA-011 / P2** | Não há cartão de instalação recomendada no fluxo de tentativa de ditado sem modelo. O erro de `pttPress` vira apenas aviso genérico. | Cartão com modelo recomendado, tamanho e Baixar; `006 AC-013`. O perfil real estava sem modelos instalados; download/transcrição real não foram executados. |
| **QA-012 / P2** | Catálogo visual mostra nome, descrição, tamanho, licença e recomendação, mas não barras de velocidade/precisão nem requisitos CPU/GPU/RAM (`Voice.tsx`). | Metadados comparáveis para escolher o modelo; `006 AC-001`. |
| **QA-013 / P2** | Configurações não oferecem teste/medidor de microfone nem seleção e teste de saída de áudio do sistema. O seletor de microfone é o único controle de dispositivo observado. | Medidores em tempo real e diagnóstico de dispositivo; `005 AC-001/002`. Não confundir a listagem de dispositivos do backend com um fluxo utilizável na UI. |

## Conversa, modelos e provedores — lacunas confirmadas por inspeção

| ID / prioridade | Problema, impacto e evidência | Comportamento esperado / contrato |
| --- | --- | --- |
| **QA-014 / P2** | `overlay/ModelPicker.tsx` permite provedor/modelo/modo, mas não esforço de raciocínio. Também não exibe capacidades de imagem, ferramentas e raciocínio. | Escolher somente esforços suportados e identificar capacidades por modelo; `002 AC-011`, `003 AC-012`. |
| **QA-015 / P2** | Histórico possui fixar, arquivar e excluir, mas nenhuma ação de renomear (`HistoryPanel.tsx`). | Renomear conversa pela UI; `002 AC-016`. |
| **QA-016 / P2** | Histórico requisita `limit: 50` e utiliza somente `.items`, sem cursor/paginação ou “carregar mais” (`HistoryPanel.tsx:23`). | Navegar todo o histórico. A busca pode recuperar conversas fora das primeiras 50, mas não substitui paginação para navegar sem conhecer seu texto. Regressão com >50 conversas; `002 FR-006`. |
| **QA-017 / P2** | Menu `/` contém `/plano`, `/tela`, comandos e Skills. `/compactar` não tem tratamento local em `session.send`. O texto pode ir ao modelo como prompt em vez de acionar compactação. | Comando produzir Item de compactação e reduzir contexto; `002 AC-023`. O botão no indicador de contexto é um caminho diferente. |
| **QA-018 / P1** | Entrada não registra `onPaste`/tratamento de imagem na área de transferência; há seletor de arquivo e drop de caminhos Tauri. | `Ctrl+V` com PNG/JPEG/WebP/GIF criar Chip e enviar a imagem; `002 AC-013`, `007 AC-001`. O teste de anexo por diálogo passou para TXT; ele não cobre colagem de imagem. |
| **QA-019 / P2** | Provedor salvo só tem Testar e Excluir (`settings/Providers.tsx`); não há editar URL, nome ou substituir credencial pela UI. | Corrigir configuração sem excluir/recriar o provedor e romper suas referências. É um problema de operação da configuração, mesmo com backend capaz de salvar. |
| **QA-020 / P1** | “Personalizado” oferece URL e chave, mas não formato Responses/Chat Completions/Anthropic nem cabeçalhos extras (`Providers.tsx`). | Configurar o formato correto e cabeçalhos; `003 AC-002`. Um endpoint customizado que exige outro formato fica sem configuração pela interface. |
| **QA-021 / P2** | Não há entrada manual de id de modelo/capacidades para provedor sem `/models`; seletor usa apenas listas descobertas. | Informar id e modalidades manualmente; `003 AC-013`. |

## Extensões, arquivos e memórias

| ID / prioridade / estado | Problema, impacto e evidência | Comportamento esperado / contrato |
| --- | --- | --- |
| **QA-022 / P2 / Inspeção** | Lista de Skills tem nome, descrição e Excluir; não mostra origem, ativar/desativar ou edição (`settings/Extensions.tsx`, função `Skills`). | Gerenciar origem, ativação e edição; `008 AC-001`, `FR-001`. |
| **QA-023 / P1 / Inspeção** | Argumentos MCP são transformados com `args.split(/\s+/)` (`Extensions.tsx:159`). Aspas não protegem caminhos com espaços: `"C:\QA Folder\server.js"` vira dois argumentos com aspas remanescentes. | Editor de argumentos estruturado ou parsing com semântica definida. Validar com caminho com espaços, argumento vazio e aspas; `008 AC-005`. Nenhum servidor externo foi iniciado por esse formulário. |
| **QA-024 / P2 / Inspeção** | `disabledTools` é sempre `[]` no cadastro; lista mostra número de ferramentas, sem controles por ferramenta. | Desabilitar ferramentas individualmente nas novas conversas; `008 AC-007`. |
| **QA-025 / P2 / Inspeção** | Status MCP visual não apresenta claramente conectado/erro e últimas linhas de log; mostra contagem quando disponível e Conectar quando não autenticado. | Diagnóstico operacional de falha e lista utilizável de ferramentas; `008 AC-005`. |
| **QA-026 / P1 / Inspeção** | Memórias têm um switch geral, mas não tela de revisão, edição ou exclusão dos fatos guardados. | Usuário controlar os itens memorizados; `008 AC-015`. Não foi necessário ativar memórias para verificar a ausência da interface. |
| **QA-027 / P1 / Risco** | Prévia HTML usa `<iframe srcDoc>` (`WorkPanel.tsx:55`), enquanto CSP do pacote declara `frame-src 'none'` (`tauri.conf.json:36`). Há conflito de configuração com a prévia prevista. | Prévia HTML em sandbox funcionar no **pacote de release** sem executar scripts/rede; `008 AC-014`. Confirmar no WebView2 empacotado; a execução em Vite não valida a CSP de produção. |
| **QA-028 / P2 / Inspeção** | Prévia aceita imagens, texto/Markdown e HTML; PDF não entra em nenhum ramo e retorna `null` (`WorkPanel.tsx:16–55`). | Mostrar prévia PDF ou uma ação/explicação explícita; `008 AC-014`. Abrir arquivo externamente não é prévia no Aura. |

## Captura, privacidade e gravações — interface incompleta

As políticas existentes foram inspecionadas, sem alterá-las. Os achados abaixo são ausências de controles/representações, não evidência de que exclusões, cifragem ou permissões do backend falharam.

| ID / prioridade | Problema, impacto e evidência | Comportamento esperado / contrato |
| --- | --- | --- |
| **QA-029 / P2** | `Privacy.tsx:19` reutiliza os minutos existentes ou 10; o usuário pode selecionar Buffer recente, mas não editar a duração. | Escolher N entre 1–30 minutos; `004 AC-013`, buffer de áudio em `005`. |
| **QA-030 / P2** | Tela não oferece edição de Retenção em dias/espaço. | Configurar os limites e saber quando serão aplicados; `004 AC-016`, `005 AC-006`. Isso não afirma que a rotina interna de limpeza não existe. |
| **QA-031 / P2** | Registro visual mostra tempo relativo, fonte, ferramenta/requester e decisão crua. Não mostra ligação com Conversa, data/hora exata, miniatura ou indicação detalhada de cobertura (`Privacy.tsx:112`). | Auditoria rastreável das capturas entregues; `004 AC-008`. |
| **QA-032 / P2** | Gravações oferecem iniciar/parar/exportar/excluir, mas não player, duração explícita ou anexar à conversa (`Recordings.tsx`). | Reproduzir e anexar no Aura com duração; `004 AC-015`, `005 AC-004`. Gravações reais não foram criadas nesta sessão. |
| **QA-033 / P1** | Menu de contexto inspecionado tem tela, região, janela, seleção e arquivo; não há “últimos N minutos” de tela/áudio, escolha Microfone/Sistema/Ambos ou recorte combinado. | Usuário anexar o Buffer recente pela interface; `004 AC-014`, `005 AC-007/009`. Ferramentas que o agente pode chamar não substituem o comando explícito do usuário. Evidência: [menu compacto](evidencias-2026-10-02/26-demo-contexto-compacto-0.jpg). |

## Produtividade, diagnóstico e distribuição

| ID / prioridade / estado | Problema, impacto e evidência | Comportamento esperado / contrato |
| --- | --- | --- |
| **QA-034 / P2 / Inspeção** | Configurações de Voz não incluem escolha de voz TTS, leitura automática ou escolha de provedor TTS. Botão Ouvir existe nas mensagens. | Escolher voz offline/nuvem e leitura automática, com consentimento no caso de nuvem; `009 AC-005/006`. Não foi comprovada qualidade/saída de áudio TTS real. |
| **QA-035 / P2 / Inspeção** | Handler de teclado do Overlay não implementa `Ctrl+Shift+L` nem `Ctrl+Shift+Enter`. A entrada trata Ctrl+Enter como direcionamento de turno, sem distinguir a combinação de inserção prevista. | Atalhos para ouvir e inserir no Aplicativo anterior; `009 AC-005/009`. O botão Inserir é outro caminho e não foi acionado contra um app pessoal. |
| **QA-036 / P2 / Inspeção** | `Profiles.tsx` tem `defaultModel` no objeto, mas nenhum controle para escolher o modelo. | Configurar todos os campos do Perfil de aplicativo disponíveis no domínio, especialmente modelo padrão. Edição de nome/instruções/modo existe e **não** foi classificada como ausente. |
| **QA-037 / P2 / Inspeção + observação** | Diagnostics exibe versão, engine, porta, pasta e contagens. Não apresenta estado detalhado de Gateway/MCP/worker, fontes ativas, conta e espaço em disco. | Diagnóstico completo do pipeline; `010 AC-011`. A exportação ZIP funcionou; o achado é a tela resumida. |
| **QA-038 / P1 / Inspeção** | Chave pública do updater ainda é `REPLACE_WITH_THE_PUBLIC_KEY_FROM_...` em `tauri.conf.json:83`. | Publicação com chave válida e teste de atualização assinada; `010 AC-004`. Não foi realizado download/aplicação de atualização nem atribuído um erro de rede ao botão Verificar atualizações. |
| **QA-039 / P2 / Inspeção** | Onboarding pode ser pulado, mas não há controle em Configurações para retomá-lo. | Retomar primeiros passos nas Configurações; `010 AC-006`, `FR-003`. |
| **QA-040 / P2 / Inspeção** | Texto `onboarding.shortcut.body` ensina “Esc para fechar” (`i18n/pt-BR.ts:80`, usado por `FirstRun.tsx`). Handler atual e revisão 2 dizem que Esc nunca esconde Overlay. | Tutorial coerente com o comportamento: atalho/botão para recolher, Esc para menus/voz; `001 AC-005`. |
| **QA-041 / P2 / Divergência** | Iniciar o executável normalmente mostrou Overlay. `001 AC-001` exige iniciar somente na bandeja; há caminho `--background` no shell. | Decidir e atualizar contrato/onboarding ou ajustar inicialização. Não confundir abertura manual com o teste de logon/autostart, que não foi executado. |
| **QA-042 / P2 / Risco** | Configuração nativa tem `minWidth:480`, `minHeight:64` para o Overlay e não muda esse mínimo ao expandir. O layout expandido exige cabeçalho, corpo, entrada e rodapé. | Impedir redução do expandido até uma altura em que controles ficam cortados, mantendo mínimo próprio. Exige arraste manual conclusivo; o mínimo de 64 não foi declarado um defeito do compacto. |

## Matriz de cobertura executada

“Inspecionado” significa que a tela/ligação foi analisada; não significa que todas as ações dela foram executadas.

| Área | Execução nesta sessão | Limite / próxima validação |
| --- | --- | --- |
| Inicialização e Overlay | Lançamento real; invocar/recolher pelo atalho global; manter visível ao perder foco observado; expandir ao enviar, compactar e menus no demo | Arraste livre, persistência de resize, foco de retorno, duas telas/DPI e redimensionamento expandido pendentes. |
| Aparência | Geral real inspecionado; Sistema → Claro no demo; atualização visual imediata | Escuro real observado; não medidos contraste AA, alto contraste ou reduzir movimento. |
| Conta e modelos | Conta real conectada inspecionada; turno real concluído; login/cancelamento nativo isolado; login mock de navegador | Novo login completo, logout/revogação, renovação, segunda conta, limites do plano e fallback BYOK não executados. |
| Conversa | Prompt real sem ferramentas retornou `AURA QA OK`; fake renderizou tabela/Markdown; menus `/` e `@`, Histórico e painéis laterais | Longa resposta/interrupção/steering, retomar após crash/idle, aprovações e formulários reais não concluídos. |
| Histórico | Lista aberta no demo e conversa de QA encontrada; busca e ações analisadas no código | Fixar/arquivar/excluir/renomear e >50 conversas não executados. Não se apagou histórico pessoal. |
| Provedores | Lista e formulário reais; provedor local sem chave cadastrado no demo; estado de erro por endpoint indisponível; persistência após reinício | Provedores pagos, autenticação/chaves reais, imagens e cada protocolo em rede externa não exercitados. Teste de contrato real app-server/BYOK com upstream simulado passou. |
| Privacidade | Tela real, fontes, exclusões, gravações e registro inspecionados; exclusão do Overlay real em captura observada | Não se alteraram permissões, exclusões, Pausa ou captura contínua. Não testados OBS/Teams, mosaico de exclusões e buffers longos. |
| Voz/áudio | Catálogo, opções e dispositivo real inspecionados; botão de microfone no demo não produziu transcrição com clique breve | Nenhum modelo instalado no perfil real; sem download/worker/transcrição de fala real, loopback, VAD, qualidade/latência ASR ou TTS audível. Clique breve inconclusivo não foi usado como prova de falha do ASR. |
| Anexos | TXT sintético ingerido pelo host nativo demo: Chip “2 linhas · ~25 tokens”; arquivo UTF-8 `.xyz` aceito como texto, “1 linhas” | A extensão desconhecida com texto legível não foi classificada sozinha como bug. Faltam corpus PDF/scan/Office/planilha/áudio/vídeo, corruptos/binários/limites, redução e leitura seletiva via agente real. |
| Extensões | Tela real vazia, seis comandos embutidos; comando `qa-teste` cadastrado e listado no demo; formulário MCP em inglês inspecionado | Sem instalação de Skill, importação de configurações pessoais, servidor externo/OAuth e alterações de aprovação. |
| Arquivos e alterações | Painel vazio e combinação com Histórico exercitados | Geração real, diffs, abrir/revelar e prévias reais de cada formato pendentes. |
| Perfis | Tela real e formulário analisados | Aplicação automática com apps reais, matching de títulos/modelos e captura por perfil não exercitados. |
| Atalhos | Tela real inspecionada; invocação global e atalhos de modos/Histórico no demo | Não alterados atalhos globais pessoais; conflito de registro, duplo Ctrl e voz global não validados. |
| Diagnóstico | Tela real e demo; exportação pela UI para `target/qa-diagnostico-demo.zip`, ZIP de 1.987 bytes aberto e entradas conferidas | Pacote demo contém README, summary/settings/privacy/providers/MCP e log vazio. Não comprova redação de segredos reais inexistentes no perfil. Apagar todos os dados não executado. |
| Sobre e atualização | Tela real e Verificar atualizações acionado | Sem evidência conclusiva de resultado do check; nenhuma atualização instalada. Instalador, assinatura, desinstalação e logon/autostart não exercitados. |
| Acessibilidade | Nomes/árvore acessível examinados; teclado para input/modos/Histórico | Narrador, anúncios de frases, fluxo inteiro por teclado e contraste AA pendentes. |
| Desempenho | Bench portátil executado e métricas abaixo | Não mede abertura a quente/CPU/memória Windows sem `--pid`; sem baseline e sem cenário ocioso de 5 min. |

## Testes automatizados executados

| Comando | Resultado | O que comprova |
| --- | --- | --- |
| `pnpm -C apps/desktop test` | **PASSOU: 12 arquivos, 58 testes** | Componentes/lógica cobertos pela suíte Vitest. Não executa resize real do WebView2 nem todos os fluxos de produto. |
| `pnpm -C apps/desktop typecheck` | **PASSOU** | Verificação estática TypeScript. |
| `cargo test --workspace --exclude aura-desktop` | **PASSOU**; probes de app-server real ignorados por padrão | Suíte Rust do workspace, incluindo contratos/host/adaptadores cobertos e teste Windows de encode/decode MF. Não substitui observação do desktop. |
| `cargo test -p aura-app --test real_app_server real_app_server_byok_turn_and_mcp_tool -- --ignored` com `AURA_CODEX_BIN` apontando ao binário fixado `rust-v0.159.0` | **PASSOU: 1 teste** | Processo real de app-server, gateway e ferramenta MCP; upstream de inferência simulado, sem uma chave paga real. |
| `cargo build -p aura-desktop --release --features demo` | **PASSOU** | Build usado na inspeção visual nativa. Executável original restaurado depois. |
| `cargo run -p aura-bench -- all` | **PASSOU, exit 0** | Cenários portáteis com app-server/frame sintéticos; não é resultado de performance completa do desktop real. |

Bench em build **debug**: `first_delta.p50_ms=0,20`, `first_delta.p95_ms=0,34` (budget registrado 50 ms), `capture.p95_ms=321,18` (sem budget/baseline nessa métrica). Essa última medição merece perfil em release diante do alvo de captura de 300 ms, mas não demonstra sozinha que a UI descumpre o requisito. A saída “No regressions” sem baseline não comprova ausência de regressão histórica.

Suite WebdriverIO do repositório **não executada**: `tauri-driver` está instalado, mas `msedgedriver.exe` compatível com o WebView2 não estava disponível no PATH nem nas pastas pesquisadas. Os cenários visuais foram exercitados por Computer Use; isso não transforma a suite WebdriverIO em aprovada. Referência: `apps/desktop/e2e/README.md`.

Resíduo do teste com app-server real: o teste passou, mas deixou `crates/aura-app/aura-e2e9qtogT/` com dados sintéticos. Não foi encontrado processo com esse diretório nos argumentos (além do próprio comando de inspeção). A remoção recursiva desse diretório foi rejeitada pela aprovação automática com o motivo genérico “blocked by policy”; o diretório foi preservado. Investigar a limpeza do `TempDir` no Windows antes de tratar a suíte como livre de resíduos. Ele não pertence ao perfil pessoal do Aura.

## Evidências e uso do documento

Capturas locais em [evidencias-2026-10-02](evidencias-2026-10-02/). As imagens usam extensão `.jpg` correspondente ao formato retornado pela ferramenta. Arquivos `.txt` são snapshots auxiliares: alguns retornaram uma árvore da tela anterior durante a navegação; por isso as evidências visuais devem ser interpretadas pela imagem, e não por um snapshot auxiliar isolado. Imagens com o navegador de autenticação legível ao fundo foram descartadas. Capturas não foram commitadas.

Referências principais:

- Configurações reais: 01 Geral, 02/03 Provedores, 04/05 Privacidade, 06/07 Voz, 08 Extensões, 09 Perfis, 10 Atalhos, 11 Diagnóstico, 12 Sobre.
- Conversa real: `13-conversa-real.txt` (conteúdo sintético, sem imagem da conta).
- Demo: 15 cadastro de provedor; 19/20 compacto; 21 conversa expandida; 23 comandos; 24 Histórico; 25 conflito entre painéis; 26 contexto; 28 Aparência; 29 tema Claro; 30 inglês em Diagnóstico; 31 comando rápido; 32 MCP; 33/34 anexos.

Ordem sugerida para correção/verificação: **QA-001, QA-002 e QA-003** (janela e primeiro uso); **QA-007/008/018/020/023/026/033/038** (fluxos bloqueados ou controle do usuário); validar **QA-027/042** no pacote real; depois fechar as lacunas P2 e atualizar contratos onde houver decisão de produto. Cada correção deve ganhar uma regressão na seam adequada e uma execução nativa para os problemas de janela.

Este documento é uma auditoria de QA; não altera estados/evidências do runner Hybrid nem as projeções `todo.md`, `verification.md` e backlog.

## Reteste e correção — 03/10/2026 (em andamento)

Atualizacao TK-006: **QA-006 corrigido/verificado (EV-017)**. Host red confirmou `cancelled,failed`: AuthService ja emitiaCancelled e HostloginreemitiaFailed ao observar mesmoJoinHandle. Removida emissao duplicada; AuthService permanece fonte unica, progresso retransmitido. Duas tentativas emHost produzem somenteCancelled; falha real state mismatch ainda chega uma vez. UI limpa progressoCancelado sem aviso. Tambem corrigido mock que emitiaCompletedaposCancelar: contador descarta resposta atrasada. RedUI aguardouPromise real e confirmouwelcomeindevido; primeira tentativafakeTimers travouharness e foi descartada como redinvalido.

**27/27Host, 78/78UI**, workspaceRust, typecheck/golden/build/fmt/diff e clippycom/semfeaturee2e passaram. Native`auth-cancel-native-final.log`1/1: duas tentativas PT/EN com `waitingBrowser,cancelled,waitingBrowser,cancelled`, zeroFailed, nenhuma conta. Foto `target/qa-tools/auth-cancel-native-green.png` mostraLoginCardENsemfalha. Executavel SHA256 **A793A3CB23C907F51CC31CF95F5AAF63F43D858035D50102BB2A19E0F51716FE**, build `e2e,tauri/custom-protocol` semdemo: Host/callback/IPC/eventos/UI/Windows reais; autorizacaoexterna controlada por pagina127.0.0.1viafeaturee2eexcluidadopadrao, semtoken/JWKS/contapessoal. **NaoaprovaOAuthcompleto**. PreviaVite real permaneceuLoginaposdeadline e nova tentativa concluiuWelcome/Input; foto `auth-cancel-preview-green.jpg`.

QA-007 corrigido e verificado em EV-019: seletor controlado, preferência salva e aplicada pelo Host ao PTT. Patch ausente mantém a preferência; `null` retorna ao padrão do Windows. TDD confirmou seleção ignorada, despacho Default indevido e UI sem persistência. Greens cobrem JSON, limites, banco reaberto, prioridade explícita e UI. UI79/79, suíte Rust, golden Rust/TS, tipos, build, clippy, fmt e diff passaram. No release normal `697DBDE547A950E6380312ECF9A3BD6737E1990269E40A9E3E88B76854BDE5DD`, os testes `microphone-native-initial.log` e `microphone-native-restart.log` passaram 1/1 cada: Fifine e CABLE abriram/cancelaram WASAPI; reiniciar manteve CABLE; escolher padrão limpou a preferência. Não foi executado ASR nem guardado áudio. Revisão local Standards/Spec sem bloqueante.

QA-008 corrigido e verificado na fronteira de resultado do ASR (EV-020). O red React e o executável TK-007 retiveram apenas uma frase após dois ciclos iguais. AppStore agora identifica cada conclusão ordenada por revisão; InputBar consome a conclusão, preserva frases iguais de ciclos diferentes e ignora duplicatas terminais. UI82/82 cobre também campo limpo, eventos agrupados e dois autoenvios pela orquestração real. Tipos/build/diff passaram. No release normal `244AE84A21082999168805CA8089AA4EB8D93C8D10B92BFAB72ADB51C03D7E03`, `voice-repeat-native-green.log` passou 1/1 com eventos sintéticos de resultado e UI/IPC/WebView reais. Não aprova modelo ASR, fala real ou latência. Regressões de microfone inicial/reinício passaram 1/1 cada nesse release. Revisão local Standards/Spec sem bloqueante.

QA-009 corrigido e verificado quanto à configuração e ao parâmetro de idioma (EV-021). JSON `asrLanguage: null` agora limpa o idioma fixo; patch omitido mantém. UI e atalho local deixaram de derivar `pt` do idioma da interface; Host mantém precedência da preferência fixa. Rótulo “Automático” descreve detecção. Red React PT enviava pt; red core/nativo mantinham es após escolher automático. UI85/85, core/app/golden, tipos/build/clippy/fmt/diff passaram. Release normal `8F5C9AC15A31D805DFA6D5FA9E72699C537FF4CE24176ECA18C888B74A6B6B99`: hold-green e restart 1/1 cada, seletor es->null/persistência e ciclos reais Listening/Empty PT/EN. Parâmetros exatos foram verificados no bridge externo React; observador nativo não pôde substituir invoke imutável e essa tentativa não é red de produto. Não aprova ASR acústico, latência ou atalho físico. Voice-repeat nativo 1/1 passou. Clique rápido deixou escuta ativa por corrida entre início/soltura; risco confirmado e encaminhado ao QA-010, ainda sem correção neste ticket.

QA-010 corrigido e verificado em EV-022. Botão nativo alterna por clique/Enter/Space, não conclui ao sair com o cursor, e bloqueia operações pendentes/transcrição. Primeiro release com disabled perdeu foco para BODY e Space falhou; aria-disabled com guard mantém BUTTON e teclado operacional. Escape agora cancela também Partial, cujo red React foi confirmado. UI92/92 cobre dois cliques, teclas, início pendente, retry, Ctrl+Space sem ativação duplicada e cancelamento sem texto. Tipos/build/diff passaram. Release normal `805B30DFE31F825661E95584F495F69EA6D64037784DF9CC413F8DEECC2D01F5`: voice-toggle-native-final 1/1, Enter->Space sem refocar, mouse/pointerleave/Escape reais. Regressões nativas idioma inicial/reinício, repeat e microfone inicial/reinício passaram 1/1 cada. Resize atual passou 9/9 em duas telas a 100%; não valida DPI125/150. Não aprova ASR acústico/modelo/latência ou atalho físico. Revisão local Standards/Spec sem bloqueante final.

QA-011 corrigido e verificado (TK-011). Sem modelo, o ditado (botão e atalho) mostra cartão inline com o modelo recomendado, tamanho e Baixar; progresso, cancelar sem alerta, repetir, seleção persistida ao concluir e nenhuma captura automática. Produção sem worker informa reparo em vez de usar transcritor falso. A execução real revelou um defeito adicional: num worker sem GPU, a regra “GPU dedicada → Whisper Turbo” recomendava um modelo que levou 100–160 s para 3 s de fala e terminava em `transcription timed out`; a recomendação agora exige inferência em GPU do worker (`--capabilities`). Release `E593D5B0…CA7F`: `model-install-native-final.log` 3/3 com download real de Parakeet V3 (640 MB, cinco SHA-256 conferidos), reinício e transcrição real pela UI de fala sintética por cabo virtual (“The quick brown fox jumps over the lazy dog.”). Regressões de voz 1/1 cada e resize 9/9. Limites: worker com GPU não testado; build nativo do worker usa instruções da CPU de compilação (risco de distribuição a tratar fora deste achado).

QA-012 corrigido e verificado (TK-012). Cada modelo do catálogo mostra barras de velocidade e precisão, idiomas, requisito CPU/GPU recomendada e RAM mínima, além de tamanho, licença e selos; nomes e descrições seguem o idioma da interface. Release `E6243060…8FBF`: `tk012-final-voice-catalog.log` 1/1 em inglês e português com literais de `models.toml`.

QA-013 corrigido e verificado (TK-013). Voz nas Configurações agora tem Testar microfone e Testar áudio do sistema com medidores ao vivo (~28 Hz), seletor da saída gravada como Áudio do sistema (persistido e usado também por gravações/buffers) e tom de teste. Dispositivo escolhido ausente usa o padrão com aviso, no teste e no ditado. A execução real revelou e corrigiu: fallback silencioso no adaptador Windows e travamento ao parar captura de loopback em silêncio. Release `1330E10C…7450`: `tk013-green4-audio-test.log` 1/1 com cabo virtual e fala sintética.

QA-014 corrigido e verificado (TK-014). O seletor de modelo mostra capacidades por modelo (Imagem, Ferramentas, Raciocínio ou Somente texto) e uma seção de esforço de raciocínio com apenas os níveis suportados; trocar para modelo sem suporte descarta o esforço. Release `0C9BA802…EBE1`: com upstream local controlado, o esforço Alto chegou ao provedor como `reasoning.effort: high`.

QA-015 corrigido e verificado (TK-015). Histórico tem Renomear inline (Enter salva, Escape cancela). A execução real revelou que conversas de provedores próprios (BYOK) nunca apareciam no Histórico porque a listagem pedia só o provedor padrão do app-server; corrigido. Release `4852618E…CA88E`: conversa real renomeada pela UI e preservada após reinício.

QA-016 corrigido e verificado (TK-016). Histórico pagina por cursor com Carregar mais; nova busca reinicia. Release `D0005280…D357`: com mais de 50 conversas reais, a mais antiga só aparece após Carregar mais.

QA-017 corrigido e verificado (TK-017). `/compactar` aparece no menu, compacta a conversa pelo app-server sem ir ao modelo como prompt e mostra o item «Conversa compactada». Release `4D9E1964…FDE7`: com upstream local determinístico, o turno seguinte levou o resumo no lugar do histórico.

QA-018 corrigido e verificado (TK-018). Ctrl+V com imagem (PNG, JPEG, WebP, GIF) cria um Chip de imagem e a imagem vai ao modelo; texto continua colando normalmente. Release `A93227EE…1022`: colagem real da área de transferência do Windows chegou ao modelo como imagem.

QA-019 corrigido e verificado (TK-019). Provedores salvos têm Editar: nome, URL e substituição opcional da chave sem excluir/recriar. Release `CAA738B2…AF02`: edição real manteve o id e as conversas existentes passaram a usar a nova URL.

QA-020 corrigido e verificado (TK-020). Personalizado oferece formato Responses/Chat Completions/Anthropic e cabeçalhos extras (também na edição). Release `7234891D…D8`: requisições reais usaram o caminho do formato escolhido e levaram o cabeçalho configurado.

QA-021 corrigido e verificado (TK-021). Provedores aceitam modelos digitados à mão com id, nome, contexto e capacidades; a descoberta não os apaga e um provedor sem `/models` deixa de aparecer como erro. Release `F20F3BBA…C4D4`: modelo manual apareceu no seletor e foi usado no turno real.

QA-022 corrigido e verificado (TK-022). Skills mostram origem (Aura, Usuário, Sistema), têm chave ativar/desativar que vale para conversas novas e persiste após reiniciar, e as Skills do Aura podem ser editadas. Release `D4908B22…0DAD`: verificado com o app-server real.

QA-023 corrigido e verificado (TK-023). Argumentos MCP têm semântica definida (aspas agrupam, `""` é vazio, caminhos Windows sem escapes) com pré-visualização e erro para aspas abertas. Release `F263175C…5B03`: servidor real numa pasta com espaços recebeu o argv exato.

QA-024 corrigido e verificado (TK-024). Cada ferramenta MCP pode ser desligada individualmente e deixa de ser oferecida ao agente em conversas novas. Release `1546F528…4CC9`: verificado com servidor MCP real e app-server real.

QA-025 corrigido e verificado (TK-025). Servidores MCP mostram Conectado ou Erro com o motivo, e “Ver log” traz as últimas linhas de erro do próprio servidor. Release `2F08AC78…14FB`: verificado com servidores MCP reais (um conectado, outro falhando).

QA-026 corrigido e verificado (TK-026). Configurações mostram os fatos lembrados, permitem esquecer um a um, editar o resumo/registro e esquecer tudo; conversas novas recebem só o que restou. Release `7BD6A418…39DC`: verificado com o app-server real.

QA-027 verificado sem alteração de produto (TK-027). No WebView2 empacotado a prévia HTML em `srcdoc` funciona apesar de `frame-src 'none'` e permanece isolada: sem scripts, sem imagens/requisições de rede. Release `7BD6A418…39DC`: E2E com arquivo real do workspace e captura de tela.

QA-028 corrigido e verificado (TK-028). PDFs no painel Arquivos mostram a prévia do texto das primeiras páginas; PDF sem texto (digitalizado) recebe explicação e a ação Abrir. Release `392B583E…71D8`: verificado com arquivos reais do workspace.

QA-029 corrigido e verificado (TK-029). O Buffer recente tem duração editável de 1 a 30 minutos por fonte, validada também no Host. Release `D982A0A3…8AE6`: valor definido pela UI persiste após reiniciar.

QA-030 corrigido e verificado (TK-030). Privacidade permite configurar retenção em dias e espaço (e se vale para gravações manuais), explicando que os limites são aplicados a cada novo trecho gravado. Release `C7CFF898…805B`: valores salvos pela UI persistem após reiniciar.

QA-031 corrigido e verificado (TK-031). O Registro de acesso mostra data/hora exata, fonte, ferramenta, decisão e motivo localizados (consentimento, negação, sem resposta, N áreas cobertas), miniatura selada do que o agente recebeu (sob demanda) e botão para abrir a Conversa. Release `D45ADD2E…E634`: captura real pedida pelo agente via MCP registrada como permitida pelo usuário, com miniatura e vínculo funcional.

QA-032 corrigido e verificado (TK-032). Gravações mostram duração explícita, têm player no próprio Aura (áudio por fonte e vídeo da tela) e podem ser anexadas à Conversa aberta; o áudio anexado é transcrito pelo modelo de fala instalado. Release `72D500AB…7A0D`: duração 0:10 para 10 s gravados, reprodução real no WebView2 e anexo transcrito no Overlay.

QA-033 corrigido e verificado (TK-033). O menu de contexto oferece `@últimos minutos`: o usuário escolhe N minutos, Tela e Áudio (Microfone, Sistema ou Ambos) e anexa um Recorte com até 8 quadros e a Transcrição rotulada do mesmo intervalo; os arquivos ficam no Workspace da Conversa. Corrigidos no caminho: segmento de áudio aberto ignorado (até 10 s finais perdidos) e limite de 2 MB do Gateway que derrubava turnos com várias imagens. Release `30D1CB13…4CC6`.

QA-034 corrigido e verificado (TK-034). Configurações › Voz agora tem Leitura em voz alta: escolha entre vozes instaladas do Windows (offline, padrão) ou voz na nuvem de um provedor BYOK, com confirmação única antes de enviar texto; leitura automática das respostas; botão de teste. Release `2C3BDB8D…E672`.

QA-035 corrigido e verificado (TK-035). No Overlay, Ctrl+Shift+L lê/para a última resposta e Ctrl+Shift+Enter insere a resposta (ou o bloco de código selecionado) no aplicativo anterior, restaurando a área de transferência. Release `1AD8A59D…7480`.

QA-036 corrigido e verificado (TK-036). O Perfil de aplicativo permite escolher o modelo padrão (plano ChatGPT ou modelo de provedor) e o Overlay aberto sobre o app usa esse provedor/modelo na conversa nova. Release `00A33823…6E1B`.

QA-037 corrigido e verificado (TK-037). Diagnóstico mostra o pipeline completo: motor do agente, Gateway (respondendo ou não), servidores MCP com ferramentas/erros, worker de fala, capturas ativas, conta (e-mail mascarado) e espaço em disco. Release `A7C6F270…7E49`.

QA-038 mitigado, ainda bloqueado (TK-038). O app agora detecta a chave placeholder e informa que as atualizações automáticas não estão configuradas, e o workflow de release falha enquanto a chave pública real e o secret de assinatura não forem configurados. A correção definitiva — chave do mantenedor e teste de atualização assinada publicada — não foi feita nem simulada.

QA-039 corrigido e verificado (TK-039). Configurações › Geral permite retomar os primeiros passos, que reaparecem no Overlay. Release `46BBFEE8…18B4`.

QA-040 corrigido e verificado (TK-040). O guia de primeiros passos não ensina mais "Esc para fechar": explica atalho/Minimizar e o papel do Esc. Release `8B25626F…B882`.

QA-041 resolvido conforme o contrato (TK-041). Iniciar o Aura deixa apenas o ícone da bandeja e avisa por notificação como abrir; executar de novo abre o Overlay. Release `E2693A14…99A3`.

QA-005 foi revalidado em EV-018; QA-006 em EV-017; QA-007 em EV-019; QA-008 em EV-020; QA-009 em EV-021. Mudanças posteriores tornaram essas evidências stale. TK-001–009 permanecem implemented/stale aguardando consolidação; TK-010 done/passed. Outros achados pendentes; QA-001 DPI/relato de largura continuam não verificados. Próxima fatia QA-011; **esforço não concluído**.

Atualizacao TK-005: **QA-005 corrigido e verificado (EV-016)**. A causa foi isolada: chamadas concorrentes de getBridge criavam mocks distintos; o subscriber recebia zero eventos de login. Promise de inicializacao compartilhada corrige mock/Tauri, protege reset/injecao contra import antigo e permite retry de rejeicao. **76/76 UI**, typecheck/build/diff passaram. Previa Vite real confirmou coldreload ->loginChatGPTmock ->waiting ->welcome ->input editavel/Enviarhabilitado sem refresh; foto `target/qa-tools/bridge-preview-green.jpg`. Smoke nativo producaoSHA256 `9886E59523207542E02E2363ADF88B0B9B0495B325853A3D5228E49E16C0D5E5`, `bridge-native-smoke.log`3/3. Nenhuma autorizacaoOAuthreal atribuida a esse teste.

O append tecnico ao plan e a mudanca de bridge invalidaram EV011–015 no runner; seus resultados historicos permanecem documentados, mas TK001–004 ficam implemented/stale ate consolidacao atual. QA004 passou novamente no smoke3/3 desta producao; os outros caminhos nao foram alterados neste ticket. Proxima fatia QA006; esforco continua ativo e os achados restantes nao estao concluidos.

Estado mais recente apos TK-004: **QA-004 corrigido e verificado (EV-011)**, producao SHA256 `ECC7FD46E67E4410404E7A0A29F4F39D60847FBEFCA564FA0BFCC9A6617C8869`. Dicionarios canonicos alimentam UI, titulos/tray nativos e prompts builtin; trocar idioma atualiza paginas existentes. Anexos ganham metadados opcionais de contagem, sem traduzir filename/conteudo/payload; chips legados continuam aceitos. Comandos personalizados e enabled preservados, sem regravar templates no banco. Golden aditivo regenerado/retestado.

`localization-native-complete.log` **3/3**: duas janelas reais, tituloEN/PT/EN, Diagnostics **Stopped** no estado real, MCPstdio/HTTP, catalogo/sugestoes, TXT1linha/1line e nome `1 linhas.txt` preservado, comando personalizadoPT intacto, cinco itens do popup nativo trayEN/PT/EN sem restart. Tray foi aberto pelo callback da dependencia e lido via HMENU do Windows; isso nao comprova clique fisico/descoberta do icone. **Ready(mock), plural2lines/2linhas** e legados foram validados em React/host, nao atribuidos ao estado nativoStopped. Reds comportamentais confirmaram os relatos, incluindo titulo e tray no executavel original. Tentativas `localization-native-final/final2` falharam na limpeza de texto WebDriver e foram descartadas; o roteiro corrigido usa Ctrl+A/Backspace e confere valor, `final3`/`complete` passaram.

**72/72 UI**, typecheck, workspaceRust, goldenRust/TS, fmt/diff-check, clippyapp/desktop/core e probeaura-win `-D warnings` passaram. Regressao na mesma producao: `resize-after-localization.log` **9/9** (EV-014passed042/EV-015partial001), `panels-after-localization.log`1/1 (EV-013passed002), `providers-after-localization.log`1/1 e `localization-real-server.log`1/1 turnoenginequente+MCP (EV-012passed003). Evidencias anterioresEV007–010stale. QA-001 ainda **parcial**, com relato especifico de largura e DPI nativo125/150pendentes. TK002/003/004done; outros achados ainda pendentes. Esforco nao concluido; proxima fatia QA-005.

Esforço: `specs/011-correcao-auditoria-e2e`, baseline `5b130b69ac3ede8dc5a8fdc6b0e1ceccac7a8393`. Os 42 achados têm aceites; QA-001/042 são a primeira fatia (TK-001). QA-042 foi corrigido e verificado no ambiente descrito abaixo. QA-001 permanece parcialmente verificado; o esforço não está concluído.

O Microsoft Edge WebDriver 154.0.4258.53 foi obtido em `target/qa-tools/edge-154`, correspondente ao WebView2 instalado. Isso remove o impedimento de driver da sessão original. O executável original foi preservado em `target/qa-tools/aura-original-2026-10-03.exe`, com o mesmo SHA-256 `592C2010FE11AC9D6B8B36BF04AFAC1D5E012AF01595CE82490C8251A15CF055`. Perfis de testes são isolados em `target/qa-profile-resize-*`; não usam a conta ou configuração pessoal.

| Achado | Confirmação e resultado atual | Evidência / limite |
| --- | --- | --- |
| QA-001 | Larguras 480/640/900 acompanham janela, viewport e shell no original, DPI 100%. Arrastes nativos East/West ampliam e reduzem corretamente; amostras durante o arraste também acompanham. A variante de **altura** foi reproduzida: janela 900 × 400, shell 900 × 80, 320 px vazios. Correção reage ao resize da janela, compara a altura real e cancela efeito obsoleto ao trocar modo/menu. Release corrigido retorna 900 × 80. Passaram menus, reabertura, todas as bordas/cantos do expandido e troca entre DISPLAY2 (1920×1080) e DISPLAY1 (1366×768), ambas 100%. | `apps/desktop/e2e/specs/resize.e2e.ts`; probe `crates/aura-win/examples/qa_resize.rs` com SendInput/GetClientRect reais. Logs `target/qa-tools/resize-samples-baseline.log`, `resize-height-baseline.log`, `resize-production-final.log`. **O relato específico de largura ainda não foi reproduzido; DPI nativo 125/150% não está aprovado. Estado: parcial.** |
| QA-042 | Corte reproduzido no original: altura interna 64, entrada termina em 126. Corrigido com mínimo por modo, compensação da borda invisível e clamp de placement expandido legado. Arraste real chega a mínimo interno **360**, entrada termina em **324**, sem corte; voltar ao compacto permite altura 80. Placement legado 640×64 também volta a 640×360. Testes puros confirmam mínimos físicos 600×450 em DPI125 e 720×540 em DPI150. | Red em `target/qa-tools/resize-height-baseline.log` e `resize-legacy-red.log`; green de produção **9/9** em `resize-production-final.log`. **Corrigido e verificado no Windows 11, WebView2 154.0.4258.53, duas telas a 100%. Os testes puros de DPI não substituem arraste nativo nessas escalas.** |

Verificações executadas neste trabalho: `pnpm -C apps/desktop test` **58/58**, `pnpm -C apps/desktop typecheck` **passou**, `cargo test --workspace --exclude aura-desktop` **passou** (probes ignorados continuam ignorados). Frontend e release nativo de QA com `--features demo,tauri/custom-protocol` compilaram; o frontend é empacotado, sem dependência de Vite. A inferência/captura do backend demo continua sintética e não serve para aprovar os achados dessas áreas. Após qualquer nova mudança de produto, reavaliar evidência e executar os checks afetados antes de fechar o ticket.

O reteste final de resize usa **build de produção**, `cargo build -p aura-desktop --release --features tauri/custom-protocol`, sem demo; SHA256 `D6B3A7950E7DA3A2539A4FB42D134ECC3D14EF3101A4BF0311FA6503F56A5384`. Não enviou turno de inferência, instalou ASR ou capturou tela; valida as janelas reais, frontend empacotado, IPC, persistência e mouse Windows. Clippy com `-D warnings` executado para os módulos alterados; revisar evidência após mudanças subsequentes. Próxima fatia: QA-002, mantendo a pendência específica de QA-001 aberta.

TK-002 / QA-002 em execução: dois testes React confirmaram independentemente que abrir Arquivos mantinha Histórico e que Ctrl+H mantinha Arquivos. No frontend anterior empacotado em app nativo demo, a execução WebDriver confirmou ambos visíveis e **0px de largura para a conversa em janela de480px** (`panels-native-red-demo.log`, `panels-width-native-red.log`). Exclusão mútua atômica implementada; abaixo de1024px os painéis sobrepõem o corpo, acima ficam ao lado. Os **60 testes UI** e typecheck passaram. Build nativo novo e reteste pendentes; QA-002 ainda não está verificado. A tentativa inicial no backend de produção criou uma conversa mas não conseguiu preparar seu Histórico com provider local sem modelo; essa falha de preparação não foi contada como red de layout. Nenhuma inferência paga foi usada.

As evidências EV-002/003 de TK-001 foram marcadas **stale** pelo runner após mudanças no OverlayApp do TK-002. Os resultados históricos acima permanecem registrados, mas precisam de reteste do frontend atual antes de serem considerados atuais.

TK-002 foi verificado posteriormente: `panels-native-green.log` passou nas duas direções com rascunho/Escape. Conversa480/640 nas janelas480/640; janela1024 →704 com Arquivos e768 com Histórico. `resize-after-panels-demo.log`9/9 e `resize-after-panels-production.log`9/9 passaram; produção SHA256 A924CA5C3306DFBEBFC89EB2CA824C9CD1FCB117F1D8643F7FF2474B66EAB2C2. EV-005 aprovou AC-002 antes da próxima alteração. As evidências desse layout foram invalidadas após TK-003 alterar OverlayApp/session; serão renovadas no frontend atual. Mantém-se pendência de QA-001 em DPI125/150 e relato de largura.

TK-003 / QA-003: red de produção `providers-native-red.log` confirmou Overlay montado sem textarea após salvar primeiro provedor. TDD do host confirmou falta de notificação em cadastro, remoção e descoberta. Novo evento IPC `providersChanged` tem payload `{}`, sem segredos; golden regenerado e retestado. Overlay relê catálogo, conserva seleção válida e rascunho, escolhe local quando não há conta, descarta consulta obsoleta e mostra Erro no seletor quando host reporta falha. **66/66 UI, typecheck, workspace Rust, clippy aura-app -D warnings e golden passaram**. Evidência EV-006 parcial; build de produção e teste nativo atual ainda pendentes. Falhas de preparação anteriores não são consideradas correções aprovadas.

Estado atualizado após TK-003: produção SHA256 **7CEF37E8E8BD4B9629B3CCC84FF4D84F1E40562932B93EE550F4F81C24B25772**. QA-003 **corrigido e verificado** (EV-007): cadastro/modelos/status503/remoção refletiram no Overlay já montado, sem refresh; rascunho preservado. Reteste revelou também provider desconhecido no app-server já iniciado: nova conversa agora leva definição atual da rota gateway em config override, sem segredo nem restart. `providers-runtime-native-green.log`1/1, `providers-real-server-warm-thread.log`1/1 (segundo provider após engine quente, turno BYOK + MCP) passaram. Warming com models sem provider ficou preso e foi interrompido, não é teste aprovado; warming com thread existente completou em3,51s.

QA-002 revalidado **em produção com engine real**, upstream loopback controlado, `panels-production-runtime-green.log`1/1 (EV-008); continua corrigido/verificado. QA-042 revalidado (EV-009), `resize-after-providers-production.log`**9/9**; QA-001 mantém estado **parcial** (EV-010), sem aprovação de relato específico de largura ou DPI nativo125/150. **66/66 UI**, typecheck, workspace, golden Rust/TS, fmt/diff-check e clippy aura-app/gateway `-D warnings` passaram no código atual. TK-002 e TK-003 estão done; TK-001 implemented/partial. Próxima fatia: QA-004. Os outros achados seguem pendentes; este esforço não está concluído.

## Estado final da correção — 04/10/2026

Revalidação consolidada no código final (release de produção SHA256 `E2693A146AADB157DD0BDC26B482AC137BEC80CF4EACDD9488ED4F7C33BC99A3`; demo `D80CB945…3D48` para `overlay.e2e`; build `e2e` `846E10EB…D89C` para `auth-cancel.e2e`). Regressão final: `cargo test --workspace --exclude aura-desktop` (45 suítes), clippy `-D warnings` (workspace e `aura-desktop` com `e2e`), fmt, contrato dourado, UI 140/140, typecheck, e as specs nativas de cada achado.

**39 de 41 tickets corrigidos e verificados** (42 achados; QA-042 está no TK-001). Pendências que dependem de ação externa, sem aprovação simulada:

- **QA-001 (TK-001) parcial** — resize nativo 9/9 no build final, mas o arraste com DPI 125%/150% não foi executado (exige mudar a escala da tela do usuário) e o relato específico de largura não foi reproduzido.
- **QA-038 (TK-038) bloqueado** — o app detecta a chave placeholder do updater e informa que as atualizações não estão configuradas; o `release.yml` falha enquanto a chave pública real e o secret de assinatura não existirem. Falta o mantenedor gerar o par de chaves e testar uma atualização assinada publicada (010 AC-004).

Mudança de comportamento visível: desde o TK-041, iniciar o Aura deixa só o ícone da bandeja (001 AC-001) e mostra uma notificação; executar de novo, o atalho ou o menu da bandeja abrem o Overlay.

| Achado | Ticket | Estado | Evidência atual |
| --- | --- | --- | --- |
| QA-001, QA-042 | TK-001 | implementado, verificação parcial (implemented/partial) | EV-054 |
| QA-002 | TK-002 | corrigido e verificado (done/passed) | EV-055 |
| QA-003 | TK-003 | corrigido e verificado (done/passed) | EV-056 |
| QA-004 | TK-004 | corrigido e verificado (done/passed) | EV-057 |
| QA-005 | TK-005 | corrigido e verificado (done/passed) | EV-058 |
| QA-006 | TK-006 | corrigido e verificado (done/passed) | EV-059 |
| QA-007 | TK-007 | corrigido e verificado (done/passed) | EV-060 |
| QA-008 | TK-008 | corrigido e verificado (done/passed) | EV-061 |
| QA-009 | TK-009 | corrigido e verificado (done/passed) | EV-062 |
| QA-010 | TK-010 | corrigido e verificado (done/passed) | EV-063 |
| QA-011 | TK-011 | corrigido e verificado (done/passed) | EV-023 |
| QA-012 | TK-012 | corrigido e verificado (done/passed) | EV-024 |
| QA-013 | TK-013 | corrigido e verificado (done/passed) | EV-025 |
| QA-014 | TK-014 | corrigido e verificado (done/passed) | EV-026 |
| QA-015 | TK-015 | corrigido e verificado (done/passed) | EV-027 |
| QA-016 | TK-016 | corrigido e verificado (done/passed) | EV-028 |
| QA-017 | TK-017 | corrigido e verificado (done/passed) | EV-029 |
| QA-018 | TK-018 | corrigido e verificado (done/passed) | EV-030 |
| QA-019 | TK-019 | corrigido e verificado (done/passed) | EV-031 |
| QA-020 | TK-020 | corrigido e verificado (done/passed) | EV-032 |
| QA-021 | TK-021 | corrigido e verificado (done/passed) | EV-033 |
| QA-022 | TK-022 | corrigido e verificado (done/passed) | EV-034 |
| QA-023 | TK-023 | corrigido e verificado (done/passed) | EV-035 |
| QA-024 | TK-024 | corrigido e verificado (done/passed) | EV-036 |
| QA-025 | TK-025 | corrigido e verificado (done/passed) | EV-037 |
| QA-026 | TK-026 | corrigido e verificado (done/passed) | EV-038 |
| QA-027 | TK-027 | corrigido e verificado (done/passed) | EV-039 |
| QA-028 | TK-028 | corrigido e verificado (done/passed) | EV-040 |
| QA-029 | TK-029 | corrigido e verificado (done/passed) | EV-041 |
| QA-030 | TK-030 | corrigido e verificado (done/passed) | EV-042 |
| QA-031 | TK-031 | corrigido e verificado (done/passed) | EV-043 |
| QA-032 | TK-032 | corrigido e verificado (done/passed) | EV-044 |
| QA-033 | TK-033 | corrigido e verificado (done/passed) | EV-045 |
| QA-034 | TK-034 | corrigido e verificado (done/passed) | EV-046 |
| QA-035 | TK-035 | corrigido e verificado (done/passed) | EV-047 |
| QA-036 | TK-036 | corrigido e verificado (done/passed) | EV-048 |
| QA-037 | TK-037 | corrigido e verificado (done/passed) | EV-049 |
| QA-038 | TK-038 | mitigado, bloqueado (blocked/partial) | EV-050 |
| QA-039 | TK-039 | corrigido e verificado (done/passed) | EV-051 |
| QA-040 | TK-040 | corrigido e verificado (done/passed) | EV-052 |
| QA-041 | TK-041 | corrigido e verificado (done/passed) | EV-053 |
