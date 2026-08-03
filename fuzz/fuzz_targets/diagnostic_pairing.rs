//! Fuzz target for the canonical diagnostic pairing engine.
//!
//! The input contains two JSON-encoded inventories separated by a NUL byte,
//! followed by an optional unified diff. Valid corpus entries exercise the
//! production comparison path without reproducing any matching logic here.

#![no_main]

use libfuzzer_sys::fuzz_target;
use lintdiff_engine::{compare_inventories, parse_source_change_set};
use lintdiff_types::inventory::Inventory;

fuzz_target!(|data: &[u8]| {
    let mut parts = data.splitn(3, |byte| *byte == 0);
    let Some(base_bytes) = parts.next() else {
        return;
    };
    let Some(head_bytes) = parts.next() else {
        return;
    };
    let diff_bytes = parts.next().unwrap_or_default();
    let Ok(base) = serde_json::from_slice::<Inventory>(base_bytes) else {
        return;
    };
    let Ok(head) = serde_json::from_slice::<Inventory>(head_bytes) else {
        return;
    };
    let Ok(diff) = std::str::from_utf8(diff_bytes) else {
        return;
    };
    let Ok(source) = parse_source_change_set(diff) else {
        return;
    };

    let _comparison = compare_inventories(&base, &head, &source);
});
