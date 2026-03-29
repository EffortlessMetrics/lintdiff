//! JSON schema comparison utilities for lintdiff.
//!
//! This microcrate provides utilities for comparing JSON values and schemas,
//! identifying differences, and generating diff reports.
//!
//! # Example: Comparing JSON Values
//!
//! ```
//! use lintdiff_schema_diff::{diff_json, JsonDiff, DiffKind};
//! use serde_json::json;
//!
//! let a = json!({"name": "Alice", "age": 30});
//! let b = json!({"name": "Alice", "age": 31, "city": "NYC"});
//!
//! let diffs = diff_json(&a, &b);
//! assert_eq!(diffs.len(), 2); // age changed, city added
//! ```
//!
//! # Example: Schema Comparison
//!
//! ```
//! use lintdiff_schema_diff::{SchemaDiff, SchemaDiffConfig};
//! use serde_json::json;
//!
//! let old = json!({
//!     "type": "object",
//!     "properties": {
//!         "name": {"type": "string"},
//!         "age": {"type": "integer"}
//!     },
//!     "required": ["name"]
//! });
//!
//! let new = json!({
//!     "type": "object",
//!     "properties": {
//!         "name": {"type": "string"},
//!         "email": {"type": "string"}
//!     },
//!     "required": ["name", "email"]
//! });
//!
//! let diff = SchemaDiff::compare(&old, &new);
//! assert!(diff.has_changes());
//! ```

#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fmt;

/// A path in a JSON structure.
pub type JsonPath = Vec<PathSegment>;

/// A segment in a JSON path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PathSegment {
    /// An object key.
    Key(String),
    /// An array index.
    Index(usize),
}

impl fmt::Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(k) => write!(f, ".{k}"),
            Self::Index(i) => write!(f, "[{i}]"),
        }
    }
}

impl PathSegment {
    /// Create a key segment.
    #[must_use]
    pub const fn key(k: String) -> Self {
        Self::Key(k)
    }

    /// Create an index segment.
    #[must_use]
    pub const fn index(i: usize) -> Self {
        Self::Index(i)
    }

    /// Check if this is a key.
    #[must_use]
    pub const fn is_key(&self) -> bool {
        matches!(self, Self::Key(_))
    }

    /// Check if this is an index.
    #[must_use]
    pub const fn is_index(&self) -> bool {
        matches!(self, Self::Index(_))
    }
}

/// Format a JSON path as a string.
#[must_use]
pub fn format_path(path: &[PathSegment]) -> String {
    if path.is_empty() {
        return String::from("$");
    }
    let mut s = String::from("$");
    for segment in path {
        s.push_str(&segment.to_string());
    }
    s
}

/// Kind of difference between JSON values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiffKind {
    /// Value was added.
    Added,
    /// Value was removed.
    Removed,
    /// Value was changed.
    Changed,
    /// Type changed (e.g., string to number).
    TypeChanged,
    /// Array length changed.
    ArrayLengthChanged,
    /// Object keys changed.
    KeysChanged,
}

impl fmt::Display for DiffKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Added => write!(f, "added"),
            Self::Removed => write!(f, "removed"),
            Self::Changed => write!(f, "changed"),
            Self::TypeChanged => write!(f, "type changed"),
            Self::ArrayLengthChanged => write!(f, "array length changed"),
            Self::KeysChanged => write!(f, "keys changed"),
        }
    }
}

/// A single difference between two JSON values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonDiff {
    /// Path to the differing value.
    pub path: JsonPath,
    /// Kind of difference.
    pub kind: DiffKind,
    /// Old value (None for additions).
    pub old_value: Option<Value>,
    /// New value (None for removals).
    pub new_value: Option<Value>,
}

impl JsonDiff {
    /// Create a new diff.
    #[must_use]
    pub const fn new(
        path: JsonPath,
        kind: DiffKind,
        old_value: Option<Value>,
        new_value: Option<Value>,
    ) -> Self {
        Self {
            path,
            kind,
            old_value,
            new_value,
        }
    }

    /// Create an addition diff.
    #[must_use]
    pub const fn added(path: JsonPath, value: Value) -> Self {
        Self::new(path, DiffKind::Added, None, Some(value))
    }

    /// Create a removal diff.
    #[must_use]
    pub const fn removed(path: JsonPath, value: Value) -> Self {
        Self::new(path, DiffKind::Removed, Some(value), None)
    }

    /// Create a change diff.
    #[must_use]
    pub const fn changed(path: JsonPath, old: Value, new: Value) -> Self {
        Self::new(path, DiffKind::Changed, Some(old), Some(new))
    }

    /// Get the path as a formatted string.
    #[must_use]
    pub fn path_string(&self) -> String {
        format_path(&self.path)
    }
}

impl fmt::Display for JsonDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {}",
            self.path_string(),
            self.kind
        )
    }
}

/// Compare two JSON values and return all differences.
#[must_use]
pub fn diff_json(a: &Value, b: &Value) -> Vec<JsonDiff> {
    let empty: JsonPath = vec![];
    diff_json_at_path(a, b, &empty)
}

fn diff_json_at_path(a: &Value, b: &Value, path: &JsonPath) -> Vec<JsonDiff> {
    match (a, b) {
        // Same value
        (Value::Null, Value::Null) => vec![],
        (Value::Bool(x), Value::Bool(y)) if x == y => vec![],
        (Value::Number(x), Value::Number(y)) if x == y => vec![],
        (Value::String(x), Value::String(y)) if x == y => vec![],

        // Type changes
        (Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_), _) => {
            vec![JsonDiff::changed(path.clone(), a.clone(), Value::Null)]
        }

        // Both objects
        (Value::Object(obj_a), Value::Object(obj_b)) => {
            diff_objects(obj_a, obj_b, path)
        }

        // Both arrays
        (Value::Array(arr_a), Value::Array(arr_b)) => {
            diff_arrays(arr_a, arr_b, path)
        }

        // Mixed types (object/array vs primitive)
        _ => vec![JsonDiff::changed(path.clone(), a.clone(), Value::Null)],
    }
}

fn diff_objects(
    a: &serde_json::Map<String, Value>,
    b: &serde_json::Map<String, Value>,
    path: &JsonPath,
) -> Vec<JsonDiff> {
    let mut diffs = Vec::new();

    let keys_a: HashSet<&str> = a.keys().map(std::string::String::as_str).collect();
    let keys_b: HashSet<&str> = b.keys().map(std::string::String::as_str).collect();

    // Removed keys
    for key in keys_a.difference(&keys_b) {
        let mut key_path = path.clone();
        key_path.push(PathSegment::key((*key).to_string()));
        diffs.push(JsonDiff::removed(key_path, a[*key].clone()));
    }

    // Added keys
    for key in keys_b.difference(&keys_a) {
        let mut key_path = path.clone();
        key_path.push(PathSegment::key((*key).to_string()));
        diffs.push(JsonDiff::added(key_path, b[*key].clone()));
    }

    // Common keys
    for key in keys_a.intersection(&keys_b) {
        let mut key_path = path.clone();
        key_path.push(PathSegment::key((*key).to_string()));
        diffs.extend(diff_json_at_path(&a[*key], &b[*key], &key_path));
    }

    diffs
}

fn diff_arrays(a: &[Value], b: &[Value], path: &JsonPath) -> Vec<JsonDiff> {
    let mut diffs = Vec::new();

    // If lengths differ significantly, just report length change
    if a.len() != b.len() && (a.len() > 10 || b.len() > 10) {
        diffs.push(JsonDiff::new(
            path.clone(),
            DiffKind::ArrayLengthChanged,
            Some(Value::Number(a.len().into())),
            Some(Value::Number(b.len().into())),
        ));
        return diffs;
    }

    // Compare element by element
    let max_len = a.len().max(b.len());
    for i in 0..max_len {
        let mut idx_path = path.clone();
        idx_path.push(PathSegment::index(i));

        match (a.get(i), b.get(i)) {
            (Some(v_a), Some(v_b)) => {
                diffs.extend(diff_json_at_path(v_a, v_b, &idx_path));
            }
            (Some(v_a), None) => {
                diffs.push(JsonDiff::removed(idx_path, v_a.clone()));
            }
            (None, Some(v_b)) => {
                diffs.push(JsonDiff::added(idx_path, v_b.clone()));
            }
            (None, None) => {
                // This shouldn't happen since we iterate up to max_len
            }
        }
    }

    diffs
}

/// Configuration for schema comparison.
#[derive(Debug, Clone, Default)]
pub struct SchemaDiffConfig {
    /// Ignore these keys when comparing.
    pub ignore_keys: HashSet<String>,
    /// Treat these keys as equivalent (e.g., "id" and "_id").
    pub equivalent_keys: Vec<(String, String)>,
    /// Ignore array order when comparing.
    pub ignore_array_order: bool,
}

impl SchemaDiffConfig {
    /// Create a new config.
    #[must_use]
    pub fn new() -> Self {
        Self {
            ignore_keys: HashSet::new(),
            equivalent_keys: vec![],
            ignore_array_order: false,
        }
    }

    /// Add a key to ignore.
    #[must_use]
    pub fn with_ignore_key(mut self, key: impl Into<String>) -> Self {
        self.ignore_keys.insert(key.into());
        self
    }
}

/// Result of comparing two schemas.
#[derive(Debug, Clone, Default)]
pub struct SchemaDiff {
    /// All differences found.
    pub diffs: Vec<JsonDiff>,
    /// Configuration used.
    pub config: SchemaDiffConfig,
}

impl SchemaDiff {
    /// Create a new empty diff.
    #[must_use]
    pub fn new() -> Self {
        Self {
            diffs: vec![],
            config: SchemaDiffConfig::new(),
        }
    }

    /// Create a diff with the given config.
    #[must_use]
    pub const fn with_config(config: SchemaDiffConfig) -> Self {
        Self {
            diffs: vec![],
            config,
        }
    }

    /// Compare two schemas.
    #[must_use]
    pub fn compare(old: &Value, new: &Value) -> Self {
        Self::compare_with_config(old, new, SchemaDiffConfig::new())
    }

    /// Compare two schemas with configuration.
    #[must_use]
    pub fn compare_with_config(old: &Value, new: &Value, config: SchemaDiffConfig) -> Self {
        let mut diff = Self::with_config(config);
        diff.diffs = diff_json_filtered(old, new, &diff.config);
        diff
    }

    /// Check if there are any changes.
    #[must_use]
    pub const fn has_changes(&self) -> bool {
        !self.diffs.is_empty()
    }

    /// Get the number of changes.
    #[must_use]
    pub const fn change_count(&self) -> usize {
        self.diffs.len()
    }

    /// Get changes of a specific kind.
    #[must_use]
    pub fn changes_of_kind(&self, kind: DiffKind) -> Vec<&JsonDiff> {
        self.diffs.iter().filter(|d| d.kind == kind).collect()
    }

    /// Get added paths.
    #[must_use]
    pub fn added(&self) -> Vec<&JsonDiff> {
        self.changes_of_kind(DiffKind::Added)
    }

    /// Get removed paths.
    #[must_use]
    pub fn removed(&self) -> Vec<&JsonDiff> {
        self.changes_of_kind(DiffKind::Removed)
    }

    /// Get changed paths.
    #[must_use]
    pub fn changed(&self) -> Vec<&JsonDiff> {
        self.changes_of_kind(DiffKind::Changed)
    }
}

fn diff_json_filtered(a: &Value, b: &Value, config: &SchemaDiffConfig) -> Vec<JsonDiff> {
    let all_diffs = diff_json(a, b);
    all_diffs
        .into_iter()
        .filter(|diff| {
            // Check if any path segment is in ignore_keys
            !diff.path.iter().any(|seg| {
                if let PathSegment::Key(k) = seg {
                    config.ignore_keys.contains(k)
                } else {
                    false
                }
            })
        })
        .collect()
}

/// Summary of schema differences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchemaDiffSummary {
    /// Total number of changes.
    pub total_changes: usize,
    /// Number of additions.
    pub additions: usize,
    /// Number of removals.
    pub removals: usize,
    /// Number of modifications.
    pub modifications: usize,
    /// Paths that were added.
    pub added_paths: Vec<String>,
    /// Paths that were removed.
    pub removed_paths: Vec<String>,
    /// Paths that were modified.
    pub modified_paths: Vec<String>,
}

impl SchemaDiffSummary {
    /// Create a summary from a schema diff.
    #[must_use]
    pub fn from_diff(diff: &SchemaDiff) -> Self {
        let mut summary = Self::default();

        for d in &diff.diffs {
            summary.total_changes += 1;
            let path = d.path_string();

            match d.kind {
                DiffKind::Added => {
                    summary.additions += 1;
                    summary.added_paths.push(path);
                }
                DiffKind::Removed => {
                    summary.removals += 1;
                    summary.removed_paths.push(path);
                }
                _ => {
                    summary.modifications += 1;
                    summary.modified_paths.push(path);
                }
            }
        }

        summary
    }

    /// Check if there are any changes.
    #[must_use]
    pub const fn has_changes(&self) -> bool {
        self.total_changes > 0
    }

    /// Check if breaking changes exist (removals or modifications).
    #[must_use]
    pub const fn has_breaking_changes(&self) -> bool {
        self.removals > 0 || self.modifications > 0
    }
}

impl fmt::Display for SchemaDiffSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Schema diff: {} changes ({} added, {} removed, {} modified)",
            self.total_changes, self.additions, self.removals, self.modifications
        )
    }
}

/// Check if two JSON values are equivalent.
#[must_use]
pub fn json_eq(a: &Value, b: &Value) -> bool {
    diff_json(a, b).is_empty()
}

/// Check if two JSON values are equivalent, ignoring key order.
#[must_use]
pub fn json_eq_ignore_order(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Object(obj_a), Value::Object(obj_b)) => {
            if obj_a.len() != obj_b.len() {
                return false;
            }
            for (key, val_a) in obj_a {
                match obj_b.get(key) {
                    Some(val_b) => {
                        if !json_eq_ignore_order(val_a, val_b) {
                            return false;
                        }
                    }
                    None => return false,
                }
            }
            true
        }
        (Value::Array(arr_a), Value::Array(arr_b)) => {
            if arr_a.len() != arr_b.len() {
                return false;
            }
            arr_a.iter().zip(arr_b.iter()).all(|(a, b)| json_eq_ignore_order(a, b))
        }
        _ => a == b,
    }
}

/// Deep merge two JSON objects.
///
/// For objects, keys are merged recursively.
/// For arrays, the second array replaces the first.
/// For primitives, the second value replaces the first.
#[must_use]
pub fn merge_json(base: &Value, overlay: &Value) -> Value {
    match (base, overlay) {
        (Value::Object(base_obj), Value::Object(overlay_obj)) => {
            let mut merged = base_obj.clone();
            for (key, val) in overlay_obj {
                match merged.get(key) {
                    Some(base_val) => {
                        merged.insert(key.clone(), merge_json(base_val, val));
                    }
                    None => {
                        merged.insert(key.clone(), val.clone());
                    }
                }
            }
            Value::Object(merged)
        }
        _ => overlay.clone(),
    }
}

/// Extract all paths from a JSON value.
#[must_use]
pub fn extract_paths(value: &Value) -> Vec<JsonPath> {
    let mut paths = Vec::new();
    let empty: JsonPath = vec![];
    extract_paths_recursive(value, &empty, &mut paths);
    paths
}

fn extract_paths_recursive(value: &Value, current_path: &JsonPath, paths: &mut Vec<JsonPath>) {
    paths.push(current_path.clone());

    match value {
        Value::Object(obj) => {
            for (key, val) in obj {
                let mut child_path = current_path.clone();
                child_path.push(PathSegment::key(key.clone()));
                extract_paths_recursive(val, &child_path, paths);
            }
        }
        Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                let mut child_path = current_path.clone();
                child_path.push(PathSegment::index(i));
                extract_paths_recursive(val, &child_path, paths);
            }
        }
        _ => {}
    }
}

/// Get a value at a path.
#[must_use]
pub fn get_at_path(value: &Value, path: &[PathSegment]) -> Option<Value> {
    let mut current = value;
    for segment in path {
        match (current, segment) {
            (Value::Object(obj), PathSegment::Key(k)) => {
                current = obj.get(k)?;
            }
            (Value::Array(arr), PathSegment::Index(i)) => {
                current = arr.get(*i)?;
            }
            _ => return None,
        }
    }
    Some(current.clone())
}

/// Set a value at a path.
///
/// # Errors
///
/// Returns an error if the path is invalid for the given structure.
pub fn set_at_path(value: &mut Value, path: &[PathSegment], new_value: Value) -> Result<(), String> {
    if path.is_empty() {
        *value = new_value;
        return Ok(());
    }

    let mut current = value;
    for (i, segment) in path.iter().enumerate() {
        let is_last = i == path.len() - 1;

        match (current, segment) {
            (Value::Object(obj), PathSegment::Key(k)) => {
                if is_last {
                    obj.insert(k.clone(), new_value);
                    return Ok(());
                }
                current = obj.get_mut(k).ok_or_else(|| format!("Key not found: {k}"))?;
            }
            (Value::Array(arr), PathSegment::Index(idx)) => {
                if is_last {
                    if *idx < arr.len() {
                        arr[*idx] = new_value;
                        return Ok(());
                    }
                    return Err(format!("Index out of bounds: {idx}"));
                }
                current = arr.get_mut(*idx).ok_or_else(|| format!("Index out of bounds: {idx}"))?;
            }
            _ => return Err(format!("Invalid path segment at position {i}")),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_path_segment_display() {
        assert_eq!(PathSegment::key("foo".to_string()).to_string(), ".foo");
        assert_eq!(PathSegment::index(5).to_string(), "[5]");
    }

    #[test]
    fn test_format_path() {
        assert_eq!(format_path(&[]), "$");
        assert_eq!(
            format_path(&[
                PathSegment::key("foo".to_string()),
                PathSegment::index(0),
                PathSegment::key("bar".to_string()),
            ]),
            "$.foo[0].bar"
        );
    }

    #[test]
    fn test_diff_json_equal() {
        let a = json!({"name": "Alice", "age": 30});
        let b = json!({"name": "Alice", "age": 30});
        assert!(diff_json(&a, &b).is_empty());
    }

    #[test]
    fn test_diff_json_addition() {
        let a = json!({"name": "Alice"});
        let b = json!({"name": "Alice", "age": 30});
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].kind, DiffKind::Added);
    }

    #[test]
    fn test_diff_json_removal() {
        let a = json!({"name": "Alice", "age": 30});
        let b = json!({"name": "Alice"});
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].kind, DiffKind::Removed);
    }

    #[test]
    fn test_diff_json_change() {
        let a = json!({"age": 30});
        let b = json!({"age": 31});
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].kind, DiffKind::Changed);
    }

    #[test]
    fn test_diff_json_nested() {
        let a = json!({"user": {"name": "Alice"}});
        let b = json!({"user": {"name": "Bob"}});
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path_string(), "$.user.name");
    }

    #[test]
    fn test_diff_json_array() {
        let a = json!([1, 2, 3]);
        let b = json!([1, 2, 4]);
        let diffs = diff_json(&a, &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path_string(), "$[2]");
    }

    #[test]
    fn test_schema_diff_has_changes() {
        let old = json!({"a": 1});
        let new = json!({"a": 2});
        let diff = SchemaDiff::compare(&old, &new);
        assert!(diff.has_changes());
    }

    #[test]
    fn test_schema_diff_no_changes() {
        let old = json!({"a": 1});
        let new = json!({"a": 1});
        let diff = SchemaDiff::compare(&old, &new);
        assert!(!diff.has_changes());
    }

    #[test]
    fn test_schema_diff_summary() {
        let old = json!({"a": 1, "b": 2, "c": 3});
        let new = json!({"a": 1, "b": 3, "d": 4});
        let diff = SchemaDiff::compare(&old, &new);
        let summary = SchemaDiffSummary::from_diff(&diff);

        assert_eq!(summary.total_changes, 3);
        assert_eq!(summary.additions, 1); // d
        assert_eq!(summary.removals, 1); // c
        assert_eq!(summary.modifications, 1); // b
    }

    #[test]
    fn test_json_eq() {
        let a = json!({"a": 1});
        let b = json!({"a": 1});
        assert!(json_eq(&a, &b));

        let c = json!({"a": 2});
        assert!(!json_eq(&a, &c));
    }

    #[test]
    fn test_merge_json() {
        let base = json!({"a": 1, "b": {"c": 2}});
        let overlay = json!({"b": {"d": 3}, "e": 4});
        let merged = merge_json(&base, &overlay);

        assert_eq!(merged["a"], 1);
        assert_eq!(merged["b"]["c"], 2);
        assert_eq!(merged["b"]["d"], 3);
        assert_eq!(merged["e"], 4);
    }

    #[test]
    fn test_extract_paths() {
        let value = json!({"a": {"b": 1}, "c": [1, 2]});
        let paths = extract_paths(&value);

        assert!(paths.contains(&vec![]));
        assert!(paths.contains(&vec![PathSegment::key("a".to_string())]));
        assert!(paths.contains(&vec![PathSegment::key("a".to_string()), PathSegment::key("b".to_string())]));
        assert!(paths.contains(&vec![PathSegment::key("c".to_string())]));
        assert!(paths.contains(&vec![PathSegment::key("c".to_string()), PathSegment::index(0)]));
    }

    #[test]
    fn test_get_at_path() {
        let value = json!({"a": {"b": [1, 2, 3]}});

        let result = get_at_path(&value, &[PathSegment::key("a".to_string())]);
        assert_eq!(result, Some(json!({"b": [1, 2, 3]})));

        let result = get_at_path(&value, &[PathSegment::key("a".to_string()), PathSegment::key("b".to_string()), PathSegment::index(1)]);
        assert_eq!(result, Some(json!(2)));

        let result = get_at_path(&value, &[PathSegment::key("x".to_string())]);
        assert_eq!(result, None);
    }

    #[test]
    fn test_set_at_path() {
        let mut value = json!({"a": 1});

        set_at_path(&mut value, &[PathSegment::key("a".to_string())], json!(2)).unwrap();
        assert_eq!(value, json!({"a": 2}));

        set_at_path(&mut value, &[], json!({"b": 3})).unwrap();
        assert_eq!(value, json!({"b": 3}));
    }

    #[test]
    fn test_schema_diff_config_ignore_keys() {
        let config = SchemaDiffConfig::new().with_ignore_key("timestamp");
        let old = json!({"a": 1, "timestamp": "2024-01-01"});
        let new = json!({"a": 1, "timestamp": "2024-01-02"});

        let diff = SchemaDiff::compare_with_config(&old, &new, config);
        assert!(!diff.has_changes()); // timestamp change ignored
    }

    #[test]
    fn test_diff_kind_display() {
        assert_eq!(DiffKind::Added.to_string(), "added");
        assert_eq!(DiffKind::Removed.to_string(), "removed");
        assert_eq!(DiffKind::Changed.to_string(), "changed");
    }

    #[test]
    fn test_json_diff_display() {
        let diff = JsonDiff::added(vec![PathSegment::key("foo".to_string())], json!(1));
        assert_eq!(diff.to_string(), "$.foo: added");
    }

    #[test]
    fn test_summary_has_breaking_changes() {
        let summary = SchemaDiffSummary {
            total_changes: 2,
            additions: 1,
            removals: 1,
            modifications: 0,
            added_paths: vec!["$b".to_string()],
            removed_paths: vec!["$a".to_string()],
            modified_paths: vec![],
        };
        assert!(summary.has_breaking_changes());

        let summary2 = SchemaDiffSummary {
            total_changes: 1,
            additions: 1,
            removals: 0,
            modifications: 0,
            added_paths: vec!["$b".to_string()],
            removed_paths: vec![],
            modified_paths: vec![],
        };
        assert!(!summary2.has_breaking_changes());
    }
}
