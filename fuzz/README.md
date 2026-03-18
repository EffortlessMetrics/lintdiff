# lintdiff Fuzzing

This directory contains fuzzing infrastructure for the lintdiff project. Fuzz targets are kept out of the main workspace to avoid pulling nightly tooling into normal CI.

## Setup

```bash
# Install the cargo-fuzz tool (requires nightly Rust)
cargo install cargo-fuzz

# Ensure you have the nightly toolchain installed
rustup install nightly
```

## Available Targets

| Target | Description | Corpus Directory |
|--------|-------------|------------------|
| `diff_parser` | Tests unified diff parsing | `corpus/diff_parser/` |
| `diagnostics_parser` | Tests JSONL diagnostic parsing | `corpus/diagnostics_parser/` |
| `finding_fingerprint` | Tests fingerprint generation | `corpus/finding_fingerprint/` |

## Running Fuzz Tests

### Run All Targets

```bash
cd fuzz

# Run each target for 60 seconds (default)
cargo fuzz run diff_parser
cargo fuzz run diagnostics_parser
cargo fuzz run finding_fingerprint
```

### Run with Custom Duration

```bash
# Run for 5 minutes (300 seconds)
cargo fuzz run diff_parser -- -max_total_time=300

# Run for a specific number of iterations
cargo fuzz run diff_parser -- -runs=100000
```

### Run with Corpus

The corpus directories provide structured test cases that seed the fuzzer with known inputs:

```bash
# The fuzzer automatically uses corpus/<target>/ if it exists
cargo fuzz run diff_parser

# You can also specify corpus directories explicitly
cargo fuzz run diff_parser -- corpus/diff_parser/
```

### Run with Memory Limits

```bash
# Limit memory usage to 2GB
cargo fuzz run diff_parser -- -rss_limit_mb=2048
```

### Run Only Specific Inputs

```bash
# Run only a specific corpus file
cargo fuzz run diff_parser -- corpus/diff_parser/simple_addition.diff
```

## Corpus Structure

### diff_parser Corpus

Located in [`corpus/diff_parser/`](corpus/diff_parser/), contains sample diff files:

| File | Description |
|------|-------------|
| `simple_addition.diff` | Basic single-line addition |
| `multi_file.diff` | Changes across multiple files |
| `deletion.diff` | Line removal |
| `rename.diff` | File rename with similarity |
| `moved_code.diff` | Code moved between files |
| `empty_hunk.diff` | Empty diff hunk |
| `large_hunk.diff` | Many lines changed |
| `binary_file.diff` | Binary file handling |
| `mode_change.diff` | File permission change |
| `empty_file.diff` | Empty file creation |

### diagnostics_parser Corpus

Located in [`corpus/diagnostics_parser/`](corpus/diagnostics_parser/), contains JSONL diagnostic files:

| File | Description |
|------|-------------|
| `simple_warning.jsonl` | Basic clippy warning |
| `multi_span.jsonl` | Diagnostic with multiple spans |
| `error_message.jsonl` | Compiler error |
| `no_span.jsonl` | Diagnostic without location |
| `note_help.jsonl` | Diagnostic with children |
| `macro_expanded.jsonl` | Macro expansion span |
| `generated_file.jsonl` | Generated code location |
| `multiple_messages.jsonl` | Multiple diagnostics |
| `unicode_message.jsonl` | Unicode in message |
| `long_path.jsonl` | Deeply nested file path |

### finding_fingerprint Corpus

Located in [`corpus/finding_fingerprint/`](corpus/finding_fingerprint/), contains binary files with null-separated fields:

Format: `code\0path\0line\0message`

| File | Description |
|------|-------------|
| `simple_fingerprint.bin` | Standard fingerprint |
| `no_location.bin` | Fingerprint without path |
| `unicode_message.bin` | Unicode in message |
| `long_code.bin` | Very long lint code |
| `empty_parts.bin` | All empty fields |
| `error_fingerprint.bin` | Error-level lint |
| `multi_part.bin` | Multiple entries |

## Adding New Corpus Entries

### For diff_parser

1. Create a new `.diff` file in `corpus/diff_parser/`
2. Use standard unified diff format:

```diff
diff --git a/src/example.rs b/src/example.rs
--- a/src/example.rs
+++ b/src/example.rs
@@ -1,1 +1,2 @@
 existing line
+new line
```

### For diagnostics_parser

1. Create a new `.jsonl` file in `corpus/diagnostics_parser/`
2. Use cargo/rustc JSON message format:

```json
{"reason":"compiler-message","message":{"level":"warning","message":"description","code":{"code":"lint-name"},"spans":[{"file_name":"path","line_start":1,"line_end":1,"column_start":1,"column_end":10,"is_primary":true}]}}
```

### For finding_fingerprint

1. Create binary files with null-separated fields
2. Format: `code\0path\0line\0message`
3. Use PowerShell on Windows:

```powershell
$bytes = [System.Text.Encoding]::UTF8.GetBytes('code') + [byte]0 + [System.Text.Encoding]::UTF8.GetBytes('path') + [byte]0 + [System.Text.Encoding]::UTF8.GetBytes('line') + [byte]0 + [System.Text.Encoding]::UTF8.GetBytes('message'); [System.IO.File]::WriteAllBytes('corpus/finding_fingerprint/name.bin', $bytes)
```

Or on Unix:

```bash
printf 'code\0path\0line\0message' > corpus/finding_fingerprint/name.bin
```

## Reproducing Crashes

When a fuzzer finds a crash, it saves the crashing input to `artifacts/<target>/`.

### View Crash Details

```bash
# List crash artifacts
ls -la artifacts/diff_parser/

# View the crashing input (if text)
cat artifacts/diff_parser/crash-<hash>
```

### Reproduce a Crash

```bash
# Run the fuzzer with the crashing input
cargo fuzz run diff_parser -- artifacts/diff_parser/crash-<hash>

# For better stack traces, build with debug info
RUST_BACKTRACE=1 cargo fuzz run diff_parser -- artifacts/diff_parser/crash-<hash>
```

### Debug with LLDB/GDB

```bash
# Build with debug symbols
cargo fuzz build --debug

# Run under debugger
rust-lldb -- ./fuzz/target/x86_64-unknown-linux-gnu/debug/diff_parser artifacts/diff_parser/crash-<hash>
```

## Minimizing Crash Inputs

To find the smallest input that triggers a crash:

```bash
cargo fuzz tmin diff_parser -- artifacts/diff_parser/crash-<hash>
```

## Merging Corpus

After running fuzzing, merge new interesting inputs back to the corpus:

```bash
# Merge findings from a fuzzing session
cargo fuzz cmin diff_parser
```

## CI Integration

Fuzzing runs automatically in CI:

- **Schedule**: Weekly on Sundays at 00:00 UTC
- **Push**: When fuzz-related files change on `main`
- **Manual**: Via workflow dispatch with configurable duration

### Manual CI Run

1. Go to Actions → Fuzzing
2. Click "Run workflow"
3. Optionally specify:
   - Duration per target (seconds)
   - Specific targets (comma-separated)

### View CI Results

- Crash artifacts are uploaded and retained for 30 days
- Corpus artifacts are uploaded and retained for 7 days
- Check the workflow run logs for details

## Best Practices

1. **Add corpus entries for edge cases**: When you find a bug, add a corpus entry
2. **Keep corpus files small**: Smaller files help the fuzzer explore faster
3. **Cover different code paths**: Include various diff formats, diagnostic types
4. **Test invariants**: Fuzz targets can include assertions for properties
5. **Don't ignore crashes**: Every crash indicates a potential vulnerability

## Troubleshooting

### "error: the `fuzz` target is not a directory"

Make sure you're running from the workspace root, not inside `fuzz/`:

```bash
cd /path/to/lintdiff
cargo fuzz run diff_parser
```

### "error: toolchain 'nightly-x86_64-unknown-linux-gnu' is not installed"

```bash
rustup install nightly
rustup component add rust-src --toolchain nightly
```

### Out of Memory

Reduce the memory limit or input size:

```bash
cargo fuzz run diff_parser -- -rss_limit_mb=1024 -max_len=1024
```

### Slow Progress

Increase parallelism or use a dictionary:

```bash
# Use all available cores
cargo fuzz run diff_parser -- -jobs=0

# Use a dictionary (if available)
cargo fuzz run diff_parser -- -dict=dictionary.txt
```
