#!/usr/bin/env bash
set -euo pipefail

output_path="${1:-.shipper/auth-evidence.json}"
mkdir -p "$(dirname "$output_path")"

trusted_token="${TRUSTED_TOKEN:-}"
fallback_token="${FALLBACK_TOKEN:-}"
auth_action_outcome="${AUTH_ACTION_OUTCOME:-unknown}"
selected_source="missing"
if [ -n "$trusted_token" ]; then
  selected_source="trusted_publishing"
elif [ -n "$fallback_token" ]; then
  selected_source="fallback_secret"
fi

token_minted=false
fallback_configured=false
fallback_selected=false
if [ -n "$trusted_token" ]; then
  token_minted=true
fi
if [ -n "$fallback_token" ]; then
  fallback_configured=true
fi
if [ "$selected_source" = "fallback_secret" ]; then
  fallback_selected=true
fi

jq -n \
  --arg schema_version "lintdiff.auth-evidence.v1" \
  --arg workflow "${GITHUB_WORKFLOW:-unknown}" \
  --arg job "${GITHUB_JOB:-unknown}" \
  --arg run_id "${GITHUB_RUN_ID:-unknown}" \
  --arg run_attempt "${GITHUB_RUN_ATTEMPT:-unknown}" \
  --arg commit "${RELEASE_COMMIT:-${GITHUB_SHA:-unknown}}" \
  --arg tag "${RELEASE_TAG:-${GITHUB_REF_NAME:-unknown}}" \
  --arg environment "${RELEASE_ENVIRONMENT:-release}" \
  --arg auth_action_outcome "$auth_action_outcome" \
  --arg selected_source "$selected_source" \
  --argjson token_minted "$token_minted" \
  --argjson fallback_configured "$fallback_configured" \
  --argjson fallback_selected "$fallback_selected" \
  '{
    schema_version: $schema_version,
    workflow: $workflow,
    job: $job,
    run_id: $run_id,
    run_attempt: $run_attempt,
    commit: $commit,
    tag: $tag,
    environment: $environment,
    auth_action_outcome: $auth_action_outcome,
    token_minted: $token_minted,
    fallback_configured: $fallback_configured,
    fallback_selected: $fallback_selected,
    selected_source: $selected_source,
    limits: [
      "token values, prefixes, lengths, hashes, and authorization headers are never recorded",
      "Trusted Publishing is not proof that every crate is registered"
    ]
  }' > "$output_path"

echo "selected_source=$selected_source" >> "${GITHUB_OUTPUT:-/dev/null}"
echo "auth_evidence_path=$output_path"

if [ "$selected_source" = "missing" ]; then
  echo "no crates.io credential source is available" >&2
  exit 1
fi
