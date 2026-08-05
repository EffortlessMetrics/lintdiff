#!/usr/bin/env bash
set -euo pipefail

state_dir="${1:-.shipper}"
state_path="$state_dir/state.json"
plan_path="$state_dir/plan.json"

test -s "$state_path" || {
  echo "missing Shipper state: $state_path" >&2
  exit 1
}
test -s "$plan_path" || {
  echo "missing Shipper plan: $plan_path" >&2
  exit 1
}

state_plan_id="$(jq -er '.plan_id | strings | select(length > 0)' "$state_path")"
plan_plan_id="$(jq -er '.plan_id | strings | select(length > 0)' "$plan_path")"
test "$state_plan_id" = "$plan_plan_id" || {
  echo "Shipper state and plan IDs differ" >&2
  exit 1
}

state_registry="$(jq -er '.registry.name | strings | select(length > 0)' "$state_path")"
plan_registry="$(jq -er '.registry.name | strings | select(length > 0)' "$plan_path")"
test "$state_registry" = "crates-io"
test "$plan_registry" = "crates-io"

echo "shipper_state_check=pass plan_id=$state_plan_id registry=crates-io"
