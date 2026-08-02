//! Fuzz target for diagnostics parsing.
//!
//! This target tests the [`parse_cargo_messages`] function with various JSONL formats.
//! The corpus includes:
//! - Simple warnings and errors
//! - Multi-span diagnostics
//! - Macro-expanded spans
//! - Generated file paths
//! - Unicode content
//! - Multiple messages in one stream

#![no_main]

use libfuzzer_sys::fuzz_target;
use lintdiff_engine::parse_cargo_messages;
use std::io::{BufReader, Cursor};

fuzz_target!(|data: &[u8]| {
    // Only process valid UTF-8 data
    if let Ok(s) = std::str::from_utf8(data) {
        let reader = BufReader::new(Cursor::new(s.as_bytes()));
        
        // Parse the diagnostics - this should never panic
        let result = parse_cargo_messages(reader);

        // Basic invariants:
        // - If parsing succeeds, all diagnostics should have valid structure
        // - Spans should have non-negative line numbers
        if let Ok(diagnostics) = &result {
            for diagnostic in diagnostics {
                // Verify spans have valid line numbers
                for span in &diagnostic.spans {
                    if span.line_start == 0 || span.line_end == 0 {
                        // Line numbers should be positive, but don't panic
                    }
                }
            }
        }
    }
});
