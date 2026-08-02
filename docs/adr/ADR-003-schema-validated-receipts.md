# ADR-003: Use JSON Schema for Receipt Validation

## Status

Accepted

## Context

lintdiff produces structured output (receipts) that is consumed by various CI/CD systems, GitHub Actions, and programmatic integrations. These consumers need:

- **Reliability**: Confidence that the output structure won't unexpectedly change
- **Validation**: Ability to verify output correctness
- **Documentation**: Clear specification of the expected data format
- **Versioning**: Understanding of what changes between releases

Without a formal schema:
- Breaking changes may go undetected until runtime failures
- Consumers must reverse-engineer the output format
- No automated validation of output correctness
- Difficult to maintain compatibility across versions

## Decision

All JSON output from lintdiff (receipts and reports) must conform to **versioned JSON schemas**.

### Schema Structure

Schemas are stored in the `schemas/` directory:

```
schemas/
├── lintdiff.report.v1.json      # Report schema (version 1)
```

### Versioning Policy

1. **Semantic versioning for schemas**: Major version increments indicate breaking changes
2. **Backward compatibility**: New minor/patch versions must be backward compatible
3. **Breaking changes require new versions**: Incompatible changes must use a new major version (e.g., `v2`)
4. **Schema inclusion**: The canonical report schema is included in releases and documentation

### Validation

- The `lintdiff-types` crate includes schema validation tests
- Sample outputs are validated against schemas in CI
- Schema validation can be enabled as a runtime feature for consumers

### Schema Location

Schemas are:
- Versioned alongside the codebase
- Published to the repository for consumer reference
- Available for `$schema` references in JSON output

## Consequences

### Positive

- **Contract guarantee**: Consumers have a reliable contract for the output format
- **Automated validation**: CI can verify output correctness automatically
- **Clear documentation**: Schemas serve as machine-readable documentation
- **Version clarity**: Breaking changes are explicit through version numbers
- **Tool support**: JSON schema is widely supported by IDEs and validation tools

### Negative

- **Schema maintenance**: Schemas must be kept in sync with code changes
- **Breaking change friction**: Incompatible changes require careful versioning
- **Additional testing**: Schema validation tests must be maintained

### Mitigations

- Generate schemas from Rust types where possible (using `schemars` or similar)
- Include schema validation in the standard test suite
- Document the versioning policy clearly for consumers
- Provide migration guides when breaking changes are necessary
