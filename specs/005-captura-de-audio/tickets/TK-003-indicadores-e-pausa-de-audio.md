---
schema: hybrid/ticket
schema_version: 1.0
id: TK-003
effort: 005-captura-de-audio
type: delivery
status: implemented
ticket_revision: 3
requires: ["TK-002"]
requirement_refs: ["FR-002"]
acceptance_refs: ["AC-005"]
spec_revision: 1
plan_revision: 1
owned_areas: ["apps/desktop/src/overlay/CaptureIndicators.tsx", "crates/aura-audio/src/policy_gate.rs"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-003 — Indicadores e Pausa de privacidade para áudio

## Objetivo e limites

Entrega a integração das Fontes de áudio com `aura-policy` (Modo, Pausa) e com os indicadores da bandeja/Overlay criados no 004 (âmbar = Buffer recente, vermelho = gravando), com parada por clique.

Não inclui: novas regras de exclusão (Janelas excluídas não se aplicam a áudio).

## Leitura em ordem

1. `crates/aura-policy/src/decide.rs` (004 TK-003) → `Source::Mic|SystemAudio`.
2. `apps/desktop/src/overlay/CaptureIndicators.tsx` (004 TK-005).
3. `crates/aura-audio/src/recorder.rs` (TK-002).

## Decisões já resolvidas

- `AudioPolicyGate` consulta `decide` a cada mudança de Política e antes de iniciar; Pausa fecha o segmento corrente e para o stream do dispositivo (libera o microfone — ícone de microfone do Windows some).
- Push-to-talk (006) continua permitido durante Pausa? **Não**: Pausa bloqueia todas as Fontes, inclusive PTT; o botão de microfone mostra "Pausa ativa".
- Liberdade local: ícones.

## Mapa de alterações

- Novo: `crates/aura-audio/src/policy_gate.rs` → `AudioPolicyGate`.
- Existente: `CaptureIndicators.tsx` e `tray.rs` → Fontes de áudio.

## Contrato técnico

- Entradas: mudanças de Política/Pausa.
- Saídas: streams iniciados/parados; estado de indicadores.
- Invariantes: com Pausa, nenhum dispositivo de captura de áudio aberto pelo Aura.

## Exemplos de aceite

- **AC-005**: Mic em Buffer + Pausa → `AudioHub` sem stream ativo em ≤ 500 ms (unit com sintético); Windows: indicador de microfone em uso do sistema desaparece; indicadores do Aura mostram "pausado"; retomar → volta.

## Dependências e sequência de execução

Depende de: TK-002.

- [ ] TK-003.1 Unit gate + sintético (red→green).
- [ ] TK-003.2 Indicadores; Vitest; manual Windows; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `cargo nextest run -p aura-audio policy_gate`; `pnpm -C apps/desktop test -- CaptureIndicators`; roteiro manual.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: nenhum.

## Condição de retorno à planejadora

Retornar se parar/abrir o dispositivo repetidamente causar falhas de driver.

## Relatório de saída

Relatar resultados e EV refs.
