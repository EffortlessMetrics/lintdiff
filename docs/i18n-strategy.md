# Internationalization (i18n) Strategy

This document outlines the internationalization strategy for lintdiff, preparing the infrastructure for future multi-language support.

## Overview

While lintdiff currently only supports English, this infrastructure prepares for future internationalization needs. The strategy uses [Fluent](https://projectfluent.org/) as the localization system, which provides robust pluralization, gender support, and message formatting.

## String Extraction Strategy

### Identifying Translatable Strings

Strings should be extracted from:

1. **User-facing messages**: CLI output, error messages, help text
2. **Report content**: Verdict messages, summary descriptions
3. **Log messages** (optional): Informational messages at INFO level and above

Strings that should **not** be extracted:

- Debug/trace log messages
- Internal error messages (panics, assertions)
- JSON keys in structured output
- Technical identifiers (error codes, lint names)

### Extraction Process

```bash
# Future: Automated extraction command
cargo i18n extract --locale en-US

# Manual process:
# 1. Identify all user-facing strings in code
# 2. Add them to crates/lintdiff-i18n/src/locales/en-US/main.ftl
# 3. Use fluent macros in code: i18n::msg!("message-key", args)
```

### Message Organization

Messages are organized by module/feature in FTL files:

```
src/locales/en-US/
├── main.ftl        # Core messages, errors
├── cli.ftl         # CLI-specific messages
├── report.ftl      # Report output messages
└── errors.ftl      # Error messages
```

## Message Format (Fluent)

### FTL File Format

Fluent uses `.ftl` (Fluent Translation List) files:

```ftl
# Simple message
welcome = Welcome to lintdiff!

# Message with variable
greeting = Hello, { $name }!

# Message with attributes
error-not-found =
    .title = File Not Found
    .description = The file { $path } was not found.

# Pluralization
found-issues =
    { $count ->
        [one] Found one issue
       *[other] Found { $count } issues
    }

# Selectors
lint-severity =
    { $severity ->
        [error] Error
        [warning] Warning
       *[note] Note
    }
```

### Message Naming Convention

Use kebab-case with hierarchical prefixes:

- `cli-<command>-<action>`: CLI-related messages
- `error-<type>-<detail>`: Error messages
- `report-<section>-<item>`: Report output
- `config-<setting>-<description>`: Configuration messages

Examples:
- `cli-run-starting`
- `error-file-not-found`
- `report-summary-header`
- `config-invalid-toml`

### Placeholders

Use descriptive placeholder names:

```ftl
# Good
diff-analyzed = Analyzed { $files } files with { $additions } additions and { $deletions } deletions.

# Avoid
diff-analyzed = Analyzed { $n } files with { $a } additions and { $d } deletions.
```

## Fallback Behavior

### Language Resolution Order

1. **Explicit locale**: `--locale` CLI flag or `LINTDIFF_LOCALE` environment variable
2. **System locale**: Detected from `LANG`, `LC_ALL`, or Windows locale APIs
3. **Default fallback**: `en-US` (always available)

### Missing Message Handling

When a message is not found in the current locale:

1. Fall back to the default locale (`en-US`)
2. If still not found, return a placeholder: `[missing: message-key]`
3. Log a warning in debug builds

### Partial Translation Support

For locales with incomplete translations:

```rust
// Fallback chain: requested-locale -> en-US
let bundle = FluentBundle::new(&[requested_locale, "en-US"]);
```

## Language Detection

### Detection Priority

1. **CLI flag**: `--locale de-DE`
2. **Environment variable**: `LINTDIFF_LOCALE=de-DE`
3. **System locale**:
   - Unix: Parse `LANG` or `LC_ALL` (e.g., `de_DE.UTF-8` -> `de-DE`)
   - Windows: Use `winapi` to query user locale
4. **Default**: `en-US`

### Supported Locales

Initially, only `en-US` is supported. Additional locales can be added by:

1. Creating a new directory: `src/locales/<locale>/`
2. Copying FTL files from `en-US`
3. Translating messages
4. Adding locale to the `Locale` enum

### Locale Format

Use [Unicode BCP 47 locale identifiers](https://unicode.org/reports/tr35/):

- Format: `<language>-<REGION>` (e.g., `en-US`, `de-DE`, `zh-CN`)
- Language code: ISO 639-1 two-letter code
- Region code: ISO 3166-1 two-letter code

## Implementation Guidelines

### Rust Integration

```rust
use lintdiff_i18n::{Locale, Message, msg};

// Get message for current locale
let message = msg("welcome")?;

// With placeholders
let message = msg("greeting", {"name": "World"})?;

// Explicit locale
let locale = Locale::DeDE;
let message = locale.message("welcome")?;
```

### Testing

- All messages must have an `en-US` version
- Tests should not depend on system locale
- Use `LINTDIFF_LOCALE=en-US` in test environments

### Performance Considerations

- Fluent bundles are loaded lazily
- Bundle caching for repeated lookups
- Consider compile-time message extraction for production builds

## Future Enhancements

1. **Automated extraction**: Tool to extract strings from source code
2. **Translation platform integration**: Integrate with Crowdin/Weblate
3. **RTL support**: Right-to-left language support
4. **Locale-specific formatting**: Dates, numbers, lists
5. **Message context**: Disambiguate identical strings with different meanings

## References

- [Fluent Project](https://projectfluent.org/)
- [Fluent Rust](https://github.com/projectfluent/fluent-rs)
- [Unicode Locale Data](https://unicode.org/reports/tr35/)
