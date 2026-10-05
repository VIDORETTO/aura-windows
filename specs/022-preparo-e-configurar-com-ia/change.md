---
schema: hybrid/change
schema_version: 1.0
effort_id: 022-preparo-e-configurar-com-ia
revision: 1
status: draft
profile: compact
---

# Change: Preparo em três velocidades e Configurar com IA

## Objetivo e limites

Candidatos: CAND-033, CAND-034. Origem: plano `docs/project/plano-reuniao-e-configuracao-assistida.md`. Padrão de preparo (⚡ uma frase, 🧭 grill-me, ▶ sem preparo) e assistente que configura o Aura por conversa, com antes → depois e Desfazer.

Nota: Base: core-skills e ferramentas MCP de escrita do esforço 017. **Feito (05/10):** `settings_describe/propose/apply/undo`, Skills `aura-configurar` e `aura-preparo`, `/configurar`, `/preparo`, "Pedir à IA" na busca. **Falta:** cartão "Entendi assim" editável na UI, onboarding "conte como trabalha", Janelas excluídas e Perfis de app por IA, desfazer persistente entre sessões.

## Contrato de comportamento

- Entradas: ver requisitos e critérios abaixo.
- Saída: ver critérios de aceite.
- Erros/invariantes: segredos nunca expostos; captura só pela Política de privacidade.
- Compatibilidade: comportamento existente do Overlay, Comandos rápidos e Configurações.

## Requisitos e aceite

- **FR-001** — O Aura MUST ter o cartão "Entendi assim" e a Skill interna `aura-preparo`.
- **AC-001** — Dada uma frase, então surge o cartão com objetivo e pontos, com no máximo 2 perguntas; no 🧭, até ~8 perguntas, uma por vez, com resposta recomendada; "Começar" sempre disponível.
- **FR-002** — Ferramentas `settings_describe`, `settings_propose`, `settings_apply` e `settings_undo` MUST permitir configurar por conversa sem tocar em segredos.
- **AC-002** — Dado "não grave nada quando eu abrir o app do banco", então `settings_propose` devolve o diff de Janelas excluídas sem gravar; `settings_apply` grava com aprovação; `settings_undo` restaura; nenhuma ferramenta altera segredos nem YOLO.
- **FR-003** — O Aura MUST oferecer "Pedir à IA" na busca das Configurações, `/configurar` e onboarding "conte como trabalha".
- **AC-003** — Dada uma busca sem resultado, então "Pedir à IA" abre uma conversa com o pedido; `/configurar` abre o assistente.

## Leitura e mapa de alterações

- `[path]` → `[symbol]` — [motivo]; [existing/new].

## Plano breve

Seam: [interface sob teste]. Abordagem: [decisão técnica local]. Dependências: [none ou refs].

## Sequência e tarefas

- [ ] C-001 Escrever caso relevante e observar red pelo motivo esperado.
- [ ] C-002 Implementar o mínimo e observar green.
- [ ] C-003 Executar regressão e registrar evidência.

## Validação e evidência

Comando/procedimento: `[exact command]`

Resultado executado: [pending]

Limitações: [o que não foi verificado]. Comando apenas identificado na configuração: [none ou registro].

## Condição de retorno

Retornar se o contrato, símbolo, dependência ou decisão material não puder ser satisfeito sem ampliar o escopo.

## Estado

`change.md` é a fonte canônica deste esforço compacto. Não crie `tasks.md` ou tickets paralelos para esta mudança.
