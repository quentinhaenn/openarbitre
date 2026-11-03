#!/usr/bin/env bash
# Single verification entry point: runs scripts/ci.d/*.sh (ST-01 design, CI; T-10).
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1
# shellcheck source=scripts/tools.env
. scripts/tools.env

# No .pyc: a stale cache would skew the Python helpers.
export PYTHONDONTWRITEBYTECODE=1
actions=false
[ "${GITHUB_ACTIONS:-}" = true ] && actions=true
red=0
summary=()

fail() { # message
  echo "ci: red: $1" >&2
  $actions && echo "::error::$1"
  red=1
}

# Versions: exact match, except Python (minimum).
if ! command -v cargo-mutants >/dev/null 2>&1; then
  fail "cargo-mutants not on PATH. Install: cargo install --locked cargo-mutants@${CARGO_MUTANTS_VERSION}"
elif [ "$(cargo mutants --version | awk '{print $2}')" != "$CARGO_MUTANTS_VERSION" ]; then
  fail "cargo-mutants $(cargo mutants --version | awk '{print $2}') instead of ${CARGO_MUTANTS_VERSION}. Install: cargo install --locked --force cargo-mutants@${CARGO_MUTANTS_VERSION}"
fi
if ! command -v gitleaks >/dev/null 2>&1; then
  fail "gitleaks not on PATH. Install version ${GITLEAKS_VERSION} (https://github.com/gitleaks/gitleaks/releases/tag/v${GITLEAKS_VERSION})"
elif [ "$(gitleaks version | tr -d 'v ')" != "$GITLEAKS_VERSION" ]; then
  fail "gitleaks $(gitleaks version) instead of ${GITLEAKS_VERSION}. Install version ${GITLEAKS_VERSION} (https://github.com/gitleaks/gitleaks/releases/tag/v${GITLEAKS_VERSION})"
fi
if ! python3 -c "import sys; sys.exit(0 if sys.version_info >= tuple(map(int, '${PYTHON_MIN}'.split('.'))) else 1)" 2>/dev/null; then
  fail "python3 < ${PYTHON_MIN} (Xcode's python3 is 3.9): install Python ${PYTHON_MIN} or later, e.g. brew install python@${PYTHON_MIN}"
fi

# Diff base.
if [ -z "${CI_BASE:-}" ]; then
  CI_BASE=$(git merge-base HEAD origin/main 2>/dev/null) || {
    fail "base not found: provide CI_BASE or an origin/main branch"
    CI_BASE=HEAD
  }
fi
CI_HEAD=$(git rev-parse HEAD)
CI_OUT="$PWD/target/ci"
rm -rf "$CI_OUT"
mkdir -p "$CI_OUT"
export CI_BASE CI_HEAD CI_OUT

# LC_COLLATE=C: glob sorted bytewise, independent of the locale.
shopt -s nullglob
LC_COLLATE=C
checks=(scripts/ci.d/*.sh)
if [ ${#checks[@]} -eq 0 ]; then
  fail "scripts/ci.d/ is empty"
else
  for f in "${checks[@]}"; do
    name=$(basename "$f" .sh)
    $actions && echo "::group::$name"
    echo "ci: $name"
    start=$SECONDS
    bash "$f"
    rc=$?
    $actions && echo "::endgroup::"
    if [ $rc -ne 0 ]; then
      fail "$name: code $rc"
      summary+=("| $name | red | $((SECONDS - start)) s |")
    else
      summary+=("| $name | green | $((SECONDS - start)) s |")
    fi
  done
fi

if $actions && [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
  {
    echo "| check | verdict | duration |"
    echo "|---|---|---|"
    printf '%s\n' "${summary[@]}"
  } >>"$GITHUB_STEP_SUMMARY"
fi

if [ $red -ne 0 ]; then
  echo "ci: RED" >&2
  exit 1
fi
echo "ci: green"
