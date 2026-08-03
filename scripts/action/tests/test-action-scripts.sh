#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
resolve="$root/resolve-version.sh"
install="$root/install.sh"
run="$root/run.sh"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

fail() {
  printf 'FAIL: %s\n' "$1" >&2
  exit 1
}

assert_failure() {
  if "$@" >/dev/null 2>&1; then
    fail "expected command to fail: $*"
  fi
}

[[ "$("$resolve" v0.1.1 '')" == 'v0.1.1' ]] || fail 'exact tag did not resolve'
[[ "$("$resolve" refs/tags/v0.1.1 '')" == 'v0.1.1' ]] || fail 'refs/tags exact tag did not resolve'
[[ "$("$resolve" feature/ref v0.1.1)" == 'v0.1.1' ]] || fail 'explicit version did not resolve branch ref'
assert_failure "$resolve" v0.1.1 v0.1.2
assert_failure "$resolve" feature/ref ''
assert_failure "$resolve" feature/ref latest
assert_failure "$resolve" feature/ref v0

source_dir="$tmp/source with spaces"
runner_temp="$tmp/runner with spaces"
mkdir -p "$source_dir" "$runner_temp"
cat > "$source_dir/lintdiff" <<'EOF'
#!/usr/bin/env bash
if [[ "${1:-}" == '--version' ]]; then
  printf 'lintdiff 0.1.0\n'
fi
EOF
chmod +x "$source_dir/lintdiff"
archive="$tmp/lintdiff-0.1.0-x86_64-unknown-linux-gnu.tar.gz"
tar -czf "$archive" -C "$source_dir" lintdiff
sha256sum "$archive" > "$archive.sha256"
installed="$(RUNNER_TEMP="$runner_temp" LINTDIFF_TARGET='x86_64-unknown-linux-gnu' LINTDIFF_EXTENSION='tar.gz' LINTDIFF_ARCHIVE_URL="file://$archive" LINTDIFF_CHECKSUM_URL="file://$archive.sha256" "$install" --version v0.1.0)"
[[ -x "$installed" ]] || fail 'root-level archive did not install an executable'

printf '0000000000000000000000000000000000000000000000000000000000000000  %s\n' "$archive" > "$archive.sha256"
assert_failure env RUNNER_TEMP="$runner_temp" LINTDIFF_TARGET='x86_64-unknown-linux-gnu' LINTDIFF_EXTENSION='tar.gz' LINTDIFF_ARCHIVE_URL="file://$archive" LINTDIFF_CHECKSUM_URL="file://$archive.sha256" "$install" --version v0.1.0

args_log="$tmp/args.log"
fake="$tmp/fake lintdiff"
cat > "$fake" <<EOF
#!/usr/bin/env bash
printf '<%s>\n' "\$@" > "$args_log"
EOF
chmod +x "$fake"
LINTDIFF_BIN="$fake" "$run" ci github --base 'base ref' --out "$tmp/report path.json"
grep -Fx '<base ref>' "$args_log" >/dev/null || fail 'argument with spaces was changed'
grep -Fx "<$tmp/report path.json>" "$args_log" >/dev/null || fail 'path with spaces was changed'

printf '%s\n' 'action script tests passed'
