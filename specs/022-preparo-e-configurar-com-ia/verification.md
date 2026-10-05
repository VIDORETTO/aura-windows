# Verification

<!-- GENERATED from evidence/*.json. Evidence records are canonical. -->

## EV-001 — passed

- Ticket: `—`
- Acceptance: `AC-002`
- Procedure: `cargo test -p aura-app (settings_assistant + agent_configures_settings_with_diff_and_undo), -p aura-mcp`
- Execution: `executed`
- Environment: Linux x86_64
- Tested revision: `local:70b2d525f7b4bfbba0b7ba9ce93f650fa3663aee1e58a601bfd382f942754efb`
- Timestamp: `2026-10-05T17:13:16+00:00`
- Observations: propose não grava; apply muda; undo restaura; yolo, atalhos e valores inválidos recusados; exposição ampliada sinalizada
- Evidence refs: none
- Limitations: Janelas excluídas ainda não são configuráveis pelo agente; app-server real não exercitou as ferramentas novas

## EV-002 — partial

- Ticket: `—`
- Acceptance: `AC-001`, `AC-003`
- Procedure: `Skills internas aura-preparo e aura-configurar instaladas; /configurar, /preparo e "Pedir à IA" na busca (vitest + cargo test)`
- Execution: `executed`
- Environment: Linux x86_64
- Tested revision: `local:e07d48b4f514c1b4662f68f3e10134ad6f0a1720a7f6b16e75610d4a392f3a71`
- Timestamp: `2026-10-05T17:13:16+00:00`
- Observations: Skills e entradas existem e chamam o agente; busca envia $aura-configurar em modo Tarefa
- Evidence refs: none
- Limitations: Cartão 'Entendi assim' editável, onboarding 'conte como trabalha' e comportamento real do agente com as skills não verificados

## EV-003 — passed

- Ticket: `—`
- Acceptance: `AC-002`
- Procedure: `cargo test -p aura-app agent_adds_exclusions_and_profiles_but_cannot_remove_them`
- Execution: `executed`
- Environment: Linux x86_64
- Tested revision: `local:eb4930b499130a8d38ef4f056aef19c70b1a3dd76fb6659fcf4a9e882c2b1541`
- Timestamp: `2026-10-05T17:50:40+00:00`
- Observations: O agente lista janelas abertas, adiciona exclusão por processo/título (a política passa a bloquear), salva Perfis de app; não há ferramenta para remover exclusões; entradas inválidas são explicadas
- Evidence refs: none
- Limitations: none recorded

## EV-004 — partial

- Ticket: `—`
- Acceptance: `AC-003`
- Procedure: `pnpm -C apps/desktop test (FirstRun.test.tsx, OverlayApp first run, Extensions search Pedir à IA)`
- Execution: `executed`
- Environment: Linux x86_64, jsdom
- Tested revision: `local:6b538a60003bfbbd57fca00b5395f2f0bb1351ad4ad99ab4f549608ce7894d2b`
- Timestamp: `2026-10-05T17:50:41+00:00`
- Observations: Último passo dos primeiros passos 'Conte como você trabalha' envia a frase (⚡) ou o grill-me (🧭) ao agente com as Skills aura-configurar/aura-preparo; 'Pedir à IA' na busca e /configurar
- Evidence refs: none
- Limitations: Comportamento real do agente com as Skills não verificado; cartão 'Entendi assim' só existe como instrução da Skill e no painel Reunião; desfazer não persiste entre sessões
