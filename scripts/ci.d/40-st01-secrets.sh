#!/usr/bin/env bash
# Contrôle : secrets, chaque commit de la PR, puis justification de .gitleaksignore (RG-ST01-6.4). Contrat : archi ST-01 § CI.
set -uo pipefail
rc=0
gitleaks git --no-banner --no-color --redact --log-opts="$CI_BASE..$CI_HEAD" || rc=1
# Chaque empreinte est précédée d'une ligne « # justification » non vide.
awk '
  /^[[:space:]]*$/ { prev = 0; next }
  /^#/ { prev = ($0 ~ /^#[[:space:]]*[^[:space:]]/); next }
  { if (!prev) { print "40-st01-secrets : empreinte sans justification : " $0 > "/dev/stderr"; bad = 1 } prev = 0 }
  END { exit bad }
' .gitleaksignore || rc=1
exit $rc
