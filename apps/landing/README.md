# Aura — landing page

Página de apresentação do Aura com o download da última release do GitHub. Astro estático, publicado no GitHub Pages por `.github/workflows/landing.yml`.

```powershell
pnpm -C apps/landing install
pnpm -C apps/landing dev       # http://localhost:4321/aura-windows/
pnpm -C apps/landing build     # dist/, lê a última release pela API do GitHub
pnpm -C apps/landing test      # parser da release (node --test)
pnpm -C apps/landing check     # astro check
pnpm -C apps/landing budget    # orçamento de JS/CSS/HTML após o build
```

Imagens:

- `pnpm capture` refaz as capturas em `src/assets/screens` a partir do app real. Antes, abra a prévia com `pnpm -C apps/desktop dev --port 1421`.
- `pnpm art` gera os favicons e `public/og.png`.

Documentos:

- [PLANO.md](PLANO.md): stack, identidade visual, seções, motion, desempenho, publicação e estado.
- [IMAGENS.md](IMAGENS.md): especificação e estado de cada imagem.
