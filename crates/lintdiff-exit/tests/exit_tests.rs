//! Integration tests for the lintdiff-exit crate.

use lintdiff_exit::ExitCode;
use lintdiff_types::{Counts, FailOn, Verdict, VerdictStatus};

fn make_verdict(status: VerdictStatus) -> Verdict {
    Verdict {
        status,
        counts: Counts::default(),
        reasons: vec![],
    }
}

fn make_verdict_with_counts(status: VerdictStatus, errors: u32, warns: u32, infos: u32) -> Verdict {
    Verdict {
        status,
        counts: Counts {
            error: errors,
            warn: warns,
            info: infos,
        },
        reasons: vec![],
    }
}

fn make_verdict_with_reasons(status: VerdictStatus, reasons: Vec<&str>) -> Verdict {
    Verdict {
        status,
        counts: Counts::default(),
        reasons: reasons.iter().map(|s| s.to_string()).collect(),
    }
}

mod exit_code_values {
    use super::*;

    #[test]
    fn success_is_zero() {
        assert_eq!(ExitCode::Success.as_i32(), 0);
    }

    #[test]
    fn tool_error_is_one() {
        assert_eq!(ExitCode::ToolError.as_i32(), 1);
    }

    #[test]
    fn policy_failure_is_two() {
        assert_eq!(ExitCode::PolicyFailure.as_i32(), 2);
    }

    #[test]
    fn repr_i32_matches_values() {
        // Verify that the #[repr(i32)] attribute works correctly
        assert_eq!(ExitCode::Success as i32, 0);
        assert_eq!(ExitCode::ToolError as i32, 1);
        assert_eq!(ExitCode::PolicyFailure as i32, 2);
    }
}

mod from_verdict {
    use super::*;

    mod pass_verdict {
        use super::*;

        #[test]
        fn returns_success_with_fail_on_error() {
            let verdict = make_verdict(VerdictStatus::Pass);
            assert_eq!(
                ExitCode::from_verdict(&verdict, FailOn::Error),
                ExitCode::Success
            );
        }

        #[test]
        fn returns_success_with_fail_on_warn() {
            let verdict = make_verdict(VerdictStatus::Pass);
            assert_eq!(
                ExitCode::from_verdict(&verdict, FailOn::Warn),
                ExitCode::Success
            );
        }

        #[test]
        fn returns_success_with_fail_on_never() {
            let verdict = make_verdict(VerdictStatus::Pass);
            assert_eq!(
                ExitCode::from_verdict(&verdict, FailOn::Never),
                ExitCode::Success
            );
        }

        #[test]
        fn returns_success_with_counts() {
            let verdict = make_verdict_with_counts(VerdictStatus::Pass, 0, 0, 5);
            assert_eq!(
                ExitCode::from_verdict(&verdict, FailOn::Error),
                ExitCode::Success
            );
        }
    }

    mod warn_verdict {
        use super::*;

        #[test]
        fn returns_success_with_fail_on_error() {
            let verdict = make_verdict(VerdictStatus::Warn);
            assert_eq!(
                ExitCode::from_verdict(&verdict, FailOn::Error),
                ExitCode::Success
            );
        }

        #[test]
        fn returns_success_with_fail_on_warn() {
            let verdict = make_verdict(VerdictStatus::Warn);
            assert_eq!(
                ExitCode::from_verdict(&verdict, FailOn::Warn),
                ExitCode::Success
            );
        }

        #[test]
        fn returns_success_with_fail_on_never() {
            let verdict = make_verdict(VerdictStatus::Warn);
            assert_eq!(
                ExitCode::from_verdict(&verdict, FailOn::Never),
                ExitCode::Success
            );
        }

        #[test]
        fn returns_success_with_counts() {
            let verdict = make_verdict_with_counts(VerdictStatus::Warn, 0, 3, 2);
            assert_eq!(
                ExitCode::from_verdict(&verdict, FailOn::Error),
                ExitCode::Success
            );
        }

        #[test]
        fn returns_success_with_reasons() {
            let verdict =
                make_verdict_with_reasons(VerdictStatus::Warn, vec!["Found 3 warnings"]);
            assert_eq!(
                ExitCode::from_verdict(&verdict, FailOn::Error),
                ExitCode::Success
            );
        }
    }

    mod fail_verdict {
        use super::*;

        #[test]
        fn returns_policy_failure_with_fail_on_error() {
            let verdict = make_verdict(VerdictStatus::Fail);
            assert_eq!(
                ExitCode::from_verdict(&verdict, FailOn::Error),
                ExitCode::PolicyFailure
            );
        }

        #[test]
        fn returns_policy_failure_with_fail_on_warn() {
            let verdict = make_verdict(VerdictStatus::Fail);
            assert_eq!(
                ExitCode::from_verdict(&verdict, FailOn::Warn),
                ExitCode::PolicyFailure
            );
        }

        #[test]
        fn returns_policy_failure_with_fail_on_never() {
            let verdict = make_verdict(VerdictStatus::Fail);
            assert_eq!(
                ExitCode::from_verdict(&verdict, FailOn::Never),
                ExitCode::PolicyFailure
            );
        }

        #[test]
        fn returns_policy_failure_with_counts() {
            let verdict = make_verdict_with_counts(VerdictStatus::Fail, 2, 5, 1);
            assert_eq!(
                ExitCode::from_verdict(&verdict, FailOn::Error),
                ExitCode::PolicyFailure
            );
        }

        #[test]
        fn returns_policy_failure_with_reasons() {
            let verdict = make_verdict_with_reasons(
                VerdictStatus::Fail,
                vec!["Found 2 errors", "Exceeds threshold"],
            );
            assert_eq!(
                ExitCode::from_verdict(&verdict, FailOn::Error),
                ExitCode::PolicyFailure
            );
        }
    }

    mod skip_verdict {
        use super::*;

        #[test]
        fn returns_success_with_fail_on_error() {
            let verdict = make_verdict(VerdictStatus::Skip);
            assert_eq!(
                ExitCode::from_verdict(&verdict, FailOn::Error),
                ExitCode::Success
            );
        }

        #[test]
        fn returns_success_with_fail_on_warn() {
            let verdict = make_verdict(VerdictStatus::Skip);
            assert_eq!(
                ExitCode::from_verdict(&verdict, FailOn::Warn),
                ExitCode::Success
            );
        }

        #[test]
        fn returns_success_with_fail_on_never() {
            let verdict = make_verdict(VerdictStatus::Skip);
            assert_eq!(
                ExitCode::from_verdict(&verdict, FailOn::Never),
                ExitCode::Success
            );
        }
    }
}

mod from_trait {
    use super::*;

    #[test]
    fn success_converts_to_zero() {
        let code: i32 = ExitCode::Success.into();
        assert_eq!(code, 0);
    }

    #[test]
    fn tool_error_converts_to_one() {
        let code: i32 = ExitCode::ToolError.into();
        assert_eq!(code, 1);
    }

    #[test]
    fn policy_failure_converts_to_two() {
        let code: i32 = ExitCode::PolicyFailure.into();
        assert_eq!(code, 2);
    }

    #[test]
    fn from_trait_matches_as_i32() {
        assert_eq!(i32::from(ExitCode::Success), ExitCode::Success.as_i32());
        assert_eq!(i32::from(ExitCode::ToolError), ExitCode::ToolError.as_i32());
        assert_eq!(
            i32::from(ExitCode::PolicyFailure),
            ExitCode::PolicyFailure.as_i32()
        );
    }
}

mod trait_impls {
    use super::*;

    #[test]
    fn debug_impl_works() {
        assert!(format!("{:?}", ExitCode::Success).contains("Success"));
        assert!(format!("{:?}", ExitCode::ToolError).contains("ToolError"));
        assert!(format!("{:?}", ExitCode::PolicyFailure).contains("PolicyFailure"));
    }

    #[test]
    fn clone_impl_works() {
        let original = ExitCode::PolicyFailure;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn copy_impl_works() {
        let original = ExitCode::ToolError;
        let copied = original; // Copy semantics
        assert_eq!(original, copied);
    }

    #[test]
    fn partial_eq_impl_works() {
        assert_eq!(ExitCode::Success, ExitCode::Success);
        assert_eq!(ExitCode::ToolError, ExitCode::ToolError);
        assert_eq!(ExitCode::PolicyFailure, ExitCode::PolicyFailure);

        assert_ne!(ExitCode::Success, ExitCode::ToolError);
        assert_ne!(ExitCode::Success, ExitCode::PolicyFailure);
        assert_ne!(ExitCode::ToolError, ExitCode::PolicyFailure);
    }
}

mod edge_cases {
    use super::*;

    #[test]
    fn verdict_with_empty_reasons() {
        let verdict = Verdict {
            status: VerdictStatus::Fail,
            counts: Counts::default(),
            reasons: vec![],
        };
        assert_eq!(
            ExitCode::from_verdict(&verdict, FailOn::Error),
            ExitCode::PolicyFailure
        );
    }

    #[test]
    fn verdict_with_many_reasons() {
        let verdict = Verdict {
            status: VerdictStatus::Fail,
            counts: Counts::default(),
            reasons: vec![
                "Reason 1".to_string(),
                "Reason 2".to_string(),
                "Reason 3".to_string(),
            ],
        };
        assert_eq!(
            ExitCode::from_verdict(&verdict, FailOn::Error),
            ExitCode::PolicyFailure
        );
    }

    #[test]
    fn verdict_with_high_counts() {
        let verdict = Verdict {
            status: VerdictStatus::Fail,
            counts: Counts {
                error: u32::MAX,
                warn: u32::MAX,
                info: u32::MAX,
            },
            reasons: vec![],
        };
        assert_eq!(
            ExitCode::from_verdict(&verdict, FailOn::Error),
            ExitCode::PolicyFailure
        );
    }

    #[test]
    fn all_verdict_statuses_covered() {
        // This test ensures all verdict statuses are handled
        let statuses = [
            (VerdictStatus::Pass, ExitCode::Success),
            (VerdictStatus::Warn, ExitCode::Success),
            (VerdictStatus::Fail, ExitCode::PolicyFailure),
            (VerdictStatus::Skip, ExitCode::Success),
        ];

        for (status, expected) in statuses {
            let verdict = make_verdict(status);
            let result = ExitCode::from_verdict(&verdict, FailOn::Error);
            assert_eq!(
                result, expected,
                "VerdictStatus::{:?} should map to {:?}",
                status, expected
            );
        }
    }

    #[test]
    fn all_fail_on_values_covered() {
        // This test ensures all fail_on values are accepted
        let fail_ons = [FailOn::Error, FailOn::Warn, FailOn::Never];
        let verdict = make_verdict(VerdictStatus::Pass);

        for fail_on in &fail_ons {
            let result = ExitCode::from_verdict(&verdict, fail_on.clone());
            assert_eq!(
                result,
                ExitCode::Success,
                "Pass verdict should always return Success with fail_on {:?}",
                fail_on
            );
        }
    }
}
