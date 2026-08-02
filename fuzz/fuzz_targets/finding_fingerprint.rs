//! Fuzz target for fingerprint generation.
//!
//! This target tests the [`fingerprint`] function with various input combinations.
//! The corpus uses null-byte-separated fields:
//! - Field 0: lint code (e.g., "clippy::let_unit_value")
//! - Field 1: file path (optional)
//! - Field 2: line number (optional, must be > 0)
//! - Field 3: message text
//!
//! The corpus includes:
//! - Simple fingerprints with all fields
//! - Fingerprints without location
//! - Unicode messages
//! - Long codes and paths
//! - Empty parts

#![no_main]

use libfuzzer_sys::fuzz_target;
use lintdiff_engine::fingerprint;
use lintdiff_types::{Location, NormPath};

fuzz_target!(|data: &[u8]| {
    let code = read_part(data, 0);
    let message = read_part(data, 1);
    let path = read_part(data, 2);
    let line_raw = read_part(data, 3);

    // Parse line number - must be positive
    let line = line_raw
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|n| *n > 0);

    // Build optional location
    let location = if path.is_empty() {
        None
    } else {
        Some(Location {
            path: NormPath::new(path),
            line,
            col: None,
        })
    };

    // Generate fingerprint - this should never panic
    let result = fingerprint(code, location.as_ref(), message);

    // Basic invariants:
    // - Fingerprint should be a valid string
    // - Same inputs should produce same fingerprint (deterministic)
    #[cfg(debug_assertions)]
    {
        let result2 = fingerprint(code, location.as_ref(), message);
        assert_eq!(result, result2, "Fingerprint should be deterministic");
    }
});

/// Extract a null-byte-separated field from the data.
fn read_part(data: &[u8], idx: usize) -> &str {
    let part = data.split(|b| *b == 0).nth(idx).unwrap_or_default();
    std::str::from_utf8(part).unwrap_or("")
}
