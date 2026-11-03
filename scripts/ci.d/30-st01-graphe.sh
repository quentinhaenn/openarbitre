#!/usr/bin/env bash
# Contrôle : graphe de dépendances, règles (a) à (e) (RG-ST01-5.1, 5.2, 6.3). Contrat : archi ST-01 § CI.
set -uo pipefail
python3 scripts/lib/graphe.py
