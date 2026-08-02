#!/usr/bin/env bash
set -euo pipefail

action_ref="${1:-${ACTION_REF:-}}"
explicit_version="${2:-${INPUT_VERSION:-}}"

normalize_version() {
  local value="$1"
  value="${value#refs/tags/}"
  if [[ "$value" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    printf '%s\n' "$value"
    return 0
  fi
  if [[ "$value" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    printf 'v%s\n' "$value"
    return 0
  fi
  return 1
}

if [[ -n "$explicit_version" ]]; then
  if ! resolved_version="$(normalize_version "$explicit_version")"; then
    printf 'invalid explicit lintdiff version: %s\n' "$explicit_version" >&2
    exit 2
  fi

  if [[ -n "$action_ref" ]]; then
    normalized_ref="${action_ref#refs/tags/}"
    if [[ "$normalized_ref" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] && [[ "$normalized_ref" != "$resolved_version" ]]; then
      printf 'explicit version %s does not match exact Action tag %s\n' "$resolved_version" "$normalized_ref" >&2
      exit 2
    fi
  fi

  printf '%s\n' "$resolved_version"
  exit 0
fi

if [[ -n "$action_ref" ]]; then
  if resolved_version="$(normalize_version "$action_ref")"; then
    printf '%s\n' "$resolved_version"
    exit 0
  fi
fi

printf 'an exact vX.Y.Z Action tag or an explicit version is required; ref was: %s\n' "${action_ref:-<empty>}" >&2
exit 2
