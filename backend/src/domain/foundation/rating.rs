//! Rating value object for Pugh matrix (-2 to +2 scale).

use serde::{Deserialize, Serialize};
use std::fmt;

use super::ValidationError;

/// Pugh matrix rating: -2 (much worse) to +2 (much better).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(i8)]
pub enum Rating {
    MuchWorse = -2,
    Worse = -1,
    #[default]
    Same = 0,
    Better = 1,
    MuchBetter = 2,
}

impl Rating {
    /// Creates a Rating from an integer, returning error if out of range.
    pub fn try_from_i8(value: i8) -> Result<Self, ValidationError> {
        match value {
            -2 => Ok(Rating::MuchWorse),
            -1 => Ok(Rating::Worse),
            0 => Ok(Rating::Same),
            1 => Ok(Rating::Better),
            2 => Ok(Rating::MuchBetter),
            _ => Err(ValidationError::out_of_range(
                "rating",
                -2,
                2,
                value as i32,
            )),
        }
    }

    /// Returns the numeric value.
    pub fn value(&self) -> i8 {
        *self as i8
    }

    /// Returns the display label.
    pub fn label(&self) -> &'static str {
        match self {
            Rating::MuchWorse => "Much Worse",
            Rating::Worse => "Worse",
            Rating::Same => "Same",
            Rating::Better => "Better",
            Rating::MuchBetter => "Much Better",
        }
    }

    /// Returns true if this is a positive rating.
    pub fn is_positive(&self) -> bool {
        self.value() > 0
    }

    /// Returns true if this is a negative rating.
    pub fn is_negative(&self) -> bool {
        self.value() < 0
    }

    /// Returns true if this is neutral (Same).
    pub fn is_neutral(&self) -> bool {
        self.value() == 0
    }
}

impl fmt::Display for Rating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.value() > 0 { "+" } else { "" };
        write!(f, "{}{}", sign, self.value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rating_try_from_i8_accepts_valid_values() {
        assert_eq!(Rating::try_from_i8(-2).unwrap(), Rating::MuchWorse);
        assert_eq!(Rating::try_from_i8(-1).unwrap(), Rating::Worse);
        assert_eq!(Rating::try_from_i8(0).unwrap(), Rating::Same);
        assert_eq!(Rating::try_from_i8(1).unwrap(), Rating::Better);
        assert_eq!(Rating::try_from_i8(2).unwrap(), Rating::MuchBetter);
    }

    #[test]
    fn rating_try_from_i8_rejects_invalid_values() {
        assert!(Rating::try_from_i8(-3).is_err());
        assert!(Rating::try_from_i8(3).is_err());
        assert!(Rating::try_from_i8(-10).is_err());
        assert!(Rating::try_from_i8(10).is_err());
    }

    #[test]
    fn rating_value_returns_correct_integer() {
        assert_eq!(Rating::MuchWorse.value(), -2);
        assert_eq!(Rating::Worse.value(), -1);
        assert_eq!(Rating::Same.value(), 0);
        assert_eq!(Rating::Better.value(), 1);
        assert_eq!(Rating::MuchBetter.value(), 2);
    }

    #[test]
    fn rating_label_returns_display_text() {
        assert_eq!(Rating::MuchWorse.label(), "Much Worse");
        assert_eq!(Rating::Worse.label(), "Worse");
        assert_eq!(Rating::Same.label(), "Same");
        assert_eq!(Rating::Better.label(), "Better");
        assert_eq!(Rating::MuchBetter.label(), "Much Better");
    }

    #[test]
    fn rating_is_positive_works() {
        assert!(!Rating::MuchWorse.is_positive());
        assert!(!Rating::Worse.is_positive());
        assert!(!Rating::Same.is_positive());
        assert!(Rating::Better.is_positive());
        assert!(Rating::MuchBetter.is_positive());
    }

    #[test]
    fn rating_is_negative_works() {
        assert!(Rating::MuchWorse.is_negative());
        assert!(Rating::Worse.is_negative());
        assert!(!Rating::Same.is_negative());
        assert!(!Rating::Better.is_negative());
        assert!(!Rating::MuchBetter.is_negative());
    }

    #[test]
    fn rating_is_neutral_works() {
        assert!(!Rating::MuchWorse.is_neutral());
        assert!(Rating::Same.is_neutral());
        assert!(!Rating::MuchBetter.is_neutral());
    }

    #[test]
    fn rating_default_is_same() {
        assert_eq!(Rating::default(), Rating::Same);
    }

    #[test]
    fn rating_displays_with_sign() {
        assert_eq!(format!("{}", Rating::MuchWorse), "-2");
        assert_eq!(format!("{}", Rating::Worse), "-1");
        assert_eq!(format!("{}", Rating::Same), "0");
        assert_eq!(format!("{}", Rating::Better), "+1");
        assert_eq!(format!("{}", Rating::MuchBetter), "+2");
    }

    #[test]
    fn rating_ordering_works() {
        assert!(Rating::MuchWorse < Rating::Worse);
        assert!(Rating::Worse < Rating::Same);
        assert!(Rating::Same < Rating::Better);
        assert!(Rating::Better < Rating::MuchBetter);
    }

    #[test]
    fn rating_serializes_to_json() {
        let rating = Rating::Better;
        let json = serde_json::to_string(&rating).unwrap();
        // Enum variants serialize as their value
        assert_eq!(json, "\"Better\"");
    }

    #[test]
    fn rating_deserializes_from_json() {
        let rating: Rating = serde_json::from_str("\"MuchBetter\"").unwrap();
        assert_eq!(rating, Rating::MuchBetter);
    }
}
