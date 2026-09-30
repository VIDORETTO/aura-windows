---
schema: hybrid/spec
schema_version: "1.0"
effort_id: 006-voz-e-asr
revision: 1
status: accepted
profile: standard
---

# Specification: Voz e ASR

## Problem and desired result

Falar é mais rápido que digitar, mas cada usuário tem um idioma, um hardware e uma tolerância diferente a enviar voz para a nuvem. O resultado é um Push-to-talk fluido no Overlay (e por atalho global) com o Modelo ASR que o usuário escolher: um catálogo claro para baixar modelos locais recomendados para o seu computador, ou ASR em nuvem via Provedor BYOK — com idioma, vocabulário personalizado e parciais em tempo real quando o modelo permite.

## Consumers and actors

- Usuário (fala, escolhe e baixa modelos).
- Esforço 005 (Recortes de áudio usam o `Transcriber`), 007 (anexos de áudio/vídeo), CAND-011 (ditado em qualquer app).

## Scope

### Included

- Catálogo de modelos com metadados, licença e recomendação por hardware/idioma.
- Gerenciador de downloads com retomada, verificação, cancelamento e remoção.
- Motor de transcrição local no `aura-worker` com carga preguiçosa e descarga por inatividade.
- Push-to-talk no Overlay (segurar ou alternar) e por atalho global; VAD; cancelar; enviar automaticamente (opcional).
- ASR em nuvem por Provedor BYOK compatível (`/audio/transcriptions`).
- Idioma (automático ou fixo) e vocabulário personalizado.
- Parciais em streaming com modelos compatíveis.

### Excluded

- Ditado em qualquer aplicativo (CAND-011).
- Voz em tempo real com o modelo (CAND-012).
- TTS (009).

## User journeys and scenarios

### US-001 — Escolher e baixar um modelo (Priority: P1)

#### Acceptance scenarios

- **AC-001** — Dado a tela "Voz", quando o usuário abre o Catálogo de modelos, então cada modelo mostra nome, tamanho em disco, idiomas, barras de velocidade e precisão, requisitos (CPU/GPU, RAM) e licença, e um selo "Recomendado para você".
- **AC-002** — Dado o hardware e o idioma da interface, quando o catálogo calcula a recomendação, então: pt-BR ou outro idioma europeu com CPU comum → Parakeet V3; qualquer idioma com GPU dedicada de ≥ 6 GB → Whisper Turbo; inglês em máquina com < 8 GB de RAM → Moonshine Small; chinês/japonês/coreano → SenseVoice.
- **AC-003** — Dado um download em andamento, quando a rede cai e volta, então o download continua do ponto em que parou; quando o usuário cancela, então o arquivo parcial é apagado; quando o arquivo baixado não confere com o SHA-256 do catálogo, então é apagado e o erro é mostrado.
- **AC-004** — Dado espaço livre menor que 1,2 × o tamanho do modelo, quando o usuário pede o download, então ele é bloqueado com a quantidade necessária.
- **AC-005** — Dado um modelo instalado, quando o usuário o remove, então o espaço é liberado e, se era o selecionado, a seleção passa para outro modelo instalado ou para "nenhum".

### US-002 — Transcrever localmente (Priority: P1)

#### Acceptance scenarios

- **AC-006** — Dado um modelo local instalado, quando uma transcrição é pedida pela primeira vez, então a UI mostra "carregando modelo de voz" e o texto correto é produzido; após 2 minutos sem uso (e sem Overlay em escuta), o processo de transcrição termina e sua memória é devolvida ao sistema.

### US-003 — Falar no Overlay (Priority: P1)

Independent demonstration: segurar `Ctrl+Space` no Overlay, dizer "qual a capital da Austrália", soltar, ver o texto e enviar.

#### Acceptance scenarios

- **AC-007** — Dado o Overlay aberto e um modelo pronto, quando o usuário segura `Ctrl+Space` (ou clica no microfone para alternar), fala e solta, então o texto transcrito é inserido na posição do cursor da barra de entrada em até 1,5 s após soltar para 10 s de fala (Parakeet V3 na máquina de referência); `Esc` durante a escuta cancela sem inserir; se só houver silêncio, nada é inserido e aparece a dica "Não ouvi nada".
- **AC-008** — Dado "Enviar ao terminar de falar" ativado, quando a transcrição termina, então o turno é enviado automaticamente.
- **AC-009** — Dado o atalho global de voz (padrão segurar `Ctrl+Alt+Space`), quando o usuário o segura em qualquer app, então o Overlay abre já escutando e, ao soltar, o texto aparece na barra de entrada.
- **AC-013** — Dado nenhum modelo instalado nem ASR em nuvem configurado, quando o usuário tenta falar, então aparece um cartão com o modelo recomendado e o botão "Baixar (N MB)", e o microfone passa a funcionar ao terminar.

### US-004 — Preferências de reconhecimento (Priority: P2)

#### Acceptance scenarios

- **AC-010** — Dado um Provedor BYOK com transcrição (ex.: OpenAI, Groq), quando o usuário escolhe "ASR em nuvem" com esse provedor, então o áudio da fala é enviado a ele, o botão de microfone mostra o selo "nuvem", e em caso de erro o Aura usa o modelo local instalado (se houver) avisando.
- **AC-011** — Dado idioma fixo "Português" e vocabulário personalizado ["Aura", "Codex", "Parakeet"], quando o usuário fala esses termos, então o idioma não é trocado por detecção automática e os termos aparecem com a grafia do vocabulário.
- **AC-012** — Dado um modelo com suporte a streaming, quando o usuário fala, então o texto parcial aparece enquanto fala, com atraso ≤ 500 ms, e é substituído pelo texto final ao soltar.

## Requirements

- **FR-001** — O sistema MUST apresentar um Catálogo de modelos com metadados, licenças e checksums.
- **FR-002** — O sistema MUST recomendar um modelo pelo hardware e idioma.
- **FR-003** — O sistema MUST baixar modelos com retomada, verificação, cancelamento, verificação de espaço e remoção.
- **FR-004** — O sistema MUST transcrever localmente num processo separado, carregado sob demanda e encerrado após inatividade.
- **FR-005** — O sistema MUST oferecer Push-to-talk no Overlay e por atalho global, com VAD, cancelamento e envio automático opcional.
- **FR-006** — O sistema MUST oferecer ASR em nuvem via Provedor BYOK com fallback local.
- **FR-007** — O sistema MUST permitir idioma fixo/automático e vocabulário personalizado.
- **FR-008** — O sistema SHOULD mostrar parciais em streaming com modelos compatíveis.

## Limits, errors, and compatibility

- Fala máxima por Push-to-talk: 5 min (acima, encerra e transcreve o que tem).
- Pausa de privacidade bloqueia o microfone (005 TK-003).
- Modelos só-inglês ficam marcados e a recomendação nunca os sugere para outro idioma.
- GPU: Whisper usa Vulkan/CUDA quando disponível via `transcribe-cpp`; ONNX usa DirectML quando disponível, senão CPU.
- Espelho próprio dos modelos; nenhum download de CDN de terceiros não verificado.

## Hypotheses and dependencies

- H-006: Parakeet V3 int8 transcreve 10 s em < 1 s em 4 núcleos. Check: TK-003.
- H-014: carregar Parakeet V3 no worker leva < 2 s em SSD. Check: TK-003.
- Dependências: 005 TK-001 (`AudioHub`), 003 TK-001 (Provedores para ASR em nuvem), 001 (atalhos).

## Success criteria

### Delivery-verifiable

- **SC-001** — AC-001–AC-013 com evidência; WER do modelo recomendado em pt-BR medido num conjunto de 50 frases de referência e registrado.

### Post-delivery observation

- **SC-002** — ≥ 40% dos turnos do beta iniciados por voz entre usuários que baixaram um modelo.

## Decisions and open questions

- Push-to-talk no Overlay: `Ctrl+Space` segurar (configurável); atalho global de voz: segurar `Ctrl+Alt+Space`.
- Catálogo inicial: Parakeet V3 (25 idiomas europeus, ~456 MB), Whisper Small/Medium/Turbo/Large v3, Moonshine V2 Tiny/Small/Medium (inglês, streaming), SenseVoice (zh/en/ja/ko/yue), Canary 1B v2 (25 idiomas). Lista final depende da verificação de licença de redistribuição (TK-001).
