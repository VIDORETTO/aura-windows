---
schema: hybrid/change
schema_version: 1.0
effort_id: 013-modelos-e-esforco
revision: 1
status: closed
profile: compact
---

# Change: GPT-6.1 Sol, esforço por modelo e modo, esforços em modelos personalizados

## Objetivo e limites

Pedido do usuário em 04/10/2026:

1. Adicionar o **GPT-6.1 Sol** (OpenAI, lançado em 29/09/2026; id de API `gpt-6.1-sol`; esforços `low`, `medium` (padrão), `high`, `xhigh`, `max` — sem `none`/`minimal`; entrada texto e imagem; contexto 1.050.000; saída máxima 128.000; ferramentas pela Responses API). Disponível no Codex para planos Plus, Pro, Business, Enterprise e Edu.
2. Escolher o esforço de raciocínio de cada modelo para cada tipo de tarefa (modos Chat, Tarefa, Plano).
3. Em provedores personalizados, cadastrar modelos com os esforços que aceitam e o esforço padrão.

Fora do escopo: atualizar o app-server fixado (rust-v0.159.0 já aceita `none`…`max`); preço/cobrança; validar acesso do plano ao modelo (o servidor decide).

## Contrato de comportamento

- Entradas: catálogo do plano ChatGPT (`/v1/models` + `model/list`), descoberta `/models` de provedores, cadastro manual de modelo, seletor de modelo do Overlay, Configurações › Conta e modelos.
- Saída: GPT-6.1 Sol aparece no plano ChatGPT (quando a lista do servidor não o traz, o Aura o acrescenta) e é reconhecido em provedores compatíveis (`gpt-6.1-sol`, `openai/gpt-6.1-sol`) com seus esforços e limites. O esforço escolhido no seletor fica salvo para aquele provedor + modelo + modo e volta sozinho ao escolher o mesmo modelo/modo; a tabela em Configurações edita esses padrões. Modelos manuais guardam `efforts` e `defaultEffort`.
- Erros/invariantes: esforços só do conjunto `none, minimal, low, medium, high, xhigh, max`; padrão precisa estar entre os esforços do modelo; preferência salva que o modelo não aceita é ignorada (usa o padrão do modelo).
- Compatibilidade: modelos de provedor sem `efforts` continuam com low/medium/high quando têm raciocínio; perfis e configurações antigos seguem válidos.

## Requisitos e aceite

- **FR-001** — O catálogo MUST incluir o GPT-6.1 Sol com seus esforços e limites, no plano ChatGPT e em provedores compatíveis.
- **AC-001** — Dado um provedor compatível cujo `/models` lista `gpt-6.1-sol`, quando o usuário testa a conexão, então o modelo aparece como "GPT-6.1 Sol" com contexto 1.050.000, imagens, ferramentas e esforços Low, Medium, High, Extra high e Max; dada a lista do plano sem ele, ele é acrescentado com os mesmos esforços.
- **FR-002** — O usuário MUST definir o esforço por modelo e modo, no seletor e em Configurações.
- **AC-002** — Dado um modelo com esforços, quando o usuário escolhe Low no modo Chat e Max no modo Tarefa, então ao alternar os modos o seletor mostra Low e Max, a preferência persiste após reiniciar e o turno no modo Tarefa envia `reasoning.effort = "max"`.
- **FR-003** — Modelos manuais MUST guardar os esforços aceitos e o padrão.
- **AC-003** — Dado um provedor personalizado, quando o usuário cadastra o modelo `qa-reasoner` com esforços Low, High e Max e padrão High, então o seletor oferece exatamente "Padrão do modelo (alto)", Baixo, Alto e Máximo.

## Leitura e mapa de alterações

- `crates/aura-core/src/model_catalog.rs` → `known_model`, `EFFORTS`; new.
- `crates/aura-codex/src/models.rs` → `parse_plan_models`; existing.
- `crates/aura-gateway/src/registry.rs` → `ModelSpec.efforts/default_effort`, `save_model`; `discovery.rs` → `parse_models`; existing.
- `crates/aura-core/src/settings.rs` → `effort_presets`; existing.
- `apps/desktop/src/overlay/session.ts`, `ModelPicker.tsx`, `settings/Providers.tsx`, `settings/Account.tsx` (tabela); existing.

## Plano breve

Seam: funções puras de catálogo/descoberta/registro/settings (Rust), store da sessão e telas (React), app nativo com provedores loopback. Abordagem: catálogo estático de modelos conhecidos no núcleo; preferências em `Settings.effortPresets` com chave `provedor::modelo` e valores por modo. Dependências: none.

## Sequência e tarefas

- [x] C-001 Catálogo + plano + descoberta (red/green Rust).
- [x] C-002 Esforços em modelos manuais (Rust + React).
- [x] C-003 Preferências por modelo e modo (Rust + React).
- [x] C-004 E2E nativo, regressão e evidência.

## Validação e evidência

Comando/procedimento: `cargo test -p aura-core -p aura-codex -p aura-gateway`, `pnpm -C apps/desktop test`, `pnpm -C apps/desktop/e2e test --spec ./specs/model-efforts.e2e.ts`.

Resultado executado: EV-001 (AC-001..003) passou no release SHA256 `549FE0815780BB94C7C51F661A42D4328208B75F6114C571620B09CA5E82BE3F`; regressão: workspace Rust, clippy, fmt, contrato dourado, UI 146/146, typecheck e specs nativas `model-effort`, `profile-model`, `provider-manual-model`.

Limitações: sem conta ChatGPT no ambiente de QA, o GPT-6.1 Sol no plano é verificado pela função de catálogo, não por um turno real no plano.

## Condição de retorno

Retornar se o contrato, símbolo, dependência ou decisão material não puder ser satisfeito sem ampliar o escopo.

## Estado

`change.md` é a fonte canônica deste esforço compacto. Não crie `tasks.md` ou tickets paralelos para esta mudança.
