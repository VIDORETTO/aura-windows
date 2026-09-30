# Linguagem de interface do Aura

Objetivo: parecer parte do Windows 11, desaparecer quando não é necessário e nunca atrasar o usuário. Referências de qualidade: Raycast, Arc, barra de pesquisa do Windows 11, Linear.

## Princípios

1. **Teclado primeiro.** Tudo que importa é alcançável sem mouse; o mouse é complemento.
2. **Transparente sobre contexto.** Todo dado que será enviado aparece como Chip de contexto antes do envio.
3. **Movimento com propósito.** Animações curtas (120–200 ms), curvas `cubic-bezier(0.2, 0, 0, 1)`; nada de animação bloqueando digitação. Respeita "reduzir movimento" do Windows.
4. **Silencioso por padrão.** Sem sons, sem pop-ups; indicadores discretos para gravação e trabalho em andamento.
5. **Denso, legível.** Tipografia Segoe UI Variable, 14 px base, altura de linha 1.5; Cascadia Code para código.

## Tokens (CSS custom properties)

| Token | Claro | Escuro | Uso |
| --- | --- | --- | --- |
| `--surface` | `rgba(249,249,251,0.72)` | `rgba(28,28,32,0.68)` | Fundo do Overlay sobre Acrylic |
| `--surface-strong` | `rgba(255,255,255,0.92)` | `rgba(38,38,44,0.9)` | Cartões, menus |
| `--border` | `rgba(0,0,0,0.08)` | `rgba(255,255,255,0.08)` | Bordas 1 px |
| `--text` / `--text-muted` | `#1b1b1f` / `#5d5d66` | `#f2f2f5` / `#a0a0aa` | Texto |
| `--accent` | cor de destaque do Windows (lida do sistema) | idem | Foco, ações primárias |
| `--danger` / `--warning` / `--success` | `#c42b1c` / `#9d5d00` / `#0f7b0f` | `#ff99a4` / `#fce100` / `#6ccb5f` | Estados |
| `--radius-lg` / `--radius-md` | 14 px / 8 px | idem | Overlay / controles |
| `--shadow-overlay` | `0 16px 48px rgba(0,0,0,.18)` | `0 16px 48px rgba(0,0,0,.45)` | Elevação |

A opacidade do Overlay é ajustável (70–100%) multiplicando o alfa de `--surface`. Com "alto contraste" do Windows ativo, transparência é desligada e as cores do sistema são usadas.

## Overlay

```
Compacto (padrão ao abrir, 640 × 64)
╭──────────────────────────────────────────────────────────────╮
│ ◎  Pergunte algo… (/ comandos · @ contexto)      🎙  ⌘  ▸    │
│ [🖥 Tela · Chrome — Relatório.pdf ✕] [❝ seleção ✕]            │
╰──────────────────────────────────────────────────────────────╯

Expandido (ao enviar ou Ctrl+↓, 640 × até 70% da altura do monitor)
╭──────────────────────────────────────────────────────────────╮
│ Aura · Modelo ▾  Chat ▾   ● Usando plano ChatGPT · Gerenciar ─ ✕ │
│──────────────────────────────────────────────────────────────│
│  Você: o que significa esse erro?  [🖥 miniatura]             │
│  Aura: ▍streaming em markdown…                                │
│   ├ 🔧 aura.screen_text (Chrome) ✓ 120 ms                     │
│   └ ⚠ Aprovação: executar `pip install x`  [Aceitar][Recusar] │
│──────────────────────────────────────────────────────────────│
│ ◎  Responder…                                    🎙  ⌘  ▸    │
╰──────────────────────────────────────────────────────────────╯
```

- Aparece centralizado no terço superior do monitor do Aplicativo anterior; lembra posição e tamanho por monitor se o usuário mover/redimensionar.
- Sempre no topo, fora da taskbar e do Alt+Tab, invisível em capturas.
- Arrastável pela área vazia do cabeçalho; redimensionável pelas bordas no estado expandido.
- Ao perder foco sem resposta em andamento: esconde (configurável: "manter aberto").
- Ao perder foco com resposta em andamento: vira Minibar.

## Mapa de teclado

| Tecla | Ação |
| --- | --- |
| Atalho de invocação | Mostrar/esconder |
| `Esc` | Fecha menu aberto → cancela gravação → esconde Overlay |
| `Enter` / `Shift+Enter` | Enviar / nova linha |
| `Ctrl+Enter` durante resposta | Enviar como direcionamento (steer) |
| `Ctrl+.` | Interromper resposta |
| `/` no início | Comandos rápidos e Skills |
| `@` | Contexto: `@tela`, `@região`, `@áudio`, `@seleção`, `@arquivo` |
| `Ctrl+Shift+S` (no Overlay) | Capturar tela para Chip |
| Segurar `Ctrl+Space` (no Overlay) | Push-to-talk |
| `Ctrl+N` / `Ctrl+H` / `Ctrl+,` | Nova Conversa / histórico / configurações |
| `Ctrl+↑` / `Ctrl+↓` | Compactar / expandir |
| `Ctrl+Shift+E` | Nova Conversa efêmera |
| `Ctrl+C` na última resposta | Copiar resposta |

## Componentes-chave

- **Barra de entrada**: textarea auto-expansível (até 8 linhas), Chips de contexto, botões de microfone, contexto (`@`) e enviar; mostra atalho de dica ao focar.
- **Chip de contexto**: ícone da fonte, rótulo curto, miniatura no hover, `✕` para remover; chips gerados pela Política mostram cadeado quando algo foi bloqueado.
- **Mensagem do agente**: markdown incremental, código com destaque e botão copiar, tabelas roláveis, links abrem no navegador.
- **Cartão de Item**: ferramentas, comandos, buscas e raciocínio aparecem colapsados com status e duração; expandir mostra argumentos/saída.
- **Cartão de Aprovação**: ação, motivo, alvo, botões `Aceitar`, `Aceitar nesta conversa`, `Recusar`; foco automático e atalhos `A`/`R`.
- **Indicadores de captura**: ponto no ícone da bandeja e no cabeçalho do Overlay por Fonte ativa (vermelho = gravando, âmbar = buffer recente, cinza = sob demanda).
- **Indicador de uso do plano**: "Usando plano ChatGPT" + "Gerenciar uso" (abre `chatgpt.com/settings/usage`), conforme diretrizes do Sign in with ChatGPT; com BYOK mostra o Provedor.
- **Marca ChatGPT**: botão "Continue with ChatGPT" e ícones apenas nos formatos aprovados pela OpenAI.

## Estados vazios e erros

- Sem login: cartão único com "Continue with ChatGPT" e "Usar minha chave".
- Limite do plano atingido: modal/cartão compacto com identidade ChatGPT, "Gerenciar uso" como ação principal e "Usar outro provedor" como secundária.
- App-server indisponível: estado "reconectando" com tentativa automática e botão "Ver diagnóstico".
- Captura bloqueada pela Política: Chip com cadeado e texto "bloqueado: janela excluída".

## Acessibilidade

- Contraste mínimo AA sobre o fundo translúcido medido no pior caso (wallpaper claro/escuro).
- Todos os controles com nome acessível; regiões com `aria-live="polite"` para a resposta em streaming (anunciada por frase, não por token).
- Foco visível com `--accent`; ordem de tabulação lógica; nenhuma ação apenas por hover.
