#!/usr/bin/env bash
# Check: mutation via scripts/lib/mutation.py, exclusion channels (RG-ST01-7.x, A-7). Contract: ST-01 design, CI.
set -uo pipefail
rc=0

# A-7: exclusion channels forbidden (--exclude exists only on this command line).
if git grep -l -e 'mutants::skip' -- 'crates/**/*.rs'; then
  echo "50-st01-mutation : mutants::skip forbidden (files above)" >&2
  rc=1
fi
if git ls-files | grep -E '(^|/)mutants\.toml$'; then
  echo "50-st01-mutation : mutants.toml forbidden (files above)" >&2
  rc=1
fi

git diff "$CI_BASE...$CI_HEAD" >"$CI_OUT/diff.patch"
if [ -s "$CI_OUT/diff.patch" ]; then
  python3 scripts/lib/mutation.py --gen-files >"$CI_OUT/gen.txt" || exit 1
  mapfile -t gen <"$CI_OUT/gen.txt"
  excludes=()
  for f in "${gen[@]}"; do [ -n "$f" ] && excludes+=(--exclude "$f"); done
  cargo mutants --in-diff "$CI_OUT/diff.patch" --all-features --jobs 3 "${excludes[@]}" --output "$CI_OUT"
  code=$?
  # 0 = all caught, 2 = missed mutants, 3 = timeouts, 4 = red baseline: the verdict comes from outcomes.json.
  case $code in
  0 | 2 | 3 | 4) ;;
  *)
    echo "50-st01-mutation : cargo-mutants failed (code $code)" >&2
    exit 1
    ;;
  esac
  # Code 0 without outcomes.json: the diff touches no Rust source ("Diff changes no Rust source files"), neutral.
  if [ $code -ne 0 ] && [ ! -f "$CI_OUT/mutants.out/outcomes.json" ]; then
    echo "50-st01-mutation : outcomes.json missing (code $code)" >&2
    exit 1
  fi
fi
python3 scripts/lib/mutation.py || rc=1
exit $rc
