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

lintdiff uses an automated release workflow defined in [`.github/workflows/release.yml`](../.github/workflows/release.yml) that builds and publishes binaries for multiple platforms.

### Trigger Conditions

The release workflow is triggered by:

1. **Tag Push**: Pushing a tag matching `v*.*.*` (e.g., `v0.2.0`, `v1.0.0-beta.1`)
2. **Manual Dispatch**: Via the GitHub Actions UI with a specified version tag

### Workflow Jobs

The release workflow consists of four jobs:

| Job | Purpose |
|-----|---------|
| `prepare` | Determines version number from tag or input |
| `build` | Builds release binaries for all target platforms |
| `checksums` | Generates combined SHA256 checksums file |
| `release` | Creates GitHub Release with artifacts |

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

### Step-by-Step Guide

#### 1. Update Version Numbers

Update the version in the workspace `Cargo.toml`:

```toml
[workspace.package]
version = "0.2.0"  # Update this
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

#### 3. Commit and Push Changes

```bash
git add -A
git commit -m "chore: prepare release v0.2.0"
git push origin main
```

#### 4. Create and Push the Tag

```bash
git tag v0.2.0
git push origin v0.2.0
```

#### 5. Monitor the Workflow

1. Go to the [Actions tab](https://github.com/effortless-metrics/lintdiff/actions) in GitHub
2. Find the "Release" workflow run for your tag
3. Monitor progress through all four jobs

#### 6. Verify the Release

Once the workflow completes:

1. Go to [Releases](https://github.com/effortless-metrics/lintdiff/releases)
2. Verify the release appears with correct version
3. Download and test binaries on at least one platform:
   ```bash
   # Linux/macOS
   curl -fsSL https://github.com/effortless-metrics/lintdiff/releases/download/v0.2.0/lintdiff-x86_64-unknown-linux-gnu.tar.gz | tar xz
   ./lintdiff --version
   ```
4. Verify checksums match:
   ```bash
   sha256sum lintdiff-x86_64-unknown-linux-gnu.tar.gz
   # Compare with checksums-0.2.0.txt
   ```

### Manual Release via Workflow Dispatch

If you need to trigger a release without pushing a tag:

1. Go to [Actions → Release](https://github.com/effortless-metrics/lintdiff/actions/workflows/release.yml)
2. Click "Run workflow"
3. Enter the version tag (e.g., `v0.2.0`)
4. Click "Run workflow"

This is useful for:
- Re-releasing a failed build
- Creating a release from a specific commit
- Testing the release workflow

---

## Version Pinning for Users

### Using the GitHub Action with a Version Tag

Pin to a specific version in your workflow:

```yaml
- name: Run lintdiff
  uses: effortless-metrics/lintdiff@v0.2.0
  with:
    base: main
    head: HEAD
```

### Using the `version` Input

The GitHub Action supports a `version` input:

```yaml
- name: Run lintdiff
  uses: effortless-metrics/lintdiff@main  # Use main branch of action
  with:
    version: v0.2.0  # Pin lintdiff binary version
    base: main
    head: HEAD
```

Options for `version`:
- `latest` (default): Uses the latest release
- `v0.2.0`: Uses a specific version
- `v1.0.0-beta.1`: Uses a pre-release version

### Direct Binary Download URLs

Binaries can be downloaded directly from GitHub releases:

```
https://github.com/effortless-metrics/lintdiff/releases/download/{TAG}/lintdiff-{TARGET}.{EXT}
```

Examples:
- Linux: `https://github.com/effortless-metrics/lintdiff/releases/download/v0.2.0/lintdiff-x86_64-unknown-linux-gnu.tar.gz`
- macOS (Intel): `https://github.com/effortless-metrics/lintdiff/releases/download/v0.2.0/lintdiff-x86_64-apple-darwin.tar.gz`
- macOS (ARM): `https://github.com/effortless-metrics/lintdiff/releases/download/v0.2.0/lintdiff-aarch64-apple-darwin.tar.gz`
- Windows: `https://github.com/effortless-metrics/lintdiff/releases/download/v0.2.0/lintdiff-x86_64-pc-windows-msvc.zip`

### Checksum Verification

Always verify checksums when downloading binaries:

```bash
# Download binary and checksum
curl -fsSL -O https://github.com/effortless-metrics/lintdiff/releases/download/v0.2.0/lintdiff-x86_64-unknown-linux-gnu.tar.gz
curl -fsSL -O https://github.com/effortless-metrics/lintdiff/releases/download/v0.2.0/checksums-0.2.0.txt

# Verify checksum
sha256sum -c --ignore-missing checksums-0.2.0.txt
```

Example script for automated download with verification:

```bash
#!/bin/bash
set -euo pipefail

VERSION="v0.2.0"
TARGET="x86_64-unknown-linux-gnu"
EXT="tar.gz"

BASE_URL="https://github.com/effortless-metrics/lintdiff/releases/download/${VERSION}"
BINARY="lintdiff-${TARGET}.${EXT}"
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
# lintdiff v0.2.0 - Pinned for stability
# See: https://github.com/effortless-metrics/lintdiff/releases/tag/v0.2.0
- uses: effortless-metrics/lintdiff@v0.2.0
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

1. Go to [Releases](https://github.com/effortless-metrics/lintdiff/releases)
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
curl -fsSL https://github.com/effortless-metrics/lintdiff/releases/download/v0.2.0/lintdiff-x86_64-unknown-linux-gnu.tar.gz | tar xz
```

### Version Pinning Snippets

**GitHub Action:**
```yaml
- uses: effortless-metrics/lintdiff@v0.2.0
  with:
    base: main
```

**Direct Download:**
```bash
VERSION="v0.2.0"
curl -fsSL "https://github.com/effortless-metrics/lintdiff/releases/download/${VERSION}/lintdiff-x86_64-unknown-linux-gnu.tar.gz" | tar xz
```

**With Checksum Verification:**
```bash
VERSION="v0.2.0"
curl -fsSL "https://github.com/effortless-metrics/lintdiff/releases/download/${VERSION}/checksums-${VERSION#v}.txt" | sha256sum -c --ignore-missing
```
