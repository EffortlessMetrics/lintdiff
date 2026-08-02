#!/usr/bin/env bash
set -euo pipefail

version=""
while (($# > 0)); do
  case "$1" in
    --version)
      if (($# < 2)); then
        printf '%s\n' '--version requires a value' >&2
        exit 2
      fi
      version="$2"
      shift 2
      ;;
    *)
      printf 'unknown install option: %s\n' "$1" >&2
      exit 2
      ;;
  esac
done

if [[ ! "$version" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  printf 'install requires a strict vX.Y.Z version, got: %s\n' "$version" >&2
  exit 2
fi
if [[ -z "${RUNNER_TEMP:-}" ]]; then
  printf '%s\n' 'RUNNER_TEMP is required for release downloads' >&2
  exit 2
fi

target="${LINTDIFF_TARGET:-}"
extension="${LINTDIFF_EXTENSION:-}"
if [[ -z "$target" || -z "$extension" ]]; then
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  arch="$(uname -m)"
  case "$os:$arch" in
    linux:x86_64) target='x86_64-unknown-linux-gnu'; extension='tar.gz' ;;
    darwin:x86_64) target='x86_64-apple-darwin'; extension='tar.gz' ;;
    darwin:arm64|darwin:aarch64) target='aarch64-apple-darwin'; extension='tar.gz' ;;
    mingw*:x86_64|msys*:x86_64|cygwin*:x86_64) target='x86_64-pc-windows-msvc'; extension='zip' ;;
    *) printf 'unsupported platform: %s/%s\n' "$os" "$arch" >&2; exit 1 ;;
  esac
fi

archive_name="lintdiff-${version#v}-${target}.${extension}"
archive_url="${LINTDIFF_ARCHIVE_URL:-https://github.com/EffortlessMetrics/lintdiff/releases/download/${version}/${archive_name}}"
checksum_url="${LINTDIFF_CHECKSUM_URL:-${archive_url}.sha256}"
download_dir="$RUNNER_TEMP/lintdiff-download-${version#v}-${target}"
install_dir="$RUNNER_TEMP/lintdiff-install-${version#v}-${target}"
archive_path="$download_dir/$archive_name"
checksum_path="$archive_path.sha256"
rm -rf "$download_dir" "$install_dir"
mkdir -p "$download_dir" "$install_dir"

printf 'Downloading lintdiff %s for %s\n' "$version" "$target" >&2
curl --fail --silent --show-error --location "$archive_url" --output "$archive_path"
curl --fail --silent --show-error --location "$checksum_url" --output "$checksum_path"

read -r expected_checksum _ < "$checksum_path" || true
if [[ ! "$expected_checksum" =~ ^[[:xdigit:]]{64}$ ]]; then
  printf 'checksum file does not contain a SHA-256 digest: %s\n' "$checksum_url" >&2
  exit 1
fi
if command -v sha256sum >/dev/null 2>&1; then
  actual_checksum="$(sha256sum "$archive_path" | awk '{print $1}')"
else
  actual_checksum="$(shasum -a 256 "$archive_path" | awk '{print $1}')"
fi
if [[ "$actual_checksum" != "$expected_checksum" ]]; then
  printf 'checksum mismatch for %s\n' "$archive_name" >&2
  exit 1
fi

if [[ "$extension" == 'tar.gz' ]]; then
  tar -xzf "$archive_path" -C "$install_dir"
else
  unzip -q "$archive_path" -d "$install_dir"
fi

binary="$install_dir/lintdiff"
if [[ "$extension" == 'zip' && -f "$install_dir/lintdiff.exe" ]]; then
  binary="$install_dir/lintdiff.exe"
fi
if [[ ! -f "$binary" ]]; then
  printf 'release archive did not contain the expected root-level lintdiff binary\n' >&2
  exit 1
fi
chmod +x "$binary" 2>/dev/null || true

version_output="$("$binary" --version 2>&1)"
expected_version="${version#v}"
if [[ "$version_output" != *"lintdiff $expected_version"* ]]; then
  printf 'installed binary reported an unexpected version: %s\n' "$version_output" >&2
  exit 1
fi

printf '%s\n' "$binary"
