---
schema: hybrid/spec
schema_version: "1.0"
effort_id: 004-contexto-de-tela
revision: 1
status: accepted
profile: standard
---

# Specification: Contexto de tela e privacidade

## Problem and desired result

A IA só ajuda de verdade se enxergar o que o usuário está vendo — agora e, às vezes, alguns minutos atrás. Ao mesmo tempo, a tela contém senhas, conversas e dados de trabalho. O resultado é o usuário escolher, por Fonte, como a tela é capturada (Desligado, Sob demanda, Buffer recente, Manual, Contínuo), decidir se o agente pode olhar por conta própria (Nunca, Perguntar, Sempre), anexar a tela ou uma região com um gesto, e ter garantia de que janelas excluídas e a Pausa de privacidade são respeitadas no momento da captura, com Registro de acesso visível.

## Consumers and actors

- Usuário no Overlay e nas Configurações.
- Agente (via Ferramentas do Aura no servidor MCP local).
- Esforços 005 (reusa o motor de Política e o armazenamento segmentado), 007 (anexar Recortes), 009 (perfis por app), 010 (desempenho).

## Scope

### Included

- Captura instantânea do monitor do Aplicativo anterior ou da janela ativa como Chip (`/screen`, botão, `@tela`, atalho).
- Seleção de região com tela congelada.
- Motor de Política de privacidade: Modos por Fonte, Permissão do agente, Janelas excluídas (lista padrão + do usuário), Pausa de privacidade, indicadores, Registro de acesso.
- Servidor MCP do Aura com ferramentas de tela: captura atual, janela ativa, texto da tela (UI Automation + OCR), frames recentes.
- Buffer recente de tela (últimos N minutos), Gravação manual, modo Contínuo com Retenção, e anexar "últimos X minutos" como Recorte.
- Opção "anexar tela ao abrir o Overlay".

### Excluded

- Timeline pesquisável (CAND-015), anotações na tela (CAND-014), controle do computador (CAND-013).
- Áudio (005).

## User journeys and scenarios

### US-001 — Perguntar sobre a tela atual (Priority: P1)

Independent demonstration: com um erro na tela, abrir o Overlay, `/screen`, perguntar "o que é esse erro?".

#### Acceptance scenarios

- **AC-001** — Dado o Overlay aberto, quando o usuário digita `/screen`, clica no botão de tela ou pressiona `Ctrl+Shift+S`, então em até 300 ms surge um Chip "Tela · <app> — <título>" com miniatura do monitor do Aplicativo anterior, e o Overlay não aparece na imagem.
- **AC-002** — Dado o Chip de tela, quando o usuário alterna para "Janela ativa", então a captura passa a conter apenas a janela do Aplicativo anterior (sem sobreposições de outras janelas).
- **AC-003** — Dado "Anexar tela ao abrir" ativado, quando o Overlay é aberto, então o Chip de tela já está presente (removível) — exceto se o Aplicativo anterior for Janela excluída ou a Pausa estiver ativa, caso em que aparece um Chip bloqueado explicando o motivo.

### US-002 — Selecionar uma região (Priority: P2)

#### Acceptance scenarios

- **AC-004** — Dado o Overlay aberto, quando o usuário escolhe `@região`, então a tela congela com escurecimento, ele arrasta um retângulo (com dimensões exibidas) e ao soltar surge um Chip da região; `Esc` cancela sem Chip; funciona no monitor onde o cursor está, com DPI correto.

### US-003 — Controlar privacidade (Priority: P1)

#### Acceptance scenarios

- **AC-005** — Dado a lista padrão de Janelas excluídas (gerenciadores de senha conhecidos, janelas "InPrivate"/"Incognito", Windows Security, credenciais do Windows) e regras do usuário (processo, título com curinga), quando qualquer captura é pedida com uma Janela excluída visível, então a área dela é coberta por um bloco opaco com cadeado na imagem; se a Janela excluída ocupar o monitor inteiro ou for o alvo, a captura é bloqueada.
- **AC-006** — Dado a Pausa de privacidade ativada (bandeja ou atalho `Ctrl+Shift+Alt+P`), quando qualquer captura (usuário ou agente) é pedida ou um buffer está ativo, então nada é capturado nem gravado até retomar, e o ícone da bandeja indica pausa.
- **AC-007** — Dado a Permissão do agente para Tela = Nunca, Perguntar ou Sempre, quando o agente chama uma ferramenta de tela, então é negado com motivo, aparece cartão "Permitir que o Aura veja sua tela agora?" (Permitir uma vez / nesta Conversa / Negar), ou é permitido — nesta ordem; o padrão é Perguntar.
- **AC-008** — Dado qualquer captura entregue ao agente ou anexada num turno, quando o usuário abre "Registro de acesso", então vê data/hora, Fonte, quem pediu (usuário/agente + ferramenta), Conversa e decisão (permitido/negado/coberto), com miniatura enquanto o arquivo existir.
- **AC-009** — Dado Buffer recente, Manual ou Contínuo ativos para Tela, quando o usuário olha a bandeja ou o cabeçalho do Overlay, então vê o indicador da Fonte (âmbar para Buffer recente, vermelho para gravação) e pode parar com um clique.

### US-004 — O agente olha a tela quando precisa (Priority: P1)

#### Acceptance scenarios

- **AC-010** — Dado Permissão do agente = Sempre (ou concedida nesta Conversa), quando o usuário pergunta "o que está aberto na minha tela?" sem anexar nada, então o agente chama a ferramenta de captura, recebe a imagem e responde com base nela; o Item da ferramenta aparece colapsado na Conversa.
- **AC-011** — Dado uma janela com texto acessível (navegador, Word, VS Code), quando o agente chama "texto da tela", então recebe o texto da janela ativa via UI Automation; se não houver texto acessível, recebe texto por OCR do Windows, indicando a origem.
- **AC-012** — Dado Buffer recente de Tela ativo, quando o agente chama "frames recentes" com um intervalo (ex.: últimos 2 min, até 8 imagens), então recebe keyframes amostrados uniformemente desse intervalo, respeitando Janelas excluídas e Permissão.

### US-005 — Voltar no tempo (Priority: P2)

#### Acceptance scenarios

- **AC-013** — Dado Modo Buffer recente com N = 5 min (configurável 1–30), quando o Aura roda por 20 min, então existem no disco apenas os Segmentos dos últimos ~5 min (+ o Segmento em curso), cifrados, e nada além disso.
- **AC-014** — Dado Buffer recente ativo, quando o usuário escolhe "Anexar últimos 2 min" (menu `@tela`), então surge um Chip de Recorte com duração e miniatura; ao enviar, o turno recebe até 8 keyframes representativos e o Recorte é copiado para o Workspace da conversa.
- **AC-015** — Dado Modo Manual, quando o usuário clica em Gravar e depois Parar, então existe uma Gravação com duração correta listada em "Gravações", reproduzível no player do Aura, anexável a Conversas e excluível.
- **AC-016** — Dado Modo Contínuo com Retenção de 7 dias e 20 GB, quando qualquer limite é excedido, então as Gravações/Segmentos mais antigos são apagados até voltar ao limite; a Retenção é aplicada também na inicialização.

## Requirements

- **FR-001** — O sistema MUST capturar monitor ou janela do Aplicativo anterior sob demanda como Chip, excluindo janelas do Aura.
- **FR-002** — O sistema MUST permitir seleção de região multi-monitor com DPI correto.
- **FR-003** — O sistema MUST aplicar, no momento da captura, uma Política de privacidade única que combina Modos, Permissões do agente, Janelas excluídas e Pausa, e MUST registrar todo acesso.
- **FR-004** — O sistema MUST expor ao agente Ferramentas do Aura de tela por MCP local, sujeitas à Política.
- **FR-005** — O sistema MUST oferecer Buffer recente, Gravação manual e Contínuo para Tela com Segmentos cifrados e Retenção.
- **FR-006** — O sistema MUST permitir anexar capturas e Recortes a Turnos com amostragem de keyframes.
- **FR-007** — O sistema MUST mostrar indicadores de Fontes ativas e permitir parar/pausar com um clique.

## Limits, errors, and compatibility

- Windows.Graphics.Capture exige Windows 10 1903+ (coberto pelo mínimo 2004). Captura sem borda amarela quando suportado (`IsBorderRequired=false`, Windows 11); no Windows 10 a borda amarela pode aparecer durante buffers — documentado.
- Conteúdo protegido por DRM aparece preto; não é erro.
- Buffer a 1 fps padrão (0,5–2 fps configurável), resolução máx. 1920 px no maior lado para o buffer; capturas instantâneas em resolução nativa (reduzidas a 2048 px ao enviar ao modelo).
- Espaço em disco < 2 GB livre → buffers/gravações param com notificação.
- Captura de janela minimizada: erro "janela minimizada".

## Hypotheses and dependencies

- H-005: 1 fps H.264 por hardware ≤ 3% CPU e ≤ 300 MB/h. Check: TK-005.
- H-009: `WDA_EXCLUDEFROMCAPTURE` exclui janelas do Aura da WGC. Check: 001 TK-002 e TK-001 aqui.
- H-003: imagens de resultado MCP chegam ao modelo. Check: 002 TK-001; se falso, TK-004 usa `dynamicTools`.
- Dependências: 001 (janelas, `ForegroundTracker`, cofre), 002 (Chips `ContextTray`, `ConfigContributor`, eventos de ferramenta).

## Success criteria

### Delivery-verifiable

- **SC-001** — AC-001–AC-016 com evidência; teste automatizado de Janela excluída cobrindo KeePassXC e Bitwarden (processos reais ou dublês com mesmo nome de processo).
- **SC-002** — Benchmark do buffer (CPU/disco) na máquina de referência anexado.

### Post-delivery observation

- **SC-003** — Nenhum relato de captura de Janela excluída no beta; ≥ 60% dos usuários do beta usam `/screen` pelo menos uma vez por semana.

## Decisions and open questions

- Padrões: Tela em Modo Sob demanda; Permissão do agente = Perguntar; "Anexar tela ao abrir" desligado.
- Buffer/Gravação usam o mesmo formato segmentado (ADR 0004).
- Nenhuma pergunta de produto pendente.
