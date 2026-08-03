# Diagnostic delta experiment

This document freezes the evidence model for the diagnostic-delta experiment.
It is a design and adjudication contract, not an implementation API. The current
`lintdiff.report.v1` location-scoped receipt remains unchanged.

## Product boundary

The experiment compares two complete normalized diagnostic inventories: one
from a base analysis and one from a head analysis. It then places the comparison
in source-diff context. A result described as `new` means that no comparable base
diagnostic was paired with it; it does not prove that the source change was the
sole cause.

The protocol boundaries are deliberately separate:

| Protocol | Meaning | Status |
| --- | --- | --- |
| `lintdiff.report.v1` | Current changed-line location receipt | Existing and unchanged |
| `lintdiff.inventory.v1` | One complete normalized analysis | Experimental contract |
| `lintdiff.delta.v1` | Base/head comparison plus source context | Experimental contract |

No implementation crate, workspace package, or public API is introduced by this
model-and-corpus change.

## Pairing evidence

Pairing evidence is recorded before any confident change classification. It is
not itself a `DeltaKind`:

```text
Matched { base, head, basis }
BaseOnly { base }
HeadOnly { head }
Ambiguous { base_candidates, head_candidates, reasons }
Blocked { reason }
```

An ambiguous candidate set remains ambiguous even when it is located on a
changed line. A blocked pairing is produced when report-level comparability
does not permit a delta claim.

## Independent evidence axes

Every paired or unpaired diagnostic records these dimensions independently:

### Confident change kind

`unchanged`, `new`, `resolved`, or `modified`.

`new` and `resolved` are comparison results. `modified` means the identity is
paired but a material diagnostic field changed. An unchanged paired diagnostic
is the source of the derived `existing_touched` and `existing_untouched`
labels. Ambiguity belongs to pairing evidence and never receives a confident
change kind.

### Diff scope

`touched`, `untouched`, `no_location`, or `unknown`.

Scope is derived from earned source correspondence and hunk ranges. A missing
or unsupported location never becomes `untouched` merely because no match was
found.

### Match basis

`exact`, `line_mapped`, `rename_mapped`, `semantic`, `context`, `none`, or
`ambiguous`.

The intended confidence order is:

1. exact identity on the same normalized path and span;
2. line mapping through the same-path unified diff;
3. explicit Git rename mapping plus span correspondence;
4. semantic identity after normalized code/message comparison;
5. contextual identity using surrounding evidence;
6. `none`, when no candidate exists;
7. `ambiguous`, when the best confidence has multiple candidates.

Pairing must never use list position as a tie-breaker.

### Movement

`same`, `shifted`, `renamed`, `shifted_and_renamed`, or `unknown`.

Movement describes source correspondence, not diagnostic change. If the source
relationship cannot be earned from the diff and rename metadata, it is `unknown`.

## Provenance and comparability

Each inventory records hard provenance when available:

- repository identity;
- base/head revision;
- tool name and version;
- compiler/toolchain identity;
- producer package, manifest, target name, and target kind;
- target triple;
- enabled feature scope;
- Cargo completion and build-success evidence.

Contextual provenance may include the workflow, operating system, command-line
arguments, and environment details. Context helps explain a result but does not
override a hard mismatch.

The report-level comparability state is one of:

| State | Meaning | Default action |
| --- | --- | --- |
| `comparable` | Required hard provenance agrees and both inventories are complete | Permit pairing |
| `incomplete_base` | Base inventory lacks complete build evidence | Do not claim resolved/existing/new |
| `incomplete_head` | Head inventory lacks complete build evidence | Do not claim resolved/existing/new |
| `toolchain_mismatch` | Compiler or toolchain identity differs | Report incomparable |
| `target_mismatch` | Target triple differs | Report incomparable |
| `feature_scope_mismatch` | Enabled feature scope differs | Report incomparable |
| `repository_mismatch` | Inventories do not belong to the same repository | Report incomparable |

`incomparable` is a report-level state, not a diagnostic classification. An
incomparable report may preserve raw inventories and source evidence, but it
must not manufacture `new`, `resolved`, or causal labels.

For the first delta version, both analyses must be successful and complete for
confident `new`, `resolved`, or unchanged-pair claims. A failed-complete stream
may retain an inventory receipt, but it is not treated as a complete inventory
for comparison.

## Derived human labels

Human labels are projections over the independent axes and may not replace
them. The minimum derived labels are:

```text
new       + touched   -> new_on_diff
new       + untouched -> new_off_diff
existing  + touched   -> existing_touched
existing  + untouched -> existing_untouched
resolved             -> resolved
modified             -> modified
ambiguous pairing    -> ambiguous
```

`no_location`, `unknown`, `none`, and incomparable states remain visible in the
evidence even when a projection is budgeted or omitted.

## Ambiguity rules

- Preserve all candidates at the best confidence level.
- Emit an ambiguous pairing when two or more candidates remain tied.
- Never pair duplicate diagnostics by input order.
- Never turn an ambiguous or incomparable result into a confident change kind.
- Preserve the candidate identities and reasons needed for later adjudication.

## Policy defaults

The experiment is fail-closed for evidence claims:

- incomplete or incomparable inventories produce an explicit report state;
- unknown source regions remain unknown;
- matching confidence is retained alongside the derived label;
- policy and budgets run after inventory and comparison evidence exist;
- human projections are lossy views of the receipt, not a second model.

## Golden corpus

[`fixtures/compare/cases.toml`](../fixtures/compare/cases.toml) is the
adjudication authority for later implementation issues. Each case contains a
base JSONL inventory input, a head JSONL inventory input, a unified source diff,
the expected evidence axes, and a rationale. The corpus intentionally includes
both positive and falsifying cases:

| Case | Primary falsification |
| --- | --- |
| `unrelated_insertion_shift` | A line shift is not a new diagnostic |
| `file_rename_unchanged_warning` | Rename evidence is distinct from same-path mapping |
| `old_warning_on_touched_line` | Existing does not mean off-diff |
| `new_warning_on_untouched_line` | New does not imply touched |
| `resolved_warning` | Missing head evidence can be resolved only when comparable |
| `duplicate_identical_diagnostics` | Duplicate groups must remain ambiguous without evidence |
| `changed_message` | A paired diagnostic can be modified |
| `changed_severity` | Severity is evidence, not identity |
| `no_span_diagnostic` | Missing locations are not untouched |
| `multi_span_diagnostic` | Primary and secondary spans need explicit handling |
| `macro_generated_path` | Generated or macro paths need provenance and scope caution |
| `absolute_relative_paths` | Path normalization must not create a false mismatch |
| `incomplete_base` | Incomplete base cannot justify a new/resolved claim |
| `different_toolchains` | Toolchain mismatch is report-level incomparable |
| `different_target_features` | Target and feature scope are independent hard provenance |
| `discovery_ripgrep_3482_new_on_diff` | A historical head-only warning on added indexing code is a real `new_on_diff` candidate |
| `discovery_pst_1314_existing_adjacent` | A warning adjacent to, but not on, the changed line remains existing_untouched |
| `discovery_ripgrep_3487_incomplete_head` | A failed-complete head analysis blocks a confident delta |

Every later inventory, source-correspondence, pairing, delta-receipt, and
external-verdict issue must cite this corpus and retain its adjudications.

## Invariants for implementation

1. `lintdiff.report.v1` bytes and semantics remain unchanged.
2. Every diagnostic is retained in a complete inventory before filtering,
   source scope, policy, or budgeting.
3. Pairing evidence, confident change kind, diff scope, match basis, movement,
   comparability, and derived labels are independently inspectable.
4. A source match is claimed only when earned by hunk ranges, offsets, or
   explicit rename evidence.
5. Ambiguous candidates remain ambiguous and receive no `DeltaKind`; list order
   is never proof.
6. Incomplete or mismatched hard provenance is fail-closed.
7. `new` means new relative to a comparable base analysis, not causal proof.
8. Human projections consume the canonical typed evidence and cannot invent a
   second comparison model.
