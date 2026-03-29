//! Comprehensive tests for the lintdiff-locale-detect crate.

use lintdiff_locale_detect::{default_locale, detect_system_locale, parse_locale, Locale};

// ============================================================================
// Locale Creation Tests
// ============================================================================

#[test]
fn test_locale_new_creates_language_only_locale() {
    let locale = Locale::new("en");
    assert_eq!(locale.language, "en");
    assert_eq!(locale.region, None);
    assert_eq!(locale.script, None);
}

#[test]
fn test_locale_new_normalizes_to_lowercase() {
    let locale = Locale::new("EN");
    assert_eq!(locale.language, "en");

    let locale = Locale::new("En");
    assert_eq!(locale.language, "en");
}

#[test]
fn test_locale_new_with_three_letter_code() {
    let locale = Locale::new("ast"); // Asturian
    assert_eq!(locale.language, "ast");
}

#[test]
fn test_locale_with_region_creates_locale_with_region() {
    let locale = Locale::with_region("es", "ES");
    assert_eq!(locale.language, "es");
    assert_eq!(locale.region, Some("ES".to_string()));
    assert_eq!(locale.script, None);
}

#[test]
fn test_locale_with_region_normalizes_case() {
    let locale = Locale::with_region("ES", "es");
    assert_eq!(locale.language, "es");
    assert_eq!(locale.region, Some("ES".to_string()));
}

#[test]
fn test_locale_with_script_creates_full_locale() {
    let locale = Locale::with_script("sr", "RS", "Cyrl");
    assert_eq!(locale.language, "sr");
    assert_eq!(locale.region, Some("RS".to_string()));
    assert_eq!(locale.script, Some("Cyrl".to_string()));
}

#[test]
fn test_locale_with_script_chinese_simplified() {
    let locale = Locale::with_script("zh", "CN", "Hans");
    assert_eq!(locale.language, "zh");
    assert_eq!(locale.region, Some("CN".to_string()));
    assert_eq!(locale.script, Some("Hans".to_string()));
}

// ============================================================================
// BCP47 Formatting Tests
// ============================================================================

#[test]
fn test_to_bcp47_language_only() {
    let locale = Locale::new("en");
    assert_eq!(locale.to_bcp47(), "en");
}

#[test]
fn test_to_bcp47_with_region() {
    let locale = Locale::with_region("en", "US");
    assert_eq!(locale.to_bcp47(), "en-US");
}

#[test]
fn test_to_bcp47_with_script_and_region() {
    let locale = Locale::with_script("zh", "CN", "Hans");
    assert_eq!(locale.to_bcp47(), "zh-Hans-CN");
}

#[test]
fn test_to_bcp47_various_locales() {
    assert_eq!(Locale::with_region("es", "ES").to_bcp47(), "es-ES");
    assert_eq!(Locale::with_region("fr", "FR").to_bcp47(), "fr-FR");
    assert_eq!(Locale::with_region("de", "DE").to_bcp47(), "de-DE");
    assert_eq!(Locale::with_region("ja", "JP").to_bcp47(), "ja-JP");
    assert_eq!(Locale::with_region("ko", "KR").to_bcp47(), "ko-KR");
    assert_eq!(Locale::with_region("pt", "BR").to_bcp47(), "pt-BR");
}

// ============================================================================
// Fluent Locale Tests
// ============================================================================

#[test]
fn test_to_fluent_locale_matches_bcp47() {
    let locale = Locale::with_region("en", "US");
    assert_eq!(locale.to_fluent_locale(), locale.to_bcp47());
}

#[test]
fn test_to_fluent_locale_returns_bcp47_format() {
    assert_eq!(Locale::new("en").to_fluent_locale(), "en");
    assert_eq!(Locale::with_region("es", "ES").to_fluent_locale(), "es-ES");
}

// ============================================================================
// Language Matching Tests
// ============================================================================

#[test]
fn test_matches_language_returns_true_for_same_language() {
    let locale = Locale::with_region("en", "US");
    assert!(locale.matches_language("en"));
}

#[test]
fn test_matches_language_is_case_insensitive() {
    let locale = Locale::with_region("en", "US");
    assert!(locale.matches_language("EN"));
    assert!(locale.matches_language("En"));
    assert!(locale.matches_language("eN"));
}

#[test]
fn test_matches_language_returns_false_for_different_language() {
    let locale = Locale::with_region("en", "US");
    assert!(!locale.matches_language("es"));
    assert!(!locale.matches_language("fr"));
    assert!(!locale.matches_language("de"));
}

#[test]
fn test_matches_language_ignores_region() {
    let locale = Locale::with_region("en", "GB");
    assert!(locale.matches_language("en"));

    let locale = Locale::with_region("en", "US");
    assert!(locale.matches_language("en"));
}

#[test]
fn test_matches_language_ignores_script() {
    let locale = Locale::with_script("zh", "CN", "Hans");
    assert!(locale.matches_language("zh"));
}

// ============================================================================
// Default Tests
// ============================================================================

#[test]
fn test_default_returns_english() {
    let locale = Locale::default();
    assert_eq!(locale.language, "en");
    assert_eq!(locale.region, None);
    assert_eq!(locale.script, None);
}

#[test]
fn test_default_locale_function() {
    let locale = default_locale();
    assert_eq!(locale.language, "en");
    assert_eq!(locale.region, None);
}

// ============================================================================
// Display Tests
// ============================================================================

#[test]
fn test_display_uses_bcp47_format() {
    let locale = Locale::with_region("en", "US");
    assert_eq!(format!("{}", locale), "en-US");
}

#[test]
fn test_display_for_language_only() {
    let locale = Locale::new("fr");
    assert_eq!(format!("{}", locale), "fr");
}

#[test]
fn test_display_for_complex_locale() {
    let locale = Locale::with_script("zh", "TW", "Hant");
    assert_eq!(format!("{}", locale), "zh-Hant-TW");
}

// ============================================================================
// Parse Locale Tests - BCP47 Format
// ============================================================================

#[test]
fn test_parse_locale_bcp47_language_region() {
    let locale = parse_locale("en-US").unwrap();
    assert_eq!(locale.language, "en");
    assert_eq!(locale.region, Some("US".to_string()));
    assert_eq!(locale.script, None);
}

#[test]
fn test_parse_locale_bcp47_language_only() {
    let locale = parse_locale("en").unwrap();
    assert_eq!(locale.language, "en");
    assert_eq!(locale.region, None);
}

#[test]
fn test_parse_locale_bcp47_with_script() {
    let locale = parse_locale("zh-Hans").unwrap();
    assert_eq!(locale.language, "zh");
    assert_eq!(locale.script, Some("Hans".to_string()));
    assert_eq!(locale.region, None);
}

#[test]
fn test_parse_locale_bcp47_full_format() {
    let locale = parse_locale("zh-Hans-CN").unwrap();
    assert_eq!(locale.language, "zh");
    assert_eq!(locale.script, Some("Hans".to_string()));
    assert_eq!(locale.region, Some("CN".to_string()));
}

#[test]
fn test_parse_locale_bcp47_serbian_cyrillic() {
    let locale = parse_locale("sr-Cyrl-RS").unwrap();
    assert_eq!(locale.language, "sr");
    assert_eq!(locale.script, Some("Cyrl".to_string()));
    assert_eq!(locale.region, Some("RS".to_string()));
}

// ============================================================================
// Parse Locale Tests - POSIX Format
// ============================================================================

#[test]
fn test_parse_locale_posix_language_region() {
    let locale = parse_locale("en_US").unwrap();
    assert_eq!(locale.language, "en");
    assert_eq!(locale.region, Some("US".to_string()));
}

#[test]
fn test_parse_locale_posix_with_encoding() {
    let locale = parse_locale("en_US.UTF-8").unwrap();
    assert_eq!(locale.language, "en");
    assert_eq!(locale.region, Some("US".to_string()));
}

#[test]
fn test_parse_locale_posix_with_different_encodings() {
    let locale = parse_locale("en_US.ISO-8859-1").unwrap();
    assert_eq!(locale.language, "en");
    assert_eq!(locale.region, Some("US".to_string()));
}

#[test]
fn test_parse_locale_posix_various_locales() {
    let locale = parse_locale("es_ES").unwrap();
    assert_eq!(locale.language, "es");
    assert_eq!(locale.region, Some("ES".to_string()));

    let locale = parse_locale("fr_FR").unwrap();
    assert_eq!(locale.language, "fr");
    assert_eq!(locale.region, Some("FR".to_string()));

    let locale = parse_locale("de_DE").unwrap();
    assert_eq!(locale.language, "de");
    assert_eq!(locale.region, Some("DE".to_string()));
}

// ============================================================================
// Parse Locale Tests - Edge Cases
// ============================================================================

#[test]
fn test_parse_locale_empty_string() {
    assert_eq!(parse_locale(""), None);
}

#[test]
fn test_parse_locale_whitespace_only() {
    assert_eq!(parse_locale("   "), None);
}

#[test]
fn test_parse_locale_trims_whitespace() {
    let locale = parse_locale("  en-US  ").unwrap();
    assert_eq!(locale.language, "en");
    assert_eq!(locale.region, Some("US".to_string()));
}

#[test]
fn test_parse_locale_invalid_too_short() {
    assert_eq!(parse_locale("a"), None);
}

#[test]
fn test_parse_locale_three_letter_language() {
    let locale = parse_locale("ast-ES").unwrap(); // Asturian
    assert_eq!(locale.language, "ast");
    assert_eq!(locale.region, Some("ES".to_string()));
}

#[test]
fn test_parse_locale_numeric_region() {
    // ISO 3166-1 numeric codes (e.g., 419 for Latin America)
    let locale = parse_locale("es-419").unwrap();
    assert_eq!(locale.language, "es");
    assert_eq!(locale.region, Some("419".to_string()));
}

// ============================================================================
// System Locale Detection Tests
// ============================================================================

#[test]
fn test_detect_system_locale_returns_valid_locale() {
    let locale = detect_system_locale();
    // Should return a valid locale (not empty language)
    assert!(!locale.language.is_empty());
}

#[test]
fn test_detect_system_locale_never_panics() {
    // This test ensures the function handles all edge cases gracefully
    let _locale = detect_system_locale();
}

// ============================================================================
// Clone and PartialEq Tests
// ============================================================================

#[test]
fn test_locale_clone() {
    let locale = Locale::with_region("en", "US");
    let cloned = locale.clone();
    assert_eq!(locale, cloned);
}

#[test]
fn test_locale_equality() {
    let locale1 = Locale::with_region("en", "US");
    let locale2 = Locale::with_region("en", "US");
    assert_eq!(locale1, locale2);
}

#[test]
fn test_locale_inequality_language() {
    let locale1 = Locale::new("en");
    let locale2 = Locale::new("es");
    assert_ne!(locale1, locale2);
}

#[test]
fn test_locale_inequality_region() {
    let locale1 = Locale::with_region("en", "US");
    let locale2 = Locale::with_region("en", "GB");
    assert_ne!(locale1, locale2);
}

#[test]
fn test_locale_inequality_script() {
    let locale1 = Locale::with_script("sr", "RS", "Cyrl");
    let locale2 = Locale::with_script("sr", "RS", "Latn");
    assert_ne!(locale1, locale2);
}

// ============================================================================
// Debug Tests
// ============================================================================

#[test]
fn test_locale_debug() {
    let locale = Locale::with_region("en", "US");
    let debug_str = format!("{:?}", locale);
    assert!(debug_str.contains("en"));
    assert!(debug_str.contains("US"));
}
