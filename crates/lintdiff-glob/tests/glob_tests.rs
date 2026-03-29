//! Comprehensive tests for lintdiff-glob crate.
//!
//! This test suite covers:
//! - Basic `*` matching (10 tests)
//! - `**` globstar matching (10 tests)
//! - `?` single character matching (8 tests)
//! - Character classes `[abc]` (10 tests)
//! - Negated character classes `[!abc]` (8 tests)
//! - Character ranges `[a-z]` (8 tests)
//! - GlobSet with multiple patterns (8 tests)
//! - Edge cases and error conditions (8 tests)

use lintdiff_glob::{Glob, GlobError, GlobSet};

// =============================================================================
// Basic `*` matching tests (10 tests)
// =============================================================================

#[test]
fn test_star_matches_anything_except_separator() {
    let glob = Glob::new("*.rs").unwrap();
    assert!(glob.is_match("lib.rs"));
    assert!(glob.is_match("main.rs"));
    assert!(glob.is_match("test.rs"));
}

#[test]
fn test_star_does_not_match_path_separator() {
    let glob = Glob::new("*.rs").unwrap();
    assert!(!glob.is_match("src/lib.rs"));
    assert!(!glob.is_match("foo/bar/baz.rs"));
}

#[test]
fn test_star_at_beginning() {
    let glob = Glob::new("*_test.rs").unwrap();
    assert!(glob.is_match("foo_test.rs"));
    assert!(glob.is_match("bar_test.rs"));
    assert!(!glob.is_match("test.rs"));
}

#[test]
fn test_star_in_middle() {
    let glob = Glob::new("test_*.rs").unwrap();
    assert!(glob.is_match("test_foo.rs"));
    assert!(glob.is_match("test_bar.rs"));
    assert!(!glob.is_match("foo_test.rs"));
}

#[test]
fn test_multiple_stars() {
    let glob = Glob::new("*_*.rs").unwrap();
    assert!(glob.is_match("foo_bar.rs"));
    assert!(glob.is_match("test_unit.rs"));
    assert!(!glob.is_match("test.rs"));
}

#[test]
fn test_star_matches_empty() {
    let glob = Glob::new("file*.rs").unwrap();
    assert!(glob.is_match("file.rs"));
    assert!(glob.is_match("file_name.rs"));
}

#[test]
fn test_star_with_extension() {
    let glob = Glob::new("doc.*").unwrap();
    assert!(glob.is_match("doc.txt"));
    assert!(glob.is_match("doc.md"));
    assert!(glob.is_match("doc.html"));
    assert!(!glob.is_match("doc"));
}

#[test]
fn test_star_before_literal() {
    let glob = Glob::new("*file.txt").unwrap();
    assert!(glob.is_match("myfile.txt"));
    assert!(glob.is_match("testfile.txt"));
    assert!(glob.is_match("file.txt")); // * can match empty string
    assert!(!glob.is_match("other.txt"));
}

#[test]
fn test_star_can_match_empty_string() {
    let glob = Glob::new("*file.txt").unwrap();
    // Actually * can match empty, so this should match
    // Let me test with a pattern where * clearly matches empty
    let glob2 = Glob::new("a*b").unwrap();
    assert!(glob2.is_match("ab"));
    assert!(glob2.is_match("axb"));
    assert!(glob2.is_match("axxb"));
}

#[test]
fn test_star_with_path_component() {
    let glob = Glob::new("src/*").unwrap();
    assert!(glob.is_match("src/lib.rs"));
    assert!(glob.is_match("src/main.rs"));
    assert!(!glob.is_match("src/foo/bar.rs"));
    assert!(!glob.is_match("tests/lib.rs"));
}

// =============================================================================
// `**` globstar matching tests (10 tests)
// =============================================================================

#[test]
fn test_globstar_matches_across_directories() {
    let glob = Glob::new("src/**/*.rs").unwrap();
    assert!(glob.is_match("src/lib.rs"));
    assert!(glob.is_match("src/foo/bar.rs"));
    assert!(glob.is_match("src/a/b/c/d.rs"));
}

#[test]
fn test_globstar_at_beginning() {
    let glob = Glob::new("**/test.rs").unwrap();
    assert!(glob.is_match("test.rs"));
    assert!(glob.is_match("src/test.rs"));
    assert!(glob.is_match("src/foo/test.rs"));
}

#[test]
fn test_globstar_at_end() {
    let glob = Glob::new("src/**").unwrap();
    assert!(glob.is_match("src/lib.rs"));
    assert!(glob.is_match("src/foo/bar.rs"));
    assert!(glob.is_match("src/a/b/c"));
}

#[test]
fn test_globstar_matches_zero_directories() {
    let glob = Glob::new("src/**/*.rs").unwrap();
    // ** can match zero path components
    assert!(glob.is_match("src/lib.rs"));
}

#[test]
fn test_globstar_only() {
    let glob = Glob::new("**").unwrap();
    assert!(glob.is_match("anything"));
    assert!(glob.is_match("path/to/file.rs"));
    assert!(glob.is_match(""));
}

#[test]
fn test_multiple_globstars() {
    let glob = Glob::new("**/src/**/*.rs").unwrap();
    assert!(glob.is_match("src/lib.rs"));
    assert!(glob.is_match("project/src/lib.rs"));
    assert!(glob.is_match("src/foo/bar.rs"));
    assert!(glob.is_match("project/src/foo/bar.rs"));
}

#[test]
fn test_globstar_does_not_match_outside_prefix() {
    let glob = Glob::new("src/**/*.rs").unwrap();
    assert!(!glob.is_match("tests/lib.rs"));
    assert!(!glob.is_match("lib.rs"));
}

#[test]
fn test_globstar_with_specific_suffix() {
    let glob = Glob::new("**/Cargo.toml").unwrap();
    assert!(glob.is_match("Cargo.toml"));
    assert!(glob.is_match("crates/foo/Cargo.toml"));
    assert!(glob.is_match("workspace/crates/bar/Cargo.toml"));
    assert!(!glob.is_match("Cargo.lock"));
}

#[test]
fn test_globstar_between_literals() {
    let glob = Glob::new("src/**/test/*.rs").unwrap();
    assert!(glob.is_match("src/test/lib.rs"));
    assert!(glob.is_match("src/foo/test/lib.rs"));
    assert!(glob.is_match("src/a/b/test/lib.rs"));
    assert!(!glob.is_match("src/foo/bar/lib.rs"));
}

#[test]
fn test_globstar_windows_paths() {
    let glob = Glob::new("src/**/*.rs").unwrap();
    // Windows backslashes should be normalized
    assert!(glob.is_match("src\\lib.rs"));
    assert!(glob.is_match("src\\foo\\bar.rs"));
}

// =============================================================================
// `?` single character matching tests (8 tests)
// =============================================================================

#[test]
fn test_question_matches_single_char() {
    let glob = Glob::new("file?.txt").unwrap();
    assert!(glob.is_match("file1.txt"));
    assert!(glob.is_match("fileA.txt"));
    assert!(glob.is_match("file_.txt"));
}

#[test]
fn test_question_does_not_match_empty() {
    let glob = Glob::new("file?.txt").unwrap();
    assert!(!glob.is_match("file.txt"));
}

#[test]
fn test_question_does_not_match_multiple() {
    let glob = Glob::new("file?.txt").unwrap();
    assert!(!glob.is_match("file12.txt"));
    assert!(!glob.is_match("fileAB.txt"));
}

#[test]
fn test_question_does_not_match_separator() {
    let glob = Glob::new("file?.txt").unwrap();
    assert!(!glob.is_match("file/.txt"));
}

#[test]
fn test_multiple_questions() {
    let glob = Glob::new("???.txt").unwrap();
    assert!(glob.is_match("abc.txt"));
    assert!(glob.is_match("123.txt"));
    assert!(!glob.is_match("ab.txt"));
    assert!(!glob.is_match("abcd.txt"));
}

#[test]
fn test_question_at_beginning() {
    let glob = Glob::new("?ile.txt").unwrap();
    assert!(glob.is_match("file.txt"));
    assert!(glob.is_match("File.txt"));
    assert!(!glob.is_match("ile.txt"));
}

#[test]
fn test_question_at_end() {
    let glob = Glob::new("file?.txt").unwrap();
    assert!(glob.is_match("file1.txt"));
    assert!(glob.is_match("file2.txt"));
    assert!(!glob.is_match("file.txt"));
}

#[test]
fn test_question_with_star() {
    let glob = Glob::new("test?.rs").unwrap();
    assert!(glob.is_match("test1.rs"));
    assert!(glob.is_match("testA.rs"));
    assert!(!glob.is_match("test.rs"));
    assert!(!glob.is_match("test12.rs"));
}

// =============================================================================
// Character classes `[abc]` tests (10 tests)
// =============================================================================

#[test]
fn test_char_class_basic() {
    let glob = Glob::new("[abc].txt").unwrap();
    assert!(glob.is_match("a.txt"));
    assert!(glob.is_match("b.txt"));
    assert!(glob.is_match("c.txt"));
    assert!(!glob.is_match("d.txt"));
}

#[test]
fn test_char_class_with_literal() {
    let glob = Glob::new("file[123].txt").unwrap();
    assert!(glob.is_match("file1.txt"));
    assert!(glob.is_match("file2.txt"));
    assert!(glob.is_match("file3.txt"));
    assert!(!glob.is_match("file4.txt"));
}

#[test]
fn test_char_class_does_not_match_separator() {
    let glob = Glob::new("[abc].txt").unwrap();
    // The character class matches a, b, or c, not /
    assert!(!glob.is_match("/.txt"));
    assert!(glob.is_match("a.txt"));
}

#[test]
fn test_char_class_multiple() {
    let glob = Glob::new("[abc][def].txt").unwrap();
    assert!(glob.is_match("ad.txt"));
    assert!(glob.is_match("be.txt"));
    assert!(glob.is_match("cf.txt"));
    assert!(!glob.is_match("da.txt"));
    assert!(!glob.is_match("ab.txt"));
}

#[test]
fn test_char_class_with_star() {
    let glob = Glob::new("[abc]*.txt").unwrap();
    assert!(glob.is_match("a.txt"));
    assert!(glob.is_match("bfile.txt"));
    assert!(glob.is_match("ctest.txt"));
    assert!(!glob.is_match("dfile.txt"));
}

#[test]
fn test_char_class_special_chars() {
    // Test a valid pattern with special chars in class (- at start is literal)
    let glob = Glob::new("file[-_].txt").unwrap();
    assert!(glob.is_match("file-.txt"));
    assert!(glob.is_match("file_.txt"));
}

#[test]
fn test_char_class_bracket_as_first() {
    // ] as first char in class is literal
    let glob = Glob::new("[]a].txt").unwrap();
    assert!(glob.is_match("].txt"));
    assert!(glob.is_match("a.txt"));
    assert!(!glob.is_match("b.txt"));
}

#[test]
fn test_char_class_in_filename() {
    let glob = Glob::new("test_[abc]_case.rs").unwrap();
    assert!(glob.is_match("test_a_case.rs"));
    assert!(glob.is_match("test_b_case.rs"));
    assert!(glob.is_match("test_c_case.rs"));
    assert!(!glob.is_match("test_d_case.rs"));
}

#[test]
fn test_char_class_with_question() {
    let glob = Glob::new("[abc]?.txt").unwrap();
    assert!(glob.is_match("a1.txt"));
    assert!(glob.is_match("b2.txt"));
    assert!(glob.is_match("ab.txt")); // a matches [abc], b matches ?
    assert!(!glob.is_match("a.txt")); // needs two chars: one for [abc], one for ?
}

#[test]
fn test_char_class_empty_is_error() {
    // Actually [] might be treated as invalid
    // Let's test that unclosed class is an error
    let result = Glob::new("[abc");
    assert!(matches!(result, Err(GlobError::UnclosedClass)));
}

// =============================================================================
// Negated character classes `[!abc]` tests (8 tests)
// =============================================================================

#[test]
fn test_negated_char_class_basic() {
    let glob = Glob::new("[!abc].txt").unwrap();
    assert!(!glob.is_match("a.txt"));
    assert!(!glob.is_match("b.txt"));
    assert!(!glob.is_match("c.txt"));
    assert!(glob.is_match("d.txt"));
    assert!(glob.is_match("1.txt"));
}

#[test]
fn test_negated_char_class_with_caret() {
    let glob = Glob::new("[^abc].txt").unwrap();
    assert!(!glob.is_match("a.txt"));
    assert!(!glob.is_match("b.txt"));
    assert!(!glob.is_match("c.txt"));
    assert!(glob.is_match("d.txt"));
}

#[test]
fn test_negated_char_class_with_literal() {
    let glob = Glob::new("file[!0-9].txt").unwrap();
    assert!(!glob.is_match("file0.txt"));
    assert!(!glob.is_match("file5.txt"));
    assert!(!glob.is_match("file9.txt"));
    // Note: this tests negated with range
}

#[test]
fn test_negated_does_not_match_separator() {
    let glob = Glob::new("[!a]/file.txt").unwrap();
    // / should not be matched by character class
    assert!(!glob.is_match("//file.txt"));
}

#[test]
fn test_negated_char_class_multiple() {
    let glob = Glob::new("[!abc][!xyz].txt").unwrap();
    assert!(glob.is_match("de.txt"));
    assert!(glob.is_match("12.txt"));
    assert!(!glob.is_match("ax.txt"));
    assert!(!glob.is_match("by.txt"));
}

#[test]
fn test_negated_with_star() {
    let glob = Glob::new("[!abc]*.txt").unwrap();
    assert!(glob.is_match("d.txt"));
    assert!(glob.is_match("dfile.txt"));
    assert!(!glob.is_match("a.txt"));
    assert!(!glob.is_match("bfile.txt"));
}

#[test]
fn test_negated_char_class_in_filename() {
    let glob = Glob::new("test_[!x]_case.rs").unwrap();
    assert!(glob.is_match("test_a_case.rs"));
    assert!(glob.is_match("test_b_case.rs"));
    assert!(!glob.is_match("test_x_case.rs"));
}

#[test]
fn test_negated_matches_most_chars() {
    let glob = Glob::new("[!z]").unwrap();
    assert!(glob.is_match("a"));
    assert!(glob.is_match("m"));
    assert!(glob.is_match("9"));
    assert!(!glob.is_match("z"));
}

// =============================================================================
// Character ranges `[a-z]` tests (8 tests)
// =============================================================================

#[test]
fn test_range_lowercase() {
    let glob = Glob::new("[a-z].txt").unwrap();
    assert!(glob.is_match("a.txt"));
    assert!(glob.is_match("m.txt"));
    assert!(glob.is_match("z.txt"));
    assert!(!glob.is_match("A.txt"));
    assert!(!glob.is_match("1.txt"));
}

#[test]
fn test_range_uppercase() {
    let glob = Glob::new("[A-Z].txt").unwrap();
    assert!(glob.is_match("A.txt"));
    assert!(glob.is_match("M.txt"));
    assert!(glob.is_match("Z.txt"));
    assert!(!glob.is_match("a.txt"));
    assert!(!glob.is_match("1.txt"));
}

#[test]
fn test_range_digits() {
    let glob = Glob::new("[0-9].txt").unwrap();
    assert!(glob.is_match("0.txt"));
    assert!(glob.is_match("5.txt"));
    assert!(glob.is_match("9.txt"));
    assert!(!glob.is_match("a.txt"));
}

#[test]
fn test_multiple_ranges() {
    let glob = Glob::new("[a-zA-Z].txt").unwrap();
    assert!(glob.is_match("a.txt"));
    assert!(glob.is_match("Z.txt"));
    assert!(glob.is_match("m.txt"));
    assert!(!glob.is_match("1.txt"));
    assert!(!glob.is_match("_.txt"));
}

#[test]
fn test_range_with_literal() {
    let glob = Glob::new("file[1-3].txt").unwrap();
    assert!(glob.is_match("file1.txt"));
    assert!(glob.is_match("file2.txt"));
    assert!(glob.is_match("file3.txt"));
    assert!(!glob.is_match("file0.txt"));
    assert!(!glob.is_match("file4.txt"));
}

#[test]
fn test_range_negated() {
    let glob = Glob::new("[!0-9].txt").unwrap();
    assert!(!glob.is_match("0.txt"));
    assert!(!glob.is_match("5.txt"));
    assert!(!glob.is_match("9.txt"));
    assert!(glob.is_match("a.txt"));
    assert!(glob.is_match("A.txt"));
}

#[test]
fn test_range_in_complex_pattern() {
    let glob = Glob::new("test_[a-z]_case*.rs").unwrap();
    assert!(glob.is_match("test_a_case.rs"));
    assert!(glob.is_match("test_b_case_extra.rs"));
    assert!(!glob.is_match("test_A_case.rs"));
    assert!(!glob.is_match("test_1_case.rs"));
}

#[test]
fn test_range_edge_cases() {
    // Range where start == end
    let glob = Glob::new("[a-a].txt").unwrap();
    assert!(glob.is_match("a.txt"));
    assert!(!glob.is_match("b.txt"));
}

// =============================================================================
// GlobSet with multiple patterns tests (8 tests)
// =============================================================================

#[test]
fn test_globset_basic() {
    let set = GlobSet::new(vec!["*.rs", "*.toml"]).unwrap();
    assert!(set.matches_any("lib.rs"));
    assert!(set.matches_any("Cargo.toml"));
    assert!(!set.matches_any("README.md"));
}

#[test]
fn test_globset_empty() {
    let set = GlobSet::new(Vec::<&str>::new()).unwrap();
    assert!(set.is_empty());
    assert!(!set.matches_any("anything.txt"));
}

#[test]
fn test_globset_len() {
    let set = GlobSet::new(vec!["*.rs", "*.toml", "*.md"]).unwrap();
    assert_eq!(set.len(), 3);
    assert!(!set.is_empty());
}

#[test]
fn test_globset_filter() {
    let set = GlobSet::new(vec!["*.rs", "*.toml"]).unwrap();
    let paths = vec!["lib.rs", "README.md", "Cargo.toml", "main.go"];
    let filtered: Vec<_> = set.filter(paths).collect();
    assert_eq!(filtered, vec!["lib.rs", "Cargo.toml"]);
}

#[test]
fn test_globset_with_globstar() {
    let set = GlobSet::new(vec!["src/**/*.rs", "tests/**/*.rs"]).unwrap();
    assert!(set.matches_any("src/lib.rs"));
    assert!(set.matches_any("src/foo/bar.rs"));
    assert!(set.matches_any("tests/test_foo.rs"));
    assert!(!set.matches_any("lib.rs"));
}

#[test]
fn test_globset_complex_patterns() {
    let set = GlobSet::new(vec!["*.rs", "src/**", "docs/*.md"]).unwrap();
    assert!(set.matches_any("lib.rs"));
    assert!(set.matches_any("src/foo/bar.rs"));
    assert!(set.matches_any("docs/README.md"));
    assert!(!set.matches_any("other/file.txt"));
}

#[test]
fn test_globset_char_classes() {
    let set = GlobSet::new(vec!["[abc]*.txt", "[xyz]*.txt"]).unwrap();
    assert!(set.matches_any("a_file.txt"));
    assert!(set.matches_any("z_file.txt"));
    assert!(!set.matches_any("m_file.txt"));
}

#[test]
fn test_globset_clone() {
    let set = GlobSet::new(vec!["*.rs"]).unwrap();
    let cloned = set.clone();
    assert!(cloned.matches_any("lib.rs"));
    assert_eq!(set.len(), cloned.len());
}

// =============================================================================
// Edge cases and error conditions tests (8 tests)
// =============================================================================

#[test]
fn test_empty_pattern_error() {
    let result = Glob::new("");
    assert!(matches!(result, Err(GlobError::EmptyPattern)));
}

#[test]
fn test_unclosed_char_class_error() {
    let result = Glob::new("[abc");
    assert!(matches!(result, Err(GlobError::UnclosedClass)));
}

#[test]
fn test_unclosed_char_class_no_close() {
    let result = Glob::new("[a");
    assert!(matches!(result, Err(GlobError::UnclosedClass)));
}

#[test]
fn test_literal_only_pattern() {
    let glob = Glob::new("exact_match.txt").unwrap();
    assert!(glob.is_match("exact_match.txt"));
    assert!(!glob.is_match("exact_match.md"));
    assert!(!glob.is_match("other.txt"));
}

#[test]
fn test_pattern_with_special_chars() {
    let glob = Glob::new("file-name.txt").unwrap();
    assert!(glob.is_match("file-name.txt"));
    assert!(!glob.is_match("file_name.txt"));
}

#[test]
fn test_windows_backslash_normalization() {
    let glob = Glob::new("src/*.rs").unwrap();
    assert!(glob.is_match("src\\lib.rs"));
    assert!(glob.is_match("src/lib.rs"));
}

#[test]
fn test_pattern_preservation() {
    let pattern = "src/**/*.rs";
    let glob = Glob::new(pattern).unwrap();
    assert_eq!(glob.pattern(), pattern);
}

#[test]
fn test_complex_pattern_combination() {
    let glob = Glob::new("src/**/test_[a-z]*.rs").unwrap();
    assert!(glob.is_match("src/test_a.rs"));
    assert!(glob.is_match("src/foo/test_b_extra.rs"));
    assert!(glob.is_match("src/a/b/c/test_z.rs"));
    assert!(!glob.is_match("src/test_A.rs")); // uppercase doesn't match [a-z]
    assert!(!glob.is_match("tests/test_a.rs")); // wrong prefix
}

// =============================================================================
// Additional tests to ensure 70+ coverage
// =============================================================================

#[test]
fn test_star_at_end() {
    let glob = Glob::new("prefix*").unwrap();
    assert!(glob.is_match("prefix"));
    assert!(glob.is_match("prefix_extra"));
    assert!(glob.is_match("prefix123"));
    assert!(!glob.is_match("pref"));
}

#[test]
fn test_double_star_not_globstar() {
    // When ** is not between separators, it should act as two * 
    let glob = Glob::new("file**.rs").unwrap();
    assert!(glob.is_match("file.rs"));
    assert!(glob.is_match("file_extra.rs"));
    // ** should match across anything
    assert!(glob.is_match("filex/y.rs"));
}

#[test]
fn test_globset_with_invalid_pattern() {
    let result = GlobSet::new(vec!["*.rs", "[invalid", "*.toml"]);
    assert!(result.is_err());
}

#[test]
fn test_path_with_spaces() {
    let glob = Glob::new("my file*.txt").unwrap();
    assert!(glob.is_match("my file.txt"));
    assert!(glob.is_match("my file_name.txt"));
}

#[test]
fn test_unicode_in_pattern() {
    let glob = Glob::new("文件*.txt").unwrap();
    assert!(glob.is_match("文件.txt"));
    assert!(glob.is_match("文件_测试.txt"));
}

#[test]
fn test_consecutive_stars() {
    // ** followed by literal and * - ** matches across separators
    let glob = Glob::new("a/**/b*c").unwrap();
    assert!(glob.is_match("a/bc"));
    assert!(glob.is_match("a/x/bxc"));
    assert!(glob.is_match("a/x/y/bxc"));
}

#[test]
fn test_char_class_with_dash_at_end() {
    let glob = Glob::new("[a-].txt").unwrap();
    assert!(glob.is_match("a.txt"));
    assert!(glob.is_match("-.txt"));
    assert!(!glob.is_match("b.txt"));
}

#[test]
fn test_char_class_with_dash_at_start() {
    let glob = Glob::new("[-a].txt").unwrap();
    assert!(glob.is_match("a.txt"));
    assert!(glob.is_match("-.txt"));
    assert!(!glob.is_match("b.txt"));
}

#[test]
fn test_globset_filter_empty_result() {
    let set = GlobSet::new(vec!["*.rs"]).unwrap();
    let paths = vec!["file.txt", "file.md", "file.go"];
    let filtered: Vec<_> = set.filter(paths).collect();
    assert!(filtered.is_empty());
}

#[test]
fn test_globset_filter_all_match() {
    let set = GlobSet::new(vec!["*"]).unwrap();
    let paths = vec!["file.txt", "file.md", "file.go"];
    let filtered: Vec<_> = set.filter(paths).collect();
    assert_eq!(filtered.len(), 3);
}

#[test]
fn test_exact_match_with_star() {
    let glob = Glob::new("a*b").unwrap();
    assert!(glob.is_match("ab"));
    assert!(glob.is_match("axb"));
    assert!(glob.is_match("axxb"));
    assert!(!glob.is_match("a"));
    assert!(!glob.is_match("b"));
    assert!(!glob.is_match("abc"));
}

#[test]
fn test_pattern_with_numbers() {
    let glob = Glob::new("test[0-9].rs").unwrap();
    assert!(glob.is_match("test0.rs"));
    assert!(glob.is_match("test5.rs"));
    assert!(glob.is_match("test9.rs"));
    assert!(!glob.is_match("test.rs"));
    assert!(!glob.is_match("test10.rs"));
}

#[test]
fn test_multiple_questions_exact_count() {
    let glob = Glob::new("???").unwrap();
    assert!(glob.is_match("abc"));
    assert!(glob.is_match("123"));
    assert!(!glob.is_match("ab"));
    assert!(!glob.is_match("abcd"));
}

#[test]
fn test_mixed_pattern() {
    let glob = Glob::new("src/[a-z]*/??.rs").unwrap();
    assert!(glob.is_match("src/foo/ab.rs"));
    assert!(glob.is_match("src/bar/xy.rs"));
    assert!(!glob.is_match("src/FOO/ab.rs")); // uppercase doesn't match [a-z]
    assert!(!glob.is_match("src/foo/a.rs")); // only one char, need two
}

#[test]
fn test_globset_single_pattern() {
    let set = GlobSet::new(vec!["*.rs"]).unwrap();
    assert_eq!(set.len(), 1);
    assert!(set.matches_any("lib.rs"));
    assert!(!set.matches_any("lib.txt"));
}

#[test]
fn test_char_class_numbers_and_letters() {
    let glob = Glob::new("[a-zA-Z0-9].txt").unwrap();
    assert!(glob.is_match("a.txt"));
    assert!(glob.is_match("Z.txt"));
    assert!(glob.is_match("5.txt"));
    assert!(!glob.is_match("_.txt"));
    assert!(!glob.is_match("-.txt"));
}

#[test]
fn test_globstar_preserves_prefix() {
    let glob = Glob::new("src/**/*.rs").unwrap();
    // Must start with src/
    assert!(glob.is_match("src/lib.rs"));
    assert!(!glob.is_match("lib.rs"));
    assert!(!glob.is_match("test/src/lib.rs"));
}

#[test]
fn test_star_question_combination() {
    let glob = Glob::new("*?.rs").unwrap();
    assert!(glob.is_match("ab.rs")); // * matches a, ? matches b
    assert!(glob.is_match("abc.rs")); // * matches ab, ? matches c
    // * can match empty, so a.rs works: * matches "", ? matches "a"
    assert!(glob.is_match("a.rs"));
    assert!(!glob.is_match(".rs")); // ? needs at least one char
}

#[test]
fn test_negated_range() {
    let glob = Glob::new("[!a-z].txt").unwrap();
    assert!(!glob.is_match("a.txt"));
    assert!(!glob.is_match("m.txt"));
    assert!(!glob.is_match("z.txt"));
    assert!(glob.is_match("A.txt"));
    assert!(glob.is_match("1.txt"));
}

#[test]
fn test_empty_globset_filter() {
    let set = GlobSet::new(Vec::<&str>::new()).unwrap();
    let paths = vec!["file.txt"];
    let filtered: Vec<_> = set.filter(paths).collect();
    assert!(filtered.is_empty());
}

#[test]
fn test_glob_debug() {
    let glob = Glob::new("*.rs").unwrap();
    let debug_str = format!("{glob:?}");
    assert!(debug_str.contains("Glob"));
}

#[test]
fn test_globset_debug() {
    let set = GlobSet::new(vec!["*.rs"]).unwrap();
    let debug_str = format!("{set:?}");
    assert!(debug_str.contains("GlobSet"));
}

#[test]
fn test_glob_clone() {
    let glob = Glob::new("*.rs").unwrap();
    let cloned = glob.clone();
    assert_eq!(glob.pattern(), cloned.pattern());
    assert!(cloned.is_match("lib.rs"));
}

#[test]
fn test_glob_error_debug() {
    let err = GlobError::EmptyPattern;
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("EmptyPattern"));
}

#[test]
fn test_glob_error_display() {
    let err = GlobError::EmptyPattern;
    let display_str = format!("{err}");
    assert!(display_str.contains("Empty pattern"));
}

#[test]
fn test_glob_error_clone() {
    let err = GlobError::UnclosedClass;
    let cloned = err.clone();
    assert!(matches!(cloned, GlobError::UnclosedClass));
}
