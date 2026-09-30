---
schema: hybrid/ticket
schema_version: 1.0
id: TK-002
effort: 010-distribuicao-e-qualidade
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-001"]
requirement_refs: ["FR-002"]
acceptance_refs: ["AC-004", "AC-005"]
spec_revision: 1
plan_revision: 1
owned_areas: ["apps/desktop/src-tauri/src/update.rs", "apps/desktop/src/settings/UpdatesSection.tsx"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-002 — Atualização automática assinada com canais

## Objetivo e limites

Entrega `UpdateService` (verificação na inicialização e a cada 6 h, download em segundo plano, validação de assinatura, aplicação só quando ocioso), canais Estável/Beta e o pré-passo que garante a nova versão fixada do app-server antes da troca.

Não inclui: delta updates.

## Leitura em ordem

1. Docs atuais: `tauri-plugin-updater` (manifesto, `pubkey`, endpoints por canal), `tauri-plugin-process` (relaunch).
2. `crates/aura-codex/src/binary.rs` (002 TK-001) → `CodexBinary::ensure(version)`.
3. `specs/010-distribuicao-e-qualidade/plan.md` → OT-001.

## Decisões já resolvidas

- Manifesto inclui campo extra `codex_app_server: {version, asset, sha256}`; `pre_update_hook` chama `CodexBinary::ensure` para essa versão antes de oferecer reinício.
- "Ocioso" = sem turno ativo, sem gravação manual, sem download de modelo em andamento.
- Liberdade local: UI da seção.

## Mapa de alterações

- Novo: `apps/desktop/src-tauri/src/update.rs` → `UpdateService`.
- Novo: `apps/desktop/src/settings/UpdatesSection.tsx`; banner "Reiniciar para atualizar".

## Contrato técnico

- Entradas: canal; manifesto.
- Saídas: pacote validado e oferta de reinício.
- Invariantes: OT-001; assinatura inválida → descartar e registrar.

## Exemplos de aceite

- **AC-004**: `wiremock` com manifesto assinado por chave de teste → download e oferta; manifesto adulterado → descartado com log `warn`; turno ativo → oferta adiada até `TurnCompleted`.
- **AC-005**: manifesto com `codex_app_server.version` nova → `CodexBinary::ensure` chamado e concluído antes da oferta; após reiniciar, `thread/resume` de uma Conversa antiga funciona (manual com duas versões).

## Dependências e sequência de execução

Depende de: TK-001.

- [ ] TK-002.1 Validação/rejeição (unit com chave de teste) red→green.
- [ ] TK-002.2 Regra de ociosidade.
- [ ] TK-002.3 Pré-passo do app-server; manual com canal de teste; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-desktop update`; roteiro manual de atualização entre duas builds.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: sem servidor de updates de teste.

## Condição de retorno à planejadora

Retornar se o updater do Tauri não suportar campos extras no manifesto (usar arquivo auxiliar assinado).

## Relatório de saída

Relatar resultados e EV refs.
