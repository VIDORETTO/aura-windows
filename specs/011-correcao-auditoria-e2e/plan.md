---
schema: hybrid/plan
schema_version: "1.0"
effort_id: 011-correcao-auditoria-e2e
revision: 1
spec_revision: 1
status: ready
---

# Plano da auditoria

## Summary

Executar fatias verticais serializadas. Primeira fatia: reproduzir QA-001 e QA-042 por WebdriverIO em WebView2 nativo, medindo tamanho interno físico, viewport lógico e shell; usar testes React na seam nativa e corrigir só a causa confirmada. Mínimo por modo fica no shell Tauri. Demais tickets permanecem draft até reconhecimento dos caminhos, contrato técnico e risco específicos.

## Technical context

React 19/TypeScript, Tauri 2.12, Windows 11, WebView2 154.0.4258.53. Driver compatível em target/qa-tools/edge-154. Perfil de QA isolado; backend demo não prova inferência/captura real. Baseline 5b130b69ac3ede8dc5a8fdc6b0e1ceccac7a8393.

## Consumed contract

Spec revisão 1, FR-001–FR-042, AC-001–AC-042. Contratos originais continuam obrigatórios.

## Modules, interfaces, consumers, and seams

- OverlayApp/useAutoHeight: usuário vê shell acompanhar viewport; seam React com adapter SO Tauri.
- ResizeHandles/startResizeDragging: resize nativo pelas bordas; seam E2E WebView2.
- crates/aura-win/examples/qa_resize.rs: probe com SendInput e GetClientRect, exclusivo para executáveis isolados; move cursor e restaura após liberar botão. WebDriver não dirige o loop modal Windows, então este adapter real produz arrastes e amostras independentes do DOM.
- src-tauri/overlay.rs/set_mode: mínimo por modo e posicionamento físico; seam E2E nativa.
- tickets posteriores: identificar módulos e interfaces antes de ready, preservando aura-app como orquestrador e aura-win como adapter SO.

## Chosen approach and alternatives

Medir antes de alterar. Corrigir concorrência só se reproduzida e manter resize nativo. Não substituir janela Tauri ou criar simulação DOM para alegar resize Windows. Não mudar mínimo do compacto ao expandir sem restaurá-lo ao compactar.

Reconhecimento executado: Tao 0.37.1/event_loop.rs WM_GETMINMAXINFO + WM_NCHITTEST usa HTTOP em faixa superior quando undecorated shadow, e omite insets do track size sem decoração. Mínimo inclui diferença outer-inner medida, expressa em unidade lógica para acompanhar DPI. Probe de canto inicia dentro do handle DOM e abaixo da faixa nativa HTTOP; pontos nessa faixa acionam somente altura e não provam defeito no handler React.

## Data, compatibility, and external dependencies

Sem migração de dados na primeira fatia. Segredos e perfil pessoal preservados; usar AURA_HOME isolado. Driver local ignorado pelo git. Build de QA separado do executável original quando necessário.

## Verification strategy

AC-001/042: E2E com mudança nativa de tamanho e arraste pelas bordas; comparar shell e viewport com tolerância de 2 px lógicos. Larguras 480/640/900, modo expandido mínimo 360. Testar reabertura, menus e duas telas/DPI quando disponível; registrar matriz parcial se faltar execução. Vitest + typecheck; cargo test --workspace --exclude aura-desktop e build release. Demais AC: regressão na seam pública contratada + fluxo nativo pertinente definidos antes de ready.

## Change map

| Path | Existing/new | Symbol | Purpose |
| --- | --- | --- | --- |
| apps/desktop/src/overlay/OverlayApp.tsx | existing | useAutoHeight | sincronia de tamanho se confirmada |
| apps/desktop/src/overlay/ResizeHandles.tsx | existing | ResizeHandles | resize nativo |
| apps/desktop/src-tauri/src/overlay.rs | existing | set_mode | mínimo próprio por modo |
| apps/desktop/e2e/specs/resize.e2e.ts | new | jornadas | medição nativa |
| crates/aura-win/examples/qa_resize.rs | new | probe | mouse nativo e dimensão independente |
| crates/aura-core/src/placement.rs | existing | place_overlay/MIN_EXPANDED | restauração segura de tamanho expandido legado e DPI |
| apps/desktop/src/overlay/resize.test.tsx | new | regressões | seam nativa |

## Derived technical obligations

- **OT-001** → AC-001/042: comparar dimensões físicas com escala real, sem confundir DPI e CSS.
- **OT-002** → todos os AC: nunca registrar passed sem execução; serializar tickets e conferir invalidação.

## Risks and gates

G2 válido para primeira fatia. Falta de driver é ambiente, não red comportamental. Controle nativo Computer Use não está exposto nesta sessão; WebDriver disponível via CLI. Riscos e caminhos dos demais tickets serão detalhados antes de G3.

## TK-002 — Painéis e conversa legível

QA-002 confirmado por inspeção: HistoryPanel256 + WorkPanel320 + largura640 deixam64px. Alternativa aceita: exclusão mútua atômica em session.toggleHistory/toggleWork; drawer absoluto com superfície menu abaixo de1024px, aside estático em janela larga. Corpo relativo; área de conversa não encolhe no compacto horizontal do expandido. Largura útil mínima literal320px. Não redimensionar janela automaticamente e não permitir ambos simultaneamente. Seam React: ações reais do usuário preservam rascunho; seam E2E WebView2: medida renderizada em480/640/1024 e conversa sintética isolada. Paths no TK-002 estão reconhecidos. G2/G3 podem liberar esta fatia independentemente da validação DPI pendente do TK-001, mantendo apenas um in_progress.

## TK-003 — Catálogo atualizado entre janelas

Reteste real revelou provider novo ausente em processo já iniciado pelo models_list. TK003 deve enviar definição atual no config override da nova thread, com as mesmas opções GatewayConfigContributor (URL gateway/env_key sem segredo); independente de opts.model. Sem restart de conversas existentes. Refatoração local da construção de tabela para evitar divergência; teste real aquece models antes de cadastrar e executa turno BYOK controlado. Paths adicionais codex_config.rs e real_app_server.rs reconhecidos no ticket.

Host.save_provider/remove_provider/test_provider atualiza rotas sem evento; stores Zustand entre webviews são independentes. Emitir HostEvent::ProvidersChanged{} após sucesso; canal providersChanged/event{} no contrato dourado, sem dados ou segredos. App store mantém revisão de catálogo; OverlayApp recarrega quando muda. session.loadCatalog preserva escolha válida/thread, escolhe local keyless quando não há conta ChatGPT no rascunho, limpa escolha de modelo retirada. Consultas concorrentes descartam resposta obsoleta. Mock emite o mesmo contrato. Testes seam host.subscribe, React Overlay persistente, WebDriver produção mutações reais IPC sem refresh. Contrato IPC aditivo exige regenerar/retestar golden. Critério AC-003 não exige inferência local disponível. Caminhos e exemplos constam TK-003; risco de ciclo app/session evitado pelo contador de revisão no app store.

## TK-004 — Idioma coerente

Verificacao adicional do tray: probe de QA em aura-win enumera apenas processo aura-qa.exe/original isolado e classe tray_icon_app unica; envia callback WM_USER_TRAYICON6002/WM_RBUTTONUP da dependencia tray-icon0.25.1 instalada, le o HMENU real do popup Windows via GetMenuBarInfo/GetMenuString e fecha com WM_CANCELMODE. Nao e clique fisico da bandeja nem prova de descoberta do icone; comprova texto nativo atualizado no menu existente. Sem novo IPC de produto, cursor ou dado pessoal. Probe recusa ambiguidades e outros executaveis.

Dicionarios pt-BR/en como fonte unica de textos: UI usa useT; aura-app build.rs gera tabela Rust de entradas string dos arquivos TS para nativo/prompts embutidos (serde_json builddep ja presente workspace). Nomes builtin canonicos e templates customizados persistidos preservados; localizar builtins ao listar/expandir, sem migracao de banco. ContextChip ganha attachmentLabel opcional serde default/skip com nome+count/unit/image, construido a partir de summaries internos conhecidos, para UI traduzir sem regex em filenames/conteudo. Golden aditivo protegecompatibilidade. Native titulos/tray atualizam emapply_settings com menuitemstate; catalogo Overlay/InputBar observa idioma. Reconhecimento, exemplos literais e riscos constamTK004. Rerun regressao nativa atual antes de renovar evidencias afetadas.

## TK-005 — Inicializacao unica do bridge

Reconhecimento confirmou getBridge com cache so apos await import; bootstrap onHostEvent e efeitos Overlay concorrentes podem criar mocks diferentes. Previa Vite1420 real reproduziu clickChatGPT sem transicao. Compartilhar Promise de inicializacao em getBridge, tanto mock quanto Tauri, preservar setBridge injection/reset; rejeicao limpa somente promessa atual para permitir retry. setBridge deve substituir promessa em curso para futuras chamadas; inicializacao antiga nao sobrescreve bridge injetado. Seam publica testes concorrentes getBridge/onHostEvent+authLogin/authStatus com literalswaitingBrowser/completed, identidade unica e inject/reset. Sem mocks de import em teste principal, usa mockbackend real. Validacao UI/typecheck e navegador Vite real coldreload login welcome/input, sem OAuth pessoal; nativo production smoke para caminhoTauri. Sem novo IPC/dependencia/backendlogin. Caminhos bridge.ts/novo bridge.test.ts, e2e smoke se necessario e relatorio. Um ticket implementado por vez.

## TK-006 — Cancelamento sem falha duplicada

Reteste adicional confirmou previewmock: auth_cancel emiteCancelled mas auth_login atrasado ainda emiteCompleted. Contador de tentativa local no mock invalida completions cancelados/substituidos, sem mudar IPC. TesteUIred aguardou Promise real do mock depois do clickCancelar e confirmouwelcomeindevido; primeira tentativa fakeTimers travou harness e nao e red valido. mock.ts pertenceaTK006; testar cancel/novatentativa no Vite real. Code pathAuthService pendingcleanupverificar retryimediato sepergunta/teste revelar risco.

AuthService.begin_login ja emite waiting/completed/cancelled/failed, Host.forward_events retransmite. Host.login ainda observa JoinHandle e emite Completed/Failed novamente, convertendo Cancelled emFailed. Remover segunda emissao do host, mantendo o fluxo do servico; JoinHandle pode ser descartado (tarefa tokio continua). Seam Host.subscribe+login/cancel_login realauthcom endpointloopback sem shell, assertumCancelled/zeroFailedaposcompletion e autenticaaindaindisponivel; UI store removeprogresscancelled sem erro e permite novatentativa. Nativo E2E roda host/callback real com dependencia externa autorizacao controlada: feature explicita e2e (fora buildpadrao), unico envport u16 AURA_E2E_AUTH_PORT ->http127.0.0.1:port/authorize. Sem URL arbitraria/tokens/issuer/JWKSoverride, nenhumaautorizacao/credencial pessoal. WDIO beforeSession somente flagAURA_E2E_LOCAL_AUTH cria paginaHTMLlocal e porta antesdriver; captura somente canais/state, nunca authorizeUrl. ClickLogin/Cancel duas vezes, nenhumfailed, voltaLoginCard, reabrir. Build seme2e ignoraenv e checksclippyambos. Testes/realexec/review+golden(semformatnovo) antespassed.

## TK-007 — Microfone persistido e aplicado

Nativeinventorysemcapture confirmouFifine(default) eCABLEOutput(VB-Audio) distintos. SelectorVoice nao temvalue/onChange; Hostptt_press(None) sempreDefault, atalhos/InputBar passamnulo. Settings.microphoneDeviceId nullable(defaultnull); Patch distinguir chaveausente deJSONnull por deserializehelper, semserde_withdependency. Limite1024chars/semNUL, stringblanknormalizaanull, IDsnaovazios preservados byte-a-byte. SettingsRepo generico persiste semmigration. UIdefaultoptionlocalizada, valuecontrolado/onChangeupdateSettings; missingdeviceoption localizadaparavisualizarpreferenciaretida. Hostdeviceexplicito temprioridade, senao usaSettings; semdevice configuradoDefault. AudioHub/Wasapi mantemfallbackexistente; exposeaviso naVoice seconfigurednaolistadeavailable (naoprometeconsertarnotificacaofallback geralsomenteTK013). PTT no teclado/mouse/global honra hostcentral semalterarhandlers. Seam coreJSONupdates/nullmissing/validation, StoreDBreopen, HostAudioSourceboundarygravandoselecao+cancelled/restart; React2fake/devnames diferentes e remontar. Native doisnomes reais selecionados/PTTstartcancel(closehub, nentranscricao/arquivos)/restartdoappmesmoperfil; stageenv noprocessorestart. Miccapta memoria temporaria autorizadapor testereal, cancelapagatudo. GoldenaditivoSettings; clippy/UI/Rust/realE2E antespassed. DeviceIDsWindows sao friendlynamescpal, naoendpointestavel; naoalterarADR/adapteridentidade neste ticket.

## TK-008 — Transcrições iguais em ciclos diferentes

Reconhecimento: InputBar usa lastVoice.text e ignora Done igual para sempre. PushToTalk publica Listening/Transcribing/Done por ciclo; bootstrap encaminha eventos ao AppStore. Guardar voiceResultRevision local, incrementado pela transição para Done quando o estado anterior não era Done. Duplicatas terminais contíguas não incrementam; novo ciclo Listening/Transcribing permite outro resultado, mesmo texto. InputBar consome revisão uma vez, sem comparação textual. Estado derivado no reducer preserva fronteira mesmo quando React agrupa Listening+Done em um render. Sem novo IPC, backend ou ASR; teste usa seam legítima do resultado do transcritor. IDs persistentes/entrega fora de ordem não são prometidos por esse contrato ordenado.

TDD React com bridge público: frase literal duas vezes em ciclos distintos, campo limpo, caret/sem duplicata Done e autoenvio duas vezes. Harness zera revisão. App nativo de produção WebDriver recebe eventos via API Tauri event emit controlada, usando resultado sintético do ASR; provar red no binário TK007 e green no novo, preservando UI/bridge real. Não afirmar fala/modelo ASR real, latência ou qualidade acústica; essas verificações pertencem aos demais achados de voz. Risco de eventos agrupados tem caso explícito. UI/typecheck/build e release normal, regressão PTT nativo, revisão Standards/Spec antes de evidência. Caminhos InputBar.tsx, state/app.ts, test/harness.ts, OverlayApp.test.tsx e novo voice-repeat.e2e.ts.

## TK-009 — Idioma automático independente da UI

Reconhecimento adicional: rótulos voice.language.auto diziam "idioma da interface" e devem descrever "detectar idioma" em ambos os dicionários. Mesmas chaves, paridade e UI testada. Nenhuma alteração do idioma da interface ao escolher ASR automático.

Roteiro nativo ajustado após execução: Tauri invoke é não gravável/não configurável; observação de argumentos por substituição não foi instalada, tentativa descartada como erro do teste. Request language=null é verificado nos testes React através do bridge externo; nativo prova seletor, persistência e ciclo de pressão/soltura com eventos Listening/Empty reais PT/EN, sem atribuir observação de parâmetros. Clique rápido reproduziu Listening sem terminal: release pode ocorrer antes de press async. Isso é confirmado e encaminhado ao TK010 (alternância de clique), sem implementar outro ticket simultaneamente. Usar ações pointer down aguardando Listening e pointer up para o contrato de pressão atual. Não marcar correção do clique neste ticket.

InputBar pointerup/leave e shortcuts PushToTalkUp derivam pt de Language::PtBr; global dictation já passa None. Host dá precedência à preferência ASR fixa e depois linguagem explícita do caller. Remover fallback de idioma da UI nos dois callers (None/null), mantendo API e precedência fixa. SettingsPatch.asr_language reutiliza nullable_patch com skip None: JSON null precisa limpar es/pt fixo, omissão mantém. Sem alterar worker ou reconhecimento acústico. Seletor Voice já envia null; confirmar red no banco nativo antes da correção. UI PT/EN observa ptt_release real do bridge externo, null, e configurações fixo/automático. Native Settings espanhol->automático e restart; Overlay click microfone observa apenas language no invoke encaminhado, sem substituir resposta, cancela eventual captura. ASR acústico/qualidade de autodetecção requer modelo e segue sem aprovação inferida. Core JSON TDD, React UI por comportamento, IPC/golden inalterado reteste, Rust/core/host, types/build/clippy/Windows release; review Standards/Spec e regressão voz/mic. Sem dependência nova nem alteração de versão app-server.

## TK-010 — Alternância do microfone por clique e teclado

Primeiro green nativo confirmou Enter mas Space falhou: atributo HTML disabled durante press remove foco no WebView real. Usar aria-disabled/estilo e guard funcional mantendo foco, com pending/transcribing impedindo chamadas. Tests verificam atributo e ausência de comandos enquanto pendente; nativo Enter->Space sem refocar é regressão da perda de foco. Não contar esse primeiro release como aprovado.

Reconhecimento confirmou botão só pointerDown/Up/Leave; Enter/Space não chama comando. Native clique rápido no TK009 produziu somente Listening, pois release pode executar antes de press async. Migrar controle para onClick de button nativo: idle/terminal inicia, Listening/Partial conclui com language=null. Atalho global Ctrl+Space permanece pressão/soltura no shell. Guard ref e estado busy serializam operações e desabilitam enquanto press/release está pendente; transcribing desabilita botão. Label localizado Ditar/Concluir ditado e aria-pressed; mover cursor não conclui. Ctrl+Space no botão impede ativação nativa duplicada (atalho SO separado). Esc já é controlado por Overlay, incluir Partial em cancelamento conforme contrato 006 AC007.

TDD por casos: primeiro clique inicia sem liberar, segundo libera uma vez; Enter/Space equivalentes; dois cliques rápidos enquanto press externo pendente não liberam antes de Listening; Esc Listening/Partial cancela sem texto; erro permite retry. Mock somente no bridge externo/OS; orquestração real. Native novo voice-toggle.e2e: mouse clique, sair do botão, segundo clique Empty, Enter/Space via WebDriver, Esc, estados reais WASAPI em perfil isolado. Sem fala/modeloASR/acústica aprovada. Atualizar regressão de QA009 ao controle click/toggle mantendo oráculo null React e configuração nativa. UI/typecheck/build e release normal, native repeat/mic/lang apropriados, review/EV antes done. Sem novo IPC/dep.

## TK-011 — Instalação recomendada na tentativa de ditado

Lifecycle reconhecido: DownloadError::Cancelled serializa erro literal cancelled; AppStore e card classificam como cancelamento, sem alerta de falha, preservando erros genuínos/retry. Início de retry guarda objeto de progresso anterior para não reutilizar erro antigo como falha da nova tentativa. ID de modelo solicitado é fixado até conclusão mesmo se idioma UI muda; labels se atualizam. state/app.ts é owned da fatia para semântica do download, nenhum formato novo. Primeiro red Host pegava evento startup irrelevante, descartado; red corrigido filtrou Voice e confirmou [] contra Failed model_missing.

Reconhecimento nativo com worker engines real: sem modelos, ptt_press retorna asr/model_missing e botão apenas mostra aviso. Catálogo real recomenda Whisper Turbo 1624555275 bytes nesta máquina, nenhum instalado/selecionado. Worker engines compilou com libclang18 isolado (Apache2 LLVM exceptions); hash 8E6874BCBACDE0D0D65841A459B9F540660C59D54A5575B594A332E563CFD45B, não é mock. Sem atualizar deps/app-server. Risco confirmado: Windows release sem worker escolhe AsrBackend::Fake vazio. Trocar fallback por Unavailable, preservar erro worker_missing/Cloud fallback e aviso localizado de reparar instalação; não oferecer download como conserto de worker ausente. Worker real é sidecar já previsto no bundle.

Host ptt_press publica Voice Failed(model_missing) além de retornar erro existente, para a mesma oferta funcionar no caminho de hotkey; sem novo formato IPC. UI catch reconhece erro estruturado e evita toast genérico para model_missing, usando estado Failed já existente. Novo VoiceInstallCard dentro InputBar aparece em Failed/model_missing, usa voice_models/recommended, nome/tamanho e Download(N MB) localizado, progresso/cancel/retry. API voice_select após download concluído persiste preferência antes de fechar oferta; modelo já instalado pode ser usado sem baixar. Não inicia captura automaticamente. Falhas de privacidade/rede/disco continuam falhas reais, sem alterar políticas ou inferência de permissão.

TDD Host Worker sem modelos não abre AudioSource, emite Failed e erro; Unavailable nunca finge transcrição. React com seam externa literal recommended10MiB: card/name/10MB/download correctID, progresso/cancel/retry/seleção completa e nenhuma press automática. Simular apenas rede/worker/OS via bridge; nenhum mock interno. Native red no release010 com worker real confirma ausência da oferta; novo release testa ausência/instalação recomendada real com hash verificado pelo Downloader, seleção e restart. Para verificar ASR real, usar WAV sintético de fala via SAPI/test fixture e worker real; se possível alimentar CABLE Input por probe em aura-win (apenas dispositivo virtual de QA, sem alterar default/usar saída física), capturando CABLE Output no app. Nenhum áudio pessoal/modelo/binário/captura commitado. Se ASR/download/hardware não executar, registrar partial, nunca done inferido.

Caminhos: voice.rs/host.rs/tests Host, main.rs backend produção, InputBar/novo VoiceInstallCard e testes, dicionários; e2e model-install e suporte de fala/cabo em aura-win somente se necessário após reconhecer APIs. UI/core/app/workspace/types/golden(claim formatos inalterados)/clippy/fmt/build, app real/worker engines, regressão resize com oferta e ciclos de ditado; review Standards/Spec antes EV. Revalidar tickets anteriores afetados com modelo real instalado, sem reaproveitar teste de áudio vazio como acústica.
