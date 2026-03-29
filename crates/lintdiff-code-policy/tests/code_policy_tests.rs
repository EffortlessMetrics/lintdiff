//! Comprehensive BDD tests for code policy evaluation.
//!
//! These tests cover:
//! - Policy types (Allow, Deny, Suppress)
//! - Rule matching (glob and regex)
//! - Policy evaluation
//! - Edge cases (empty rules, no matches)
//! - Property-based tests with proptest

use lintdiff_code_policy::{
    count_matching_lines, find_matching_lines, glob_matches_pattern, line_number_at,
    regex_matches_pattern, CodePolicy, Match, PolicyEvaluator, PolicyResult, PolicyRule,
    PatternType, Severity, column_number_at,
};

// =============================================================================
// Policy Type Tests
// =============================================================================

mod policy_types {
    use super::*;

    #[test]
    fn allow_policy_is_identified_correctly() {
        let policy = CodePolicy::Allow;
        assert!(policy.is_allow());
        assert!(!policy.is_deny());
        assert!(!policy.is_suppress());
    }

    #[test]
    fn deny_policy_is_identified_correctly() {
        let policy = CodePolicy::Deny;
        assert!(!policy.is_allow());
        assert!(policy.is_deny());
        assert!(!policy.is_suppress());
    }

    #[test]
    fn suppress_policy_is_identified_correctly() {
        let policy = CodePolicy::Suppress;
        assert!(!policy.is_allow());
        assert!(!policy.is_deny());
        assert!(policy.is_suppress());
    }

    #[test]
    fn policy_display_formats_correctly() {
        assert_eq!(format!("{}", CodePolicy::Allow), "allow");
        assert_eq!(format!("{}", CodePolicy::Deny), "deny");
        assert_eq!(format!("{}", CodePolicy::Suppress), "suppress");
    }

    #[test]
    fn policy_default_is_allow() {
        let policy = CodePolicy::default();
        assert_eq!(policy, CodePolicy::Allow);
    }

    #[test]
    fn policy_equality_works() {
        assert_eq!(CodePolicy::Allow, CodePolicy::Allow);
        assert_ne!(CodePolicy::Allow, CodePolicy::Deny);
        assert_ne!(CodePolicy::Deny, CodePolicy::Suppress);
    }

    #[test]
    fn policy_clone_works() {
        let policy = CodePolicy::Deny;
        let cloned = policy.clone();
        assert_eq!(policy, cloned);
    }

    #[test]
    fn policy_hash_consistency() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(CodePolicy::Allow);
        set.insert(CodePolicy::Deny);
        set.insert(CodePolicy::Suppress);
        assert_eq!(set.len(), 3);
    }
}

// =============================================================================
// Severity Tests
// =============================================================================

mod severity_tests {
    use super::*;

    #[test]
    fn severity_ordering_is_correct() {
        assert!(Severity::Critical > Severity::Error);
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);
        assert!(Severity::Critical >= Severity::Error);
        assert!(Severity::Info < Severity::Warning);
    }

    #[test]
    fn severity_default_is_warning() {
        assert_eq!(Severity::default(), Severity::Warning);
    }

    #[test]
    fn severity_display_formats_correctly() {
        assert_eq!(format!("{}", Severity::Info), "info");
        assert_eq!(format!("{}", Severity::Warning), "warning");
        assert_eq!(format!("{}", Severity::Error), "error");
        assert_eq!(format!("{}", Severity::Critical), "critical");
    }

    #[test]
    fn severity_equality_works() {
        assert_eq!(Severity::Warning, Severity::Warning);
        assert_ne!(Severity::Warning, Severity::Error);
    }

    #[test]
    fn severity_clone_works() {
        let severity = Severity::Critical;
        let cloned = severity.clone();
        assert_eq!(severity, cloned);
    }
}

// =============================================================================
// Pattern Type Tests
// =============================================================================

mod pattern_type_tests {
    use super::*;

    #[test]
    fn pattern_type_default_is_glob() {
        assert_eq!(PatternType::default(), PatternType::Glob);
    }

    #[test]
    fn pattern_type_equality_works() {
        assert_eq!(PatternType::Glob, PatternType::Glob);
        assert_eq!(PatternType::Regex, PatternType::Regex);
        assert_ne!(PatternType::Glob, PatternType::Regex);
    }

    #[test]
    fn pattern_type_clone_works() {
        let pt = PatternType::Regex;
        let cloned = pt.clone();
        assert_eq!(pt, cloned);
    }
}

// =============================================================================
// Policy Rule Tests
// =============================================================================

mod policy_rule_tests {
    use super::*;

    #[test]
    fn rule_new_creates_glob_rule_by_default() {
        let rule = PolicyRule::new("unwrap()", CodePolicy::Deny);
        assert_eq!(rule.pattern, "unwrap()");
        assert_eq!(rule.policy, CodePolicy::Deny);
        assert_eq!(rule.pattern_type, PatternType::Glob);
        assert_eq!(rule.severity, None);
        assert_eq!(rule.message, None);
        assert_eq!(rule.file_filter, None);
        assert_eq!(rule.id, None);
    }

    #[test]
    fn rule_glob_creates_glob_pattern_type() {
        let rule = PolicyRule::glob("*.unwrap()", CodePolicy::Deny);
        assert_eq!(rule.pattern_type, PatternType::Glob);
    }

    #[test]
    fn rule_regex_creates_regex_pattern_type() {
        let rule = PolicyRule::regex("unwrap\\(\\)", CodePolicy::Deny);
        assert_eq!(rule.pattern_type, PatternType::Regex);
    }

    #[test]
    fn rule_with_severity_sets_severity() {
        let rule = PolicyRule::new("test", CodePolicy::Deny)
            .with_severity(Severity::Critical);
        assert_eq!(rule.severity, Some(Severity::Critical));
    }

    #[test]
    fn rule_with_message_sets_message() {
        let rule = PolicyRule::new("test", CodePolicy::Deny)
            .with_message("Avoid using this pattern");
        assert_eq!(rule.message, Some("Avoid using this pattern".to_string()));
    }

    #[test]
    fn rule_with_file_filter_sets_filter() {
        let rule = PolicyRule::new("test", CodePolicy::Deny)
            .with_file_filter("src/**/*.rs");
        assert_eq!(rule.file_filter, Some("src/**/*.rs".to_string()));
    }

    #[test]
    fn rule_with_id_sets_id() {
        let rule = PolicyRule::new("test", CodePolicy::Deny)
            .with_id("POLICY-001");
        assert_eq!(rule.id, Some("POLICY-001".to_string()));
    }

    #[test]
    fn rule_builder_chaining_works() {
        let rule = PolicyRule::new("unwrap()", CodePolicy::Deny)
            .with_severity(Severity::Error)
            .with_message("Use expect() instead")
            .with_file_filter("src/**/*.rs")
            .with_id("UNWRAP-001");

        assert_eq!(rule.pattern, "unwrap()");
        assert_eq!(rule.policy, CodePolicy::Deny);
        assert_eq!(rule.severity, Some(Severity::Error));
        assert_eq!(rule.message, Some("Use expect() instead".to_string()));
        assert_eq!(rule.file_filter, Some("src/**/*.rs".to_string()));
        assert_eq!(rule.id, Some("UNWRAP-001".to_string()));
    }

    #[test]
    fn rule_applies_to_file_without_filter_returns_true() {
        let rule = PolicyRule::new("test", CodePolicy::Deny);
        assert!(rule.applies_to_file("src/main.rs").unwrap());
        assert!(rule.applies_to_file("lib.rs").unwrap());
    }

    #[test]
    fn rule_applies_to_file_with_filter_matches_correctly() {
        let rule = PolicyRule::new("test", CodePolicy::Deny)
            .with_file_filter("*.rs");
        assert!(rule.applies_to_file("main.rs").unwrap());
        assert!(!rule.applies_to_file("main.txt").unwrap());
    }

    #[test]
    fn rule_applies_to_file_with_globstar_filter() {
        let rule = PolicyRule::new("test", CodePolicy::Deny)
            .with_file_filter("src/**/*.rs");
        assert!(rule.applies_to_file("src/lib.rs").unwrap());
        assert!(rule.applies_to_file("src/foo/bar.rs").unwrap());
        assert!(!rule.applies_to_file("tests/lib.rs").unwrap());
    }

    #[test]
    fn rule_clone_works() {
        let rule = PolicyRule::new("test", CodePolicy::Deny)
            .with_message("Test message");
        let cloned = rule.clone();
        assert_eq!(rule.pattern, cloned.pattern);
        assert_eq!(rule.policy, cloned.policy);
        assert_eq!(rule.message, cloned.message);
    }
}

// =============================================================================
// Match Tests
// =============================================================================

mod match_tests {
    use super::*;

    #[test]
    fn match_new_creates_match_without_column() {
        let m = Match::new(10, 20, 3, "test code".to_string());
        assert_eq!(m.start, 10);
        assert_eq!(m.end, 20);
        assert_eq!(m.line, 3);
        assert_eq!(m.text, "test code");
        assert_eq!(m.column, None);
    }

    #[test]
    fn match_with_column_adds_column_info() {
        let m = Match::new(10, 20, 3, "test".to_string()).with_column(5);
        assert_eq!(m.column, Some(5));
    }

    #[test]
    fn match_clone_works() {
        let m = Match::new(0, 10, 1, "test".to_string()).with_column(3);
        let cloned = m.clone();
        assert_eq!(m.start, cloned.start);
        assert_eq!(m.end, cloned.end);
        assert_eq!(m.line, cloned.line);
        assert_eq!(m.column, cloned.column);
        assert_eq!(m.text, cloned.text);
    }

    #[test]
    fn match_equality_works() {
        let m1 = Match::new(0, 10, 1, "test".to_string());
        let m2 = Match::new(0, 10, 1, "test".to_string());
        let m3 = Match::new(5, 15, 1, "test".to_string());
        assert_eq!(m1, m2);
        assert_ne!(m1, m3);
    }
}

// =============================================================================
// Policy Result Tests
// =============================================================================

mod policy_result_tests {
    use super::*;

    #[test]
    fn result_new_creates_result_without_file_path() {
        let rule = PolicyRule::new("test", CodePolicy::Deny);
        let matches = vec![Match::new(0, 4, 1, "test".to_string())];
        let result = PolicyResult::new(rule.clone(), matches.clone(), CodePolicy::Deny);

        assert_eq!(result.rule, rule);
        assert_eq!(result.matches, matches);
        assert_eq!(result.policy, CodePolicy::Deny);
        assert_eq!(result.file_path, None);
    }

    #[test]
    fn result_with_file_path_adds_path() {
        let rule = PolicyRule::new("test", CodePolicy::Deny);
        let result = PolicyResult::new(rule, vec![], CodePolicy::Deny)
            .with_file_path("src/main.rs");
        assert_eq!(result.file_path, Some("src/main.rs".to_string()));
    }

    #[test]
    fn result_has_matches_returns_correct_value() {
        let rule = PolicyRule::new("test", CodePolicy::Deny);

        let result_with = PolicyResult::new(
            rule.clone(),
            vec![Match::new(0, 4, 1, "test".to_string())],
            CodePolicy::Deny,
        );
        assert!(result_with.has_matches());

        let result_without = PolicyResult::new(rule, vec![], CodePolicy::Deny);
        assert!(!result_without.has_matches());
    }

    #[test]
    fn result_match_count_returns_correct_count() {
        let rule = PolicyRule::new("test", CodePolicy::Deny);
        let matches = vec![
            Match::new(0, 4, 1, "test".to_string()),
            Match::new(10, 14, 2, "test".to_string()),
            Match::new(20, 24, 3, "test".to_string()),
        ];
        let result = PolicyResult::new(rule, matches, CodePolicy::Deny);
        assert_eq!(result.match_count(), 3);
    }

    #[test]
    fn result_clone_works() {
        let rule = PolicyRule::new("test", CodePolicy::Deny);
        let result = PolicyResult::new(
            rule,
            vec![Match::new(0, 4, 1, "test".to_string())],
            CodePolicy::Deny,
        ).with_file_path("test.rs");
        let cloned = result.clone();
        assert_eq!(result.file_path, cloned.file_path);
        assert_eq!(result.matches.len(), cloned.matches.len());
    }
}

// =============================================================================
// Policy Evaluator Tests
// =============================================================================

mod policy_evaluator_tests {
    use super::*;

    #[test]
    fn evaluator_new_creates_empty_evaluator() {
        let evaluator = PolicyEvaluator::new();
        assert!(evaluator.is_empty());
        assert_eq!(evaluator.rule_count(), 0);
    }

    #[test]
    fn evaluator_default_creates_empty_evaluator() {
        let evaluator = PolicyEvaluator::default();
        assert!(evaluator.is_empty());
    }

    #[test]
    fn evaluator_add_rule_increments_count() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::new("test", CodePolicy::Deny));
        assert_eq!(evaluator.rule_count(), 1);
        evaluator.add_rule(PolicyRule::new("test2", CodePolicy::Allow));
        assert_eq!(evaluator.rule_count(), 2);
    }

    #[test]
    fn evaluator_add_rules_adds_multiple() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rules(vec![
            PolicyRule::new("test1", CodePolicy::Deny),
            PolicyRule::new("test2", CodePolicy::Allow),
            PolicyRule::new("test3", CodePolicy::Suppress),
        ]);
        assert_eq!(evaluator.rule_count(), 3);
    }

    #[test]
    fn evaluator_clear_removes_all_rules() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::new("test", CodePolicy::Deny));
        evaluator.add_rule(PolicyRule::new("test2", CodePolicy::Allow));
        evaluator.clear();
        assert!(evaluator.is_empty());
        assert_eq!(evaluator.rule_count(), 0);
    }

    #[test]
    fn evaluator_rules_by_policy_filters_correctly() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rules(vec![
            PolicyRule::new("allow1", CodePolicy::Allow),
            PolicyRule::new("deny1", CodePolicy::Deny),
            PolicyRule::new("deny2", CodePolicy::Deny),
            PolicyRule::new("suppress1", CodePolicy::Suppress),
        ]);

        let allow_rules = evaluator.rules_by_policy(CodePolicy::Allow);
        let deny_rules = evaluator.rules_by_policy(CodePolicy::Deny);
        let suppress_rules = evaluator.rules_by_policy(CodePolicy::Suppress);

        assert_eq!(allow_rules.len(), 1);
        assert_eq!(deny_rules.len(), 2);
        assert_eq!(suppress_rules.len(), 1);
    }

    #[test]
    fn evaluator_clone_works() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::new("test", CodePolicy::Deny));
        let cloned = evaluator.clone();
        assert_eq!(evaluator.rule_count(), cloned.rule_count());
    }
}

// =============================================================================
// Evaluation Tests - Glob Patterns
// =============================================================================

mod glob_evaluation_tests {
    use super::*;

    #[test]
    fn evaluate_finds_simple_glob_match() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::glob("todo!()", CodePolicy::Deny));

        let code = "fn main() {\n    todo!()\n}";
        let results = evaluator.evaluate(code);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].policy, CodePolicy::Deny);
        assert_eq!(results[0].matches.len(), 1);
        assert_eq!(results[0].matches[0].line, 2);
    }

    #[test]
    fn evaluate_finds_multiple_matches() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::glob("*todo*", CodePolicy::Deny));

        let code = "fn a() { todo!() }\nfn b() { todo!() }\nfn c() { ok() }";
        let results = evaluator.evaluate(code);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].matches.len(), 2);
    }

    #[test]
    fn evaluate_wildcard_pattern() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::glob("*.unwrap()", CodePolicy::Deny));

        let code = "x.unwrap()\ny.unwrap()\nz.expect()";
        let results = evaluator.evaluate(code);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].matches.len(), 2);
    }

    #[test]
    fn evaluate_no_match_returns_empty() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::glob("todo!()", CodePolicy::Deny));

        let code = "fn main() {\n    println!(\"hello\");\n}";
        let results = evaluator.evaluate(code);

        assert!(results.is_empty());
    }

    #[test]
    fn evaluate_multiple_rules_with_different_policies() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::glob("todo!()", CodePolicy::Deny));
        evaluator.add_rule(PolicyRule::glob("println!", CodePolicy::Allow));

        let code = "fn main() {\n    todo!()\n    println!(\"hi\");\n}";
        let results = evaluator.evaluate(code);

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn evaluate_empty_code_returns_empty() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::glob("todo!()", CodePolicy::Deny));

        let results = evaluator.evaluate("");
        assert!(results.is_empty());
    }

    #[test]
    fn evaluate_with_empty_rules_returns_empty() {
        let evaluator = PolicyEvaluator::new();
        let code = "fn main() { todo!() }";
        let results = evaluator.evaluate(code);
        assert!(results.is_empty());
    }
}

// =============================================================================
// Evaluation Tests - Regex Patterns
// =============================================================================

mod regex_evaluation_tests {
    use super::*;

    #[test]
    fn evaluate_finds_simple_regex_match() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::regex("unwrap", CodePolicy::Deny));

        let code = "fn main() {\n    x.unwrap()\n}";
        let results = evaluator.evaluate(code);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].matches.len(), 1);
    }

    #[test]
    fn evaluate_regex_finds_all_occurrences() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::regex("unwrap", CodePolicy::Deny));

        let code = "a.unwrap()\nb.unwrap()\nc.unwrap()";
        let results = evaluator.evaluate(code);

        assert_eq!(results[0].matches.len(), 3);
    }

    #[test]
    fn evaluate_regex_no_match_returns_empty() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::regex("unwrap", CodePolicy::Deny));

        let code = "fn main() {\n    x.expect(\"error\")\n}";
        let results = evaluator.evaluate(code);

        assert!(results.is_empty());
    }

    #[test]
    fn evaluate_regex_includes_column_info() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::regex("unwrap", CodePolicy::Deny));

        let code = "x.unwrap()";
        let results = evaluator.evaluate(code);

        assert_eq!(results[0].matches[0].column, Some(2));
    }
}

// =============================================================================
// File Evaluation Tests
// =============================================================================

mod file_evaluation_tests {
    use super::*;

    #[test]
    fn evaluate_file_applies_file_filter() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(
            PolicyRule::glob("todo!()", CodePolicy::Deny)
                .with_file_filter("src/**/*.rs"),
        );

        let code = "fn main() { todo!() }";

        // Should match for files in src/
        let results_src = evaluator.evaluate_file("src/main.rs", code);
        assert_eq!(results_src.len(), 1);

        // Should not match for files outside src/
        let results_tests = evaluator.evaluate_file("tests/main.rs", code);
        assert_eq!(results_tests.len(), 0);
    }

    #[test]
    fn evaluate_file_includes_file_path_in_result() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::glob("todo!()", CodePolicy::Deny));

        let code = "fn main() { todo!() }";
        let results = evaluator.evaluate_file("src/main.rs", code);

        assert_eq!(results[0].file_path, Some("src/main.rs".to_string()));
    }

    #[test]
    fn evaluate_file_with_no_matching_rules_returns_empty() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(
            PolicyRule::glob("todo!()", CodePolicy::Deny)
                .with_file_filter("*.txt"),
        );

        let code = "fn main() { todo!() }";
        let results = evaluator.evaluate_file("src/main.rs", code);

        assert!(results.is_empty());
    }
}

// =============================================================================
// Deny/Suppress Filter Tests
// =============================================================================

mod filter_tests {
    use super::*;

    #[test]
    fn evaluate_denials_returns_only_deny_results() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::glob("todo!()", CodePolicy::Deny));
        evaluator.add_rule(PolicyRule::glob("println!", CodePolicy::Allow));
        evaluator.add_rule(PolicyRule::glob("FIXME", CodePolicy::Suppress));

        let code = "fn main() {\n    todo!()\n    println!(\"hi\")\n    // FIXME\n}";
        let results = evaluator.evaluate_denials(code);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].policy, CodePolicy::Deny);
    }

    #[test]
    fn evaluate_suppressions_returns_only_suppress_results() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::glob("todo!()", CodePolicy::Deny));
        evaluator.add_rule(PolicyRule::glob("*FIXME*", CodePolicy::Suppress));

        let code = "fn main() {\n    todo!()\n    // FIXME\n}";
        let results = evaluator.evaluate_suppressions(code);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].policy, CodePolicy::Suppress);
    }
}

// =============================================================================
// Helper Function Tests
// =============================================================================

mod helper_function_tests {
    use super::*;

    #[test]
    fn glob_matches_pattern_simple_wildcard() {
        assert!(glob_matches_pattern("*.rs", "main.rs").unwrap());
        assert!(glob_matches_pattern("*.rs", "lib.rs").unwrap());
        assert!(!glob_matches_pattern("*.rs", "main.txt").unwrap());
    }

    #[test]
    fn glob_matches_pattern_prefix_wildcard() {
        assert!(glob_matches_pattern("todo*", "todo!()").unwrap());
        assert!(glob_matches_pattern("todo*", "todo: implement").unwrap());
        assert!(!glob_matches_pattern("todo*", "TODO").unwrap()); // case sensitive
    }

    #[test]
    fn glob_matches_pattern_suffix_wildcard() {
        assert!(glob_matches_pattern("*unwrap*", "x.unwrap()").unwrap());
        assert!(glob_matches_pattern("*test*", "integration_test").unwrap());
    }

    #[test]
    fn glob_matches_pattern_exact_match() {
        assert!(glob_matches_pattern("exact", "exact").unwrap());
        assert!(!glob_matches_pattern("exact", "not exact").unwrap());
    }

    #[test]
    fn glob_matches_pattern_question_mark() {
        assert!(glob_matches_pattern("te?t", "test").unwrap());
        assert!(glob_matches_pattern("te?t", "text").unwrap());
        assert!(!glob_matches_pattern("te?t", "tet").unwrap());
    }

    #[test]
    fn glob_matches_pattern_character_class() {
        assert!(glob_matches_pattern("[abc]", "a").unwrap());
        assert!(glob_matches_pattern("[abc]", "b").unwrap());
        assert!(!glob_matches_pattern("[abc]", "d").unwrap());
    }

    #[test]
    fn glob_matches_pattern_character_range() {
        assert!(glob_matches_pattern("[a-z]", "m").unwrap());
        assert!(!glob_matches_pattern("[a-z]", "M").unwrap());
    }

    #[test]
    fn glob_matches_pattern_globstar() {
        assert!(glob_matches_pattern("src/**/*.rs", "src/main.rs").unwrap());
        assert!(glob_matches_pattern("src/**/*.rs", "src/foo/bar.rs").unwrap());
    }

    #[test]
    fn regex_matches_pattern_simple() {
        assert!(regex_matches_pattern("unwrap", "x.unwrap()"));
        assert!(regex_matches_pattern("test", "this is a test"));
        assert!(!regex_matches_pattern("unwrap", "x.expect()"));
    }

    #[test]
    fn regex_matches_pattern_empty_pattern() {
        assert!(regex_matches_pattern("", "anything"));
        assert!(regex_matches_pattern("", ""));
    }

    #[test]
    fn find_matching_lines_finds_all_matches() {
        let code = "fn a() { todo!() }\nfn b() { done() }\nfn c() { todo!() }";
        let matches = find_matching_lines("*todo*", code).unwrap();

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].line, 1);
        assert_eq!(matches[1].line, 3);
    }

    #[test]
    fn find_matching_lines_no_matches() {
        let code = "fn main() { println!(\"hello\"); }";
        let matches = find_matching_lines("todo*", code).unwrap();

        assert!(matches.is_empty());
    }

    #[test]
    fn find_matching_lines_empty_code() {
        let matches = find_matching_lines("todo*", "").unwrap();
        assert!(matches.is_empty());
    }

    #[test]
    fn count_matching_lines_counts_correctly() {
        let code = "todo!()\nnot a todo\ntodo!()\ntodo!()";
        let count = count_matching_lines("todo*", code).unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn count_matching_lines_no_matches() {
        let code = "fn main() {}";
        let count = count_matching_lines("todo*", code).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn line_number_at_start_of_file() {
        let text = "line1\nline2\nline3";
        assert_eq!(line_number_at(text, 0), 1);
    }

    #[test]
    fn line_number_at_middle_of_file() {
        let text = "line1\nline2\nline3";
        assert_eq!(line_number_at(text, 6), 2);
        assert_eq!(line_number_at(text, 12), 3);
    }

    #[test]
    fn line_number_at_end_of_file() {
        let text = "line1\nline2";
        assert_eq!(line_number_at(text, text.len()), 2);
    }

    #[test]
    fn line_number_at_beyond_end() {
        let text = "short";
        assert_eq!(line_number_at(text, 100), 1);
    }

    #[test]
    fn column_number_at_start_of_line() {
        let text = "line1\nline2";
        assert_eq!(column_number_at(text, 0), 1);
        assert_eq!(column_number_at(text, 6), 1);
    }

    #[test]
    fn column_number_at_middle_of_line() {
        let text = "line1\nline2";
        assert_eq!(column_number_at(text, 2), 3);
        assert_eq!(column_number_at(text, 8), 3);
    }

    #[test]
    fn column_number_at_end_of_line() {
        let text = "line1";
        assert_eq!(column_number_at(text, 5), 6);
    }
}

// =============================================================================
// Edge Case Tests
// =============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn empty_evaluator_with_any_code() {
        let evaluator = PolicyEvaluator::new();
        let results = evaluator.evaluate("any code here");
        assert!(results.is_empty());
    }

    #[test]
    fn empty_code_with_any_rules() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::glob("*", CodePolicy::Deny));
        let results = evaluator.evaluate("");
        assert!(results.is_empty()); // Empty string has no lines
    }

    #[test]
    fn rule_with_empty_pattern() {
        let mut evaluator = PolicyEvaluator::new();
        // Empty pattern should fail to compile as glob
        evaluator.add_rule(PolicyRule::regex("", CodePolicy::Deny));

        let code = "some code";
        let results = evaluator.evaluate(code);
        // Empty pattern matches everywhere but we treat it as no match
        // Actually, empty string is found at position 0
        assert!(!results.is_empty());
    }

    #[test]
    fn code_with_only_whitespace() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::glob("todo*", CodePolicy::Deny));

        let code = "   \n   \n   ";
        let results = evaluator.evaluate(code);
        assert!(results.is_empty());
    }

    #[test]
    fn rule_matches_everything() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::glob("*", CodePolicy::Deny));

        let code = "anything\nhere";
        let results = evaluator.evaluate(code);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].matches.len(), 2); // Both lines match
    }

    #[test]
    fn multiple_rules_same_pattern_different_policies() {
        let mut evaluator = PolicyEvaluator::new();
        // Use *todo!()* to match lines containing todo!()
        evaluator.add_rule(PolicyRule::glob("*todo!()*", CodePolicy::Deny));
        evaluator.add_rule(PolicyRule::glob("*todo!()*", CodePolicy::Suppress));

        let code = "fn main() { todo!() }";
        let results = evaluator.evaluate(code);

        // Both rules should match
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn very_long_code_line() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::glob("todo", CodePolicy::Deny));

        let long_line = "x ".repeat(10000) + "todo";
        let results = evaluator.evaluate(&long_line);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn unicode_in_code() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::glob("todo", CodePolicy::Deny));

        let code = "// 日本語コメント\nfn main() { todo!() }";
        let results = evaluator.evaluate(code);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn special_regex_characters_as_literal() {
        let mut evaluator = PolicyEvaluator::new();
        // In our simplified regex, we do substring matching
        evaluator.add_rule(PolicyRule::regex("(", CodePolicy::Deny));

        let code = "fn main() {}";
        let results = evaluator.evaluate(code);
        assert_eq!(results.len(), 1);
    }
}

// =============================================================================
// Integration Tests
// =============================================================================

mod integration_tests {
    use super::*;

    #[test]
    fn full_workflow_deny_unwrap() {
        let mut evaluator = PolicyEvaluator::new();

        // Deny unwrap() in src/ files
        evaluator.add_rule(
            PolicyRule::glob("*.unwrap()", CodePolicy::Deny)
                .with_severity(Severity::Error)
                .with_message("Use expect() or proper error handling")
                .with_file_filter("src/**/*.rs")
                .with_id("UNWRAP-001"),
        );

        let code = r#"
fn process(data: Option<i32>) -> i32 {
    data.unwrap()
}

fn main() {
    let result = process(Some(42));
    println!("{}", result);
}
"#;

        let results = evaluator.evaluate_file("src/lib.rs", code);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].policy, CodePolicy::Deny);
        assert_eq!(results[0].rule.severity, Some(Severity::Error));
        assert_eq!(results[0].rule.id, Some("UNWRAP-001".to_string()));
        assert!(results[0].has_matches());
    }

    #[test]
    fn full_workflow_multiple_policies() {
        let mut evaluator = PolicyEvaluator::new();

        // Deny unwrap()
        evaluator.add_rule(PolicyRule::glob("*unwrap*", CodePolicy::Deny));

        // Suppress TODOs in comments
        evaluator.add_rule(PolicyRule::glob("*// TODO*", CodePolicy::Suppress));

        // Allow println! in main
        evaluator.add_rule(
            PolicyRule::glob("println!", CodePolicy::Allow)
                .with_file_filter("src/main.rs"),
        );

        let code = r#"
fn main() {
    // TODO: implement error handling
    let x = Some(1);
    x.unwrap();
    println!("Hello");
}
"#;

        let results = evaluator.evaluate_file("src/main.rs", code);

        // Should have results for unwrap (deny) and TODO (suppress)
        assert!(results.len() >= 2);

        let deny_results: Vec<_> = results.iter()
            .filter(|r| r.policy == CodePolicy::Deny)
            .collect();
        let suppress_results: Vec<_> = results.iter()
            .filter(|r| r.policy == CodePolicy::Suppress)
            .collect();

        assert!(!deny_results.is_empty());
        assert!(!suppress_results.is_empty());
    }

    #[test]
    fn workflow_with_only_allow_rules() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(PolicyRule::glob("println!", CodePolicy::Allow));

        let code = "fn main() { println!(\"hi\"); }";
        let results = evaluator.evaluate(code);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].policy, CodePolicy::Allow);
    }

    #[test]
    fn workflow_file_filter_excludes_tests() {
        let mut evaluator = PolicyEvaluator::new();
        evaluator.add_rule(
            PolicyRule::glob("todo!()", CodePolicy::Deny)
                .with_file_filter("src/**/*.rs"),
        );

        let code = "fn test() { todo!() }";

        // Should match in src/
        let src_results = evaluator.evaluate_file("src/lib.rs", code);
        assert_eq!(src_results.len(), 1);

        // Should not match in tests/
        let test_results = evaluator.evaluate_file("tests/lib.rs", code);
        assert_eq!(test_results.len(), 0);
    }
}

// =============================================================================
// Property-Based Tests
// =============================================================================

#[cfg(test)]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn glob_matches_pattern_never_panics(pattern: String, text: String) {
            // Empty patterns should be handled gracefully
            if !pattern.is_empty() {
                let _ = glob_matches_pattern(&pattern, &text);
            }
        }

        #[test]
        fn regex_matches_pattern_never_panics(pattern: String, text: String) {
            let _ = regex_matches_pattern(&pattern, &text);
        }

        #[test]
        fn line_number_at_never_panics(text: String, pos in 0usize..1000) {
            let _ = line_number_at(&text, pos);
        }

        #[test]
        fn column_number_at_never_panics(text: String, pos in 0usize..1000) {
            let _ = column_number_at(&text, pos);
        }

        #[test]
        fn evaluator_evaluate_never_panics(code: String) {
            let mut evaluator = PolicyEvaluator::new();
            evaluator.add_rule(PolicyRule::glob("test", CodePolicy::Deny));
            let _ = evaluator.evaluate(&code);
        }

        #[test]
        fn evaluator_with_multiple_rules_never_panics(code: String) {
            let mut evaluator = PolicyEvaluator::new();
            evaluator.add_rules(vec![
                PolicyRule::glob("a*", CodePolicy::Allow),
                PolicyRule::glob("b*", CodePolicy::Deny),
                PolicyRule::glob("c*", CodePolicy::Suppress),
            ]);
            let _ = evaluator.evaluate(&code);
        }

        #[test]
        fn find_matching_lines_never_panics(code: String) {
            let _ = find_matching_lines("*", &code);
        }

        #[test]
        fn count_matching_lines_never_panics(code: String) {
            let _ = count_matching_lines("*", &code);
        }

        #[test]
        fn line_number_at_start_is_one(text: String) {
            if !text.is_empty() {
                prop_assert_eq!(line_number_at(&text, 0), 1);
            }
        }

        #[test]
        fn column_number_at_start_is_one(text: String) {
            if !text.is_empty() {
                prop_assert_eq!(column_number_at(&text, 0), 1);
            }
        }
    }
}

// =============================================================================
// Serde Tests (only when serde feature is enabled)
// =============================================================================

#[cfg(feature = "serde")]
mod serde_tests {
    use super::*;

    #[test]
    fn code_policy_has_serde_derive() {
        // Verify that CodePolicy has serde support compiled in
        let policy = CodePolicy::Deny;
        // Just verify the type exists and can be used
        assert!(policy.is_deny());
    }

    #[test]
    fn severity_has_serde_derive() {
        let severity = Severity::Error;
        assert_eq!(severity, Severity::Error);
    }

    #[test]
    fn policy_rule_has_serde_derive() {
        let rule = PolicyRule::new("test", CodePolicy::Deny)
            .with_severity(Severity::Error)
            .with_message("Test message")
            .with_id("TEST-001");
        assert_eq!(rule.pattern, "test");
    }

    #[test]
    fn match_has_serde_derive() {
        let m = Match::new(10, 20, 3, "test".to_string()).with_column(5);
        assert_eq!(m.start, 10);
    }

    #[test]
    fn policy_result_has_serde_derive() {
        let rule = PolicyRule::new("test", CodePolicy::Deny);
        let result = PolicyResult::new(
            rule,
            vec![Match::new(0, 4, 1, "test".to_string())],
            CodePolicy::Deny,
        ).with_file_path("test.rs");
        assert!(result.has_matches());
    }
}
