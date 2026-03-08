//! Stable receipt fingerprint generation.
//!
//! Fingerprints intentionally normalize message whitespace so noisy renderer
//! differences do not alter identity for the same diagnostic at the same code/location.

use lintdiff_types::Location;
use sha2::{Digest, Sha256};

/// Create a deterministic digest for a diagnostic finding.
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
