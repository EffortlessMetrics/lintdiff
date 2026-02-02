#![no_main]

use libfuzzer_sys::fuzz_target;
use lintdiff_diagnostics::parse_cargo_messages;
use std::io::{BufReader, Cursor};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let reader = BufReader::new(Cursor::new(s.as_bytes()));
        let _ = parse_cargo_messages(reader);
    }
});
