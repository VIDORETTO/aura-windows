# Descoberta — seletores de modelo e raciocínio

Data: 06/10/2026. Baseline: `ed9a9654c6e76e48186fc010a12eba77023b00f5`, branch `release/0.2.0`. Trabalho preexistente não rastreado em `crates/aura-app/aura-e2e9qtogT/` e `docs/qa/evidencias-2026-10-02/` preservado; não é evidência deste esforço.

## Fatos e limites

- Relato: não consegue ver/escolher modelo/raciocínio. Não reproduzido em janela nativa; nenhum teste da aplicação executado nesta sessão.
- `apps/desktop/src/overlay/Header.tsx`, `Header`: monta `ModelPicker compact` e `ModelPicker`. Componente não removido no código atual.
- `ModelPicker.tsx`: opções dependem de catálogo/capacidades; não oferece recuperação de catálogo.
- `session.ts`, `loadCatalog`: `api.modelsList().catch(() => [] as ModelInfo[])` converte erro em vazio; provedores sofrem conversão equivalente. Atualização substitui listas. A proteção de corrida `catalogRequest` existe e deve ser preservada.
- `crates/aura-app/src/host.rs`, `Host::models`: `codex_models().await?` aborta antes de `plan_catalog` em erro. Caminho sustentado para menu sem opções; não prova causa do relato.
- Cabeçalho combina controles não retráteis e modelo `min-w-0`. `apps/desktop/src-tauri/src/overlay.rs`, `apply_minimum`: largura mínima 480 pixels lógicos. Compressão é hipótese visual.
- `ui/floating.tsx`, `Floating`, e `OverlayApp.tsx`, `useAutoHeight`: portal/crescimento já existem; portal não prova ausência de corte pela janela nativa.
- Cobertura anterior: `Composer.test.tsx`, `OverlayApp.test.tsx`; E2E composer/model-effort/model-efforts/profile-model/resize. EV de 013/017 não aprovam esta regressão.

## Diagnóstico delimitado para implementação

1. Confirmar executável/revisão em uso, sem publicar/reinstalar automaticamente.
2. Separar botão ausente/comprimido, catálogo vazio e painel fora da janela.
3. Observar `models_list` e Host → app-server, com erros redigidos.
4. Reproduzir erro de IPC controlado e cabeçalho a 480 px. Se DOM passar, seguir para janela real.
5. Criar regressão no comportamento observado. Falta de WebDriver/conta/sidecar não é red de produto.

## Fontes e decisão

Contratos: esforços 002/003/013/014/017; ADRs 0001/0003/0007/0009; `CONTEXT.md`, `docs/HANDOFF.md`. A [documentação oficial do app-server](https://learn.chatgpt.com/docs/app-server) descreve esforços suportados e padrão por modelo; conferir o schema fixado, pois documentação atual pode ser mais nova.

Perfil standard: UI, carga assíncrona, preferências, envio e Windows. Autorização atual: plano, tickets e TDD documental. Implementação pendente.
