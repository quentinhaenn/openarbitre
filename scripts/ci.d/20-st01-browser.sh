#!/usr/bin/env bash
# Check: browser list compiled to wasm32 (RG-ST01-6.2). Contract: ST-01 design, CI.
set -uo pipefail
members=$(cargo metadata --format-version 1 --locked --no-deps |
  python3 -c 'import json,sys; print(" ".join(p["name"] for p in json.load(sys.stdin)["packages"]))') || exit 1
# arbitre-core always passed: if absent, cargo check fails (AMB-ST01-5).
present=(-p arbitre-core)
case " $members " in *" arbitre-plan "*) present+=(-p arbitre-plan) ;; esac
cargo check --locked --target wasm32-unknown-unknown "${present[@]}"
