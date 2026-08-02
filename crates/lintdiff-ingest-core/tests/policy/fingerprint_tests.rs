//! Comprehensive tests for fingerprint generation.
//!
//! These tests cover:
//! - Fingerprint generation determinism
//! - Fingerprint stability across runs
//! - Different inputs produce different fingerprints
//! - Whitespace normalization

use lintdiff_ingest_core::fingerprint;
use lintdiff_types::{Location, NormPath};

// =============================================================================
// Determinism tests
// =============================================================================

mod determinism {
    use super::*;

    #[test]
    fn same_inputs_produce_same_fingerprint() {
        let loc = Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(42),
            col: None,
        };
        let a = fingerprint("test.code", Some(&loc), "test message");
        let b = fingerprint("test.code", Some(&loc), "test message");
        assert_eq!(a, b);
    }

    #[test]
    fn fingerprint_is_deterministic_multiple_calls() {
        let loc = Location {
            path: NormPath::new("src/main.rs"),
            line: Some(1),
            col: Some(1),
        };

        // Call multiple times to ensure determinism
        let results: Vec<String> = (0..10)
            .map(|_| fingerprint("clippy::test", Some(&loc), "message"))
            .collect();

        let first = &results[0];
        assert!(results.iter().all(|r| r == first));
    }

    #[test]
    fn none_location_produces_consistent_fingerprint() {
        let a = fingerprint("test.code", None, "message");
        let b = fingerprint("test.code", None, "message");
        assert_eq!(a, b);
    }
}

// =============================================================================
// Whitespace normalization tests
// =============================================================================

mod whitespace_normalization {
    use super::*;

    #[test]
    fn leading_whitespace_normalized() {
        let fp1 = fingerprint("code", None, "  message");
        let fp2 = fingerprint("code", None, "message");
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn trailing_whitespace_normalized() {
        let fp1 = fingerprint("code", None, "message  ");
        let fp2 = fingerprint("code", None, "message");
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn tabs_converted_to_spaces() {
        let fp1 = fingerprint("code", None, "one\ttwo");
        let fp2 = fingerprint("code", None, "one two");
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn newlines_converted_to_spaces() {
        let fp1 = fingerprint("code", None, "one\ntwo");
        let fp2 = fingerprint("code", None, "one two");
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn carriage_returns_converted_to_spaces() {
        let fp1 = fingerprint("code", None, "one\rtwo");
        let fp2 = fingerprint("code", None, "one two");
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn multiple_whitespace_collapsed() {
        let fp1 = fingerprint("code", None, "one   two    three");
        let fp2 = fingerprint("code", None, "one two three");
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn mixed_whitespace_normalized() {
        let fp1 = fingerprint("code", None, "  one\t\ttwo\n\nthree  ");
        let fp2 = fingerprint("code", None, "one two three");
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn empty_message_whitespace_only() {
        let fp1 = fingerprint("code", None, "   \t\n  ");
        let fp2 = fingerprint("code", None, "");
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn complex_whitespace_scenario() {
        let loc = Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(10),
            col: None,
        };

        let messy = "  this\tis  a\nmessage\r\nwith  \t  weird   whitespace  ";
        let clean = "this is a message with weird whitespace";

        let fp1 = fingerprint("test.code", Some(&loc), messy);
        let fp2 = fingerprint("test.code", Some(&loc), clean);

        assert_eq!(fp1, fp2);
    }
}

// =============================================================================
// Different inputs produce different fingerprints
// =============================================================================

mod uniqueness {
    use super::*;

    #[test]
    fn different_codes_different_fingerprints() {
        let loc = Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(1),
            col: None,
        };
        let a = fingerprint("code.a", Some(&loc), "msg");
        let b = fingerprint("code.b", Some(&loc), "msg");
        assert_ne!(a, b);
    }

    #[test]
    fn different_paths_different_fingerprints() {
        let loc1 = Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(1),
            col: None,
        };
        let loc2 = Location {
            path: NormPath::new("src/main.rs"),
            line: Some(1),
            col: None,
        };
        let a = fingerprint("code", Some(&loc1), "msg");
        let b = fingerprint("code", Some(&loc2), "msg");
        assert_ne!(a, b);
    }

    #[test]
    fn different_lines_different_fingerprints() {
        let loc1 = Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(1),
            col: None,
        };
        let loc2 = Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(2),
            col: None,
        };
        let a = fingerprint("code", Some(&loc1), "msg");
        let b = fingerprint("code", Some(&loc2), "msg");
        assert_ne!(a, b);
    }

    #[test]
    fn different_messages_different_fingerprints() {
        let loc = Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(1),
            col: None,
        };
        let a = fingerprint("code", Some(&loc), "message a");
        let b = fingerprint("code", Some(&loc), "message b");
        assert_ne!(a, b);
    }

    #[test]
    fn with_and_without_location_different_fingerprints() {
        let loc = Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(1),
            col: None,
        };
        let a = fingerprint("code", Some(&loc), "msg");
        let b = fingerprint("code", None, "msg");
        assert_ne!(a, b);
    }

    #[test]
    fn line_none_vs_some_different_fingerprints() {
        let loc1 = Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(1),
            col: None,
        };
        let loc2 = Location {
            path: NormPath::new("src/lib.rs"),
            line: None,
            col: None,
        };
        let a = fingerprint("code", Some(&loc1), "msg");
        let b = fingerprint("code", Some(&loc2), "msg");
        assert_ne!(a, b);
    }

    #[test]
    fn similar_codes_different_fingerprints() {
        let a = fingerprint("lintdiff.diagnostic.clippy.needless_borrow", None, "msg");
        let b = fingerprint("lintdiff.diagnostic.clippy.needless_borrows", None, "msg");
        assert_ne!(a, b);
    }
}

// =============================================================================
// Fingerprint format tests
// =============================================================================

mod format {
    use super::*;

    #[test]
    fn fingerprint_is_hex_string() {
        let fp = fingerprint("code", None, "message");
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn fingerprint_is_64_characters() {
        // SHA-256 produces 32 bytes = 64 hex characters
        let fp = fingerprint("code", None, "message");
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn fingerprint_is_lowercase() {
        let fp = fingerprint("code", None, "message");
        assert!(fp.chars().all(|c| !c.is_ascii_uppercase()));
    }
}

// =============================================================================
// Stability tests
// =============================================================================

mod stability {
    use super::*;

    #[test]
    fn reference_vector_known_fingerprint() {
        // This test ensures the fingerprint algorithm doesn't change unexpectedly
        let fp = fingerprint("lintdiff.diagnostic.unknown", None, "message");
        // This is a known stable value - if this changes, it's a breaking change
        assert_eq!(
            fp,
            "34415bcd691d11774caf32d55122d0540df005ff0c100a9eb9c7c3af3131d725"
        );
    }

    #[test]
    fn reference_vector_with_location() {
        let loc = Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(9),
            col: Some(1),
        };
        let fp = fingerprint(
            "lintdiff.diagnostic.clippy.needless_borrow",
            Some(&loc),
            "one two three",
        );
        // Verify it produces a consistent result
        let expected = fingerprint(
            "lintdiff.diagnostic.clippy.needless_borrow",
            Some(&loc),
            "one two three",
        );
        assert_eq!(fp, expected);
    }

    #[test]
    fn clippy_code_stability() {
        let fp1 = fingerprint("lintdiff.diagnostic.clippy.needless_borrow", None, "test");
        let fp2 = fingerprint("lintdiff.diagnostic.clippy.needless_borrow", None, "test");
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn rustc_code_stability() {
        let fp1 = fingerprint("lintdiff.diagnostic.rustc.E0502", None, "test");
        let fp2 = fingerprint("lintdiff.diagnostic.rustc.E0502", None, "test");
        assert_eq!(fp1, fp2);
    }
}

// =============================================================================
// Edge cases
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn empty_code() {
        let fp = fingerprint("", None, "message");
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn empty_message() {
        let fp = fingerprint("code", None, "");
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn empty_code_and_message() {
        let fp = fingerprint("", None, "");
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn very_long_code() {
        let long_code = "a".repeat(1000);
        let fp = fingerprint(&long_code, None, "message");
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn very_long_message() {
        let long_msg = "x".repeat(10000);
        let fp = fingerprint("code", None, &long_msg);
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn unicode_in_code() {
        let fp = fingerprint("code_αβγ", None, "message");
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn unicode_in_message() {
        let fp = fingerprint("code", None, "message with unicode: 日本語 🦀");
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn unicode_in_path() {
        let loc = Location {
            path: NormPath::new("src/日本語/lib.rs"),
            line: Some(1),
            col: None,
        };
        let fp = fingerprint("code", Some(&loc), "message");
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn special_characters_in_message() {
        let fp = fingerprint("code", None, "message with special chars: \x00\x01\x02");
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn high_line_number() {
        let loc = Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(u32::MAX),
            col: None,
        };
        let fp = fingerprint("code", Some(&loc), "message");
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn line_zero() {
        let loc = Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(0),
            col: None,
        };
        let fp = fingerprint("code", Some(&loc), "message");
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn path_with_special_characters() {
        let loc = Location {
            path: NormPath::new("src/path with spaces/and-dashes/lib.rs"),
            line: Some(1),
            col: None,
        };
        let fp = fingerprint("code", Some(&loc), "message");
        assert_eq!(fp.len(), 64);
    }

    #[test]
    fn deeply_nested_path() {
        let loc = Location {
            path: NormPath::new("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/lib.rs"),
            line: Some(1),
            col: None,
        };
        let fp = fingerprint("code", Some(&loc), "message");
        assert_eq!(fp.len(), 64);
    }
}

// =============================================================================
// Location handling tests
// =============================================================================

mod location_handling {
    use super::*;

    #[test]
    fn column_not_included_in_fingerprint() {
        // Column is not used in fingerprint calculation
        let loc1 = Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(10),
            col: Some(1),
        };
        let loc2 = Location {
            path: NormPath::new("src/lib.rs"),
            line: Some(10),
            col: Some(99),
        };
        let fp1 = fingerprint("code", Some(&loc1), "msg");
        let fp2 = fingerprint("code", Some(&loc2), "msg");
        // Column is not included, so these should be the same
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn path_order_matters() {
        let loc = Location {
            path: NormPath::new("a/b/c.rs"),
            line: Some(1),
            col: None,
        };
        let fp = fingerprint("code", Some(&loc), "msg");

        // Different path should produce different fingerprint
        let loc2 = Location {
            path: NormPath::new("c/b/a.rs"),
            line: Some(1),
            col: None,
        };
        let fp2 = fingerprint("code", Some(&loc2), "msg");
        assert_ne!(fp, fp2);
    }
}
