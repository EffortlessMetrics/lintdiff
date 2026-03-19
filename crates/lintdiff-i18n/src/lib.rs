//! Internationalization infrastructure for lintdiff.
//!
//! This crate provides the infrastructure for future internationalization (i18n)
//! support using the Fluent localization system. While currently only English
//! (en-US) is supported, this infrastructure prepares for easy addition of
//! additional languages.
//!
//! # Architecture
//!
//! - [`Locale`]: Enum representing supported locales
//! - [`Message`]: Trait for localizable messages
//! - [`LocalizationBundle`]: Manages Fluent bundles for message lookup
//!
//! # Example
//!
//! ```ignore
//! use lintdiff_i18n::{Locale, Message, LocalizationBundle};
//!
//! let bundle = LocalizationBundle::new(Locale::EnUS)?;
//! let message = bundle.get("welcome")?;
//! println!("{}", message);
//! ```
//!
//! # Future Development
//!
//! To add a new locale:
//! 1. Add variant to [`Locale`] enum
//! 2. Create `src/locales/<locale>/` directory
//! 3. Add FTL translation files
//! 4. Update [`LocalizationBundle::new`] to handle the new locale

use std::cell::RefCell;
use std::collections::HashMap;

use fluent::FluentArgs;
use fluent_bundle::{FluentBundle, FluentResource};
use thiserror::Error;
use unic_langid::{LanguageIdentifier, LanguageIdentifierError};

/// Default locale when none is specified or detected.
pub const DEFAULT_LOCALE: Locale = Locale::EnUS;

/// Supported locales for lintdiff.
///
/// Each variant corresponds to a Unicode BCP 47 locale identifier.
/// The `Display` implementation returns the canonical locale string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Locale {
    /// English (United States) - the default locale
    #[default]
    EnUS,
    // Future locales can be added here:
    // DeDE,  // German (Germany)
    // FrFR,  // French (France)
    // JaJP,  // Japanese (Japan)
    // ZhCN,  // Chinese (Simplified)
}

impl Locale {
    /// Returns the canonical BCP 47 language tag for this locale.
    #[must_use]
    pub const fn language_tag(&self) -> &'static str {
        match self {
            Self::EnUS => "en-US",
        }
    }

    /// Detects the locale from the system environment.
    ///
    /// Checks in order:
    /// 1. `LINTDIFF_LOCALE` environment variable
    /// 2. System locale (LANG on Unix, user locale on Windows)
    /// 3. Falls back to [`DEFAULT_LOCALE`]
    #[must_use]
    pub fn detect() -> Self {
        use std::str::FromStr;
        // Check environment variable first
        if let Ok(locale_str) = std::env::var("LINTDIFF_LOCALE") {
            if let Ok(locale) = Self::from_str(&locale_str) {
                return locale;
            }
        }

        // Try system locale detection
        #[cfg(unix)]
        {
            if let Ok(lang) = std::env::var("LANG") {
                // Parse locale like "en_US.UTF-8" -> "en-US"
                let normalized = lang.split('.').next().unwrap_or(&lang).replace('_', "-");
                if let Ok(locale) = Self::from_str(&normalized) {
                    return locale;
                }
            }
        }

        #[cfg(windows)]
        {
            // Windows locale detection would go here
            // For now, fall through to default
        }

        DEFAULT_LOCALE
    }
}

impl std::str::FromStr for Locale {
    type Err = LocaleError;

    /// Attempts to parse a locale string into a [`Locale`].
    ///
    /// # Errors
    ///
    /// Returns [`LocaleError::UnknownLocale`] if the locale is not supported.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en-US" | "en_US" | "en" => Ok(Self::EnUS),
            _ => Err(LocaleError::UnknownLocale(s.to_string())),
        }
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.language_tag())
    }
}

impl TryFrom<&str> for Locale {
    type Error = LocaleError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        std::str::FromStr::from_str(s)
    }
}

/// Trait for types that can provide localized messages.
///
/// This trait defines the interface for message lookup and formatting.
/// Implementations typically wrap a Fluent bundle.
pub trait Message {
    /// Gets a simple message by key.
    ///
    /// # Errors
    ///
    /// Returns [`LocaleError::MessageNotFound`] if the key doesn't exist.
    fn get(&self, key: &str) -> Result<String, LocaleError>;

    /// Gets a message with variable interpolation.
    ///
    /// The `args` map provides values for placeholders in the message.
    ///
    /// # Errors
    ///
    /// Returns [`LocaleError::MessageNotFound`] if the key doesn't exist,
    /// or [`LocaleError::FormattingError`] if interpolation fails.
    fn get_with_args(
        &self,
        key: &str,
        args: &HashMap<&str, fluent::FluentValue<'_>>,
    ) -> Result<String, LocaleError>;

    /// Gets a message attribute.
    ///
    /// For messages with attributes like:
    /// ```ftl
    /// error-not-found =
    ///     .title = Not Found
    ///     .description = The file was not found.
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`LocaleError::AttributeNotFound`] if the attribute doesn't exist.
    fn get_attribute(&self, key: &str, attr: &str) -> Result<String, LocaleError>;
}

/// Manages Fluent bundles for message localization.
///
/// This is the primary interface for retrieving localized messages.
/// Bundles are cached for performance.
pub struct LocalizationBundle {
    locale: Locale,
    bundle: FluentBundle<FluentResource>,
}

impl LocalizationBundle {
    /// Creates a new localization bundle for the given locale.
    ///
    /// # Errors
    ///
    /// Returns [`LocaleError`] if the bundle cannot be created.
    pub fn new(locale: Locale) -> Result<Self, LocaleError> {
        let bundle = create_bundle(locale)?;
        Ok(Self { locale, bundle })
    }

    /// Returns the locale this bundle was created for.
    #[must_use]
    pub const fn locale(&self) -> Locale {
        self.locale
    }
}

impl Message for LocalizationBundle {
    fn get(&self, key: &str) -> Result<String, LocaleError> {
        let message = self
            .bundle
            .get_message(key)
            .ok_or_else(|| LocaleError::MessageNotFound(key.to_string()))?;

        let value = message
            .value()
            .ok_or_else(|| LocaleError::MessageNotFound(key.to_string()))?;

        let mut errors = Vec::new();
        let formatted = self.bundle.format_pattern(value, None, &mut errors);

        if !errors.is_empty() {
            return Err(LocaleError::FormattingError(
                errors
                    .into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
        }

        Ok(formatted.into_owned())
    }

    fn get_with_args(
        &self,
        key: &str,
        args: &HashMap<&str, fluent::FluentValue<'_>>,
    ) -> Result<String, LocaleError> {
        let message = self
            .bundle
            .get_message(key)
            .ok_or_else(|| LocaleError::MessageNotFound(key.to_string()))?;

        let value = message
            .value()
            .ok_or_else(|| LocaleError::MessageNotFound(key.to_string()))?;

        let mut fluent_args = FluentArgs::new();
        for (k, v) in args {
            fluent_args.set(*k, v.clone());
        }

        let mut errors = Vec::new();
        let formatted = self
            .bundle
            .format_pattern(value, Some(&fluent_args), &mut errors);

        if !errors.is_empty() {
            return Err(LocaleError::FormattingError(
                errors
                    .into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
        }

        Ok(formatted.into_owned())
    }

    fn get_attribute(&self, key: &str, attr: &str) -> Result<String, LocaleError> {
        let message = self
            .bundle
            .get_message(key)
            .ok_or_else(|| LocaleError::MessageNotFound(key.to_string()))?;

        let attribute = message
            .get_attribute(attr)
            .ok_or_else(|| LocaleError::AttributeNotFound(key.to_string(), attr.to_string()))?;

        let mut errors = Vec::new();
        let formatted = self
            .bundle
            .format_pattern(attribute.value(), None, &mut errors);

        if !errors.is_empty() {
            return Err(LocaleError::FormattingError(
                errors
                    .into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
        }

        Ok(formatted.into_owned())
    }
}

thread_local! {
    /// Thread-local bundle cache for the default locale.
    ///
    /// Since FluentBundle is not thread-safe, we use thread-local storage.
    /// The bundle is created lazily on first access within each thread.
    static DEFAULT_BUNDLE: RefCell<Option<LocalizationBundle>> = const { RefCell::new(None) };
}

/// Initializes the thread-local bundle if not already created.
fn ensure_bundle() -> Result<(), LocaleError> {
    DEFAULT_BUNDLE.with(|cell| {
        let borrowed = cell.borrow();
        if borrowed.is_some() {
            return Ok(());
        }
        drop(borrowed);

        let locale = Locale::detect();
        let bundle = LocalizationBundle::new(locale)?;
        *cell.borrow_mut() = Some(bundle);
        Ok(())
    })
}

/// Gets a localized message from the thread-local bundle.
///
/// # Errors
///
/// Returns [`LocaleError`] if the message cannot be retrieved.
///
/// # Panics
///
/// Panics if the thread-local bundle has not been initialized (should never happen after `ensure_bundle()` succeeds).
///
/// # Example
///
/// ```ignore
/// let msg = get_message("welcome")?;
/// println!("{}", msg);
/// ```
pub fn get_message(key: &str) -> Result<String, LocaleError> {
    ensure_bundle()?;
    DEFAULT_BUNDLE.with(|cell| {
        let borrowed = cell.borrow();
        let bundle = borrowed
            .as_ref()
            .ok_or_else(|| LocaleError::BundleError("Bundle not initialized".to_string()))?;
        bundle.get(key)
    })
}

/// Gets a localized message with arguments from the thread-local bundle.
///
/// # Errors
///
/// Returns [`LocaleError`] if the message cannot be retrieved or formatted.
///
/// # Panics
///
/// Panics if the thread-local bundle has not been initialized (should never happen after `ensure_bundle()` succeeds).
///
/// # Example
///
/// ```ignore
/// use std::collections::HashMap;
/// use fluent::FluentValue;
///
/// let mut args = HashMap::new();
/// args.insert("name", FluentValue::from("World"));
/// let msg = get_message_with_args("greeting", &args)?;
/// println!("{}", msg);
/// ```
#[allow(clippy::implicit_hasher)]
pub fn get_message_with_args(
    key: &str,
    args: &HashMap<&str, fluent::FluentValue<'_>>,
) -> Result<String, LocaleError> {
    ensure_bundle()?;
    DEFAULT_BUNDLE.with(|cell| {
        let borrowed = cell.borrow();
        let bundle = borrowed
            .as_ref()
            .ok_or_else(|| LocaleError::BundleError("Bundle not initialized".to_string()))?;
        bundle.get_with_args(key, args)
    })
}

/// Gets a localized message attribute from the thread-local bundle.
///
/// # Errors
///
/// Returns [`LocaleError`] if the message or attribute cannot be retrieved.
///
/// # Panics
///
/// Panics if the thread-local bundle has not been initialized (should never happen after `ensure_bundle()` succeeds).
///
/// # Example
///
/// ```ignore
/// let title = get_message_attribute("error-not-found", "title")?;
/// println!("{}", title);
/// ```
pub fn get_message_attribute(key: &str, attr: &str) -> Result<String, LocaleError> {
    ensure_bundle()?;
    DEFAULT_BUNDLE.with(|cell| {
        let borrowed = cell.borrow();
        let bundle = borrowed
            .as_ref()
            .ok_or_else(|| LocaleError::BundleError("Bundle not initialized".to_string()))?;
        bundle.get_attribute(key, attr)
    })
}

/// Creates a Fluent bundle for the given locale.
fn create_bundle(locale: Locale) -> Result<FluentBundle<FluentResource>, LocaleError> {
    let lang_id: LanguageIdentifier = locale
        .language_tag()
        .parse()
        .map_err(LocaleError::InvalidLanguageId)?;

    let mut bundle = FluentBundle::new(vec![lang_id]);

    // Load FTL resources for this locale
    let resources = load_ftl_resources(locale)?;

    for resource in resources {
        bundle.add_resource(resource).map_err(|errors: Vec<_>| {
            LocaleError::BundleError(
                errors
                    .into_iter()
                    .map(|e| format!("{e:?}"))
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        })?;
    }

    Ok(bundle)
}

/// Loads FTL resources for the given locale.
fn load_ftl_resources(locale: Locale) -> Result<Vec<FluentResource>, LocaleError> {
    let mut resources = Vec::new();

    // List of FTL files to load (in order)
    let ftl_files = match locale {
        Locale::EnUS => &["main", "cli", "report", "errors"],
    };

    for file_name in ftl_files {
        let ftl_content = get_ftl_content(locale, file_name)?;
        let resource = FluentResource::try_new(ftl_content).map_err(|(_, e)| {
            LocaleError::ParseError(format!("Failed to parse {file_name}.ftl: {e:?}"))
        })?;
        resources.push(resource);
    }

    Ok(resources)
}

/// Gets the FTL content for a specific file and locale.
///
/// FTL content is embedded at compile time using `include_str!`.
#[allow(clippy::allow_attributes)]
#[allow(dead_code)] // Will be used when FTL files have content
fn get_ftl_content(locale: Locale, file_name: &str) -> Result<String, LocaleError> {
    let content = match locale {
        Locale::EnUS => match file_name {
            "main" => include_str!("locales/en-US/main.ftl"),
            "cli" => include_str!("locales/en-US/cli.ftl"),
            "report" => include_str!("locales/en-US/report.ftl"),
            "errors" => include_str!("locales/en-US/errors.ftl"),
            _ => return Err(LocaleError::ResourceNotFound(file_name.to_string())),
        },
    };

    Ok(content.to_string())
}

/// Errors that can occur during localization operations.
#[derive(Debug, Error)]
pub enum LocaleError {
    /// The requested locale is not supported.
    #[error("Unknown locale: {0}")]
    UnknownLocale(String),

    /// The language identifier is invalid.
    #[error("Invalid language identifier: {0}")]
    InvalidLanguageId(#[source] LanguageIdentifierError),

    /// The requested message was not found.
    #[error("Message not found: {0}")]
    MessageNotFound(String),

    /// The requested attribute was not found.
    #[error("Attribute '{1}' not found on message '{0}'")]
    AttributeNotFound(String, String),

    /// An error occurred during message formatting.
    #[error("Formatting error: {0}")]
    FormattingError(String),

    /// The FTL resource file was not found.
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// An error occurred parsing FTL content.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// An error occurred creating the Fluent bundle.
    #[error("Bundle error: {0}")]
    BundleError(String),
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_locale_default() {
        assert_eq!(Locale::default(), Locale::EnUS);
    }

    #[test]
    fn test_locale_language_tag() {
        assert_eq!(Locale::EnUS.language_tag(), "en-US");
    }

    #[test]
    fn test_locale_from_str() {
        assert_eq!(Locale::from_str("en-US").unwrap(), Locale::EnUS);
        assert_eq!(Locale::from_str("en_US").unwrap(), Locale::EnUS);
        assert_eq!(Locale::from_str("en").unwrap(), Locale::EnUS);
        assert!(matches!(
            Locale::from_str("de-DE"),
            Err(LocaleError::UnknownLocale(_))
        ));
    }

    #[test]
    fn test_locale_display() {
        assert_eq!(format!("{}", Locale::EnUS), "en-US");
    }

    #[test]
    fn test_bundle_creation() {
        let bundle = LocalizationBundle::new(Locale::EnUS);
        assert!(bundle.is_ok());
    }

    #[test]
    fn test_get_message() {
        let result = get_message("brand-name");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "lintdiff");
    }

    #[test]
    fn test_get_message_not_found() {
        let result = get_message("nonexistent-message-key");
        assert!(matches!(result, Err(LocaleError::MessageNotFound(_))));
    }

    #[test]
    fn test_bundle_direct_message() {
        let bundle = LocalizationBundle::new(Locale::EnUS).unwrap();
        let result = bundle.get("brand-name");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "lintdiff");
    }

    #[test]
    fn test_bundle_message_with_args() {
        let bundle = LocalizationBundle::new(Locale::EnUS).unwrap();
        let mut args = HashMap::new();
        args.insert("path", fluent::FluentValue::from("/test/file.rs"));
        let result = bundle.get_with_args("file-not-found", &args);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("/test/file.rs"));
    }
}
