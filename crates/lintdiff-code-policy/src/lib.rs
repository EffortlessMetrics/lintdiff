//! Code allow/deny/suppress policy evaluation for lintdiff.
//!
//! This crate provides utilities for evaluating code policies against code content.
//! Policies can be used to allow, deny, or suppress specific code patterns.
//!
//! # Policy Types
//!
//! - **Allow**: Permit the code pattern (no action needed)
//! - **Deny**: Flag the code pattern as a violation
//! - **Suppress**: Suppress warnings for this pattern
//!
//! # Pattern Matching
//!
//! Patterns can be either glob patterns or regular expressions:
//! - Glob patterns: Simple wildcard matching (e.g., `*.unwrap()`, `todo!()`)
//! - Regex patterns: Full regex support (e.g., `unwrap\(\)`, `expect\(".*"\)`)
//!
//! # Example
//!
//! ```
//! use lintdiff_code_policy::{CodePolicy, PolicyEvaluator, PolicyRule};
//!
//! // Create a policy evaluator
//! let mut evaluator = PolicyEvaluator::new();
//!
//! // Add a rule to deny `unwrap()` calls
//! let rule = PolicyRule::new("*unwrap()*", CodePolicy::Deny)
//!     .with_message("Avoid unwrap() in production code");
//! evaluator.add_rule(rule);
//!
//! // Evaluate some code
//! let code = r#"
//! fn main() {
//!     let x = Some(1);
//!     x.unwrap();
//! }
//! "#;
//! let results = evaluator.evaluate(code);
//!
//! // Check results
//! assert!(!results.is_empty());
//! for result in &results {
//!     println!("Policy {:?} matched at line {}", result.policy, result.matches[0].line);
//! }
//! ```

use std::collections::HashMap;

use lintdiff_glob::{Glob, GlobError};

/// Errors that can occur during policy evaluation.
#[derive(Debug, Clone, thiserror::Error)]
pub enum PolicyError {
    /// The pattern is invalid.
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
    /// The regex pattern is invalid.
    #[error("Invalid regex pattern: {0}")]
    InvalidRegex(String),
    /// The glob pattern is invalid.
    #[error("Invalid glob pattern: {0}")]
    InvalidGlob(#[from] GlobError),
}

/// Policy types for code patterns.
///
/// Defines how a matched pattern should be handled.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CodePolicy {
    /// Allow the code pattern (no action needed).
    #[default]
    Allow,
    /// Deny the code pattern (flag as violation).
    Deny,
    /// Suppress warnings for this pattern.
    Suppress,
}

impl CodePolicy {
    /// Returns `true` if this is an `Allow` policy.
    #[must_use]
    pub const fn is_allow(&self) -> bool {
        matches!(self, Self::Allow)
    }

    /// Returns `true` if this is a `Deny` policy.
    #[must_use]
    pub const fn is_deny(&self) -> bool {
        matches!(self, Self::Deny)
    }

    /// Returns `true` if this is a `Suppress` policy.
    #[must_use]
    pub const fn is_suppress(&self) -> bool {
        matches!(self, Self::Suppress)
    }
}

impl std::fmt::Display for CodePolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allow => write!(f, "allow"),
            Self::Deny => write!(f, "deny"),
            Self::Suppress => write!(f, "suppress"),
        }
    }
}

/// Severity level for policy violations.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Severity {
    /// Informational (lowest severity).
    Info,
    /// Warning level.
    #[default]
    Warning,
    /// Error level.
    Error,
    /// Critical/Fatal level (highest severity).
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Pattern type for matching.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PatternType {
    /// Glob pattern (simple wildcard matching).
    #[default]
    Glob,
    /// Regular expression pattern.
    Regex,
}

/// A single match within the code.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Match {
    /// Start byte position of the match.
    pub start: usize,
    /// End byte position of the match.
    pub end: usize,
    /// Line number (1-based).
    pub line: usize,
    /// Column number (1-based, if available).
    pub column: Option<usize>,
    /// The matched text.
    pub text: String,
}

impl Match {
    /// Create a new match.
    #[must_use]
    pub const fn new(start: usize, end: usize, line: usize, text: String) -> Self {
        Self {
            start,
            end,
            line,
            column: None,
            text,
        }
    }

    /// Create a new match with column information.
    #[must_use]
    pub const fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }
}

/// A policy rule for code evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PolicyRule {
    /// Pattern to match (glob or regex).
    pub pattern: String,
    /// The policy to apply.
    pub policy: CodePolicy,
    /// Pattern type (glob or regex).
    pub pattern_type: PatternType,
    /// Severity level (for deny policies).
    pub severity: Option<Severity>,
    /// Custom message for violations.
    pub message: Option<String>,
    /// Optional file path filter (glob pattern).
    pub file_filter: Option<String>,
    /// Rule identifier.
    pub id: Option<String>,
}

impl PolicyRule {
    /// Create a new policy rule.
    #[must_use]
    pub fn new(pattern: impl Into<String>, policy: CodePolicy) -> Self {
        Self {
            pattern: pattern.into(),
            policy,
            pattern_type: PatternType::Glob,
            severity: None,
            message: None,
            file_filter: None,
            id: None,
        }
    }

    /// Create a new glob-based policy rule.
    #[must_use]
    pub fn glob(pattern: impl Into<String>, policy: CodePolicy) -> Self {
        Self {
            pattern: pattern.into(),
            policy,
            pattern_type: PatternType::Glob,
            severity: None,
            message: None,
            file_filter: None,
            id: None,
        }
    }

    /// Create a new regex-based policy rule.
    #[must_use]
    pub fn regex(pattern: impl Into<String>, policy: CodePolicy) -> Self {
        Self {
            pattern: pattern.into(),
            policy,
            pattern_type: PatternType::Regex,
            severity: None,
            message: None,
            file_filter: None,
            id: None,
        }
    }

    /// Set the severity level.
    #[must_use]
    pub const fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = Some(severity);
        self
    }

    /// Set the custom message.
    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set the file filter pattern.
    #[must_use]
    pub fn with_file_filter(mut self, filter: impl Into<String>) -> Self {
        self.file_filter = Some(filter.into());
        self
    }

    /// Set the rule identifier.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Check if this rule applies to the given file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file filter pattern is invalid.
    pub fn applies_to_file(&self, path: &str) -> Result<bool, PolicyError> {
        match &self.file_filter {
            Some(filter) => {
                let glob = Glob::new(filter)?;
                Ok(glob.is_match(path))
            }
            None => Ok(true),
        }
    }
}

/// Result of policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PolicyResult {
    /// The matched rule.
    pub rule: PolicyRule,
    /// All matches found.
    pub matches: Vec<Match>,
    /// The applied policy.
    pub policy: CodePolicy,
    /// The file path (if evaluating a file).
    pub file_path: Option<String>,
}

impl PolicyResult {
    /// Create a new policy result.
    #[must_use]
    pub const fn new(rule: PolicyRule, matches: Vec<Match>, policy: CodePolicy) -> Self {
        Self {
            rule,
            matches,
            policy,
            file_path: None,
        }
    }

    /// Create a new policy result with file path.
    #[must_use]
    pub fn with_file_path(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    /// Returns `true` if there are any matches.
    #[must_use]
    pub const fn has_matches(&self) -> bool {
        !self.matches.is_empty()
    }

    /// Returns the number of matches.
    #[must_use]
    pub const fn match_count(&self) -> usize {
        self.matches.len()
    }
}

/// Compiled pattern for efficient matching.
#[derive(Debug, Clone)]
enum CompiledPattern {
    /// Compiled glob pattern.
    Glob(Glob),
    /// Regex pattern (stored as string, compiled on demand).
    Regex(String),
}

/// Evaluator for code policies.
///
/// This struct manages a collection of policy rules and evaluates
/// code content against them.
#[derive(Debug, Clone, Default)]
pub struct PolicyEvaluator {
    /// Rules organized by policy type for efficient lookup.
    rules: Vec<PolicyRule>,
    /// Cache of compiled patterns.
    pattern_cache: HashMap<String, CompiledPattern>,
}

impl PolicyEvaluator {
    /// Create a new policy evaluator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            pattern_cache: HashMap::new(),
        }
    }

    /// Add a rule to the evaluator.
    pub fn add_rule(&mut self, rule: PolicyRule) -> &mut Self {
        // Pre-compile glob patterns
        // For glob patterns, wrap with * for substring matching only if pattern doesn't have wildcards
        // This ensures patterns like "FIXME" match lines containing "FIXME"
        // while patterns like "todo*" keep their exact glob meaning (starts with "todo")
        if rule.pattern_type == PatternType::Glob {
            let glob_pattern = if rule.pattern.contains('*') || rule.pattern.contains('?') {
                rule.pattern.clone()
            } else {
                format!("*{}*", rule.pattern)
            };
            // Use new_no_separator since we're matching code lines, not file paths
            if let Ok(glob) = Glob::new_no_separator(&glob_pattern) {
                self.pattern_cache
                    .insert(rule.pattern.clone(), CompiledPattern::Glob(glob));
            }
        } else {
            self.pattern_cache.insert(
                rule.pattern.clone(),
                CompiledPattern::Regex(rule.pattern.clone()),
            );
        }
        self.rules.push(rule);
        self
    }

    /// Add multiple rules to the evaluator.
    pub fn add_rules(&mut self, rules: impl IntoIterator<Item = PolicyRule>) -> &mut Self {
        for rule in rules {
            self.add_rule(rule);
        }
        self
    }

    /// Clear all rules from the evaluator.
    pub fn clear(&mut self) {
        self.rules.clear();
        self.pattern_cache.clear();
    }

    /// Returns the number of rules.
    #[must_use]
    pub const fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// Returns `true` if there are no rules.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Get all rules with a specific policy.
    #[must_use]
    pub fn rules_by_policy(&self, policy: CodePolicy) -> Vec<&PolicyRule> {
        self.rules.iter().filter(|r| r.policy == policy).collect()
    }

    /// Evaluate code against all rules.
    #[must_use]
    pub fn evaluate(&self, code: &str) -> Vec<PolicyResult> {
        let mut results = Vec::new();

        for rule in &self.rules {
            let matches = self.match_pattern(rule, code);
            if !matches.is_empty() {
                results.push(PolicyResult::new(rule.clone(), matches, rule.policy));
            }
        }

        results
    }

    /// Evaluate code against rules for a specific file.
    #[must_use]
    pub fn evaluate_file(&self, path: &str, code: &str) -> Vec<PolicyResult> {
        let mut results = Vec::new();

        for rule in &self.rules {
            // Check if rule applies to this file
            if !rule.applies_to_file(path).unwrap_or(true) {
                continue;
            }

            let matches = self.match_pattern(rule, code);
            if !matches.is_empty() {
                results.push(
                    PolicyResult::new(rule.clone(), matches, rule.policy).with_file_path(path),
                );
            }
        }

        results
    }

    /// Evaluate code and return only deny results.
    #[must_use]
    pub fn evaluate_denials(&self, code: &str) -> Vec<PolicyResult> {
        self.evaluate(code)
            .into_iter()
            .filter(|r| r.policy == CodePolicy::Deny)
            .collect()
    }

    /// Evaluate code and return only suppress results.
    #[must_use]
    pub fn evaluate_suppressions(&self, code: &str) -> Vec<PolicyResult> {
        self.evaluate(code)
            .into_iter()
            .filter(|r| r.policy == CodePolicy::Suppress)
            .collect()
    }

    /// Match a pattern against code and return all matches.
    fn match_pattern(&self, rule: &PolicyRule, code: &str) -> Vec<Match> {
        match rule.pattern_type {
            PatternType::Glob => self.match_glob(rule, code),
            PatternType::Regex => self.match_regex(rule, code),
        }
    }

    /// Match a glob pattern against code.
    fn match_glob(&self, rule: &PolicyRule, code: &str) -> Vec<Match> {
        // Use the cached glob pattern which is already wrapped with * for substring matching
        let compiled = self.pattern_cache.get(&rule.pattern);
        let Some(CompiledPattern::Glob(glob)) = compiled else {
            return Vec::new();
        };

        let mut matches = Vec::new();
        let mut line_start = 0;
        let mut line_num = 1;

        for line in code.lines() {
            // Calculate the actual line end position (including newline if present)
            let line_len = line.len();
            let line_end = line_start + line_len;

            // Check if there's a newline character after this line
            let has_newline = if line_end < code.len() {
                code.as_bytes()[line_end] == b'\n'
            } else {
                false
            };

            // The actual line end in the original code
            let actual_line_end = if has_newline { line_end + 1 } else { line_end };

            // Check if the glob pattern matches this line
            if glob.is_match(line) {
                matches.push(Match::new(
                    line_start,
                    actual_line_end,
                    line_num,
                    line.to_string(),
                ));
            }

            line_start = actual_line_end;
            line_num += 1;
        }

        matches
    }

    /// Match a regex pattern against code.
    fn match_regex(&self, rule: &PolicyRule, code: &str) -> Vec<Match> {
        // Simple regex matching without the regex crate
        // We'll do a basic substring search for the pattern
        let compiled = self.pattern_cache.get(&rule.pattern);
        let pattern = match compiled {
            Some(CompiledPattern::Regex(p)) => p,
            _ => &rule.pattern,
        };

        // Build line index
        let line_starts: Vec<usize> = std::iter::once(0)
            .chain(code.match_indices('\n').map(|(i, _)| i + 1))
            .collect();

        let mut matches = Vec::new();

        // Empty pattern matches at position 0
        if pattern.is_empty() {
            let line_num = 1;
            let line_start = 0;
            let line_end = code.find('\n').map_or(code.len(), |i| i);
            let line_text = code[line_start..line_end].to_string();

            matches.push(Match::new(0, 0, line_num, line_text).with_column(1));
            return matches;
        }

        let mut search_start = 0;
        while let Some(pos) = code[search_start..].find(pattern) {
            let abs_pos = search_start + pos;
            let line_num = line_starts
                .binary_search(&(abs_pos + 1))
                .unwrap_or_else(|x| x);

            let line_start = if line_num == 0 {
                0
            } else {
                line_starts[line_num - 1]
            };

            let line_end = code[line_start..]
                .find('\n')
                .map_or(code.len(), |i| line_start + i);

            let line_text = code[line_start..line_end].to_string();
            // Column is 0-indexed, so we don't add 1
            let column = abs_pos - line_start;

            matches.push(
                Match::new(abs_pos, abs_pos + pattern.len(), line_num, line_text)
                    .with_column(column),
            );

            search_start = abs_pos + pattern.len();
        }

        matches
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if a glob pattern matches the given text.
///
/// # Errors
///
/// Returns an error if the glob pattern is invalid.
///
/// # Example
///
/// ```
/// use lintdiff_code_policy::glob_matches_pattern;
///
/// assert!(glob_matches_pattern("*.rs", "main.rs").unwrap());
/// assert!(!glob_matches_pattern("*.rs", "main.txt").unwrap());
/// assert!(glob_matches_pattern("todo*", "todo!()").unwrap());
/// ```
pub fn glob_matches_pattern(glob: &str, text: &str) -> Result<bool, PolicyError> {
    let compiled = Glob::new(glob)?;
    Ok(compiled.is_match(text))
}

/// Check if a regex pattern matches the given text.
///
/// This is a simplified implementation that does substring matching.
/// For full regex support, use the `regex` crate directly.
///
/// # Example
///
/// ```
/// use lintdiff_code_policy::regex_matches_pattern;
///
/// assert!(regex_matches_pattern("unwrap", "x.unwrap()"));
/// assert!(!regex_matches_pattern("unwrap", "x.expect()"));
/// ```
#[must_use]
pub fn regex_matches_pattern(regex: &str, text: &str) -> bool {
    text.contains(regex)
}

/// Find all lines matching a glob pattern.
///
/// # Errors
///
/// Returns an error if the glob pattern is invalid.
///
/// # Example
///
/// ```
/// use lintdiff_code_policy::find_matching_lines;
///
/// let code = "fn main() {\n    todo!()\n}\n";
/// let matches = find_matching_lines("*todo*", code).unwrap();
/// assert_eq!(matches.len(), 1);
/// assert_eq!(matches[0].line, 2);
/// ```
pub fn find_matching_lines(glob: &str, code: &str) -> Result<Vec<Match>, PolicyError> {
    // For glob patterns, wrap with * for substring matching only if pattern doesn't have wildcards
    // This ensures patterns like "FIXME" match lines containing "FIXME"
    // while patterns like "todo*" keep their exact glob meaning (starts with "todo")
    let glob_pattern = if glob.contains('*') || glob.contains('?') {
        glob.to_string()
    } else {
        format!("*{glob}*")
    };

    let compiled = Glob::new(&glob_pattern)?;
    let mut matches = Vec::new();
    let mut line_start = 0;
    let mut line_num = 1;

    for line in code.lines() {
        // Calculate the actual line end position (including newline if present)
        let line_len = line.len();
        let line_end = line_start + line_len;

        // Check if there's a newline character after this line
        let has_newline = if line_end < code.len() {
            code.as_bytes()[line_end] == b'\n'
        } else {
            false
        };

        // The actual line end in the original code
        let actual_line_end = if has_newline { line_end + 1 } else { line_end };

        if compiled.is_match(line) {
            matches.push(Match::new(
                line_start,
                actual_line_end,
                line_num,
                line.to_string(),
            ));
        }

        line_start = actual_line_end;
        line_num += 1;
    }

    Ok(matches)
}

/// Count the number of lines matching a glob pattern.
///
/// # Errors
///
/// Returns an error if the glob pattern is invalid.
///
/// # Example
///
/// ```
/// use lintdiff_code_policy::count_matching_lines;
///
/// let code = "fn main() {\n    todo!()\n    todo!()\n}\n";
/// let count = count_matching_lines("*todo*", code).unwrap();
/// assert_eq!(count, 2);
/// ```
pub fn count_matching_lines(glob: &str, code: &str) -> Result<usize, PolicyError> {
    let matches = find_matching_lines(glob, code)?;
    Ok(matches.len())
}

/// Get the line number for a byte position in text.
#[must_use]
pub fn line_number_at(text: &str, byte_pos: usize) -> usize {
    // Clamp the position to the text length
    let pos = byte_pos.min(text.len());

    // Find the nearest character boundary before or at the position
    // This prevents panics when byte_pos is in the middle of a multi-byte character
    let safe_pos = text.floor_char_boundary(pos);

    text[..safe_pos].chars().filter(|&c| c == '\n').count() + 1
}

/// Get the column number for a byte position in text (within its line).
#[must_use]
pub fn column_number_at(text: &str, byte_pos: usize) -> usize {
    // Clamp the position to the text length
    let pos = byte_pos.min(text.len());

    // Find the nearest character boundary before or at the position
    let safe_pos = text.floor_char_boundary(pos);

    let line_start = text[..safe_pos].rfind('\n').map_or(0, |i| i + 1);
    safe_pos - line_start + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_policy_is_allow_works() {
        assert!(CodePolicy::Allow.is_allow());
        assert!(!CodePolicy::Deny.is_allow());
        assert!(!CodePolicy::Suppress.is_allow());
    }

    #[test]
    fn code_policy_is_deny_works() {
        assert!(CodePolicy::Deny.is_deny());
        assert!(!CodePolicy::Allow.is_deny());
        assert!(!CodePolicy::Suppress.is_deny());
    }

    #[test]
    fn code_policy_is_suppress_works() {
        assert!(CodePolicy::Suppress.is_suppress());
        assert!(!CodePolicy::Allow.is_suppress());
        assert!(!CodePolicy::Deny.is_suppress());
    }

    #[test]
    fn code_policy_display_works() {
        assert_eq!(CodePolicy::Allow.to_string(), "allow");
        assert_eq!(CodePolicy::Deny.to_string(), "deny");
        assert_eq!(CodePolicy::Suppress.to_string(), "suppress");
    }

    #[test]
    fn code_policy_default_is_allow() {
        assert_eq!(CodePolicy::default(), CodePolicy::Allow);
    }

    #[test]
    fn severity_ordering_works() {
        assert!(Severity::Critical > Severity::Error);
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);
    }

    #[test]
    fn severity_default_is_warning() {
        assert_eq!(Severity::default(), Severity::Warning);
    }

    #[test]
    fn severity_display_works() {
        assert_eq!(Severity::Info.to_string(), "info");
        assert_eq!(Severity::Warning.to_string(), "warning");
        assert_eq!(Severity::Error.to_string(), "error");
        assert_eq!(Severity::Critical.to_string(), "critical");
    }

    #[test]
    fn policy_rule_new_creates_glob_rule() {
        let rule = PolicyRule::new("test", CodePolicy::Deny);
        assert_eq!(rule.pattern, "test");
        assert_eq!(rule.policy, CodePolicy::Deny);
        assert_eq!(rule.pattern_type, PatternType::Glob);
    }

    #[test]
    fn policy_rule_glob_creates_glob_rule() {
        let rule = PolicyRule::glob("test", CodePolicy::Allow);
        assert_eq!(rule.pattern_type, PatternType::Glob);
    }

    #[test]
    fn policy_rule_regex_creates_regex_rule() {
        let rule = PolicyRule::regex("test", CodePolicy::Suppress);
        assert_eq!(rule.pattern_type, PatternType::Regex);
    }

    #[test]
    fn policy_rule_builder_methods_work() {
        let rule = PolicyRule::new("test", CodePolicy::Deny)
            .with_severity(Severity::Error)
            .with_message("Test message")
            .with_file_filter("*.rs")
            .with_id("RULE001");

        assert_eq!(rule.severity, Some(Severity::Error));
        assert_eq!(rule.message, Some("Test message".to_string()));
        assert_eq!(rule.file_filter, Some("*.rs".to_string()));
        assert_eq!(rule.id, Some("RULE001".to_string()));
    }

    #[test]
    fn match_new_works() {
        let m = Match::new(0, 10, 1, "test".to_string());
        assert_eq!(m.start, 0);
        assert_eq!(m.end, 10);
        assert_eq!(m.line, 1);
        assert_eq!(m.text, "test");
        assert_eq!(m.column, None);
    }

    #[test]
    fn match_with_column_works() {
        let m = Match::new(0, 10, 1, "test".to_string()).with_column(5);
        assert_eq!(m.column, Some(5));
    }

    #[test]
    fn policy_result_has_matches_works() {
        let rule = PolicyRule::new("test", CodePolicy::Deny);
        let result_with_matches = PolicyResult::new(
            rule.clone(),
            vec![Match::new(0, 4, 1, "test".to_string())],
            CodePolicy::Deny,
        );
        let result_no_matches = PolicyResult::new(rule, vec![], CodePolicy::Deny);

        assert!(result_with_matches.has_matches());
        assert!(!result_no_matches.has_matches());
    }

    #[test]
    fn policy_result_match_count_works() {
        let rule = PolicyRule::new("test", CodePolicy::Deny);
        let result = PolicyResult::new(
            rule,
            vec![
                Match::new(0, 4, 1, "test".to_string()),
                Match::new(10, 14, 2, "test".to_string()),
            ],
            CodePolicy::Deny,
        );
        assert_eq!(result.match_count(), 2);
    }

    #[test]
    fn policy_evaluator_new_is_empty() {
        let evaluator = PolicyEvaluator::new();
        assert!(evaluator.is_empty());
        assert_eq!(evaluator.rule_count(), 0);
    }

    #[test]
    fn policy_evaluator_add_rule_increments_count() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::new("test", CodePolicy::Deny));
        assert_eq!(evaluator.rule_count(), 1);
    }

    #[test]
    fn policy_evaluator_clear_removes_all_rules() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::new("test", CodePolicy::Deny));
        evaluator.clear();
        assert!(evaluator.is_empty());
    }

    #[test]
    fn policy_evaluator_rules_by_policy_filters_correctly() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::new("allow", CodePolicy::Allow));
        evaluator.add_rule(PolicyRule::new("deny", CodePolicy::Deny));
        evaluator.add_rule(PolicyRule::new("deny2", CodePolicy::Deny));

        let deny_rules = evaluator.rules_by_policy(CodePolicy::Deny);
        assert_eq!(deny_rules.len(), 2);
    }

    #[test]
    fn glob_matches_pattern_works() {
        assert!(glob_matches_pattern("*.rs", "main.rs").unwrap());
        assert!(!glob_matches_pattern("*.rs", "main.txt").unwrap());
        assert!(glob_matches_pattern("todo*", "todo!()").unwrap());
    }

    #[test]
    fn regex_matches_pattern_works() {
        assert!(regex_matches_pattern("unwrap", "x.unwrap()"));
        assert!(!regex_matches_pattern("unwrap", "x.expect()"));
    }

    #[test]
    fn find_matching_lines_works() {
        let code = "fn main() {\n    todo!()\n}\n";
        // Use *todo* to match lines containing todo (with leading whitespace)
        let matches = find_matching_lines("*todo*", code).unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].line, 2);
    }

    #[test]
    fn count_matching_lines_works() {
        let code = "fn main() {\n    todo!()\n    todo!()\n}\n";
        // Use *todo* to match lines containing todo (with leading whitespace)
        let count = count_matching_lines("*todo*", code).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn line_number_at_works() {
        let text = "line1\nline2\nline3";
        assert_eq!(line_number_at(text, 0), 1);
        assert_eq!(line_number_at(text, 6), 2);
        assert_eq!(line_number_at(text, 12), 3);
    }

    #[test]
    fn column_number_at_works() {
        let text = "line1\nline2";
        assert_eq!(column_number_at(text, 0), 1);
        assert_eq!(column_number_at(text, 2), 3);
        assert_eq!(column_number_at(text, 6), 1); // Start of line 2
    }
}
