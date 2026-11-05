set -uo pipefail
rc=0
cargo fmt --all --check || rc=1
cargo clippy --workspace --all-features --all-targets --locked -- -D warnings || rc=1
# Complexity gate on production code only; threshold in clippy.toml.
cargo clippy --workspace --all-features --lib --bins --locked -- -D clippy::cognitive_complexity || rc=1
exit $rc
