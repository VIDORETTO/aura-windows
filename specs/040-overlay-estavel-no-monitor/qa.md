# Verificação — estabilidade no monitor

Executado em 10/10/2026 (UTC 11/10), Windows 11 10.0.26200, MSVC, WebView2/EdgeWebDriver 154.0.4258.62. Baseline fixa `ed9a9654c6e76e48186fc010a12eba77023b00f5`; árvore de trabalho com mudanças locais de 038/039 preservadas.

## Causa e red

`set_mode` e `remember_placement` selecionavam o monitor pelo Aplicativo anterior. Ao abrir em DISPLAY1 e arrastar para DISPLAY2, expandir devolveu x2000 para x1280, embora a janela estivesse em B. Falha comportamental real em `native-red-3.log`; código do shell anterior à correção, equivalente ao baseline nesse arquivo. Falhas anteriores de origin/focus aconteceram antes desse comportamento e não são reds.

Teste puro inicial `cargo test -p aura-core current_window_on_negative_origin_monitor_does_not_follow_previous_app`: stub retornava None, esperado B; falhou, depois implementação por interseção e regressão passaram. Log `core-red.log`. Esses reds descrevem versões anteriores; fingerprints de código dos greens são os arquivos atuais.

## Execuções atuais

| Procedimento | Resultado observado |
| --- | --- |
| `cargo test -p aura-core` | 56 testes passaram; 11 de placement, incluindo origem negativa, DPI, empate, ausência, recuperação e mínimos. |
| `cargo test -p aura-desktop` | 1 teste passou; log desktop-tests.log. |
| `pnpm -C apps/desktop test` | 253 testes, 33 arquivos passaram; ui-tests.log. |
| `pnpm -C apps/desktop typecheck` | Passou. |
| `cargo clippy -p aura-core -p aura-win -p aura-desktop --all-targets -- -D warnings` | Passou após a última alteração dos helpers; clippy-final.log. |
| `cargo fmt --all -- --check`; `git diff --check` | Passaram; avisos de normalização CRLF de arquivos anteriores não são erros. |
| `pnpm -C apps/desktop/e2e test -- --spec ./specs/monitor-stability.e2e.ts` | 3/3 passaram, 30,4s; native-final-fit-anchor.log. |
| `pnpm -C apps/desktop/e2e test -- --spec ./specs/model-picker-recovery.e2e.ts --spec ./specs/resize.e2e.ts` | 2/2 seletores/teclado e 9/9 resize, 51s; regression-final.log. |

Raiz dos logs/perfis isolados: `C:/Users/gabri/AppData/Local/Temp/aura-040-monitor-20261010`. Nas execuções nativas: PATH começa com `target/qa-tools/edge-154`; AURA_E2E_APP aponta para `aura-qa.exe` nessa raiz; AURA_HOME termina em `final-fit-anchor-profile` ou `regression-profile`; AURA_CODEX_BIN=`C:/Users/gabri/AppData/Local/Aura/bin/codex/rust-v0.159.0/bin/codex-app-server.exe`. Nenhuma conta/chave do usuário é necessária para o modelo scripted de 040.

Monitor real A: DISPLAY1, área útil (0,0,1920,1032), escala1. B: DISPLAY2, (1920,0,1366,720), escala1.

- AC-001: posição expandida antiga B=(2000,80), altura500 comprovada por IPC. Arraste compacto B=(2080,60), Aplicativo anterior ainda A. Quatro alternâncias conservaram exatamente (2080,60); tamanho salvo não reancorou a janela.
- AC-002: salvar/reabrir B recuperou (2000,80), largura820; reabrir A recuperou (80,80), largura700. Só ações e IPC públicos, sem leitura lateral de SQLite.
- AC-001 durante Turno: rede do modelo foi segurada por fixture externa, UI enviou mensagem, foco em janela vazia A produziu Minibar real, clique voltou à conversa e resposta literal chegou pelo Codex/Host. Âncora antes/depois=(2000,80).
- AC-003: regra física, recuperação sem interseção/monitor ausente e limites comprovados pelos testes puros e integração revisada. Não foi feita desconexão física de monitor durante o teste.

Ativação assistida por Computer Use somente nas janelas vazias `qa_foreground.exe`, selecionadas do inventário. Helper exige foreground real antes de READY. Nenhum guard/oráculo foi relaxado. Arraste usa ponto observado no espaço livre do cabeçalho; centro com botão e Snap perto do rótulo foram interferentes de preparo. O primeiro caso de tamanho salvo usou altura722 numa área720 e falhou y60→0, corretamente limitado: precondição corrigida para500, sem mudar expectativa de estabilidade quando cabe.

## Artefatos reais

QA: `cargo rustc -p aura-desktop --release --features e2e,tauri/custom-protocol --bin aura -- -C extra-filename=-qa040`, TAURI_CONFIG somente no processo com identifier `app.aura.qa040`; copiado de deps para TEMP/aura-qa.exe. SHA256 `ECD389DD45E104EDC982696688585592F462BEAA861B943503AA5AC1EEDB20B9`. Sem demo; identidade/perfil próprios para não atingir a instância em uso.

Produção: removido TAURI_CONFIG do processo; `cargo rustc -p aura-desktop --release --features tauri/custom-protocol --bin aura -- -C extra-filename=-monitor-fix` terminou exit0 em2m56. Cópia de deps para `target/release/aura-monitor-fix.exe`. SHA256 `D440E061E0986416E46991AFBED332376D7978FB32C708FCD47A529D3D9698AA`. Sem features e2e/demo, sem bundle/instalação. Usar após Sair da versão antiga pela bandeja.

A instância antiga PID32200 e seu executável `target/release/aura.exe`, SHA256 `340CEE21FC66030D6A3BB2455D5610BB36D944E08FBA5D99CFD2F034203A510F`, foram preservados. Esta entrega compila a correção, não atualiza um processo que já está aberto.

## Limites

DPI nativo disponível é100%; 150/200% de 038 permanece pendente. Coordenadas/DPI diferentes nos testes puros não são evidência de WebView2 nesses DPI. Nenhuma garantia de ausência de toda causa possível no uso cotidiano (SC-002). Nenhuma mudança de IPC, schema, sidecar pin, dependência ou configuração pessoal neste esforço.
