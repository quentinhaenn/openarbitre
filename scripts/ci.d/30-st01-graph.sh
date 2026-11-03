#!/usr/bin/env bash
# Check: dependency graph, rules (a) to (e) (RG-ST01-5.1, 5.2, 6.3). Contract: ST-01 design, CI.
set -uo pipefail
python3 scripts/lib/graph.py
