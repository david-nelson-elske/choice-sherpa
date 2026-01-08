//! Percentage value object (0-100 scale).

use serde::{Deserialize, Serialize};
use std::fmt;

use super::ValidationError;

/// A value between 0 and 100 inclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Percentage(u8);

impl Percentage {
    /// Zero percent.
    pub const ZERO: Self = Self(0);

    /// One hundred percent.
    pub const HUNDRED: Self = Self(100);

    /// Creates a new Percentage, clamping to valid range.
    pub fn new(value: u8) -> Self {
        Self(value.min(100))
    }

    /// Creates a Percentage, returning error if out of range.
    pub fn try_new(value: u8) -> Result<Self, ValidationError> {
        if value > 100 {
            return Err(ValidationError::out_of_range(
                "percentage",
                0,
                100,
                value as i32,
            ));
        }
        Ok(Self(value))
    }

    /// Returns the value as u8.
    pub fn value(&self) -> u8 {
        self.0
    }

    /// Returns the value as a fraction (0.0 to 1.0).
    pub fn as_fraction(&self) -> f64 {
        f64::from(self.0) / 100.0
    }
}

impl Default for Percentage {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentage_new_accepts_valid_values() {
        assert_eq!(Percentage::new(0).value(), 0);
        assert_eq!(Percentage::new(50).value(), 50);
        assert_eq!(Percentage::new(100).value(), 100);
    }

    #[test]
    fn percentage_new_clamps_to_100() {
        assert_eq!(Percentage::new(101).value(), 100);
        assert_eq!(Percentage::new(255).value(), 100);
    }

    #[test]
    fn percentage_try_new_accepts_valid_values() {
        assert!(Percentage::try_new(0).is_ok());
        assert!(Percentage::try_new(50).is_ok());
        assert!(Percentage::try_new(100).is_ok());
    }

    #[test]
    fn percentage_try_new_rejects_over_100() {
        let result = Percentage::try_new(101);
        assert!(result.is_err());
        match result {
            Err(ValidationError::OutOfRange { field, min, max, actual }) => {
                assert_eq!(field, "percentage");
                assert_eq!(min, 0);
                assert_eq!(max, 100);
                assert_eq!(actual, 101);
            }
            _ => panic!("Expected OutOfRange error"),
        }
    }

    #[test]
    fn percentage_as_fraction_converts_correctly() {
        assert!((Percentage::new(0).as_fraction() - 0.0).abs() < f64::EPSILON);
        assert!((Percentage::new(50).as_fraction() - 0.5).abs() < f64::EPSILON);
        assert!((Percentage::new(100).as_fraction() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn percentage_displays_correctly() {
        assert_eq!(format!("{}", Percentage::new(75)), "75%");
        assert_eq!(format!("{}", Percentage::ZERO), "0%");
        assert_eq!(format!("{}", Percentage::HUNDRED), "100%");
    }

    #[test]
    fn percentage_default_is_zero() {
        assert_eq!(Percentage::default(), Percentage::ZERO);
    }

    #[test]
    fn percentage_serializes_to_json() {
        let pct = Percentage::new(42);
        let json = serde_json::to_string(&pct).unwrap();
        assert_eq!(json, "42");
    }

    #[test]
    fn percentage_deserializes_from_json() {
        let pct: Percentage = serde_json::from_str("75").unwrap();
        assert_eq!(pct.value(), 75);
    }

    #[test]
    fn percentage_ordering_works() {
        let p1 = Percentage::new(25);
        let p2 = Percentage::new(75);
        assert!(p1 < p2);
        assert!(p2 > p1);
    }
}
