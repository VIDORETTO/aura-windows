---
schema: hybrid/ticket
schema_version: 1.0
id: TK-003
effort: 009-produtividade
type: delivery
status: implemented
ticket_revision: 3
requires: []
requirement_refs: ["FR-003"]
acceptance_refs: ["AC-005", "AC-006"]
spec_revision: 1
plan_revision: 1
owned_areas: ["apps/desktop/src-tauri/src/productivity/tts.rs", "apps/desktop/src/conversation/SpeakButton.tsx", "apps/desktop/src/settings/voice/TtsSection.tsx"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-003 — Leitura em voz alta (TTS)

## Objetivo e limites

Entrega `speakable(markdown)`, `WindowsSpeaker` (SpeechSynthesis WinRT, seleção de voz/velocidade), `CloudSpeaker` (Provedor BYOK com `/audio/speech`) com consentimento único, botão por mensagem, atalho e "ler automaticamente".

Não inclui: voz em tempo real (CAND-012).

## Leitura em ordem

1. `specs/009-produtividade/spec.md` → AC-005, AC-006.
2. Docs atuais: `Windows.Media.SpeechSynthesis.SpeechSynthesizer` (voices, `SynthesizeTextToStreamAsync`), `rodio`; OpenAI `/v1/audio/speech`.
3. `crates/aura-gateway/src/{registry.rs,credentials.rs}` (003) para o `CloudSpeaker`.

## Decisões já resolvidas

- `speakable`: remove blocos de código (substitui por "bloco de código omitido"), links → texto do link ou domínio, tabelas → "tabela com N linhas", Markdown → texto plano; divide em frases para começar a falar em ≤ 300 ms.
- Voz padrão: primeira voz do idioma da UI; configurável.
- Áudio do TTS marcado para exclusão da captura de sistema quando possível (005 risco).
- Liberdade local: UI do seletor.

## Mapa de alterações

- Novo: `apps/desktop/src-tauri/src/productivity/tts.rs` → `speakable`, `Speaker`, `WindowsSpeaker`, `CloudSpeaker`.
- Novo: `apps/desktop/src/conversation/SpeakButton.tsx`, `apps/desktop/src/settings/voice/TtsSection.tsx`.

## Contrato técnico

- Entradas: markdown da mensagem; voz.
- Saídas: áudio no dispositivo padrão.
- Invariantes: um único áudio por vez; `stop` imediato (≤ 100 ms).
- Erros: nenhuma voz instalada no idioma → usar qualquer voz + aviso.

## Exemplos de aceite

- **AC-005** (unit): `speakable("Veja:\n```rust\nfn a(){}\n```\nMais em [docs](https://exemplo.com/x/y).")` → "Veja: bloco de código omitido. Mais em docs."; manual: leitura começa ≤ 300 ms, clicar de novo para.
- **AC-006**: escolher voz OpenAI → modal de consentimento uma vez; aceito → `POST /v1/audio/speech` (wiremock) com o texto `speakable`; recusado → volta à voz do Windows.

## Dependências e sequência de execução

Depende de: 002-conversa-agente-codex/TK-003 e 003-byok-gateway/TK-001 (outros esforços).

- [ ] TK-003.1 Unit `speakable` caso a caso red→green.
- [ ] TK-003.2 `WindowsSpeaker` (manual + teste de síntese para stream não vazio).
- [ ] TK-003.3 `CloudSpeaker` + consentimento; UI; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-desktop tts`; `cargo nextest run -p aura-desktop --features win-integration tts_windows`; `pnpm -C apps/desktop test -- SpeakButton TtsSection`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: runner sem vozes instaladas.

## Condição de retorno à planejadora

Nenhuma prevista.

## Relatório de saída

Relatar resultados e EV refs.
