---
schema: hybrid/change
schema_version: 1.0
effort_id: 012-selecao-e-cor-de-destaque
revision: 1
status: closed
profile: compact
---

# Change: Seleção ao voltar ao Overlay e cor de destaque

## Objetivo e limites

Pedido do usuário em 04/10/2026:

1. A seleção de texto do aplicativo anterior (usada por `@seleção` e comandos rápidos como reescrever/traduzir) só era lida quando o Overlay **abria**. Com o Overlay aberto (ele não some ao perder o foco), selecionar texto em outro app e voltar não trazia a seleção; era preciso esconder e mostrar o Overlay.
2. Escolher a cor de destaque do app: cores prontas e uma personalizada (seletor RGB e código hex). Tudo que hoje usa o roxo/azul padrão passa a usar a cor escolhida.

Fora do escopo: cópia sintética com Ctrl+C (efeitos colaterais em terminais e editores que copiam a linha inteira sem seleção), ícone da bandeja/executável (imagens nativas fixas), cores semânticas (erro, aviso, sucesso) e cores de strings/números no realce de código.

## Contrato de comportamento

- Entradas: Overlay visível e sem foco; o usuário seleciona texto em outro app e volta ao Overlay. Configurações › Geral › Cor de destaque.
- Saída: ao voltar o foco ao Overlay, a seleção atual do app anterior vira (ou atualiza) o Chip de seleção, sem esconder/mostrar. A cor escolhida é aplicada a todas as janelas imediatamente e persiste.
- Erros/invariantes: um único Chip de seleção por bandeja (nova seleção substitui; mesma seleção não duplica; seleção removida pelo usuário não volta sozinha enquanto o texto for o mesmo, mas volta por `@seleção`). Pausa e exclusões continuam valendo. Hex inválido é recusado (`#RRGGBB`). Texto sobre a cor usa preto ou branco conforme o contraste.
- Compatibilidade: abrir pelo atalho continua capturando a seleção; alto contraste do Windows continua usando `Highlight`; sem cor escolhida o destaque é o `#5b5bf7` atual.

## Requisitos e aceite

- **FR-001** — O Overlay MUST atualizar a seleção do aplicativo anterior quando recupera o foco, sem precisar ser escondido e mostrado.
- **AC-001** — Dado o Overlay visível e um app com o texto "Aura QA seleção" selecionado, quando o foco volta ao Overlay, então surge um Chip "❝ Aura QA seleção"; ao selecionar outro texto e voltar, o mesmo Chip passa a ter o novo texto (um único Chip).
- **FR-002** — O usuário MUST escolher a cor de destaque entre cores prontas ou uma personalizada (seletor RGB e hex), aplicada a toda a interface.
- **AC-002** — Dado Configurações › Geral, quando o usuário escolhe a cor personalizada `#e4572e`, então os elementos de destaque (botão primário, foco, indicadores, logo) usam `rgb(228, 87, 46)` em Overlay e Configurações, e a cor permanece após reiniciar; "Padrão" volta ao `#5b5bf7`.

## Leitura e mapa de alterações

- `crates/aura-win/src/uia.rs` → `focused_selection` — procurar TextPattern nos ancestrais do elemento focado; existing.
- `apps/desktop/src-tauri/src/win_platform.rs` → `WinForeground::snapshot/selection` — não ler a seleção quando o Aura está em primeiro plano; não consumir a seleção; existing.
- `apps/desktop/src-tauri/src/overlay.rs` / `main.rs` → acompanhar o app anterior enquanto o Overlay está visível sem foco; existing.
- `crates/aura-app/src/host.rs` → `capture_selection` — Chip único, deduplicado, `explicit`; existing.
- `crates/aura-core/src/settings.rs` → `accent_color`; existing.
- `apps/desktop/src/state/app.ts` → `applyTheme`; `apps/desktop/src/styles/app.css`; `apps/desktop/src/settings/General.tsx`; logo inline; existing/new.

## Plano breve

Seam: `Host::capture_selection` e `Settings::apply` (Rust), `applyTheme`/Configurações (React), app nativo com janela QA WinForms. Abordagem: rastreio leve por UI Automation a cada 300 ms só enquanto o Overlay está visível e sem foco; a cor vira `--accent` e `--accent-contrast` no `:root`. Dependências: none.

## Sequência e tarefas

- [x] C-001 Red/green Host para Chip de seleção único e `explicit`.
- [x] C-002 Rastreio nativo e captura ao recuperar o foco; red/green nativo.
- [x] C-003 Red/green de `accentColor` (Rust, React) e aplicação nativa.
- [x] C-004 Regressão e evidência.

## Validação e evidência

Comando/procedimento: `cargo test -p aura-app --test host`, `cargo test -p aura-core`, `pnpm -C apps/desktop test`, `pnpm -C apps/desktop/e2e test --spec ./specs/selection-refocus.e2e.ts` e `./specs/accent-color.e2e.ts`.

Resultado executado: EV-001 (AC-001) e EV-002 (AC-002) passaram no release de produção SHA256 `191860544E94A3AB7B22280323DAE7F9F42AC01A28AB59F58998459C66EF73DA`; regressão: workspace Rust, clippy, fmt, contrato dourado, UI 142/142, typecheck, `answer-shortcuts` e `profile-model` nativos.

Limitações: apps sem UI Automation TextPattern continuam sem seleção (sem Ctrl+C sintético).

## Condição de retorno

Retornar se o contrato, símbolo, dependência ou decisão material não puder ser satisfeito sem ampliar o escopo.

## Estado

`change.md` é a fonte canônica deste esforço compacto. Não crie `tasks.md` ou tickets paralelos para esta mudança.
