# ADR-004: Use Fluent for Internationalization Infrastructure

## Status

Accepted

## Context

As lintdiff grows, there may be future requirements for internationalization (i18n) to support users in different languages. While full i18n is not currently needed, preparing the infrastructure early makes future implementation significantly easier.

We evaluated several options for i18n infrastructure:

1. **gettext**: Traditional Unix approach with `.po` files
2. **Fluent**: Modern system developed by Mozilla
3. **Custom solution**: Build our own message system
4. **No preparation**: Defer all i18n work until needed

### Evaluation Criteria

- **Pluralization support**: Must handle complex plural rules
- **Gender/grammar support**: Some languages require grammatical agreement
- **Rust ecosystem**: Quality of available crates
- **Tooling**: Availability of extraction and validation tools
- **Maintainability**: Ease of managing translations

## Decision

We adopt **Fluent** as our i18n infrastructure with a dedicated `lintdiff-i18n` crate for preparation purposes.

### Fluent Selection Rationale

1. **Modern design**: Handles complex language features naturally
2. **Rust support**: `fluent` and `fluent-bundle` crates are well-maintained
3. **Asymmetric localization**: Translators can add context without code changes
4. **Placeholders**: First-class support for variable interpolation
5. **Pluralization**: Built-in support for complex plural forms

### Crate Structure

```
crates/lintdiff-i18n/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Message trait, Locale enum
│   └── locales/
│       └── en-US/
│           ├── main.ftl    # Core messages
│           ├── cli.ftl     # CLI messages
│           ├── report.ftl  # Report output
│           └── errors.ftl  # Error messages
```

### Key Components

1. **`Locale` enum**: Supported locales with `en-US` as default
2. **`Message` trait**: Interface for localizable messages
3. **FTL files**: Fluent translation files organized by feature
4. **Fallback chain**: Requested locale → `en-US` → placeholder

## Consequences

### Positive

- **Future-ready**: Infrastructure in place for easy translation
- **Clean separation**: Localization logic isolated in dedicated crate
- **Type-safe**: `Locale` enum prevents invalid locale strings
- **Incremental adoption**: Can add translations module by module
- **Tooling ready**: Fluent has good tooling support (Pontoon, Crowdin)

### Negative

- **Additional dependency**: Fluent crates add to compile time
- **Learning curve**: Team needs to learn FTL syntax
- **Placeholder code**: Some code won't be used until translations are added
- **Maintenance**: FTL files need to be kept in sync with code changes

### Mitigations

- Document FTL syntax and best practices in `docs/i18n-strategy.md`
- Start with minimal infrastructure, expand as needed
- Use `allow(dead_code)` for preparation code that isn't yet used
- Consider automated extraction tools when translation work begins

## Implementation Notes

- Only `en-US` locale is initially supported
- The `Message` trait provides a stable API for future expansion
- CLI flag `--locale` is reserved for future use
- Environment variable `LINTDIFF_LOCALE` is reserved for future use
