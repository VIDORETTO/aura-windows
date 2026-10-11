# Fontes do material visual

`composicoes.html` contém os 56 layouts editáveis com imagens e logo relativos. Abra o arquivo no navegador para conferir as composições.

`indice.json` relaciona cada composição ao arquivo final. `verificacao.json` registra dimensões, conteúdo e integridade de layout na exportação original.

## Regeneração

Use Node.js 22 ou posterior. Dentro desta pasta:

```powershell
npm install
npx playwright install chromium
node compose.cjs
node export.cjs
```

`compose.cjs` exporta os PNG das composições e os índices. `export.cjs` gera WebP, verifica as dimensões e atualiza a galeria. A aparência depende das fontes Segoe UI instaladas no Windows.

Para refazer também as capturas, execute a prévia do repositório em outro terminal (`pnpm -C apps/desktop dev --port 1421`), depois rode `node capture.cjs` antes dos dois comandos de exportação. `AURA_PREVIEW_URL` permite usar outra URL local. O script injeta apenas dados sintéticos no backend de demonstração da prévia.

A regeneração sobrescreve as imagens locais. Depois de editar, recalcule `SHA256SUMS.txt` antes de distribuir o pacote.
