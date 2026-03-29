//! Run timing and information utilities for lintdiff.
//!
//! This crate provides utilities for tracking run times, statuses, and formatting
//! duration/timestamp information for lintdiff reports.
//!
//! # Features
//!
//! - `serde` - Enable serde serialization for [`RunInfo`] and [`RunStatus`]
//!
//! # Examples
//!
//! ```
//! use lintdiff_run_info::{RunTimer, format_duration, now_utc};
//!
//! // Create and use a timer
//! let mut timer = RunTimer::new();
//! let elapsed = timer.elapsed();
//! let info = timer.complete().unwrap();
//! assert!(info.duration.is_some());
//! ```

use std::time::Duration as StdDuration;

use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

/// Error type for run info operations.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum RunInfoError {
    /// The run has already been completed.
    #[error("run has already been completed")]
    AlreadyCompleted,

    /// The run has not been started.
    #[error("run has not been started")]
    NotStarted,

    /// Invalid duration value.
    #[error("invalid duration: {0}")]
    InvalidDuration(String),
}

impl RunInfoError {
    /// Create a new already completed error.
    #[must_use]
    pub const fn already_completed() -> Self {
        Self::AlreadyCompleted
    }

    /// Create a new not started error.
    #[must_use]
    pub const fn not_started() -> Self {
        Self::NotStarted
    }

    /// Create a new invalid duration error.
    #[must_use]
    pub fn invalid_duration(msg: impl Into<String>) -> Self {
        Self::InvalidDuration(msg.into())
    }
}

/// Status of a run.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RunStatus {
    /// Currently running.
    #[default]
    Running,
    /// Successfully completed.
    Completed,
    /// Failed with an error.
    Failed,
    /// Cancelled by the user.
    Cancelled,
}

impl RunStatus {
    /// Check if the run is still in progress.
    #[must_use]
    pub const fn is_in_progress(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Check if the run has finished (regardless of outcome).
    #[must_use]
    pub const fn is_finished(&self) -> bool {
        !self.is_in_progress()
    }

    /// Check if the run completed successfully.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Completed)
    }

    /// Check if the run failed.
    #[must_use]
    pub const fn is_failure(&self) -> bool {
        matches!(self, Self::Failed | Self::Cancelled)
    }

    /// Get a human-readable label for the status.
    #[must_use]
    pub const fn as_label(&self) -> &'static str {
        match self {
            Self::Running => "Running",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }
}

impl std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_label())
    }
}

/// Information about a run.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RunInfo {
    /// Run start time.
    pub start_time: OffsetDateTime,
    /// Run end time (None if still running).
    pub end_time: Option<OffsetDateTime>,
    /// Run duration (None if still running).
    pub duration: Option<Duration>,
    /// Current status.
    pub status: RunStatus,
}

impl RunInfo {
    /// Create a new run info with the given start time.
    #[must_use]
    pub const fn new(start_time: OffsetDateTime) -> Self {
        Self {
            start_time,
            end_time: None,
            duration: None,
            status: RunStatus::Running,
        }
    }

    /// Create a new run info starting now.
    #[must_use]
    pub fn now() -> Self {
        Self::new(now_utc())
    }

    /// Create a completed run info.
    #[must_use]
    pub fn completed(start_time: OffsetDateTime, end_time: OffsetDateTime) -> Self {
        Self {
            start_time,
            end_time: Some(end_time),
            duration: Some(end_time - start_time),
            status: RunStatus::Completed,
        }
    }

    /// Create a failed run info.
    #[must_use]
    pub fn failed(start_time: OffsetDateTime, end_time: OffsetDateTime) -> Self {
        Self {
            start_time,
            end_time: Some(end_time),
            duration: Some(end_time - start_time),
            status: RunStatus::Failed,
        }
    }

    /// Create a cancelled run info.
    #[must_use]
    pub fn cancelled(start_time: OffsetDateTime, end_time: OffsetDateTime) -> Self {
        Self {
            start_time,
            end_time: Some(end_time),
            duration: Some(end_time - start_time),
            status: RunStatus::Cancelled,
        }
    }

    /// Check if the run is still in progress.
    #[must_use]
    pub const fn is_in_progress(&self) -> bool {
        self.status.is_in_progress()
    }

    /// Check if the run has finished.
    #[must_use]
    pub const fn is_finished(&self) -> bool {
        self.status.is_finished()
    }

    /// Check if the run completed successfully.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.status.is_success()
    }

    /// Check if the run failed.
    #[must_use]
    pub const fn is_failure(&self) -> bool {
        self.status.is_failure()
    }

    /// Get the duration as a standard library duration.
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub fn duration_std(&self) -> Option<StdDuration> {
        self.duration.and_then(|d| {
            let secs = d.whole_seconds();
            let nanos = d.subsec_nanoseconds();
            if secs >= 0 && nanos >= 0 {
                Some(StdDuration::new(secs as u64, nanos as u32))
            } else {
                None
            }
        })
    }

    /// Get a formatted duration string.
    #[must_use]
    pub fn formatted_duration(&self) -> Option<String> {
        self.duration.as_ref().map(format_duration)
    }

    /// Get a formatted short duration string.
    #[must_use]
    pub fn formatted_duration_short(&self) -> Option<String> {
        self.duration.as_ref().map(format_duration_short)
    }

    /// Get the start time formatted as RFC3339.
    #[must_use]
    pub fn formatted_start_time(&self) -> String {
        format_timestamp_rfc3339(&self.start_time)
    }

    /// Get the end time formatted as RFC3339.
    #[must_use]
    pub fn formatted_end_time(&self) -> Option<String> {
        self.end_time.as_ref().map(format_timestamp_rfc3339)
    }
}

/// Timer for tracking runs.
#[derive(Debug, Clone)]
pub struct RunTimer {
    start_time: OffsetDateTime,
    completed: bool,
    info: Option<RunInfo>,
}

impl RunTimer {
    /// Create and start a new timer.
    #[must_use]
    pub fn new() -> Self {
        Self {
            start_time: now_utc(),
            completed: false,
            info: None,
        }
    }

    /// Create a timer with a specific start time (useful for testing).
    #[must_use]
    pub const fn with_start_time(start_time: OffsetDateTime) -> Self {
        Self {
            start_time,
            completed: false,
            info: None,
        }
    }

    /// Get the start time.
    #[must_use]
    pub const fn start_time(&self) -> OffsetDateTime {
        self.start_time
    }

    /// Get the elapsed time since the timer started.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        now_utc() - self.start_time
    }

    /// Get the elapsed time as a standard library duration.
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub fn elapsed_std(&self) -> StdDuration {
        let elapsed = self.elapsed();
        let secs = elapsed.whole_seconds().max(0) as u64;
        let nanos = elapsed.subsec_nanoseconds().max(0) as u32;
        StdDuration::new(secs, nanos)
    }

    /// Get a formatted elapsed time string.
    #[must_use]
    pub fn formatted_elapsed(&self) -> String {
        format_duration(&self.elapsed())
    }

    /// Check if the timer is still running.
    #[must_use]
    pub const fn is_running(&self) -> bool {
        !self.completed
    }

    /// Check if the timer has been completed.
    #[must_use]
    pub const fn is_completed(&self) -> bool {
        self.completed
    }

    /// Mark the timer as complete and return the run info.
    /// Returns an error if already completed.
    ///
    /// # Errors
    ///
    /// Returns [`RunInfoError::AlreadyCompleted`] if the timer has already been marked.
    pub fn complete(&mut self) -> Result<RunInfo, RunInfoError> {
        if self.completed {
            return Err(RunInfoError::already_completed());
        }
        let end_time = now_utc();
        let info = RunInfo::completed(self.start_time, end_time);
        self.completed = true;
        self.info = Some(info.clone());
        Ok(info)
    }

    /// Mark the timer as failed and return the run info.
    /// Returns an error if already completed.
    ///
    /// # Errors
    ///
    /// Returns [`RunInfoError::AlreadyCompleted`] if the timer has already been marked.
    pub fn fail(&mut self) -> Result<RunInfo, RunInfoError> {
        if self.completed {
            return Err(RunInfoError::already_completed());
        }
        let end_time = now_utc();
        let info = RunInfo::failed(self.start_time, end_time);
        self.completed = true;
        self.info = Some(info.clone());
        Ok(info)
    }

    /// Mark the timer as cancelled and return the run info.
    /// Returns an error if already completed.
    ///
    /// # Errors
    ///
    /// Returns [`RunInfoError::AlreadyCompleted`] if the timer has already been marked.
    pub fn cancel(&mut self) -> Result<RunInfo, RunInfoError> {
        if self.completed {
            return Err(RunInfoError::already_completed());
        }
        let end_time = now_utc();
        let info = RunInfo::cancelled(self.start_time, end_time);
        self.completed = true;
        self.info = Some(info.clone());
        Ok(info)
    }

    /// Get the final run info if completed.
    #[must_use]
    pub const fn final_info(&self) -> Option<&RunInfo> {
        self.info.as_ref()
    }
}

impl Default for RunTimer {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Time Utilities
// =============================================================================

/// Get the current UTC time.
///
/// # Examples
///
/// ```
/// use lintdiff_run_info::now_utc;
///
/// let dt = now_utc();
/// assert!(dt.year() >= 2024);
/// ```
#[must_use]
pub fn now_utc() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

/// Format a duration in a human-readable format.
///
/// Output examples:
/// - `0ms`
/// - `125ms`
/// - `1.5s`
/// - `1m30s`
/// - `1h15m30s`
/// - `1d2h15m30s`
///
/// # Examples
///
/// ```
/// use lintdiff_run_info::format_duration;
/// use time::Duration;
///
/// assert_eq!(format_duration(&Duration::milliseconds(125)), "125ms");
/// assert_eq!(format_duration(&Duration::seconds(90)), "1m30s");
/// assert_eq!(format_duration(&Duration::seconds(4530)), "1h15m30s");
/// ```
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn format_duration(d: &Duration) -> String {
    let total_seconds = d.whole_seconds();
    let millis = d.whole_milliseconds();

    // Handle negative durations
    if total_seconds < 0 {
        // For reasonable durations, this cast is safe
        let abs_millis = (-millis) as i64;
        let abs_d = Duration::milliseconds(abs_millis);
        return format!("-{}", format_duration(&abs_d));
    }

    // Less than 1 second: show milliseconds
    if total_seconds == 0 {
        let ms = d.whole_milliseconds();
        if ms == 0 {
            return "0ms".to_string();
        }
        return format!("{ms}ms");
    }

    let mut parts = Vec::new();

    let days = total_seconds / 86_400;
    let remainder = total_seconds % 86_400;
    let hours = remainder / 3_600;
    let remainder = remainder % 3_600;
    let minutes = remainder / 60;
    let seconds = remainder % 60;

    if days > 0 {
        parts.push(format!("{days}d"));
    }
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    if minutes > 0 {
        parts.push(format!("{minutes}m"));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{seconds}s"));
    }

    parts.join("")
}

/// Format a duration in a short format (decimal seconds).
///
/// Output examples:
/// - `0.000s`
/// - `0.125s`
/// - `90.000s`
/// - `3600.000s`
///
/// # Examples
///
/// ```
/// use lintdiff_run_info::format_duration_short;
/// use time::Duration;
///
/// assert_eq!(format_duration_short(&Duration::milliseconds(125)), "0.125s");
/// assert_eq!(format_duration_short(&Duration::seconds(90)), "90.000s");
/// assert_eq!(format_duration_short(&Duration::seconds(4530)), "4530.000s");
/// ```
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn format_duration_short(d: &Duration) -> String {
    let total_millis = d.whole_milliseconds();

    // Handle negative durations
    if total_millis < 0 {
        // For reasonable durations, this cast is safe
        let abs_millis = (-total_millis) as i64;
        let abs_d = Duration::milliseconds(abs_millis);
        return format!("-{}", format_duration_short(&abs_d));
    }

    let abs_millis = d.whole_milliseconds().unsigned_abs();
    let secs = abs_millis / 1000;
    let ms = abs_millis % 1000;

    format!("{secs}.{ms:03}s")
}

/// Format a timestamp as RFC3339.
///
/// Output format: `2024-01-15T10:30:00Z`
///
/// # Examples
///
/// ```
/// use lintdiff_run_info::format_timestamp_rfc3339;
/// use time::macros::datetime;
///
/// let dt = datetime!(2024-01-15 10:30:00 UTC);
/// let formatted = format_timestamp_rfc3339(&dt);
/// assert!(formatted.starts_with("2024-01-15T10:30:00"));
/// ```
#[must_use]
pub fn format_timestamp_rfc3339(dt: &OffsetDateTime) -> String {
    dt.format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

/// Check if a run is expired (older than the maximum age).
///
/// # Examples
///
/// ```
/// use lintdiff_run_info::{is_expired, RunInfo, now_utc};
/// use time::Duration;
/// use time::macros::datetime;
///
/// // Create a run info from 2 hours ago
/// let start = datetime!(2024-01-15 08:00:00 UTC);
/// let end = datetime!(2024-01-15 08:30:00 UTC);
/// let info = RunInfo::completed(start, end);
///
/// // Check if it's older than 1 hour (using a fixed time for comparison)
/// let max_age = Duration::hours(1);
/// // Note: In practice, this compares against the current time
/// ```
#[must_use]
pub fn is_expired(info: &RunInfo, max_age: &Duration) -> bool {
    // A run is expired if its end time (or start time if still running)
    // is older than max_age from now
    let reference_time = info.end_time.unwrap_or(info.start_time);
    let now = now_utc();
    let age = now - reference_time;
    age > *max_age
}

/// Check if a run is expired relative to a specific reference time.
///
/// This is useful for testing or when comparing against a fixed time.
#[must_use]
pub fn is_expired_at(info: &RunInfo, max_age: &Duration, reference_time: OffsetDateTime) -> bool {
    let run_time = info.end_time.unwrap_or(info.start_time);
    let age = reference_time - run_time;
    age > *max_age
}

// =============================================================================
// Additional Utility Functions
// =============================================================================

/// Parse an RFC3339 timestamp string.
///
/// # Errors
///
/// Returns [`RunInfoError::InvalidDuration`] if the string cannot be parsed.
pub fn parse_timestamp(s: &str) -> Result<OffsetDateTime, RunInfoError> {
    OffsetDateTime::parse(s, &Rfc3339).map_err(|e| RunInfoError::invalid_duration(e.to_string()))
}

/// Create a duration from seconds.
#[must_use]
pub const fn duration_from_secs(secs: i64) -> Duration {
    Duration::seconds(secs)
}

/// Create a duration from milliseconds.
#[must_use]
pub const fn duration_from_millis(millis: i64) -> Duration {
    Duration::milliseconds(millis)
}

/// Create a duration from minutes.
#[must_use]
pub const fn duration_from_minutes(minutes: i64) -> Duration {
    Duration::minutes(minutes)
}

/// Create a duration from hours.
#[must_use]
pub const fn duration_from_hours(hours: i64) -> Duration {
    Duration::hours(hours)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn test_run_status_is_in_progress() {
        assert!(RunStatus::Running.is_in_progress());
        assert!(!RunStatus::Completed.is_in_progress());
        assert!(!RunStatus::Failed.is_in_progress());
        assert!(!RunStatus::Cancelled.is_in_progress());
    }

    #[test]
    fn test_run_status_is_finished() {
        assert!(!RunStatus::Running.is_finished());
        assert!(RunStatus::Completed.is_finished());
        assert!(RunStatus::Failed.is_finished());
        assert!(RunStatus::Cancelled.is_finished());
    }

    #[test]
    fn test_run_status_is_success() {
        assert!(!RunStatus::Running.is_success());
        assert!(RunStatus::Completed.is_success());
        assert!(!RunStatus::Failed.is_success());
        assert!(!RunStatus::Cancelled.is_success());
    }

    #[test]
    fn test_run_status_is_failure() {
        assert!(!RunStatus::Running.is_failure());
        assert!(!RunStatus::Completed.is_failure());
        assert!(RunStatus::Failed.is_failure());
        assert!(RunStatus::Cancelled.is_failure());
    }

    #[test]
    fn test_run_status_as_label() {
        assert_eq!(RunStatus::Running.as_label(), "Running");
        assert_eq!(RunStatus::Completed.as_label(), "Completed");
        assert_eq!(RunStatus::Failed.as_label(), "Failed");
        assert_eq!(RunStatus::Cancelled.as_label(), "Cancelled");
    }

    #[test]
    fn test_run_status_display() {
        assert_eq!(format!("{}", RunStatus::Running), "Running");
        assert_eq!(format!("{}", RunStatus::Completed), "Completed");
        assert_eq!(format!("{}", RunStatus::Failed), "Failed");
        assert_eq!(format!("{}", RunStatus::Cancelled), "Cancelled");
    }

    #[test]
    fn test_run_status_default() {
        assert_eq!(RunStatus::default(), RunStatus::Running);
    }

    #[test]
    fn test_run_info_new() {
        let start = datetime!(2024-01-15 10:00:00 UTC);
        let info = RunInfo::new(start);
        assert_eq!(info.start_time, start);
        assert_eq!(info.end_time, None);
        assert_eq!(info.duration, None);
        assert_eq!(info.status, RunStatus::Running);
    }

    #[test]
    fn test_run_info_completed() {
        let start = datetime!(2024-01-15 10:00:00 UTC);
        let end = datetime!(2024-01-15 10:30:00 UTC);
        let info = RunInfo::completed(start, end);
        assert_eq!(info.start_time, start);
        assert_eq!(info.end_time, Some(end));
        assert_eq!(info.duration, Some(Duration::minutes(30)));
        assert_eq!(info.status, RunStatus::Completed);
    }

    #[test]
    fn test_run_info_failed() {
        let start = datetime!(2024-01-15 10:00:00 UTC);
        let end = datetime!(2024-01-15 10:15:00 UTC);
        let info = RunInfo::failed(start, end);
        assert_eq!(info.start_time, start);
        assert_eq!(info.end_time, Some(end));
        assert_eq!(info.duration, Some(Duration::minutes(15)));
        assert_eq!(info.status, RunStatus::Failed);
    }

    #[test]
    fn test_run_info_cancelled() {
        let start = datetime!(2024-01-15 10:00:00 UTC);
        let end = datetime!(2024-01-15 10:05:00 UTC);
        let info = RunInfo::cancelled(start, end);
        assert_eq!(info.start_time, start);
        assert_eq!(info.end_time, Some(end));
        assert_eq!(info.duration, Some(Duration::minutes(5)));
        assert_eq!(info.status, RunStatus::Cancelled);
    }

    #[test]
    fn test_format_duration_zero() {
        assert_eq!(format_duration(&Duration::seconds(0)), "0ms");
    }

    #[test]
    fn test_format_duration_milliseconds() {
        assert_eq!(format_duration(&Duration::milliseconds(125)), "125ms");
        assert_eq!(format_duration(&Duration::milliseconds(999)), "999ms");
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(&Duration::seconds(30)), "30s");
        assert_eq!(format_duration(&Duration::seconds(59)), "59s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(&Duration::seconds(60)), "1m");
        assert_eq!(format_duration(&Duration::seconds(90)), "1m30s");
        assert_eq!(format_duration(&Duration::seconds(120)), "2m");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(&Duration::seconds(3600)), "1h");
        assert_eq!(format_duration(&Duration::seconds(4530)), "1h15m30s");
    }

    #[test]
    fn test_format_duration_days() {
        assert_eq!(format_duration(&Duration::seconds(86400)), "1d");
        assert_eq!(format_duration(&Duration::seconds(93630)), "1d2h30s");
    }

    #[test]
    fn test_format_duration_negative() {
        assert_eq!(format_duration(&Duration::milliseconds(-125)), "-125ms");
        assert_eq!(format_duration(&Duration::seconds(-90)), "-1m30s");
    }

    #[test]
    fn test_format_duration_short() {
        assert_eq!(format_duration_short(&Duration::seconds(0)), "0.000s");
        assert_eq!(
            format_duration_short(&Duration::milliseconds(125)),
            "0.125s"
        );
        assert_eq!(format_duration_short(&Duration::seconds(90)), "90.000s");
        assert_eq!(format_duration_short(&Duration::seconds(4530)), "4530.000s");
    }

    #[test]
    fn test_format_duration_short_negative() {
        // Negative durations are handled by negating the output
        let result = format_duration_short(&Duration::milliseconds(-125));
        // The implementation handles negative durations
        assert!(result.contains("0.125s") || result == "-0.125s");
    }

    #[test]
    fn test_format_timestamp_rfc3339() {
        let dt = datetime!(2024-01-15 10:30:00 UTC);
        let formatted = format_timestamp_rfc3339(&dt);
        assert!(formatted.starts_with("2024-01-15T10:30:00"));
    }

    #[test]
    fn test_run_timer_new() {
        let timer = RunTimer::new();
        assert!(timer.is_running());
        assert!(!timer.is_completed());
    }

    #[test]
    fn test_run_timer_elapsed() {
        let timer = RunTimer::new();
        let elapsed = timer.elapsed();
        // Should be very small since we just created it
        assert!(elapsed.whole_seconds() < 1);
    }

    #[test]
    fn test_run_timer_complete() {
        let mut timer = RunTimer::new();
        let info = timer.complete().unwrap();
        assert!(timer.is_completed());
        assert_eq!(info.status, RunStatus::Completed);
        assert!(info.duration.is_some());
    }

    #[test]
    fn test_run_timer_fail() {
        let mut timer = RunTimer::new();
        let info = timer.fail().unwrap();
        assert!(timer.is_completed());
        assert_eq!(info.status, RunStatus::Failed);
        assert!(info.duration.is_some());
    }

    #[test]
    fn test_run_timer_cancel() {
        let mut timer = RunTimer::new();
        let info = timer.cancel().unwrap();
        assert!(timer.is_completed());
        assert_eq!(info.status, RunStatus::Cancelled);
        assert!(info.duration.is_some());
    }

    #[test]
    fn test_run_timer_double_complete() {
        let mut timer = RunTimer::new();
        let _ = timer.complete().unwrap();
        let result = timer.complete();
        assert!(matches!(result, Err(RunInfoError::AlreadyCompleted)));
    }

    #[test]
    fn test_is_expired_at() {
        let start = datetime!(2024-01-15 08:00:00 UTC);
        let end = datetime!(2024-01-15 08:30:00 UTC);
        let info = RunInfo::completed(start, end);

        // Check at 09:00 (30 minutes after end)
        let reference = datetime!(2024-01-15 09:00:00 UTC);
        let max_age = Duration::minutes(60);
        assert!(!is_expired_at(&info, &max_age, reference));

        // Check at 10:00 (90 minutes after end)
        let reference = datetime!(2024-01-15 10:00:00 UTC);
        assert!(is_expired_at(&info, &max_age, reference));
    }

    #[test]
    fn test_duration_helpers() {
        assert_eq!(duration_from_secs(60), Duration::seconds(60));
        assert_eq!(duration_from_millis(1000), Duration::milliseconds(1000));
        assert_eq!(duration_from_minutes(5), Duration::minutes(5));
        assert_eq!(duration_from_hours(2), Duration::hours(2));
    }
}
