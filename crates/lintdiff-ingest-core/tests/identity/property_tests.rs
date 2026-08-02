//! Property-based tests for fingerprint generation.
//!
//! These tests use proptest to verify invariants across a wide range of inputs.

use lintdiff_ingest_core::fingerprint;
use lintdiff_types::{Location, NormPath};
use proptest::prelude::*;

// =============================================================================
// Fingerprint Determinism Tests
// =============================================================================

proptest! {
    /// Same inputs must always produce the same fingerprint (determinism).
    #[test]
    fn fingerprint_deterministic_simple(
        code in "[A-Za-z0-9._:-]{1,40}",
        path in "[A-Za-z0-9_./-]{1,40}",
        line in prop::option::of(1u32..10000),
        message in "[ -~\\t\\n\\r]{0,200}",
    ) {
        let loc = line.map(|l| Location {
            path: NormPath::new(&path),
            line: Some(l),
            col: None,
        });

        let fp1 = fingerprint(&code, loc.as_ref(), &message);
        let fp2 = fingerprint(&code, loc.as_ref(), &message);
        prop_assert_eq!(fp1, fp2, "Same inputs must produce identical fingerprints");
    }

    /// Determinism with all components including column.
    #[test]
    fn fingerprint_deterministic_with_column(
        code in "[A-Za-z0-9._:-]{1,40}",
        path in "[A-Za-z0-9_./-]{1,40}",
        line in 1u32..10000,
        col in 1u32..200,
        message in "[ -~\\t\\n\\r]{0,200}",
    ) {
        let loc = Location {
            path: NormPath::new(&path),
            line: Some(line),
            col: Some(col),
        };

        let fp1 = fingerprint(&code, Some(&loc), &message);
        let fp2 = fingerprint(&code, Some(&loc), &message);
        prop_assert_eq!(fp1, fp2);
    }

    /// Determinism without location.
    #[test]
    fn fingerprint_deterministic_no_location(
        code in "[A-Za-z0-9._:-]{1,40}",
        message in "[ -~\\t\\n\\r]{0,200}",
    ) {
        let fp1 = fingerprint(&code, None, &message);
        let fp2 = fingerprint(&code, None, &message);
        prop_assert_eq!(fp1, fp2);
    }
}

// =============================================================================
// Fingerprint Uniqueness Tests
// =============================================================================

proptest! {
    /// Different codes should produce different fingerprints (with same other inputs).
    #[test]
    fn different_codes_different_fingerprints(
        code1 in "[A-Z]{3,10}",
        code2 in "[A-Z]{3,10}",
        path in "[A-Za-z0-9_./-]{1,40}",
        line in 1u32..1000,
        message in "[A-Za-z ]{5,50}",
    ) {
        prop_assume!(code1 != code2);

        let loc = Location {
            path: NormPath::new(&path),
            line: Some(line),
            col: None,
        };

        let fp1 = fingerprint(&code1, Some(&loc), &message);
        let fp2 = fingerprint(&code2, Some(&loc), &message);
        prop_assert_ne!(fp1, fp2, "Different codes must produce different fingerprints");
    }

    /// Different paths should produce different fingerprints.
    #[test]
    fn different_paths_different_fingerprints(
        code in "[A-Z]{3,10}",
        path1 in "[a-z]{3,10}\\.rs",
        path2 in "[a-z]{3,10}\\.rs",
        line in 1u32..1000,
        message in "[A-Za-z ]{5,50}",
    ) {
        prop_assume!(path1 != path2);

        let loc1 = Location {
            path: NormPath::new(&path1),
            line: Some(line),
            col: None,
        };
        let loc2 = Location {
            path: NormPath::new(&path2),
            line: Some(line),
            col: None,
        };

        let fp1 = fingerprint(&code, Some(&loc1), &message);
        let fp2 = fingerprint(&code, Some(&loc2), &message);
        prop_assert_ne!(fp1, fp2, "Different paths must produce different fingerprints");
    }

    /// Different lines should produce different fingerprints.
    #[test]
    fn different_lines_different_fingerprints(
        code in "[A-Z]{3,10}",
        path in "[a-z]{3,10}\\.rs",
        line1 in 1u32..500,
        line2 in 1u32..500,
        message in "[A-Za-z ]{5,50}",
    ) {
        prop_assume!(line1 != line2);

        let loc1 = Location {
            path: NormPath::new(&path),
            line: Some(line1),
            col: None,
        };
        let loc2 = Location {
            path: NormPath::new(&path),
            line: Some(line2),
            col: None,
        };

        let fp1 = fingerprint(&code, Some(&loc1), &message);
        let fp2 = fingerprint(&code, Some(&loc2), &message);
        prop_assert_ne!(fp1, fp2, "Different lines must produce different fingerprints");
    }

    /// Different messages should produce different fingerprints.
    #[test]
    fn different_messages_different_fingerprints(
        code in "[A-Z]{3,10}",
        path in "[a-z]{3,10}\\.rs",
        line in 1u32..1000,
        msg1 in "[A-Za-z ]{5,50}",
        msg2 in "[A-Za-z ]{5,50}",
    ) {
        prop_assume!(msg1 != msg2);

        let loc = Location {
            path: NormPath::new(&path),
            line: Some(line),
            col: None,
        };

        let fp1 = fingerprint(&code, Some(&loc), &msg1);
        let fp2 = fingerprint(&code, Some(&loc), &msg2);
        prop_assert_ne!(fp1, fp2, "Different messages must produce different fingerprints");
    }
}

// =============================================================================
// Output Format Tests
// =============================================================================

proptest! {
    /// Fingerprint must always be a 64-character hex string.
    #[test]
    fn fingerprint_is_64_char_hex(
        code in ".*{0,100}",
        path in ".*{0,100}",
        line in prop::option::of(1u32..10000),
        message in ".*{0,200}",
    ) {
        let loc = line.map(|l| Location {
            path: NormPath::new(&path),
            line: Some(l),
            col: None,
        });

        let fp = fingerprint(&code, loc.as_ref(), &message);

        prop_assert_eq!(fp.len(), 64, "Fingerprint must be exactly 64 characters");
        prop_assert!(
            fp.chars().all(|c| c.is_ascii_hexdigit()),
            "Fingerprint must be hexadecimal: got '{}'",
            fp
        );
        // Verify lowercase: hex digits should not have uppercase A-F
        prop_assert!(
            !fp.chars().any(|c| matches!(c, 'A'..='F')),
            "Fingerprint must be lowercase hex: got '{}'",
            fp
        );
    }

    /// Empty inputs still produce valid 64-char hex.
    #[test]
    fn empty_inputs_valid_hex(
        code in "",
        _path in "",
        message in "",
    ) {
        let fp = fingerprint(&code, None, &message);
        prop_assert_eq!(fp.len(), 64);
        prop_assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// Unicode inputs still produce valid 64-char hex.
    #[test]
    fn unicode_inputs_valid_hex(
        code in "[\\p{L}\\p{N}]{0,20}",
        message in "[\\p{L}\\p{N}\\s]{0,50}",
    ) {
        let fp = fingerprint(&code, None, &message);
        prop_assert_eq!(fp.len(), 64);
        prop_assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

// =============================================================================
// Whitespace Normalization Tests
// =============================================================================

proptest! {
    /// Whitespace variations should produce the same fingerprint.
    #[test]
    fn whitespace_normalization_equivalent(
        code in "[A-Z]{3,10}",
        path in "[a-z]{3,10}\\.rs",
        line in 1u32..1000,
        words in prop::collection::vec("[a-z]{2,8}", 2..6),
        leading in "[ \\t\\n\\r]{0,3}",
        trailing in "[ \\t\\n\\r]{0,3}",
    ) {
        let loc = Location {
            path: NormPath::new(&path),
            line: Some(line),
            col: None,
        };

        // Normalized message: single spaces between words
        let normalized = words.join(" ");

        // Noisy message: various whitespace between words (one per gap)
        let num_gaps = words.len() - 1;
        let mut noisy = String::new();
        noisy.push_str(&leading);
        for (i, word) in words.iter().enumerate() {
            noisy.push_str(word);
            if i < num_gaps {
                // Use a deterministic but varying whitespace pattern
                let ws_char = match i % 4 {
                    0 => " ",
                    1 => "\t",
                    2 => "\n",
                    _ => "  ", // double space
                };
                noisy.push_str(ws_char);
            }
        }
        noisy.push_str(&trailing);

        let fp_normalized = fingerprint(&code, Some(&loc), &normalized);
        let fp_noisy = fingerprint(&code, Some(&loc), &noisy);

        prop_assert_eq!(
            fp_normalized, fp_noisy,
            "Whitespace variations should produce same fingerprint: normalized='{}', noisy='{}'",
            normalized, noisy
        );
    }

    /// Leading/trailing whitespace is ignored.
    #[test]
    fn leading_trailing_whitespace_ignored(
        code in "[A-Z]{3,10}",
        message in "[a-z ]{5,30}",
        padding in "[ \\t\\n\\r]{1,10}",
    ) {
        let fp_clean = fingerprint(&code, None, &message);
        let fp_padded = fingerprint(&code, None, &format!("{}{}{}", padding, message, padding));

        prop_assert_eq!(fp_clean, fp_padded);
    }

    /// Multiple internal spaces collapse to single space.
    #[test]
    fn multiple_spaces_collapse(
        code in "[A-Z]{3,10}",
        word1 in "[a-z]{3,10}",
        word2 in "[a-z]{3,10}",
        spaces in "[ ]{2,10}",
    ) {
        let fp_single = fingerprint(&code, None, &format!("{} {}", word1, word2));
        let fp_multiple = fingerprint(&code, None, &format!("{}{}{}", word1, spaces, word2));

        prop_assert_eq!(fp_single, fp_multiple);
    }

    /// Tabs and newlines are treated like spaces.
    #[test]
    fn tabs_newlines_treated_as_spaces(
        code in "[A-Z]{3,10}",
        word1 in "[a-z]{3,10}",
        word2 in "[a-z]{3,10}",
        whitespace in "[ \\t\\n\\r]{1,5}",
    ) {
        let fp_space = fingerprint(&code, None, &format!("{} {}", word1, word2));
        let fp_other = fingerprint(&code, None, &format!("{}{}{}", word1, whitespace, word2));

        prop_assert_eq!(fp_space, fp_other);
    }
}

// =============================================================================
// Edge Case Tests
// =============================================================================

proptest! {
    /// Very long messages should still work.
    #[test]
    fn long_messages_work(
        code in "[A-Z]{3,10}",
        message in "[a-z ]{1000,5000}",
    ) {
        let fp = fingerprint(&code, None, &message);
        prop_assert_eq!(fp.len(), 64);
    }

    /// Very long paths should still work.
    #[test]
    fn long_paths_work(
        code in "[A-Z]{3,10}",
        path_segments in prop::collection::vec("[a-z]{3,10}", 20..50),
        message in "[a-z ]{5,30}",
    ) {
        let path = path_segments.join("/");
        let loc = Location {
            path: NormPath::new(&path),
            line: Some(1),
            col: None,
        };

        let fp = fingerprint(&code, Some(&loc), &message);
        prop_assert_eq!(fp.len(), 64);
    }

    /// Special characters in code are handled.
    #[test]
    fn special_chars_in_code(
        code in "[A-Za-z0-9._:\\-\\[\\]]{1,30}",
        message in "[a-z ]{5,30}",
    ) {
        let fp = fingerprint(&code, None, &message);
        prop_assert_eq!(fp.len(), 64);
    }

    /// Location without line still produces valid fingerprint.
    #[test]
    fn location_without_line(
        code in "[A-Z]{3,10}",
        path in "[a-z]{3,10}\\.rs",
        message in "[a-z ]{5,30}",
    ) {
        let loc = Location {
            path: NormPath::new(&path),
            line: None,
            col: None,
        };

        let fp = fingerprint(&code, Some(&loc), &message);
        prop_assert_eq!(fp.len(), 64);
    }
}
