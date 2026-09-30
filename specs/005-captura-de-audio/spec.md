---
schema: hybrid/spec
schema_version: "1.0"
effort_id: 005-captura-de-audio
revision: 1
status: accepted
profile: standard
---

# Specification: Captura de áudio

## Problem and desired result

Muitas perguntas dependem do que foi dito ou ouvido há pouco: "o que ele acabou de falar na call?", "resuma os últimos 5 minutos desse vídeo". O resultado é capturar Microfone e Áudio do sistema como Fontes independentes, com os mesmos Modos da tela (Desligado, Sob demanda, Buffer recente, Manual, Contínuo), sob a mesma Política de privacidade, podendo anexar recortes transcritos à Conversa ou deixar o agente consultá-los com Permissão.

## Consumers and actors

- Usuário (configurações, indicadores, anexos).
- Agente (ferramenta de áudio recente).
- Esforço 006 (push-to-talk e transcrição consomem a captura de microfone e o formato de segmentos).

## Scope

### Included

- Captura de Microfone (dispositivo escolhido ou padrão) e Áudio do sistema (loopback do dispositivo de saída padrão), com medidor de nível.
- Modos por Fonte; Buffer recente com N minutos; Gravação manual e Contínuo com Retenção.
- Salvar/anexar "últimos X minutos" de uma ou ambas as Fontes (mixadas ou separadas).
- Ferramenta do Aura `audio_recent` (transcrição) sujeita à Permissão do agente.
- Indicadores e Pausa (reuso do 004).

### Excluded

- Transcrição em si (006) — este esforço entrega áudio e chama a Interface de transcrição.
- Diarização/identificação de falantes e assistente de reuniões (CAND-016).
- Captura por aplicativo específico (process loopback) — futuro.

## User journeys and scenarios

### US-001 — Escolher fontes e ver que estão funcionando (Priority: P1)

#### Acceptance scenarios

- **AC-001** — Dado as Configurações de áudio, quando o usuário escolhe o microfone e liga o teste, então um medidor mostra o nível em tempo real (atualização ≥ 20 Hz); o mesmo para Áudio do sistema tocando um som.
- **AC-002** — Dado o dispositivo escolhido desconectado, quando uma captura é pedida, então o Aura usa o dispositivo padrão atual e avisa; ao reconectar, volta ao escolhido.

### US-002 — Buffer recente e gravações de áudio (Priority: P1)

#### Acceptance scenarios

- **AC-003** — Dado Microfone em Buffer recente com N = 5 min, quando o Aura roda por 20 min, então só existem Segmentos cifrados dos últimos ~5 min (+ corrente) daquela Fonte.
- **AC-004** — Dado Áudio do sistema em Manual, quando o usuário grava 60 s e para, então há uma Gravação de 60 ± 1 s reproduzível no Aura.
- **AC-005** — Dado qualquer Fonte de áudio ativa (Buffer/Manual/Contínuo), quando o usuário olha a bandeja ou o Overlay, então vê o indicador da Fonte; a Pausa de privacidade interrompe a captura de áudio como a de tela.
- **AC-006** — Dado Modo Contínuo com Retenção, quando os limites são excedidos, então os Segmentos mais antigos são removidos (mesma regra do 004).

### US-003 — Levar o áudio para a Conversa (Priority: P1)

#### Acceptance scenarios

- **AC-007** — Dado Buffer recente de áudio ativo, quando o usuário escolhe "Anexar últimos 2 min de áudio" (Microfone, Sistema ou Ambos), então surge um Chip de Recorte de áudio; ao enviar, a Conversa recebe a Transcrição com marcações de tempo e rótulo da Fonte ("Você"/"Sistema"), e o arquivo de áudio fica no Workspace da conversa.
- **AC-008** — Dado Permissão do agente para áudio = Sempre (ou concedida), quando o agente chama `audio_recent{minutes, source}`, então recebe a Transcrição do intervalo; com Nunca/Perguntar, comporta-se como a tela (negado ou cartão).
- **AC-009** — Dado Buffer de tela e de áudio ativos, quando o usuário anexa "últimos 2 min" com "tela + áudio", então o Chip combina keyframes e Transcrição do mesmo intervalo.

## Requirements

- **FR-001** — O sistema MUST capturar Microfone e Áudio do sistema como Fontes independentes com seleção de dispositivo e medidor.
- **FR-002** — O sistema MUST oferecer os Modos por Fonte de áudio usando Segmentos cifrados e Retenção compartilhados com a tela.
- **FR-003** — O sistema MUST anexar Recortes de áudio como Transcrição com tempo e Fonte, guardando o áudio no Workspace da conversa.
- **FR-004** — O sistema MUST expor `audio_recent` ao agente sob a Política de privacidade.

## Limits, errors, and compatibility

- Formato de captura interno: 16 kHz mono por Fonte para ASR + Opus 24 kbps nos Segmentos (qualidade de fala).
- Áudio do sistema captura tudo o que toca no dispositivo padrão (inclusive do próprio Aura, como TTS — o Aura silencia seu TTS da captura marcando a sessão; se não for possível, documentar).
- Sem microfone disponível → Fonte indisponível com motivo; permissão de microfone negada nas configurações de privacidade do Windows → mensagem com link `ms-settings:privacy-microphone`.
- Anexar mais de 30 min de áudio de uma vez é recusado (sugere Gravação + anexo de arquivo pelo 007).

## Hypotheses and dependencies

- H-013: Opus a 24 kbps mono mantém qualidade suficiente para ASR (WER ≤ +1 ponto vs. PCM). Check: TK-002 com arquivo de referência.
- Dependências: 004 (Política, segmentos, retenção, indicadores, MCP), 006 (Interface `Transcriber`).

## Success criteria

### Delivery-verifiable

- **SC-001** — AC-001–AC-009 com evidência; teste de loopback com tom conhecido de 1 kHz reconhecido no Segmento decifrado.

### Post-delivery observation

- **SC-002** — Usuários do beta que ligam Buffer de áudio o mantêm ligado por ≥ 1 semana (indicador de confiança).

## Decisions and open questions

- Padrões: Microfone = Sob demanda (só push-to-talk); Áudio do sistema = Desligado; Permissão do agente para áudio = Perguntar.
- Nenhuma pergunta pendente.
