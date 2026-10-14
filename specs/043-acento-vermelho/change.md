---
schema: hybrid/change
schema_version: 1.0
effort_id: 043-acento-vermelho
revision: 1
status: accepted
profile: compact
---

# Change: Acento vermelho da marca

## Objetivo e limites

Pedido do usuário (13/10/2026): a identidade do Aura passa a ser vermelha; o roxo/índigo atual
(`#5b5bf7`, logo `#5B5BF7 → #A24BF2`) deixa de ser usado. A landing page (044) e as capturas de
divulgação dependem dessa troca para ficarem coerentes com o app.

Inclui: cor de destaque padrão, logo (SVG/PNG), ícones do Tauri (janela, bandeja, instalador),
neutros do tema levemente quentes para combinar com o vermelho.
Exclui: material de mídia histórico já publicado (`docs/media/2026-10-11`), versão/release.

## Contrato de comportamento

- Entradas: Configurações › Geral › Cor de destaque, sem cor escolhida (`accentColor: null`).
- Saída: o destaque padrão é `#d92d35` (`rgb(217, 45, 53)`) no Overlay e em Configurações; o
  amostrador "Padrão" mostra esse vermelho; o campo hex mostra `#d92d35`.
- Erros/invariantes: cor personalizada continua válida e persistida (012); alto contraste continua
  usando `Highlight`; texto branco sobre o botão primário mantém contraste ≥ 4,5:1 (4,8:1).
- Compatibilidade: formato de `settings` e IPC inalterados. O preset "Vermelho" (`#d13438`) sai da
  lista porque duplicaria o padrão; quem o escolheu antes vê a cor como "personalizada".

## Requisitos e aceite

- **FR-001** — O destaque padrão DEVE ser o vermelho da marca `#d92d35`.
- **AC-001** — Dado Configurações › Geral sem cor escolhida, então o amostrador "Padrão" tem fundo
  `rgb(217, 45, 53)`, não há opção "Vermelho" separada e o campo hex mostra `#d92d35`.
- **FR-002** — Logo e ícones DEVEM usar o gradiente `#FF5A5F → #B3141F`, sem roxo.
- **AC-002** — `docs/assets/logo.svg` e `apps/desktop/public/logo.svg` não contêm `#5B5BF7`/`#A24BF2`;
  os ícones em `apps/desktop/src-tauri/icons` são gerados por `tauri icon` a partir do logo vermelho.

## Leitura e mapa de alterações

- `apps/desktop/src/styles/app.css` → `--accent`, `--surface-rgb`, `--strong-rgb`, `--text*` — vermelho e neutros quentes; existing.
- `apps/desktop/src/settings/General.tsx` → `ACCENTS`, `DEFAULT_ACCENT`; existing.
- `apps/desktop/src/i18n/{pt-BR,en}.ts` → remove `general.accent.red`; existing.
- `apps/desktop/src/settings/SettingsApp.test.tsx` — teste AC-001; existing.
- `apps/desktop/e2e/specs/accent-color.e2e.ts` — "Padrão" volta ao vermelho; existing.
- `docs/assets/logo.{svg,png}`, `apps/desktop/public/logo.svg`, `apps/desktop/src-tauri/icons/*`; existing.
- `docs/design/ui-ux.md` → tabela de tokens; existing.

## Plano breve

Seam: `SettingsApp` com a ponte de demonstração (Vitest). Abordagem: trocar o token único de
destaque; ícones regenerados com `pnpm -C apps/desktop tauri icon ../../docs/assets/logo.svg`,
copiando só os arquivos já versionados. Dependências: none.

## Sequência e tarefas

- [x] C-001 AC-001 red: amostrador "Padrão" era `rgb(91, 91, 247)`.
- [x] C-002 Token, presets, i18n → green.
- [x] C-003 Logo e ícones (AC-002).
- [x] C-004 Regressão da UI (`pnpm -C apps/desktop test`, `typecheck`).

## Validação e evidência

Comando/procedimento: `pnpm -C apps/desktop test`; `pnpm -C apps/desktop typecheck`.

Resultado executado: ver `evidence/`.

Limitações: E2E WebDriver (`accent-color.e2e.ts`) não executado nesta máquina (falta
`msedgedriver` da versão do WebView2). O ícone da bandeja vem do ícone
da janela (`default_window_icon`), regenerado aqui.

## Publicação 0.3.2

Pedido do usuário (14/10/2026): enviar tudo ao GitHub e criar a release com a nova cor. Versão
0.3.2 em Cargo/package/tauri.conf. `prepare-sidecars.ps1 -Release -DirectML` (com `LIBCLANG_PATH`
da cópia local) e `tauri build --ci --bundles nsis,msi` exit 0; NSIS e MSI aceitos pela pública
permanente e rejeitados pela chave aposentada e com bytes adulterados (`updater-verify`);
pública embutida = permanente; ProductVersion 0.3.2 em aura.exe e no NSIS. Regressão:
`cargo test --workspace --exclude aura-desktop` 519 passed; UI 254/254; typecheck limpo.

| Asset | SHA-256 |
| --- | --- |
| Aura_0.3.2_x64-setup.exe | `d19962626c5846428244feff831a488996c34dbb81b6b7d44310b132c6dea751` |
| Aura_0.3.2_x64_en-US.msi | `80359db08ace31291208d40b1ae3863e745ff007bd7c6bc527b4f3f94191ec34` |

Não executado: instalação/atualização em Windows limpo.

## Condição de retorno

Retornar se o vermelho padrão reprovar o contraste mínimo do botão primário.

## Estado

`change.md` é a fonte canônica deste esforço compacto. Não crie `tasks.md` ou tickets paralelos para esta mudança.
