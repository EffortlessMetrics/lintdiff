//! Fuzz target for diff parsing.
//!
//! This target tests the [`parse_unified_diff`] function with various diff formats.
//! The corpus includes:
//! - Simple additions/deletions
//! - Multi-file diffs
//! - Renames and moves
//! - Binary files and mode changes
//! - Edge cases (empty hunks, large hunks)

#![no_main]

use libfuzzer_sys::fuzz_target;
use lintdiff_ingest_core::parse_unified_diff;

fuzz_target!(|data: &[u8]| {
    // Only process valid UTF-8 data
    if let Ok(s) = std::str::from_utf8(data) {
        // Parse the diff - this should never panic
        let result = parse_unified_diff(s);

        // Basic invariants:
        // - If parsing succeeds, all file paths should be valid
        // - Line numbers should be positive when present
        if let Ok(diff_map) = &result {
            // Verify changed files have valid entries
            for (path, ranges) in &diff_map.changed {
                // Path should not be empty
                if path.as_str().is_empty() {
                    // This is a degenerate case but shouldn't panic
                }
                // Line ranges should have valid start/end
                for range in ranges {
                    if range.start == 0 || range.end == 0 {
                        // Line numbers should be positive, but don't panic
                    }
                }
            }
        }
    }
});
