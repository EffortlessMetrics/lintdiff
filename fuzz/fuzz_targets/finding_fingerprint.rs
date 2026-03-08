#![no_main]

use libfuzzer_sys::fuzz_target;
use lintdiff_fingerprint::fingerprint;
use lintdiff_types::{Location, NormPath};

fuzz_target!(|data: &[u8]| {
    let code = read_part(data, 0);
    let message = read_part(data, 1);
    let path = read_part(data, 2);
    let line_raw = read_part(data, 3);

    let line = line_raw
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|n| *n > 0);

    let location = if path.is_empty() {
        None
    } else {
        Some(Location {
            path: NormPath::new(path),
            line,
            col: None,
        })
    };

    let _ = fingerprint(code, location.as_ref(), message);
});

fn read_part(data: &[u8], idx: usize) -> &str {
    let part = data.split(|b| *b == 0).nth(idx).unwrap_or_default();
    std::str::from_utf8(part).unwrap_or("")
}
