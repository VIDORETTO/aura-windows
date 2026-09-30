#!/usr/bin/env bash
# Runs cargo with minimum CPU/IO priority and a single job.
# Used on shared machines (e.g. the planning VPS) so builds never starve other services.
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
export CARGO_INCREMENTAL=0
exec nice -n 19 ionice -c3 cargo "$@"
