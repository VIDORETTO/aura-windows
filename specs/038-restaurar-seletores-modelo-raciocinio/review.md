# Revisão — esforço 038

## QA nativo atual — TK-003, Windows 100%

Build `pnpm -C apps/desktop tauri build --no-bundle --features e2e` concluído exit0 em2m57s, sem demo. Binário próprio `TEMP/aura-038-final-native-20261010-181914/aura-qa.exe`; WebView2 154.0.4258.62 e pin0.159.0. Spec1/plan3 e baseline preservados; nenhuma alteração produtiva nesta verificação.

Model-picker-recovery2/2 em5,2s: 480px lógicos, compacto/expandido, nomes longos, botão/seta acionáveis, nenhuma sobreposição, nome acessível completo, menu dentro da janela/área útil. Scale nativa1/dpr1; screenshots compact/expanded inspecionados. Teclado pt-BR/en abre, seleciona Baixo/Low e Escape restaura foco, mantendo entrada/Overlay. UI253/typecheck e Host74 da renovação anterior continuam atuais. AC-009 aprovado por esta execução; AC-008 somente parcial em100%, sem simular150/200.

Regressões composer/model-effort/model-efforts/profile-model:4/4 passaram, incluindo envio real ao upstream scripted e perfil via janela Windows. Resize8/9 em39s: primeiro arraste East-120 conservou largura900, esperado menor que850; as outras oito verificações passaram. Repetição apenas da suíte resize em perfil novo:6/9 em15,5s; três casos de entrada recusados pelo helper porque Aura não recebeu foreground. O guard foi preservado, nenhuma entrada enviada nessas recusas. A causa da largura inalterada da primeira sessão não foi atribuída ao produto nem descartada como ambiente sem prova. Não aprovar regressão completa ou DPI150/200; TK-003 permanece bloqueado/parcial.

Standards: somente perfis/binaries/logs próprios em TEMP, CLI E2E existente e helper restrito ao nome aura-qa.exe; sem mudança de preferências de monitor, pin, dependências ou oracle. Spec: progresso de teclado/geometria registrado separado dos gates pendentes. Nenhuma publicação/commit.

## Renovação atual após integração web — TK-001/TK-002

Baseline ed9a9654c6e76e48186fc010a12eba77023b00f5 mantida; committed/staged vazios, mudanças compartilhadas e arquivos novos incluídos. Spec1/plan3 preservados. Código dos seletores e preferências não foi alterado nesta renovação; a integração web acrescenta configurações/metadados independentes sem substituir catálogo, esforço ou semântica do próximo Turno.

Standards: UI253/33arquivos em26,66s e typecheck passaram; Host74/74 em1,82s, incluindo persistência por APIs públicas e reinício real do store. Fmt/diffcheck atuais passaram; nenhuma dependência/IPC de seleção/pin/segredo novo. Oráculos originais qa-reasoner/[low,high,max], capacidades e exemplos de startup preservados. Nenhum achado Standards bloqueante das duas fatias.

Spec: AC-001..004 continuam cobertos por acesso compacto/expandido, pendente/vazio/erro, retry aberto, ordem de catálogo e conservação BYOK. AC-005..007 cobertos por envio modelo/esforço válidos, ausência de suporte, memória por modelo/modo/reinício, alias legado e revalidação imediatamente antes de enviar. Nenhum red artificial nesta renovação. TK-001/TK-002 podem voltar a done com novos registros atuais. Pixels, DPI, resize/foreground e acesso real de conta não foram aprovados por esses testes; permanecem no TK-003. Build com adaptadores Windows reais está em execução e não é aprovação antecipada.

Baseline fixo: `ed9a9654c6e76e48186fc010a12eba77023b00f5`. Inclui staged/unstaged e arquivos novos relevantes; sem commits novos. Revisão em dois passes por eixo, no contexto deste esforço. Não substitui verificação nativa do TK-003.

## TK-001 — recuperação de catálogo

Escopo: `session.ts`, `ModelPicker.tsx`, `Composer.test.tsx` e i18n pt-BR/en; regressões existentes `OverlayApp.test.tsx`. Spec r1, plan r1, AC-001 a AC-004. Evidência EV-001.

### Standards

IPC continua tipado e sem alteração de wire; estado UI em Zustand; nenhuma API Windows ou dependência nova; rótulos nos dois idiomas; nenhum segredo/conteúdo de exceção exposto. Mock na fronteira IPC, oráculos literais e UI real no teste; corrida conserva request mais novo. `typecheck` e paridade passaram. Nenhum achado bloqueante neste escopo.

### Spec

Controle abre em compacto antes de mensagem e em Conversa expandida com provedor fixo. Estado pending/vazio/erro é distinto; retry atualiza diálogo aberto; erro preserva modelos/esforço e BYOK; resposta antiga não substitui catálogo novo. Testes de acesso eram cobertura existente, enriquecida com Conversa em andamento; red/green novo para falha e carregamento. Nenhuma expansão comercial/sidecar ou mudança de permissão. Nenhum achado bloqueante nos AC desta fatia.

Limites: jsdom não prova largura/DPI, reservados para AC-008/TK-003. Preferências/reconciliação AC-005..007 pertencem ao TK-002. Resultado: ticket liberado para done no escopo testado; esforço ainda em andamento.

## TK-001 renovado / TK-002 — modelo, esforço e persistência

Spec r1 / plan r2; baseline preservado. Escopo adicional `session.ts`, casos públicos em Composer, texto i18n e teste Host de reinício. EV-002 renova AC-001..004, EV-003 cobre AC-005..007. EV-001 anterior foi invalidada, sem reuso silencioso.

### Standards

Revisão separada do diff atual: sem API Windows, segredo, nova dependência ou alteração IPC; mocks somente IPC/processo/SO. Teste de persistência usa store em disco temporário e `Host::settings/update_settings`, sem consulta SQLite lateral. Oráculos literais, sem algoritmo duplicado. Rustfmt/typecheck/paridade e 91 testes UI/i18n + 74 Host passaram. Nenhum achado bloqueante neste escopo.

### Spec

Escolha qa-reasoner/max chega ao Turno; modelo sem suporte não envia esforço. Preferências por modelo/modo voltam após troca/recriação de Overlay, catálogo tardio e reinício real do Host. Alias legado continua legível e outros modos preservados em nova escolha explícita. Mudança confirmada invalida/avisa; falha de refresh conserva escolha. Caso de corrida no startup reproduziu envio inválido e agora revalida esforço imediatamente antes do IPC, conservando escolha de modelo do Turno. AC-001..004 continuam verdes após mudança compartilhada. Nenhum achado bloqueante nos AC das duas fatias.

Limites mantidos: não afirmar geometria/DPI nem acesso real de conta. Resultado: TK-001/TK-002 liberados para done; executar TK-003 e depois esforço 039.

## Renovação após o ajuste de largura do TK-003

Baseline e revisões mantidos. EV-004/EV-005 substituem EV-002/EV-003 desatualizadas. Inspeção do diff compartilhado: `min-w-[104px]` no seletor compacto e subtítulo somente a partir de 600 px não mudam a seleção, catálogo, persistência ou IPC. Standards: tokens existentes, nomes completos no rótulo acessível, sem dependência ou API nativa nova. Spec: os 91 testes públicos UI/i18n e 74 Host voltaram a passar, inclusive recuperação, corrida e reinício. Typecheck, rustfmt e diff check também passaram. Nenhum achado bloqueante para TK-001/TK-002; a prova nativa completa permanece no TK-003.

## Plan r3 — janela nativa e evidências atuais

EV-006/EV-007 renovam TK-001/TK-002 depois da correção do aviso de pausa. Standards: abreviação visual abaixo de 600 px preserva ícone, tooltip e nome acessível; UI existente e ambos os idiomas, sem nova dependência, IPC ou acesso Windows. Teste E2E de perfil passou a selecionar campos por rótulo porque a busca de Configurações tornava incorretos os índices globais de inputs; acrescentadas expectativas literais de processo/título sem enfraquecer o esperado de modelo. 244 UI, 74 Host, typecheck e verificações de formato verdes. Nenhum achado bloqueante nesses dois tickets; liberados para done.

Spec: red nativo reproduziu o aviso sobrepondo Reunião; green atual não tem sobreposição, mantém nome completo, botão/seta e menu dentro da área útil no monitor menor. Inspeção dos screenshots compacto/expandido do build normal confirma a geometria positiva a 480 px/100%. EV-008 comprova teclado pt-BR/en e Escape/foco; EV-009 é parcial e não aprova AC-008 inteiro. Os dois monitores estão a 100%; 150%/200% reais não foram testados. Regressão de resizing recusou entrada porque o QA Overlay não recebeu foco nativo; não remover sua proteção. Perfil corrigido alcançou QA target/qa-other, mas a primeira execução expirou durante preparo do sidecar e aguarda repetição com binário fixado já disponível. TK-003 não está liberado para done. Resolver os bloqueios depois das fatias disponíveis do 039, conforme pedido do usuário.

Repetição de perfil concluída: EV-010 registra 1/1 passado com o app-server fixado já disponível, incluindo `profile_active=QA target` e `model=qa-other` observado no upstream. Essa regressão está resolvida. Permanecem somente os gates nativos de DPI 150%/200% e resize/foreground.

## Resize renovado — EV015 / 10-10-2026

Baseline fixa preservada; nenhuma alteração em resizing/qa_resize/oráculos/guard de foreground nesta repetição. Build OS nativa atual e perfil QA novo: resize9/9 passou44s, incluindo primeiro East-120 antes falho, todos edges/corners, menus/restauração e monitores disponíveis. A falha anterior não se reproduziu; causa definitiva não atribuída nem guard enfraquecido. Standards sem novo achado; Spec libera o gate resize com prova atual.

AC008 permanece parcial: monitores observados100%, geometria150%/200% real ainda não executada. EV013/014 preservam nome completo/teclado/480px100% anteriores. TK003 continua blocked somente por ambiente DPI; pergunta de disponibilidade enviada ao usuário. Não alterar sua configuração pessoal ou chamar DPR simulado de DPI nativo.

## Catálogo ChatGPT real — 10-10-2026

Verificação adicional do seletor nativo com a conta já conectada: catálogo real retorna GPT-6 Luna/GPT-6.1 Sol/GPT-6 Astra; a UI mostra modelos e, para Luna, padrão médio/baixo/médio/alto/muito alto/máximo habilitados. Escolha de provider/modelo somente na Session em memória; nenhum chip de esforço foi selecionado, nenhuma inferência/captura e settings_get antes/depois idênticos. Report real-chatgpt-picker-2026-10-10.json. A recuperação do catálogo é pela ação pública existente, quando necessário. Primeiro harness tentou controle de provedor antes da recuperação; falha preservada como harness, não red do produto.

Standards: seam nativa/publicIPC, não injeção de store/evento nem catálogo sintético, sem alterações de produto/preferência/conta. Spec: confirma controle real disponível com dados do ChatGPT a100%; não substitui testes completos por estado, teclado ou DPI150/200. TK003continua blocked somente por geometria em escala real.
## Revalidação após correção de monitor040 — EV017/018

Standards: geometria do shell e helper mudaram em040; EV013/014/015 foram explicitamente invalidadas. Nova execução atual `model-picker-recovery`2/2 e `resize`9/9 usa build OS com pin fixo/perfil/identidade QA próprios; nenhum guard/oráculo reduzido ou configuração pessoal alterada.

Spec: EV017 renova AC009 teclado/Escape/foco em pt-BR/en; EV018 é parcial para AC008, com 480px lógicos, nome completo, seta acionável, zero sobreposições e menu dentro da área útil nos dois modos a 100% real. A correção040 passou 3/3 nativos, mas não cobre 150/200% nativos. TK003 continua blocked/partial exclusivamente por esses DPI. Provas e hashes em `specs/040-overlay-estavel-no-monitor/qa.md`.

