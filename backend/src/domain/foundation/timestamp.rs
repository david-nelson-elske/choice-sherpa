//! Timestamp value object for immutable points in time.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Immutable point in time, always UTC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    /// Creates a timestamp for the current moment.
    pub fn now() -> Self {
        Self(Utc::now())
    }

    /// Creates a timestamp from a DateTime<Utc>.
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }

    /// Returns the inner DateTime.
    pub fn as_datetime(&self) -> &DateTime<Utc> {
        &self.0
    }

    /// Checks if this timestamp is before another.
    pub fn is_before(&self, other: &Timestamp) -> bool {
        self.0 < other.0
    }

    /// Checks if this timestamp is after another.
    pub fn is_after(&self, other: &Timestamp) -> bool {
        self.0 > other.0
    }

    /// Returns the duration from another timestamp to this one.
    ///
    /// Returns negative duration if other is after self.
    pub fn duration_since(&self, other: &Timestamp) -> Duration {
        self.0.signed_duration_since(other.0)
    }

    /// Creates a new timestamp by adding the specified number of days.
    ///
    /// Negative values subtract days.
    pub fn add_days(&self, days: i64) -> Self {
        Self(self.0 + Duration::days(days))
    }

    /// Creates a new timestamp by adding the specified number of months.
    ///
    /// Note: Uses 30 days per month approximation.
    pub fn add_months(&self, months: i64) -> Self {
        Self(self.0 + Duration::days(months * 30))
    }

    /// Creates a new timestamp by subtracting the specified number of days.
    pub fn minus_days(&self, days: i64) -> Self {
        Self(self.0 - Duration::days(days))
    }

    /// Creates a new timestamp by adding the specified number of days.
    ///
    /// Alias for `add_days` with clearer naming for positive offsets.
    pub fn plus_days(&self, days: i64) -> Self {
        Self(self.0 + Duration::days(days))
    }

    /// Returns a timestamp for the start of today (00:00:00 UTC).
    pub fn start_of_today() -> Self {
        let now = Utc::now();
        let start = now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        Self(start)
    }

    /// Creates a timestamp from Unix seconds.
    pub fn from_unix_secs(secs: u64) -> Self {
        use chrono::TimeZone;
        Self(Utc.timestamp_opt(secs as i64, 0).unwrap())
    }

    /// Returns the timestamp as Unix seconds.
    pub fn as_unix_secs(&self) -> u64 {
        self.0.timestamp() as u64
    }

    /// Creates a new timestamp by adding the specified number of seconds.
    pub fn plus_secs(&self, secs: u64) -> Self {
        Self(self.0 + Duration::seconds(secs as i64))
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn timestamp_now_creates_current_time() {
        let before = Utc::now();
        let ts = Timestamp::now();
        let after = Utc::now();

        assert!(ts.as_datetime() >= &before);
        assert!(ts.as_datetime() <= &after);
    }

    #[test]
    fn timestamp_from_datetime_preserves_value() {
        let dt = Utc::now();
        let ts = Timestamp::from_datetime(dt);
        assert_eq!(ts.as_datetime(), &dt);
    }

    #[test]
    fn timestamp_is_before_works_correctly() {
        let ts1 = Timestamp::now();
        sleep(Duration::from_millis(10));
        let ts2 = Timestamp::now();

        assert!(ts1.is_before(&ts2));
        assert!(!ts2.is_before(&ts1));
    }

    #[test]
    fn timestamp_is_after_works_correctly() {
        let ts1 = Timestamp::now();
        sleep(Duration::from_millis(10));
        let ts2 = Timestamp::now();

        assert!(ts2.is_after(&ts1));
        assert!(!ts1.is_after(&ts2));
    }

    #[test]
    fn timestamp_serializes_to_json() {
        let dt = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let ts = Timestamp::from_datetime(dt);

        let json = serde_json::to_string(&ts).unwrap();
        assert!(json.contains("2024-01-15"));
    }

    #[test]
    fn timestamp_deserializes_from_json() {
        let json = "\"2024-01-15T10:30:00Z\"";
        let ts: Timestamp = serde_json::from_str(json).unwrap();

        assert_eq!(ts.as_datetime().year(), 2024);
    }

    #[test]
    fn timestamp_ordering_works() {
        let ts1 = Timestamp::now();
        sleep(Duration::from_millis(10));
        let ts2 = Timestamp::now();

        assert!(ts1 < ts2);
        assert!(ts2 > ts1);
    }

    #[test]
    fn timestamp_from_unix_secs_works() {
        // 2024-01-15T00:00:00Z
        let ts = Timestamp::from_unix_secs(1705276800);
        assert_eq!(ts.as_datetime().year(), 2024);
        assert_eq!(ts.as_datetime().month(), 1);
        assert_eq!(ts.as_datetime().day(), 15);
    }

    #[test]
    fn timestamp_as_unix_secs_roundtrips() {
        let unix_secs = 1705276800_u64;
        let ts = Timestamp::from_unix_secs(unix_secs);
        assert_eq!(ts.as_unix_secs(), unix_secs);
    }

    #[test]
    fn timestamp_plus_secs_adds_correctly() {
        let ts1 = Timestamp::from_unix_secs(1000);
        let ts2 = ts1.plus_secs(60);
        assert_eq!(ts2.as_unix_secs(), 1060);
    }
}
