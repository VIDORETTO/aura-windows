---
schema: hybrid/ticket
schema_version: 1.0
id: TK-003
effort: 039-web-gratuita-do-agente
type: delivery
status: done
ticket_revision: 62
requires: ["TK-002"]
requirement_refs: ["FR-001", "FR-005", "FR-007", "FR-008"]
acceptance_refs: ["AC-001", "AC-007", "AC-008", "AC-011", "AC-014"]
spec_revision: 1
plan_revision: 11
owned_areas: ["crates/aura-app/src/host.rs", "crates/aura-app/src/tools.rs", "crates/aura-app/src/web.rs", "crates/aura-app/tests/web_tools.rs", "crates/aura-app/tests/real_app_server.rs", "crates/aura-mcp", "crates/aura-codex", "crates/aura-gateway", "crates/aura-app/Cargo.toml", "apps/desktop/src-tauri/src/main.rs", "apps/desktop/src/state/app.ts", "apps/desktop/src/state/app.test.ts", "apps/desktop/src/i18n"]
verification_status: passed
last_update: Git LF normalization explicitly revalidated; exact original fingerprints recoverable without behavioral edits. Canonical evidence/current0.3 release documented in041.
---





























































# TK-003 — Integrar pesquisa e leitura ao agente nos modos existentes

## Objetivo e limites

Ferramentas MCP do Aura, pesquisa iterativa/leitura/citações, isolamento de conteúdo externo e cancelamento do Turno em Chat/Plano/Tarefa; protocolo fixado e provedores traduzidos. Não inclui novo loop LLM, pagar busca hospedada ou abrir sandbox de comandos.

Autorização: o usuário pediu executar todos os esforços e tickets, um por vez, deixando bloqueios para o final. Não inclui itens fora da fatia.

## Leitura em ordem

1. `AGENTS.md`, `docs/HANDOFF.md`, `CONTEXT.md` e ADRs do plano.
2. `specs/039-web-gratuita-do-agente/spec.md` r1, `plan.md` r11, `contracts.md`, `tdd.md` → contratos e casos.
3. `crates/aura-app/src/tools.rs` → HostTools::call.
4. `crates/aura-app/src/host.rs` → composição/send/interrupt/close.
5. `crates/aura-mcp/src/tools.rs` → all/annotations.
6. `crates/aura-mcp/src/lib.rs` → ToolHandler/CallContext.
7. `crates/aura-codex/src/home.rs` → BaseConfig.web_search/render.
8. `crates/aura-codex/src/modes.rs` → persona/modes.
9. `crates/aura-app/tests/real_app_server.rs` → contrato real.

## Decisões já resolvidas

Abordagem/contratos/limites do plano r11. Seams públicas confirmadas; testes de Host/MCP e processo fixado executados e registrados no runner. Não alterar comportamento para fixture passar.

Liberdade local: nomes privados/organização e mensagens redigidas dentro do contrato. Alternativas no plano. Dependência/contrato material novo retorna à planejadora.

## Mapa de alterações

Novo web.rs/tests/web_tools.rs; compor serviço no Host. Schema web_search/web_fetch em tools::all, openWorldHint=true. Persona pt-BR/en e config web_search disabled; fixado real/gateway namespace. Sem upgrade ou login alternativo.

Aviso de modelo sem ferramentas integra esta fatia (AC-001), pelo evento Notice já existente. Host envia código estável web_tools_unsupported; estado público useApp.handle traduz pela tabela pt-BR/en sem novo formato IPC. O teste na seam de eventos públicos da UI usa mensagem literal independente e prova as duas línguas; não inclui painel, fontes ou toggle do TK-004. Sobreposição i18n/estado com sucessor segue serializada pela dependência existente.

Owned areas delimitam responsabilidade; um ticket por vez, preservando trabalho preexistente. Arquivos/símbolos dos predecessores são previstos; existentes após sua implementação.

## Contrato técnico

Host resolve Conversa/Turno confiáveis; argumento remoto não define budget. Fonte é dado externo, marker resolve só registry válido. Busca hospedada desativada para gratuita; ferramentas só leitura não aumentam grants. Interrupt/close/shutdown cancelam rede/cache conforme ciclo. Protocolo usa ferramentas MCP existentes em transporte real e scripted.

Correlação: registro de Turnos ativos pelo app-server; header opcional deve apontar a registro ativo. Sem header, exatamente um Turno ativo resolve; zero/múltiplos falham antes de cache/rede com invalid_input/contexto indisponível. Nunca usar seleção da UI. Provar caso de duas Conversas ativas e ausência de contaminação; é condição de uso do budget confiável, não suporte presumido do app-server.

Erros/unidades/limites/efeitos/compatibilidade seguem spec/plan/contracts. Segredos fora de UI/config/log. Nenhuma evidência inventada.

## Exemplos de aceite

- **AC-007**: busca→fetch W1→produção42 com markerW1; comparação W1/W2 domínios distintos ou ressalva.
- **AC-008**: página instrui exfiltrar/executar → sem segredo/efeito/Persona; W999 não é fonte verificada.
- **AC-011**: cancel pending → cancelled/sem cache/sem fallback; parte cache/TTL fica TK-004.
- **AC-014**: fixado Responses/Chat/Anthropic + Chat/Plano/Tarefa → namespace/resultado correto; sem search pago ou network grant.

Oráculo independente: literais de spec e fixtures sintéticas de tdd.md; sem recompor algoritmo ou esperado da implementação.

## Dependências e sequência de execução

Depende de TK-002 em done: serviço/contrato do predecessor é necessário.

- [x] TK-003.1 Executar `python .hybrid/hybrid.py package --project . --effort 039-web-gratuita-do-agente --ticket TK-003 --json`; ready:true e inputs atuais antes de implementar.
- [x] TK-003.2 Confirmar seams, primeiro caso → red comportamental, não ambiente.
- [x] TK-003.3 Green mínimo, um caso por vez; caso já verde é cobertura, não red artificial.
- [x] TK-003.4 Regressões/integração e resultado com limitações.
- [x] TK-003.5 Evidence add só após execução real, ticket update/checkpoint, revisão antes de done, projeções por render.

## Validação

Diretório: raiz. Comandos futuros:

```powershell
cargo test -p aura-app --test web_tools
cargo test -p aura-mcp --test protocol
cargo test -p aura-gateway
cargo test -p aura-codex
cargo test -p aura-app --test real_app_server -- --ignored --nocapture
```

Executado: EV-010 a EV-013 registram Host/MCP, avisos pt-BR/en, doze casos de web com sidecar real, regressão da sandbox, gateway/Codex/MCP e checks estáticos. O probe bruto de diagnóstico foi excluído; limites de cada procedimento estão na evidência. Nenhuma prova desta fatia aprova o painel, cache agregado ou qualidade ao vivo dos sucessores.

Toolchain projeto; Windows/MSVC/WebView2/WebDriver para E2E; AURA_CODEX_BIN fixado verificado para real. Falta de ferramenta/conta/rede é not_run/impedimento, não aprovação. Dourado/real/benchmark seguem plano/TDD, sem dumps de conteúdo/chaves.

## Condição de retorno à planejadora

Contrato real incompatível, predecessor não done, input mudou, necessidade de pago/JS/upgrade, mudança de comportamento/autoridade → devolver caminho/símbolo/resultado. Preservar progresso; não ampliar escopo/reduzir oráculo.

## Relatório de saída

Concluído no escopo TK-003 com EV-010 a EV-013 e revisão Standards/Spec no baseline fixo. Próxima ação: executar TK-004, preservar os limites da prova e renovar os checks quando código compartilhado mudar.

## Reabertura r11 — consumidor SIWC

FD004: implementar somente a diretiva de namespace MCP direto em `CodexService::start`/`ensure_loaded`, com os casos públicos de início/retomada descritos no plano11. O override vazio do chamador não pode ocultar ferramentas do Aura. Registro do protótipo não aprova esta implementação. Gate final das três jornadas permanece no TK005. Os testes e evidências anteriores são históricos até renovação.
