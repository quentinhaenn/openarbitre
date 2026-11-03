#!/usr/bin/env bash
# Contrôle : mutation via scripts/lib/mutation.py, canaux d'exclusion (RG-ST01-7.x, A-7). Contrat : archi ST-01 § CI.
set -uo pipefail
rc=0

# A-7 : canaux d'exclusion interdits (--exclude n'existe que dans cette ligne de commande).
if git grep -l -e 'mutants::skip' -- 'crates/**/*.rs'; then
  echo "50-st01-mutation : mutants::skip interdit (fichiers ci-dessus)" >&2
  rc=1
fi
if git ls-files | grep -E '(^|/)mutants\.toml$'; then
  echo "50-st01-mutation : mutants.toml interdit (fichiers ci-dessus)" >&2
  rc=1
fi

mkdir -p "$CI_OUT"
rm -rf "$CI_OUT"/mutants.out "$CI_OUT"/mutants.out.old
git diff "$CI_BASE...$CI_HEAD" >"$CI_OUT/diff.patch"
if [ -s "$CI_OUT/diff.patch" ]; then
  if ! command -v cargo-mutants >/dev/null 2>&1; then
    echo "50-st01-mutation : cargo-mutants absent du PATH" >&2
    exit 1
  fi
  cargo mutants --in-diff "$CI_OUT/diff.patch" --all-features --output "$CI_OUT"
  code=$?
  # 0 = tout détecté, 2 = mutants manqués, 3 = timeouts, 4 = base rouge : le verdict vient d'outcomes.json.
  case $code in
  0 | 2 | 3 | 4) ;;
  *)
    echo "50-st01-mutation : cargo-mutants a échoué (code $code)" >&2
    exit 1
    ;;
  esac
  # Code 0 sans outcomes.json : le diff ne touche aucune source Rust (« Diff changes no Rust source files »), neutre.
  if [ $code -ne 0 ] && [ ! -f "$CI_OUT/mutants.out/outcomes.json" ]; then
    echo "50-st01-mutation : outcomes.json absent (code $code)" >&2
    exit 1
  fi
fi
python3 scripts/lib/mutation.py || rc=1
exit $rc
