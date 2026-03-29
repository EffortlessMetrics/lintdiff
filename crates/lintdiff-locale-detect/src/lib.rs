//! System locale detection for lintdiff internationalization.
//!
//! This crate provides locale detection capabilities for the lintdiff project,
//! supporting BCP47 language tag formatting and locale parsing.
//!
//! # Example
//!
//! ```rust
//! use lintdiff_locale_detect::{Locale, detect_system_locale, parse_locale};
//!
//! // Detect the system locale
//! let locale = detect_system_locale();
//! println!("System locale: {}", locale);
//!
//! // Parse a locale string
//! let parsed = parse_locale("en-US").unwrap();
//! assert_eq!(parsed.language, "en");
//! assert_eq!(parsed.region, Some("US".to_string()));
//!
//! // Create a locale manually
//! let locale = Locale::with_region("es", "ES");
//! assert_eq!(locale.to_bcp47(), "es-ES");
//! ```

#![warn(missing_docs)]

/// Detected locale information.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Locale {
    /// Language code (e.g., "en", "es", "fr").
    pub language: String,
    /// Optional region code (e.g., "US", "ES", "FR").
    pub region: Option<String>,
    /// Optional script code (e.g., "Latn", "Cyrl").
    pub script: Option<String>,
}

impl Locale {
    /// Create a new locale with just a language code.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lintdiff_locale_detect::Locale;
    ///
    /// let locale = Locale::new("en");
    /// assert_eq!(locale.language, "en");
    /// assert_eq!(locale.region, None);
    /// ```
    #[must_use]
    pub fn new(language: impl Into<String>) -> Self {
        Self {
            language: language.into().to_lowercase(),
            region: None,
            script: None,
        }
    }

    /// Create a locale with language and region.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lintdiff_locale_detect::Locale;
    ///
    /// let locale = Locale::with_region("en", "US");
    /// assert_eq!(locale.language, "en");
    /// assert_eq!(locale.region, Some("US".to_string()));
    /// ```
    #[must_use]
    pub fn with_region(language: impl Into<String>, region: impl Into<String>) -> Self {
        Self {
            language: language.into().to_lowercase(),
            region: Some(region.into().to_uppercase()),
            script: None,
        }
    }

    /// Create a locale with language, region, and script.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lintdiff_locale_detect::Locale;
    ///
    /// let locale = Locale::with_script("sr", "RS", "Cyrl");
    /// assert_eq!(locale.language, "sr");
    /// assert_eq!(locale.region, Some("RS".to_string()));
    /// assert_eq!(locale.script, Some("Cyrl".to_string()));
    /// ```
    #[must_use]
    pub fn with_script(language: impl Into<String>, region: impl Into<String>, script: impl Into<String>) -> Self {
        Self {
            language: language.into().to_lowercase(),
            region: Some(region.into().to_uppercase()),
            script: Some(script.into()),
        }
    }

    /// Get the locale as a BCP47 language tag (e.g., "en-US").
    ///
    /// BCP47 format: `language[-script][-region]`
    ///
    /// # Example
    ///
    /// ```rust
    /// use lintdiff_locale_detect::Locale;
    ///
    /// let locale = Locale::with_region("en", "US");
    /// assert_eq!(locale.to_bcp47(), "en-US");
    ///
    /// let locale = Locale::new("fr");
    /// assert_eq!(locale.to_bcp47(), "fr");
    /// ```
    #[must_use]
    pub fn to_bcp47(&self) -> String {
        let mut parts = vec![self.language.clone()];
        
        if let Some(script) = &self.script {
            parts.push(script.clone());
        }
        
        if let Some(region) = &self.region {
            parts.push(region.clone());
        }
        
        parts.join("-")
    }

    /// Get the locale as a Fluent locale string (e.g., "en-US").
    ///
    /// This is an alias for [`to_bcp47`][Self::to_bcp47] since Fluent uses BCP47 format.
    ///
    /// # Example
    ///
    /// ```rust
    /// use lintdiff_locale_detect::Locale;
    ///
    /// let locale = Locale::with_region("en", "US");
    /// assert_eq!(locale.to_fluent_locale(), "en-US");
    /// ```
    #[must_use]
    pub fn to_fluent_locale(&self) -> String {
        self.to_bcp47()
    }

    /// Check if this locale matches another (language only).
    ///
    /// # Example
    ///
    /// ```rust
    /// use lintdiff_locale_detect::Locale;
    ///
    /// let locale = Locale::with_region("en", "US");
    /// assert!(locale.matches_language("en"));
    /// assert!(locale.matches_language("EN")); // Case insensitive
    /// assert!(!locale.matches_language("es"));
    /// ```
    #[must_use]
    pub fn matches_language(&self, other: &str) -> bool {
        self.language.eq_ignore_ascii_case(other)
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::new("en")
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_bcp47())
    }
}

/// Detect the system locale.
///
/// This function uses the `sys-locale` crate to detect the user's preferred
/// locale from the operating system. If detection fails or returns an invalid
/// locale, it falls back to English ("en").
///
/// # Example
///
/// ```rust
/// use lintdiff_locale_detect::detect_system_locale;
///
/// let locale = detect_system_locale();
/// println!("Detected locale: {}", locale);
/// ```
#[must_use]
pub fn detect_system_locale() -> Locale {
    sys_locale::get_locale()
        .and_then(|s| parse_locale(&s))
        .unwrap_or_default()
}

/// Get the default locale (English).
///
/// # Example
///
/// ```rust
/// use lintdiff_locale_detect::default_locale;
///
/// let locale = default_locale();
/// assert_eq!(locale.language, "en");
/// ```
#[must_use]
pub fn default_locale() -> Locale {
    Locale::default()
}

/// Parse a locale string (e.g., "en-US", "`es_ES`", "fr").
///
/// This function handles various locale string formats:
/// - BCP47 format: `en-US`, `zh-Hans-CN`
/// - POSIX format: `en_US`, `es_ES`
/// - Language only: `en`, `fr`, `de`
///
/// # Example
///
/// ```rust
/// use lintdiff_locale_detect::parse_locale;
///
/// // BCP47 format
/// let locale = parse_locale("en-US").unwrap();
/// assert_eq!(locale.language, "en");
/// assert_eq!(locale.region, Some("US".to_string()));
///
/// // POSIX format
/// let locale = parse_locale("es_ES").unwrap();
/// assert_eq!(locale.language, "es");
/// assert_eq!(locale.region, Some("ES".to_string()));
///
/// // Language only
/// let locale = parse_locale("fr").unwrap();
/// assert_eq!(locale.language, "fr");
/// assert_eq!(locale.region, None);
/// ```
#[must_use]
pub fn parse_locale(s: &str) -> Option<Locale> {
    let s = s.trim();
    
    if s.is_empty() {
        return None;
    }
    
    // Handle encoding suffix (e.g., "en_US.UTF-8")
    let s = s.split('.').next()?.trim();
    
    // Try BCP47 format first (language-Script-Region or language-Region)
    if s.contains('-') {
        let parts: Vec<&str> = s.split('-').collect();
        return parse_bcp47_parts(&parts);
    }
    
    // Try POSIX format (language_Script_Region or language_Region)
    if s.contains('_') {
        let parts: Vec<&str> = s.split('_').collect();
        return parse_posix_parts(&parts);
    }
    
    // Just a language code
    if is_valid_language_code(s) {
        Some(Locale::new(s))
    } else {
        None
    }
}

/// Parse BCP47 format parts (e.g., `[en, US]` or `[zh, Hans, CN]`)
fn parse_bcp47_parts(parts: &[&str]) -> Option<Locale> {
    if parts.is_empty() {
        return None;
    }
    
    let language = parts[0];
    if !is_valid_language_code(language) {
        return None;
    }
    
    match parts.len() {
        1 => Some(Locale::new(language)),
        2 => {
            let second = parts[1];
            // Check if it's a script (4 letters, title case) or region (2-3 letters/numbers)
            if is_valid_script_code(second) {
                Some(Locale {
                    language: language.to_lowercase(),
                    region: None,
                    script: Some(second.to_string()),
                })
            } else if is_valid_region_code(second) {
                Some(Locale::with_region(language, second))
            } else {
                None
            }
        }
        3 => {
            let second = parts[1];
            let third = parts[2];
            
            // Format: language-script-region
            if is_valid_script_code(second) && is_valid_region_code(third) {
                Some(Locale::with_script(language, third, second))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Parse POSIX format parts (e.g., `[en, US]` or `[zh, Hans, CN]`)
fn parse_posix_parts(parts: &[&str]) -> Option<Locale> {
    if parts.is_empty() {
        return None;
    }
    
    let language = parts[0];
    if !is_valid_language_code(language) {
        return None;
    }
    
    match parts.len() {
        1 => Some(Locale::new(language)),
        2 => {
            let second = parts[1];
            // In POSIX format, second part is typically the region
            if is_valid_region_code(second) {
                Some(Locale::with_region(language, second))
            } else if is_valid_script_code(second) {
                Some(Locale {
                    language: language.to_lowercase(),
                    region: None,
                    script: Some(second.to_string()),
                })
            } else {
                None
            }
        }
        3 => {
            let second = parts[1];
            let third = parts[2];
            
            // Format: language_script_region
            if is_valid_script_code(second) && is_valid_region_code(third) {
                Some(Locale::with_script(language, third, second))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Check if a string is a valid ISO 639 language code (2-3 letters)
fn is_valid_language_code(s: &str) -> bool {
    let len = s.len();
    (len == 2 || len == 3) && s.chars().all(|c| c.is_ascii_alphabetic())
}

/// Check if a string is a valid ISO 15924 script code (4 letters, title case)
fn is_valid_script_code(s: &str) -> bool {
    s.len() == 4
        && s.chars().next().is_some_and(char::is_uppercase)
        && s.chars().skip(1).all(char::is_lowercase)
}

/// Check if a string is a valid region code (2 letters or 3 digits)
fn is_valid_region_code(s: &str) -> bool {
    let len = s.len();
    (len == 2 && s.chars().all(|c| c.is_ascii_alphabetic()))
        || (len == 3 && s.chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_new() {
        let locale = Locale::new("en");
        assert_eq!(locale.language, "en");
        assert_eq!(locale.region, None);
        assert_eq!(locale.script, None);
    }

    #[test]
    fn test_locale_new_normalizes_case() {
        let locale = Locale::new("EN");
        assert_eq!(locale.language, "en");
    }

    #[test]
    fn test_locale_with_region() {
        let locale = Locale::with_region("en", "us");
        assert_eq!(locale.language, "en");
        assert_eq!(locale.region, Some("US".to_string()));
        assert_eq!(locale.script, None);
    }

    #[test]
    fn test_locale_with_script() {
        let locale = Locale::with_script("sr", "rs", "Cyrl");
        assert_eq!(locale.language, "sr");
        assert_eq!(locale.region, Some("RS".to_string()));
        assert_eq!(locale.script, Some("Cyrl".to_string()));
    }

    #[test]
    fn test_to_bcp47() {
        assert_eq!(Locale::new("en").to_bcp47(), "en");
        assert_eq!(Locale::with_region("en", "US").to_bcp47(), "en-US");
        assert_eq!(Locale::with_script("zh", "CN", "Hans").to_bcp47(), "zh-Hans-CN");
    }

    #[test]
    fn test_to_fluent_locale() {
        assert_eq!(Locale::with_region("en", "US").to_fluent_locale(), "en-US");
    }

    #[test]
    fn test_matches_language() {
        let locale = Locale::with_region("en", "US");
        assert!(locale.matches_language("en"));
        assert!(locale.matches_language("EN"));
        assert!(!locale.matches_language("es"));
    }

    #[test]
    fn test_default() {
        let locale = Locale::default();
        assert_eq!(locale.language, "en");
        assert_eq!(locale.region, None);
    }

    #[test]
    fn test_display() {
        let locale = Locale::with_region("en", "US");
        assert_eq!(format!("{}", locale), "en-US");
    }

    #[test]
    fn test_parse_locale_bcp47() {
        let locale = parse_locale("en-US").unwrap();
        assert_eq!(locale.language, "en");
        assert_eq!(locale.region, Some("US".to_string()));
    }

    #[test]
    fn test_parse_locale_posix() {
        let locale = parse_locale("en_US").unwrap();
        assert_eq!(locale.language, "en");
        assert_eq!(locale.region, Some("US".to_string()));
    }

    #[test]
    fn test_parse_locale_with_encoding() {
        let locale = parse_locale("en_US.UTF-8").unwrap();
        assert_eq!(locale.language, "en");
        assert_eq!(locale.region, Some("US".to_string()));
    }

    #[test]
    fn test_parse_locale_language_only() {
        let locale = parse_locale("fr").unwrap();
        assert_eq!(locale.language, "fr");
        assert_eq!(locale.region, None);
    }

    #[test]
    fn test_parse_locale_with_script() {
        let locale = parse_locale("zh-Hans-CN").unwrap();
        assert_eq!(locale.language, "zh");
        assert_eq!(locale.script, Some("Hans".to_string()));
        assert_eq!(locale.region, Some("CN".to_string()));
    }

    #[test]
    fn test_parse_locale_invalid() {
        assert_eq!(parse_locale(""), None);
        assert_eq!(parse_locale("   "), None);
        assert_eq!(parse_locale("a"), None); // Too short
    }

    #[test]
    fn test_detect_system_locale() {
        // Just verify it doesn't panic and returns something valid
        let locale = detect_system_locale();
        assert!(!locale.language.is_empty());
    }

    #[test]
    fn test_default_locale() {
        let locale = default_locale();
        assert_eq!(locale.language, "en");
    }
}
