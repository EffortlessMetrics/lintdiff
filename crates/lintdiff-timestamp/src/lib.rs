//! Timestamp formatting utilities for lintdiff reports.
//!
//! This crate provides consistent timestamp handling across all lintdiff reports,
//! supporting ISO 8601 formatting, parsing, and duration representation.
//!
//! # Features
//!
//! - `serde` - Enable serde serialization for timestamps
//!
//! # Examples
//!
//! ```
//! use lintdiff_timestamp::{format_timestamp, format_now, parse_timestamp, now_utc};
//!
//! // Get current time as ISO 8601
//! let now_str = format_now();
//! assert!(now_str.ends_with('Z') || now_str.contains("+00:00"));
//!
//! // Format a specific timestamp
//! let dt = now_utc();
//! let formatted = format_timestamp(&dt);
//! assert!(!formatted.is_empty());
//!
//! // Parse a timestamp
//! let parsed = parse_timestamp("2024-01-15T10:30:00Z").unwrap();
//! assert_eq!(parsed.year(), 2024);
//! ```

use std::time::Duration as StdDuration;

use thiserror::Error;
use time::format_description::well_known::{Iso8601, Rfc3339};
use time::{Duration, OffsetDateTime, UtcOffset};

/// Error type for timestamp parsing failures.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum TimestampError {
    /// The input string could not be parsed as a valid timestamp.
    #[error("failed to parse timestamp: {0}")]
    ParseError(String),

    /// The timestamp format is not supported.
    #[error("unsupported timestamp format: {0}")]
    UnsupportedFormat(String),

    /// The timestamp value is out of range.
    #[error("timestamp out of range: {0}")]
    OutOfRange(String),
}

impl TimestampError {
    /// Create a new parse error with the given message.
    #[must_use]
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::ParseError(msg.into())
    }

    /// Create a new unsupported format error.
    #[must_use]
    pub fn unsupported(format: impl Into<String>) -> Self {
        Self::UnsupportedFormat(format.into())
    }

    /// Create a new out of range error.
    #[must_use]
    pub fn out_of_range(msg: impl Into<String>) -> Self {
        Self::OutOfRange(msg.into())
    }
}

/// Format a datetime as ISO 8601 without milliseconds.
///
/// Output format: `2024-01-15T10:30:00Z`
///
/// # Examples
///
/// ```
/// use lintdiff_timestamp::{format_timestamp, now_utc};
///
/// let dt = now_utc();
/// let formatted = format_timestamp(&dt);
/// assert!(!formatted.is_empty());
/// ```
#[must_use]
pub fn format_timestamp(dt: &OffsetDateTime) -> String {
    dt.format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

/// Format a datetime as ISO 8601 with milliseconds.
///
/// Output format: `2024-01-15T10:30:00.123Z`
///
/// # Panics
///
/// This function does not panic, but uses a fallback value if formatting fails.
///
/// # Examples
///
/// ```
/// use lintdiff_timestamp::{format_timestamp_millis, now_utc};
///
/// let dt = now_utc();
/// let formatted = format_timestamp_millis(&dt);
/// assert!(formatted.ends_with('Z') || formatted.contains("+00:00"));
/// assert!(formatted.contains('.'));
/// ```
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn format_timestamp_millis(dt: &OffsetDateTime) -> String {
    // Use a static format description to avoid runtime parsing
    const FORMAT: &[time::format_description::BorrowedFormatItem<'static>] = time::macros::format_description!(
        "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z"
    );
    dt.format(FORMAT)
        .unwrap_or_else(|_| "1970-01-01T00:00:00.000Z".to_string())
}

/// Parse an ISO 8601 timestamp string.
///
/// Supports various ISO 8601 formats:
/// - `2024-01-15T10:30:00Z`
/// - `2024-01-15T10:30:00.123Z`
/// - `2024-01-15T10:30:00+00:00`
///
/// # Examples
///
/// ```
/// use lintdiff_timestamp::parse_timestamp;
///
/// let dt = parse_timestamp("2024-01-15T10:30:00Z").unwrap();
/// assert_eq!(dt.year(), 2024);
/// assert_eq!(dt.month() as u8, 1);
/// assert_eq!(dt.day(), 15);
/// ```
///
/// # Errors
///
/// Returns [`TimestampError::ParseError`] if the string cannot be parsed.
pub fn parse_timestamp(s: &str) -> Result<OffsetDateTime, TimestampError> {
    // Try RFC 3339 first (most common format)
    if let Ok(dt) = OffsetDateTime::parse(s, &Rfc3339) {
        return Ok(dt);
    }

    // Try ISO 8601 with various configurations
    if let Ok(dt) = OffsetDateTime::parse(s, &Iso8601::DEFAULT) {
        return Ok(dt);
    }

    // Try parsing as UTC if no timezone specified
    let utc_string = if !s.ends_with('Z') && !s.contains('+') && !s.contains('T') {
        // Handle date-only format
        format!("{s}T00:00:00Z")
    } else if !s.ends_with('Z') && !s.contains('+') && s.contains('T') {
        // Handle datetime without timezone
        format!("{s}Z")
    } else {
        s.to_string()
    };

    OffsetDateTime::parse(&utc_string, &Rfc3339).map_err(|e| TimestampError::parse(e.to_string()))
}

/// Get the current UTC time.
///
/// # Examples
///
/// ```
/// use lintdiff_timestamp::now_utc;
///
/// let dt = now_utc();
/// assert!(dt.year() >= 2024);
/// ```
#[must_use]
pub fn now_utc() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

/// Get the current time as an ISO 8601 formatted string.
///
/// # Examples
///
/// ```
/// use lintdiff_timestamp::format_now;
///
/// let now = format_now();
/// assert!(now.ends_with('Z') || now.contains("+00:00"));
/// ```
#[must_use]
pub fn format_now() -> String {
    format_timestamp(&now_utc())
}

/// Format a duration in a human-readable format.
///
/// Output examples:
/// - `125ms`
/// - `1.5s`
/// - `2m 30s`
/// - `1h 15m 30s`
///
/// # Examples
///
/// ```
/// use lintdiff_timestamp::format_duration;
/// use std::time::Duration;
///
/// assert_eq!(format_duration(&Duration::from_millis(125)), "125ms");
/// assert_eq!(format_duration(&Duration::from_secs(90)), "1m 30s");
/// ```
#[must_use]
pub fn format_duration(d: &StdDuration) -> String {
    let total_secs = d.as_secs();
    let millis = d.subsec_millis();

    if total_secs == 0 {
        return format!("{millis}ms");
    }

    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else if millis > 0 {
        format!("{seconds}.{millis:03}s")
    } else {
        format!("{seconds}s")
    }
}

/// A builder for custom timestamp formatting.
///
/// # Examples
///
/// ```
/// use lintdiff_timestamp::{TimestampFormatter, now_utc};
///
/// let formatter = TimestampFormatter::new()
///     .with_milliseconds(true)
///     .with_timezone(true);
///
/// let dt = now_utc();
/// let formatted = formatter.format(&dt);
/// assert!(formatted.contains('.'));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimestampFormatter {
    include_milliseconds: bool,
    include_timezone: bool,
}

impl Default for TimestampFormatter {
    fn default() -> Self {
        Self {
            include_milliseconds: false,
            include_timezone: true,
        }
    }
}

impl TimestampFormatter {
    /// Create a new formatter with default settings.
    ///
    /// Defaults:
    /// - No milliseconds
    /// - Timezone included
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to include milliseconds in the output.
    #[must_use]
    pub const fn with_milliseconds(mut self, include: bool) -> Self {
        self.include_milliseconds = include;
        self
    }

    /// Set whether to include timezone in the output.
    #[must_use]
    pub const fn with_timezone(mut self, include: bool) -> Self {
        self.include_timezone = include;
        self
    }

    /// Format a datetime according to the configured options.
    ///
    /// # Panics
    ///
    /// This function does not panic, but uses a fallback value if formatting fails.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn format(&self, dt: &OffsetDateTime) -> String {
        if self.include_milliseconds {
            if self.include_timezone {
                format_timestamp_millis(dt)
            } else {
                // Format without timezone
                const FORMAT: &[time::format_description::BorrowedFormatItem<'static>] =
                    time::macros::format_description!(
                        "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]"
                    );
                dt.format(FORMAT)
                    .unwrap_or_else(|_| "1970-01-01T00:00:00.000".to_string())
            }
        } else if self.include_timezone {
            format_timestamp(dt)
        } else {
            const FORMAT: &[time::format_description::BorrowedFormatItem<'static>] =
                time::macros::format_description!(
                    "[year]-[month]-[day]T[hour]:[minute]:[second]"
                );
            dt.format(FORMAT)
                .unwrap_or_else(|_| "1970-01-01T00:00:00".to_string())
        }
    }

    /// Check if milliseconds are included in the output.
    #[must_use]
    pub const fn has_milliseconds(&self) -> bool {
        self.include_milliseconds
    }

    /// Check if timezone is included in the output.
    #[must_use]
    pub const fn has_timezone(&self) -> bool {
        self.include_timezone
    }
}

/// A time span representing a range between two timestamps.
///
/// # Examples
///
/// ```
/// use lintdiff_timestamp::{TimeSpan, now_utc};
/// use time::Duration;
///
/// let start = now_utc();
/// let end = start + Duration::seconds(90);
///
/// let span = TimeSpan::new(start, end);
/// assert_eq!(span.duration(), Duration::seconds(90));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeSpan {
    /// The start of the time span.
    pub start: OffsetDateTime,
    /// The end of the time span.
    pub end: OffsetDateTime,
}

impl TimeSpan {
    /// Create a new time span.
    ///
    /// # Panics
    ///
    /// Panics if `end` is before `start`.
    #[must_use]
    pub fn new(start: OffsetDateTime, end: OffsetDateTime) -> Self {
        assert!(
            end >= start,
            "TimeSpan end must be >= start: end={end:?}, start={start:?}"
        );
        Self { start, end }
    }

    /// Create a time span from a start point and duration.
    #[must_use]
    pub fn from_start_and_duration(start: OffsetDateTime, duration: Duration) -> Self {
        Self {
            start,
            end: start + duration,
        }
    }

    /// Create a time span from an end point and duration.
    #[must_use]
    pub fn from_end_and_duration(end: OffsetDateTime, duration: Duration) -> Self {
        Self {
            start: end - duration,
            end,
        }
    }

    /// Get the duration of this time span.
    #[must_use]
    pub fn duration(&self) -> Duration {
        self.end - self.start
    }

    /// Format the time span as a human-readable range.
    ///
    /// Output format: `2024-01-15T10:30:00Z to 2024-01-15T11:45:00Z`
    #[must_use]
    pub fn format_range(&self) -> String {
        format!(
            "{} to {}",
            format_timestamp(&self.start),
            format_timestamp(&self.end)
        )
    }

    /// Format the time span with duration.
    ///
    /// Output format: `2024-01-15T10:30:00Z to 2024-01-15T11:45:00Z (1h 15m 0s)`
    #[must_use]
    pub fn format_range_with_duration(&self) -> String {
        let duration = format_duration(&self.duration().try_into().unwrap_or(StdDuration::ZERO));
        format!("{} ({})", self.format_range(), duration)
    }

    /// Check if a timestamp falls within this span.
    #[must_use]
    pub fn contains(&self, dt: OffsetDateTime) -> bool {
        dt >= self.start && dt <= self.end
    }

    /// Check if this span overlaps with another span.
    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Get the intersection of two spans, if any.
    #[must_use]
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        if !self.overlaps(other) {
            return None;
        }
        let start = self.start.max(other.start);
        let end = self.end.min(other.end);
        Some(Self { start, end })
    }

    /// Check if this span is empty (zero duration).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Get the midpoint of this time span.
    #[must_use]
    pub fn midpoint(&self) -> OffsetDateTime {
        let half_duration = self.duration() / 2;
        self.start + half_duration
    }
}

/// A date-only representation for cases where time is not needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Date {
    /// Year (e.g., 2024)
    pub year: i32,
    /// Month (1-12)
    pub month: u8,
    /// Day (1-31)
    pub day: u8,
}

impl Date {
    /// Create a new date.
    ///
    /// # Panics
    ///
    /// Panics if the date is invalid (month not 1-12 or invalid day for month).
    #[must_use]
    #[allow(clippy::missing_panics_doc, clippy::panic)]
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        // Validate by creating a time::Date - panic on invalid
        let month_enum = time::Month::try_from(month)
            .inspect_err(|_| panic!("invalid month: {month}"))
            .unwrap_or(time::Month::January);
        time::Date::from_calendar_date(year, month_enum, day)
            .inspect_err(|e| panic!("invalid date: {e}"))
            .unwrap_or(time::Date::MIN);
        Self { year, month, day }
    }

    /// Get today's date in UTC.
    #[must_use]
    pub fn today() -> Self {
        let now = now_utc();
        Self {
            year: now.year(),
            month: now.month() as u8,
            day: now.day(),
        }
    }

    /// Convert to an ISO 8601 date string.
    #[must_use]
    pub fn to_iso8601(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// Parse from an ISO 8601 date string.
    ///
    /// # Errors
    ///
    /// Returns [`TimestampError::ParseError`] if the string cannot be parsed.
    pub fn parse(s: &str) -> Result<Self, TimestampError> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 {
            return Err(TimestampError::parse("expected YYYY-MM-DD format"));
        }
        let year: i32 = parts[0]
            .parse()
            .map_err(|e: std::num::ParseIntError| TimestampError::parse(e.to_string()))?;
        let month: u8 = parts[1]
            .parse()
            .map_err(|e: std::num::ParseIntError| TimestampError::parse(e.to_string()))?;
        let day: u8 = parts[2]
            .parse()
            .map_err(|e: std::num::ParseIntError| TimestampError::parse(e.to_string()))?;
        // Validate the date using time::Date
        time::Date::from_calendar_date(
            year,
            time::Month::try_from(month).map_err(|_| TimestampError::parse("invalid month"))?,
            day,
        )
        .map_err(|e| TimestampError::parse(e.to_string()))?;
        Ok(Self { year, month, day })
    }

    /// Convert to an `OffsetDateTime` at midnight UTC.
    ///
    /// # Panics
    ///
    /// This function does not panic as the date is validated on construction.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn to_midnight_utc(&self) -> OffsetDateTime {
        let month_enum = time::Month::try_from(self.month).unwrap_or(time::Month::January);
        let date = time::Date::from_calendar_date(self.year, month_enum, self.day)
            .unwrap_or(time::Date::MIN);
        date.with_time(time::Time::MIDNIGHT)
            .assume_offset(UtcOffset::UTC)
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// Compare two timestamps for approximate equality within a tolerance.
///
/// # Examples
///
/// ```
/// use lintdiff_timestamp::{now_utc, timestamps_approx_equal};
/// use time::Duration;
///
/// let a = now_utc();
/// let b = a + Duration::milliseconds(100);
///
/// assert!(timestamps_approx_equal(&a, &b, Duration::seconds(1)));
/// assert!(!timestamps_approx_equal(&a, &b, Duration::milliseconds(50)));
/// ```
#[must_use]
pub fn timestamps_approx_equal(a: &OffsetDateTime, b: &OffsetDateTime, tolerance: Duration) -> bool {
    let diff = if a > b { *a - *b } else { *b - *a };
    diff <= tolerance
}

/// Check if a year is a leap year.
///
/// # Examples
///
/// ```
/// use lintdiff_timestamp::is_leap_year;
///
/// assert!(is_leap_year(2024));
/// assert!(!is_leap_year(2023));
/// assert!(is_leap_year(2000));
/// assert!(!is_leap_year(1900));
/// ```
#[must_use]
pub const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Get the number of days in a month.
///
/// # Examples
///
/// ```
/// use lintdiff_timestamp::days_in_month;
///
/// assert_eq!(days_in_month(2024, 2), 29); // Leap year
/// assert_eq!(days_in_month(2023, 2), 28);
/// assert_eq!(days_in_month(2024, 1), 31);
/// assert_eq!(days_in_month(2024, 4), 30);
/// ```
#[must_use]
pub const fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0, // Invalid month
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;
    use time::Duration;

    #[test]
    fn test_format_timestamp_basic() {
        let dt = datetime!(2024-01-15 10:30:00 UTC);
        let formatted = format_timestamp(&dt);
        assert_eq!(formatted, "2024-01-15T10:30:00Z");
    }

    #[test]
    fn test_format_timestamp_millis() {
        let dt = datetime!(2024-01-15 10:30:00.123 UTC);
        let formatted = format_timestamp_millis(&dt);
        assert!(formatted.contains("2024-01-15"));
        assert!(formatted.contains(".123"));
    }

    #[test]
    fn test_parse_timestamp_basic() {
        let dt = parse_timestamp("2024-01-15T10:30:00Z").unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month() as u8, 1);
        assert_eq!(dt.day(), 15);
    }

    #[test]
    fn test_parse_timestamp_with_tz() {
        let dt = parse_timestamp("2024-01-15T10:30:00+00:00").unwrap();
        assert_eq!(dt.year(), 2024);
    }

    #[test]
    fn test_format_duration_millis() {
        assert_eq!(format_duration(&StdDuration::from_millis(125)), "125ms");
        assert_eq!(format_duration(&StdDuration::from_millis(999)), "999ms");
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(&StdDuration::from_secs(1)), "1s");
        assert_eq!(format_duration(&StdDuration::from_secs(30)), "30s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(&StdDuration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(&StdDuration::from_secs(120)), "2m 0s");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(&StdDuration::from_secs(3661)), "1h 1m 1s");
        assert_eq!(format_duration(&StdDuration::from_secs(7200)), "2h 0m 0s");
    }

    #[test]
    fn test_timestamp_formatter_defaults() {
        let formatter = TimestampFormatter::new();
        assert!(!formatter.has_milliseconds());
        assert!(formatter.has_timezone());
    }

    #[test]
    fn test_timestamp_formatter_with_millis() {
        let formatter = TimestampFormatter::new().with_milliseconds(true);
        assert!(formatter.has_milliseconds());
    }

    #[test]
    fn test_time_span_duration() {
        let start = datetime!(2024-01-15 10:00:00 UTC);
        let end = datetime!(2024-01-15 10:01:30 UTC);
        let span = TimeSpan::new(start, end);
        assert_eq!(span.duration(), Duration::seconds(90));
    }

    #[test]
    fn test_time_span_contains() {
        let start = datetime!(2024-01-15 10:00:00 UTC);
        let end = datetime!(2024-01-15 11:00:00 UTC);
        let span = TimeSpan::new(start, end);

        assert!(span.contains(datetime!(2024-01-15 10:30:00 UTC)));
        assert!(span.contains(start));
        assert!(span.contains(end));
        assert!(!span.contains(datetime!(2024-01-15 09:59:59 UTC)));
        assert!(!span.contains(datetime!(2024-01-15 11:00:01 UTC)));
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2024));
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(2023));
        assert!(!is_leap_year(1900));
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2024, 1), 31);
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2024, 4), 30);
    }

    #[test]
    fn test_date_today() {
        let today = Date::today();
        assert!(today.year >= 2024);
        assert!(today.month >= 1 && today.month <= 12);
        assert!(today.day >= 1 && today.day <= 31);
    }

    #[test]
    fn test_date_to_iso8601() {
        let date = Date::new(2024, 1, 15);
        assert_eq!(date.to_iso8601(), "2024-01-15");
    }

    #[test]
    fn test_date_parse() {
        let date = Date::parse("2024-01-15").unwrap();
        assert_eq!(date.year, 2024);
        assert_eq!(date.month, 1);
        assert_eq!(date.day, 15);
    }
}
