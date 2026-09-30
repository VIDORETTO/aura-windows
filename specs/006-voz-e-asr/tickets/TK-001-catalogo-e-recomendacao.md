---
schema: hybrid/ticket
schema_version: 1.0
id: TK-001
effort: 006-voz-e-asr
type: delivery
status: implemented
ticket_revision: 3
requires: []
requirement_refs: ["FR-001", "FR-002"]
acceptance_refs: ["AC-001", "AC-002"]
spec_revision: 1
plan_revision: 1
owned_areas: ["crates/aura-asr/src/catalog.rs", "crates/aura-asr/src/hardware.rs", "crates/aura-asr/src/recommend.rs", "crates/aura-asr/models.toml", "apps/desktop/src/settings/voice/ModelCatalog.tsx"]
verification_status: partial
last_update: Lógica implementada e testada no Linux; validação no Windows pendente (docs/HANDOFF.md §4 e §6).
---



# TK-001 — Catálogo de modelos, detecção de hardware e recomendação

## Objetivo e limites

Entrega `models.toml` (com licença, checksum e URL do espelho), `Catalog`, `hardware::probe` (RAM, núcleos, GPUs com VRAM, NPU) e `recommend`, além da UI do Catálogo com selo "Recomendado para você".

Não inclui: download (TK-002) — botões ficam desabilitados com "em breve" até TK-002.

## Leitura em ordem

1. `specs/006-voz-e-asr/spec.md` → AC-001, AC-002, "Decisions" (catálogo inicial).
2. `specs/006-voz-e-asr/plan.md` → catalog, hardware, recommend, OT-004.
3. Referência: `cjpais/Handy` `src-tauri/src/managers/model.rs` (metadados, idiomas, tamanhos).
4. Licenças das fontes originais: NVIDIA Parakeet (Hugging Face `nvidia/parakeet-tdt-0.6b-v3`), OpenAI Whisper, Moonshine, SenseVoice, Canary.

## Decisões já resolvidas

- `models.toml` campos: `id, name, family, engine, size_bytes, sha256, url, languages[], speed(0–1), accuracy(0–1), streaming, min_ram_mb, gpu_recommended, license, redistribution_ok, source_url`.
- Entradas sem `redistribution_ok = true` não aparecem no catálogo público (OT-004).
- `recommend` usa as regras de AC-002 na ordem: GPU dedicada ≥ 6 GB → Whisper Turbo; idioma em {zh, ja, ko, yue} → SenseVoice; inglês e RAM < 8 GB → Moonshine Small; idioma coberto pelo Parakeet V3 → Parakeet V3; senão → Whisper Small.
- Liberdade local: design dos cartões.

## Mapa de alterações

- Novo: `crates/aura-asr/{Cargo.toml,src/lib.rs,src/catalog.rs,src/hardware.rs,src/recommend.rs,models.toml}`.
- Novo: `apps/desktop/src-tauri/src/voice.rs` → comandos `asr_catalog`, `asr_recommendation`.
- Novo: `apps/desktop/src/settings/voice/{VoiceSection.tsx,ModelCatalog.tsx,ModelCard.tsx}`.

## Contrato técnico

- Entradas: catálogo embutido, `Hardware`, idioma da UI.
- Saídas: lista de `ModelEntry` e `ModelId` recomendado.
- Invariantes: recomendação sempre aponta para entrada existente e publicada.
- Erros: catálogo inválido → erro de build (teste que parseia o arquivo real).

## Exemplos de aceite

- **AC-001**: teste parseia `models.toml` real → todas as entradas com os campos obrigatórios e `sha256` de 64 hex; Vitest renderiza cartão "Parakeet V3 · 456 MB · 25 idiomas · CC-BY-4.0".
- **AC-002**: `recommend(Hardware{ram:16384, gpus:[]}, "pt-BR")` → `parakeet-tdt-0.6b-v3`; `(16384, [Nvidia 8192 dedicated])` → `whisper-turbo`; `(6144, [], "en")` → `moonshine-small-streaming-en`; `(8192, [], "ja")` → `sense-voice-int8`; `(8192, [], "tr")` → `whisper-small` (turco fora do Parakeet V3).

## Dependências e sequência de execução

Depende de: nenhum.

- [ ] TK-001.1 Verificar licenças e montar `models.toml` (registrar decisão por modelo).
- [ ] TK-001.2 Teste de parse do catálogo real (red→green).
- [ ] TK-001.3 Unit `recommend` caso a caso.
- [ ] TK-001.4 `hardware::probe` Windows (integração: valores plausíveis > 0).
- [ ] TK-001.5 UI + Vitest; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-asr catalog recommend hardware`; `pnpm -C apps/desktop test -- ModelCatalog`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: nenhum.

## Condição de retorno à planejadora

Retornar se o Parakeet V3 não puder ser redistribuído (mudaria a recomendação padrão para pt-BR).

## Relatório de saída

Relatar catálogo final com licenças, resultados, EV refs.
