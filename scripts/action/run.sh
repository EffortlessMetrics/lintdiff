#!/usr/bin/env bash
set -euo pipefail

if (($# == 0)); then
  printf '%s\n' 'run.sh requires a lintdiff argument list' >&2
  exit 2
fi

lintdiff_command="${LINTDIFF_BIN:-lintdiff}"
exec "$lintdiff_command" "$@"
