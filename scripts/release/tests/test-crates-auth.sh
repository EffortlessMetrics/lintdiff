#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
selector="$script_dir/../select-crates-auth.sh"
fixture_dir="$(mktemp -d)"
trap 'rm -rf "$fixture_dir"' EXIT

run_case() {
  local name="$1"
  local trusted="$2"
  local fallback="$3"
  local expected_source="$4"
  local path="$fixture_dir/$name.json"
  env \
    TRUSTED_TOKEN="$trusted" \
    FALLBACK_TOKEN="$fallback" \
    AUTH_ACTION_OUTCOME="success" \
    RELEASE_COMMIT="commit-$name" \
    RELEASE_TAG="tag-$name" \
    bash "$selector" "$path" >/dev/null
  test "$(jq -r '.selected_source' "$path")" = "$expected_source"
  test "$(jq -r '.schema_version' "$path")" = "lintdiff.auth-evidence.v1"
  test "$(jq -r '.limits | length' "$path")" = "2"
}

run_case trusted trusted-token fallback-token trusted_publishing
run_case fallback "" fallback-token fallback_secret

missing_path="$fixture_dir/missing.json"
if env -u TRUSTED_TOKEN -u FALLBACK_TOKEN AUTH_ACTION_OUTCOME=failure \
  bash "$selector" "$missing_path" >/dev/null 2>&1; then
  echo "missing credential source unexpectedly passed" >&2
  exit 1
fi
test "$(jq -r '.selected_source' "$missing_path")" = "missing"
test "$(jq -r '.fallback_configured' "$missing_path")" = "false"

echo "crates_auth_fixture_check=pass trusted=fallback-safe fallback=pass missing=fail"
