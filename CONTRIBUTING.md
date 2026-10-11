# Contributing to Aura

Thanks for helping! Aura is a Windows app, but most of the code is
OS-independent and builds/tests on Linux and macOS too.

## Setup

| Tool | Version |
| --- | --- |
| Rust | 1.98.1 (pinned in `rust-toolchain.toml`) |
| Node.js | 22 LTS + pnpm |
| Windows (for the app) | 10 22H2+/11, WebView2, Visual Studio Build Tools (MSVC) |

```bash
# Everything that is not Windows-specific
cargo test                      # default workspace members
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
pnpm -C apps/desktop install
pnpm -C apps/desktop test       # Vitest + Testing Library
pnpm -C apps/desktop dev        # UI in a browser with the mock backend
```

On Windows, the full app:

```powershell
./scripts/prepare-sidecars.ps1          # builds aura-worker sidecar
pnpm -C apps/desktop tauri dev          # real app
pnpm -C apps/desktop tauri dev --features demo   # no account: fake agent + synthetic OS
```

## Where things live

See `AGENTS.md` (structure, conventions) and `docs/HANDOFF.md` (status per
area, what is verified where). Behaviour is specified in `specs/<effort>/`
(Hybrid Development Kit): change the spec before changing behaviour.

## Pull requests

- One topic per PR; tests first (red → green) at the public seam.
- `clippy -D warnings`, `rustfmt`, `pnpm typecheck` and tests must pass.
- IPC shape changes: update `apps/desktop/src/ipc/types.ts` and regenerate
  the golden with `UPDATE_GOLDEN=1 cargo test -p aura-app --test ipc_contract`.
- UI strings go to both `src/i18n/pt-BR.ts` and `src/i18n/en.ts`.
- Never log or persist secrets; never add non-permissive dependencies.

By contributing you agree that your work is licensed under MIT OR Apache-2.0.
