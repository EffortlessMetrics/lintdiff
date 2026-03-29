//! Comprehensive tests for the lintdiff-explain crate.

use lintdiff_explain::{Disposition, Explainable, Explanation};

// ============================================================================
// Disposition Tests
// ============================================================================

#[test]
fn all_disposition_variants_exist() {
    // Ensure all variants can be constructed
    let _ = Disposition::Included;
    let _ = Disposition::OutsideDiff;
    let _ = Disposition::GeneratedFile;
    let _ = Disposition::Suppressed;
    let _ = Disposition::NoSpan;
    let _ = Disposition::NonWorkspace;
}

#[test]
fn disposition_is_included_only_true_for_included() {
    assert!(Disposition::Included.is_included());
    assert!(!Disposition::OutsideDiff.is_included());
    assert!(!Disposition::GeneratedFile.is_included());
    assert!(!Disposition::Suppressed.is_included());
    assert!(!Disposition::NoSpan.is_included());
    assert!(!Disposition::NonWorkspace.is_included());
}

#[test]
fn disposition_as_str_matches_variant_names() {
    assert_eq!(Disposition::Included.as_str(), "included");
    assert_eq!(Disposition::OutsideDiff.as_str(), "outside_diff");
    assert_eq!(Disposition::GeneratedFile.as_str(), "generated_file");
    assert_eq!(Disposition::Suppressed.as_str(), "suppressed");
    assert_eq!(Disposition::NoSpan.as_str(), "no_span");
    assert_eq!(Disposition::NonWorkspace.as_str(), "non_workspace");
}

#[test]
fn disposition_display_uses_as_str() {
    assert_eq!(format!("{}", Disposition::Included), "included");
    assert_eq!(format!("{}", Disposition::OutsideDiff), "outside_diff");
    assert_eq!(format!("{}", Disposition::GeneratedFile), "generated_file");
    assert_eq!(format!("{}", Disposition::Suppressed), "suppressed");
    assert_eq!(format!("{}", Disposition::NoSpan), "no_span");
    assert_eq!(format!("{}", Disposition::NonWorkspace), "non_workspace");
}

#[test]
fn disposition_clone() {
    let d = Disposition::Included;
    let cloned = d.clone();
    assert_eq!(d, cloned);
}

#[test]
fn disposition_copy() {
    let d = Disposition::Included;
    let copied: Disposition = d; // Copy is implicit
    assert_eq!(d, copied);
}

#[test]
fn disposition_eq() {
    assert_eq!(Disposition::Included, Disposition::Included);
    assert_ne!(Disposition::Included, Disposition::OutsideDiff);
}

#[test]
fn disposition_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Disposition::Included);
    set.insert(Disposition::OutsideDiff);
    set.insert(Disposition::Included); // Duplicate
    
    assert_eq!(set.len(), 2);
}

// ============================================================================
// Explanation Constructor Tests
// ============================================================================

#[test]
fn explanation_new_basic() {
    let explanation = Explanation::new(Disposition::Included, "Test reason");
    assert_eq!(explanation.disposition, Disposition::Included);
    assert_eq!(explanation.reason, "Test reason");
    assert_eq!(explanation.code, None);
}

#[test]
fn explanation_new_with_string() {
    let reason = String::from("Test reason");
    let explanation = Explanation::new(Disposition::Suppressed, reason);
    assert_eq!(explanation.reason, "Test reason");
}

#[test]
fn explanation_with_code() {
    let explanation = Explanation::new(Disposition::Suppressed, "Test")
        .with_code("dead_code");
    assert_eq!(explanation.code, Some("dead_code".to_string()));
}

#[test]
fn explanation_with_code_string() {
    let code = String::from("clippy::all");
    let explanation = Explanation::new(Disposition::Suppressed, "Test")
        .with_code(code);
    assert_eq!(explanation.code, Some("clippy::all".to_string()));
}

// ============================================================================
// Explanation Factory Method Tests
// ============================================================================

#[test]
fn explanation_included() {
    let explanation = Explanation::included("Warning on changed line 42");
    assert_eq!(explanation.disposition, Disposition::Included);
    assert_eq!(explanation.reason, "Warning on changed line 42");
    assert_eq!(explanation.code, None);
    assert!(explanation.is_included());
}

#[test]
fn explanation_outside_diff() {
    let explanation = Explanation::outside_diff();
    assert_eq!(explanation.disposition, Disposition::OutsideDiff);
    assert!(!explanation.reason.is_empty());
    assert_eq!(explanation.code, None);
    assert!(!explanation.is_included());
}

#[test]
fn explanation_generated_file() {
    let explanation = Explanation::generated_file();
    assert_eq!(explanation.disposition, Disposition::GeneratedFile);
    assert!(!explanation.reason.is_empty());
    assert_eq!(explanation.code, None);
    assert!(!explanation.is_included());
}

#[test]
fn explanation_suppressed() {
    let explanation = Explanation::suppressed("dead_code");
    assert_eq!(explanation.disposition, Disposition::Suppressed);
    assert!(!explanation.reason.is_empty());
    assert_eq!(explanation.code, Some("dead_code".to_string()));
    assert!(!explanation.is_included());
}

#[test]
fn explanation_suppressed_with_clippy_code() {
    let explanation = Explanation::suppressed("clippy::unwrap_used");
    assert_eq!(explanation.disposition, Disposition::Suppressed);
    assert_eq!(explanation.code, Some("clippy::unwrap_used".to_string()));
    assert!(explanation.reason.contains("clippy::unwrap_used"));
}

#[test]
fn explanation_no_span() {
    let explanation = Explanation::no_span();
    assert_eq!(explanation.disposition, Disposition::NoSpan);
    assert!(!explanation.reason.is_empty());
    assert_eq!(explanation.code, None);
    assert!(!explanation.is_included());
}

#[test]
fn explanation_non_workspace() {
    let explanation = Explanation::non_workspace();
    assert_eq!(explanation.disposition, Disposition::NonWorkspace);
    assert!(!explanation.reason.is_empty());
    assert_eq!(explanation.code, None);
    assert!(!explanation.is_included());
}

// ============================================================================
// Explanation is_included Tests
// ============================================================================

#[test]
fn is_included_true_only_for_included_disposition() {
    assert!(Explanation::included("Test").is_included());
    assert!(!Explanation::outside_diff().is_included());
    assert!(!Explanation::generated_file().is_included());
    assert!(!Explanation::suppressed("code").is_included());
    assert!(!Explanation::no_span().is_included());
    assert!(!Explanation::non_workspace().is_included());
}

// ============================================================================
// Explanation Clone/Eq Tests
// ============================================================================

#[test]
fn explanation_clone() {
    let original = Explanation::suppressed("dead_code");
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn explanation_eq() {
    let a = Explanation::included("Test");
    let b = Explanation::included("Test");
    let c = Explanation::included("Different");
    
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// ============================================================================
// Serialization Tests
// ============================================================================

mod serialization {
    use lintdiff_explain::{Disposition, Explanation};

    #[test]
    fn disposition_serialize_included() {
        let json = serde_json::to_string(&Disposition::Included).unwrap();
        assert_eq!(json, "\"included\"");
    }

    #[test]
    fn disposition_serialize_outside_diff() {
        let json = serde_json::to_string(&Disposition::OutsideDiff).unwrap();
        assert_eq!(json, "\"outside_diff\"");
    }

    #[test]
    fn disposition_serialize_generated_file() {
        let json = serde_json::to_string(&Disposition::GeneratedFile).unwrap();
        assert_eq!(json, "\"generated_file\"");
    }

    #[test]
    fn disposition_serialize_suppressed() {
        let json = serde_json::to_string(&Disposition::Suppressed).unwrap();
        assert_eq!(json, "\"suppressed\"");
    }

    #[test]
    fn disposition_serialize_no_span() {
        let json = serde_json::to_string(&Disposition::NoSpan).unwrap();
        assert_eq!(json, "\"no_span\"");
    }

    #[test]
    fn disposition_serialize_non_workspace() {
        let json = serde_json::to_string(&Disposition::NonWorkspace).unwrap();
        assert_eq!(json, "\"non_workspace\"");
    }

    #[test]
    fn disposition_deserialize_included() {
        let parsed: Disposition = serde_json::from_str("\"included\"").unwrap();
        assert_eq!(parsed, Disposition::Included);
    }

    #[test]
    fn disposition_deserialize_outside_diff() {
        let parsed: Disposition = serde_json::from_str("\"outside_diff\"").unwrap();
        assert_eq!(parsed, Disposition::OutsideDiff);
    }

    #[test]
    fn disposition_roundtrip() {
        for disposition in [
            Disposition::Included,
            Disposition::OutsideDiff,
            Disposition::GeneratedFile,
            Disposition::Suppressed,
            Disposition::NoSpan,
            Disposition::NonWorkspace,
        ] {
            let json = serde_json::to_string(&disposition).unwrap();
            let parsed: Disposition = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, disposition);
        }
    }

    #[test]
    fn explanation_serialize_basic() {
        let explanation = Explanation::included("Test reason");
        let json = serde_json::to_string(&explanation).unwrap();
        assert!(json.contains("\"disposition\":\"included\""));
        assert!(json.contains("\"reason\":\"Test reason\""));
        assert!(json.contains("\"code\":null"));
    }

    #[test]
    fn explanation_serialize_with_code() {
        let explanation = Explanation::suppressed("dead_code");
        let json = serde_json::to_string(&explanation).unwrap();
        assert!(json.contains("\"disposition\":\"suppressed\""));
        assert!(json.contains("\"code\":\"dead_code\""));
    }

    #[test]
    fn explanation_deserialize() {
        let json = r#"{"disposition":"included","reason":"Test","code":null}"#;
        let parsed: Explanation = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.disposition, Disposition::Included);
        assert_eq!(parsed.reason, "Test");
        assert_eq!(parsed.code, None);
    }

    #[test]
    fn explanation_deserialize_with_code() {
        let json = r#"{"disposition":"suppressed","reason":"Test","code":"dead_code"}"#;
        let parsed: Explanation = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.disposition, Disposition::Suppressed);
        assert_eq!(parsed.code, Some("dead_code".to_string()));
    }

    #[test]
    fn explanation_roundtrip() {
        let original = Explanation::suppressed("clippy::all");
        let json = serde_json::to_string(&original).unwrap();
        let parsed: Explanation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, original);
    }
}

// ============================================================================
// Explainable Trait Tests
// ============================================================================

struct TestDiagnostic {
    code: String,
    included: bool,
}

impl Explainable for TestDiagnostic {
    fn explain(&self) -> Explanation {
        if self.included {
            Explanation::included(format!("Diagnostic {} matched", self.code))
        } else {
            Explanation::suppressed(&self.code)
        }
    }
}

#[test]
fn explainable_trait_for_included_diagnostic() {
    let diag = TestDiagnostic {
        code: "dead_code".into(),
        included: true,
    };
    let explanation = diag.explain();
    assert_eq!(explanation.disposition, Disposition::Included);
    assert!(explanation.is_included());
}

#[test]
fn explainable_trait_for_excluded_diagnostic() {
    let diag = TestDiagnostic {
        code: "unused_variable".into(),
        included: false,
    };
    let explanation = diag.explain();
    assert_eq!(explanation.disposition, Disposition::Suppressed);
    assert!(!explanation.is_included());
    assert_eq!(explanation.code, Some("unused_variable".to_string()));
}

// ============================================================================
// Debug Trait Tests
// ============================================================================

#[test]
fn disposition_debug() {
    let d = Disposition::Included;
    let debug = format!("{:?}", d);
    assert!(debug.contains("Included"));
}

#[test]
fn explanation_debug() {
    let e = Explanation::included("Test");
    let debug = format!("{:?}", e);
    assert!(debug.contains("Included"));
    assert!(debug.contains("Test"));
}
