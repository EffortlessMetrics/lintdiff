# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for the lintdiff project.

## What are ADRs?

Architecture Decision Records are short text documents that capture important architectural decisions made in a project. Each ADR describes:

- **Context**: The issue motivating the decision
- **Decision**: The change being proposed or made
- **Consequences**: What becomes easier or harder as a result

ADRs serve as a historical record of why the architecture is the way it is, helping current and future contributors understand the reasoning behind design choices.

## ADR Numbering Scheme

ADRs are numbered sequentially using the format `ADR-NNN`:

- `ADR-001` - First architecture decision
- `ADR-002` - Second architecture decision
- `ADR-003` - And so on...

Each ADR file follows the naming convention:
```
ADR-NNN-short-title.md
```

Where `short-title` is a kebab-case summary of the decision.

## Status Options

Each ADR has a status field that indicates the current state of the decision:

| Status | Description |
|--------|-------------|
| **Proposed** | The decision is under discussion and not yet finalized |
| **Accepted** | The decision has been approved and is currently in effect |
| **Deprecated** | The decision was once accepted but is no longer recommended |
| **Superseded** | The decision has been replaced by a newer ADR |

When an ADR is superseded, it should reference the new ADR that replaces it.

## Creating a New ADR

1. Copy the template below to a new file: `docs/adr/ADR-NNN-short-title.md`
2. Fill in all sections (Title, Status, Context, Decision, Consequences)
3. Submit the ADR for review as part of a pull request
4. Update the status from "Proposed" to "Accepted" once approved

### ADR Template

```markdown
# ADR-NNN: [Title]

## Status

[Proposed|Accepted|Deprecated|Superseded]

## Context

[Describe the situation and problem that motivated this decision]

## Decision

[Describe the architectural decision being made]

## Consequences

[Describe the impact of this decision - both positive and negative]
```

## Index of ADRs

| ADR | Title | Status |
|-----|-------|--------|
| [ADR-001](ADR-001-hexagonal-architecture.md) | Adopt Hexagonal Architecture with Microcrate Layout | Accepted |
| [ADR-002](ADR-002-deterministic-output.md) | Ensure Deterministic Output | Accepted |
| [ADR-003](ADR-003-schema-validated-receipts.md) | Use JSON Schema for Receipt Validation | Accepted |
| [ADR-004](ADR-004-i18n-strategy.md) | Use Fluent for Internationalization Infrastructure | Accepted |
| [ADR-005](ADR-005-publication-distribution-model.md) | Select publication distribution model before manifest hardening | Accepted |
