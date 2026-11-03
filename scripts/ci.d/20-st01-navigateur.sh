#!/usr/bin/env bash
# Contrôle : liste navigateur compilée en wasm32 (RG-ST01-6.2). Contrat : archi ST-01 § CI.
set -uo pipefail
membres=$(cargo metadata --format-version 1 --locked --no-deps |
  python3 -c 'import json,sys; print(" ".join(p["name"] for p in json.load(sys.stdin)["packages"]))') || exit 1
# arbitre-core toujours passé : absent ⇒ cargo check rouge (AMB-ST01-5).
presents=(-p arbitre-core)
case " $membres " in *" arbitre-plan "*) presents+=(-p arbitre-plan) ;; esac
cargo check --locked --target wasm32-unknown-unknown "${presents[@]}"
