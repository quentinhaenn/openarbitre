#!/usr/bin/env bash
# Point d'entrée unique de la vérification : exécute scripts/ci.d/*.sh (archi ST-01 § CI, T-10).
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1
# shellcheck source=scripts/outils.env
. scripts/outils.env

# Pas de .pyc : un cache périmé fausserait les aides Python.
export PYTHONDONTWRITEBYTECODE=1
actions=false
[ "${GITHUB_ACTIONS:-}" = true ] && actions=true
rouge=0
resume=()

echec() { # message
  echo "ci : rouge : $1" >&2
  $actions && echo "::error::$1"
  rouge=1
}

# Versions : égalité exacte, sauf Python (minimum).
if ! command -v cargo-mutants >/dev/null 2>&1; then
  echec "cargo-mutants absent du PATH. Installer : cargo install --locked cargo-mutants@${CARGO_MUTANTS_VERSION}"
elif [ "$(cargo mutants --version | awk '{print $2}')" != "$CARGO_MUTANTS_VERSION" ]; then
  echec "cargo-mutants $(cargo mutants --version | awk '{print $2}') au lieu de ${CARGO_MUTANTS_VERSION}. Installer : cargo install --locked --force cargo-mutants@${CARGO_MUTANTS_VERSION}"
fi
if ! command -v gitleaks >/dev/null 2>&1; then
  echec "gitleaks absent du PATH. Installer la version ${GITLEAKS_VERSION} (https://github.com/gitleaks/gitleaks/releases/tag/v${GITLEAKS_VERSION})"
elif [ "$(gitleaks version | tr -d 'v ')" != "$GITLEAKS_VERSION" ]; then
  echec "gitleaks $(gitleaks version) au lieu de ${GITLEAKS_VERSION}. Installer la version ${GITLEAKS_VERSION} (https://github.com/gitleaks/gitleaks/releases/tag/v${GITLEAKS_VERSION})"
fi
if ! python3 -c "import sys; sys.exit(0 if sys.version_info >= tuple(map(int, '${PYTHON_MIN}'.split('.'))) else 1)" 2>/dev/null; then
  echec "python3 < ${PYTHON_MIN} (le python3 d'Xcode est en 3.9) : installer Python ${PYTHON_MIN} ou plus, par exemple brew install python@${PYTHON_MIN}"
fi

# Base du diff.
if [ -z "${CI_BASE:-}" ]; then
  CI_BASE=$(git merge-base HEAD origin/main 2>/dev/null) || {
    echec "base introuvable : fournir CI_BASE ou une branche origin/main"
    CI_BASE=HEAD
  }
fi
CI_HEAD=$(git rev-parse HEAD)
CI_OUT="$PWD/target/ci"
rm -rf "$CI_OUT"
mkdir -p "$CI_OUT"
export CI_BASE CI_HEAD CI_OUT

controles=()
for f in scripts/ci.d/*.sh; do
  [ -e "$f" ] && controles+=("$f")
done
if [ ${#controles[@]} -eq 0 ]; then
  echec "scripts/ci.d/ est vide"
else
  # LC_ALL=C : ordre indépendant de la locale.
  sorted=$(printf '%s\n' "${controles[@]}" | LC_ALL=C sort)
  while IFS= read -r f; do
    nom=$(basename "$f" .sh)
    $actions && echo "::group::$nom"
    echo "ci : $nom"
    debut=$SECONDS
    bash "$f"
    rc=$?
    $actions && echo "::endgroup::"
    if [ $rc -ne 0 ]; then
      echec "$nom : code $rc"
      resume+=("| $nom | rouge | $((SECONDS - debut)) s |")
    else
      resume+=("| $nom | vert | $((SECONDS - debut)) s |")
    fi
  done <<<"$sorted"
fi

if $actions && [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
  {
    echo "| contrôle | verdict | durée |"
    echo "|---|---|---|"
    printf '%s\n' "${resume[@]}"
  } >>"$GITHUB_STEP_SUMMARY"
fi

if [ $rouge -ne 0 ]; then
  echo "ci : ROUGE" >&2
  exit 1
fi
echo "ci : vert"
