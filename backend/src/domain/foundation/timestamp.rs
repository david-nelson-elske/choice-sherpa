//! Timestamp value object for immutable points in time.

use chrono::{DateTime, Utc};
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
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;
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
}
