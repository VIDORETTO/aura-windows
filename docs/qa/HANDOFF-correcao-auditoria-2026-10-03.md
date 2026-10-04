# Handoff — correção integral da auditoria E2E

Atualizado em 04/10/2026 (America/Sao_Paulo). Repositório: raiz deste projeto.

## Estado em 04/10/2026 — leia primeiro

- Esforço `specs/011-correcao-auditoria-e2e/`, checkpoint revisão **73**, última evidência **EV-063**.
- **39 de 41 tickets done/passed**, todos verificados no app de produção com WebDriver/tauri-driver (specs em `apps/desktop/e2e/specs/`, logs em `target/qa-tools/`). TK-001..010 tiveram a evidência renovada no build final (EV-054..063).
- **TK-001 implemented/partial**: resize 9/9 no build final; falta arraste nativo com DPI 125%/150% (mudar a escala da tela) e o relato de largura.
- **TK-038 blocked/partial**: detecção da chave placeholder do updater, mensagem honesta na UI e guarda `scripts/check-updater-key.mjs` no `release.yml`. Falta o mantenedor gerar o par (`pnpm tauri signer generate`), configurar a pública e o secret, e testar uma atualização assinada publicada. Nenhuma chave foi gerada ou simulada.
- Mudança visível (TK-041): o Aura inicia só na bandeja, com notificação; executar de novo, o atalho ou a bandeja abrem o Overlay. Specs que precisam do Overlay visível o abrem relançando o executável.
- Resumo por achado: seção "Estado final da correção — 04/10/2026" em `docs/qa/auditoria-e2e-2026-10-02.md`.

### Como rodar as specs nativas (resumo)

```powershell
pnpm -C apps/desktop build
cargo build -p aura-desktop --release --features tauri/custom-protocol
Copy-Item target/release/aura.exe target/qa-tools/aura-qa.exe   # probes qa_resize/qa_tray exigem esse nome
$env:PATH = (Join-Path (Get-Location) 'target/qa-tools/edge-154') + ';' + $env:PATH
$env:AURA_E2E_APP = Join-Path (Get-Location) 'target/qa-tools/aura-qa.exe'
$env:AURA_HOME = Join-Path (Get-Location) 'target/qa-profile-<nome>'   # perfil isolado
pnpm -C apps/desktop/e2e test --spec ./specs/<spec>.e2e.ts --mochaOpts.timeout 400000
```

- `overlay.e2e` usa a build `--features demo`; `auth-cancel.e2e` usa `--features e2e`, executável `aura-auth-qa.exe` e `AURA_E2E_LOCAL_AUTH=1`; `panels.e2e` usa `AURA_E2E_REAL_PROVIDER=1`.
- Specs de voz e de anexo de áudio precisam de perfil com modelo de fala instalado (ex.: `target/qa-profile-model-install2`, Parakeet V3); áudio de teste via VB-Cable + SAPI, nunca microfone ambiente.
- Specs com modelo loopback (`e2e/responses.ts`) declaram `supported_parameters: ["tools"]` e escolhem o provedor/modelo no seletor.

## Histórico (estado em 03/10/2026, mantido para contexto)

## Pedido original e regras

O usuário pediu corrigir **todos os 42 achados** de `docs/qa/auditoria-e2e-2026-10-02.md`, priorizando redimensionamento compacto/expandido, confirmando relatos e riscos antes das correções, trabalhando **um ticket por vez**, com testes e **execução real do app**. Não encerrar como concluído nada sem verificação. A última solicitação foi criar este handoff para outra IA continuar.

Leia `AGENTS.md`, `docs/HANDOFF.md`, `CONTEXT.md`, arquitetura/ADRs e os artefatos do esforço. Use as skills locais `.agents/skills/hybrid-*`. Não há autorização para delegar a subagentes. Não foram feitos commits. Preserve alterações existentes; não faça reset/checkout indiscriminado. Execute ferramentas diretamente, sem `rtk`, wrappers ou shims. Shell: PowerShell; aprovação configurada como never, filesystem unrestricted.

Hybrid: implementar somente após package `ready: true`; ticket/state/evidência pelo runner; evidência somente após execução real. Não editar manualmente `todo.md`, `verification.md` ou `docs/project/backlog.md` — são projeções. TDD por comportamento, oráculo independente; falha de ambiente não é red de produto. Capturas, modelos e executáveis ficam em `target/`, ignorado, sem commit.

## Estado que deve ser retomado

- Esforço: `specs/011-correcao-auditoria-e2e/`.
- Spec aceita revisão 1, FR/AC-001 até 042. Plano revisão 1 com detalhamento incremental.
- Baseline: `5b130b69ac3ede8dc5a8fdc6b0e1ceccac7a8393` (main).
- `state.json`: revisão **42**, phase implementation, status active, active_ticket **TK-011**, last_evidence **EV-022**.
- **TK-001 até TK-010: implemented/stale**. Possuem resultados históricos, mas fingerprints globais do plano invalidaram evidências. Não confundir histórico aprovado com evidência atual.
- **TK-011: in_progress/not_run**, sem nova evidência registrada. Pacotes iniciais e extensão retornaram ready true (`target/qa-tools/package-TK011.json`, `package-TK011-extension.json`); graph em `graph-TK011.json`.
- TK-012 até TK-041 seguem pendentes/draft. QA-042 está agrupado em TK-001.
- Goal original permanece active. Não marcar complete. O relatório de auditoria possui anotações cronológicas que às vezes mencionam estados antigos; runner/state prevalecem.

Mudanças globais no plano podem invalidar evidência e reabrir tickets automaticamente. Para manter execução serial, após invalidar os tickets anteriores foram restaurados para implemented/stale pelo runner. Consolidar retestes no código final; não repetir toda a suíte antiga a cada parágrafo novo de plano.

## Correções anteriores e limites

| Ticket / achado | Implementação e execução histórica | Limite ainda relevante |
| --- | --- | --- |
| 001 / QA-001,042 | Autoaltura responde a resize real; cancela efeito obsoleto; mínimos por modo, compensação da borda invisível, clamp de placement legado. Probe Windows SendInput validou bordas/cantos e duas telas; último resize 9/9 no release TK-010. | **QA-001 parcial:** relato específico de largura não reproduzido; arraste nativo com DPI 125/150% não executado. Teste puro de DPI não substitui isso. |
| 002 / QA-002 | Histórico/Arquivos exclusivos e atômicos; abaixo de 1024px sobrepõem corpo, preservando conversa/rascunho. Produção com app-server real e upstream loopback controlado passou. | Renovar evidência no frontend final. |
| 003 / QA-003 | Evento providersChanged, catálogo atualizado sem remontar Overlay; preserva seleção válida/rascunho. Provider cadastrado após engine quente entra em config override da nova conversa. Golden e turno real controlado + MCP passaram. | Renovar evidência atual. |
| 004 / QA-004 | Localização Rust gerada dos dicionários, títulos de janela/tray, Diagnóstico/MCP/aria, templates padrão, metadados de anexos. Execução nativa EN/PT/EN e probe de tray passaram. | Catálogo de modelos ainda exige trabalho em QA-012; não alegar clique físico no tray quando foi probe. |
| 005 / QA-005 | Inicialização concorrente do bridge compartilha promise, reset/injeção descarta resposta velha, retry após rejeição. Preview e IPC/eventos nativos passaram. | Não aprova OAuth externo. |
| 006 / QA-006 | Host deixou de emitir Failed após AuthService emitir Cancelled; mock descarta Completed tardio. Duas tentativas nativas PT/EN passaram. | Autorização externa foi HTML loopback por feature e2e; **não aprova OAuth completo**. |
| 007 / QA-007 | Preferência de microfone nullable persistida; Host explícito > salvo > padrão. Seletor controlado, dispositivo ausente preservado. Fifine/CABLE WASAPI e restart passaram. | Na época não houve modelo ASR ou fala real. |
| 008 / QA-008 | Revisão por conclusão ASR aceita frases iguais de ciclos diferentes e ignora duplicata terminal. React e eventos sintéticos em WebView real passaram. | **Não aprova reconhecimento acústico**. |
| 009 / QA-009 | null limpa idioma ASR; patch ausente mantém; UI/atalho não derivam pt do idioma da interface. Config/restart e ciclos nativos passaram. | Parâmetros exatos verificados no bridge React; invoke Tauri nativo é imutável. Sem aprovação acústica/atalho físico. |
| 010 / QA-010 | Botão alterna por clique/Enter/Space, não termina ao sair com cursor; guard pendente/transcrição; aria-disabled mantém foco; Escape cancela também Partial. | Último release passou teclado/mouse nativos e regressões. Na época ASR vazio era fallback Fake; TK-011 remove essa falsa disponibilidade. |

Última evidência anterior: EV-022 (agora stale). `review.md` contém passes locais Standards/Spec por ticket, sem aprovação externa inventada. A suíte UI de TK-010 tinha 92 testes; **a suíte atual de TK-011 passou 96/96**, verificada após a interrupção, em `target/qa-tools/model-offer-ui-suite.log`.

## TK-011 — alterações atuais e próximos passos

Objetivo: quando ditado não tem modelo, oferecer cartão com recomendado/tamanho/Baixar, progresso/cancelamento/retry, seleção persistida após instalação; depois comprovar funcionamento real. Leia o ticket para owned_areas e plano técnico.

Reconhecimento real confirmou: com worker engines presente e perfil sem modelo, `ptt_press` retorna `{code: "asr", message: "model_missing"}` antes de abrir microfone. App antigo só mostra aviso genérico. Recomendação no hardware atual: **Whisper Turbo**, **1.624.555.275 bytes** (~1,62 GB). Nenhum modelo foi baixado ainda.

Alterações implementadas, ainda não empacotadas no app nativo:

- `Host.ptt_press` publica Failed model_missing/worker_missing para Overlay, inclusive caminho do atalho; mantém erro asr.
- `AsrBackend::Unavailable` substitui Fake vazio em configuração de produção sem worker; transcriber preserva erro e fallback cloud explícito. Demo/testes continuam podendo usar Fake.
- `InputBar` trata falta de modelo com cartão inline e falta de worker com mensagem de reparo localizada.
- Novo `VoiceInstallCard.tsx`: recomendado/tamanho, download, progresso, cancelar, retry, voiceSelect após Done para persistir, fecha sem captura automática. Installed pode selecionar sem baixar.
- `state/app.ts` não transforma erro literal `cancelled` em notificação de falha; cartão mostra status de cancelamento.
- Últimas correções: snapshot da tentativa é estado React (ref não re-renderizava no segundo retry); ID ativo preservado quando idioma altera recomendação.
- Dicionários PT/EN atualizados. Nenhum novo formato IPC foi introduzido.

Reds/greens executados:

| Procedimento | Logs em `target/qa-tools/` | Resultado |
| --- | --- | --- |
| Host sem modelo emite Failed, zero AudioSource opens | model-offer-host-red-final.log / model-offer-host-green.log | Red comportamental válido -> green |
| Config nativa de produção nunca usa Fake | model-offer-production-red.log / model-offer-production-green.log | Red -> green via cargo test aura-desktop |
| Oferta recomendado + 10 MiB literal | model-offer-ui-red.log / model-offer-ui-green.log | Red -> green |
| Done seleciona ID e não inicia captura | model-offer-complete-green.log | Green |
| Cancelamento + retry | model-offer-cancel-red-final.log / model-offer-cancel-green-final.log | Green após correção de render no retry |
| Troca de idioma não troca download ativo | model-offer-language-red.log / model-offer-card-green-final.log | Red mostrou Other Voice -> green 4/4 |
| Suíte UI atual | model-offer-ui-suite.log | **96/96, 15 arquivos** |

`model-offer-cancel-green.log` é uma tentativa **falha**, apesar do nome. Não citá-la como aprovação. `model-offer-types.log` passou antes das últimas pequenas mudanças; repetir tipos. `cargo fmt --all` foi executado antes das últimas alterações TS. Ainda faltam workspace/golden/clippy/build/revisão e toda a validação nativa do cartão/download/ASR. O comando de build frontend após suíte foi interrompido antes de começar: `model-offer-ui-build.log` não existe.

Sequência recomendada, sem iniciar TK-012:

1. Conferir diff/pacote atual, repetir typecheck e checks Rust pertinentes (incluindo erro worker_missing e fallback cloud). Quatro testes do cartão e suíte96 já passaram, não inventar resultado dos demais.
2. Criar `apps/desktop/e2e/specs/model-install.e2e.ts` (owned_areas já contempla) e confirmar red do cartão no executável TK-010 com worker real/perfil sem modelo.
3. Build frontend e release produção; copiar artefato somente após sucesso e quando não estiver em uso.
4. Executar cartão em WebView nativa com download **real** recomendado, acompanhar progresso/cancelamento quando pertinente, hash de instalação, seleção/restart persistidos e início/cancelamento de microfone.
5. Executar ASR com fala sintética conhecida, worker real e modelo real. Preferir loopback virtual CABLE Input -> CABLE Output via WASAPI, sem saída física/dados pessoais. API offline `speak`/SAPI pode produzir WAV; código/helper ainda **não foram explorados ou escritos**. Se usar teste WorkerClient separado + captura nativa, registrar limite: isso não prova fala completa pela UI. Não declarar aceite completo sem evidência exigida.
6. Atualizar audit/review, evidence add após execução, estado/checkpoint/render via runner. Só então seguir ticket seguinte.

## Ambiente de execução nativa

- Windows 11; duas telas atualmente 100%: DISPLAY2 1920×1080 em (0,0); DISPLAY1 1366×768 em (1920,0). Pendência de DPI125/150 é real.
- Microfones: `Microfone (fifine Microphone)` padrão e `CABLE Output (VB-Audio Virtual Cable)` não padrão. Saídas ainda não inventariadas para loop de áudio.
- tauri-driver instalado. Edge/WebView2 **154.0.4258.53**; msedgedriver correspondente em `target/qa-tools/edge-154/msedgedriver.exe`.
- Automação real: WDIO do repositório + probes aura-win. **CUA native está desabilitado**; CUA browser/preview não representa execução de janela Windows.
- WDIO serial em porta4444; config prepara provedor Ollama sem chave e onboarding em perfil isolado. Preserve testes de restart separados.
- App-server fixado rust-v0.159.0 em `target/qa-profile-panels/bin/codex/rust-v0.159.0/bin/codex-app-server.exe`. Não atualizar versão. Real-server tests usam upstream loopback controlado, sem inferência paga/conta pessoal.
- Tauri `window.__TAURI_INTERNALS__.invoke` não pode ser sobrescrito (immutable). Observe eventos nativos e persistência, não use interceptador impossível.

Artefatos existentes:

| Arquivo | SHA256 / propósito |
| --- | --- |
| target/qa-tools/aura-qa.exe e target/release/aura.exe | `805B30DFE31F825661E95584F495F69EA6D64037784DF9CC413F8DEECC2D01F5`; TK-010 produção, sem demo/e2e. **Não contém TK-011**. |
| target/qa-tools/aura-auth-qa.exe | `A793A3CB23C907F51CC31CF95F5AAF63F43D858035D50102BB2A19E0F51716FE`; e2e auth local, sem demo. |
| target/qa-tools/aura-original-2026-10-03.exe | `592C2010FE11AC9D6B8B36BF04AFAC1D5E012AF01595CE82490C8251A15CF055`; baseline preservado, não sobrescrever. |
| target/qa-tools/aura-worker.exe | `8E6874BCBACDE0D0D65841A459B9F540660C59D54A5575B594A332E563CFD45B`; worker **engines reais** recém-compilado. |

Worker engines originalmente nunca compilava por falta de libclang. Foi resolvido sem dependência de produto/install global: wheel oficial `libclang==18.1.1`, Apache2+LLVM exceptions, extraído em target. `LIBCLANG_PATH` correto: `target/qa-tools/libclang18/libclang-18.1.1.data/platlib/clang/native`. CMake já existe. Log aprovado: `asr-worker-engines-libclang-build-final.log`. As duas tentativas anteriores de preparação falharam e não são reds do produto.

**A presença desse worker ao lado de aura-qa muda os testes de voz antigos:** agora eles falham model_missing sem modelo. Não remover worker para fabricar greens. Executar depois da instalação com modelo real e atualizar evidência/limites. Build de engines sozinho não aprova ASR.

Recon nativa em `target/qa-tools/asr-missing-recon.e2e.ts`, log `asr-missing-recon-native.log`, perfil `target/qa-profile-asr-missing`. Catalog mostra sete modelos, nenhum instalado/selecionado. Outros tamanhos observados: Whisper Base147951465, Small487601967, MediumQ5 539212467, TurboQ5 574041195; ParakeetV2 661331545/V3 670619803. URLs/hashes em `crates/aura-asr/models.toml`.

## Comandos de referência

```powershell
python .hybrid/hybrid.py package --project . --effort 011-correcao-auditoria-e2e --ticket TK-011 --json
pnpm -C apps/desktop typecheck
cargo test --workspace --exclude aura-desktop
pnpm -C apps/desktop build
cargo build -p aura-desktop --release --features tauri/custom-protocol
Copy-Item -LiteralPath target/release/aura.exe -Destination target/qa-tools/aura-qa.exe

$env:AURA_HOME = Join-Path (Get-Location) 'target/qa-profile-voice-real'
$env:AURA_E2E_APP = Join-Path (Get-Location) 'target/qa-tools/aura-qa.exe'
$env:PATH = (Join-Path (Get-Location) 'target/qa-tools/edge-154') + ';' + $env:PATH
pnpm -C apps/desktop/e2e test --spec ./specs/model-install.e2e.ts
```

O novo spec model-install **ainda não existe**. Download1,62GB exige timeout explícito (ex.:20min); não bloquear ferramentas longamente sem updates. Release link costuma levar2–3min. Testes ignored real_app_server precisam AURA_CODEX_BIN apontando para binário fixado; ler testes antes de rodar. Golden só regenerar se formato mudar: `UPDATE_GOLDEN=1` em ambiente PowerShell, depois teste de contrato.

Use `ticket update`, `evidence add --executed`, `checkpoint write --expected-revision 42` (ou revisão atual lida) e `render`; consulte `--help` para argumentos exatos. Não adulterar JSON/frontmatter para simular transição.

## Fila restante

Consultar texto literal de cada achado/spec, não implementar só por esta lista:

- 012 catálogo ASR velocidade/acurácia/hardware; 013 seleção entrada/saída, medidores/teste/fallback; 014 esforço de raciocínio por capacidade.
- 015 renomear histórico; 016 paginação50; 017 /compactar; 018 imagem clipboard; 019 editar provedores; 020 protocolo/headers; 021 capacidades de modelo manual.
- 022 skills origem/habilitar/editar; 023 args MCP com aspas; 024 tools vazio em MCP desabilitado; 025 status/logs MCP.
- 026 memória revisar/editar/excluir; 027 CSP frame-src e HTML srcDoc; 028 fallback preview PDF.
- 029 buffer recente1–30min; 030 retenção dias/espaço; 031 access log hora/conversa/thumb; 032 player duração/anexar; 033 buffer fontes combinadas/anexar.
- 034 TTS provedor/voz/consentimento; 035 CtrlShiftL/CtrlShiftEnter; 036 perfil modelo padrão; 037 diagnóstico pipeline.
- 038 updater pubkey placeholder/assinatura real (pode exigir chave legítima; não gerar aprovação fictícia); 039 retomar onboarding; 040 instrução Escape; 041 startup vs tray.
- Retestar001 com DPI125/150 e esclarecer relato de largura; consolidar evidências stale001–010 e regressões finais.

## Cuidados na retomada

- O working tree contém todas as correções anteriores, arquivos untracked do esforço/testes, auditoria original/evidências e `crates/aura-app/aura-e2e9qtogT/` preexistente. Não excluir esses arquivos como lixo sem entender origem.
- Na leitura de processos deste handoff **não havia aura, aura-worker, cargo, tauri-driver ou msedgedriver ativos**. Havia processos node antigos; uma sessão Vite26724 pode estar viva. Não matar processos node indiscriminadamente.
- O usuário interrompeu um exec durante a suíte UI; o log foi conferido depois e ela terminou96/96. Build não iniciou. Nenhum reteste nativo TK011 ocorreu.
- PowerShell não interpreta `\"` como escape de aspas: prefira apply_patch para texto com aspas. Um comando cujo último Get-Content sucede pode retornar0 apesar de teste anterior falhar: confira logs/exit do teste.
- Falha de hunk em apply_patch pode impedir patch inteiro; conferir resultado. Nunca nome de arquivo green como prova sem ler resultado.
- Mantenha atualizações concisas em português durante execução. Seja explícito sobre simulação, evento sintético, seam testada e limites reais. Não prometer encerramento com42 resolvidos enquanto isso não estiver demonstrado.

## Prompt sugerido para a outra IA

> Continue a correção integral dos42 achados de docs/qa/auditoria-e2e-2026-10-02.md no repositório Aura. Leia AGENTS.md e este handoff, depois o esforço specs/011-correcao-auditoria-e2e. Preserve todo o working tree. Retome TK-011 (in_progress/not_run, checkpoint42), valide cartão/download e ASR no app de produção com worker/modelo reais antes de fechar. Siga Hybrid, um ticket por vez, confirmação de relatos/riscos, TDD, execução nativa e evidências honestas. Depois resolva012–041 e pendências001/042, renove evidências stale e atualize relatório. Não marque concluído o que não foi verificado. Não use subagentes ou faça commits sem autorização adicional.
