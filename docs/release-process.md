# Release Process and Version Pinning

This document describes the automated release process for lintdiff, including version numbering strategy, creating releases, version pinning for users, and rollback procedures.

## Table of Contents

- [Release Workflow Overview](#release-workflow-overview)
- [Version Numbering Strategy](#version-numbering-strategy)
- [Creating a Release](#creating-a-release)
- [Version Pinning for Users](#version-pinning-for-users)
- [Rollback Procedures](#rollback-procedures)
- [Changelog Management](#changelog-management)

---

## Release Workflow Overview

lintdiff uses an automated, tag-driven release workflow defined in [`.github/workflows/release.yml`](../.github/workflows/release.yml). It builds binaries, publishes the four-crate registry closure through Shipper, and creates the GitHub Release only after registry publication succeeds.

## Current v0.1.1 support boundary

`v0.1.1` is released and its exact-tag Action canary passed. The maintained
product is the changed-line receipt workflow (`ingest`, `run`, `ci github`,
the exact-tag Action, and `lintdiff.report.v1`).

The same binary ships `inventory` and `compare` as experimental, advisory
research commands. They emit `lintdiff.inventory.v1` and `lintdiff.delta.v1`
from caller-supplied evidence, but do not build base/head revisions and do not
provide an externally validated strict blocking or causal-detection promise.
See [the product contract](../PRODUCT.md) and the
[external-verdict memo](../plans/diagnostic-delta-external-verdict-2026-08.md).

### Trigger Conditions

The release workflow is triggered only by:

1. **Tag Push**: Pushing an annotated tag matching `v*.*.*` (e.g., `v0.1.1`, `v1.0.0-beta.1`)

### Workflow Jobs

The release workflow consists of five jobs:

| Job | Purpose |
|-----|---------|
| `prepare` | Validates the annotated tag and determines the release version |
| `build` | Builds release binaries for all target platforms |
| `checksums` | Generates combined SHA256 checksums file |
| `publish-crates-io` | Runs the pinned Shipper plan, preflight, and publish stages |
| `release` | Creates GitHub Release with binaries, checksums, and Shipper state |

### Produced Artifacts

For each release, the following artifacts are produced:

| Platform | Target Triple | Archive Format |
|----------|---------------|----------------|
| Linux (x86_64) | `x86_64-unknown-linux-gnu` | `.tar.gz` |
| macOS (Intel) | `x86_64-apple-darwin` | `.tar.gz` |
| macOS (Apple Silicon) | `aarch64-apple-darwin` | `.tar.gz` |
| Windows (x86_64) | `x86_64-pc-windows-msvc` | `.zip` |

Each archive contains a single `lintdiff` (or `lintdiff.exe` on Windows) binary.

Additionally:
- Individual SHA256 checksums for each archive (`.sha256` files)
- Combined `checksums-{version}.txt` file containing all checksums
- Final Shipper state (`shipper-release-state.tar.gz`) containing publication evidence

### Release Notes

The release notes are automatically extracted from [`CHANGELOG.md`](../CHANGELOG.md). The workflow looks for a section starting with `## [{version}]` and includes everything up to the next version header.

---

## Version Numbering Strategy

lintdiff follows [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html).

### Version Format

```
MAJOR.MINOR.PATCH[-PRERELEASE]
```

Examples:
- `0.1.0` - Initial release
- `0.2.0` - New features, backward compatible
- `1.0.0` - First stable release
- `1.1.0-beta.1` - Pre-release version

### When to Bump Each Component

| Component | When to Bump | Example |
|-----------|--------------|---------|
| **MAJOR** | Breaking changes to CLI arguments, output format, or GitHub Action inputs | `0.1.0` → `1.0.0` |
| **MINOR** | New features, new CLI subcommands, new output fields (backward compatible) | `0.1.0` → `0.2.0` |
| **PATCH** | Bug fixes, documentation updates, internal improvements | `0.1.0` → `0.1.1` |

### Pre-release Versions

Pre-release versions use the following suffixes:

| Suffix | Purpose | Example |
|--------|---------|---------|
| `-alpha.N` | Early testing, internal review | `v1.0.0-alpha.1` |
| `-beta.N` | Feature complete, public testing | `v1.0.0-beta.1` |
| `-rc.N` | Release candidate, final testing | `v1.0.0-rc.1` |

Pre-release versions are automatically marked as "pre-release" on GitHub (the workflow detects the `-` in the version string).

### Version 0.x Considerations

While in version `0.x`:
- Minor version bumps may include breaking changes
- Clearly document any breaking changes in CHANGELOG.md
- Aim for stability as you approach `1.0.0`

---

## Creating a Release

### Pre-release Checklist

Before creating a release, ensure:

- [ ] All tests pass on `main` branch
- [ ] CHANGELOG.md is updated with the new version section
- [ ] Version number is updated in all `Cargo.toml` files (workspace and crates)
- [ ] Documentation is updated for any new features
- [ ] Breaking changes are clearly documented
- [ ] Any new dependencies are properly licensed
- [ ] `cargo run -p xtask -- package-check` passes for all four registry packages
- [ ] `cargo semver-checks -p lintdiff-types` passes against the published baseline
- [ ] The Shipper plan, preflight, publication evidence, and clean registry-consumer proof are ready

### Step-by-Step Guide

#### 1. Update Version Numbers

Update the version in the workspace `Cargo.toml`:

```toml
[workspace.package]
version = "0.1.2"  # Update this for the coordinated registry closure
```

The workspace version is inherited by all crates. Verify individual crate `Cargo.toml` files use workspace inheritance:

```toml
[package]
version.workspace = true
```

#### 2. Update CHANGELOG.md

Add a new section for the release:

```markdown
## [0.2.0] - 2026-03-20

### Added
- New `--json-output` flag for machine-readable output
- Support for custom diagnostic formats

### Changed
- Improved error messages for invalid configuration

### Fixed
- Span matching now handles multi-line diagnostics correctly
```

Move any unreleased items from the `[Unreleased]` section to the new version section.

#### 3. Prepare and approve the release commit

```bash
git add -A
git commit -m "chore(release): prepare v0.1.2"
git push origin release/v0.1.2
```

Open a release PR from the prepared branch and run the full release gates. Tagging,
publishing, and release creation require explicit authorization after the exact
release commit and asset plan have been reviewed.

#### 4. Create and Push the Tag after authorization

```bash
RELEASE_COMMIT="<approved squash merge commit SHA>"
git fetch origin main
test "$(git rev-parse origin/main)" = "$RELEASE_COMMIT"
git tag v0.1.2 "$RELEASE_COMMIT"
git push origin v0.1.2
```

#### 5. Monitor the Workflow

1. Go to the [Actions tab](https://github.com/effortless-metrics/lintdiff/actions) in GitHub
2. Find the "Release" workflow run for your tag
3. Monitor `prepare`, `build`, `checksums`, `publish-crates-io`, and `release`

#### 6. Verify the Release

Once the workflow completes:

1. Go to [Releases](https://github.com/EffortlessMetrics/lintdiff/releases)
2. Verify the release appears with correct version
3. Download and test binaries on at least one platform:

   ```bash
   # Linux/macOS
   curl -fsSL -O https://github.com/EffortlessMetrics/lintdiff/releases/download/v0.1.1/lintdiff-0.1.1-x86_64-unknown-linux-gnu.tar.gz
   curl -fsSL -O https://github.com/EffortlessMetrics/lintdiff/releases/download/v0.1.1/checksums-0.1.1.txt
   sha256sum -c --ignore-missing checksums-0.1.1.txt
   tar xzf lintdiff-0.1.1-x86_64-unknown-linux-gnu.tar.gz
   ./lintdiff --version
   ```

4. Verify checksums match:

   ```bash
   sha256sum lintdiff-0.1.1-x86_64-unknown-linux-gnu.tar.gz
   # Compare with checksums-0.1.1.txt
   ```

### v0.1.1 Post-release canary

After the exact-tag Action canary confirms the installed executable reports
`lintdiff 0.1.1`, run the receipt checks below against the published tag:

- ordinary repository paths still produce a valid `lintdiff.report.v1`;
- a repository containing real top-level `a/` and `b/` directories preserves
  those path components in diagnostics and changed-file matching;
- quoted and space-containing Git paths, including a rename record, produce
  the expected repository-relative paths;
- a failed upstream Cargo run still leaves a receipt with its exact exit code,
  completion evidence, and schema-valid output;
- each downloaded archive matches its companion checksum, and the root-level
  Unix archive extracts the executable without stripping its only path entry.

The release also contains the experimental `inventory` and `compare` commands.
Their evidence protocols remain advisory research surfaces; do not represent
their `new` or `resolved` results as generally reliable blocking decisions.

### v0.1.2 crates.io publication closure

The `0.1.2` publication target includes the four runtime packages. `xtask` is
repository tooling and remains private:

1. `lintdiff-types`
2. `lintdiff-engine`
3. `lintdiff-render`
4. `lintdiff`

The tag-triggered workflow delegates registry execution to the pinned Shipper
release engine. Shipper derives the dependency order from the workspace and
persists its plan, preflight result, publish state, receipts, readiness checks,
and resumable execution state under `.shipper/`. The final product proof must
run from a clean consumer context:

```bash
cargo install lintdiff --registry crates-io --version 0.1.2 --locked
lintdiff --version
```

Then run a real changed-line receipt using the installed executable and validate
the resulting `lintdiff.report.v1` artifact. The engine, types, and renderer must
also resolve from crates.io in temporary consumers without path dependencies or
patch overrides. This section is a preparation contract; it does not claim that
the packages are already published.

The workflow runs the following pinned commands from the exact annotated tag:

```bash
cargo install shipper --version 0.4.0 --locked
shipper --version
shipper plan --registry crates-io --state-dir .shipper --format json
shipper preflight --registry crates-io --state-dir .shipper --policy safe --format json
shipper publish --registry crates-io --state-dir .shipper \
  --policy safe --verify-mode package --readiness-method both \
  --readiness-timeout 15m --verify-timeout 10m --max-attempts 12 \
  --base-delay 10s --max-delay 15m --retry-strategy exponential \
  --format json
```

The release job is downstream of `publish-crates-io`, so GitHub Release
creation cannot run until Shipper completes the four-crate registry closure.
Plan and preflight evidence are uploaded as hidden-file-inclusive workflow
artifacts. The final `.shipper` state is also bundled as
`shipper-release-state.tar.gz` and attached to the GitHub Release. A failed or
ambiguous Shipper run must resume from its retained state; it must not be
restarted by manually uploading crates outside Shipper.

Before publication, the packaged-source and local consumer proof can be run with:

```powershell
pwsh -File scripts/verify-publication-consumers.ps1
```

That proof unpacks the four `.crate` archives, builds temporary consumers with
local patch overrides, and installs the extracted `lintdiff` package locally.
It does not prove crates.io-only resolution; that remains a post-publication
gate using `cargo install lintdiff --registry crates-io --version 0.1.2 --locked` from a clean
consumer context.

---

## Version Pinning for Users

### Using the GitHub Action with a Version Tag

Pin to a specific version in your workflow:

```yaml
- name: Run lintdiff
  uses: EffortlessMetrics/lintdiff@v0.1.1
  with:
    base: main
    head: HEAD
```

### Using the `version` Input

The GitHub Action supports a `version` input:

```yaml
- name: Run lintdiff
  uses: EffortlessMetrics/lintdiff@main  # Development/ref testing only
  with:
    version: v0.1.1  # Required when the Action ref is not an exact tag
    base: main
    head: HEAD
```

The `version` input accepts only an exact release version such as `v0.1.1` or
`v1.0.0-beta.1`. An exact Action tag derives its default version from that tag;
branch and SHA refs must provide `version`, and missing or conflicting values
fail closed. There is no `latest` fallback or moving `v0` alias.

### Direct Binary Download URLs

Binaries can be downloaded directly from GitHub releases:

```text
https://github.com/EffortlessMetrics/lintdiff/releases/download/{TAG}/lintdiff-{VERSION}-{TARGET}.{EXT}
```

Examples:
- Linux: `https://github.com/EffortlessMetrics/lintdiff/releases/download/v0.1.1/lintdiff-0.1.1-x86_64-unknown-linux-gnu.tar.gz`
- macOS (Intel): `https://github.com/EffortlessMetrics/lintdiff/releases/download/v0.1.1/lintdiff-0.1.1-x86_64-apple-darwin.tar.gz`
- macOS (ARM): `https://github.com/EffortlessMetrics/lintdiff/releases/download/v0.1.1/lintdiff-0.1.1-aarch64-apple-darwin.tar.gz`
- Windows: `https://github.com/EffortlessMetrics/lintdiff/releases/download/v0.1.1/lintdiff-0.1.1-x86_64-pc-windows-msvc.zip`

### Checksum Verification

Always verify checksums when downloading binaries:

```bash
# Download binary and checksum
curl -fsSL -O https://github.com/EffortlessMetrics/lintdiff/releases/download/v0.1.1/lintdiff-0.1.1-x86_64-unknown-linux-gnu.tar.gz
curl -fsSL -O https://github.com/EffortlessMetrics/lintdiff/releases/download/v0.1.1/checksums-0.1.1.txt

# Verify checksum
sha256sum -c --ignore-missing checksums-0.1.1.txt
```

Example script for automated download with verification:

```bash
#!/bin/bash
set -euo pipefail

VERSION="v0.1.1"
TARGET="x86_64-unknown-linux-gnu"
EXT="tar.gz"

BASE_URL="https://github.com/EffortlessMetrics/lintdiff/releases/download/${VERSION}"
BINARY="lintdiff-${VERSION#v}-${TARGET}.${EXT}"
CHECKSUMS="checksums-${VERSION#v}.txt"

# Download files
curl -fsSL -O "${BASE_URL}/${BINARY}"
curl -fsSL -O "${BASE_URL}/${CHECKSUMS}"

# Verify checksum
if sha256sum -c --ignore-missing "${CHECKSUMS}"; then
    echo "Checksum verified!"
    tar xzf "${BINARY}"
    ./lintdiff --version
else
    echo "Checksum verification failed!"
    exit 1
fi
```

### Version Pinning Best Practices

1. **Always pin to a specific version** in production CI workflows
2. **Use `@main` only for development/testing** of the action itself
3. **Test new versions in a branch** before updating production workflows
4. **Subscribe to releases** via GitHub watch notifications
5. **Document the version** in comments for clarity:

```yaml
# lintdiff v0.1.1 - Pinned for stability
# See: https://github.com/EffortlessMetrics/lintdiff/releases/tag/v0.1.1
- uses: EffortlessMetrics/lintdiff@v0.1.1
```

---

## Rollback Procedures

### If a Release Has Issues

#### 1. Assess the Severity

| Severity | Action |
|----------|--------|
| **Critical** (broken functionality) | Yank the release immediately |
| **Major** (significant bugs) | Create a patch release |
| **Minor** (cosmetic issues) | Document in next release |

#### 2. Yanking a Release

To "yank" a problematic release:

1. Go to [Releases](https://github.com/EffortlessMetrics/lintdiff/releases)
2. Find the problematic release
3. Click "Edit"
4. Check "Set as a pre-release" and uncheck "Set as the latest release"
5. Add a prominent warning to the release notes:

   ```markdown
   > ⚠️ **WARNING**: This release has been yanked due to [reason].
   > Please use [v0.2.1](link) instead.
   ```

6. Optionally delete the release if it's critically broken

#### 3. Communicate the Issue

- Update CHANGELOG.md with a note about the yanked version
- Create a GitHub Issue documenting the problem
- Notify users via GitHub Discussions (if enabled)

#### 4. Create a Fix Release

```bash
# Fix the issue, then:
git add -A
git commit -m "fix: resolve critical issue from v0.2.0"
# Bump patch version
git tag v0.2.1
git push origin main
git push origin v0.2.1
```

### Recovery Checklist

- [ ] Yanked release marked as pre-release
- [ ] Warning added to release notes
- [ ] CHANGELOG.md updated with yank notice
- [ ] GitHub Issue created
- [ ] Fix developed and tested
- [ ] New patch/minor release created
- [ ] Users notified

---

## Changelog Management

### CHANGELOG.md Format

The changelog follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format:

```markdown
# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- Description of new feature

### Changed
- Description of change

### Deprecated
- Description of deprecation

### Removed
- Description of removal

### Fixed
- Description of bug fix

### Security
- Description of security fix

## [0.2.0] - 2026-03-20

### Added
- Feature descriptions...

## [0.1.0] - 2026-03-16

### Added
- Initial release...
```

### When to Update CHANGELOG.md

- **During development**: Add entries to `[Unreleased]`
- **Before release**: Create new version section, move items from `[Unreleased]`
- **After hotfix**: Add entry to released version section

### Linking to GitHub Releases

Each version header should link to the GitHub release comparison:

```markdown
## [0.2.0] - 2026-03-20
<!-- Compare: https://github.com/effortless-metrics/lintdiff/compare/v0.1.0...v0.2.0 -->
```

For the bottom of the file, include full release links:

```markdown
[0.2.0]: https://github.com/effortless-metrics/lintdiff/releases/tag/v0.2.0
[0.1.0]: https://github.com/effortless-metrics/lintdiff/releases/tag/v0.1.0
```

### Automatic Extraction

The release workflow automatically extracts the relevant section from CHANGELOG.md using:

```bash
sed -n "/^## \\[${VERSION}\\]/,/^## \\[/p" CHANGELOG.md | sed '$d'
```

This means:
- Version headers must use the format `## [X.Y.Z]`
- Content between version headers is included in release notes
- Keep entries concise but informative

### Best Practices

1. **Write for users**, not developers
2. **Include migration notes** for breaking changes
3. **Reference issues and PRs** by number
4. **Group changes** by category (Added, Changed, Fixed, etc.)
5. **Be specific** about what changed and why

Example good entry:
```markdown
### Changed
- `--fail-on` now accepts `error`, `warn`, or `never` (previously `true`/`false`).
  Migrate by replacing `--fail-on true` with `--fail-on error`. (#123)
```

---

## Quick Reference

### Common Commands

```bash
# Create and push a tag
git tag v0.2.0 && git push origin v0.2.0

# Delete an unpushed tag
git tag -d v0.2.0

# Delete a pushed tag (use carefully!)
git push --delete origin v0.2.0

# List all tags
git tag -l

# Download specific version
curl -fsSL https://github.com/EffortlessMetrics/lintdiff/releases/download/v0.2.0/lintdiff-x86_64-unknown-linux-gnu.tar.gz | tar xz
```

### Version Pinning Snippets

**GitHub Action:**
```yaml
- uses: EffortlessMetrics/lintdiff@v0.2.0
  with:
    base: main
```

**Direct Download:**
```bash
VERSION="v0.2.0"
curl -fsSL "https://github.com/EffortlessMetrics/lintdiff/releases/download/${VERSION}/lintdiff-x86_64-unknown-linux-gnu.tar.gz" | tar xz
```

**With Checksum Verification:**
```bash
VERSION="v0.2.0"
curl -fsSL "https://github.com/EffortlessMetrics/lintdiff/releases/download/${VERSION}/checksums-${VERSION#v}.txt" | sha256sum -c --ignore-missing
```
