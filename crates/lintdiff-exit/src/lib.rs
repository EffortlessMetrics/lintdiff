//! Exit code classification and determination for lintdiff.
//!
//! This microcrate provides a single responsibility: determining the appropriate
//! exit code based on a verdict and fail-on policy.
//!
//! # Exit Codes
//!
//! | Code | Variant | Meaning |
//! |------|---------|---------|
//! | 0 | Success | Pass or warn verdict (no policy failure) |
//! | 1 | ToolError | Runtime error (invalid input, parse error, etc.) |
//! | 2 | PolicyFailure | Fail verdict (blocking issues found) |
//!
//! # Example
//!
//! ```
//! use lintdiff_exit::ExitCode;
//! use lintdiff_types::{Verdict, VerdictStatus, Counts, FailOn};
//!
//! let verdict = Verdict {
//!     status: VerdictStatus::Pass,
//!     counts: Counts::default(),
//!     reasons: vec![],
//! };
//!
//! let exit_code = ExitCode::from_verdict(&verdict, FailOn::Error);
//! assert_eq!(exit_code, ExitCode::Success);
//! assert_eq!(exit_code.as_i32(), 0);
//! ```

#![warn(missing_docs)]

use lintdiff_types::{FailOn, Verdict, VerdictStatus};

/// Exit codes for the lintdiff CLI.
///
/// These exit codes follow Unix conventions where 0 indicates success and
/// non-zero values indicate various types of failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ExitCode {
    /// Success (pass or warn verdict).
    ///
    /// The lintdiff run completed successfully without triggering a policy failure.
    /// This includes both clean passes and warnings that don't meet the fail threshold.
    Success = 0,
    /// Tool/runtime error (invalid input, parse error).
    ///
    /// Indicates a problem with the tool's operation rather than a policy failure.
    /// Examples include invalid input files, parse errors, or missing dependencies.
    ToolError = 1,
    /// Policy failure (fail verdict).
    ///
    /// The lintdiff run completed but found issues that violate the configured policy.
    /// This should cause CI/CD pipelines to fail.
    PolicyFailure = 2,
}

impl ExitCode {
    /// Determine exit code from a verdict and fail_on policy.
    ///
    /// The exit code is determined primarily by the verdict status:
    /// - `Pass`, `Warn`, `Skip` → `Success`
    /// - `Fail` → `PolicyFailure`
    ///
    /// The `fail_on` parameter is accepted for API completeness and potential
    /// future use in consistency checking. The verdict is expected to already
    /// incorporate the fail_on policy when it was computed.
    ///
    /// # Arguments
    ///
    /// * `verdict` - The verdict from a lintdiff run
    /// * `fail_on` - The fail-on policy (currently unused, reserved for future use)
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_exit::ExitCode;
    /// use lintdiff_types::{Verdict, VerdictStatus, Counts, FailOn};
    ///
    /// let fail_verdict = Verdict {
    ///     status: VerdictStatus::Fail,
    ///     counts: Counts::default(),
    ///     reasons: vec!["Found 2 errors".to_string()],
    /// };
    ///
    /// let exit_code = ExitCode::from_verdict(&fail_verdict, FailOn::Error);
    /// assert_eq!(exit_code, ExitCode::PolicyFailure);
    /// ```
    pub fn from_verdict(verdict: &Verdict, _fail_on: FailOn) -> Self {
        match verdict.status {
            VerdictStatus::Pass | VerdictStatus::Warn | VerdictStatus::Skip => ExitCode::Success,
            VerdictStatus::Fail => ExitCode::PolicyFailure,
        }
    }

    /// Convert to process exit code.
    ///
    /// Returns the raw `i32` value suitable for use with `std::process::exit()`.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_exit::ExitCode;
    ///
    /// assert_eq!(ExitCode::Success.as_i32(), 0);
    /// assert_eq!(ExitCode::ToolError.as_i32(), 1);
    /// assert_eq!(ExitCode::PolicyFailure.as_i32(), 2);
    /// ```
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> i32 {
        code.as_i32()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lintdiff_types::{Counts, FailOn, Verdict, VerdictStatus};

    fn make_verdict(status: VerdictStatus) -> Verdict {
        Verdict {
            status,
            counts: Counts::default(),
            reasons: vec![],
        }
    }

    #[test]
    fn test_exit_code_values() {
        assert_eq!(ExitCode::Success as i32, 0);
        assert_eq!(ExitCode::ToolError as i32, 1);
        assert_eq!(ExitCode::PolicyFailure as i32, 2);
    }

    #[test]
    fn test_as_i32() {
        assert_eq!(ExitCode::Success.as_i32(), 0);
        assert_eq!(ExitCode::ToolError.as_i32(), 1);
        assert_eq!(ExitCode::PolicyFailure.as_i32(), 2);
    }

    #[test]
    fn test_from_into_i32() {
        let code: i32 = ExitCode::Success.into();
        assert_eq!(code, 0);

        let code: i32 = ExitCode::ToolError.into();
        assert_eq!(code, 1);

        let code: i32 = ExitCode::PolicyFailure.into();
        assert_eq!(code, 2);
    }

    #[test]
    fn test_pass_verdict_success() {
        let verdict = make_verdict(VerdictStatus::Pass);

        assert_eq!(
            ExitCode::from_verdict(&verdict, FailOn::Error),
            ExitCode::Success
        );
        assert_eq!(
            ExitCode::from_verdict(&verdict, FailOn::Warn),
            ExitCode::Success
        );
        assert_eq!(
            ExitCode::from_verdict(&verdict, FailOn::Never),
            ExitCode::Success
        );
    }

    #[test]
    fn test_warn_verdict_success() {
        let verdict = make_verdict(VerdictStatus::Warn);

        assert_eq!(
            ExitCode::from_verdict(&verdict, FailOn::Error),
            ExitCode::Success
        );
        assert_eq!(
            ExitCode::from_verdict(&verdict, FailOn::Warn),
            ExitCode::Success
        );
        assert_eq!(
            ExitCode::from_verdict(&verdict, FailOn::Never),
            ExitCode::Success
        );
    }

    #[test]
    fn test_skip_verdict_success() {
        let verdict = make_verdict(VerdictStatus::Skip);

        assert_eq!(
            ExitCode::from_verdict(&verdict, FailOn::Error),
            ExitCode::Success
        );
        assert_eq!(
            ExitCode::from_verdict(&verdict, FailOn::Warn),
            ExitCode::Success
        );
        assert_eq!(
            ExitCode::from_verdict(&verdict, FailOn::Never),
            ExitCode::Success
        );
    }

    #[test]
    fn test_fail_verdict_policy_failure() {
        let verdict = make_verdict(VerdictStatus::Fail);

        assert_eq!(
            ExitCode::from_verdict(&verdict, FailOn::Error),
            ExitCode::PolicyFailure
        );
        assert_eq!(
            ExitCode::from_verdict(&verdict, FailOn::Warn),
            ExitCode::PolicyFailure
        );
        assert_eq!(
            ExitCode::from_verdict(&verdict, FailOn::Never),
            ExitCode::PolicyFailure
        );
    }

    #[test]
    fn test_all_verdict_fail_on_combinations() {
        let statuses = [
            VerdictStatus::Pass,
            VerdictStatus::Warn,
            VerdictStatus::Fail,
            VerdictStatus::Skip,
        ];
        let fail_ons = [FailOn::Error, FailOn::Warn, FailOn::Never];

        for status in statuses {
            let verdict = make_verdict(status);
            for fail_on in &fail_ons {
                let exit_code = ExitCode::from_verdict(&verdict, fail_on.clone());

                match status {
                    VerdictStatus::Pass | VerdictStatus::Warn | VerdictStatus::Skip => {
                        assert_eq!(
                            exit_code,
                            ExitCode::Success,
                            "Expected Success for status {:?} with fail_on {:?}",
                            status,
                            fail_on
                        );
                    }
                    VerdictStatus::Fail => {
                        assert_eq!(
                            exit_code,
                            ExitCode::PolicyFailure,
                            "Expected PolicyFailure for status {:?} with fail_on {:?}",
                            status,
                            fail_on
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_debug_impl() {
        assert!(format!("{:?}", ExitCode::Success).contains("Success"));
        assert!(format!("{:?}", ExitCode::ToolError).contains("ToolError"));
        assert!(format!("{:?}", ExitCode::PolicyFailure).contains("PolicyFailure"));
    }

    #[test]
    fn test_clone_impl() {
        let code = ExitCode::Success;
        let cloned = code.clone();
        assert_eq!(code, cloned);
    }

    #[test]
    fn test_copy_impl() {
        let code = ExitCode::Success;
        let copied: ExitCode = code;
        assert_eq!(code, copied);
    }

    #[test]
    fn test_eq_impl() {
        assert_eq!(ExitCode::Success, ExitCode::Success);
        assert_ne!(ExitCode::Success, ExitCode::ToolError);
        assert_ne!(ExitCode::Success, ExitCode::PolicyFailure);
        assert_ne!(ExitCode::ToolError, ExitCode::PolicyFailure);
    }
}
