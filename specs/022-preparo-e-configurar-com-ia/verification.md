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
