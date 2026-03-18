# ADR-001: Adopt Hexagonal Architecture with Microcrate Layout

## Status

Accepted

## Context

lintdiff is a CLI tool that analyzes lint diagnostics in the context of git diffs. As the project grows, we need an architecture that:

- **Supports testability**: Core business logic should be testable without requiring external dependencies like git repositories or file systems
- **Enables modularity**: Different components should be independently developable and testable
- **Facilitates code reuse**: Domain logic should be reusable across different contexts (CLI, library, GitHub Action)
- **Maintains clarity**: New contributors should understand where to add code and how components interact

The traditional monolithic crate structure can lead to:
- Tight coupling between I/O operations and business logic
- Difficulty in unit testing without integration infrastructure
- Unclear boundaries between domain concepts and infrastructure concerns

## Decision

We adopt **hexagonal architecture** (also known as ports and adapters) combined with a **microcrate layout** for lintdiff.

### Hexagonal Architecture

The architecture separates the codebase into:

1. **Domain Core** (`lintdiff-domain`): Pure business logic with no external dependencies
2. **Ports**: Traits/interfaces that define contracts for external interactions
3. **Adapters**: Concrete implementations of ports for specific technologies
4. **Application Layer** (`lintdiff-app`, `lintdiff-app-*`): Orchestrates domain logic through ports

### Microcrate Layout

The project is decomposed into focused crates:

| Crate | Responsibility |
|-------|---------------|
| `lintdiff-domain` | Core domain types and business rules |
| `lintdiff-core` | Main application orchestration |
| `lintdiff-app` | Application traits (ports) |
| `lintdiff-app-io` | I/O adapter implementations |
| `lintdiff-app-git` | Git adapter implementations |
| `lintdiff-cli` | Command-line interface entry point |
| `lintdiff-diff` | Diff parsing and analysis |
| `lintdiff-diagnostics` | Diagnostic parsing and handling |
| `lintdiff-ingest`, `lintdiff-ingest-core` | Data ingestion pipeline |
| `lintdiff-match` | Diff-to-diagnostic matching logic |
| `lintdiff-policy` | Policy evaluation and verdicts |
| `lintdiff-render` | Output rendering (JSON, Markdown) |
| `lintdiff-types` | Shared type definitions |
| `lintdiff-fingerprint` | Content fingerprinting |
| `lintdiff-feature-flags` | Feature flag management |
| `lintdiff-bdd-*` | BDD testing infrastructure |

## Consequences

### Positive

- **Testability**: Domain logic can be tested in isolation using mock adapters
- **Clear boundaries**: Each crate has a well-defined responsibility
- **Flexibility**: New adapters can be added without modifying core logic (e.g., supporting different VCS systems)
- **Reusability**: The domain layer can be consumed as a library by different frontends
- **Parallel development**: Team members can work on different crates with minimal conflicts

### Negative

- **Crate management**: More crates mean more `Cargo.toml` files to maintain
- **Dependency complexity**: Care must be taken to avoid circular dependencies
- **Build overhead**: More crates can increase build times if not properly managed
- **Learning curve**: New contributors need to understand the hexagonal pattern

### Mitigations

- Use workspace-level dependency management in the root `Cargo.toml`
- Document the architecture clearly in `docs/architecture.md`
- Provide examples and templates for common development tasks
