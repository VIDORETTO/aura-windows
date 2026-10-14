---
schema: hybrid/change
schema_version: 1.0
effort_id: 044-landing-page
revision: 1
status: accepted
profile: compact
---

# Change: Landing page

## Objetivo e limites

Pedido do usuário (13–14/10/2026): uma landing page moderna, limpa, leve e com motion para
apresentar o Aura e baixar a última release do GitHub; paleta vermelha, sem o roxo do app.
Plano detalhado em `apps/landing/PLANO.md`; imagens em `apps/landing/IMAGENS.md`.

Inclui: `apps/landing` (Astro estático), capturas do app real recoloridas, favicons/OG,
workflow do GitHub Pages. Exclui: domínio próprio, versão em inglês, vídeo, imagens geradas por
IA e capturas manuais do Windows (IMG-01/03/07/09).

## Contrato de comportamento

- Entradas: API `releases/latest` do GitHub no build e, uma vez por sessão, no navegador.
- Saída: botões "Baixar" apontam para o `Aura_<versão>_x64-setup.exe` da última release, com
  versão, tamanho e SHA-256; links para MSI, notas e todas as versões.
- Erros/invariantes: sem API (limite, rede), o build não falha e os botões apontam para
  `/releases/latest`; o navegador só altera a página quando a release é mais nova que a do build;
  rascunhos e respostas que não são release são ignorados. A página funciona sem JS.
- Compatibilidade: nenhum código do app muda neste esforço.

## Requisitos e aceite

- **FR-001** — O download DEVE levar ao instalador NSIS da última release, nunca à assinatura `.sig`.
- **AC-001** — Dada a resposta da API da v0.3.1, então o asset escolhido é `Aura_0.3.1_x64-setup.exe`
  com SHA-256 `def9e599…8197`; o MSI é `Aura_0.3.1_x64_en-US.msi`; rascunho → nenhuma release.
- **AC-002** — `newer("0.10.0","0.9.2")` é verdadeiro; `newer("0.3.1","0.3.1")` é falso;
  `newer("0.4.0","0.4.0-beta.1")` é verdadeiro.
- **FR-002** — A página DEVE cumprir o orçamento do PLANO.md §7 e AA de contraste.
- **AC-003** — Após `pnpm build`, `pnpm budget` passa (JS ≤ 12 KB, CSS ≤ 20 KB gzip) e o Lighthouse
  dá Acessibilidade 100 em desktop e mobile, sem rolagem horizontal a 390 px.

## Leitura e mapa de alterações

- `apps/landing/src/lib/release.ts` → `parseRelease`, `latestRelease`, `newer`, `megabytes`; new.
- `apps/landing/src/lib/release.test.ts` — AC-001/AC-002; new.
- `apps/landing/src/components/*.astro`, `layouts/Layout.astro`, `pages/index.astro`, `styles/global.css`, `scripts/main.ts`; new.
- `apps/landing/scripts/{capture,art,budget}.mjs` — capturas, arte e orçamento; new.
- `.github/workflows/landing.yml` — build, teste e Pages; new.
- `docs/HANDOFF.md` → §6.3 e fila; existing.

## Plano breve

Seam: `parseRelease`/`newer` (node --test) e a página construída (budget + Lighthouse). Abordagem:
HTML estático com três ilhas de comportamento num único script; imagens do app real por Playwright
sobre a prévia de demonstração. Dependências: 043 (acento vermelho no app).

## Sequência e tarefas

- [x] C-001 Parser da release e comparação de versões (AC-001, AC-002).
- [x] C-002 Capturas recoloridas e novas (pesquisa web, voz, seletor de modelo).
- [x] C-003 Página: hero com demo, passos, recursos, privacidade, modelos, atalhos, FAQ, download.
- [x] C-004 Favicons, OG, SEO, JSON-LD, sitemap, robots.
- [x] C-005 Orçamento e Lighthouse (AC-003); correções de CLS e contraste.
- [x] C-006 Workflow do GitHub Pages.

## Validação e evidência

Comando/procedimento: `pnpm -C apps/landing test`; `pnpm -C apps/landing check`;
`pnpm -C apps/landing build && pnpm -C apps/landing budget`; Lighthouse 12.8.2 (Edge) em
`astro preview`, mobile e desktop; Playwright a 1440 × 900 e 390 × 844.

Resultado executado: ver `evidence/`.

Limitações: LCP mobile
simulado 2,2 s acima da meta de 1,8 s (troca de fonte do título); IMG-01/03/07 pendentes.

## Condição de retorno

Retornar se a API do GitHub deixar de expor os assets da release ou se o Pages exigir outro host.

## Estado

`change.md` é a fonte canônica deste esforço compacto. Não crie `tasks.md` ou tickets paralelos para esta mudança.
