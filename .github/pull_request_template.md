## What and why

## How it was tested
- [ ] `cargo test` / `cargo clippy --all-targets -- -D warnings`
- [ ] `pnpm -C apps/desktop test` / `pnpm -C apps/desktop typecheck`
- [ ] Manual on Windows (describe)

## Checklist
- [ ] IPC shape changes: updated `src/ipc/types.ts` and regenerated the golden (`UPDATE_GOLDEN=1 cargo test -p aura-app --test ipc_contract`)
- [ ] Hybrid ticket/spec updated when behaviour changed
