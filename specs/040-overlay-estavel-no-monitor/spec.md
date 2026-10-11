---
schema: hybrid/spec
schema_version: 1.0
effort_id: 040-overlay-estavel-no-monitor
revision: 1
status: accepted
profile: standard
---

# Manter o Overlay no monitor durante a conversa

## Problem and desired result

O usuário relata mudanças involuntárias de monitor durante a conversa. O Overlay visível deve permanecer onde o usuário o colocou quando muda de tamanho ou modo. Pedido explícito autoriza corrigir esse defeito.

## Consumers and actors

Usuário de Windows com múltiplos monitores; consumidor técnico: shell e UI do Overlay.

## Scope

### Included

- Alternância compacto/expandido/Minibar e memória de posição por monitor.

### Excluded

- Alteração de IPC, banco, atalhos, privacidade ou provedor; instalação/publicação.

## User journeys and scenarios

### US-001 — Conversar sem troca involuntária de monitor (P1)

Ao conversar, o usuário conserva o monitor e a âncora de posição que escolheu.

Demonstração independente: arrastar de A para B, manter Aplicativo anterior em A, alternar modos e observar posição real.

#### Acceptance scenarios

- **AC-001** — Overlay aberto em A, arrastado para B, com Aplicativo anterior ainda em A: expandir, compactar, voltar da Minibar ou iniciar conversa permanece em B. Borda superior/posição horizontal permanecem iguais quando a área útil comporta o tamanho, inclusive com tamanho salvo do outro modo em B.
- **AC-002** — Salvar após arrastar para B guarda posição para B, não sobrescreve A. Reabrir em B recupera sua largura/posição; reabrir em A conserva sua posição anterior.
- **AC-003** — Seleção por maior interseção física funciona com origem negativa e DPI distintos. Nenhuma interseção recupera posição pela regra existente. Tamanho fica na área útil e respeita mínimos; nenhum monitor retorna ausência, sem pânico.

## Requirements

- **FR-001** — Mudanças de modo usam o monitor atual do Overlay, sem seguir Aplicativo anterior de outro monitor.
- **FR-002** — Posição/tamanho são lembrados sob o monitor que contém a janela.
- **FR-003** — Preservar abertura contextual de Overlay oculto, DPI, mínimos e recuperação de monitor desconectado.

## Limits, errors, and compatibility

Coordenadas físicas. Sem alterar formato persistido ou IDs. Abertura oculta continua contextual; a janela é limitada à área útil disponível se o monitor foi desconectado. Não mover por novo conteúdo textual.

## Hypotheses and dependencies

- Hipótese: set_mode/remember_placement usam target_monitor pelo Aplicativo anterior. UI envia mudanças ao iniciar conversa/voltar da Minibar. Confirmar com red nativo antes da correção.
- Dependência: dois monitores, MSVC/WebView2/WebDriver existentes; nenhum serviço externo novo.

## Success criteria

### Delivery-verifiable

- **SC-001** — Regressão de dois monitores e testes puros passam; binário de produção compila.

### Post-delivery observation

- **SC-002** — Recorrência no uso cotidiano permanece observação posterior, sem afirmar prova de todas as causas possíveis.

## Decisions and open questions

Preservar âncora da janela nas transições. Nenhuma decisão material pendente; ajuste técnico reversível no plano.
