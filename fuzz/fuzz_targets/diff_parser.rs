#![no_main]

use libfuzzer_sys::fuzz_target;
use lintdiff_diff::parse_unified_diff;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parse_unified_diff(s);
    }
});
