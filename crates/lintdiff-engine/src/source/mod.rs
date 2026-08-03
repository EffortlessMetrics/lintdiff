//! Unified diff parsing into changed ranges and old/new source correspondence.
//!
//! This crate provides a parser for unified diff format output, extracting
//! the changed line ranges on the "new" side of the diff. It's designed to be
//! forgiving about metadata and focuses on:
//! - File identity (new path preferred)
//! - Hunk boundaries
//! - New-side line numbers for added (`+`) lines
//!
//! # Example
//!
//! ```
//! use lintdiff_engine::parse_unified_diff;
//!
//! let diff = r#"
//! diff --git a/src/lib.rs b/src/lib.rs
//! --- a/src/lib.rs
//! +++ b/src/lib.rs
//! @@ -1,0 +1,3 @@
//! +fn a() {}
//! +fn b() {}
//! +fn c() {}
//! "#;
//!
//! let map = parse_unified_diff(diff).unwrap();
//! assert_eq!(map.stats.files, 1);
//! assert_eq!(map.stats.hunks, 1);
//! assert_eq!(map.stats.added_lines, 3);
//! ```
//!
//! # Data Structures
//!
//! - [`DiffMap`]: The main output containing changed lines per file and rename tracking
//! - [`DiffStats`]: Statistics about the parsed diff (files, hunks, added lines)
//! - [`DiffParseError`]: Error type for parsing failures

use std::collections::{BTreeMap, BTreeSet};

use lintdiff_types::{LineRange, NormPath};
use thiserror::Error;

/// Statistics about a parsed diff.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiffStats {
    /// Number of files in the diff.
    pub files: u32,
    /// Total number of hunks across all files.
    pub hunks: u32,
    /// Total number of added lines across all files.
    pub added_lines: u32,
}

/// A map of file paths to their changed line ranges.
///
/// This is the main output of [`parse_unified_diff`]. It contains:
/// - Changed line ranges for each file (new-side line numbers)
/// - Rename mappings (old path -> new path)
/// - Statistics about the diff
#[derive(Clone, Debug, Default)]
pub struct DiffMap {
    /// New-path -> merged changed line ranges (new-side).
    pub changed: BTreeMap<NormPath, Vec<LineRange>>,
    /// Old-path -> new-path (best effort).
    pub renames: BTreeMap<NormPath, NormPath>,
    /// Statistics about the parsed diff.
    pub stats: DiffStats,
}

/// A parsed source change set with enough evidence to map unchanged lines.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceChangeSet {
    /// File deltas keyed by their new path, or old path for deleted files.
    pub files: BTreeMap<NormPath, FileDelta>,
    /// Statistics retained for compatibility with [`DiffMap`].
    pub stats: DiffStats,
}

/// The old and new identity plus line evidence for one changed file.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileDelta {
    pub old_path: Option<NormPath>,
    pub new_path: Option<NormPath>,
    pub hunks: Vec<HunkDelta>,
    pub added: Vec<LineRange>,
    pub deleted: Vec<LineRange>,
    pub unchanged_segments: Vec<LineMapSegment>,
    pub tail_mapping: Option<LineOffset>,
}

/// Complete old/new coordinates from a unified-diff hunk header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HunkDelta {
    pub old_start: u32,
    pub old_len: u32,
    pub new_start: u32,
    pub new_len: u32,
}

/// A finite unchanged line interval and its old-to-new offset.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineMapSegment {
    pub old: LineRange,
    pub new: LineRange,
    pub offset: LineOffset,
}

/// An open-ended line offset after the final hunk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineOffset {
    pub value: i64,
}

/// The result of asking how an old location relates to the parsed change set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LocationMapping {
    Exact {
        new_path: NormPath,
        new_line: u32,
    },
    Shifted {
        new_path: NormPath,
        new_line: u32,
    },
    Renamed {
        old_path: NormPath,
        new_path: NormPath,
        new_line: u32,
    },
    ShiftedAndRenamed {
        old_path: NormPath,
        new_path: NormPath,
        new_line: u32,
    },
    UnmappableChangedRegion {
        path: NormPath,
        line: u32,
    },
    FileCreated {
        path: NormPath,
    },
    FileDeleted {
        path: NormPath,
    },
    MissingFile,
}

/// Error type for diff parsing failures.
#[derive(Debug, Error)]
pub enum DiffParseError {
    /// The diff content was invalid at a specific line.
    #[error("unified diff parse error at line {line}: {msg}")]
    Invalid { line: usize, msg: String },
}

#[derive(Debug, Default)]
struct FileState {
    old_path: Option<NormPath>,
    new_path: Option<NormPath>,
    has_diff_transport_prefixes: bool,
    rename_from: Option<NormPath>,
    rename_to: Option<NormPath>,
    in_hunk: bool,
    old_line: u32,
    new_line: u32,
    changed_lines: BTreeSet<u32>,
    deleted_lines: BTreeSet<u32>,
    hunk_deltas: Vec<HunkDelta>,
    hunks: u32,
    added_lines: u32,
}

impl SourceChangeSet {
    /// Map one old repository location using only earned diff correspondence.
    pub fn map_old_location(&self, old_path: &NormPath, old_line: u32) -> LocationMapping {
        let Some(file) = self
            .files
            .values()
            .find(|file| file.old_path.as_ref() == Some(old_path))
            .or_else(|| {
                self.files.values().find(|file| {
                    file.old_path.is_none() && file.new_path.as_ref() == Some(old_path)
                })
            })
        else {
            return LocationMapping::MissingFile;
        };

        let Some(new_path) = file.new_path.as_ref() else {
            return LocationMapping::FileDeleted {
                path: old_path.clone(),
            };
        };
        if file.old_path.is_none() {
            return LocationMapping::FileCreated {
                path: new_path.clone(),
            };
        }
        if file.hunks.is_empty() {
            return if file.old_path.as_ref() == Some(new_path) {
                LocationMapping::Exact {
                    new_path: new_path.clone(),
                    new_line: old_line,
                }
            } else {
                LocationMapping::Renamed {
                    old_path: old_path.clone(),
                    new_path: new_path.clone(),
                    new_line: old_line,
                }
            };
        }

        for hunk in &file.hunks {
            let old_end = hunk.old_start.saturating_add(hunk.old_len);
            if hunk.old_len > 0 && old_line >= hunk.old_start && old_line < old_end {
                return LocationMapping::UnmappableChangedRegion {
                    path: old_path.clone(),
                    line: old_line,
                };
            }
        }

        let offset = file
            .unchanged_segments
            .iter()
            .find(|segment| old_line >= segment.old.start && old_line <= segment.old.end)
            .map(|segment| segment.offset)
            .or_else(|| {
                let first = file.hunks.first()?;
                if old_line < first.old_start {
                    Some(LineOffset {
                        value: i64::from(first.new_start) - i64::from(first.old_start),
                    })
                } else {
                    file.tail_mapping
                }
            });

        let Some(offset) = offset else {
            return LocationMapping::UnmappableChangedRegion {
                path: old_path.clone(),
                line: old_line,
            };
        };
        let Some(new_line) = apply_offset(old_line, offset) else {
            return LocationMapping::UnmappableChangedRegion {
                path: old_path.clone(),
                line: old_line,
            };
        };
        let renamed = file.old_path.as_ref() != Some(new_path);
        match (offset.value == 0, renamed) {
            (true, false) => LocationMapping::Exact {
                new_path: new_path.clone(),
                new_line,
            },
            (false, false) => LocationMapping::Shifted {
                new_path: new_path.clone(),
                new_line,
            },
            (true, true) => LocationMapping::Renamed {
                old_path: old_path.clone(),
                new_path: new_path.clone(),
                new_line,
            },
            (false, true) => LocationMapping::ShiftedAndRenamed {
                old_path: old_path.clone(),
                new_path: new_path.clone(),
                new_line,
            },
        }
    }
}

fn apply_offset(line: u32, offset: LineOffset) -> Option<u32> {
    let mapped = i64::from(line).checked_add(offset.value)?;
    u32::try_from(mapped).ok().filter(|line| *line > 0)
}

/// Parse a unified diff into the compatibility `DiffMap`.
///
/// This parser is intentionally forgiving about metadata; it cares about:
/// - file identity (new path preferred)
/// - hunk boundaries
/// - new-side line numbers for `+` lines
pub fn parse_unified_diff(input: &str) -> Result<DiffMap, DiffParseError> {
    let source = parse_source_change_set(input)?;
    let mut out = DiffMap {
        stats: source.stats.clone(),
        ..DiffMap::default()
    };
    for file in source.files.values() {
        if let (Some(old), Some(new)) = (&file.old_path, &file.new_path) {
            if old != new {
                out.renames.insert(old.clone(), new.clone());
            }
        }
        if let Some(new) = &file.new_path {
            if !file.added.is_empty() {
                out.changed.insert(new.clone(), file.added.clone());
            }
        }
    }
    Ok(out)
}

/// Parse a unified diff into the richer source-correspondence model.
pub fn parse_source_change_set(input: &str) -> Result<SourceChangeSet, DiffParseError> {
    let mut out = DiffMap::default();
    let mut source = SourceChangeSet::default();

    let mut current: Option<FileState> = None;

    for (idx, raw_line) in input.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw_line;

        if line.starts_with("diff --git ") {
            flush_file_state(&mut out, &mut source, current.take());
            current = Some(FileState::default());
            // best-effort path capture from the diff header:
            // diff --git a/foo b/foo
            if let Some(st) = current.as_mut() {
                if let Some((a, b, has_transport_prefixes)) = parse_diff_git_paths(line) {
                    st.has_diff_transport_prefixes = has_transport_prefixes;
                    st.old_path = Some(repo_path_from_diff_path(&a, has_transport_prefixes));
                    st.new_path = Some(repo_path_from_diff_path(&b, has_transport_prefixes));
                }
            }
            continue;
        }

        if current.is_none() && line.starts_with("--- ") {
            let mut state = FileState::default();
            let path = line.trim_start_matches("--- ").trim();
            if path != "/dev/null" {
                state.has_diff_transport_prefixes = path.starts_with("a/");
                state.old_path = Some(repo_path_from_diff_path(
                    path,
                    state.has_diff_transport_prefixes,
                ));
            }
            current = Some(state);
        }

        let Some(st) = current.as_mut() else {
            // Ignore leading junk until first file header.
            continue;
        };

        if line.starts_with("rename from ") {
            st.rename_from = Some(repo_path_from_repository_path(
                line.trim_start_matches("rename from ").trim(),
            ));
            continue;
        }
        if line.starts_with("rename to ") {
            st.rename_to = Some(repo_path_from_repository_path(
                line.trim_start_matches("rename to ").trim(),
            ));
            continue;
        }

        if line.starts_with("--- ") {
            let p = line.trim_start_matches("--- ").trim();
            if p == "/dev/null" {
                st.old_path = None;
            } else {
                st.old_path = Some(repo_path_from_diff_path(p, st.has_diff_transport_prefixes));
            }
            continue;
        }

        if line.starts_with("+++ ") {
            let p = line.trim_start_matches("+++ ").trim();
            if p == "/dev/null" {
                st.new_path = None;
            } else {
                if st.old_path.is_none() && p.starts_with("b/") {
                    st.has_diff_transport_prefixes = true;
                }
                st.new_path = Some(repo_path_from_diff_path(p, st.has_diff_transport_prefixes));
            }
            continue;
        }

        if line.starts_with("@@ ") {
            let (old_start, old_len, new_start, new_len) = parse_hunk_header(line)
                .map_err(|msg| DiffParseError::Invalid { line: line_no, msg })?;
            st.in_hunk = true;
            st.hunks += 1;
            st.old_line = old_start;
            st.new_line = new_start;
            st.hunk_deltas.push(HunkDelta {
                old_start,
                old_len,
                new_start,
                new_len,
            });
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
                if st.old_line >= 1 {
                    st.deleted_lines.insert(st.old_line);
                }
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

    flush_file_state(&mut out, &mut source, current.take());
    source.stats = out.stats;
    Ok(source)
}

fn flush_file_state(out: &mut DiffMap, source: &mut SourceChangeSet, st: Option<FileState>) {
    let Some(st) = st else {
        return;
    };

    let old_path = st.rename_from.clone().or_else(|| st.old_path.clone());
    let new_path = st.rename_to.clone().or_else(|| st.new_path.clone());

    if let (Some(old), Some(new)) = (old_path.clone(), new_path.clone()) {
        if old != new {
            out.renames.insert(old, new);
        }
    }

    let added = merge_lines_to_ranges(st.changed_lines.into_iter().collect());
    let deleted = merge_lines_to_ranges(st.deleted_lines.into_iter().collect());
    if let Some(new) = &new_path {
        if !added.is_empty() {
            out.changed.insert(new.clone(), added.clone());
        }
    }

    let file = FileDelta {
        old_path: old_path.clone(),
        new_path: new_path.clone(),
        hunks: st.hunk_deltas.clone(),
        added,
        deleted,
        unchanged_segments: unchanged_segments(&st.hunk_deltas),
        tail_mapping: tail_mapping(&st.hunk_deltas),
    };
    if let Some(key) = new_path.or(old_path) {
        source.files.insert(key, file);
    }

    out.stats.files += 1;
    out.stats.hunks += st.hunks;
    out.stats.added_lines += st.added_lines;
}

fn unchanged_segments(hunks: &[HunkDelta]) -> Vec<LineMapSegment> {
    if hunks.is_empty() {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut previous_old_end = 1_u32;
    let mut previous_new_end = 1_u32;
    for hunk in hunks {
        let old_end = hunk.old_start.saturating_add(hunk.old_len);
        let new_end = hunk.new_start.saturating_add(hunk.new_len);
        if hunk.old_start < previous_old_end || hunk.new_start < previous_new_end {
            return Vec::new();
        }
        if hunk.old_start > previous_old_end {
            let offset = LineOffset {
                value: i64::from(previous_new_end) - i64::from(previous_old_end),
            };
            let old = LineRange::new(previous_old_end, hunk.old_start - 1);
            let Some(new_start) = apply_offset(old.start, offset) else {
                return Vec::new();
            };
            let Some(new_end_line) = apply_offset(old.end, offset) else {
                return Vec::new();
            };
            segments.push(LineMapSegment {
                old,
                new: LineRange::new(new_start, new_end_line),
                offset,
            });
        }
        previous_old_end = old_end;
        previous_new_end = new_end;
    }
    segments
}

fn tail_mapping(hunks: &[HunkDelta]) -> Option<LineOffset> {
    let last = hunks.last()?;
    for pair in hunks.windows(2) {
        let previous = &pair[0];
        let current = &pair[1];
        if current.old_start < previous.old_start.saturating_add(previous.old_len)
            || current.new_start < previous.new_start.saturating_add(previous.new_len)
        {
            return None;
        }
    }
    Some(LineOffset {
        value: i64::from(last.new_start.saturating_add(last.new_len))
            - i64::from(last.old_start.saturating_add(last.old_len)),
    })
}

fn parse_diff_git_paths(line: &str) -> Option<(String, String, bool)> {
    let rest = line.strip_prefix("diff --git ")?;
    let (a, rest) = take_git_path_token(rest)?;
    let (b, _) = take_git_path_token(rest)?;
    // The header parser retains compatibility with historical unquoted paths
    // containing whitespace. In that form the token split is not reliable,
    // but the second transport path is still identifiable by its `b/` marker.
    let has_transport_prefixes = (a.starts_with("a/") && b.starts_with("b/"))
        || rest.contains(" b/")
        || rest.contains("\"b/");
    Some((a, b, has_transport_prefixes))
}

fn repo_path_from_diff_path(raw: &str, has_transport_prefix: bool) -> NormPath {
    let decoded = decode_git_path(raw);
    let path = if has_transport_prefix {
        decoded
            .strip_prefix("a/")
            .or_else(|| decoded.strip_prefix("b/"))
            .unwrap_or(&decoded)
    } else {
        &decoded
    };
    NormPath::from_repo_path(path)
}

fn repo_path_from_repository_path(raw: &str) -> NormPath {
    NormPath::from_repo_path(decode_git_path(raw))
}

fn take_git_path_token(input: &str) -> Option<(String, &str)> {
    let input = input.trim_start();
    if input.starts_with('"') {
        let bytes = input.as_bytes();
        let mut escaped = false;
        for (index, byte) in bytes.iter().enumerate().skip(1) {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                return Some((decode_git_path(&input[..=index]), &input[index + 1..]));
            }
        }
        None
    } else {
        let end = input.find(char::is_whitespace).unwrap_or(input.len());
        Some((input[..end].to_string(), &input[end..]))
    }
}

fn decode_git_path(raw: &str) -> String {
    let raw = raw.trim();
    if !(raw.starts_with('"') && raw.ends_with('"') && raw.len() >= 2) {
        return raw.to_string();
    }

    let bytes = raw.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len().saturating_sub(2));
    let mut index = 1;
    while index + 1 < bytes.len() {
        if bytes[index] != b'\\' {
            decoded.push(bytes[index]);
            index += 1;
            continue;
        }

        index += 1;
        let Some(&escaped) = bytes.get(index) else {
            break;
        };
        let decoded_byte = match escaped {
            b'a' => 0x07,
            b'b' => 0x08,
            b't' => b'\t',
            b'n' => b'\n',
            b'v' => 0x0b,
            b'f' => 0x0c,
            b'r' => b'\r',
            b'\\' => b'\\',
            b'"' => b'"',
            b'0'..=b'7' => {
                let mut value: u16 = u16::from(escaped - b'0');
                let mut digits = 1;
                while digits < 3 {
                    let Some(&next) = bytes.get(index + 1) else {
                        break;
                    };
                    if !(b'0'..=b'7').contains(&next) {
                        break;
                    }
                    value = value * 8 + u16::from(next - b'0');
                    index += 1;
                    digits += 1;
                }
                u8::try_from(value).unwrap_or(b'?')
            }
            other => other,
        };
        decoded.push(decoded_byte);
        index += 1;
    }

    String::from_utf8_lossy(&decoded).into_owned()
}

fn parse_hunk_header(line: &str) -> Result<(u32, u32, u32, u32), String> {
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

    let (old_start, old_len) = parse_hunk_range(minus_seg, "old")?;
    let (new_start, new_len) = parse_hunk_range(plus_seg, "new")?;

    Ok((old_start.max(1), old_len, new_start.max(1), new_len))
}

fn parse_hunk_range(segment: &str, side: &str) -> Result<(u32, u32), String> {
    let mut parts = segment.split(',');
    let start = parts
        .next()
        .ok_or_else(|| format!("invalid {side} range"))?
        .parse::<u32>()
        .map_err(|_| format!("invalid {side}_start"))?;
    let len = parts
        .next()
        .map(|value| {
            value
                .parse::<u32>()
                .map_err(|_| format!("invalid {side}_len"))
        })
        .transpose()?
        .unwrap_or(1);
    Ok((start, len))
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

    #[test]
    fn strips_only_the_diff_transport_prefix_for_a_directory() {
        let diff = r#"
diff --git a/a/src/lib.rs b/a/src/lib.rs
--- a/a/src/lib.rs
+++ b/a/src/lib.rs
@@ -1,0 +1,1 @@
+fn a() {}
"#;

        let map = parse_unified_diff(diff).unwrap();
        assert!(map
            .changed
            .contains_key(&NormPath::from_repo_path("a/src/lib.rs")));
    }

    #[test]
    fn decodes_quoted_paths_with_spaces() {
        let diff = r#"
diff --git "a/src/my file.rs" "b/src/my file.rs"
--- "a/src/my file.rs"
+++ "b/src/my file.rs"
@@ -1,0 +1,1 @@
+content
"#;

        let map = parse_unified_diff(diff).unwrap();
        assert!(map
            .changed
            .contains_key(&NormPath::from_repo_path("src/my file.rs")));
    }

    #[test]
    fn preserves_repository_path_in_prefixless_diff_input() {
        let diff = r#"
diff --git a/src/lib.rs a/src/lib.rs
--- a/src/lib.rs
+++ a/src/lib.rs
@@ -1,0 +1,1 @@
+content
"#;

        let map = parse_unified_diff(diff).unwrap();
        assert!(map
            .changed
            .contains_key(&NormPath::from_repo_path("a/src/lib.rs")));
    }

    #[test]
    fn accepts_patch_fragments_without_diff_git_metadata() {
        let diff = r#"--- a/src/lib.rs
+++ b/src/lib.rs
@@ -2 +2 @@
-old
+new
"#;
        let source = parse_source_change_set(diff).expect("valid patch fragment");
        assert_eq!(source.files.len(), 1);
        assert!(matches!(
            source.map_old_location(&NormPath::from_repo_path("src/lib.rs"), 2),
            LocationMapping::UnmappableChangedRegion { line: 2, .. }
        ));
    }

    #[test]
    fn decodes_escaped_and_out_of_range_octal_path_bytes_deterministically() {
        assert_eq!(
            decode_git_path(r#""a/space\040name.rs""#),
            "a/space name.rs"
        );
        assert_eq!(
            decode_git_path(r#""a/invalid\400name.rs""#),
            "a/invalid?name.rs"
        );
    }

    #[test]
    fn preserves_repository_a_directory_in_rename_records() {
        let diff = r#"
diff --git a/a/old.rs b/a/new.rs
similarity index 100%
rename from a/old.rs
rename to a/new.rs
"#;

        let map = parse_unified_diff(diff).unwrap();
        assert_eq!(
            map.renames.get(&NormPath::from_repo_path("a/old.rs")),
            Some(&NormPath::from_repo_path("a/new.rs"))
        );
    }

    fn map(diff: &str, path: &str, line: u32) -> LocationMapping {
        parse_source_change_set(diff)
            .expect("valid source diff")
            .map_old_location(&NormPath::from_repo_path(path), line)
    }

    #[test]
    fn pure_insertion_shifts_following_lines_without_mapping_the_insertion() {
        let diff = r#"diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -3,0 +3,2 @@
+added one
+added two
"#;
        assert_eq!(
            map(diff, "src/lib.rs", 2),
            LocationMapping::Exact {
                new_path: NormPath::from_repo_path("src/lib.rs"),
                new_line: 2,
            }
        );
        assert_eq!(
            map(diff, "src/lib.rs", 3),
            LocationMapping::Shifted {
                new_path: NormPath::from_repo_path("src/lib.rs"),
                new_line: 5,
            }
        );
    }

    #[test]
    fn deletion_and_replacement_leave_changed_old_regions_unmappable() {
        let deletion = r#"diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -3,2 +3,0 @@
-deleted one
-deleted two
"#;
        assert!(matches!(
            map(deletion, "src/lib.rs", 3),
            LocationMapping::UnmappableChangedRegion { line: 3, .. }
        ));
        assert_eq!(
            map(deletion, "src/lib.rs", 5),
            LocationMapping::Shifted {
                new_path: NormPath::from_repo_path("src/lib.rs"),
                new_line: 3,
            }
        );

        let replacement = r#"diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -3,2 +3,1 @@
-old one
-old two
+new one
"#;
        assert!(matches!(
            map(replacement, "src/lib.rs", 4),
            LocationMapping::UnmappableChangedRegion { line: 4, .. }
        ));
        assert_eq!(
            map(replacement, "src/lib.rs", 5),
            LocationMapping::Shifted {
                new_path: NormPath::from_repo_path("src/lib.rs"),
                new_line: 4,
            }
        );
    }

    #[test]
    fn rename_only_and_rename_with_edit_preserve_movement_evidence() {
        let rename_only = r#"diff --git a/src/old.rs b/src/new.rs
similarity index 100%
rename from src/old.rs
rename to src/new.rs
"#;
        assert_eq!(
            map(rename_only, "src/old.rs", 7),
            LocationMapping::Renamed {
                old_path: NormPath::from_repo_path("src/old.rs"),
                new_path: NormPath::from_repo_path("src/new.rs"),
                new_line: 7,
            }
        );

        let rename_with_edit = r#"diff --git a/src/old.rs b/src/new.rs
--- a/src/old.rs
+++ b/src/new.rs
@@ -2,0 +2,1 @@
+inserted
"#;
        assert_eq!(
            map(rename_with_edit, "src/old.rs", 2),
            LocationMapping::ShiftedAndRenamed {
                old_path: NormPath::from_repo_path("src/old.rs"),
                new_path: NormPath::from_repo_path("src/new.rs"),
                new_line: 3,
            }
        );
    }

    #[test]
    fn created_deleted_and_missing_files_are_explicit() {
        let created = r#"diff --git a/new.rs b/new.rs
new file mode 100644
--- /dev/null
+++ b/new.rs
@@ -0,0 +1,1 @@
+new
"#;
        assert_eq!(
            map(created, "new.rs", 1),
            LocationMapping::FileCreated {
                path: NormPath::from_repo_path("new.rs")
            }
        );

        let deleted = r#"diff --git a/old.rs b/old.rs
deleted file mode 100644
--- a/old.rs
+++ /dev/null
@@ -1,1 +0,0 @@
-old
"#;
        assert_eq!(
            map(deleted, "old.rs", 1),
            LocationMapping::FileDeleted {
                path: NormPath::from_repo_path("old.rs")
            }
        );
        assert_eq!(map(deleted, "missing.rs", 1), LocationMapping::MissingFile);
    }

    #[test]
    fn overlapping_hunks_do_not_bridge_unknown_regions() {
        let diff = r#"diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,1 +1,1 @@
-old
+new
@@ -1,1 +1,1 @@
-old again
+new again
"#;
        assert!(matches!(
            map(diff, "src/lib.rs", 3),
            LocationMapping::UnmappableChangedRegion { .. }
        ));
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
