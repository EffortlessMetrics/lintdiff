#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
validator="$script_dir/../validate-shipper-state.sh"
fixture_dir="$(mktemp -d)"
trap 'rm -rf "$fixture_dir"' EXIT

expect_failure() {
  if "$@" >/dev/null 2>&1; then
    echo "expected failure: $*" >&2
    exit 1
  fi
}

expect_failure bash "$validator" "$fixture_dir/missing"

mkdir -p "$fixture_dir/state"
printf '{"plan_id":"plan-a","registry":{"name":"crates-io"}}\n' \
  > "$fixture_dir/state/plan.json"
expect_failure bash "$validator" "$fixture_dir/state"

printf '{"plan_id":"plan-b","registry":{"name":"crates-io"}}\n' \
  > "$fixture_dir/state/state.json"
expect_failure bash "$validator" "$fixture_dir/state"

printf '{"plan_id":"plan-a","registry":{"name":"other-registry"}}\n' \
  > "$fixture_dir/state/state.json"
expect_failure bash "$validator" "$fixture_dir/state"

printf '{"plan_id":"plan-a","registry":{"name":"crates-io"}}\n' \
  > "$fixture_dir/state/state.json"
bash "$validator" "$fixture_dir/state" >/dev/null

echo "shipper_state_fixture_check=pass missing_state=fail wrong_plan_id=fail wrong_registry=fail valid=pass"
