//! Diagnostic disposition tracking for lintdiff.
//!
//! This microcrate provides a single responsibility: explaining why each diagnostic
//! was included, filtered, or suppressed in the lintdiff results.
//!
//! # Dispositions
//!
//! Every diagnostic processed by lintdiff receives a disposition that explains
//! why it was or wasn't included in the final findings:
//!
//! | Disposition | Meaning |
//! |-------------|---------|
//! | `Included` | Diagnostic matched a changed line |
//! | `OutsideDiff` | Diagnostic was outside the diff |
//! | `GeneratedFile` | Diagnostic was in a generated file |
//! | `Suppressed` | Diagnostic was suppressed by config |
//! | `NoSpan` | Diagnostic had no span information |
//! | `NonWorkspace` | Diagnostic was in a non-workspace file |
//!
//! # Example
//!
//! ```
//! use lintdiff_explain::{Disposition, Explanation};
//!
//! // Create an included explanation
//! let included = Explanation::included("Warning on line 42");
//! assert!(included.is_included());
//!
//! // Create a suppressed explanation
//! let suppressed = Explanation::suppressed("dead_code");
//! assert!(!suppressed.is_included());
//!
//! // Check the disposition
//! let outside = Explanation::outside_diff();
//! assert_eq!(outside.disposition, Disposition::OutsideDiff);
//! ```

#![warn(missing_docs)]

use serde::{Deserialize, Serialize};

/// Why a diagnostic was or wasn't included in findings.
///
/// This enum captures all possible dispositions for a diagnostic,
/// explaining the filtering decision made by lintdiff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    /// Diagnostic matched a changed line and was included in findings.
    Included,
    /// Diagnostic was outside the diff hunks.
    OutsideDiff,
    /// Diagnostic was in a generated file.
    GeneratedFile,
    /// Diagnostic was suppressed by configuration (e.g., suppress_code).
    Suppressed,
    /// Diagnostic had no span information.
    NoSpan,
    /// Diagnostic was in a file outside the workspace.
    NonWorkspace,
}

impl Disposition {
    /// Returns `true` if this disposition means the diagnostic was included.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_explain::Disposition;
    ///
    /// assert!(Disposition::Included.is_included());
    /// assert!(!Disposition::OutsideDiff.is_included());
    /// assert!(!Disposition::Suppressed.is_included());
    /// ```
    pub fn is_included(self) -> bool {
        matches!(self, Disposition::Included)
    }

    /// Returns a human-readable name for this disposition.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_explain::Disposition;
    ///
    /// assert_eq!(Disposition::Included.as_str(), "included");
    /// assert_eq!(Disposition::OutsideDiff.as_str(), "outside_diff");
    /// ```
    pub fn as_str(self) -> &'static str {
        match self {
            Disposition::Included => "included",
            Disposition::OutsideDiff => "outside_diff",
            Disposition::GeneratedFile => "generated_file",
            Disposition::Suppressed => "suppressed",
            Disposition::NoSpan => "no_span",
            Disposition::NonWorkspace => "non_workspace",
        }
    }
}

impl std::fmt::Display for Disposition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Explanation for a single diagnostic's disposition.
///
/// This struct captures why a diagnostic was or wasn't included in the
/// lintdiff findings, along with optional context like the diagnostic code
/// that was matched.
///
/// # Example
///
/// ```
/// use lintdiff_explain::{Disposition, Explanation};
///
/// let explanation = Explanation::new(
///     Disposition::Suppressed,
///     "Suppressed by configuration"
/// ).with_code("dead_code");
///
/// assert_eq!(explanation.disposition, Disposition::Suppressed);
/// assert_eq!(explanation.reason, "Suppressed by configuration");
/// assert_eq!(explanation.code, Some("dead_code".to_string()));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Explanation {
    /// The disposition (why included or not).
    pub disposition: Disposition,
    /// Human-readable reason for the disposition.
    pub reason: String,
    /// Optional code that was matched (for Suppressed).
    pub code: Option<String>,
}

impl Explanation {
    /// Create a new explanation with the given disposition and reason.
    ///
    /// # Arguments
    ///
    /// * `disposition` - Why the diagnostic was or wasn't included
    /// * `reason` - Human-readable explanation
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_explain::{Disposition, Explanation};
    ///
    /// let explanation = Explanation::new(Disposition::Included, "Matched line 42");
    /// assert_eq!(explanation.disposition, Disposition::Included);
    /// ```
    pub fn new(disposition: Disposition, reason: impl Into<String>) -> Self {
        Self {
            disposition,
            reason: reason.into(),
            code: None,
        }
    }

    /// Add a code to this explanation.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_explain::{Disposition, Explanation};
    ///
    /// let explanation = Explanation::new(Disposition::Suppressed, "Config suppression")
    ///     .with_code("clippy::all");
    ///
    /// assert_eq!(explanation.code, Some("clippy::all".to_string()));
    /// ```
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Create an `Included` explanation.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_explain::{Disposition, Explanation};
    ///
    /// let explanation = Explanation::included("Warning on changed line 42");
    /// assert_eq!(explanation.disposition, Disposition::Included);
    /// assert!(explanation.is_included());
    /// ```
    pub fn included(reason: impl Into<String>) -> Self {
        Self::new(Disposition::Included, reason)
    }

    /// Create an `OutsideDiff` explanation.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_explain::{Disposition, Explanation};
    ///
    /// let explanation = Explanation::outside_diff();
    /// assert_eq!(explanation.disposition, Disposition::OutsideDiff);
    /// assert!(!explanation.is_included());
    /// ```
    pub fn outside_diff() -> Self {
        Self::new(Disposition::OutsideDiff, "Diagnostic is outside the diff hunks")
    }

    /// Create a `GeneratedFile` explanation.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_explain::{Disposition, Explanation};
    ///
    /// let explanation = Explanation::generated_file();
    /// assert_eq!(explanation.disposition, Disposition::GeneratedFile);
    /// ```
    pub fn generated_file() -> Self {
        Self::new(Disposition::GeneratedFile, "Diagnostic is in a generated file")
    }

    /// Create a `Suppressed` explanation with the given code.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_explain::{Disposition, Explanation};
    ///
    /// let explanation = Explanation::suppressed("dead_code");
    /// assert_eq!(explanation.disposition, Disposition::Suppressed);
    /// assert_eq!(explanation.code, Some("dead_code".to_string()));
    /// ```
    pub fn suppressed(code: impl Into<String>) -> Self {
        let code = code.into();
        Self::new(Disposition::Suppressed, format!("Diagnostic '{}' was suppressed by configuration", code))
            .with_code(code)
    }

    /// Create a `NoSpan` explanation.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_explain::{Disposition, Explanation};
    ///
    /// let explanation = Explanation::no_span();
    /// assert_eq!(explanation.disposition, Disposition::NoSpan);
    /// ```
    pub fn no_span() -> Self {
        Self::new(Disposition::NoSpan, "Diagnostic has no span information")
    }

    /// Create a `NonWorkspace` explanation.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_explain::{Disposition, Explanation};
    ///
    /// let explanation = Explanation::non_workspace();
    /// assert_eq!(explanation.disposition, Disposition::NonWorkspace);
    /// ```
    pub fn non_workspace() -> Self {
        Self::new(Disposition::NonWorkspace, "Diagnostic is in a file outside the workspace")
    }

    /// Check if this explanation means the diagnostic was included.
    ///
    /// This is a convenience method that delegates to `disposition.is_included()`.
    ///
    /// # Example
    ///
    /// ```
    /// use lintdiff_explain::Explanation;
    ///
    /// let included = Explanation::included("Matched");
    /// let outside = Explanation::outside_diff();
    ///
    /// assert!(included.is_included());
    /// assert!(!outside.is_included());
    /// ```
    pub fn is_included(&self) -> bool {
        self.disposition.is_included()
    }
}

/// Trait for types that can explain their disposition.
///
/// This trait allows diagnostic types to provide their own explanation
/// for why they were or weren't included in findings.
///
/// # Example
///
/// ```
/// use lintdiff_explain::{Explainable, Explanation, Disposition};
///
/// struct MyDiagnostic {
///     code: String,
///     included: bool,
/// }
///
/// impl Explainable for MyDiagnostic {
///     fn explain(&self) -> Explanation {
///         if self.included {
///             Explanation::included(format!("Diagnostic {} matched", self.code))
///         } else {
///             Explanation::suppressed(&self.code)
///         }
///     }
/// }
///
/// let diag = MyDiagnostic { code: "dead_code".into(), included: false };
/// let explanation = diag.explain();
/// assert_eq!(explanation.disposition, Disposition::Suppressed);
/// ```
pub trait Explainable {
    /// Returns an explanation for this item's disposition.
    fn explain(&self) -> Explanation;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disposition_is_included() {
        assert!(Disposition::Included.is_included());
        assert!(!Disposition::OutsideDiff.is_included());
        assert!(!Disposition::GeneratedFile.is_included());
        assert!(!Disposition::Suppressed.is_included());
        assert!(!Disposition::NoSpan.is_included());
        assert!(!Disposition::NonWorkspace.is_included());
    }

    #[test]
    fn test_disposition_as_str() {
        assert_eq!(Disposition::Included.as_str(), "included");
        assert_eq!(Disposition::OutsideDiff.as_str(), "outside_diff");
        assert_eq!(Disposition::GeneratedFile.as_str(), "generated_file");
        assert_eq!(Disposition::Suppressed.as_str(), "suppressed");
        assert_eq!(Disposition::NoSpan.as_str(), "no_span");
        assert_eq!(Disposition::NonWorkspace.as_str(), "non_workspace");
    }

    #[test]
    fn test_disposition_display() {
        assert_eq!(format!("{}", Disposition::Included), "included");
        assert_eq!(format!("{}", Disposition::OutsideDiff), "outside_diff");
    }

    #[test]
    fn test_explanation_new() {
        let explanation = Explanation::new(Disposition::Included, "Test reason");
        assert_eq!(explanation.disposition, Disposition::Included);
        assert_eq!(explanation.reason, "Test reason");
        assert_eq!(explanation.code, None);
    }

    #[test]
    fn test_explanation_with_code() {
        let explanation = Explanation::new(Disposition::Suppressed, "Test")
            .with_code("dead_code");
        assert_eq!(explanation.code, Some("dead_code".to_string()));
    }

    #[test]
    fn test_explanation_included() {
        let explanation = Explanation::included("Matched line 42");
        assert_eq!(explanation.disposition, Disposition::Included);
        assert_eq!(explanation.reason, "Matched line 42");
        assert!(explanation.is_included());
    }

    #[test]
    fn test_explanation_outside_diff() {
        let explanation = Explanation::outside_diff();
        assert_eq!(explanation.disposition, Disposition::OutsideDiff);
        assert!(!explanation.is_included());
    }

    #[test]
    fn test_explanation_generated_file() {
        let explanation = Explanation::generated_file();
        assert_eq!(explanation.disposition, Disposition::GeneratedFile);
        assert!(!explanation.is_included());
    }

    #[test]
    fn test_explanation_suppressed() {
        let explanation = Explanation::suppressed("clippy::all");
        assert_eq!(explanation.disposition, Disposition::Suppressed);
        assert_eq!(explanation.code, Some("clippy::all".to_string()));
        assert!(!explanation.is_included());
    }

    #[test]
    fn test_explanation_no_span() {
        let explanation = Explanation::no_span();
        assert_eq!(explanation.disposition, Disposition::NoSpan);
        assert!(!explanation.is_included());
    }

    #[test]
    fn test_explanation_non_workspace() {
        let explanation = Explanation::non_workspace();
        assert_eq!(explanation.disposition, Disposition::NonWorkspace);
        assert!(!explanation.is_included());
    }

    #[test]
    fn test_disposition_serialization() {
        let disposition = Disposition::Included;
        let json = serde_json::to_string(&disposition).unwrap();
        assert_eq!(json, "\"included\"");

        let parsed: Disposition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, disposition);
    }

    #[test]
    fn test_explanation_serialization() {
        let explanation = Explanation::suppressed("dead_code");
        let json = serde_json::to_string(&explanation).unwrap();
        assert!(json.contains("\"disposition\":\"suppressed\""));
        assert!(json.contains("\"code\":\"dead_code\""));

        let parsed: Explanation = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, explanation);
    }
}
