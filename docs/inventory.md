# Diagnostic inventory

`lintdiff inventory` emits one complete `lintdiff.inventory.v1` artifact from a
Cargo JSON-lines diagnostics stream. It runs before changed-line filtering,
repository code policy, verdict policy, or report budgets.

```text
lintdiff inventory \
  --diagnostics artifacts/clippy.jsonl \
  --root . \
  --analysis-command-file artifacts/analysis-command.json \
  --upstream-exit-code 0 \
  --out artifacts/lintdiff/inventory.json
```

The command file is a JSON string array, for example:

```json
["cargo", "clippy", "--workspace", "--message-format=json"]
```

The artifact retains every Cargo `compiler-message`, including observations
without spans, and records package and target producer identity, raw and
normalized diagnostic values, children, suggestions, source spans, completion
state, and exact process status when supplied. Raw zero or missing source
positions remain distinct from a normalized source position.

`observation_id_v1` identifies one emitted observation, `occurrence_id_v1`
represents exact producer/code/message/location evidence, and
`semantic_id_v1` is a line-independent candidate identity. `context_id` is
`null` until source context has been earned by a later analysis stage. Existing
`lintdiff.report.v1` output and its finding fingerprint semantics are
unchanged.

The inventory schema is [lintdiff.inventory.v1](../schemas/lintdiff.inventory.v1.json).
Inventory is an experimental evidence contract for the diagnostic-delta work;
it does not run base or head builds and does not claim that a head-only
diagnostic was caused by a source change.
