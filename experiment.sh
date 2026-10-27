#!/usr/bin/env bash
# DRAFT. Convenience entrypoint; candidate checks stay in orchestrator.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
export PYTHONDONTWRITEBYTECODE=1
tasks=(STATE-INIT)

case "${1:-preview}" in
  preview)
    if (( $# > 1 )); then exit 2; fi
    exec python3 -m orchestrator run-plan --tasks "${tasks[@]}" --dry-run
    ;;
  run)
    if (( $# != 1 )); then exit 2; fi
    exec python3 -m orchestrator run-plan --tasks "${tasks[@]}" --execute-codex
    ;;
  resume)
    if (( $# != 2 )); then echo 'Usage: ./experiment.sh resume RUN_ID' >&2; exit 2; fi
    exec python3 -m orchestrator resume --run-id "$2" --execute-codex
    ;;
  report)
    if (( $# != 2 )); then echo 'Usage: ./experiment.sh report RUN_ID' >&2; exit 2; fi
    exec python3 -m orchestrator report --run-id "$2"
    ;;
  *)
    echo 'Usage: ./experiment.sh [preview | run | resume RUN_ID | report RUN_ID]' >&2
    exit 2
    ;;
esac
