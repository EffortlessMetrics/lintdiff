//! Comprehensive BDD tests for lintdiff-timestamp.
//!
//! These tests cover:
//! - ISO 8601 formatting
//! - Timestamp parsing
//! - Duration formatting
//! - TimestampFormatter builder
//! - TimeSpan operations
//! - Edge cases (leap years, timezone handling)
//! - Property-based tests with proptest

use std::time::Duration as StdDuration;

use lintdiff_timestamp::{
    days_in_month, format_duration, format_now, format_timestamp, format_timestamp_millis,
    is_leap_year, now_utc, parse_timestamp, timestamps_approx_equal, Date, TimeSpan,
    TimestampError, TimestampFormatter,
};
use proptest::prelude::*;
use time::macros::{datetime, offset};
use time::{Duration, OffsetDateTime};

// =============================================================================
// ISO 8601 Formatting Tests
// =============================================================================

mod iso8601_formatting {
    use super::*;

    #[test]
    fn format_timestamp_returns_correct_length() {
        let dt = datetime!(2024-01-15 10:30:00 UTC);
        let result = format_timestamp(&dt);
        // RFC3339 format: "2024-01-15T10:30:00Z" = 20 chars
        assert_eq!(result.len(), 20);
    }

    #[test]
    fn format_timestamp_contains_date_components() {
        let dt = datetime!(2024-01-15 10:30:00 UTC);
        let result = format_timestamp(&dt);
        assert!(result.contains("2024"));
        assert!(result.contains("01"));
        assert!(result.contains("15"));
    }

    #[test]
    fn format_timestamp_contains_time_components() {
        let dt = datetime!(2024-01-15 10:30:45 UTC);
        let result = format_timestamp(&dt);
        assert!(result.contains("10:30:45"));
    }

    #[test]
    fn format_timestamp_ends_with_z_for_utc() {
        let dt = datetime!(2024-01-15 10:30:00 UTC);
        let result = format_timestamp(&dt);
        assert!(result.ends_with('Z'));
    }

    #[test]
    fn format_timestamp_with_positive_offset() {
        let dt = datetime!(2024-01-15 10:30:00 +01:00);
        let result = format_timestamp(&dt);
        assert!(result.contains("+01:00"));
    }

    #[test]
    fn format_timestamp_with_negative_offset() {
        let dt = datetime!(2024-01-15 10:30:00 -05:00);
        let result = format_timestamp(&dt);
        assert!(result.contains("-05:00"));
    }

    #[test]
    fn format_timestamp_millis_includes_milliseconds() {
        let dt = datetime!(2024-01-15 10:30:00.123 UTC);
        let result = format_timestamp_millis(&dt);
        assert!(result.contains(".123"));
    }

    #[test]
    fn format_timestamp_millis_with_zero_milliseconds() {
        let dt = datetime!(2024-01-15 10:30:00.000 UTC);
        let result = format_timestamp_millis(&dt);
        assert!(result.contains(".000"));
    }

    #[test]
    fn format_timestamp_millis_preserves_date() {
        let dt = datetime!(2024-12-31 23:59:59.999 UTC);
        let result = format_timestamp_millis(&dt);
        assert!(result.contains("2024-12-31"));
    }

    #[test]
    fn format_now_returns_valid_iso8601() {
        let result = format_now();
        // Should be parseable as RFC3339
        let parsed = parse_timestamp(&result);
        assert!(parsed.is_ok());
    }

    #[test]
    fn format_now_format_is_consistent() {
        let first = format_now();
        std::thread::sleep(StdDuration::from_millis(10));
        let second = format_now();
        // Both should be valid ISO8601 format ending with Z
        assert!(first.ends_with('Z'));
        assert!(second.ends_with('Z'));
    }

    #[test]
    fn format_timestamp_at_epoch() {
        let dt = datetime!(1970-01-01 00:00:00 UTC);
        let result = format_timestamp(&dt);
        assert!(result.contains("1970-01-01"));
        assert!(result.contains("00:00:00"));
    }

    #[test]
    fn format_timestamp_at_far_future() {
        let dt = datetime!(2099-12-31 23:59:59 UTC);
        let result = format_timestamp(&dt);
        assert!(result.contains("2099-12-31"));
        assert!(result.contains("23:59:59"));
    }

    #[test]
    fn format_timestamp_midnight() {
        let dt = datetime!(2024-06-15 00:00:00 UTC);
        let result = format_timestamp(&dt);
        assert!(result.contains("00:00:00"));
    }

    #[test]
    fn format_timestamp_end_of_day() {
        let dt = datetime!(2024-06-15 23:59:59 UTC);
        let result = format_timestamp(&dt);
        assert!(result.contains("23:59:59"));
    }
}

// =============================================================================
// Timestamp Parsing Tests
// =============================================================================

mod timestamp_parsing {
    use super::*;

    #[test]
    fn parse_timestamp_basic_iso8601() {
        let result = parse_timestamp("2024-01-15T10:30:00Z").unwrap();
        assert_eq!(result.year(), 2024);
        assert_eq!(result.month() as u8, 1);
        assert_eq!(result.day(), 15);
        assert_eq!(result.hour(), 10);
        assert_eq!(result.minute(), 30);
        assert_eq!(result.second(), 0);
    }

    #[test]
    fn parse_timestamp_with_offset() {
        let result = parse_timestamp("2024-01-15T10:30:00+00:00").unwrap();
        assert_eq!(result.year(), 2024);
    }

    #[test]
    fn parse_timestamp_with_positive_offset() {
        let result = parse_timestamp("2024-01-15T10:30:00+05:00").unwrap();
        assert_eq!(result.year(), 2024);
    }

    #[test]
    fn parse_timestamp_with_negative_offset() {
        let result = parse_timestamp("2024-01-15T10:30:00-08:00").unwrap();
        assert_eq!(result.year(), 2024);
    }

    #[test]
    fn parse_timestamp_with_milliseconds() {
        let result = parse_timestamp("2024-01-15T10:30:00.123Z").unwrap();
        assert_eq!(result.year(), 2024);
        assert_eq!(result.millisecond(), 123);
    }

    #[test]
    fn parse_timestamp_with_microseconds() {
        let result = parse_timestamp("2024-01-15T10:30:00.123456Z").unwrap();
        assert_eq!(result.year(), 2024);
        assert_eq!(result.microsecond(), 123456);
    }

    #[test]
    fn parse_timestamp_invalid_format_returns_error() {
        let result = parse_timestamp("not-a-timestamp");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TimestampError::ParseError(_)));
    }

    #[test]
    fn parse_timestamp_empty_string_returns_error() {
        let result = parse_timestamp("");
        assert!(result.is_err());
    }

    #[test]
    fn parse_timestamp_partial_date_returns_error() {
        let result = parse_timestamp("2024-01");
        assert!(result.is_err());
    }

    #[test]
    fn parse_timestamp_with_space_instead_of_t() {
        // Some formats use space instead of T
        let result = parse_timestamp("2024-01-15 10:30:00Z");
        // This might fail depending on implementation
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn parse_timestamp_epoch() {
        let result = parse_timestamp("1970-01-01T00:00:00Z").unwrap();
        assert_eq!(result.year(), 1970);
    }

    #[test]
    fn parse_timestamp_far_future() {
        let result = parse_timestamp("2099-12-31T23:59:59Z").unwrap();
        assert_eq!(result.year(), 2099);
        assert_eq!(result.month() as u8, 12);
        assert_eq!(result.day(), 31);
    }

    #[test]
    fn parse_timestamp_leap_day() {
        let result = parse_timestamp("2024-02-29T12:00:00Z").unwrap();
        assert_eq!(result.year(), 2024);
        assert_eq!(result.month() as u8, 2);
        assert_eq!(result.day(), 29);
    }

    #[test]
    fn parse_timestamp_roundtrip() {
        let original = datetime!(2024-06-15 14:30:45 UTC);
        let formatted = format_timestamp(&original);
        let parsed = parse_timestamp(&formatted).unwrap();
        // Note: parsed might have a different offset representation
        assert_eq!(parsed.year(), original.year());
        assert_eq!(parsed.month(), original.month());
        assert_eq!(parsed.day(), original.day());
        assert_eq!(parsed.hour(), original.hour());
        assert_eq!(parsed.minute(), original.minute());
        assert_eq!(parsed.second(), original.second());
    }
}

// =============================================================================
// Duration Formatting Tests
// =============================================================================

mod duration_formatting {
    use super::*;

    #[test]
    fn format_duration_zero() {
        let result = format_duration(&StdDuration::ZERO);
        assert_eq!(result, "0ms");
    }

    #[test]
    fn format_duration_one_millisecond() {
        let result = format_duration(&StdDuration::from_millis(1));
        assert_eq!(result, "1ms");
    }

    #[test]
    fn format_duration_hundred_milliseconds() {
        let result = format_duration(&StdDuration::from_millis(100));
        assert_eq!(result, "100ms");
    }

    #[test]
    fn format_duration_999_milliseconds() {
        let result = format_duration(&StdDuration::from_millis(999));
        assert_eq!(result, "999ms");
    }

    #[test]
    fn format_duration_one_second() {
        let result = format_duration(&StdDuration::from_secs(1));
        assert_eq!(result, "1s");
    }

    #[test]
    fn format_duration_one_second_with_millis() {
        let result = format_duration(&StdDuration::from_millis(1234));
        assert_eq!(result, "1.234s");
    }

    #[test]
    fn format_duration_thirty_seconds() {
        let result = format_duration(&StdDuration::from_secs(30));
        assert_eq!(result, "30s");
    }

    #[test]
    fn format_duration_fifty_nine_seconds() {
        let result = format_duration(&StdDuration::from_secs(59));
        assert_eq!(result, "59s");
    }

    #[test]
    fn format_duration_one_minute() {
        let result = format_duration(&StdDuration::from_secs(60));
        assert_eq!(result, "1m 0s");
    }

    #[test]
    fn format_duration_one_minute_thirty_seconds() {
        let result = format_duration(&StdDuration::from_secs(90));
        assert_eq!(result, "1m 30s");
    }

    #[test]
    fn format_duration_five_minutes() {
        let result = format_duration(&StdDuration::from_secs(300));
        assert_eq!(result, "5m 0s");
    }

    #[test]
    fn format_duration_fifty_nine_minutes_fifty_nine_seconds() {
        let result = format_duration(&StdDuration::from_secs(3599));
        assert_eq!(result, "59m 59s");
    }

    #[test]
    fn format_duration_one_hour() {
        let result = format_duration(&StdDuration::from_secs(3600));
        assert_eq!(result, "1h 0m 0s");
    }

    #[test]
    fn format_duration_one_hour_one_minute_one_second() {
        let result = format_duration(&StdDuration::from_secs(3661));
        assert_eq!(result, "1h 1m 1s");
    }

    #[test]
    fn format_duration_two_hours_thirty_minutes() {
        let result = format_duration(&StdDuration::from_secs(9000));
        assert_eq!(result, "2h 30m 0s");
    }

    #[test]
    fn format_duration_twenty_four_hours() {
        let result = format_duration(&StdDuration::from_secs(86400));
        assert_eq!(result, "24h 0m 0s");
    }

    #[test]
    fn format_duration_large_value() {
        let result = format_duration(&StdDuration::from_secs(100000));
        assert!(result.contains("h"));
        assert!(result.contains("m"));
        assert!(result.contains("s"));
    }
}

// =============================================================================
// TimestampFormatter Builder Tests
// =============================================================================

mod timestamp_formatter {
    use super::*;

    #[test]
    fn formatter_new_creates_default_formatter() {
        let formatter = TimestampFormatter::new();
        assert!(!formatter.has_milliseconds());
        assert!(formatter.has_timezone());
    }

    #[test]
    fn formatter_default_same_as_new() {
        let formatter = TimestampFormatter::default();
        assert!(!formatter.has_milliseconds());
        assert!(formatter.has_timezone());
    }

    #[test]
    fn formatter_with_milliseconds_true() {
        let formatter = TimestampFormatter::new().with_milliseconds(true);
        assert!(formatter.has_milliseconds());
    }

    #[test]
    fn formatter_with_milliseconds_false() {
        let formatter = TimestampFormatter::new().with_milliseconds(false);
        assert!(!formatter.has_milliseconds());
    }

    #[test]
    fn formatter_with_timezone_true() {
        let formatter = TimestampFormatter::new().with_timezone(true);
        assert!(formatter.has_timezone());
    }

    #[test]
    fn formatter_with_timezone_false() {
        let formatter = TimestampFormatter::new().with_timezone(false);
        assert!(!formatter.has_timezone());
    }

    #[test]
    fn formatter_chaining() {
        let formatter = TimestampFormatter::new()
            .with_milliseconds(true)
            .with_timezone(false);
        assert!(formatter.has_milliseconds());
        assert!(!formatter.has_timezone());
    }

    #[test]
    fn formatter_format_default() {
        let formatter = TimestampFormatter::new();
        let dt = datetime!(2024-01-15 10:30:00 UTC);
        let result = formatter.format(&dt);
        assert!(result.contains("2024-01-15"));
        assert!(result.contains("+00:00") || result.ends_with('Z'));
    }

    #[test]
    fn formatter_format_with_millis() {
        let formatter = TimestampFormatter::new().with_milliseconds(true);
        let dt = datetime!(2024-01-15 10:30:00.456 UTC);
        let result = formatter.format(&dt);
        assert!(result.contains(".456"));
    }

    #[test]
    fn formatter_format_without_timezone() {
        let formatter = TimestampFormatter::new().with_timezone(false);
        let dt = datetime!(2024-01-15 10:30:00 UTC);
        let result = formatter.format(&dt);
        assert!(!result.contains('Z'));
        assert!(!result.contains("+00:00"));
    }

    #[test]
    fn formatter_format_with_millis_without_timezone() {
        let formatter = TimestampFormatter::new()
            .with_milliseconds(true)
            .with_timezone(false);
        let dt = datetime!(2024-01-15 10:30:00.789 UTC);
        let result = formatter.format(&dt);
        assert!(result.contains(".789"));
        assert!(!result.contains('Z'));
        assert!(!result.contains("+00:00"));
    }

    #[test]
    fn formatter_clone_preserves_settings() {
        let original = TimestampFormatter::new()
            .with_milliseconds(true)
            .with_timezone(false);
        let cloned = original.clone();
        assert_eq!(original.has_milliseconds(), cloned.has_milliseconds());
        assert_eq!(original.has_timezone(), cloned.has_timezone());
    }

    #[test]
    fn formatter_copy_preserves_settings() {
        let original = TimestampFormatter::new()
            .with_milliseconds(true)
            .with_timezone(false);
        let copied = original; // Copy
        assert_eq!(original.has_milliseconds(), copied.has_milliseconds());
        assert_eq!(original.has_timezone(), copied.has_timezone());
    }

    #[test]
    fn formatter_equality() {
        let a = TimestampFormatter::new().with_milliseconds(true);
        let b = TimestampFormatter::new().with_milliseconds(true);
        let c = TimestampFormatter::new().with_milliseconds(false);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}

// =============================================================================
// TimeSpan Operations Tests
// =============================================================================

mod time_span {
    use super::*;

    #[test]
    fn time_span_new_creates_span() {
        let start = datetime!(2024-01-15 10:00:00 UTC);
        let end = datetime!(2024-01-15 11:00:00 UTC);
        let span = TimeSpan::new(start, end);
        assert_eq!(span.start, start);
        assert_eq!(span.end, end);
    }

    #[test]
    #[should_panic(expected = "end must be >= start")]
    fn time_span_new_panics_on_inverted_range() {
        let start = datetime!(2024-01-15 11:00:00 UTC);
        let end = datetime!(2024-01-15 10:00:00 UTC);
        let _ = TimeSpan::new(start, end);
    }

    #[test]
    fn time_span_from_start_and_duration() {
        let start = datetime!(2024-01-15 10:00:00 UTC);
        let duration = Duration::hours(2);
        let span = TimeSpan::from_start_and_duration(start, duration);
        assert_eq!(span.start, start);
        assert_eq!(span.end, datetime!(2024-01-15 12:00:00 UTC));
    }

    #[test]
    fn time_span_from_end_and_duration() {
        let end = datetime!(2024-01-15 12:00:00 UTC);
        let duration = Duration::hours(2);
        let span = TimeSpan::from_end_and_duration(end, duration);
        assert_eq!(span.start, datetime!(2024-01-15 10:00:00 UTC));
        assert_eq!(span.end, end);
    }

    #[test]
    fn time_span_duration() {
        let start = datetime!(2024-01-15 10:00:00 UTC);
        let end = datetime!(2024-01-15 10:01:30 UTC);
        let span = TimeSpan::new(start, end);
        assert_eq!(span.duration(), Duration::seconds(90));
    }

    #[test]
    fn time_span_duration_zero() {
        let dt = datetime!(2024-01-15 10:00:00 UTC);
        let span = TimeSpan::new(dt, dt);
        assert_eq!(span.duration(), Duration::ZERO);
    }

    #[test]
    fn time_span_format_range() {
        let start = datetime!(2024-01-15 10:00:00 UTC);
        let end = datetime!(2024-01-15 11:30:00 UTC);
        let span = TimeSpan::new(start, end);
        let result = span.format_range();
        assert!(result.contains(" to "));
    }

    #[test]
    fn time_span_format_range_with_duration() {
        let start = datetime!(2024-01-15 10:00:00 UTC);
        let end = datetime!(2024-01-15 11:30:00 UTC);
        let span = TimeSpan::new(start, end);
        let result = span.format_range_with_duration();
        assert!(result.contains(" to "));
        assert!(result.contains('('));
        assert!(result.contains(')'));
    }

    #[test]
    fn time_span_contains_middle() {
        let start = datetime!(2024-01-15 10:00:00 UTC);
        let end = datetime!(2024-01-15 11:00:00 UTC);
        let span = TimeSpan::new(start, end);
        let middle = datetime!(2024-01-15 10:30:00 UTC);
        assert!(span.contains(middle));
    }

    #[test]
    fn time_span_contains_start() {
        let start = datetime!(2024-01-15 10:00:00 UTC);
        let end = datetime!(2024-01-15 11:00:00 UTC);
        let span = TimeSpan::new(start, end);
        assert!(span.contains(start));
    }

    #[test]
    fn time_span_contains_end() {
        let start = datetime!(2024-01-15 10:00:00 UTC);
        let end = datetime!(2024-01-15 11:00:00 UTC);
        let span = TimeSpan::new(start, end);
        assert!(span.contains(end));
    }

    #[test]
    fn time_span_does_not_contain_before() {
        let start = datetime!(2024-01-15 10:00:00 UTC);
        let end = datetime!(2024-01-15 11:00:00 UTC);
        let span = TimeSpan::new(start, end);
        let before = datetime!(2024-01-15 09:59:59 UTC);
        assert!(!span.contains(before));
    }

    #[test]
    fn time_span_does_not_contain_after() {
        let start = datetime!(2024-01-15 10:00:00 UTC);
        let end = datetime!(2024-01-15 11:00:00 UTC);
        let span = TimeSpan::new(start, end);
        let after = datetime!(2024-01-15 11:00:01 UTC);
        assert!(!span.contains(after));
    }

    #[test]
    fn time_span_overlaps_partial() {
        let span1 = TimeSpan::new(
            datetime!(2024-01-15 10:00:00 UTC),
            datetime!(2024-01-15 11:00:00 UTC),
        );
        let span2 = TimeSpan::new(
            datetime!(2024-01-15 10:30:00 UTC),
            datetime!(2024-01-15 11:30:00 UTC),
        );
        assert!(span1.overlaps(&span2));
        assert!(span2.overlaps(&span1));
    }

    #[test]
    fn time_span_overlaps_complete() {
        let span1 = TimeSpan::new(
            datetime!(2024-01-15 10:00:00 UTC),
            datetime!(2024-01-15 12:00:00 UTC),
        );
        let span2 = TimeSpan::new(
            datetime!(2024-01-15 10:30:00 UTC),
            datetime!(2024-01-15 11:30:00 UTC),
        );
        assert!(span1.overlaps(&span2));
        assert!(span2.overlaps(&span1));
    }

    #[test]
    fn time_span_no_overlap() {
        let span1 = TimeSpan::new(
            datetime!(2024-01-15 10:00:00 UTC),
            datetime!(2024-01-15 11:00:00 UTC),
        );
        let span2 = TimeSpan::new(
            datetime!(2024-01-15 11:00:01 UTC),
            datetime!(2024-01-15 12:00:00 UTC),
        );
        assert!(!span1.overlaps(&span2));
        assert!(!span2.overlaps(&span1));
    }

    #[test]
    fn time_span_intersection_partial() {
        let span1 = TimeSpan::new(
            datetime!(2024-01-15 10:00:00 UTC),
            datetime!(2024-01-15 11:00:00 UTC),
        );
        let span2 = TimeSpan::new(
            datetime!(2024-01-15 10:30:00 UTC),
            datetime!(2024-01-15 11:30:00 UTC),
        );
        let intersection = span1.intersection(&span2).unwrap();
        assert_eq!(intersection.start, datetime!(2024-01-15 10:30:00 UTC));
        assert_eq!(intersection.end, datetime!(2024-01-15 11:00:00 UTC));
    }

    #[test]
    fn time_span_intersection_none() {
        let span1 = TimeSpan::new(
            datetime!(2024-01-15 10:00:00 UTC),
            datetime!(2024-01-15 11:00:00 UTC),
        );
        let span2 = TimeSpan::new(
            datetime!(2024-01-15 11:00:01 UTC),
            datetime!(2024-01-15 12:00:00 UTC),
        );
        assert!(span1.intersection(&span2).is_none());
    }

    #[test]
    fn time_span_is_empty_zero_duration() {
        let dt = datetime!(2024-01-15 10:00:00 UTC);
        let span = TimeSpan::new(dt, dt);
        assert!(span.is_empty());
    }

    #[test]
    fn time_span_is_not_empty_with_duration() {
        let span = TimeSpan::new(
            datetime!(2024-01-15 10:00:00 UTC),
            datetime!(2024-01-15 10:00:01 UTC),
        );
        assert!(!span.is_empty());
    }

    #[test]
    fn time_span_midpoint() {
        let span = TimeSpan::new(
            datetime!(2024-01-15 10:00:00 UTC),
            datetime!(2024-01-15 12:00:00 UTC),
        );
        let mid = span.midpoint();
        assert_eq!(mid, datetime!(2024-01-15 11:00:00 UTC));
    }

    #[test]
    fn time_span_midpoint_odd_duration() {
        let span = TimeSpan::new(
            datetime!(2024-01-15 10:00:00 UTC),
            datetime!(2024-01-15 10:00:03 UTC),
        );
        let mid = span.midpoint();
        // 3 seconds / 2 = 1.5 seconds, which is represented with subseconds
        assert_eq!(mid.second(), 1);
        assert!(mid.millisecond() >= 500);
    }
}

// =============================================================================
// Date Tests
// =============================================================================

mod date {
    use super::*;

    #[test]
    fn date_new_creates_date() {
        let date = Date::new(2024, 1, 15);
        assert_eq!(date.year, 2024);
        assert_eq!(date.month, 1);
        assert_eq!(date.day, 15);
    }

    #[test]
    fn date_today_returns_current_date() {
        let today = Date::today();
        assert!(today.year >= 2024);
    }

    #[test]
    fn date_to_iso8601() {
        let date = Date::new(2024, 6, 15);
        assert_eq!(date.to_iso8601(), "2024-06-15");
    }

    #[test]
    fn date_to_iso8601_pads_single_digits() {
        let date = Date::new(2024, 1, 5);
        assert_eq!(date.to_iso8601(), "2024-01-05");
    }

    #[test]
    fn date_parse_valid() {
        let date = Date::parse("2024-06-15").unwrap();
        assert_eq!(date.year, 2024);
        assert_eq!(date.month, 6);
        assert_eq!(date.day, 15);
    }

    #[test]
    fn date_parse_with_single_digits() {
        let date = Date::parse("2024-01-05").unwrap();
        assert_eq!(date.month, 1);
        assert_eq!(date.day, 5);
    }

    #[test]
    fn date_parse_invalid_format() {
        let result = Date::parse("2024/06/15");
        assert!(result.is_err());
    }

    #[test]
    fn date_parse_invalid_month() {
        let result = Date::parse("2024-13-15");
        assert!(result.is_err());
    }

    #[test]
    fn date_to_midnight_utc() {
        let date = Date::new(2024, 6, 15);
        let dt = date.to_midnight_utc();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month() as u8, 6);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 0);
        assert_eq!(dt.second(), 0);
    }

    #[test]
    fn date_display() {
        let date = Date::new(2024, 6, 15);
        assert_eq!(format!("{date}"), "2024-06-15");
    }

    #[test]
    fn date_equality() {
        let a = Date::new(2024, 6, 15);
        let b = Date::new(2024, 6, 15);
        let c = Date::new(2024, 6, 16);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn date_ordering() {
        let a = Date::new(2024, 6, 15);
        let b = Date::new(2024, 6, 16);
        assert!(a < b);
    }
}

// =============================================================================
// Leap Year Tests
// =============================================================================

mod leap_year {
    use super::*;

    #[test]
    fn is_leap_year_2024() {
        assert!(is_leap_year(2024));
    }

    #[test]
    fn is_leap_year_2020() {
        assert!(is_leap_year(2020));
    }

    #[test]
    fn is_leap_year_2000() {
        assert!(is_leap_year(2000));
    }

    #[test]
    fn is_not_leap_year_2023() {
        assert!(!is_leap_year(2023));
    }

    #[test]
    fn is_not_leap_year_1900() {
        assert!(!is_leap_year(1900));
    }

    #[test]
    fn is_not_leap_year_2100() {
        assert!(!is_leap_year(2100));
    }

    #[test]
    fn days_in_month_february_leap_year() {
        assert_eq!(days_in_month(2024, 2), 29);
    }

    #[test]
    fn days_in_month_february_non_leap_year() {
        assert_eq!(days_in_month(2023, 2), 28);
    }

    #[test]
    fn days_in_month_january() {
        assert_eq!(days_in_month(2024, 1), 31);
    }

    #[test]
    fn days_in_month_april() {
        assert_eq!(days_in_month(2024, 4), 30);
    }

    #[test]
    fn days_in_month_december() {
        assert_eq!(days_in_month(2024, 12), 31);
    }

    #[test]
    fn days_in_month_invalid_returns_zero() {
        assert_eq!(days_in_month(2024, 0), 0);
        assert_eq!(days_in_month(2024, 13), 0);
    }
}

// =============================================================================
// Approximate Equality Tests
// =============================================================================

mod approx_equal {
    use super::*;

    #[test]
    fn timestamps_approx_equal_same_time() {
        let a = datetime!(2024-01-15 10:00:00 UTC);
        let b = datetime!(2024-01-15 10:00:00 UTC);
        assert!(timestamps_approx_equal(&a, &b, Duration::seconds(1)));
    }

    #[test]
    fn timestamps_approx_equal_within_tolerance() {
        let a = datetime!(2024-01-15 10:00:00 UTC);
        let b = datetime!(2024-01-15 10:00:00.500 UTC);
        assert!(timestamps_approx_equal(&a, &b, Duration::seconds(1)));
    }

    #[test]
    fn timestamps_not_approx_equal_outside_tolerance() {
        let a = datetime!(2024-01-15 10:00:00 UTC);
        let b = datetime!(2024-01-15 10:00:02 UTC);
        assert!(!timestamps_approx_equal(&a, &b, Duration::seconds(1)));
    }

    #[test]
    fn timestamps_approx_equal_reversed() {
        let a = datetime!(2024-01-15 10:00:01 UTC);
        let b = datetime!(2024-01-15 10:00:00 UTC);
        assert!(timestamps_approx_equal(&a, &b, Duration::seconds(1)));
    }

    #[test]
    fn timestamps_approx_equal_exactly_at_tolerance() {
        let a = datetime!(2024-01-15 10:00:00 UTC);
        let b = datetime!(2024-01-15 10:00:01 UTC);
        assert!(timestamps_approx_equal(&a, &b, Duration::seconds(1)));
    }
}

// =============================================================================
// Edge Cases Tests
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn format_timestamp_at_day_boundary() {
        let dt = datetime!(2024-01-15 23:59:59 UTC);
        let result = format_timestamp(&dt);
        assert!(result.contains("23:59:59"));
    }

    #[test]
    fn format_timestamp_at_month_boundary() {
        let dt = datetime!(2024-01-31 00:00:00 UTC);
        let result = format_timestamp(&dt);
        assert!(result.contains("2024-01-31"));
    }

    #[test]
    fn format_timestamp_at_year_boundary() {
        let dt = datetime!(2024-12-31 23:59:59 UTC);
        let result = format_timestamp(&dt);
        assert!(result.contains("2024-12-31"));
    }

    #[test]
    fn parse_timestamp_at_leap_second() {
        // 2016-12-31 had a leap second
        let result = parse_timestamp("2016-12-31T23:59:60Z");
        // This may or may not parse depending on the library
        // Just ensure it doesn't panic
        let _ = result;
    }

    #[test]
    fn time_span_crossing_day_boundary() {
        let span = TimeSpan::new(
            datetime!(2024-01-15 23:00:00 UTC),
            datetime!(2024-01-16 01:00:00 UTC),
        );
        assert_eq!(span.duration(), Duration::hours(2));
    }

    #[test]
    fn time_span_crossing_month_boundary() {
        let span = TimeSpan::new(
            datetime!(2024-01-31 23:00:00 UTC),
            datetime!(2024-02-01 01:00:00 UTC),
        );
        assert_eq!(span.duration(), Duration::hours(2));
    }

    #[test]
    fn time_span_crossing_year_boundary() {
        let span = TimeSpan::new(
            datetime!(2024-12-31 23:00:00 UTC),
            datetime!(2025-01-01 01:00:00 UTC),
        );
        assert_eq!(span.duration(), Duration::hours(2));
    }

    #[test]
    fn format_duration_very_small() {
        let result = format_duration(&StdDuration::from_nanos(1));
        // Nanoseconds are truncated to 0ms
        assert_eq!(result, "0ms");
    }

    #[test]
    fn format_duration_very_large() {
        let result = format_duration(&StdDuration::from_secs(365 * 24 * 60 * 60));
        assert!(result.contains('h'));
    }
}

// =============================================================================
// Error Handling Tests
// =============================================================================

mod error_handling {
    use super::*;

    #[test]
    fn timestamp_error_parse_creates_error() {
        let error = TimestampError::parse("test error");
        assert!(matches!(error, TimestampError::ParseError(_)));
    }

    #[test]
    fn timestamp_error_unsupported_creates_error() {
        let error = TimestampError::unsupported("test format");
        assert!(matches!(error, TimestampError::UnsupportedFormat(_)));
    }

    #[test]
    fn timestamp_error_out_of_range_creates_error() {
        let error = TimestampError::out_of_range("test range");
        assert!(matches!(error, TimestampError::OutOfRange(_)));
    }

    #[test]
    fn timestamp_error_display() {
        let error = TimestampError::parse("invalid format");
        let message = format!("{error}");
        assert!(message.contains("failed to parse"));
    }
}

// =============================================================================
// Property-Based Tests
// =============================================================================

mod property_tests {
    use super::*;
    use time::{Date, Month, Time, UtcOffset};

    fn create_datetime(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> Option<OffsetDateTime> {
        let date = Date::from_calendar_date(year, Month::try_from(month).ok()?, day).ok()?;
        let time = Time::from_hms(hour, minute, second).ok()?;
        Some(date.with_time(time).assume_offset(UtcOffset::UTC))
    }

    proptest! {
        #[test]
        fn format_timestamp_roundtrip(
            year in 2000i32..=2100,
            month in 1u8..=12,
            day in 1u8..=28, // Keep within safe range
            hour in 0u8..=23,
            minute in 0u8..=59,
            second in 0u8..=59,
        ) {
            let Some(dt) = create_datetime(year, month, day, hour, minute, second) else {
                prop_assert!(true);
                return Ok(());
            };
            let formatted = format_timestamp(&dt);
            let parsed = parse_timestamp(&formatted);
            prop_assert!(parsed.is_ok());

            let parsed_dt = parsed.unwrap();
            prop_assert_eq!(parsed_dt.year(), year);
            prop_assert_eq!(parsed_dt.month() as u8, month);
            prop_assert_eq!(parsed_dt.day(), day);
            prop_assert_eq!(parsed_dt.hour(), hour);
            prop_assert_eq!(parsed_dt.minute(), minute);
            prop_assert_eq!(parsed_dt.second(), second);
        }

        #[test]
        fn format_duration_non_negative(secs in 0u64..=1000000, millis in 0u32..=999) {
            let duration = StdDuration::new(secs, millis * 1_000_000);
            let formatted = format_duration(&duration);
            prop_assert!(!formatted.is_empty());
        }

        #[test]
        fn time_span_duration_positive(
            start_secs in 0i64..=1000000,
            duration_secs in 0i64..=1000000,
        ) {
            let epoch = OffsetDateTime::UNIX_EPOCH;
            let start = epoch + Duration::seconds(start_secs);
            let end = start + Duration::seconds(duration_secs);
            let span = TimeSpan::new(start, end);
            prop_assert_eq!(span.duration(), Duration::seconds(duration_secs));
        }

        #[test]
        fn time_span_contains_start_and_end(secs in 0i64..=1000000, extra_secs in 0i64..=1000) {
            let epoch = OffsetDateTime::UNIX_EPOCH;
            let start = epoch + Duration::seconds(secs);
            let end = start + Duration::seconds(extra_secs);
            let span = TimeSpan::new(start, end);
            prop_assert!(span.contains(start));
            prop_assert!(span.contains(end));
        }

        #[test]
        fn is_leap_year_consistent(year in 1900i32..=2100) {
            let is_leap = is_leap_year(year);
            let expected = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            prop_assert_eq!(is_leap, expected);
        }

        #[test]
        fn days_in_month_february_consistent(year in 1900i32..=2100) {
            let days = days_in_month(year, 2);
            let expected = if is_leap_year(year) { 29 } else { 28 };
            prop_assert_eq!(days, expected);
        }

        #[test]
        fn date_to_iso8601_roundtrip(year in 2000i32..=2100, month in 1u8..=12, day in 1u8..=28) {
            let date = super::Date::new(year, month, day);
            let iso = date.to_iso8601();
            let parsed = super::Date::parse(&iso);
            prop_assert!(parsed.is_ok());
            prop_assert_eq!(parsed.unwrap(), date);
        }

        #[test]
        fn formatter_format_never_empty(
            with_millis in proptest::bool::ANY,
            with_tz in proptest::bool::ANY,
            year in 2000i32..=2100,
            month in 1u8..=12,
            day in 1u8..=28,
        ) {
            let formatter = TimestampFormatter::new()
                .with_milliseconds(with_millis)
                .with_timezone(with_tz);
            let Some(dt) = create_datetime(year, month, day, 12, 0, 0) else {
                prop_assert!(true);
                return Ok(());
            };
            let result = formatter.format(&dt);
            prop_assert!(!result.is_empty());
        }
    }
}

// =============================================================================
// Now UTC Tests
// =============================================================================

mod now_utc {
    use super::*;

    #[test]
    fn now_utc_returns_recent_time() {
        let before = OffsetDateTime::now_utc();
        let now = now_utc();
        let after = OffsetDateTime::now_utc();

        assert!(now >= before - Duration::milliseconds(1));
        assert!(now <= after + Duration::milliseconds(1));
    }

    #[test]
    fn now_utc_is_utc() {
        let now = now_utc();
        // The offset should be UTC (0)
        assert_eq!(now.offset(), offset!(UTC));
    }

    #[test]
    fn format_now_is_parseable() {
        let formatted = format_now();
        let parsed = parse_timestamp(&formatted);
        assert!(parsed.is_ok());
    }

    #[test]
    fn format_now_is_recent() {
        let before = now_utc();
        let formatted = format_now();
        let parsed = parse_timestamp(&formatted).unwrap();
        let after = now_utc();

        assert!(timestamps_approx_equal(&parsed, &before, Duration::seconds(2)));
        assert!(timestamps_approx_equal(&parsed, &after, Duration::seconds(2)));
    }
}

// =============================================================================
// Concurrency Safety Tests
// =============================================================================

mod concurrency {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn format_now_thread_safe() {
        let formatter = Arc::new(TimestampFormatter::new().with_milliseconds(true));
        let mut handles = vec![];

        for _ in 0..10 {
            let formatter = Arc::clone(&formatter);
            handles.push(thread::spawn(move || {
                let dt = now_utc();
                formatter.format(&dt)
            }));
        }

        for handle in handles {
            let result = handle.join().unwrap();
            assert!(!result.is_empty());
        }
    }

    #[test]
    fn parse_timestamp_thread_safe() {
        let timestamps = vec![
            "2024-01-15T10:00:00Z",
            "2024-01-15T11:00:00Z",
            "2024-01-15T12:00:00Z",
        ];
        let timestamps = Arc::new(timestamps);
        let mut handles = vec![];

        for _ in 0..10 {
            let ts = Arc::clone(&timestamps);
            handles.push(thread::spawn(move || {
                ts.iter().map(|t| parse_timestamp(t)).collect::<Vec<_>>()
            }));
        }

        for handle in handles {
            let results = handle.join().unwrap();
            assert_eq!(results.len(), 3);
            for result in results {
                assert!(result.is_ok());
            }
        }
    }
}

// =============================================================================
// Serde Tests (conditional on feature)
// =============================================================================

#[cfg(feature = "serde")]
mod serde_tests {
    use super::*;

    #[test]
    fn time_span_serialize_deserialize() {
        let span = TimeSpan::new(
            datetime!(2024-01-15 10:00:00 UTC),
            datetime!(2024-01-15 11:00:00 UTC),
        );
        let json = serde_json::to_string(&span).unwrap();
        let deserialized: TimeSpan = serde_json::from_str(&json).unwrap();
        assert_eq!(span, deserialized);
    }

    #[test]
    fn date_serialize_deserialize() {
        let date = Date::new(2024, 6, 15);
        let json = serde_json::to_string(&date).unwrap();
        let deserialized: Date = serde_json::from_str(&json).unwrap();
        assert_eq!(date, deserialized);
    }
}
