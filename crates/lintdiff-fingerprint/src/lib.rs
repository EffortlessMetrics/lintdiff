//! Stable receipt fingerprint generation.
//!
//! This crate provides deterministic fingerprint generation for diagnostic findings.
//! Fingerprints are used to uniquely identify lint warnings/errors across different
//! runs of the linting tool, enabling tracking of issues across code changes.
//!
//! # Fingerprint Composition
//!
//! A fingerprint is computed from:
//! - **Code**: The diagnostic code (e.g., "clippy::needless_borrow", "E001")
//! - **Location**: Optional file path and line number
//! - **Message**: The diagnostic message text
//!
//! # Normalization
//!
//! Message text is normalized to ensure stable fingerprints:
//! - Leading and trailing whitespace is trimmed
//! - All internal whitespace (spaces, tabs, newlines) is collapsed to single spaces
//! - This prevents noisy formatting differences from changing the fingerprint
//!
//! # Output Format
//!
//! Fingerprints are returned as 64-character lowercase hexadecimal strings
//! (SHA-256 hash encoded as hex).
//!
//! # Example
//!
//! ```
//! use lintdiff_fingerprint::fingerprint;
//! use lintdiff_types::{Location, NormPath};
//!
//! // Simple fingerprint without location
//! let fp = fingerprint("E001", None, "Variable unused");
//! assert_eq!(fp.len(), 64);
//!
//! // Fingerprint with location
//! let loc = Location {
//!     path: NormPath::new("src/main.rs"),
//!     line: Some(42),
//!     col: Some(10),
//! };
//! let fp = fingerprint("clippy::unwrap_used", Some(&loc), "used unwrap");
//! assert_eq!(fp.len(), 64);
//!
//! // Whitespace normalization - these produce the same fingerprint
//! let fp1 = fingerprint("CODE", None, "  hello   world  ");
//! let fp2 = fingerprint("CODE", None, "hello world");
//! assert_eq!(fp1, fp2);
//! ```
//!
//! # Stability Guarantee
//!
//! Fingerprints are designed to be stable across versions. The same inputs
//! will always produce the same fingerprint. This enables:
//!
//! - Tracking issues across CI runs
//! - Comparing lint results between branches
//! - Suppressing known issues with stable identifiers
//!
//! # Thread Safety
//!
//! The [`fingerprint`] function is thread-safe and can be called concurrently
//! from multiple threads.

use lintdiff_types::Location;
use sha2::{Digest, Sha256};

/// Create a deterministic digest for a diagnostic finding.
///
/// Generates a SHA-256 hash from the diagnostic code, optional location,
/// and message. The message is normalized to ensure whitespace variations
/// don't affect the fingerprint.
///
/// # Arguments
///
/// * `code` - The diagnostic code (e.g., "clippy::needless_borrow", "E001")
/// * `loc` - Optional location containing file path and line number
/// * `msg` - The diagnostic message text (will be whitespace-normalized)
///
/// # Returns
///
/// A 64-character lowercase hexadecimal string representing the SHA-256 hash.
///
/// # Examples
///
/// ```
/// use lintdiff_fingerprint::fingerprint;
///
/// // Without location
/// let fp = fingerprint("WARN001", None, "General warning");
/// assert_eq!(fp.len(), 64);
/// assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
/// ```
///
/// ```
/// use lintdiff_fingerprint::fingerprint;
/// use lintdiff_types::{Location, NormPath};
///
/// // With location
/// let loc = Location {
///     path: NormPath::new("src/lib.rs"),
///     line: Some(10),
///     col: None,
/// };
/// let fp = fingerprint("CODE", Some(&loc), "message");
/// assert_eq!(fp.len(), 64);
/// ```
///
/// # Whitespace Normalization
///
/// ```
/// use lintdiff_fingerprint::fingerprint;
/// use lintdiff_types::{Location, NormPath};
///
/// let loc = Location {
///     path: NormPath::new("test.rs"),
///     line: Some(1),
///     col: None,
/// };
///
/// // All these produce the same fingerprint
/// let fp1 = fingerprint("CODE", Some(&loc), "one two three");
/// let fp2 = fingerprint("CODE", Some(&loc), "  one   two   three  ");
/// let fp3 = fingerprint("CODE", Some(&loc), "one\ttwo\nthree");
/// assert_eq!(fp1, fp2);
/// assert_eq!(fp2, fp3);
/// ```
///
/// # Determinism
///
/// ```
/// use lintdiff_fingerprint::fingerprint;
///
/// // Same inputs always produce same output
/// let a = fingerprint("CODE", None, "message");
/// let b = fingerprint("CODE", None, "message");
/// assert_eq!(a, b);
/// ```
pub fn fingerprint(code: &str, loc: Option<&Location>, msg: &str) -> String {
    let mut h = Sha256::new();
    h.update(code.as_bytes());
    h.update(b"|");
    if let Some(loc) = loc {
        h.update(loc.path.as_str().as_bytes());
        h.update(b":");
        if let Some(line) = loc.line {
            h.update(line.to_string().as_bytes());
        }
        h.update(b":");
    }
    h.update(normalize_message(msg).as_bytes());
    hex::encode(h.finalize())
}

fn normalize_message(msg: &str) -> String {
    let mut out = String::new();
    let mut prev_ws = false;
    for ch in msg.trim().chars() {
        let ws = ch.is_whitespace();
        if ws {
            if !prev_ws {
                out.push(' ');
            }
            prev_ws = true;
        } else {
            out.push(ch);
            prev_ws = false;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::fingerprint;
    use lintdiff_types::{Location, NormPath};
    use proptest::prelude::*;

    #[test]
    fn whitespace_normalization_is_stable() {
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
        assert_eq!(a, b);
    }

    #[test]
    fn reference_vector_without_location_is_stable() {
        let actual = fingerprint("lintdiff.diagnostic.unknown", None, "message");
        assert_eq!(
            actual,
            "34415bcd691d11774caf32d55122d0540df005ff0c100a9eb9c7c3af3131d725"
        );
    }

    proptest! {
        #[test]
        fn deterministic_for_same_inputs(
            code in "[A-Za-z0-9._:-]{1,40}",
            path in "[A-Za-z0-9_./-]{1,40}",
            line in prop::option::of(1u32..2000),
            msg in "[ -~\\t\\n\\r]{0,80}",
        ) {
            let loc = Location {
                path: NormPath::new(path),
                line,
                col: None,
            };
            let a = fingerprint(&code, Some(&loc), &msg);
            let b = fingerprint(&code, Some(&loc), &msg);
            prop_assert_eq!(a, b);
        }

        #[test]
        fn equivalent_whitespace_has_same_fingerprint(
            code in "[A-Za-z0-9._:-]{1,40}",
            path in "[A-Za-z0-9_./-]{1,40}",
            line in prop::option::of(1u32..2000),
            segments in prop::collection::vec("[A-Za-z0-9_]{1,8}", 1..8),
            whitespace in prop::collection::vec("[ \\t\\n\\r]{1,4}", 1..8),
            leading in "[ \\t\\n\\r]{0,3}",
            trailing in "[ \\t\\n\\r]{0,3}",
        ) {
            let normalized = segments.join(" ");
            let mut noisy = String::new();
            noisy.push_str(&leading);
            noisy.push_str(&segments[0]);
            for (idx, seg) in segments.iter().enumerate().skip(1) {
                noisy.push_str(&whitespace[idx % whitespace.len()]);
                noisy.push_str(seg);
            }
            noisy.push_str(&trailing);

            let loc = Location {
                path: NormPath::new(path),
                line,
                col: None,
            };
            let clean = fingerprint(&code, Some(&loc), &normalized);
            let with_noise = fingerprint(&code, Some(&loc), &noisy);
            prop_assert_eq!(clean, with_noise);
        }
    }
}
