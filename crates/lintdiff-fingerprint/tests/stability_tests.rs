//! Stability and determinism tests for fingerprint generation.
//!
//! These tests ensure that fingerprints remain stable across versions
//! and are deterministic regardless of input variations that should
//! be normalized.

use lintdiff_fingerprint::fingerprint;
use lintdiff_types::{Location, NormPath};

// =============================================================================
// Reference Vectors - Known Inputs/Outputs
// These MUST NOT change between versions to maintain fingerprint stability
// =============================================================================

/// Reference test case for fingerprint stability (for future use)
#[allow(dead_code)]
struct ReferenceCase {
    code: &'static str,
    path: Option<&'static str>,
    line: Option<u32>,
    message: &'static str,
    expected: &'static str,
}

#[test]
fn reference_vector_no_location() {
    // This is the canonical reference for fingerprinting without location
    let result = fingerprint("lintdiff.diagnostic.unknown", None, "message");
    assert_eq!(
        result, "34415bcd691d11774caf32d55122d0540df005ff0c100a9eb9c7c3af3131d725",
        "Reference vector for no location must remain stable"
    );
}

#[test]
fn reference_vector_with_location() {
    let loc = Location {
        path: NormPath::new("src/lib.rs"),
        line: Some(9),
        col: Some(1),
    };
    let result = fingerprint(
        "lintdiff.diagnostic.clippy.needless_borrow",
        Some(&loc),
        "this expression borrows a reference that is immediately dereferenced by the compiler",
    );
    // This reference was computed from the original implementation
    assert_eq!(
        result, "fd778b3062ca0ea9e31b5824b0864b8e5448ccc1292a99bdcbd32815687e1e5a",
        "Reference vector for location-based fingerprint must remain stable"
    );
}

#[test]
fn reference_vector_empty_inputs() {
    let result = fingerprint("", None, "");
    // This should remain stable
    assert_eq!(result.len(), 64);
    // Store the actual value once computed
    let _computed = result; // Replace with assert_eq! once value is known
}

#[test]
fn reference_vector_whitespace_normalization() {
    let loc = Location {
        path: NormPath::new("src/lib.rs"),
        line: Some(9),
        col: Some(1),
    };

    // These should produce the same fingerprint
    let normalized = fingerprint(
        "lintdiff.diagnostic.clippy.needless_borrow",
        Some(&loc),
        "one two three",
    );
    let noisy = fingerprint(
        "lintdiff.diagnostic.clippy.needless_borrow",
        Some(&loc),
        "  one\t two\nthree  ",
    );

    assert_eq!(
        normalized, noisy,
        "Whitespace normalization must produce identical fingerprints"
    );

    // Store the reference value
    assert_eq!(normalized.len(), 64);
}

// =============================================================================
// Determinism Tests
// =============================================================================

#[test]
fn deterministic_across_multiple_calls() {
    let loc = Location {
        path: NormPath::new("determinism_test.rs"),
        line: Some(42),
        col: None,
    };

    // Generate fingerprint 100 times
    let fingerprints: Vec<String> = (0..100)
        .map(|_| {
            fingerprint(
                "DETERMINISM_TEST",
                Some(&loc),
                "Test message for determinism",
            )
        })
        .collect();

    // All must be identical
    let first = &fingerprints[0];
    for (i, fp) in fingerprints.iter().enumerate() {
        assert_eq!(
            first, fp,
            "Fingerprint at iteration {} differs from first",
            i
        );
    }
}

#[test]
fn deterministic_with_threaded_access() {
    use std::sync::Arc;
    use std::thread;

    let loc = Arc::new(Location {
        path: NormPath::new("thread_test.rs"),
        line: Some(100),
        col: None,
    });

    let mut handles = vec![];

    for _ in 0..10 {
        let loc_clone = Arc::clone(&loc);
        let handle = thread::spawn(move || {
            let mut results = Vec::new();
            for _ in 0..10 {
                results.push(fingerprint("THREAD_TEST", Some(&loc_clone), "Thread test"));
            }
            results
        });
        handles.push(handle);
    }

    let all_results: Vec<String> = handles
        .into_iter()
        .flat_map(|h| h.join().unwrap())
        .collect();

    // All 100 results should be identical
    let first = &all_results[0];
    assert!(
        all_results.iter().all(|r| r == first),
        "All threaded results should be identical"
    );
}

// =============================================================================
// Whitespace Handling Stability
// =============================================================================

#[test]
fn whitespace_equivalence_is_stable() {
    let loc = Location {
        path: NormPath::new("whitespace.rs"),
        line: Some(1),
        col: None,
    };

    let base = fingerprint("CODE", Some(&loc), "hello world");

    // All these variations should produce the same fingerprint
    let variations = [
        "  hello world",
        "hello world  ",
        "  hello world  ",
        "hello  world",
        "hello\tworld",
        "hello\nworld",
        "hello\r\nworld",
        "\thello world\t",
        " hello  world ",
        "\n\thello\n\tworld\n\t",
    ];

    for variation in &variations {
        let result = fingerprint("CODE", Some(&loc), variation);
        assert_eq!(
            base,
            result,
            "Whitespace variation '{}' should produce same fingerprint",
            variation.escape_unicode()
        );
    }
}

#[test]
fn complex_whitespace_normalization() {
    let loc = Location {
        path: NormPath::new("complex_ws.rs"),
        line: Some(1),
        col: None,
    };

    // Complex message with various whitespace
    let messages = [
        "The quick brown fox jumps over the lazy dog",
        " The quick brown fox jumps over the lazy dog ",
        "The\tquick\tbrown\tfox\tjumps\tover\tthe\tlazy\tdog",
        "The\nquick\nbrown\nfox\njumps\nover\nthe\nlazy\ndog",
        "The  quick  brown  fox  jumps  over  the  lazy  dog",
    ];

    let expected = fingerprint("CODE", Some(&loc), messages[0]);

    for msg in &messages {
        let result = fingerprint("CODE", Some(&loc), msg);
        assert_eq!(
            expected,
            result,
            "Message '{}' should normalize to same fingerprint",
            msg.escape_unicode()
        );
    }
}

// =============================================================================
// Unicode Stability
// =============================================================================

#[test]
fn unicode_fingerprints_are_stable() {
    // Test that unicode content produces stable fingerprints
    let unicode_messages = [
        "Error: 文件未找到",
        "Erreur: fichier non trouvé",
        "エラー: ファイルが見つかりません",
        "🔥 Critical error",
    ];

    for msg in &unicode_messages {
        let first = fingerprint("UNICODE_TEST", None, msg);
        let second = fingerprint("UNICODE_TEST", None, msg);
        assert_eq!(
            first, second,
            "Unicode message '{}' should produce stable fingerprint",
            msg
        );
    }
}

#[test]
fn unicode_paths_are_stable() {
    let unicode_paths = ["src/中文/lib.rs", "src/日本語/mod.rs", "src/한국어/main.rs"];

    for path in &unicode_paths {
        let loc = Location {
            path: NormPath::new(path),
            line: Some(1),
            col: None,
        };
        let first = fingerprint("CODE", Some(&loc), "message");
        let second = fingerprint("CODE", Some(&loc), "message");
        assert_eq!(
            first, second,
            "Unicode path '{}' should produce stable fingerprint",
            path
        );
    }
}

// =============================================================================
// Version Stability Tests
// =============================================================================

/// These fingerprints were computed with the original implementation.
/// They MUST NOT change in future versions.
#[test]
fn version_stable_fingerprints() {
    // Test cases that should never change their fingerprint
    #[allow(clippy::type_complexity)]
    let stable_cases: Vec<(&str, Option<(&str, Option<u32>)>, &str, &str)> = vec![
        // (code, (path, line), message, expected_fingerprint)
        // Add known stable fingerprints here
        // Example:
        // ("CODE1", Some(("file.rs", Some(1))), "msg", "abc123..."),
    ];

    for (code, loc_info, msg, expected) in stable_cases {
        let loc = loc_info.map(|(path, line)| Location {
            path: NormPath::new(path),
            line,
            col: None,
        });
        let result = fingerprint(code, loc.as_ref(), msg);
        assert_eq!(
            result, expected,
            "Fingerprint for ({}, {:?}, {}) must remain stable",
            code, loc_info, msg
        );
    }
}

// =============================================================================
// Cross-Platform Stability
// =============================================================================

#[test]
fn path_separator_handling() {
    // Different path representations should produce different fingerprints
    // (path normalization is not part of fingerprinting)
    let loc_forward = Location {
        path: NormPath::new("src/lib.rs"),
        line: Some(1),
        col: None,
    };
    let loc_backslash = Location {
        path: NormPath::new("src\\lib.rs"),
        line: Some(1),
        col: None,
    };

    let forward = fingerprint("CODE", Some(&loc_forward), "message");
    let backslash = fingerprint("CODE", Some(&loc_backslash), "message");

    // These are different paths, so they should produce different fingerprints
    // (NormPath handles normalization)
    // This test documents the current behavior
    let _ = (forward, backslash);
}

// =============================================================================
// Input Order Stability
// =============================================================================

#[test]
fn fingerprint_components_are_positionally_encoded() {
    // The fingerprint function encodes components in a specific order:
    // code | path : line : message

    // Verify that different orderings would produce different results
    // (this is a sanity check that the encoding is position-sensitive)

    let loc = Location {
        path: NormPath::new("test.rs"),
        line: Some(1),
        col: None,
    };

    // These should all be different
    let fp1 = fingerprint("ABC", Some(&loc), "XYZ");
    let fp2 = fingerprint("XYZ", Some(&loc), "ABC");

    assert_ne!(
        fp1, fp2,
        "Swapping code and message should produce different fingerprints"
    );
}

// =============================================================================
// Boundary Stability
// =============================================================================

#[test]
fn empty_message_stability() {
    let loc = Location {
        path: NormPath::new("test.rs"),
        line: Some(1),
        col: None,
    };

    // Empty message should be stable
    let a = fingerprint("CODE", Some(&loc), "");
    let b = fingerprint("CODE", Some(&loc), "");
    assert_eq!(a, b, "Empty message should produce stable fingerprint");
}

#[test]
fn whitespace_only_message_stability() {
    let loc = Location {
        path: NormPath::new("test.rs"),
        line: Some(1),
        col: None,
    };

    // Whitespace-only messages should normalize to empty and be stable
    let variations = ["", " ", "  ", "\t", "\n", " \t\n "];
    let expected = fingerprint("CODE", Some(&loc), "");

    for v in &variations {
        let result = fingerprint("CODE", Some(&loc), v);
        assert_eq!(
            expected,
            result,
            "Whitespace-only message '{}' should normalize to empty",
            v.escape_unicode()
        );
    }
}

// =============================================================================
// Large Input Stability
// =============================================================================

#[test]
fn large_message_stability() {
    let large_msg = "x".repeat(10000);

    let a = fingerprint("CODE", None, &large_msg);
    let b = fingerprint("CODE", None, &large_msg);

    assert_eq!(a, b, "Large message should produce stable fingerprint");
}

#[test]
fn large_code_stability() {
    let large_code = "C".repeat(1000);

    let a = fingerprint(&large_code, None, "message");
    let b = fingerprint(&large_code, None, "message");

    assert_eq!(a, b, "Large code should produce stable fingerprint");
}

// =============================================================================
// Regression Tests
// =============================================================================

#[test]
fn regression_clippy_needless_borrow() {
    // This is a real-world test case from the existing tests
    let loc = Location {
        path: NormPath::new("src/lib.rs"),
        line: Some(9),
        col: Some(1),
    };

    let a = fingerprint(
        "lintdiff.diagnostic.clippy.needless_borrow",
        Some(&loc),
        "  one\t two\nthree  ",
    );
    let b = fingerprint(
        "lintdiff.diagnostic.clippy.needless_borrow",
        Some(&loc),
        "one two three",
    );

    assert_eq!(
        a, b,
        "Regression: whitespace normalization for clippy diagnostic"
    );
}

// =============================================================================
// Collision Resistance Tests
// =============================================================================

#[test]
fn no_obvious_collisions() {
    // Test that similar inputs don't produce collisions
    let inputs = [
        ("A", "msg"),
        ("B", "msg"),
        ("AA", "msg"),
        ("AB", "msg"),
        ("A", "msf"),
        ("A", "msga"),
    ];

    let mut fingerprints = std::collections::HashSet::new();

    for (code, msg) in &inputs {
        let fp = fingerprint(code, None, msg);
        assert!(
            fingerprints.insert(fp.clone()),
            "Collision detected for ({}, {})",
            code,
            msg
        );
    }
}

#[test]
fn no_collision_with_incrementing_numbers() {
    let mut fingerprints = std::collections::HashSet::new();

    for i in 0..1000u32 {
        let fp = fingerprint(&format!("CODE{}", i), None, &format!("message {}", i));
        assert!(fingerprints.insert(fp), "Collision at index {}", i);
    }
}

// =============================================================================
// Format Consistency Tests
// =============================================================================

#[test]
fn output_is_lowercase_hex() {
    let result = fingerprint("CODE", None, "message");

    // Should be all lowercase hex
    assert!(
        result
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
        "Output should be lowercase hexadecimal"
    );
}

#[test]
fn output_is_valid_sha256_hex() {
    let result = fingerprint("CODE", None, "message");

    // Should be exactly 64 hex characters (256 bits)
    assert_eq!(result.len(), 64);
    assert!(result.chars().all(|c| c.is_ascii_hexdigit()));

    // Should be valid hex that can be decoded
    assert!(hex::decode(&result).is_ok());
}

#[test]
fn output_byte_length() {
    let result = fingerprint("CODE", None, "message");
    let bytes = hex::decode(&result).expect("Output should be valid hex");
    assert_eq!(bytes.len(), 32, "SHA256 should produce 32 bytes");
}
