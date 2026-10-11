---
schema: hybrid/plan
schema_version: 1.0
effort_id: 040-overlay-estavel-no-monitor
revision: 1
spec_revision: 1
status: ready
---

# Plano — monitor e âncora atuais

Consome spec r1; baseline ed9a9654c6e76e48186fc010a12eba77023b00f5. Preservar diffs de 038/039, um ticket, sem dependência/migração/API Windows nova.

## Módulos e abordagem

`crates/aura-core/src/placement.rs`: novo monitor_for_rect público/puro; maior interseção positiva com área útil, desempate estável, área i64, None fora de todos. Literais de origem negativa/DPI/fronteira/ausência.

`apps/desktop/src-tauri/src/overlay.rs`: ler retângulo atual antes de mudar mínimo/tamanho; set_mode escolhe monitor geométrico, fallback contextual somente sem interseção. Tamanho salvo/padrão do novo modo com DPI atual, mas âncora x/y atual, limitada à área útil. remember_placement usa monitor geométrico. show oculto continua contextual. Session/UI não mudam.

Alternativas descartadas: fixar monitor principal/configuração, seguir cursor nas transições, trocar Aplicativo anterior artificialmente. Seleção baseada em área útil pode divergir do monitor nativo sobre a barra de tarefas; recuperação continua limitando a janela à área disponível.

## Seams, TDD e validação

Novo `apps/desktop/e2e/specs/monitor-stability.e2e.ts`: processo nativo/perfil isolado, QA helper guardado existente arrasta, IPC público lê posição/muda modos/salva; sem injetar Session/contexto. Novo `crates/aura-win/examples/qa_foreground.rs`: cria somente sua própria janela nativa vazia, aguarda foreground real por até20s e permanece até30s após READY; não inspeciona/envia input a outros apps. Faz abertura contextual em A/B determinística. Compilar `cargo build -p aura-win --example qa_foreground`. Capturar previous_app, escolher outro monitor, confirmar red por retorno ao monitor anterior. Green nos dois monitores, posição salva e modos. Para Minibar, modelo externo scripted em loopback; shell/Host/Codex/UI reais.

Raiz: `cargo test -p aura-core placement`; `cargo test -p aura-desktop`; `cargo fmt --all -- --check`; `cargo clippy -p aura-core -p aura-desktop --all-targets -- -D warnings`; `pnpm -C apps/desktop typecheck`; `pnpm -C apps/desktop/e2e test -- --spec ./specs/monitor-stability.e2e.ts`; regressões model-picker-recovery/resize em build OS nativo atual.

A instância de produção está em uso. QA: TAURI_CONFIG identifier app.aura.qa040 somente no processo de build e `cargo rustc -p aura-desktop --release --features e2e,tauri/custom-protocol --bin aura -- -C extra-filename=-qa040`; copiar de target/release/deps para TEMP/aura-qa.exe. A identidade QA evita instância única da produção. Sufixo confirmado no manual oficial https://doc.rust-lang.org/rustc/codegen-options/index.html#extra-filename. Primeiro build sem tauri/custom-protocol compilou, mas origin de IPC inválido impediu antes do comportamento: erro de ambiente, não red. A feature incorpora o protocolo de assets exigido pelo empacotamento Tauri. Produção final sem configuração/feature e2e, mantendo tauri/custom-protocol.

## Gates

Detalhe QA: qa_resize Move aceita ponto inicial medido pelo DOM no espaço livre do cabeçalho com data-tauri-drag-region, convertido por scale_factor. O centro do cabeçalho pode ser botão após selecionar modelo; essa falha de arraste não representa salto produtivo. Arrastar pelo rótulo próximo à borda também acionou Snap nativo (largura 640→681, x1913/y0); o espaço livre evita esse interferente, sem relaxar o oráculo de posição. Guard de filename/janela única/foreground e limites do client preservados. A fixture de contexto pode precisar de ativação por Computer Use exclusivamente em sua janela vazia retornada pelo inventário; nunca alvo navegador/perfil do usuário. Após ativação, o próprio helper ainda exige GetForegroundWindow igual à janela antes de READY.

G1 pedido aceito; G2 seams/oráculos definidos; G3 package ready antes de implementar. Foreground guard preservado. AC-001 sem reâncora de saved: semear expandido B em (2000,80) com altura física 500 comprovada por IPC; arrastar compacto para (2080,60) e alternar. Ambas as posições comportam 500 na área útil 720. O primeiro preparo com altura722 do monitor A excedeu B720 e corretamente limitou y a0; falha de precondição, sem mudar expectativa do produto. Mudança de posicionamento exige renovar geometria/resize 038; 100% não aprova 150/200 reais. Evidência somente após execução, revisão Standards/Spec separadas antes de done. Não instalar/publicar ou encerrar a instância do usuário sem necessidade.
