#!/bin/sh
# Reproduces the S4-R numbers. Corpora: crates.io sources already in the local cargo registry (parsed, never built).
set -e
cd "$(dirname "$0")"
cargo build --release -p demo
REG="$HOME/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f"
./target/release/demo check ../../crates rules/openarbitre.txt
./target/release/demo bench ../../crates
for d in tokio-1.54.1 ignore-0.4.23 globset-0.4.16 regex-automata-0.4.18 syn-3.0.9; do
  echo "== $d"
  ./target/release/demo bench "$REG/$d"
done
