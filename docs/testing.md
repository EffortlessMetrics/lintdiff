# lintdiff test strategy

lintdiff is a gatekeeper-style tool. The “architecture” is mostly the test discipline.

## Layers of tests

### 1) Golden fixtures (contract tests)

Fixtures are the primary contract.

- Inputs:
  - `tests/fixtures/*.diff`
  - `tests/fixtures/*.jsonl`
- Outputs (golden):
  - `tests/fixtures/expected/*.report.json`
  - `tests/fixtures/expected/*.comment.md`

Golden tests enforce:

- schema compliance
- deterministic ordering
- stable truncation behavior

### 2) BDD scenarios (behavior, not internals)

A small number of “Given/When/Then” scenarios ensure the tool behaves as reviewers expect:

- match diagnostics on changed lines
- ignore diagnostics outside the diff
- primary span selection
- workspace-only filtering
- missing input semantics (`skip`, not `pass`)

BDD is intentionally thin; golden fixtures carry the detailed surface contract.

### 3) Property tests (proptest)

Property tests protect the small core algorithms:

- range merge/idempotence
- intersection correctness
- fingerprint stability

### 4) Fuzzing (cargo-fuzz)

Fuzzing protects parsers against panics and pathological inputs:

- diff parser
- diagnostics stream parser

Run timeboxed (CI schedule or on-demand), not per PR by default.

### 5) Mutation testing (cargo-mutants)

Mutation testing keeps you honest about coverage and the quality of assertions.

Recommendation:

- run on schedule (weekly) or on demand
- treat “mutants survived” as a prompt to improve tests, not a release blocker

## Determinism tests

Add a determinism test that runs the same ingest twice and asserts:

- JSON bytes identical
- Markdown bytes identical

This prevents “churny” PR comments.

