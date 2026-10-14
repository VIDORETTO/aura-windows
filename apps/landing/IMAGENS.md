# Landing page — pendências de imagem

Cada item diz o que é, onde entra, formato e como deve ficar. Paleta e tokens em [PLANO.md](PLANO.md#3-identidade-visual).

Regras gerais:

- **Nenhum roxo ou índigo.** As imagens de `docs/media/2026-10-11/01-portfolio` e `02-instagram` **não serão usadas**: têm o logo e o acento roxos e textos próprios embutidos.
- As capturas de `docs/media/2026-10-11/03-capturas-originais` servem de **referência de enquadramento**, mas também têm acento roxo (ponto de status, balão do usuário, botão enviar). Elas serão **recapturadas** (IMG-02).
- Só dados de demonstração. Nenhum e-mail, nome, arquivo ou janela pessoal.
- Entregar o arquivo-mestre (PNG sem perda) em `apps/landing/src/assets/`; o build gera AVIF/WebP. Não commitar vídeos acima de 3 MB nem capturas brutas desnecessárias.

## Resumo

| ID | Imagem | Quem produz | Prioridade | Estado (14/10/2026) |
| --- | --- | --- | --- | --- |
| IMG-01 | Fundo da área de trabalho do hero | geração por IA ou arte | Alta | **Pendente.** A página usa um degradê vermelho em CSS (`.wall` em `Hero.astro`) no lugar. |
| IMG-02 | Recaptura das telas com acento vermelho | `pnpm capture` | Alta | Feito, incluindo as novas 02e, 02f e 02g. |
| IMG-03 | Overlay sobre um app real do Windows | captura manual | Alta | **Pendente.** O passo 1 usa o Overlay compacto sobre o degradê. |
| IMG-04 | Logo vermelho | código (SVG) | Alta | Feito no app (043): `docs/assets/logo.svg`, ícones do Tauri. |
| IMG-05 | Imagem de compartilhamento (Open Graph) | `pnpm art` | Média | Feito: `public/og.png`. |
| IMG-06 | Favicons | `pnpm art` | Média | Feito. |
| IMG-07 | Seleção de região na tela | captura manual | Média | **Pendente.** O passo 2 usa o menu `@`. |
| IMG-08 | Textura de grão | código (SVG) | Baixa | Feito em `global.css`. |
| IMG-09 | Vídeo curto do hero (opcional) | gravação ou Remotion | Baixa | Fora da v1, como recomendado. |

Para trocar um item pendente: salve o PNG em `src/assets/art/` (IMG-01) ou `src/assets/screens/` (IMG-03, IMG-07) e substitua o fundo em CSS ou a imagem do passo correspondente.

---

## IMG-01 — Fundo da área de trabalho do hero

- **Onde:** atrás do OverlayDemo no hero, levemente desfocado (`blur(2px)`) e escurecido.
- **Formato:** PNG 3840 × 2160 (16:9), sem texto, sem ícones, sem barra de tarefas, sem logos.
- **Descrição:** papel de parede abstrato no espírito do Windows 11 (formas fluidas, como pétalas ou dobras de tecido translúcido), porém em **vermelho escuro e preto**. Base quase preta `#0A0909`. Uma "flor" de luz vermelha (`#F04044` → `#A81B26`) concentrada no **terço direito, um pouco acima do centro**, onde o Overlay vai ficar. O **terço esquerdo deve ser quase preto e liso**, porque o título fica ali. Sem azul, sem roxo, sem magenta: matizes apenas entre 350° e 10°.
- **Iluminação:** suave, sem reflexos duros; profundidade com desfoque nas bordas.
- **Prompt sugerido:** "Abstract Windows 11 style wallpaper, translucent folded glass ribbons forming a soft bloom, deep crimson and scarlet light (#F04044 to #A81B26) on near-black background (#0A0909), bloom positioned right-of-center slightly above middle, left third almost pure black and empty, smooth gradients, subtle depth of field, no text, no logo, no purple, no blue, no magenta, 16:9, 4K."
- **Aceite:** com a captura do Overlay por cima, o título branco à esquerda tem contraste ≥ 7:1 em toda a área do texto.

## IMG-02 — Recaptura das telas com acento vermelho

- **Onde:** hero, "Como funciona", bento, privacidade, modelos.
- **Como:** com a prévia aberta (`pnpm -C apps/desktop dev --port 1421`), rodar `pnpm -C apps/landing capture` (`scripts/capture.mjs`). O app já usa o vermelho (043): nenhum override de CSS é necessário.
- **Formato:** PNG 2× (mesma escala das originais), tema escuro, opacidade 100%, saída em `apps/landing/src/assets/screens/`.
- **Conferir em cada captura:** ponto de status, balão do usuário, botão enviar, toggles, abas e links ficaram vermelhos; nenhum pixel roxo (amostrar `#5B5BF7` com tolerância deve dar zero).

### Telas que já existem (só recolorir)

| Arquivo novo | Base | Uso |
| --- | --- | --- |
| `overlay-compacto.png` | `overlay-compacto.png` | Hero, passo 1 |
| `conversa.png` | `conversa.png` | Hero, passo 3 |
| `contexto.png` | `contexto.png` | Passo 2 |
| `tarefas.png` | `tarefas.png` | Bento — modos |
| `arquivos.png` | `arquivos.png` | Bento — anexos |
| `reuniao.png` | `reuniao.png` | Bento — reuniões |
| `preparo-painel.png` | `preparo-painel.png` | Bento — reuniões (alternativa) |
| `minibar.png` | `minibar.png` | Bento — minibar |
| `historico.png` | `historico.png` | Bento — histórico |
| `extensions-secao-1.png` | idem | Bento — extensões |
| `privacy-secao-1..3.png` | idem | Privacidade |
| `registro-de-acesso.png` | seção de `privacy` | Privacidade |
| `providers.png` | `providers.png` | Modelos |
| `voice-secao-1.png` | idem | Bento — voz (fallback) |

### Telas novas (não existem hoje)

**IMG-02e — Pesquisa web com fontes.** O recurso principal da 0.3.x ainda não tem captura.
- Overlay expandido, 1640 × 1220 (mesmo tamanho de `conversa.png`).
- Pergunta do usuário: "Quais são as novidades da WCAG 2.2? Cite as fontes."
- Resposta com 3 tópicos curtos e marcadores de fonte numerados, mostrando abaixo a lista de fontes clicáveis (w3.org e mais uma página pública) e o indicador "Pesquisou 2 páginas". Um link de fonte em estado hover.
- Deve caber legível num cartão de ~800 px de largura: nada de parágrafos longos.

**IMG-02f — Ditado por voz ativo.**
- Overlay compacto, 1640 × 160, com o microfone ativo (onda/nível de áudio animado congelado no pico) e o texto parcial "Resuma este documento em três tópicos…" aparecendo no campo.
- Se o estado de gravação não puder ser forçado na prévia, capturar do app real.

**IMG-02g — Seletor de modelo e raciocínio aberto.**
- Overlay expandido com o menu de modelo/raciocínio aberto (modelo selecionado + níveis Baixo/Médio/Alto). Recorte de ~900 × 700 em torno do menu.
- Uso: seção Modelos.

## IMG-03 — Overlay sobre um app real do Windows

- **Onde:** passo 1 de "Como funciona" e alternativa ao fundo do hero.
- **Formato:** PNG 2560 × 1600, captura real do Windows 11 em tema escuro, escala 100%.
- **Cena:** VS Code (tema escuro padrão) ou Microsoft Edge abertos em tela cheia com conteúdo neutro de demonstração (ex.: um arquivo `README.md` fictício ou a página da WCAG no w3.org). O **Overlay compacto do Aura recolorido** aparece no terço superior, centralizado, com a pergunta "Explique este trecho" já digitada.
- Barra de tarefas visível, com o ícone do Aura (vermelho) na bandeja. Papel de parede: IMG-01.
- **Remover/evitar:** nome de usuário, abas pessoais, notificações, relógio com data real (usar 14:00), outros ícones de bandeja identificáveis.
- **Aceite:** a pessoa entende em 2 segundos que "o Aura flutua por cima do que estou fazendo".

## IMG-04 — Logo vermelho

- Mesmo desenho de `docs/assets/logo.svg` (quadrado arredondado, anel grosso, ponto central, anel externo a 35%).
- Trocar o gradiente `#5B5BF7 → #A24BF2` por **`#FF5A5F` (topo esquerdo) → `#B3141F` (base direita)**; manter o brilho branco radial.
- Variantes: `logo.svg` (colorido), `logo-mono.svg` (só branco, para fundos vermelhos), `logo-mark.svg` (só os anéis, sem quadrado, para o motivo gráfico do hero).
- Produzido em código na fase L1. Se o app adotar o vermelho, o mesmo arquivo gera os ícones do Tauri.

## IMG-05 — Imagem de compartilhamento (Open Graph)

- **Formato:** PNG 1200 × 630, < 300 KB.
- **Layout:** fundo `#0A0909` com halo vermelho à direita; logo vermelho 72 px no canto superior esquerdo; título "Sua IA no Windows, a um atalho de distância." em Geist 600, 64 px, branco, duas linhas à esquerda; abaixo, keycaps `Ctrl` `Shift` `Espaço`; à direita, recorte do `conversa.png` recolorido inclinado 4° com sombra e halo.
- Margens de 64 px; nada importante nos 40 px das bordas (recorte de redes sociais).
- Gerar com uma página HTML + Playwright, igual a `docs/media/2026-10-11/fontes/compose.cjs`.

## IMG-06 — Favicons

- A partir do IMG-04: `favicon.svg` (com `prefers-color-scheme`), `favicon.ico` 32/48, `apple-touch-icon.png` 180 × 180 (fundo `#0A0909`), `icon-512.png`.

## IMG-07 — Seleção de região na tela

- **Onde:** passo 2 de "Como funciona" (alternância com o menu `@`).
- **Formato:** PNG 2560 × 1600.
- **Cena:** a mesma de IMG-03, com a camada escurecida de seleção do `@região` ativa e um retângulo selecionado sobre um gráfico ou trecho de código; borda do retângulo no acento vermelho e o rótulo de dimensões visível.

## IMG-08 — Textura de grão

- SVG com `feTurbulence` (`baseFrequency 0.9`, `numOctaves 2`), tile de 200 × 200, usado a 3–4% de opacidade sobre o fundo. Gerado em código; < 2 KB.

## IMG-09 — Vídeo curto do hero (opcional, v2)

- **Formato:** WebM (VP9) + MP4 (H.264), 1600 × 1000, 30 fps, 8 s em loop, sem áudio, ≤ 1,5 MB cada, mais pôster AVIF do último quadro.
- **Roteiro:** 0–1 s área de trabalho (IMG-03) → 1–1,6 s keycaps aparecem e são pressionados → 1,6–2,2 s Overlay desce → 2,2–4 s pergunta é digitada → 4–6,5 s resposta chega em streaming → 6,5–7,5 s clique em "Inserir no app" e o texto aparece no editor → 7,5–8 s fade para o início.
- Produção: gravação real do app recolorido ou composição em Remotion com as capturas de IMG-02/IMG-03.
