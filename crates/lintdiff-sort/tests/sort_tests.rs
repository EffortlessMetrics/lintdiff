//! Comprehensive tests for lintdiff-sort crate.
//!
//! Test coverage:
//! 1. SortKey enum (8 tests)
//! 2. SortDirection enum (4 tests)
//! 3. SortConfig builder methods (10 tests)
//! 4. compare_by_key function (7 tests)
//! 5. compare function with multiple keys (8 tests)
//! 6. sort_slice function (6 tests)
//! 7. natural_compare function (8 tests)
//! 8. natural_sort functions (4 tests)

use lintdiff_sort::{
    compare, compare_by_key, natural_compare, natural_sort, natural_sort_owned, sort_slice, sorted,
    SortConfig, SortDirection, SortKey, Sortable,
};
use std::cmp::Ordering;

/// Test item for sorting tests
#[derive(Debug, Clone)]
struct TestItem {
    path: String,
    severity: u8,
    line: u32,
    column: u32,
    code: String,
    message: String,
    fingerprint: String,
}

impl Sortable for TestItem {
    fn sort_path(&self) -> &str {
        &self.path
    }
    fn sort_severity(&self) -> u8 {
        self.severity
    }
    fn sort_line(&self) -> u32 {
        self.line
    }
    fn sort_column(&self) -> u32 {
        self.column
    }
    fn sort_code(&self) -> &str {
        &self.code
    }
    fn sort_message(&self) -> &str {
        &self.message
    }
    fn sort_fingerprint(&self) -> &str {
        &self.fingerprint
    }
}

impl Default for TestItem {
    fn default() -> Self {
        Self {
            path: String::new(),
            severity: 0,
            line: 0,
            column: 0,
            code: String::new(),
            message: String::new(),
            fingerprint: String::new(),
        }
    }
}

impl TestItem {
    fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            ..Default::default()
        }
    }

    fn with_severity(mut self, severity: u8) -> Self {
        self.severity = severity;
        self
    }

    fn with_line(mut self, line: u32) -> Self {
        self.line = line;
        self
    }

    fn with_column(mut self, column: u32) -> Self {
        self.column = column;
        self
    }

    fn with_code(mut self, code: &str) -> Self {
        self.code = code.to_string();
        self
    }

    fn with_message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    fn with_fingerprint(mut self, fingerprint: &str) -> Self {
        self.fingerprint = fingerprint.to_string();
        self
    }
}

// ============================================================================
// 1. SortKey enum tests (8 tests)
// ============================================================================

mod sort_key_tests {
    use super::*;

    #[test]
    fn sort_key_default_is_path() {
        assert_eq!(SortKey::default(), SortKey::Path);
    }

    #[test]
    fn sort_key_variants_are_distinct() {
        let keys = [
            SortKey::Path,
            SortKey::Severity,
            SortKey::Line,
            SortKey::Column,
            SortKey::Code,
            SortKey::Message,
            SortKey::Fingerprint,
        ];

        // All variants should be unique
        for (i, key1) in keys.iter().enumerate() {
            for (j, key2) in keys.iter().enumerate() {
                if i != j {
                    assert_ne!(key1, key2, "SortKey variants should be distinct");
                }
            }
        }
    }

    #[test]
    fn sort_key_clone() {
        let key = SortKey::Severity;
        let cloned = key.clone();
        assert_eq!(key, cloned);
    }

    #[test]
    fn sort_key_copy() {
        let key = SortKey::Line;
        let copied: SortKey = key;
        assert_eq!(key, copied);
    }

    #[test]
    fn sort_key_debug_format() {
        assert!(format!("{:?}", SortKey::Path).contains("Path"));
        assert!(format!("{:?}", SortKey::Severity).contains("Severity"));
        assert!(format!("{:?}", SortKey::Line).contains("Line"));
    }

    #[test]
    fn sort_key_eq() {
        assert_eq!(SortKey::Path, SortKey::Path);
        assert_ne!(SortKey::Path, SortKey::Severity);
    }

    #[test]
    fn sort_key_hash_consistency() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(SortKey::Path);
        set.insert(SortKey::Severity);
        set.insert(SortKey::Path); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn sort_key_all_variants_covered() {
        // Ensure we can match all variants
        let keys = [
            SortKey::Path,
            SortKey::Severity,
            SortKey::Line,
            SortKey::Column,
            SortKey::Code,
            SortKey::Message,
            SortKey::Fingerprint,
        ];

        for key in keys {
            let _ = match key {
                SortKey::Path => "path",
                SortKey::Severity => "severity",
                SortKey::Line => "line",
                SortKey::Column => "column",
                SortKey::Code => "code",
                SortKey::Message => "message",
                SortKey::Fingerprint => "fingerprint",
            };
        }
    }
}

// ============================================================================
// 2. SortDirection enum tests (4 tests)
// ============================================================================

mod sort_direction_tests {
    use super::*;

    #[test]
    fn sort_direction_default_is_ascending() {
        assert_eq!(SortDirection::default(), SortDirection::Ascending);
    }

    #[test]
    fn sort_direction_variants_are_distinct() {
        assert_ne!(SortDirection::Ascending, SortDirection::Descending);
    }

    #[test]
    fn sort_direction_debug_format() {
        assert!(format!("{:?}", SortDirection::Ascending).contains("Ascending"));
        assert!(format!("{:?}", SortDirection::Descending).contains("Descending"));
    }

    #[test]
    fn sort_direction_clone_copy_eq() {
        let dir = SortDirection::Descending;
        let cloned = dir.clone();
        let copied: SortDirection = dir;
        assert_eq!(dir, cloned);
        assert_eq!(dir, copied);
    }
}

// ============================================================================
// 3. SortConfig builder tests (10 tests)
// ============================================================================

mod sort_config_tests {
    use super::*;

    #[test]
    fn sort_config_default() {
        let config = SortConfig::default();
        assert_eq!(config.primary, SortKey::Path);
        assert_eq!(config.secondary, Some(SortKey::Line));
        assert_eq!(config.tertiary, Some(SortKey::Column));
        assert_eq!(config.direction, SortDirection::Ascending);
    }

    #[test]
    fn sort_config_new() {
        let config = SortConfig::new(SortKey::Severity);
        assert_eq!(config.primary, SortKey::Severity);
        // Secondary and tertiary should come from default
        assert_eq!(config.secondary, Some(SortKey::Line));
        assert_eq!(config.tertiary, Some(SortKey::Column));
    }

    #[test]
    fn sort_config_with_secondary() {
        let config = SortConfig::new(SortKey::Path).with_secondary(SortKey::Code);
        assert_eq!(config.secondary, Some(SortKey::Code));
    }

    #[test]
    fn sort_config_with_tertiary() {
        let config = SortConfig::new(SortKey::Path).with_tertiary(SortKey::Message);
        assert_eq!(config.tertiary, Some(SortKey::Message));
    }

    #[test]
    fn sort_config_descending() {
        let config = SortConfig::default().descending();
        assert_eq!(config.direction, SortDirection::Descending);
    }

    #[test]
    fn sort_config_ascending() {
        let config = SortConfig::default().descending().ascending();
        assert_eq!(config.direction, SortDirection::Ascending);
    }

    #[test]
    fn sort_config_by_severity() {
        let config = SortConfig::by_severity();
        assert_eq!(config.primary, SortKey::Severity);
        assert_eq!(config.secondary, Some(SortKey::Path));
        assert_eq!(config.tertiary, Some(SortKey::Column)); // From default
                                                            // Note: by_severity uses ascending direction because compare_by_key for Severity
                                                            // already reverses the order (higher severity first)
        assert_eq!(config.direction, SortDirection::Ascending);
    }

    #[test]
    fn sort_config_by_path() {
        let config = SortConfig::by_path();
        assert_eq!(config.primary, SortKey::Path);
        assert_eq!(config.secondary, Some(SortKey::Line));
        assert_eq!(config.tertiary, Some(SortKey::Column));
        assert_eq!(config.direction, SortDirection::Ascending);
    }

    #[test]
    fn sort_config_by_code() {
        let config = SortConfig::by_code();
        assert_eq!(config.primary, SortKey::Code);
        assert_eq!(config.secondary, Some(SortKey::Path));
        assert_eq!(config.tertiary, Some(SortKey::Column)); // From default
    }

    #[test]
    fn sort_config_builder_chain() {
        let config = SortConfig::new(SortKey::Message)
            .with_secondary(SortKey::Code)
            .with_tertiary(SortKey::Path)
            .descending();

        assert_eq!(config.primary, SortKey::Message);
        assert_eq!(config.secondary, Some(SortKey::Code));
        assert_eq!(config.tertiary, Some(SortKey::Path));
        assert_eq!(config.direction, SortDirection::Descending);
    }
}

// ============================================================================
// 4. compare_by_key function tests (7 tests)
// ============================================================================

mod compare_by_key_tests {
    use super::*;

    #[test]
    fn compare_by_key_path() {
        let a = TestItem::new("a.rs");
        let b = TestItem::new("b.rs");

        assert_eq!(compare_by_key(&a, &b, SortKey::Path), Ordering::Less);
        assert_eq!(compare_by_key(&b, &a, SortKey::Path), Ordering::Greater);
        assert_eq!(compare_by_key(&a, &a, SortKey::Path), Ordering::Equal);
    }

    #[test]
    fn compare_by_key_severity() {
        let low = TestItem::new("test").with_severity(1);
        let high = TestItem::new("test").with_severity(3);

        // Higher severity comes first (so low > high means Less is returned)
        assert_eq!(
            compare_by_key(&low, &high, SortKey::Severity),
            Ordering::Greater
        );
        assert_eq!(
            compare_by_key(&high, &low, SortKey::Severity),
            Ordering::Less
        );
        assert_eq!(
            compare_by_key(&low, &low, SortKey::Severity),
            Ordering::Equal
        );
    }

    #[test]
    fn compare_by_key_line() {
        let low = TestItem::new("test").with_line(10);
        let high = TestItem::new("test").with_line(20);

        assert_eq!(compare_by_key(&low, &high, SortKey::Line), Ordering::Less);
        assert_eq!(
            compare_by_key(&high, &low, SortKey::Line),
            Ordering::Greater
        );
    }

    #[test]
    fn compare_by_key_column() {
        let low = TestItem::new("test").with_column(5);
        let high = TestItem::new("test").with_column(15);

        assert_eq!(compare_by_key(&low, &high, SortKey::Column), Ordering::Less);
        assert_eq!(
            compare_by_key(&high, &low, SortKey::Column),
            Ordering::Greater
        );
    }

    #[test]
    fn compare_by_key_code() {
        let a = TestItem::new("test").with_code("E001");
        let b = TestItem::new("test").with_code("E002");

        assert_eq!(compare_by_key(&a, &b, SortKey::Code), Ordering::Less);
        assert_eq!(compare_by_key(&b, &a, SortKey::Code), Ordering::Greater);
    }

    #[test]
    fn compare_by_key_message() {
        let a = TestItem::new("test").with_message("error a");
        let b = TestItem::new("test").with_message("error b");

        assert_eq!(compare_by_key(&a, &b, SortKey::Message), Ordering::Less);
        assert_eq!(compare_by_key(&b, &a, SortKey::Message), Ordering::Greater);
    }

    #[test]
    fn compare_by_key_fingerprint() {
        let a = TestItem::new("test").with_fingerprint("abc123");
        let b = TestItem::new("test").with_fingerprint("def456");

        assert_eq!(compare_by_key(&a, &b, SortKey::Fingerprint), Ordering::Less);
        assert_eq!(
            compare_by_key(&b, &a, SortKey::Fingerprint),
            Ordering::Greater
        );
    }
}

// ============================================================================
// 5. compare function with multiple keys tests (8 tests)
// ============================================================================

mod compare_tests {
    use super::*;

    #[test]
    fn compare_primary_key_only() {
        let a = TestItem::new("a.rs");
        let b = TestItem::new("b.rs");
        let config = SortConfig::new(SortKey::Path);

        assert_eq!(compare(&a, &b, &config), Ordering::Less);
    }

    #[test]
    fn compare_uses_secondary_on_primary_tie() {
        let a = TestItem::new("same.rs").with_line(10);
        let b = TestItem::new("same.rs").with_line(20);
        let config = SortConfig::new(SortKey::Path).with_secondary(SortKey::Line);

        assert_eq!(compare(&a, &b, &config), Ordering::Less);
    }

    #[test]
    fn compare_uses_tertiary_on_secondary_tie() {
        let a = TestItem::new("same.rs").with_line(10).with_column(5);
        let b = TestItem::new("same.rs").with_line(10).with_column(10);
        let config = SortConfig::new(SortKey::Path)
            .with_secondary(SortKey::Line)
            .with_tertiary(SortKey::Column);

        assert_eq!(compare(&a, &b, &config), Ordering::Less);
    }

    #[test]
    fn compare_returns_equal_when_all_keys_tie() {
        let a = TestItem::new("same.rs").with_line(10).with_column(5);
        let b = TestItem::new("same.rs").with_line(10).with_column(5);
        let config = SortConfig::new(SortKey::Path)
            .with_secondary(SortKey::Line)
            .with_tertiary(SortKey::Column);

        assert_eq!(compare(&a, &b, &config), Ordering::Equal);
    }

    #[test]
    fn compare_descending_reverses_order() {
        let a = TestItem::new("a.rs");
        let b = TestItem::new("b.rs");
        let config = SortConfig::new(SortKey::Path).descending();

        assert_eq!(compare(&a, &b, &config), Ordering::Greater);
    }

    #[test]
    fn compare_no_secondary_key() {
        let a = TestItem::new("same.rs").with_line(10);
        let b = TestItem::new("same.rs").with_line(20);
        let config = SortConfig {
            primary: SortKey::Path,
            secondary: None,
            tertiary: None,
            direction: SortDirection::Ascending,
        };

        assert_eq!(compare(&a, &b, &config), Ordering::Equal);
    }

    #[test]
    fn compare_no_tertiary_key() {
        let a = TestItem::new("same.rs").with_line(10).with_column(5);
        let b = TestItem::new("same.rs").with_line(10).with_column(10);
        let config = SortConfig {
            primary: SortKey::Path,
            secondary: Some(SortKey::Line),
            tertiary: None,
            direction: SortDirection::Ascending,
        };

        assert_eq!(compare(&a, &b, &config), Ordering::Equal);
    }

    #[test]
    fn compare_severity_config() {
        let low = TestItem::new("a.rs").with_severity(1);
        let high = TestItem::new("b.rs").with_severity(3);
        let config = SortConfig::by_severity();

        // High severity comes first
        assert_eq!(compare(&low, &high, &config), Ordering::Greater);
    }
}

// ============================================================================
// 6. sort_slice function tests (6 tests)
// ============================================================================

mod sort_slice_tests {
    use super::*;

    #[test]
    fn sort_slice_basic() {
        let mut items = vec![
            TestItem::new("c.rs"),
            TestItem::new("a.rs"),
            TestItem::new("b.rs"),
        ];

        sort_slice(&mut items, &SortConfig::by_path());

        assert_eq!(items[0].path, "a.rs");
        assert_eq!(items[1].path, "b.rs");
        assert_eq!(items[2].path, "c.rs");
    }

    #[test]
    fn sort_slice_empty() {
        let mut items: Vec<TestItem> = vec![];
        sort_slice(&mut items, &SortConfig::default());
        assert!(items.is_empty());
    }

    #[test]
    fn sort_slice_single_item() {
        let mut items = vec![TestItem::new("only.rs")];
        sort_slice(&mut items, &SortConfig::default());
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].path, "only.rs");
    }

    #[test]
    fn sort_slice_with_secondary_key() {
        let mut items = vec![
            TestItem::new("same.rs").with_line(30),
            TestItem::new("same.rs").with_line(10),
            TestItem::new("same.rs").with_line(20),
        ];

        sort_slice(&mut items, &SortConfig::by_path());

        assert_eq!(items[0].line, 10);
        assert_eq!(items[1].line, 20);
        assert_eq!(items[2].line, 30);
    }

    #[test]
    fn sort_slice_descending() {
        let mut items = vec![
            TestItem::new("a.rs"),
            TestItem::new("c.rs"),
            TestItem::new("b.rs"),
        ];

        sort_slice(&mut items, &SortConfig::new(SortKey::Path).descending());

        assert_eq!(items[0].path, "c.rs");
        assert_eq!(items[1].path, "b.rs");
        assert_eq!(items[2].path, "a.rs");
    }

    #[test]
    fn sort_slice_by_severity() {
        let mut items = vec![
            TestItem::new("a.rs").with_severity(1),
            TestItem::new("b.rs").with_severity(3),
            TestItem::new("c.rs").with_severity(2),
        ];

        sort_slice(&mut items, &SortConfig::by_severity());

        // Highest severity first
        assert_eq!(items[0].severity, 3);
        assert_eq!(items[1].severity, 2);
        assert_eq!(items[2].severity, 1);
    }
}

// ============================================================================
// 7. natural_compare function tests (8 tests)
// ============================================================================

mod natural_compare_tests {
    use super::*;

    #[test]
    fn natural_compare_numeric_ordering() {
        assert_eq!(natural_compare("file2", "file10"), Ordering::Less);
        assert_eq!(natural_compare("file10", "file2"), Ordering::Greater);
        assert_eq!(natural_compare("file2", "file2"), Ordering::Equal);
    }

    #[test]
    fn natural_compare_multiple_numbers() {
        assert_eq!(natural_compare("file1.txt", "file10.txt"), Ordering::Less);
        assert_eq!(
            natural_compare("file10.txt", "file2.txt"),
            Ordering::Greater
        );
        assert_eq!(natural_compare("file2part1", "file2part10"), Ordering::Less);
    }

    #[test]
    fn natural_compare_with_letters() {
        assert_eq!(natural_compare("file2a", "file2b"), Ordering::Less);
        assert_eq!(natural_compare("file2b", "file2a"), Ordering::Greater);
        assert_eq!(natural_compare("file2a", "file2a"), Ordering::Equal);
    }

    #[test]
    fn natural_compare_pure_strings() {
        assert_eq!(natural_compare("abc", "def"), Ordering::Less);
        assert_eq!(natural_compare("def", "abc"), Ordering::Greater);
        assert_eq!(natural_compare("abc", "abc"), Ordering::Equal);
    }

    #[test]
    fn natural_compare_pure_numbers() {
        assert_eq!(natural_compare("100", "20"), Ordering::Greater);
        assert_eq!(natural_compare("20", "100"), Ordering::Less);
        assert_eq!(natural_compare("20", "20"), Ordering::Equal);
    }

    #[test]
    fn natural_compare_empty_strings() {
        assert_eq!(natural_compare("", ""), Ordering::Equal);
        assert_eq!(natural_compare("", "a"), Ordering::Less);
        assert_eq!(natural_compare("a", ""), Ordering::Greater);
    }

    #[test]
    fn natural_compare_numbers_before_letters() {
        // In natural sort, numbers typically come before letters
        assert_eq!(natural_compare("123", "abc"), Ordering::Less);
        assert_eq!(natural_compare("abc", "123"), Ordering::Greater);
    }

    #[test]
    fn natural_compare_complex_paths() {
        assert_eq!(
            natural_compare("src/v1/file.rs", "src/v10/file.rs"),
            Ordering::Less
        );
        assert_eq!(
            natural_compare("test_001.rs", "test_010.rs"),
            Ordering::Less
        );
    }
}

// ============================================================================
// 8. natural_sort functions tests (4 tests)
// ============================================================================

mod natural_sort_tests {
    use super::*;

    #[test]
    fn natural_sort_str_slice() {
        let mut strings = ["file10", "file2", "file1"];
        natural_sort(&mut strings);
        assert_eq!(strings, ["file1", "file2", "file10"]);
    }

    #[test]
    fn natural_sort_owned_strings() {
        let mut strings = vec![
            "file10".to_string(),
            "file2".to_string(),
            "file1".to_string(),
        ];
        natural_sort_owned(&mut strings);
        assert_eq!(
            strings,
            vec![
                "file1".to_string(),
                "file2".to_string(),
                "file10".to_string()
            ]
        );
    }

    #[test]
    fn natural_sort_empty() {
        let mut strings: [&str; 0] = [];
        natural_sort(&mut strings);
        assert!(strings.is_empty());

        let mut owned: Vec<String> = vec![];
        natural_sort_owned(&mut owned);
        assert!(owned.is_empty());
    }

    #[test]
    fn natural_sort_preserves_equal_elements() {
        let mut strings = ["file2", "file2", "file1"];
        natural_sort(&mut strings);
        assert_eq!(strings, ["file1", "file2", "file2"]);
    }
}

// ============================================================================
// Additional tests for sorted iterator
// ============================================================================

mod sorted_iterator_tests {
    use super::*;

    #[test]
    fn sorted_iterator_basic() {
        let items = vec![
            TestItem::new("c.rs"),
            TestItem::new("a.rs"),
            TestItem::new("b.rs"),
        ];

        let sorted_items: Vec<_> = sorted(items, &SortConfig::by_path()).collect();

        assert_eq!(sorted_items[0].path, "a.rs");
        assert_eq!(sorted_items[1].path, "b.rs");
        assert_eq!(sorted_items[2].path, "c.rs");
    }

    #[test]
    fn sorted_iterator_empty() {
        let items: Vec<TestItem> = vec![];
        let sorted_items: Vec<_> = sorted(items, &SortConfig::default()).collect();
        assert!(sorted_items.is_empty());
    }
}

// ============================================================================
// Edge case tests
// ============================================================================

mod edge_case_tests {
    use super::*;

    #[test]
    fn unicode_paths() {
        let a = TestItem::new("α.rs");
        let b = TestItem::new("β.rs");

        assert_eq!(compare_by_key(&a, &b, SortKey::Path), Ordering::Less);
    }

    #[test]
    fn very_long_paths() {
        let long_path = "a".repeat(1000);
        let a = TestItem::new(&long_path);
        let b = TestItem::new(&format!("{}b", long_path));

        assert_eq!(compare_by_key(&a, &b, SortKey::Path), Ordering::Less);
    }

    #[test]
    fn max_severity_values() {
        let a = TestItem::new("test").with_severity(u8::MAX);
        let b = TestItem::new("test").with_severity(0);

        assert_eq!(compare_by_key(&a, &b, SortKey::Severity), Ordering::Less); // Higher severity first
    }

    #[test]
    fn max_line_column_values() {
        let a = TestItem::new("test")
            .with_line(u32::MAX)
            .with_column(u32::MAX);
        let b = TestItem::new("test").with_line(0).with_column(0);

        assert_eq!(compare_by_key(&a, &b, SortKey::Line), Ordering::Greater);
        assert_eq!(compare_by_key(&a, &b, SortKey::Column), Ordering::Greater);
    }

    #[test]
    fn special_characters_in_code() {
        let a = TestItem::new("test").with_code("E-001");
        let b = TestItem::new("test").with_code("E_002");

        // '-' < '_' in ASCII
        assert_eq!(compare_by_key(&a, &b, SortKey::Code), Ordering::Less);
    }
}

// ============================================================================
// Property-based tests using proptest
// ============================================================================

mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn natural_compare_reflexive(s in ".*") {
            prop_assert_eq!(natural_compare(&s, &s), Ordering::Equal);
        }

        #[test]
        fn natural_compare_symmetric(a in ".*", b in ".*") {
            let cmp_ab = natural_compare(&a, &b);
            let cmp_ba = natural_compare(&b, &a);
            prop_assert_eq!(cmp_ab, cmp_ba.reverse());
        }

        #[test]
        fn sort_slice_idempotent(items in prop::collection::vec(".*", 0..20)) {
            let test_items: Vec<TestItem> = items.iter().map(|s| TestItem::new(s)).collect();
            let mut test_items1 = test_items.clone();
            let mut test_items2 = test_items.clone();

            sort_slice(&mut test_items1, &SortConfig::by_path());
            sort_slice(&mut test_items2, &SortConfig::by_path());

            for (a, b) in test_items1.iter().zip(test_items2.iter()) {
                prop_assert_eq!(&a.path, &b.path);
            }
        }

        #[test]
        fn compare_transitive_on_paths(a in ".*", b in ".*", c in ".*") {
            let item_a = TestItem::new(&a);
            let item_b = TestItem::new(&b);
            let item_c = TestItem::new(&c);
            let config = SortConfig::by_path();

            let cmp_ab = compare(&item_a, &item_b, &config);
            let cmp_bc = compare(&item_b, &item_c, &config);

            // If a <= b and b <= c, then a <= c
            if cmp_ab != Ordering::Greater && cmp_bc != Ordering::Greater {
                let cmp_ac = compare(&item_a, &item_c, &config);
                prop_assert!(cmp_ac != Ordering::Greater);
            }
        }
    }
}
