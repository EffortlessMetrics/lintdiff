//! Comprehensive BDD tests for lintdiff-git-info crate.
//!
//! Test coverage:
//! 1. GitSha creation and validation (10 tests)
//! 2. GitSha short SHA generation (6 tests)
//! 3. GitSha zero detection (4 tests)
//! 4. GitSha from bytes (3 tests)
//! 5. GitRef parsing (8 tests)
//! 6. GitRef type checking (6 tests)
//! 7. GitInfo construction (8 tests)
//! 8. Parse functions (5 tests)
//! 9. Edge cases (6 tests)
//! 10. Property-based tests with proptest (4 tests)

use lintdiff_git_info::{
    is_valid_sha, parse_ref, parse_sha, GitInfo, GitInfoError, GitRef, GitSha,
};

// =============================================================================
// 1. GitSha creation and validation (10 tests)
// =============================================================================

mod git_sha_creation {
    use super::*;

    #[test]
    fn git_sha_new_with_valid_lowercase_hex() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        assert_eq!(sha.as_str(), "0123456789abcdef0123456789abcdef01234567");
    }

    #[test]
    fn git_sha_new_with_valid_uppercase_hex_is_normalized() {
        let sha = GitSha::new("0123456789ABCDEF0123456789ABCDEF01234567").unwrap();
        assert_eq!(sha.as_str(), "0123456789abcdef0123456789abcdef01234567");
    }

    #[test]
    fn git_sha_new_with_mixed_case_is_normalized() {
        let sha = GitSha::new("AaBbCcDdEeFf00112233445566778899AaBbCcDd").unwrap();
        assert_eq!(sha.as_str(), "aabbccddeeff00112233445566778899aabbccdd");
    }

    #[test]
    fn git_sha_new_with_all_zeros() {
        let sha = GitSha::new("0000000000000000000000000000000000000000").unwrap();
        assert_eq!(sha.as_str(), "0000000000000000000000000000000000000000");
    }

    #[test]
    fn git_sha_new_with_all_f() {
        let sha = GitSha::new("ffffffffffffffffffffffffffffffffffffffff").unwrap();
        assert_eq!(sha.as_str(), "ffffffffffffffffffffffffffffffffffffffff");
    }

    #[test]
    fn git_sha_new_rejects_too_short() {
        let result = GitSha::new("abc");
        assert!(result.is_err());
        match result {
            Err(GitInfoError::InvalidShaLength { expected, actual }) => {
                assert_eq!(expected, 40);
                assert_eq!(actual, 3);
            }
            _ => panic!("Expected InvalidShaLength error"),
        }
    }

    #[test]
    fn git_sha_new_rejects_too_long() {
        let result = GitSha::new("0123456789abcdef0123456789abcdef012345678");
        assert!(result.is_err());
        match result {
            Err(GitInfoError::InvalidShaLength { expected, actual }) => {
                assert_eq!(expected, 40);
                assert_eq!(actual, 41);
            }
            _ => panic!("Expected InvalidShaLength error"),
        }
    }

    #[test]
    fn git_sha_new_rejects_invalid_hex_character() {
        let result = GitSha::new("0123456789ghijef0123456789abcdef01234567");
        assert!(result.is_err());
        match result {
            Err(GitInfoError::InvalidHexCharacter { position, character }) => {
                assert_eq!(position, 10);
                assert_eq!(character, 'g');
            }
            _ => panic!("Expected InvalidHexCharacter error"),
        }
    }

    #[test]
    fn git_sha_new_rejects_empty_string() {
        let result = GitSha::new("");
        assert!(result.is_err());
    }

    #[test]
    fn git_sha_from_str_trait() {
        use std::str::FromStr;
        let sha = GitSha::from_str("0123456789abcdef0123456789abcdef01234567").unwrap();
        assert_eq!(sha.as_str(), "0123456789abcdef0123456789abcdef01234567");
    }
}

// =============================================================================
// 2. GitSha short SHA generation (6 tests)
// =============================================================================

mod git_sha_short {
    use super::*;

    #[test]
    fn short_sha_default_length() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        assert_eq!(sha.short_default(), "0123456");
    }

    #[test]
    fn short_sha_custom_length() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        assert_eq!(sha.short(8), "01234567");
    }

    #[test]
    fn short_sha_length_zero() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        assert_eq!(sha.short(0), "");
    }

    #[test]
    fn short_sha_length_exceeds_full_length() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        assert_eq!(sha.short(50), "0123456789abcdef0123456789abcdef01234567");
    }

    #[test]
    fn short_sha_full_length() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        assert_eq!(sha.short(40), "0123456789abcdef0123456789abcdef01234567");
    }

    #[test]
    fn short_sha_one_character() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        assert_eq!(sha.short(1), "0");
    }
}

// =============================================================================
// 3. GitSha zero detection (4 tests)
// =============================================================================

mod git_sha_zero {
    use super::*;

    #[test]
    fn is_zero_returns_true_for_all_zeros() {
        let sha = GitSha::new("0000000000000000000000000000000000000000").unwrap();
        assert!(sha.is_zero());
    }

    #[test]
    fn is_zero_returns_false_for_non_zero() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        assert!(!sha.is_zero());
    }

    #[test]
    fn zero_factory_creates_zero_sha() {
        let sha = GitSha::zero();
        assert!(sha.is_zero());
    }

    #[test]
    fn default_is_zero_sha() {
        let sha = GitSha::default();
        assert!(sha.is_zero());
    }
}

// =============================================================================
// 4. GitSha from bytes (3 tests)
// =============================================================================

mod git_sha_from_bytes {
    use super::*;

    #[test]
    fn from_bytes_zero_array() {
        let bytes = [0u8; 20];
        let sha = GitSha::from_bytes(bytes);
        assert!(sha.is_zero());
    }

    #[test]
    fn from_bytes_max_array() {
        let bytes = [0xFFu8; 20];
        let sha = GitSha::from_bytes(bytes);
        assert_eq!(sha.as_str(), "ffffffffffffffffffffffffffffffffffffffff");
    }

    #[test]
    fn from_bytes_mixed_array() {
        let mut bytes = [0u8; 20];
        bytes[0] = 0x01;
        bytes[1] = 0x23;
        bytes[2] = 0x45;
        bytes[3] = 0x67;
        bytes[4] = 0x89;
        bytes[5] = 0xAB;
        bytes[6] = 0xCD;
        bytes[7] = 0xEF;
        let sha = GitSha::from_bytes(bytes);
        assert!(sha.as_str().starts_with("0123456789abcdef"));
    }
}

// =============================================================================
// 5. GitRef parsing (8 tests)
// =============================================================================

mod git_ref_parsing {
    use super::*;

    #[test]
    fn parse_ref_branch() {
        let git_ref = parse_ref("refs/heads/main");
        assert!(git_ref.is_branch());
        assert_eq!(git_ref.as_branch(), Some("main"));
    }

    #[test]
    fn parse_ref_branch_with_slashes() {
        let git_ref = parse_ref("refs/heads/feature/my-feature");
        assert!(git_ref.is_branch());
        assert_eq!(git_ref.as_branch(), Some("feature/my-feature"));
    }

    #[test]
    fn parse_ref_tag() {
        let git_ref = parse_ref("refs/tags/v1.0.0");
        assert!(git_ref.is_tag());
        assert_eq!(git_ref.as_tag(), Some("v1.0.0"));
    }

    #[test]
    fn parse_ref_head() {
        let git_ref = parse_ref("HEAD");
        assert!(git_ref.is_head());
    }

    #[test]
    fn parse_ref_commit_sha() {
        let git_ref = parse_ref("0123456789abcdef0123456789abcdef01234567");
        assert!(git_ref.as_commit().is_some());
    }

    #[test]
    fn parse_ref_unknown_string() {
        let git_ref = parse_ref("some-random-string");
        assert_eq!(git_ref, GitRef::Unknown);
    }

    #[test]
    fn parse_ref_with_whitespace() {
        let git_ref = parse_ref("  refs/heads/main  ");
        assert!(git_ref.is_branch());
    }

    #[test]
    fn parse_ref_empty_string() {
        let git_ref = parse_ref("");
        assert_eq!(git_ref, GitRef::Unknown);
    }
}

// =============================================================================
// 6. GitRef type checking (6 tests)
// =============================================================================

mod git_ref_type_checking {
    use super::*;

    #[test]
    fn git_ref_is_branch() {
        assert!(GitRef::Branch("main".to_string()).is_branch());
        assert!(!GitRef::Tag("v1.0.0".to_string()).is_branch());
        assert!(!GitRef::Head.is_branch());
    }

    #[test]
    fn git_ref_is_tag() {
        assert!(GitRef::Tag("v1.0.0".to_string()).is_tag());
        assert!(!GitRef::Branch("main".to_string()).is_tag());
        assert!(!GitRef::Head.is_tag());
    }

    #[test]
    fn git_ref_is_head() {
        assert!(GitRef::Head.is_head());
        assert!(!GitRef::Branch("main".to_string()).is_head());
        assert!(!GitRef::Tag("v1.0.0".to_string()).is_head());
    }

    #[test]
    fn git_ref_as_branch() {
        let git_ref = GitRef::Branch("main".to_string());
        assert_eq!(git_ref.as_branch(), Some("main"));
        assert_eq!(GitRef::Head.as_branch(), None);
    }

    #[test]
    fn git_ref_as_tag() {
        let git_ref = GitRef::Tag("v1.0.0".to_string());
        assert_eq!(git_ref.as_tag(), Some("v1.0.0"));
        assert_eq!(GitRef::Head.as_tag(), None);
    }

    #[test]
    fn git_ref_name() {
        assert_eq!(GitRef::Branch("main".to_string()).name(), Some("main"));
        assert_eq!(GitRef::Tag("v1.0.0".to_string()).name(), Some("v1.0.0"));
        assert_eq!(GitRef::Head.name(), Some("HEAD"));
        assert_eq!(GitRef::Unknown.name(), None);
    }
}

// =============================================================================
// 7. GitInfo construction (8 tests)
// =============================================================================

mod git_info_construction {
    use super::*;

    #[test]
    fn git_info_new() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        let info = GitInfo::new(sha.clone());
        assert_eq!(info.sha, sha);
        assert_eq!(info.ref_name, None);
        assert!(!info.is_dirty);
        assert_eq!(info.message, None);
    }

    #[test]
    fn git_info_with_ref_name() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        let info = GitInfo::new(sha).with_ref_name("main".to_string());
        assert_eq!(info.ref_name, Some("main".to_string()));
    }

    #[test]
    fn git_info_with_dirty() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        let info = GitInfo::new(sha).with_dirty(true);
        assert!(info.is_dirty);
    }

    #[test]
    fn git_info_with_message() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        let info = GitInfo::new(sha).with_message("Initial commit".to_string());
        assert_eq!(info.message, Some("Initial commit".to_string()));
    }

    #[test]
    fn git_info_builder_chain() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        let info = GitInfo::new(sha.clone())
            .with_ref_name("develop".to_string())
            .with_dirty(true)
            .with_message("WIP".to_string());
        assert_eq!(info.sha, sha);
        assert_eq!(info.ref_name, Some("develop".to_string()));
        assert!(info.is_dirty);
        assert_eq!(info.message, Some("WIP".to_string()));
    }

    #[test]
    fn git_info_is_clean() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        let clean = GitInfo::new(sha.clone()).with_dirty(false);
        let dirty = GitInfo::new(sha).with_dirty(true);
        assert!(clean.is_clean());
        assert!(!dirty.is_clean());
    }

    #[test]
    fn git_info_short_sha() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        let info = GitInfo::new(sha);
        assert_eq!(info.short_sha(), "0123456");
    }

    #[test]
    fn git_info_empty() {
        let info = GitInfo::empty();
        assert!(info.sha.is_zero());
        assert_eq!(info.ref_name, None);
        assert!(!info.is_dirty);
        assert_eq!(info.message, None);
    }
}

// =============================================================================
// 8. Parse functions (5 tests)
// =============================================================================

mod parse_functions {
    use super::*;

    #[test]
    fn parse_sha_valid() {
        let sha = parse_sha("0123456789abcdef0123456789abcdef01234567").unwrap();
        assert_eq!(sha.as_str(), "0123456789abcdef0123456789abcdef01234567");
    }

    #[test]
    fn parse_sha_invalid() {
        let result = parse_sha("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn is_valid_sha_returns_true_for_valid() {
        assert!(is_valid_sha("0123456789abcdef0123456789abcdef01234567"));
        assert!(is_valid_sha("0000000000000000000000000000000000000000"));
    }

    #[test]
    fn is_valid_sha_returns_false_for_invalid() {
        assert!(!is_valid_sha("invalid"));
        assert!(!is_valid_sha("0123456789abcdef")); // Too short
        assert!(!is_valid_sha(""));
    }

    #[test]
    fn parse_sha_error_message() {
        let result = parse_sha("abc");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid SHA"));
    }
}

// =============================================================================
// 9. Edge cases (6 tests)
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn git_sha_display() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        assert_eq!(format!("{sha}"), "0123456789abcdef0123456789abcdef01234567");
    }

    #[test]
    fn git_sha_as_ref() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        let s: &str = sha.as_ref();
        assert_eq!(s, "0123456789abcdef0123456789abcdef01234567");
    }

    #[test]
    fn git_ref_display() {
        assert_eq!(format!("{}", GitRef::Branch("main".to_string())), "refs/heads/main");
        assert_eq!(format!("{}", GitRef::Tag("v1.0.0".to_string())), "refs/tags/v1.0.0");
        assert_eq!(format!("{}", GitRef::Head), "HEAD");
        assert_eq!(format!("{}", GitRef::Unknown), "(unknown)");
    }

    #[test]
    fn git_ref_default() {
        assert_eq!(GitRef::default(), GitRef::Unknown);
    }

    #[test]
    fn git_info_default() {
        let info = GitInfo::default();
        assert!(info.sha.is_zero());
    }

    #[test]
    fn git_sha_equality() {
        let sha1 = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        let sha2 = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        let sha3 = GitSha::new("ffffffffffffffffffffffffffffffffffffffff").unwrap();
        
        assert_eq!(sha1, sha2);
        assert_ne!(sha1, sha3);
    }
}

// =============================================================================
// 10. Property-based tests with proptest (4 tests)
// =============================================================================

#[cfg(test)]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    prop_compose! {
        fn valid_sha_string()(s in "[0-9a-fA-F]{40}") -> String {
            s
        }
    }

    proptest! {
        #[test]
        fn proptest_valid_sha_accepts_all_hex(s in valid_sha_string()) {
            prop_assert!(is_valid_sha(&s));
        }

        #[test]
        fn proptest_sha_normalizes_to_lowercase(s in valid_sha_string()) {
            let sha = GitSha::new(&s).unwrap();
            prop_assert_eq!(sha.as_str(), s.to_lowercase());
        }

        #[test]
        fn proptest_short_sha_never_panics(s in valid_sha_string(), len in 0usize..100) {
            let sha = GitSha::new(&s).unwrap();
            let short = sha.short(len);
            prop_assert!(short.len() <= 40);
            prop_assert!(short.len() == len.min(40));
        }

        #[test]
        fn proptest_sha_roundtrip(s in valid_sha_string()) {
            let sha1 = GitSha::new(&s).unwrap();
            let sha_str = sha1.as_str().to_string();
            let sha2 = GitSha::new(&sha_str).unwrap();
            prop_assert_eq!(sha1, sha2);
        }
    }
}

// =============================================================================
// Additional tests for serde feature (conditional)
// =============================================================================

#[cfg(feature = "serde")]
mod serde_tests {
    use super::*;

    #[test]
    fn git_sha_serialize_deserialize() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        let json = serde_json::to_string(&sha).unwrap();
        assert_eq!(json, "\"0123456789abcdef0123456789abcdef01234567\"");
        
        let deserialized: GitSha = serde_json::from_str(&json).unwrap();
        assert_eq!(sha, deserialized);
    }

    #[test]
    fn git_ref_serialize_branch() {
        let git_ref = GitRef::Branch("main".to_string());
        let json = serde_json::to_string(&git_ref).unwrap();
        assert!(json.contains("Branch"));
        assert!(json.contains("main"));
    }

    #[test]
    fn git_info_serialize() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        let info = GitInfo::new(sha)
            .with_ref_name("main".to_string())
            .with_dirty(false);
        
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("0123456789abcdef0123456789abcdef01234567"));
        assert!(json.contains("main"));
    }
}

// =============================================================================
// Error type tests
// =============================================================================

mod error_tests {
    use super::*;

    #[test]
    fn invalid_sha_error_display() {
        let err = GitInfoError::InvalidSha("bad".to_string());
        assert!(err.to_string().contains("Invalid SHA"));
    }

    #[test]
    fn invalid_sha_length_error_display() {
        let err = GitInfoError::InvalidShaLength {
            expected: 40,
            actual: 10,
        };
        let msg = err.to_string();
        assert!(msg.contains("expected 40"));
        assert!(msg.contains("got 10"));
    }

    #[test]
    fn invalid_hex_character_error_display() {
        let err = GitInfoError::InvalidHexCharacter {
            position: 5,
            character: 'g',
        };
        let msg = err.to_string();
        assert!(msg.contains("position 5"));
        assert!(msg.contains("'g'"));
    }

    #[test]
    fn error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<GitInfoError>();
    }
}

// =============================================================================
// Comparison and ordering tests
// =============================================================================

mod ordering_tests {
    use super::*;

    #[test]
    fn git_sha_ordering() {
        let sha1 = GitSha::new("0000000000000000000000000000000000000001").unwrap();
        let sha2 = GitSha::new("0000000000000000000000000000000000000002").unwrap();
        assert!(sha1 < sha2);
    }

    #[test]
    fn git_sha_hash_consistency() {
        use std::collections::HashSet;
        
        let sha1 = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        let sha2 = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        
        let mut set = HashSet::new();
        set.insert(sha1.clone());
        set.insert(sha2.clone());
        
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn git_ref_equality() {
        assert_eq!(GitRef::Head, GitRef::Head);
        assert_eq!(
            GitRef::Branch("main".to_string()),
            GitRef::Branch("main".to_string())
        );
        assert_ne!(
            GitRef::Branch("main".to_string()),
            GitRef::Branch("develop".to_string())
        );
    }
}

// =============================================================================
// Integration-style tests
// =============================================================================

mod integration_tests {
    use super::*;

    #[test]
    fn typical_github_actions_sha_workflow() {
        // Simulate parsing a SHA from GITHUB_SHA environment variable
        let env_sha = "0123456789abcdef0123456789abcdef01234567";
        let sha = parse_sha(env_sha).unwrap();
        
        // Simulate parsing a ref from GITHUB_REF
        let env_ref = "refs/heads/main";
        let git_ref = parse_ref(env_ref);
        
        // Build GitInfo
        let info = GitInfo::new(sha)
            .with_ref_name(git_ref.as_branch().unwrap().to_string())
            .with_dirty(false)
            .with_message("Add feature X".to_string());
        
        assert_eq!(info.short_sha(), "0123456");
        assert_eq!(info.ref_name, Some("main".to_string()));
        assert!(info.is_clean());
    }

    #[test]
    fn typical_tag_release_workflow() {
        let env_sha = "0123456789abcdef0123456789abcdef01234567";
        let env_ref = "refs/tags/v1.0.0";
        
        let sha = parse_sha(env_sha).unwrap();
        let git_ref = parse_ref(env_ref);
        
        let info = GitInfo::new(sha)
            .with_ref_name(git_ref.as_tag().unwrap().to_string());
        
        assert_eq!(info.ref_name, Some("v1.0.0".to_string()));
    }

    #[test]
    fn detached_head_workflow() {
        let env_sha = "0123456789abcdef0123456789abcdef01234567";
        let sha = parse_sha(env_sha).unwrap();
        
        // Detached HEAD - no ref name
        let info = GitInfo::new(sha);
        
        assert!(info.ref_name.is_none());
    }

    #[test]
    fn dirty_working_directory() {
        let sha = GitSha::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        let info = GitInfo::new(sha).with_dirty(true);
        
        assert!(!info.is_clean());
        assert!(info.is_dirty);
    }
}
