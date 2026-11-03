#!/usr/bin/env bash
# Check: secrets in each PR commit, then justification of .gitleaksignore (RG-ST01-6.4). Contract: ST-01 design, CI.
set -uo pipefail
rc=0
gitleaks git --no-banner --no-color --redact --verbose --log-opts="$CI_BASE..$CI_HEAD" || rc=1
# Each fingerprint is preceded by a non-empty "# justification" line.
awk '
  /^[[:space:]]*$/ { prev = 0; next }
  /^#/ { prev = ($0 ~ /^#[[:space:]]*[^[:space:]]/); next }
  { if (!prev) { print "40-st01-secrets : fingerprint without justification : " $0 > "/dev/stderr"; bad = 1 } prev = 0 }
  END { exit bad }
' .gitleaksignore || rc=1
exit $rc
