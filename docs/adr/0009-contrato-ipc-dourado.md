---
status: accepted
---

# Contrato de IPC por arquivo dourado em vez de tauri-specta

Os tipos TypeScript do IPC (`apps/desktop/src/ipc/types.ts`) são escritos à mão e travados por um contrato dourado: `crates/aura-app/tests/ipc_contract.rs` serializa um exemplar de cada formato que cruza a fronteira (eventos, chips, políticas, decisões, specs MCP…) em `apps/desktop/src/ipc/__fixtures__/contract.json`, e `src/ipc/contract.test.ts` passa esse mesmo arquivo pelos tipos e redutores da UI. Qualquer mudança de formato quebra um dos dois lados.

## Considered options

- `tauri-specta` (planejado): gera bindings a partir dos comandos, mas acopla os crates de domínio ao ecossistema specta e não é verificável sem compilar o shell Tauri — que nesta fase não compila fora do Windows.
- `ts-rs`: gera tipos, mas não cobre o comportamento (redutores) nem o formato real serializado.

## Consequences

- Toda mudança de formato exige atualizar `types.ts` e regenerar o dourado (`UPDATE_GOLDEN=1 cargo test -p aura-app --test ipc_contract`).
- Enums Rust usam `rename_all_fields = "camelCase"` para que campos de variantes também cheguem em camelCase.
- Os nomes de comandos são mantidos em sincronia manualmente entre `src-tauri/src/commands.rs` e `src/ipc/commands.ts` (o mock `src/ipc/mock.ts` falha em testes para comandos desconhecidos).
