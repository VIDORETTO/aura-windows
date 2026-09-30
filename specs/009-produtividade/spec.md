---
schema: hybrid/spec
schema_version: "1.0"
effort_id: 009-produtividade
revision: 1
status: accepted
profile: standard
---

# Specification: Produtividade no desktop

## Problem and desired result

O Overlay precisa encaixar no fluxo de trabalho: trazer o texto selecionado sem copiar/colar, não sumir quando o usuário troca de janela durante uma resposta, avisar quando termina, ler respostas em voz alta, adaptar-se ao aplicativo em uso e devolver a resposta ao lugar certo. O resultado são cinco comportamentos que eliminam as idas e vindas entre o Aura e o Aplicativo anterior.

## Consumers and actors

- Usuário no Overlay e no Aplicativo anterior.

## Scope

### Included

- Texto selecionado no Aplicativo anterior vira Chip de citação ao abrir o Overlay.
- Minibar quando o Overlay perde o foco com resposta em andamento; notificação ao concluir.
- Leitura em voz alta (TTS) das respostas.
- Perfis de aplicativo (instruções e contexto padrão por processo).
- Inserir/colar a resposta no Aplicativo anterior.

### Excluded

- Ditado em qualquer app (CAND-011); controle do computador (CAND-013).

## User journeys and scenarios

### US-001 — Perguntar sobre a seleção (Priority: P1)

#### Acceptance scenarios

- **AC-001** — Dado texto selecionado no Aplicativo anterior (navegador, Word, VS Code, Bloco de Notas), quando o usuário abre o Overlay, então aparece um Chip "Seleção · N caracteres" com o texto (prévia no hover) e a área de transferência do usuário permanece inalterada após a captura.
- **AC-002** — Dado nenhuma seleção, quando o Overlay abre, então nenhum Chip de seleção aparece e nada é colado ou perdido; dado seleção num Aplicativo anterior que é Janela excluída, então nada é capturado.

### US-002 — Não perder a resposta de vista (Priority: P2)

#### Acceptance scenarios

- **AC-003** — Dado uma resposta em andamento, quando o usuário clica em outra janela, então o Overlay vira a Minibar (faixa de ~40 px no topo do monitor, sempre no topo, invisível em capturas) com status e prévia da última linha; clicar na Minibar restaura o Overlay.
- **AC-004** — Dado a Minibar ou o Overlay oculto, quando a resposta termina ou uma Aprovação é pedida, então uma notificação nativa do Windows aparece (respeitando o modo Foco/Não perturbe) e clicar nela abre o Overlay na Conversa.

### US-003 — Ouvir a resposta (Priority: P2)

#### Acceptance scenarios

- **AC-005** — Dado uma resposta concluída, quando o usuário clica no alto-falante (ou `Ctrl+Shift+L`), então a resposta é lida com a voz escolhida (vozes do Windows, offline, por padrão), sem ler blocos de código nem URLs completas; clicar de novo para; "Ler respostas automaticamente" opcional.
- **AC-006** — Dado um Provedor BYOK com TTS (ex.: OpenAI) escolhido como voz, quando usado pela primeira vez, então o Aura informa que o texto será enviado ao provedor e pede confirmação única.

### US-004 — O Aura se adapta ao app (Priority: P2)

#### Acceptance scenarios

- **AC-007** — Dado um Perfil de aplicativo para `code.exe` com instruções "Responda com código TypeScript" e "Anexar tela ao abrir: sim", quando o usuário abre o Overlay com o VS Code em primeiro plano, então a Conversa nova usa essas instruções (indicadas por um selo "Perfil: VS Code") e o Chip de tela já aparece; em outro app, não.
- **AC-008** — Dado o seletor de perfis, quando o usuário cria um perfil a partir do Aplicativo anterior atual ("Criar perfil para este app"), então o processo é preenchido automaticamente.

### US-005 — Devolver a resposta (Priority: P1)

#### Acceptance scenarios

- **AC-009** — Dado uma resposta concluída, quando o usuário clica "Inserir no app" (ou `Ctrl+Shift+Enter`), então o Overlay se esconde, o foco volta ao Aplicativo anterior e o texto da resposta (ou do bloco de código selecionado) é inserido na posição do cursor; a área de transferência original é restaurada depois.
- **AC-010** — Dado que o Aplicativo anterior foi fechado ou é elevado (administrador), quando o usuário escolhe "Inserir no app", então o texto é copiado para a área de transferência e uma mensagem explica o motivo.

## Requirements

- **FR-001** — O sistema MUST capturar a seleção do Aplicativo anterior como Chip preservando a área de transferência e respeitando Janelas excluídas.
- **FR-002** — O sistema MUST mostrar a Minibar durante respostas fora de foco e notificar conclusões e Aprovações.
- **FR-003** — O sistema MUST ler respostas em voz alta com vozes locais por padrão e provedores opcionais com consentimento.
- **FR-004** — O sistema MUST aplicar Perfis de aplicativo a Conversas novas conforme o Aplicativo anterior.
- **FR-005** — O sistema MUST inserir respostas no Aplicativo anterior com fallback para a área de transferência.

## Limits, errors, and compatibility

- Captura de seleção por UI Automation (`TextPattern.GetSelection`) primeiro; fallback por cópia simulada (`Ctrl+C`) com salvar/restaurar da área de transferência (todos os formatos) em ≤ 300 ms; apps que bloqueiam cópia ficam sem Chip.
- Inserção por colagem simulada (`Ctrl+V`) com restauração; campos que bloqueiam colagem → digitação simulada (`SendInput` unicode) para até 2 000 caracteres.
- Minibar e Overlay obedecem `WDA_EXCLUDEFROMCAPTURE`.

## Hypotheses and dependencies

- H-018: UI Automation retorna a seleção em ≥ 80% dos apps comuns (Chrome/Edge, Office, VS Code, Bloco de Notas). Check: TK-001.
- Dependências: 001 (`ForegroundTracker`, janelas), 002 (Conversas, Chips, eventos), 004 (Política/exclusões), 008 TK-006 (composição de instruções).

## Success criteria

### Delivery-verifiable

- **SC-001** — AC-001–AC-010 com evidência; matriz de apps testados para seleção e inserção.

### Post-delivery observation

- **SC-002** — ≥ 30% das Conversas do beta começam com Chip de seleção ou terminam com "Inserir no app".

## Decisions and open questions

- Voz padrão: Windows (`Windows.Media.SpeechSynthesis`), offline; Edge TTS não é padrão (envia texto a serviço externo).
- Atalho "Inserir no app": `Ctrl+Shift+Enter`.
