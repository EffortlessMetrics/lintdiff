//! Comprehensive BDD tests for lintdiff-run-info.
//!
//! These tests cover:
//! - RunInfo construction and methods
//! - RunStatus variants and behavior
//! - RunTimer usage and state transitions
//! - Duration formatting (human-readable and short)
//! - Timestamp formatting
//! - Edge cases (zero duration, very long durations, negative durations)
//! - Property-based tests with proptest

use std::time::Duration as StdDuration;

use lintdiff_run_info::{
    duration_from_hours, duration_from_millis, duration_from_minutes, duration_from_secs,
    format_duration, format_duration_short, format_timestamp_rfc3339, is_expired, is_expired_at,
    now_utc, parse_timestamp, RunInfo, RunInfoError, RunStatus, RunTimer,
};
use proptest::prelude::*;
use time::macros::datetime;
use time::{Duration, OffsetDateTime};

// =============================================================================
// RunStatus Tests
// =============================================================================

#[test]
fn run_status_is_in_progress_returns_true_only_for_running() {
    assert!(RunStatus::Running.is_in_progress());
    assert!(!RunStatus::Completed.is_in_progress());
    assert!(!RunStatus::Failed.is_in_progress());
    assert!(!RunStatus::Cancelled.is_in_progress());
}

#[test]
fn run_status_is_finished_returns_true_for_all_non_running_states() {
    assert!(!RunStatus::Running.is_finished());
    assert!(RunStatus::Completed.is_finished());
    assert!(RunStatus::Failed.is_finished());
    assert!(RunStatus::Cancelled.is_finished());
}

#[test]
fn run_status_is_success_returns_true_only_for_completed() {
    assert!(!RunStatus::Running.is_success());
    assert!(RunStatus::Completed.is_success());
    assert!(!RunStatus::Failed.is_success());
    assert!(!RunStatus::Cancelled.is_success());
}

#[test]
fn run_status_is_failure_returns_true_for_failed_and_cancelled() {
    assert!(!RunStatus::Running.is_failure());
    assert!(!RunStatus::Completed.is_failure());
    assert!(RunStatus::Failed.is_failure());
    assert!(RunStatus::Cancelled.is_failure());
}

#[test]
fn run_status_as_label_returns_correct_string() {
    assert_eq!(RunStatus::Running.as_label(), "Running");
    assert_eq!(RunStatus::Completed.as_label(), "Completed");
    assert_eq!(RunStatus::Failed.as_label(), "Failed");
    assert_eq!(RunStatus::Cancelled.as_label(), "Cancelled");
}

#[test]
fn run_status_display_trait_formats_correctly() {
    assert_eq!(format!("{}", RunStatus::Running), "Running");
    assert_eq!(format!("{}", RunStatus::Completed), "Completed");
    assert_eq!(format!("{}", RunStatus::Failed), "Failed");
    assert_eq!(format!("{}", RunStatus::Cancelled), "Cancelled");
}

#[test]
fn run_status_default_trait_returns_running() {
    assert_eq!(RunStatus::default(), RunStatus::Running);
}

// =============================================================================
// RunInfo Tests
// =============================================================================

#[test]
fn run_info_new_creates_a_running_run_info_with_start_time() {
    let start = datetime!(2024-01-15 10:00:00 UTC);
    let info = RunInfo::new(start);

    assert_eq!(info.start_time, start);
    assert_eq!(info.end_time, None);
    assert_eq!(info.duration, None);
    assert_eq!(info.status, RunStatus::Running);
}

#[test]
fn run_info_now_creates_a_running_run_info() {
    let before = now_utc();
    let info = RunInfo::now();
    let after = now_utc();

    assert!(info.start_time >= before);
    assert!(info.start_time <= after);
    assert_eq!(info.status, RunStatus::Running);
}

#[test]
fn run_info_completed_creates_a_completed_run_info() {
    let start = datetime!(2024-01-15 10:00:00 UTC);
    let end = datetime!(2024-01-15 10:30:00 UTC);
    let info = RunInfo::completed(start, end);

    assert_eq!(info.start_time, start);
    assert_eq!(info.end_time, Some(end));
    assert_eq!(info.duration, Some(Duration::minutes(30)));
    assert_eq!(info.status, RunStatus::Completed);
}

#[test]
fn run_info_completed_calculates_duration_correctly() {
    let start = datetime!(2024-01-15 10:00:00 UTC);
    let end = datetime!(2024-01-15 12:45:30 UTC);
    let info = RunInfo::completed(start, end);

    let expected_duration = Duration::hours(2) + Duration::minutes(45) + Duration::seconds(30);
    assert_eq!(info.duration, Some(expected_duration));
}

#[test]
fn run_info_completed_handles_zero_duration() {
    let start = datetime!(2024-01-15 10:00:00 UTC);
    let info = RunInfo::completed(start, start);

    assert_eq!(info.duration, Some(Duration::seconds(0)));
}

#[test]
fn run_info_failed_creates_a_failed_run_info() {
    let start = datetime!(2024-01-15 10:00:00 UTC);
    let end = datetime!(2024-01-15 10:15:00 UTC);
    let info = RunInfo::failed(start, end);

    assert_eq!(info.status, RunStatus::Failed);
    assert_eq!(info.duration, Some(Duration::minutes(15)));
}

#[test]
fn run_info_cancelled_creates_a_cancelled_run_info() {
    let start = datetime!(2024-01-15 10:00:00 UTC);
    let end = datetime!(2024-01-15 10:05:00 UTC);
    let info = RunInfo::cancelled(start, end);

    assert_eq!(info.status, RunStatus::Cancelled);
    assert_eq!(info.duration, Some(Duration::minutes(5)));
}

#[test]
fn run_info_is_in_progress_delegates_to_status() {
    let running_info = RunInfo::new(datetime!(2024-01-15 10:00:00 UTC));
    let completed_info = RunInfo::completed(
        datetime!(2024-01-15 10:00:00 UTC),
        datetime!(2024-01-15 10:30:00 UTC),
    );

    assert!(running_info.is_in_progress());
    assert!(!completed_info.is_in_progress());
}

#[test]
fn run_info_is_finished_delegates_to_status() {
    let running_info = RunInfo::new(datetime!(2024-01-15 10:00:00 UTC));
    let completed_info = RunInfo::completed(
        datetime!(2024-01-15 10:00:00 UTC),
        datetime!(2024-01-15 10:30:00 UTC),
    );

    assert!(!running_info.is_finished());
    assert!(completed_info.is_finished());
}

#[test]
fn run_info_is_success_delegates_to_status() {
    let completed_info = RunInfo::completed(
        datetime!(2024-01-15 10:00:00 UTC),
        datetime!(2024-01-15 10:30:00 UTC),
    );
    let failed_info = RunInfo::failed(
        datetime!(2024-01-15 10:00:00 UTC),
        datetime!(2024-01-15 10:30:00 UTC),
    );

    assert!(completed_info.is_success());
    assert!(!failed_info.is_success());
}

#[test]
fn run_info_is_failure_delegates_to_status() {
    let completed_info = RunInfo::completed(
        datetime!(2024-01-15 10:00:00 UTC),
        datetime!(2024-01-15 10:30:00 UTC),
    );
    let failed_info = RunInfo::failed(
        datetime!(2024-01-15 10:00:00 UTC),
        datetime!(2024-01-15 10:30:00 UTC),
    );
    let cancelled_info = RunInfo::cancelled(
        datetime!(2024-01-15 10:00:00 UTC),
        datetime!(2024-01-15 10:30:00 UTC),
    );

    assert!(!completed_info.is_failure());
    assert!(failed_info.is_failure());
    assert!(cancelled_info.is_failure());
}

#[test]
fn run_info_formatted_duration_returns_formatted_string() {
    let start = datetime!(2024-01-15 10:00:00 UTC);
    let end = datetime!(2024-01-15 10:00:30 UTC);
    let info = RunInfo::completed(start, end);

    assert_eq!(info.formatted_duration(), Some("30s".to_string()));
}

#[test]
fn run_info_formatted_duration_returns_none_for_running() {
    let info = RunInfo::new(datetime!(2024-01-15 10:00:00 UTC));

    assert_eq!(info.formatted_duration(), None);
}

#[test]
fn run_info_formatted_duration_short_returns_short_format() {
    let start = datetime!(2024-01-15 10:00:00 UTC);
    let end = datetime!(2024-01-15 10:00:01 UTC);
    let info = RunInfo::completed(start, end);

    assert_eq!(info.formatted_duration_short(), Some("1.000s".to_string()));
}

#[test]
fn run_info_formatted_start_time_returns_rfc3339_format() {
    let start = datetime!(2024-01-15 10:30:00 UTC);
    let info = RunInfo::new(start);

    let formatted = info.formatted_start_time();
    assert!(formatted.starts_with("2024-01-15T10:30:00"));
}

#[test]
fn run_info_formatted_end_time_returns_some_for_completed() {
    let start = datetime!(2024-01-15 10:00:00 UTC);
    let end = datetime!(2024-01-15 10:30:00 UTC);
    let info = RunInfo::completed(start, end);

    let formatted = info.formatted_end_time();
    assert!(formatted.is_some());
    assert!(formatted.unwrap().starts_with("2024-01-15T10:30:00"));
}

#[test]
fn run_info_formatted_end_time_returns_none_for_running() {
    let info = RunInfo::new(datetime!(2024-01-15 10:00:00 UTC));

    assert_eq!(info.formatted_end_time(), None);
}

#[test]
fn run_info_duration_std_converts_to_std_duration() {
    let start = datetime!(2024-01-15 10:00:00 UTC);
    let end = datetime!(2024-01-15 10:00:05 UTC);
    let info = RunInfo::completed(start, end);

    let std_dur = info.duration_std();
    assert_eq!(std_dur, Some(StdDuration::from_secs(5)));
}

#[test]
fn run_info_duration_std_returns_none_for_running() {
    let info = RunInfo::new(datetime!(2024-01-15 10:00:00 UTC));

    assert_eq!(info.duration_std(), None);
}

#[test]
fn run_info_duration_std_handles_milliseconds() {
    let start = datetime!(2024-01-15 10:00:00.000 UTC);
    let end = datetime!(2024-01-15 10:00:00.500 UTC);
    let info = RunInfo::completed(start, end);

    let std_dur = info.duration_std();
    assert_eq!(std_dur, Some(StdDuration::from_millis(500)));
}

// =============================================================================
// RunTimer Tests
// =============================================================================

#[test]
fn run_timer_new_creates_a_new_running_timer() {
    let timer = RunTimer::new();

    assert!(timer.is_running());
    assert!(!timer.is_completed());
}

#[test]
fn run_timer_new_starts_with_current_time() {
    let before = now_utc();
    let timer = RunTimer::new();
    let after = now_utc();

    let start = timer.start_time();
    assert!(start >= before);
    assert!(start <= after);
}

#[test]
fn run_timer_with_start_time_creates_timer_with_specific_start_time() {
    let start = datetime!(2024-01-15 10:00:00 UTC);
    let timer = RunTimer::with_start_time(start);

    assert_eq!(timer.start_time(), start);
}

#[test]
fn run_timer_elapsed_returns_elapsed_time_since_start() {
    let timer = RunTimer::new();
    let elapsed = timer.elapsed();

    // Should be very small since we just created it
    assert!(elapsed.whole_seconds() < 1);
}

#[test]
fn run_timer_elapsed_std_returns_std_duration() {
    let timer = RunTimer::new();
    let elapsed = timer.elapsed_std();

    // Should be very small
    assert!(elapsed.as_secs() < 1);
}

#[test]
fn run_timer_formatted_elapsed_returns_formatted_elapsed_time() {
    let timer = RunTimer::new();
    let formatted = timer.formatted_elapsed();

    // Should be milliseconds since we just created it
    assert!(formatted.ends_with("ms") || formatted.ends_with("s"));
}

#[test]
fn run_timer_complete_marks_timer_as_completed() {
    let mut timer = RunTimer::new();
    let info = timer.complete().unwrap();

    assert!(timer.is_completed());
    assert_eq!(info.status, RunStatus::Completed);
    assert!(info.duration.is_some());
    assert!(info.end_time.is_some());
}

#[test]
fn run_timer_complete_returns_error_on_double_complete() {
    let mut timer = RunTimer::new();
    let _ = timer.complete().unwrap();

    let result = timer.complete();
    assert!(matches!(result, Err(RunInfoError::AlreadyCompleted)));
}

#[test]
fn run_timer_complete_stores_final_info() {
    let mut timer = RunTimer::new();
    let info = timer.complete().unwrap();

    let final_info = timer.final_info();
    assert!(final_info.is_some());
    assert_eq!(final_info.unwrap().status, RunStatus::Completed);
}

#[test]
fn run_timer_fail_marks_timer_as_failed() {
    let mut timer = RunTimer::new();
    let info = timer.fail().unwrap();

    assert!(timer.is_completed());
    assert_eq!(info.status, RunStatus::Failed);
}

#[test]
fn run_timer_fail_returns_error_if_already_completed() {
    let mut timer = RunTimer::new();
    let _ = timer.complete().unwrap();

    let result = timer.fail();
    assert!(matches!(result, Err(RunInfoError::AlreadyCompleted)));
}

#[test]
fn run_timer_cancel_marks_timer_as_cancelled() {
    let mut timer = RunTimer::new();
    let info = timer.cancel().unwrap();

    assert!(timer.is_completed());
    assert_eq!(info.status, RunStatus::Cancelled);
}

#[test]
fn run_timer_cancel_returns_error_if_already_completed() {
    let mut timer = RunTimer::new();
    let _ = timer.complete().unwrap();

    let result = timer.cancel();
    assert!(matches!(result, Err(RunInfoError::AlreadyCompleted)));
}

#[test]
fn run_timer_cannot_transition_from_failed_to_complete() {
    let mut timer = RunTimer::new();
    let _ = timer.fail().unwrap();

    let result = timer.complete();
    assert!(matches!(result, Err(RunInfoError::AlreadyCompleted)));
}

#[test]
fn run_timer_cannot_transition_from_cancelled_to_fail() {
    let mut timer = RunTimer::new();
    let _ = timer.cancel().unwrap();

    let result = timer.fail();
    assert!(matches!(result, Err(RunInfoError::AlreadyCompleted)));
}

#[test]
fn run_timer_default_creates_a_new_timer() {
    let timer = RunTimer::default();

    assert!(timer.is_running());
}

// =============================================================================
// Duration Formatting Tests
// =============================================================================

#[test]
fn format_duration_formatsZeroDuration() {
    assert_eq!(format_duration(&Duration::seconds(0)), "0ms");
}

#[test]
fn format_duration_formats_milliseconds_less_than_1_second() {
    assert_eq!(format_duration(&Duration::milliseconds(1)), "1ms");
    assert_eq!(format_duration(&Duration::milliseconds(125)), "125ms");
    assert_eq!(format_duration(&Duration::milliseconds(999)), "999ms");
}

#[test]
fn format_duration_formats_seconds_only() {
    assert_eq!(format_duration(&Duration::seconds(1)), "1s");
    assert_eq!(format_duration(&Duration::seconds(30)), "30s");
    assert_eq!(format_duration(&Duration::seconds(59)), "59s");
}

#[test]
fn format_duration_formats_minutes_and_seconds() {
    assert_eq!(format_duration(&Duration::seconds(60)), "1m");
    assert_eq!(format_duration(&Duration::seconds(90)), "1m30s");
    assert_eq!(format_duration(&Duration::seconds(119)), "1m59s");
    assert_eq!(format_duration(&Duration::seconds(150)), "2m30s");
}

#[test]
fn format_duration_formats_hours_minutes_and_seconds() {
    assert_eq!(format_duration(&Duration::seconds(3600)), "1h");
    assert_eq!(format_duration(&Duration::seconds(3661)), "1h1m1s");
    assert_eq!(format_duration(&Duration::seconds(4530)), "1h15m30s");
    assert_eq!(format_duration(&Duration::seconds(7325)), "2h2m5s");
}

#[test]
fn format_duration_formats_days_hours_minutes_and_seconds() {
    assert_eq!(format_duration(&Duration::seconds(86400)), "1d");
    assert_eq!(format_duration(&Duration::seconds(90061)), "1d1h1m1s");
    assert_eq!(format_duration(&Duration::seconds(93630)), "1d2h30s");
    assert_eq!(format_duration(&Duration::seconds(172800)), "2d");
}

#[test]
fn format_duration_formats_very_long_durations() {
    let duration =
        Duration::days(7) + Duration::hours(12) + Duration::minutes(30) + Duration::seconds(45);
    assert_eq!(format_duration(&duration), "7d12h30m45s");
}

#[test]
fn format_duration_formats_negative_durations() {
    assert_eq!(format_duration(&Duration::milliseconds(-125)), "-125ms");
    assert_eq!(format_duration(&Duration::seconds(-90)), "-1m30s");
    assert_eq!(format_duration(&Duration::seconds(-3661)), "-1h1m1s");
}

#[test]
fn format_duration_short_formats_zero_duration() {
    assert_eq!(format_duration_short(&Duration::seconds(0)), "0.000s");
}

#[test]
fn format_duration_short_formats_milliseconds_as_decimal_seconds() {
    assert_eq!(
        format_duration_short(&Duration::milliseconds(125)),
        "0.125s"
    );
    assert_eq!(
        format_duration_short(&Duration::milliseconds(500)),
        "0.500s"
    );
    assert_eq!(
        format_duration_short(&Duration::milliseconds(999)),
        "0.999s"
    );
}

#[test]
fn format_duration_short_formats_seconds_with_decimal() {
    assert_eq!(format_duration_short(&Duration::seconds(1)), "1.000s");
    assert_eq!(format_duration_short(&Duration::seconds(30)), "30.000s");
    assert_eq!(format_duration_short(&Duration::seconds(90)), "90.000s");
}

#[test]
fn format_duration_short_formats_large_durations_in_seconds() {
    assert_eq!(format_duration_short(&Duration::seconds(3600)), "3600.000s");
    assert_eq!(
        format_duration_short(&Duration::seconds(86400)),
        "86400.000s"
    );
}

#[test]
fn format_duration_short_formats_negative_durations() {
    assert_eq!(
        format_duration_short(&Duration::milliseconds(-125)),
        "-0.125s"
    );
    assert_eq!(format_duration_short(&Duration::seconds(-90)), "-90.000s");
}

#[test]
fn format_duration_short_combines_seconds_and_milliseconds() {
    let duration = Duration::seconds(5) + Duration::milliseconds(250);
    assert_eq!(format_duration_short(&duration), "5.250s");
}

// =============================================================================
// Timestamp Formatting Tests
// =============================================================================

#[test]
fn format_timestamp_rfc3339_formats_utc_timestamp_correctly() {
    let dt = datetime!(2024-01-15 10:30:00 UTC);
    let formatted = format_timestamp_rfc3339(&dt);

    assert!(formatted.starts_with("2024-01-15T10:30:00"));
}

#[test]
fn format_timestamp_rfc3339_formats_timestamp_with_positive_offset() {
    let dt = datetime!(2024-01-15 10:30:00 +01:00);
    let formatted = format_timestamp_rfc3339(&dt);

    assert!(formatted.contains("+01:00"));
}

#[test]
fn format_timestamp_rfc3339_formats_timestamp_with_negative_offset() {
    let dt = datetime!(2024-01-15 10:30:00 -05:00);
    let formatted = format_timestamp_rfc3339(&dt);

    assert!(formatted.contains("-05:00"));
}

#[test]
fn format_timestamp_rfc3339_formats_date_at_year_boundary() {
    let dt = datetime!(2024-12-31 23:59:59 UTC);
    let formatted = format_timestamp_rfc3339(&dt);

    assert!(formatted.contains("2024-12-31"));
    assert!(formatted.contains("23:59:59"));
}

#[test]
fn format_timestamp_rfc3339_formats_date_at_year_start() {
    let dt = datetime!(2024-01-01 00:00:00 UTC);
    let formatted = format_timestamp_rfc3339(&dt);

    assert!(formatted.contains("2024-01-01"));
    assert!(formatted.contains("00:00:00"));
}

#[test]
fn now_utc_returns_current_time() {
    let before = now_utc();
    let now = now_utc();
    let after = now_utc();

    assert!(now >= before);
    assert!(now <= after);
}

#[test]
fn now_utc_returns_time_in_recent_past() {
    let now = now_utc();
    let year = now.year();

    assert!(year >= 2024);
}

#[test]
fn parse_timestamp_parses_valid_rfc3339_timestamp() {
    let result = parse_timestamp("2024-01-15T10:30:00Z");
    assert!(result.is_ok());

    let dt = result.unwrap();
    assert_eq!(dt.year(), 2024);
    assert_eq!(dt.month() as u8, 1);
    assert_eq!(dt.day(), 15);
}

#[test]
fn parse_timestamp_parses_timestamp_with_offset() {
    let result = parse_timestamp("2024-01-15T10:30:00+01:00");
    assert!(result.is_ok());
}

#[test]
fn parse_timestamp_returns_error_for_invalid_timestamp() {
    let result = parse_timestamp("not-a-timestamp");
    assert!(result.is_err());
}

// =============================================================================
// Expiration Tests
// =============================================================================

#[test]
fn is_expired_at_returns_false_for_recent_run() {
    let start = datetime!(2024-01-15 10:00:00 UTC);
    let end = datetime!(2024-01-15 10:30:00 UTC);
    let info = RunInfo::completed(start, end);

    let reference = datetime!(2024-01-15 10:45:00 UTC);
    let max_age = Duration::hours(1);

    assert!(!is_expired_at(&info, &max_age, reference));
}

#[test]
fn is_expired_at_returns_true_for_old_run() {
    let start = datetime!(2024-01-15 08:00:00 UTC);
    let end = datetime!(2024-01-15 08:30:00 UTC);
    let info = RunInfo::completed(start, end);

    let reference = datetime!(2024-01-15 10:00:00 UTC);
    let max_age = Duration::hours(1);

    assert!(is_expired_at(&info, &max_age, reference));
}

#[test]
fn is_expired_at_uses_start_time_for_running_runs() {
    let start = datetime!(2024-01-15 08:00:00 UTC);
    let info = RunInfo::new(start);

    let reference = datetime!(2024-01-15 10:00:00 UTC);
    let max_age = Duration::hours(1);

    assert!(is_expired_at(&info, &max_age, reference));
}

#[test]
fn is_expired_at_returns_false_at_exact_boundary() {
    let start = datetime!(2024-01-15 09:00:00 UTC);
    let end = datetime!(2024-01-15 09:30:00 UTC);
    let info = RunInfo::completed(start, end);

    let reference = datetime!(2024-01-15 10:30:00 UTC);
    let max_age = Duration::hours(1);

    // Exactly 1 hour, not expired (needs to be > max_age)
    assert!(!is_expired_at(&info, &max_age, reference));
}

#[test]
fn is_expired_at_returns_true_just_past_boundary() {
    let start = datetime!(2024-01-15 09:00:00 UTC);
    let end = datetime!(2024-01-15 09:30:00 UTC);
    let info = RunInfo::completed(start, end);

    let reference = datetime!(2024-01-15 10:30:01 UTC);
    let max_age = Duration::hours(1);

    // Just past 1 hour
    assert!(is_expired_at(&info, &max_age, reference));
}

#[test]
fn is_expired_compares_against_current_time() {
    // Create a run from the past
    let start = datetime!(2024-01-15 08:00:00 UTC);
    let end = datetime!(2024-01-15 08:30:00 UTC);
    let info = RunInfo::completed(start, end);

    // With a very short max age, it should be expired
    // Note: This test may be flaky if run exactly at the boundary
    // In practice, a run from 2024 will always be expired
    assert!(is_expired(&info, &Duration::days(1)));
}

// =============================================================================
// Duration Helper Tests
// =============================================================================

#[test]
fn duration_from_secs_creates_duration_from_seconds() {
    assert_eq!(duration_from_secs(60), Duration::seconds(60));
    assert_eq!(duration_from_secs(0), Duration::seconds(0));
    assert_eq!(duration_from_secs(-30), Duration::seconds(-30));
}

#[test]
fn duration_from_millis_creates_duration_from_milliseconds() {
    assert_eq!(duration_from_millis(1000), Duration::milliseconds(1000));
    assert_eq!(duration_from_millis(500), Duration::milliseconds(500));
}

#[test]
fn duration_from_minutes_creates_duration_from_minutes() {
    assert_eq!(duration_from_minutes(5), Duration::minutes(5));
    assert_eq!(duration_from_minutes(60), Duration::hours(1));
}

#[test]
fn duration_from_hours_creates_duration_from_hours() {
    assert_eq!(duration_from_hours(2), Duration::hours(2));
    assert_eq!(duration_from_hours(24), Duration::hours(24));
}

// =============================================================================
// RunInfoError Tests
// =============================================================================

#[test]
fn run_info_error_already_completed_creates_correct_error() {
    let error = RunInfoError::already_completed();
    assert_eq!(error.to_string(), "run has already been completed");
}

#[test]
fn run_info_error_not_started_creates_correct_error() {
    let error = RunInfoError::not_started();
    assert_eq!(error.to_string(), "run has not been started");
}

#[test]
fn run_info_error_invalid_duration_creates_correct_error() {
    let error = RunInfoError::invalid_duration("test message");
    assert_eq!(error.to_string(), "invalid duration: test message");
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn format_duration_very_long_365_days() {
    let duration = Duration::days(365);
    let formatted = format_duration(&duration);
    assert_eq!(formatted, "365d");
}

#[test]
fn format_duration_negative_365_days() {
    let duration = Duration::days(-365);
    let formatted = format_duration(&duration);
    assert_eq!(formatted, "-365d");
}

#[test]
fn format_duration_very_short_1_millisecond() {
    assert_eq!(format_duration(&Duration::milliseconds(1)), "1ms");
}

#[test]
fn format_duration_short_0_milliseconds() {
    assert_eq!(format_duration_short(&Duration::seconds(0)), "0.000s");
}

#[test]
fn format_duration_boundary_exactly_60_seconds() {
    assert_eq!(format_duration(&Duration::seconds(60)), "1m");
}

#[test]
fn format_duration_boundary_exactly_3600_seconds() {
    assert_eq!(format_duration(&Duration::seconds(3600)), "1h");
}

#[test]
fn format_duration_boundary_exactly_86400_seconds() {
    assert_eq!(format_duration(&Duration::seconds(86400)), "1d");
}

// =============================================================================
// Property-Based Tests
// =============================================================================

proptest! {
    #[test]
    fn prop_format_duration_never_panics(secs in -1_000_000i64..1_000_000i64) {
        let duration = Duration::seconds(secs);
        let _ = format_duration(&duration);
    }

    #[test]
    fn prop_format_duration_short_never_panics(secs in -1_000_000i64..1_000_000i64) {
        let duration = Duration::seconds(secs);
        let _ = format_duration_short(&duration);
    }

    #[test]
    fn prop_format_duration_positive_output_contains_valid_units(secs in 0i64..864000i64) {
        let duration = Duration::seconds(secs);
        let formatted = format_duration(&duration);

        // Should only contain valid characters
        let valid_chars = formatted.chars().all(|c| c.is_ascii_digit() || c == 'd' || c == 'h' || c == 'm' || c == 's' || c == '-');
        assert!(valid_chars);
    }

    #[test]
    fn prop_format_duration_short_ends_with_s(secs in -1_000_000i64..1_000_000i64) {
        let duration = Duration::seconds(secs);
        let formatted = format_duration_short(&duration);

        assert!(formatted.ends_with('s'));
    }

    #[test]
    fn prop_run_info_completed_duration_matches_times(
        start_secs in 0i64..1_000_000i64,
        duration_secs in 0i64..86400i64
    ) {
        let start = datetime!(2024-01-01 00:00:00 UTC) + Duration::seconds(start_secs);
        let end = start + Duration::seconds(duration_secs);

        let info = RunInfo::completed(start, end);

        assert_eq!(info.duration, Some(Duration::seconds(duration_secs)));
    }

    #[test]
    fn prop_is_expired_consistent_with_duration(
        offset_mins in 0i64..1440i64,  // 0 to 24 hours in minutes
        max_age_mins in 1i64..1440i64
    ) {
        let reference = datetime!(2024-01-15 12:00:00 UTC);
        let end = reference - Duration::minutes(offset_mins);

        let info = RunInfo::completed(end - Duration::minutes(30), end);
        let max_age = Duration::minutes(max_age_mins);

        let expected = offset_mins > max_age_mins;
        assert_eq!(is_expired_at(&info, &max_age, reference), expected);
    }

    #[test]
    fn prop_duration_helpers_are_consistent(value in 0i64..10000i64) {
        assert_eq!(duration_from_secs(value), Duration::seconds(value));
        assert_eq!(duration_from_millis(value), Duration::milliseconds(value));
        assert_eq!(duration_from_minutes(value), Duration::minutes(value));
        assert_eq!(duration_from_hours(value), Duration::hours(value));
    }

    #[test]
    fn prop_format_timestamp_rfc3339_is_parseable(
        offset_secs in 0i64..31536000i64  // 0 to ~1 year in seconds
    ) {
        // Create a datetime from a base time plus offset
        let base = datetime!(2024-01-01 00:00:00 UTC);
        let dt = base + Duration::seconds(offset_secs);
        let formatted = format_timestamp_rfc3339(&dt);
        let parsed = parse_timestamp(&formatted);

        assert!(parsed.is_ok());
    }
}

// =============================================================================
// Serde Tests (only run with serde feature)
// =============================================================================

#[cfg(feature = "serde")]
mod serde_tests {
    use super::*;

    #[test]
    fn test_run_status_serde() {
        let status = RunStatus::Completed;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: RunStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_run_info_serde() {
        let start = datetime!(2024-01-15 10:00:00 UTC);
        let end = datetime!(2024-01-15 10:30:00 UTC);
        let info = RunInfo::completed(start, end);

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: RunInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info, deserialized);
    }
}
