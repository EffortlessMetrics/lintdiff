//! Unified diff parsing into new-side changed line ranges.

use std::collections::{BTreeMap, BTreeSet};

use lintdiff_types::{normalize_path, LineRange, NormPath};
use thiserror::Error;

#[derive(Clone, Debug, Default)]
pub struct DiffStats {
    pub files: u32,
    pub hunks: u32,
    pub added_lines: u32,
}

#[derive(Clone, Debug, Default)]
pub struct DiffMap {
    /// New-path -> merged changed line ranges (new-side).
    pub changed: BTreeMap<NormPath, Vec<LineRange>>,
    /// Old-path -> new-path (best effort).
    pub renames: BTreeMap<NormPath, NormPath>,
    pub stats: DiffStats,
}

#[derive(Debug, Error)]
pub enum DiffParseError {
    #[error("unified diff parse error at line {line}: {msg}")]
    Invalid { line: usize, msg: String },
}

#[derive(Debug, Default)]
struct FileState {
    old_path: Option<NormPath>,
    new_path: Option<NormPath>,
    rename_from: Option<NormPath>,
    rename_to: Option<NormPath>,
    in_hunk: bool,
    old_line: u32,
    new_line: u32,
    changed_lines: BTreeSet<u32>,
    hunks: u32,
    added_lines: u32,
}

/// Parse a unified diff into a `DiffMap`.
///
/// This parser is intentionally forgiving about metadata; it cares about:
/// - file identity (new path preferred)
/// - hunk boundaries
/// - new-side line numbers for `+` lines
pub fn parse_unified_diff(input: &str) -> Result<DiffMap, DiffParseError> {
    let mut out = DiffMap::default();

    let mut current: Option<FileState> = None;

    for (idx, raw_line) in input.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw_line;

        if line.starts_with("diff --git ") {
            flush_file_state(&mut out, current.take());
            current = Some(FileState::default());
            // best-effort path capture from the diff header:
            // diff --git a/foo b/foo
            if let Some(st) = current.as_mut() {
                if let Some((a, b)) = parse_diff_git_paths(line) {
                    st.old_path = Some(NormPath::new(a));
                    st.new_path = Some(NormPath::new(b));
                }
            }
            continue;
        }

        let Some(st) = current.as_mut() else {
            // Ignore leading junk until first file header.
            continue;
        };

        if line.starts_with("rename from ") {
            st.rename_from = Some(NormPath::new(
                line.trim_start_matches("rename from ").trim(),
            ));
            continue;
        }
        if line.starts_with("rename to ") {
            st.rename_to = Some(NormPath::new(line.trim_start_matches("rename to ").trim()));
            continue;
        }

        if line.starts_with("--- ") {
            let p = line.trim_start_matches("--- ").trim();
            if p == "/dev/null" {
                st.old_path = None;
            } else {
                st.old_path = Some(NormPath::new(extract_diff_path(p)));
            }
            continue;
        }

        if line.starts_with("+++ ") {
            let p = line.trim_start_matches("+++ ").trim();
            if p == "/dev/null" {
                st.new_path = None;
            } else {
                st.new_path = Some(NormPath::new(extract_diff_path(p)));
            }
            continue;
        }

        if line.starts_with("@@ ") {
            let (old_start, new_start) = parse_hunk_header(line)
                .map_err(|msg| DiffParseError::Invalid { line: line_no, msg })?;
            st.in_hunk = true;
            st.hunks += 1;
            st.old_line = old_start;
            st.new_line = new_start;
            continue;
        }

        if st.in_hunk {
            if line.starts_with('+') && !line.starts_with("+++ ") {
                // new-side changed line
                if st.new_line >= 1 {
                    st.changed_lines.insert(st.new_line);
                }
                st.new_line = st.new_line.saturating_add(1);
                st.added_lines += 1;
                continue;
            }
            if line.starts_with('-') && !line.starts_with("--- ") {
                st.old_line = st.old_line.saturating_add(1);
                continue;
            }
            if line.starts_with(' ') {
                st.old_line = st.old_line.saturating_add(1);
                st.new_line = st.new_line.saturating_add(1);
                continue;
            }
            if line.starts_with('\\') {
                // "\ No newline at end of file" – ignore
                continue;
            }

            // If we encounter metadata, we assume we've left the hunk.
            st.in_hunk = false;
        }

        // ignore other metadata lines
    }

    flush_file_state(&mut out, current.take());
    Ok(out)
}

fn flush_file_state(out: &mut DiffMap, st: Option<FileState>) {
    let Some(st) = st else {
        return;
    };

    let old_path = st
        .rename_from
        .clone()
        .or_else(|| st.old_path.clone())
        .map(|p| normalize_path(p.as_str()));
    let new_path = st
        .rename_to
        .clone()
        .or_else(|| st.new_path.clone())
        .map(|p| normalize_path(p.as_str()));

    if let (Some(old), Some(new)) = (old_path.clone(), new_path.clone()) {
        if old != new {
            out.renames.insert(old, new);
        }
    }

    // only record changed ranges when we have a new path and at least one changed line
    if let Some(new) = new_path {
        if !st.changed_lines.is_empty() {
            let ranges = merge_lines_to_ranges(st.changed_lines.into_iter().collect());
            out.changed.entry(new).or_default().extend(ranges);
        }
    }

    out.stats.files += 1;
    out.stats.hunks += st.hunks;
    out.stats.added_lines += st.added_lines;
}

fn parse_diff_git_paths(line: &str) -> Option<(String, String)> {
    // diff --git a/foo b/foo
    let mut parts = line.split_whitespace();
    let _diff = parts.next()?;
    let _git = parts.next()?;
    let a = parts.next()?;
    let b = parts.next()?;
    Some((
        extract_diff_path(a).to_string(),
        extract_diff_path(b).to_string(),
    ))
}

fn extract_diff_path(p: &str) -> &str {
    // strip a/ or b/ prefixes but do not normalize further here
    p.strip_prefix("a/")
        .or_else(|| p.strip_prefix("b/"))
        .unwrap_or(p)
}

fn parse_hunk_header(line: &str) -> Result<(u32, u32), String> {
    // @@ -old_start,old_len +new_start,new_len @@
    // old_len/new_len may be omitted.
    let line = line.trim();
    if !line.starts_with("@@") {
        return Err("not a hunk header".to_string());
    }
    // Find the '-' and '+' segments.
    let minus_pos = line.find('-').ok_or("missing '-' segment")?;
    let plus_pos = line.find('+').ok_or("missing '+' segment")?;
    let after_minus = &line[minus_pos + 1..];
    let minus_seg = after_minus
        .split_whitespace()
        .next()
        .ok_or("invalid '-' segment")?;
    let after_plus = &line[plus_pos + 1..];
    let plus_seg = after_plus
        .split_whitespace()
        .next()
        .ok_or("invalid '+' segment")?;

    let old_start = minus_seg
        .split(',')
        .next()
        .ok_or("invalid old range")?
        .parse::<u32>()
        .map_err(|_| "invalid old_start".to_string())?;
    let new_start = plus_seg
        .split(',')
        .next()
        .ok_or("invalid new range")?
        .parse::<u32>()
        .map_err(|_| "invalid new_start".to_string())?;

    Ok((old_start.max(1), new_start.max(1)))
}

fn merge_lines_to_ranges(mut lines: Vec<u32>) -> Vec<LineRange> {
    lines.sort_unstable();
    lines.dedup();

    let mut out: Vec<LineRange> = Vec::new();
    let mut start: Option<u32> = None;
    let mut prev: u32 = 0;

    for line in lines {
        if start.is_none() {
            start = Some(line);
            prev = line;
            continue;
        }

        if line == prev + 1 {
            prev = line;
            continue;
        }

        // close previous range
        let s = start.take().unwrap();
        out.push(LineRange::new(s, prev));
        start = Some(line);
        prev = line;
    }

    if let Some(s) = start {
        out.push(LineRange::new(s, prev));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_added_lines() {
        let diff = r#"
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,0 +1,3 @@
+fn a() {}
+fn b() {}
+fn c() {}
"#;

        let map = parse_unified_diff(diff).unwrap();
        let ranges = map.changed.get(&NormPath::new("src/lib.rs")).unwrap();
        assert_eq!(ranges, &vec![LineRange::new(1, 3)]);
        assert_eq!(map.stats.hunks, 1);
        assert_eq!(map.stats.added_lines, 3);
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn merge_lines_to_ranges_is_idempotent(lines in proptest::collection::vec(1u32..5000, 0..200)) {
            let ranges1 = merge_lines_to_ranges(lines.clone());

            // Expand ranges back to a line vector and merge again.
            let mut expanded: Vec<u32> = Vec::new();
            for r in &ranges1 {
                for l in r.start..=r.end {
                    expanded.push(l);
                }
            }
            let ranges2 = merge_lines_to_ranges(expanded);
            prop_assert_eq!(ranges1, ranges2);
        }

        #[test]
        fn merged_ranges_are_strictly_increasing(lines in proptest::collection::vec(1u32..5000, 0..200)) {
            let ranges = merge_lines_to_ranges(lines);
            for w in ranges.windows(2) {
                prop_assert!(w[0].end < w[1].start);
            }
        }
    }
}
