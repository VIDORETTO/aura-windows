# Performance baseline

`aura-bench` compares a run against `baseline.json` and fails when any metric
is worse than `baseline × 1.2` or above an absolute budget
(`docs/architecture/overview.md` → "Orçamentos de desempenho").

```powershell
# 1. Build and start Aura (release), leave it idle with the Overlay hidden.
pnpm -C apps/desktop tauri build
Start-Process "target\release\aura.exe" -ArgumentList "--background"
$pid = (Get-Process aura).Id

# 2. Measure (5 min idle, 50 hotkey opens) and compare.
cargo run -p aura-bench --release -- all --pid $pid --out bench/latest.json --baseline bench/baseline.json
```

The committed `baseline.json` is a placeholder equal to the budgets. After the
first run on the reference machine, copy `bench/latest.json` over it and commit
it with the machine description.
