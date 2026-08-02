//! Comprehensive tests for fingerprint generation.
//!
//! These tests cover basic fingerprint creation, input variations,
//! and various edge cases.

use lintdiff_ingest_core::fingerprint;
use lintdiff_types::{Location, NormPath};

// =============================================================================
// Basic Fingerprint Creation Tests
// =============================================================================

#[test]
fn fingerprint_returns_64_character_hex_string() {
    let result = fingerprint("code", None, "message");
    assert_eq!(
        result.len(),
        64,
        "SHA256 hex output should be 64 characters"
    );
    assert!(
        result.chars().all(|c| c.is_ascii_hexdigit()),
        "Output should be hexadecimal"
    );
}

#[test]
fn fingerprint_with_all_components() {
    let loc = Location {
        path: NormPath::new("src/main.rs"),
        line: Some(42),
        col: Some(10),
    };
    let result = fingerprint("E001", Some(&loc), "Variable unused");
    assert_eq!(result.len(), 64);
}

#[test]
fn fingerprint_with_no_location() {
    let result = fingerprint("WARN001", None, "General warning");
    assert_eq!(result.len(), 64);
}

#[test]
fn fingerprint_with_location_no_line() {
    let loc = Location {
        path: NormPath::new("config.toml"),
        line: None,
        col: None,
    };
    let result = fingerprint("CFG001", Some(&loc), "Config issue");
    assert_eq!(result.len(), 64);
}

// =============================================================================
// Different Inputs Produce Different Fingerprints
// =============================================================================

#[test]
fn different_codes_produce_different_fingerprints() {
    let loc = Location {
        path: NormPath::new("src/lib.rs"),
        line: Some(10),
        col: None,
    };
    let a = fingerprint("CODE_A", Some(&loc), "message");
    let b = fingerprint("CODE_B", Some(&loc), "message");
    assert_ne!(
        a, b,
        "Different codes should produce different fingerprints"
    );
}

#[test]
fn different_paths_produce_different_fingerprints() {
    let loc_a = Location {
        path: NormPath::new("src/a.rs"),
        line: Some(10),
        col: None,
    };
    let loc_b = Location {
        path: NormPath::new("src/b.rs"),
        line: Some(10),
        col: None,
    };
    let a = fingerprint("CODE", Some(&loc_a), "message");
    let b = fingerprint("CODE", Some(&loc_b), "message");
    assert_ne!(
        a, b,
        "Different paths should produce different fingerprints"
    );
}

#[test]
fn different_lines_produce_different_fingerprints() {
    let loc_a = Location {
        path: NormPath::new("src/lib.rs"),
        line: Some(10),
        col: None,
    };
    let loc_b = Location {
        path: NormPath::new("src/lib.rs"),
        line: Some(20),
        col: None,
    };
    let a = fingerprint("CODE", Some(&loc_a), "message");
    let b = fingerprint("CODE", Some(&loc_b), "message");
    assert_ne!(
        a, b,
        "Different lines should produce different fingerprints"
    );
}

#[test]
fn different_messages_produce_different_fingerprints() {
    let loc = Location {
        path: NormPath::new("src/lib.rs"),
        line: Some(10),
        col: None,
    };
    let a = fingerprint("CODE", Some(&loc), "message one");
    let b = fingerprint("CODE", Some(&loc), "message two");
    assert_ne!(
        a, b,
        "Different messages should produce different fingerprints"
    );
}

#[test]
fn location_vs_no_location_different() {
    let loc = Location {
        path: NormPath::new("src/lib.rs"),
        line: Some(10),
        col: None,
    };
    let with_loc = fingerprint("CODE", Some(&loc), "message");
    let without_loc = fingerprint("CODE", None, "message");
    assert_ne!(
        with_loc, without_loc,
        "With location and without location should differ"
    );
}

#[test]
fn line_vs_no_line_different() {
    let loc_with_line = Location {
        path: NormPath::new("src/lib.rs"),
        line: Some(10),
        col: None,
    };
    let loc_no_line = Location {
        path: NormPath::new("src/lib.rs"),
        line: None,
        col: None,
    };
    let with_line = fingerprint("CODE", Some(&loc_with_line), "message");
    let no_line = fingerprint("CODE", Some(&loc_no_line), "message");
    assert_ne!(
        with_line, no_line,
        "With line and without line should differ"
    );
}

// =============================================================================
// Same Inputs Produce Same Fingerprint
// =============================================================================

#[test]
fn same_inputs_produce_same_fingerprint() {
    let loc = Location {
        path: NormPath::new("src/lib.rs"),
        line: Some(10),
        col: Some(5),
    };
    let a = fingerprint("CODE", Some(&loc), "message");
    let b = fingerprint("CODE", Some(&loc), "message");
    assert_eq!(a, b, "Same inputs should produce same fingerprint");
}

#[test]
fn fingerprint_is_deterministic_multiple_calls() {
    let loc = Location {
        path: NormPath::new("src/main.rs"),
        line: Some(100),
        col: None,
    };

    // Call multiple times to ensure deterministic output
    let results: Vec<String> = (0..10)
        .map(|_| fingerprint("DETERMINISTIC", Some(&loc), "test message"))
        .collect();

    let first = &results[0];
    assert!(
        results.iter().all(|r| r == first),
        "All calls should produce the same result"
    );
}

#[test]
fn fingerprint_with_no_location_is_deterministic() {
    let results: Vec<String> = (0..10)
        .map(|_| fingerprint("CODE", None, "message"))
        .collect();

    let first = &results[0];
    assert!(
        results.iter().all(|r| r == first),
        "All calls should produce the same result"
    );
}

// =============================================================================
// Edge Cases - Empty Inputs
// =============================================================================

#[test]
fn empty_code_is_valid() {
    let result = fingerprint("", None, "message");
    assert_eq!(
        result.len(),
        64,
        "Empty code should still produce valid fingerprint"
    );
}

#[test]
fn empty_message_is_valid() {
    let result = fingerprint("CODE", None, "");
    assert_eq!(
        result.len(),
        64,
        "Empty message should still produce valid fingerprint"
    );
}

#[test]
fn empty_code_and_message_is_valid() {
    let result = fingerprint("", None, "");
    assert_eq!(
        result.len(),
        64,
        "Empty code and message should still produce valid fingerprint"
    );
}

#[test]
fn empty_path_is_valid() {
    let loc = Location {
        path: NormPath::new(""),
        line: Some(1),
        col: None,
    };
    let result = fingerprint("CODE", Some(&loc), "message");
    assert_eq!(
        result.len(),
        64,
        "Empty path should still produce valid fingerprint"
    );
}

// =============================================================================
// Edge Cases - Very Long Inputs
// =============================================================================

#[test]
fn very_long_code() {
    let long_code = "A".repeat(1000);
    let result = fingerprint(&long_code, None, "message");
    assert_eq!(result.len(), 64);
}

#[test]
fn very_long_message() {
    let long_message = "This is a very long message. ".repeat(100);
    let result = fingerprint("CODE", None, &long_message);
    assert_eq!(result.len(), 64);
}

#[test]
fn very_long_path() {
    let loc = Location {
        path: NormPath::new("very/".repeat(200)),
        line: Some(1),
        col: None,
    };
    let result = fingerprint("CODE", Some(&loc), "message");
    assert_eq!(result.len(), 64);
}

#[test]
fn very_long_code_and_message() {
    let long_code = "X".repeat(500);
    let long_message = "Y".repeat(500);
    let result = fingerprint(&long_code, None, &long_message);
    assert_eq!(result.len(), 64);
}

// =============================================================================
// Edge Cases - Special Characters
// =============================================================================

#[test]
fn special_characters_in_code() {
    let codes = [
        "code-with-dashes",
        "code_with_underscores",
        "code.with.dots",
        "code:with:colons",
        "code/with/slashes",
        "code\\with\\backslashes",
        "code@with#special$chars",
        "code!with?symbols",
    ];

    let results: Vec<String> = codes.iter().map(|&c| fingerprint(c, None, "msg")).collect();

    // All should be unique
    for i in 0..results.len() {
        for j in (i + 1)..results.len() {
            assert_ne!(
                results[i], results[j],
                "Codes '{}' and '{}' should differ",
                codes[i], codes[j]
            );
        }
    }
}

#[test]
fn special_characters_in_message() {
    let messages = vec![
        "Message with 'quotes'",
        "Message with \"double quotes\"",
        "Message with\nnewline",
        "Message with\ttab",
        "Message with\r\nCRLF",
        "Message with \\ escape",
        "Message with {brackets}",
        "Message with [array]",
        "Message with (parens)",
        "Message with <angle>",
    ];

    let loc = Location {
        path: NormPath::new("test.rs"),
        line: Some(1),
        col: None,
    };

    let results: Vec<String> = messages
        .iter()
        .map(|&m| fingerprint("CODE", Some(&loc), m))
        .collect();

    // All should be unique
    for i in 0..results.len() {
        for j in (i + 1)..results.len() {
            assert_ne!(results[i], results[j], "Messages should differ");
        }
    }
}

#[test]
fn special_characters_in_path() {
    let paths = [
        "src/path/with/slashes",
        "src\\path\\with\\backslashes",
        "path with spaces",
        "path.with.dots",
        "path-with-dashes",
        "path_with_underscores",
    ];

    let results: Vec<String> = paths
        .iter()
        .map(|&p| {
            let loc = Location {
                path: NormPath::new(p),
                line: Some(1),
                col: None,
            };
            fingerprint("CODE", Some(&loc), "message")
        })
        .collect();

    // All should be unique
    for i in 0..results.len() {
        for j in (i + 1)..results.len() {
            assert_ne!(results[i], results[j], "Paths should differ");
        }
    }
}

// =============================================================================
// Edge Cases - Unicode Content
// =============================================================================

#[test]
fn unicode_in_code() {
    let result = fingerprint("代码错误", None, "message");
    assert_eq!(result.len(), 64);
}

#[test]
fn unicode_in_message() {
    let messages = vec![
        "Error: 文件未找到",
        "Erreur: fichier non trouvé",
        "Fehler: Datei nicht gefunden",
        "エラー: ファイルが見つかりません",
        "오류: 파일을 찾을 수 없습니다",
        "❌ Error emoji",
        "✓ Checkmark",
        "→ Arrow symbol",
    ];

    for msg in messages {
        let result = fingerprint("CODE", None, msg);
        assert_eq!(
            result.len(),
            64,
            "Unicode message '{}' should produce valid fingerprint",
            msg
        );
    }
}

#[test]
fn unicode_in_path() {
    let paths = vec![
        "src/中文/文件.rs",
        "src/日本語/ファイル.rs",
        "src/한국어/파일.rs",
        "src/ελληνικά/αρχείο.rs",
        "src/русский/файл.rs",
    ];

    for path in paths {
        let loc = Location {
            path: NormPath::new(path),
            line: Some(1),
            col: None,
        };
        let result = fingerprint("CODE", Some(&loc), "message");
        assert_eq!(
            result.len(),
            64,
            "Unicode path '{}' should produce valid fingerprint",
            path
        );
    }
}

#[test]
fn emoji_in_message() {
    let messages = [
        "🔥 Critical error",
        "⚠️ Warning",
        "💡 Suggestion",
        "🐛 Bug detected",
        "🚀 Performance issue",
    ];

    let results: Vec<String> = messages
        .iter()
        .map(|&m| fingerprint("CODE", None, m))
        .collect();

    // All should be unique
    for i in 0..results.len() {
        for j in (i + 1)..results.len() {
            assert_ne!(results[i], results[j], "Emoji messages should differ");
        }
    }
}

// =============================================================================
// Code Format Tests
// =============================================================================

#[test]
fn realistic_lint_codes() {
    let codes = vec![
        "clippy::needless_borrow",
        "clippy::unwrap_used",
        "rustc::E0433",
        "rustc::unused_variable",
        "eslint/no-unused-vars",
        "eslint/prefer-const",
        "pylint/unused-import",
        "mypy/no-untyped-def",
        "E501",
        "W293",
    ];

    let loc = Location {
        path: NormPath::new("src/lib.rs"),
        line: Some(10),
        col: None,
    };

    let results: Vec<String> = codes
        .iter()
        .map(|&c| fingerprint(c, Some(&loc), "message"))
        .collect();

    // All should be unique
    for i in 0..results.len() {
        for j in (i + 1)..results.len() {
            assert_ne!(results[i], results[j], "Lint codes should differ");
        }
    }
}

// =============================================================================
// Column Number Handling
// =============================================================================

#[test]
fn column_number_does_not_affect_fingerprint() {
    // Column is not included in fingerprint (only path and line)
    let loc_a = Location {
        path: NormPath::new("src/lib.rs"),
        line: Some(10),
        col: Some(1),
    };
    let loc_b = Location {
        path: NormPath::new("src/lib.rs"),
        line: Some(10),
        col: Some(100),
    };
    let a = fingerprint("CODE", Some(&loc_a), "message");
    let b = fingerprint("CODE", Some(&loc_b), "message");
    assert_eq!(
        a, b,
        "Different columns should not affect fingerprint (only path and line are used)"
    );
}

#[test]
fn column_none_vs_some_same_fingerprint() {
    let loc_none = Location {
        path: NormPath::new("src/lib.rs"),
        line: Some(10),
        col: None,
    };
    let loc_some = Location {
        path: NormPath::new("src/lib.rs"),
        line: Some(10),
        col: Some(5),
    };
    let none = fingerprint("CODE", Some(&loc_none), "message");
    let some = fingerprint("CODE", Some(&loc_some), "message");
    assert_eq!(none, some, "Column presence should not affect fingerprint");
}

// =============================================================================
// Boundary Tests
// =============================================================================

#[test]
fn line_number_zero() {
    let loc = Location {
        path: NormPath::new("src/lib.rs"),
        line: Some(0),
        col: None,
    };
    let result = fingerprint("CODE", Some(&loc), "message");
    assert_eq!(result.len(), 64);
}

#[test]
fn line_number_max() {
    let loc = Location {
        path: NormPath::new("src/lib.rs"),
        line: Some(u32::MAX),
        col: None,
    };
    let result = fingerprint("CODE", Some(&loc), "message");
    assert_eq!(result.len(), 64);
}

#[test]
fn line_number_one() {
    let loc = Location {
        path: NormPath::new("src/lib.rs"),
        line: Some(1),
        col: None,
    };
    let result = fingerprint("CODE", Some(&loc), "message");
    assert_eq!(result.len(), 64);
}

// =============================================================================
// Message Normalization Tests
// =============================================================================

#[test]
fn message_leading_trailing_whitespace_normalized() {
    let loc = Location {
        path: NormPath::new("test.rs"),
        line: Some(1),
        col: None,
    };
    let a = fingerprint("CODE", Some(&loc), "  message  ");
    let b = fingerprint("CODE", Some(&loc), "message");
    assert_eq!(a, b, "Leading/trailing whitespace should be normalized");
}

#[test]
fn message_internal_whitespace_normalized() {
    let loc = Location {
        path: NormPath::new("test.rs"),
        line: Some(1),
        col: None,
    };
    let a = fingerprint("CODE", Some(&loc), "one  two   three");
    let b = fingerprint("CODE", Some(&loc), "one two three");
    assert_eq!(a, b, "Internal whitespace should be normalized");
}

#[test]
fn message_tabs_normalized_to_space() {
    let loc = Location {
        path: NormPath::new("test.rs"),
        line: Some(1),
        col: None,
    };
    let a = fingerprint("CODE", Some(&loc), "one\ttwo\tthree");
    let b = fingerprint("CODE", Some(&loc), "one two three");
    assert_eq!(a, b, "Tabs should be normalized to spaces");
}

#[test]
fn message_newlines_normalized_to_space() {
    let loc = Location {
        path: NormPath::new("test.rs"),
        line: Some(1),
        col: None,
    };
    let a = fingerprint("CODE", Some(&loc), "one\ntwo\nthree");
    let b = fingerprint("CODE", Some(&loc), "one two three");
    assert_eq!(a, b, "Newlines should be normalized to spaces");
}

#[test]
fn message_mixed_whitespace_normalized() {
    let loc = Location {
        path: NormPath::new("test.rs"),
        line: Some(1),
        col: None,
    };
    let a = fingerprint("CODE", Some(&loc), " \t one \n two \r three \t ");
    let b = fingerprint("CODE", Some(&loc), "one two three");
    assert_eq!(a, b, "Mixed whitespace should be normalized");
}

#[test]
fn message_only_whitespace_becomes_empty() {
    let loc = Location {
        path: NormPath::new("test.rs"),
        line: Some(1),
        col: None,
    };
    let a = fingerprint("CODE", Some(&loc), "   \t\n\r   ");
    let b = fingerprint("CODE", Some(&loc), "");
    assert_eq!(a, b, "Only whitespace should normalize to empty");
}

// =============================================================================
// Hash Distribution Tests
// =============================================================================

#[test]
fn similar_inputs_have_different_hashes() {
    let loc = Location {
        path: NormPath::new("test.rs"),
        line: Some(1),
        col: None,
    };

    // Test that small changes produce different hashes
    let a = fingerprint("CODE1", Some(&loc), "message");
    let b = fingerprint("CODE2", Some(&loc), "message");
    let c = fingerprint("CODE1", Some(&loc), "messagf"); // one char different

    assert_ne!(a, b);
    assert_ne!(a, c);
    assert_ne!(b, c);
}

#[test]
fn hash_bits_are_well_distributed() {
    // Generate many hashes and check they don't all cluster
    let mut hashes = Vec::new();
    for i in 0..100 {
        let result = fingerprint(&format!("CODE{}", i), None, &format!("message {}", i));
        hashes.push(result);
    }

    // Check uniqueness
    let unique: std::collections::HashSet<_> = hashes.iter().collect();
    assert_eq!(unique.len(), 100, "All hashes should be unique");
}
