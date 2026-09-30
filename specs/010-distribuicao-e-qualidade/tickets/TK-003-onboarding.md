---
schema: hybrid/ticket
schema_version: 1.0
id: TK-003
effort: 010-distribuicao-e-qualidade
type: delivery
status: implemented
ticket_revision: 3
requires: []
requirement_refs: ["FR-003"]
acceptance_refs: ["AC-006"]
spec_revision: 1
plan_revision: 1
owned_areas: ["apps/desktop/src/onboarding", "apps/desktop/src-tauri/src/onboarding.rs"]
verification_status: partial
last_update: Implementado e testado no Linux; validação no Windows pendente (docs/HANDOFF.md).
---



# TK-003 — Onboarding de primeira execução

## Objetivo e limites

Entrega a janela de onboarding em 5 passos (atalho com teste ao vivo; "Continue with ChatGPT" ou "Usar minha chave"; privacidade com padrões seguros; voz com modelo recomendado opcional; primeiro pedido guiado), com pular/retomar e estado persistido.

Não inclui: tutoriais em vídeo.

## Leitura em ordem

1. `specs/010-distribuicao-e-qualidade/spec.md` → AC-006.
2. Componentes existentes: `ContinueWithChatGptButton`/`FirstUseModal` (002 TK-002), `ProviderForm` (003 TK-001), `PrivacySection` (004 TK-003), `ModelCatalog` (006 TK-001), atalho (001 TK-004).
3. `https://developers.openai.com/siwc/ui-ux-guidelines.md` → oferecer uso do plano no onboarding.

## Decisões já resolvidas

- Reusar componentes existentes; o onboarding só orquestra.
- Passo de privacidade mostra os padrões (Tela sob demanda, Microfone sob demanda, Áudio do sistema desligado, Permissão do agente = Perguntar) sem alterar nada até o usuário mudar.
- Primeiro pedido guiado: abrir o Overlay e sugerir "/screen o que está na minha tela?".
- Liberdade local: ilustrações e textos.

## Mapa de alterações

- Novo: `apps/desktop/src/onboarding/{Onboarding.tsx,steps/*.tsx}`; `apps/desktop/src-tauri/src/onboarding.rs` (estado `onboarding.step`, `completed`).

## Contrato técnico

- Entradas: primeira execução (`onboarding.completed=false`).
- Saídas: configurações escolhidas; `completed=true` ou `step` salvo ao pular.
- Invariantes: pular não altera configurações.

## Exemplos de aceite

- **AC-006**: Vitest/E2E: primeira execução → janela com 5 passos; testar atalho no passo 1 marca "funcionou" ao detectar o toggle; pular no passo 3 → `step=3` salvo; Configurações > "Retomar configuração inicial" reabre no passo 3; concluir → `completed=true` e Overlay aberto com a sugestão.

## Dependências e sequência de execução

Depende de: 002-conversa-agente-codex/TK-002, 003-byok-gateway/TK-001, 004-contexto-de-tela/TK-003, 006-voz-e-asr/TK-002 (outros esforços).

- [ ] TK-003.1 Estado do onboarding (unit) red→green.
- [ ] TK-003.2 Passos com componentes existentes; Vitest.
- [ ] TK-003.3 E2E; evidências.

## Validação

- Diretório: raiz do repositório.
- Comando/procedimento exato: `pnpm -C apps/desktop test -- onboarding`; `pnpm -C apps/desktop e2e -- --spec e2e/onboarding.spec.ts`.
- Estado esperado: verdes.
- Comando identificado na configuração mas não executado: nenhum.
- Distinguir defeito de ambiente: nenhum.

## Condição de retorno à planejadora

Nenhuma prevista.

## Relatório de saída

Relatar resultados e EV refs.
