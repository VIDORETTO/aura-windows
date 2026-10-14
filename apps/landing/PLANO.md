# Landing page do Aura — plano

Status: **publicado** (14/10/2026) em https://vidoretto.github.io/aura-windows/. Versão de referência do app: 0.3.1. Esforços: `specs/043-acento-vermelho` (app) e `specs/044-landing-page` (página).

Objetivo: uma página única, moderna, limpa e leve que explique o Aura em poucos segundos e leve a pessoa ao instalador da última release do GitHub. Paleta principal vermelha. Nada do roxo/índigo atual do app (`#5B5BF7` → `#A24BF2`).

Pendências de imagem: [IMAGENS.md](IMAGENS.md).

---

## 1. Princípios

1. **Um objetivo só:** baixar. O botão "Baixar para Windows" aparece no topo, no fim e na barra fixa; nenhuma outra ação compete com ele.
2. **Mostrar o app de verdade.** Capturas reais (recoloridas para vermelho), nada de mockup inventado.
3. **Motion com propósito.** Movimento conta a história "atalho → pergunta → resposta → ação". Só `transform`/`opacity`, 60 fps, desligado com `prefers-reduced-motion`.
4. **Leve.** Site estático, quase sem JavaScript. A navegação tem de ser fluida num notebook básico.
5. **Honesto.** Mesmo tom do README: sem promessas que o app não cumpre, aviso claro sobre o SmartScreen (o instalador não tem certificado Authenticode).

## 2. Stack

| Escolha | Motivo |
| --- | --- |
| **Astro 5** (saída estática) | HTML pronto no build, zero JS por padrão, `astro:assets` gera AVIF/WebP responsivos. Licença MIT. |
| **CSS próprio com tokens** (sem Tailwind) | A página é pequena; tokens em `:root` deixam o CSS final abaixo de 20 KB. |
| **TypeScript mínimo**, sem framework de UI | Só três "ilhas": revelar ao rolar, demo do hero e botão de download. |
| **Rolagem nativa** + `scroll-behavior: smooth` | Mais fluida e acessível que bibliotecas de smooth scroll (Lenis etc.), sem sequestrar a roda do mouse. |
| Animações: **CSS scroll-driven animations** com fallback por `IntersectionObserver` | Rodam fora da thread principal no Chrome/Edge; o fallback cobre Firefox/Safari. |
| Fonte: **Geist** + **Geist Mono** (OFL, auto-hospedadas, subset latin) | Geométrica e limpa; a mono serve para os atalhos (keycaps). |
| Ícones: SVG do **Lucide** inline | Mesmo conjunto do app. |

Pacote independente, como o desktop: `pnpm -C apps/landing dev | build | preview`. Não há workspace pnpm na raiz; não vamos criar um.

### Estrutura prevista

```
apps/landing/
  PLANO.md · IMAGENS.md · README.md
  package.json · astro.config.mjs · tsconfig.json
  public/            favicon, og-image, robots.txt
  src/
    assets/
      screens/       capturas recoloridas (vindas da recaptura, ver IMAGENS.md)
      art/           fundo do hero, texturas
    components/      Nav, Hero, OverlayDemo, Steps, Bento, Privacy, Models, Shortcuts, Faq, Download, Footer
    lib/release.ts   busca da última release (build + cliente)
    i18n/pt-BR.ts    textos (en.ts depois, mesmas chaves)
    styles/tokens.css · base.css
    pages/index.astro
```

## 3. Identidade visual

### Paleta (tema escuro como padrão)

| Token | Valor | Uso |
| --- | --- | --- |
| `--bg` | `#0A0909` | Fundo da página (preto levemente quente, não azulado) |
| `--bg-elev` | `#121010` | Seções alternadas |
| `--surface` | `#181515` | Cartões |
| `--line` | `rgb(255 255 255 / 0.08)` | Bordas, divisórias |
| `--text` | `#F6F3F2` | Texto principal |
| `--muted` | `#A8A09E` | Texto secundário |
| `--subtle` | `#6E6866` | Legendas, rodapé |
| `--red-300` | `#FF8A8A` | Destaques sutis, gradiente do título |
| `--red-400` | `#FF6B6B` | Links e texto vermelho pequeno sobre fundo escuro (contraste ≈ 7:1) |
| `--red-500` | `#F04044` | Cor da marca: brilhos, ícones, foco, traços |
| `--red-600` | `#D92D35` | Fundo do botão primário com texto branco (≈ 4,8:1, passa AA) |
| `--red-700` | `#A81B26` | Hover/pressed do botão, fim do gradiente do logo |
| `--red-900` | `#2A0B0E` | Fundos tingidos (badges, chip "Novo") |
| `--glow` | `rgb(240 64 68 / 0.35)` | Halo atrás do Overlay e do botão |

Regras: vermelho só em 5–10% da tela (CTA, brilho, palavras-chave). Nada de vermelho para "erro" na página, para não confundir marca com alerta. Verde só no ponto "online" da UI capturada.

Tema claro: fora do escopo da v1. A página força `color-scheme: dark`, coerente com as capturas.

### Tipografia

- Display: Geist 600, `clamp(2.75rem, 6vw + 1rem, 5.5rem)`, `letter-spacing: -0.035em`, `line-height: 1`.
- Títulos de seção: Geist 600, `clamp(2rem, 3vw + 1rem, 3.25rem)`.
- Corpo: Geist 400, 17–18 px, `line-height: 1.6`, largura máxima de 62ch.
- Eyebrow (sobretítulo): Geist Mono 500, 12 px, caixa alta, `letter-spacing: 0.14em`, cor `--red-400`.

### Elementos de assinatura

- **Anel do Aura:** o símbolo do logo (dois anéis + ponto) vira motivo gráfico. Anéis concêntricos finos (1 px, `--line`) atrás do hero, pulsando devagar a partir do ponto central.
- **Keycaps:** `Ctrl` `Shift` `Espaço` desenhados em HTML/CSS com Geist Mono, sombra interna e estado "pressionado" animado.
- **Moldura de janela:** capturas dentro de uma moldura com cantos de 12 px, borda `--line`, sombra longa e halo `--glow`.
- **Grão:** textura de ruído em SVG (gerada, < 2 KB) a 3–4% de opacidade para o preto não ficar chapado.

## 4. Seções

### 0. Barra de navegação (fixa)
Logo vermelho + "Aura" · links âncora: Recursos, Privacidade, Atalhos, Perguntas · botão "Baixar" pequeno. Transparente no topo; ao rolar, fundo `--bg` 70% com `backdrop-filter: blur(12px)` e borda inferior. No celular, só logo + "Baixar" (o app é para desktop; não precisa de menu hambúrguer).

### 1. Hero
- Eyebrow: `ASSISTENTE DE IA PARA WINDOWS · v0.3.1` (versão vinda da release).
- Título: **"Sua IA no Windows, a um atalho de distância."** (com "um atalho" em gradiente `--red-300 → --red-500`).
- Subtítulo: "Converse, pesquise na web e use o contexto da sua tela sem sair do que está fazendo. Entre com sua conta ChatGPT ou use seu próprio provedor."
- CTA primário: **Baixar para Windows** + linha abaixo: `Windows 10/11 · x64 · v0.3.1 · <tamanho> MB` (versão e tamanho lidos da release). CTA secundário: "Ver no GitHub" (texto + ícone).
- Visual (direita no desktop, abaixo no celular): **OverlayDemo**, a cena principal:
  1. Fundo da área de trabalho (IMG-01) desfocado.
  2. Keycaps `Ctrl` `Shift` `Espaço` "pressionam" em sequência (0,6 s).
  3. O Overlay compacto (captura recolorida) desce com mola suave (`translateY` + `opacity`, 450 ms).
  4. Ele se expande para a captura de conversa (crossfade + `clip-path`), e as linhas da resposta surgem uma a uma.
  5. Pausa de 4 s, repete. Para quando sai da tela e com `prefers-reduced-motion` mostra só o quadro final.
  Tudo é HTML + imagens; nada de vídeo no carregamento inicial. Se o vídeo opcional (IMG-09) existir, ele substitui os passos 3–4 com `preload="none"` e pôster.
- Fundo: anéis concêntricos + halo vermelho radial atrás do Overlay, movendo-se alguns pixels com o mouse (parallax leve, desligado em toque e com movimento reduzido).

### 2. "Como funciona" — 3 passos com rolagem fixa
Seção com o visual preso (`position: sticky`) e o texto rolando ao lado. Cada passo troca a captura com crossfade:
1. **Abra com um atalho.** Ctrl+Shift+Espaço, de qualquer aplicativo. → captura do Overlay sobre um app (IMG-03).
2. **Pergunte com contexto.** Digite `@região`, `@seleção` ou `@arquivo`; ou fale. → `contexto` (menu `@`).
3. **Leve a resposta de volta.** Copie, ouça ou insira direto no app de origem. → `conversa` com a barra de ações destacada (anel vermelho animado em "Inserir no app").
Indicador de progresso: três pontos verticais que preenchem em vermelho. No celular vira lista simples, sem sticky.

### 3. Recursos — grade bento
Grade de 6 colunas; cartões de tamanhos diferentes, cada um com recorte de captura, ícone e duas linhas de texto. Hover: leve elevação + brilho vermelho na borda que segue o cursor (CSS `radial-gradient` com variáveis `--x/--y`).

| Cartão | Tamanho | Imagem |
| --- | --- | --- |
| Pesquisa web com fontes clicáveis, sem chave paga | grande (4×2) | IMG-02e (nova captura) |
| Voz com transcrição local | médio (2×2) | IMG-02f (nova) ou `voice-secao-1` |
| Modos Chat, Tarefa e Plano | médio (3×1) | `tarefas` |
| Anexos: imagens, PDFs, planilhas, áudio | médio (3×1) | `arquivos` |
| Reuniões: preparo, transcrição, notas | médio (2×1) | `reuniao` / `preparo-painel` |
| Minibar: acompanha enquanto você trabalha | largo (4×1) | `minibar` |
| Extensões: Skills, MCP e comandos rápidos | pequeno (2×1) | `extensions-secao-1` |
| Histórico, memórias e perfis por app | pequeno (2×1) | `historico` |

Entrada: cartões aparecem com `opacity 0 → 1` e `translateY(16px) → 0`, escalonados em 60 ms.

### 4. Privacidade — "Você decide o contexto."
Fundo `--bg-elev`. À esquerda, 4 itens com ícone: dados no seu computador (`%LOCALAPPDATA%\Aura`) · credenciais protegidas pelo Windows (DPAPI) · janelas excluídas e retenção configurável · registro de cada acesso à tela. À direita, pilha de 3 capturas (`privacy-secao-*`, `29-registro-de-acesso`) que se abrem em leque ao entrar na tela. Frase de apoio: "Nada sai da sua máquina sem você pedir."

### 5. Modelos — "Use a IA que você já tem."
Três cartões lado a lado: **Conta ChatGPT** (login, sem chave) · **Sua chave** (OpenAI e compatíveis) · **Local** (Ollama/servidor local). Abaixo, a captura `providers` em moldura. Chips de texto com nomes de provedores, sem logos de marcas de terceiros.

### 6. Atalhos — teclado interativo
Linha de keycaps clicáveis/focáveis: ao passar ou focar, mostra a ação ("Ctrl+Shift+S → anexar a tela"). Lista vinda do README. Funciona por teclado (botões com `aria-describedby`).

### 7. Código aberto
Faixa fina: "Código aberto · MIT ou Apache-2.0 · Rust + Tauri". Botão "Ver código-fonte". Número de estrelas e da versão obtidos no build (sem chamada em tempo real).

### 8. Perguntas frequentes (`<details>`, sem JS)
- É gratuito? (o app sim; o modelo depende do seu plano/provedor)
- Preciso de conta ChatGPT? (não; pode usar provedor próprio ou local)
- **O Windows mostrou "O Windows protegeu o computador". É seguro?** (explicar SmartScreen, "Mais informações → Executar assim mesmo", verificação por SHA-256 e assinatura do updater)
- Requisitos (Windows 10/11 x64, WebView2, internet para modelos remotos)
- Como atualizo? (Configurações → Sobre)
- Como desinstalo? (Aplicativos do Windows; exporte dados antes)
- Funciona em macOS/Linux? (não)

### 9. Download final + rodapé
Bloco central grande com o logo, título "Pronto quando você precisar.", botão de download, tamanho, data da release, hash SHA-256 com botão copiar, links "Notas da versão", "Instalador MSI", "Todas as versões". Rodapé: licença, GitHub, Issues, Segurança, idioma.

## 5. Download da última release

`src/lib/release.ts`:

1. **No build:** `GET https://api.github.com/repos/VIDORETTO/aura-windows/releases/latest`. Escolhe o asset que casa com `/^Aura_.*_x64-setup\.exe$/` (e o `.msi` como secundário). Extrai versão, tamanho, data, URL direta (`browser_download_url`) e o SHA-256 (asset `.sha256`/`SHA256SUMS` se existir, senão o corpo da release). Grava no HTML.
2. **No cliente (opcional, ~1 KB):** ao carregar, consulta a mesma API uma vez (cache em `sessionStorage`). Se houver versão mais nova que a do build, atualiza botão e textos. Isso cobre o intervalo entre uma release e a próxima publicação do site.
3. **Fallback:** se a API falhar (limite de 60 req/h sem token, rede), o botão aponta para `https://github.com/VIDORETTO/aura-windows/releases/latest`.
4. Detecção de plataforma: em macOS/Linux/celular o botão diz "Disponível para Windows 10/11" e oferece "Enviar link para mim" (copiar link). Sem bloquear o download.

## 6. Motion — catálogo

| Nome | Onde | Técnica | Duração / curva |
| --- | --- | --- | --- |
| `reveal-up` | títulos, parágrafos, cartões | `animation-timeline: view()` + fallback IO | 600 ms, `cubic-bezier(0.2, 0, 0, 1)` |
| `stagger` | bento, listas | `--i` por item × 60 ms | — |
| `overlay-drop` | hero | keyframes `translateY(-24px) scale(.98) → 0` | 450 ms, mola via `linear()` |
| `keypress` | keycaps | `translateY(2px)` + sombra menor | 120 ms |
| `ring-pulse` | anéis do hero | `scale` + `opacity`, infinito | 6 s, `ease-in-out` |
| `glow-follow` | bordas do bento | variáveis CSS atualizadas no `pointermove` (rAF) | contínuo |
| `fan-out` | pilha de privacidade | `rotate` + `translate` por item | 700 ms |
| `sticky-swap` | "Como funciona" | crossfade por passo ativo | 400 ms |

Regras: nada anima `width/height/top/left`; `will-change` só durante a animação; com `prefers-reduced-motion: reduce` tudo vira estado final sem transição; animações em loop pausam fora da tela.

## 7. Orçamento de desempenho

| Métrica | Meta |
| --- | --- |
| JS total (gzip) | ≤ 12 KB |
| CSS total (gzip) | ≤ 20 KB |
| Fontes | 2 arquivos WOFF2 subset latin, `font-display: swap`, preload da display |
| Peso da primeira carga | ≤ 500 KB (imagens abaixo da dobra com `loading="lazy"`) |
| LCP | < 1,8 s em 4G simulado |
| CLS | < 0,05 (todas as imagens com dimensões) |
| INP | < 100 ms |
| Lighthouse | ≥ 95 em Performance, Acessibilidade, Boas práticas, SEO |

Imagens: AVIF com fallback WebP, `srcset` em 640/1024/1600/2400 px. A imagem do hero tem `fetchpriority="high"`.

## 8. Acessibilidade e SEO

- Contraste AA em tudo; foco visível com anel `--red-400` de 2 px.
- Navegação toda por teclado; `<details>` nativo; "Pular para o conteúdo".
- Todas as capturas com `alt` descritivo do que mostram.
- `lang="pt-BR"`, título e descrição próprios, Open Graph/Twitter com IMG-05, JSON-LD `SoftwareApplication` (sistema operacional, versão, preço 0, URL de download), `sitemap.xml`, `robots.txt`, URL canônica.
- Sem cookies e sem analytics na v1 (coerente com a mensagem de privacidade). Contagem de downloads vem da API do GitHub.

## 9. Publicação

- **GitHub Pages** com workflow `.github/workflows/landing.yml`: roda em push para `apps/landing/**` e no evento `release: published` (assim a página reconstrói com a versão nova automaticamente).
- Domínio: provisório `vidoretto.github.io/aura-windows`; domínio próprio a decidir.
- Checagens no CI: `astro check`, build, Lighthouse CI com os limites da seção 7, verificação de links.

## 10. Fases

| # | Entrega | Depende de |
| --- | --- | --- |
| L0 | Scaffold Astro, tokens, fontes, base CSS, layout e Nav | — |
| L1 | Logo e favicon vermelhos (IMG-04, IMG-06) | — |
| L2 | `release.ts` + componentes de download (hero, final, FAQ SmartScreen) | L0 |
| L3 | Recaptura das telas com acento vermelho (IMG-02) | — |
| L4 | Hero com OverlayDemo | L0, L1, L3, IMG-01 |
| L5 | Como funciona (sticky) + Bento | L3, IMG-02e/f, IMG-03 |
| L6 | Privacidade, Modelos, Atalhos, Código aberto, FAQ, rodapé | L3 |
| L7 | OG image, SEO, acessibilidade, orçamento de desempenho | L4–L6, IMG-05 |
| L8 | Workflow do Pages e publicação | L7 |
| L9 (opcional) | Versão em inglês `/en`; vídeo do hero (IMG-09) | L8 |

L1–L3 podem andar em paralelo com L0–L2. Na implementação, registrar como esforço `specs/043-landing-page` no fluxo do Hybrid Development Kit, um ticket por fase.

## 11. Melhorias sugeridas fora da landing

1. **Trocar o acento do app para o vermelho da marca.** O app usa um único token (`--accent: #5b5bf7` em `apps/desktop/src/styles/app.css`); o logo usa o gradiente roxo. Mudar os dois deixa site, ícone da bandeja e capturas coerentes. Tamanho pequeno, mas merece ticket próprio (afeta ícones `.ico`/`.png` do Tauri e as capturas do portfólio).
2. **Fundo escuro levemente mais quente no app** (`28 28 32` → `26 24 24`) para combinar com o vermelho. Opcional.
3. **Publicar o SHA-256 como asset (`SHA256SUMS.txt`) em toda release**, para a landing ler sem depender do texto das notas.

## 12. Decisões em aberto

| Decisão | Recomendação |
| --- | --- |
| Hospedagem e domínio | GitHub Pages agora; domínio próprio quando houver |
| Recolorir o app antes do site? | Sim, para as capturas da landing virem direto do app (ver 11.1). Se não, a recaptura usa só um override de CSS. |
| Versão em inglês na v1? | Não; preparar as chaves e lançar depois |
| Vídeo no hero | Não na v1; a demo em HTML é mais leve e nítida |

## 13. Estado da implementação (14/10/2026)

Todas as recomendações foram seguidas: o app ficou vermelho antes do site (043), GitHub Pages, sem inglês e sem vídeo na v1. O `sha256.txt` já era publicado em cada release; a página lê o `digest` da API e usa esse arquivo como reserva.

Desvios do plano:

- **Astro 7.3.8** em vez do 5 (versão atual na implementação). Mesmas APIs usadas.
- O revelar ao rolar usa só `IntersectionObserver`; não houve ganho em manter também as scroll-driven animations do CSS.
- Lighthouse CI virou `pnpm budget` (tamanho de JS, CSS, HTML e fontes) no workflow. O Lighthouse foi rodado localmente.
- O título do hero não começa transparente: ele é o elemento de LCP.

Resultado medido localmente (Lighthouse 12.8.2 no Edge, `astro preview`):

| | Desempenho | Acessibilidade | Boas práticas | SEO | LCP | CLS |
| --- | --- | --- | --- | --- | --- | --- |
| Desktop | 100 | 100 | 100 | 100 | 0,6 s | 0 |
| Mobile (4G simulado) | 98 | 100 | 100 | 100 | 2,2 s | 0,019 |

O LCP no mobile fica acima da meta de 1,8 s por causa da troca para a fonte Geist no título. `pnpm budget`: JS 2,1 KB, CSS 7,0 KB, HTML 16,6 KB (gzip).

Comandos: `pnpm -C apps/landing dev | build | preview | test | check | budget | capture | art`.

Publicação: o GitHub Actions foi ativado só para o workflow `Landing` (decisão do usuário em 14/10/2026); CI, Performance e Release ficam desativados individualmente. O Pages usa a fonte "GitHub Actions" e o ambiente `github-pages` aceita a `main` e as tags `v*`, para que uma release publicada reconstrua a página.

Pendente: as imagens IMG-01, IMG-03 e IMG-07 ([IMAGENS.md](IMAGENS.md)).
