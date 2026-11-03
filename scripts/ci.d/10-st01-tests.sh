#!/usr/bin/env bash
# Contrôle : tests Rust et Python (RG-ST01-6.1). Contrat : archi ST-01 § CI.
set -uo pipefail
rc=0
cargo test --workspace --all-features --locked || rc=1
python3 -m unittest scripts/lib/test_ci.py || rc=1
exit $rc
