#!/usr/bin/env bash
# Check: Rust and Python tests (RG-ST01-6.1). Contract: ST-01 design, CI.
set -uo pipefail
rc=0
cargo test --workspace --all-features --locked || rc=1
python3 -m unittest scripts/lib/test_ci.py || rc=1
exit $rc
