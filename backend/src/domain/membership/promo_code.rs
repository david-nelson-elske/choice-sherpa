//! Promo code value object.
//!
//! Represents a promotional code that grants free membership.
//! Format: PREFIX-RANDOM (e.g., WORKSHOP2026-A7K9M3)
//!
//! # Validation Rules
//!
//! - Format: `PREFIX-RANDOM`
//! - PREFIX: 4-20 uppercase alphanumeric characters
//! - RANDOM: 6 uppercase alphanumeric characters
//! - Total length: 11-27 characters (including hyphen)

use crate::domain::foundation::ValidationError;
use serde::{Deserialize, Serialize};

/// A validated promotional code.
///
/// Promo codes grant free membership access and are typically
/// distributed at workshops or promotional events.
///
/// # Format
///
/// `PREFIX-RANDOM` where:
/// - PREFIX: Event/campaign identifier (e.g., "WORKSHOP2026")
/// - RANDOM: 6-character unique suffix (e.g., "A7K9M3")
///
/// # Example
///
/// ```ignore
/// let code = PromoCode::try_new("WORKSHOP2026-A7K9M3")?;
/// assert_eq!(code.prefix(), "WORKSHOP2026");
/// assert_eq!(code.suffix(), "A7K9M3");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PromoCode {
    /// The full promo code string (normalized to uppercase).
    code: String,
    /// The prefix portion before the hyphen.
    prefix: String,
    /// The random suffix after the hyphen.
    suffix: String,
}

impl PromoCode {
    /// Creates a new PromoCode from a string, validating the format.
    ///
    /// # Errors
    ///
    /// Returns `ValidationError` if:
    /// - Code is empty
    /// - Code doesn't contain exactly one hyphen
    /// - Prefix is too short (< 4 chars) or too long (> 20 chars)
    /// - Suffix is not exactly 6 characters
    /// - Characters are not alphanumeric
    pub fn try_new(code: &str) -> Result<Self, ValidationError> {
        // 1. Check not empty
        if code.is_empty() {
            return Err(ValidationError::empty_field("promo_code"));
        }

        // 2. Normalize to uppercase
        let normalized = code.to_uppercase();

        // 3. Split on hyphen
        let parts: Vec<&str> = normalized.split('-').collect();
        if parts.len() != 2 {
            return Err(ValidationError::invalid_format(
                "promo_code",
                format!("expected format PREFIX-RANDOM, got '{}'", normalized),
            ));
        }

        // Convert to owned strings early to avoid borrow issues
        let prefix = parts[0].to_string();
        let suffix = parts[1].to_string();

        // 4. Validate prefix length (4-20 chars)
        if prefix.len() < 4 || prefix.len() > 20 {
            return Err(ValidationError::out_of_range(
                "promo_code_prefix_length",
                4,
                20,
                prefix.len() as i32,
            ));
        }

        // 5. Validate suffix length (exactly 6 chars)
        if suffix.len() != 6 {
            return Err(ValidationError::out_of_range(
                "promo_code_suffix_length",
                6,
                6,
                suffix.len() as i32,
            ));
        }

        // 6. Validate prefix is alphanumeric
        if !prefix.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(ValidationError::invalid_format(
                "promo_code_prefix",
                "alphanumeric characters only",
            ));
        }

        // 7. Validate suffix is alphanumeric
        if !suffix.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(ValidationError::invalid_format(
                "promo_code_suffix",
                "alphanumeric characters only",
            ));
        }

        Ok(Self {
            code: normalized,
            prefix,
            suffix,
        })
    }

    /// Returns the full promo code string.
    pub fn as_str(&self) -> &str {
        &self.code
    }

    /// Returns the prefix portion of the code.
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// Returns the suffix (random) portion of the code.
    pub fn suffix(&self) -> &str {
        &self.suffix
    }
}

impl std::fmt::Display for PromoCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}

impl TryFrom<&str> for PromoCode {
    type Error = ValidationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl TryFrom<String> for PromoCode {
    type Error = ValidationError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_new(&value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ════════════════════════════════════════════════════════════════════════════
    // Valid Code Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn valid_workshop_code_parses_successfully() {
        let code = PromoCode::try_new("WORKSHOP2026-A7K9M3").unwrap();
        assert_eq!(code.as_str(), "WORKSHOP2026-A7K9M3");
        assert_eq!(code.prefix(), "WORKSHOP2026");
        assert_eq!(code.suffix(), "A7K9M3");
    }

    #[test]
    fn valid_short_prefix_parses() {
        // Minimum prefix length: 4
        let code = PromoCode::try_new("BETA-ABC123").unwrap();
        assert_eq!(code.prefix(), "BETA");
        assert_eq!(code.suffix(), "ABC123");
    }

    #[test]
    fn valid_long_prefix_parses() {
        // Maximum prefix length: 20
        let code = PromoCode::try_new("SUPERLONGPREFIX12345-XYZ789").unwrap();
        assert_eq!(code.prefix(), "SUPERLONGPREFIX12345");
        assert_eq!(code.suffix(), "XYZ789");
    }

    #[test]
    fn lowercase_input_normalizes_to_uppercase() {
        let code = PromoCode::try_new("workshop2026-a7k9m3").unwrap();
        assert_eq!(code.as_str(), "WORKSHOP2026-A7K9M3");
        assert_eq!(code.prefix(), "WORKSHOP2026");
        assert_eq!(code.suffix(), "A7K9M3");
    }

    #[test]
    fn mixed_case_input_normalizes() {
        let code = PromoCode::try_new("WorkShop2026-A7k9M3").unwrap();
        assert_eq!(code.as_str(), "WORKSHOP2026-A7K9M3");
    }

    #[test]
    fn numeric_prefix_is_valid() {
        let code = PromoCode::try_new("2026CONF-123ABC").unwrap();
        assert_eq!(code.prefix(), "2026CONF");
    }

    #[test]
    fn numeric_suffix_is_valid() {
        let code = PromoCode::try_new("EVENT-123456").unwrap();
        assert_eq!(code.suffix(), "123456");
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Invalid Code Tests - Format
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn empty_code_returns_error() {
        let result = PromoCode::try_new("");
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::EmptyField { field } => assert_eq!(field, "promo_code"),
            _ => panic!("Expected EmptyField error"),
        }
    }

    #[test]
    fn code_without_hyphen_returns_error() {
        let result = PromoCode::try_new("WORKSHOP2026A7K9M3");
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::InvalidFormat { field, .. } => assert_eq!(field, "promo_code"),
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn code_with_multiple_hyphens_returns_error() {
        let result = PromoCode::try_new("WORK-SHOP-A7K9M3");
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::InvalidFormat { field, .. } => assert_eq!(field, "promo_code"),
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Invalid Code Tests - Prefix Length
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn prefix_too_short_returns_error() {
        // Prefix "ABC" is only 3 chars, minimum is 4
        let result = PromoCode::try_new("ABC-123456");
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::OutOfRange { field, min, max, actual } => {
                assert_eq!(field, "promo_code_prefix_length");
                assert_eq!(min, 4);
                assert_eq!(max, 20);
                assert_eq!(actual, 3);
            }
            _ => panic!("Expected OutOfRange error"),
        }
    }

    #[test]
    fn prefix_too_long_returns_error() {
        // Prefix with 21 chars, maximum is 20
        let result = PromoCode::try_new("SUPERLONGPREFIX123456-ABCDEF");
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::OutOfRange { field, .. } => {
                assert_eq!(field, "promo_code_prefix_length");
            }
            _ => panic!("Expected OutOfRange error"),
        }
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Invalid Code Tests - Suffix Length
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn suffix_too_short_returns_error() {
        // Suffix "ABC12" is only 5 chars, must be exactly 6
        let result = PromoCode::try_new("WORKSHOP-ABC12");
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::OutOfRange { field, min, max, actual } => {
                assert_eq!(field, "promo_code_suffix_length");
                assert_eq!(min, 6);
                assert_eq!(max, 6);
                assert_eq!(actual, 5);
            }
            _ => panic!("Expected OutOfRange error"),
        }
    }

    #[test]
    fn suffix_too_long_returns_error() {
        // Suffix "ABC1234" is 7 chars, must be exactly 6
        let result = PromoCode::try_new("WORKSHOP-ABC1234");
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::OutOfRange { field, .. } => {
                assert_eq!(field, "promo_code_suffix_length");
            }
            _ => panic!("Expected OutOfRange error"),
        }
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Invalid Code Tests - Character Validation
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn prefix_with_special_chars_returns_error() {
        let result = PromoCode::try_new("WORK@SHOP-A7K9M3");
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::InvalidFormat { field, .. } => {
                assert_eq!(field, "promo_code_prefix");
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn suffix_with_special_chars_returns_error() {
        let result = PromoCode::try_new("WORKSHOP-A7K@M3");
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::InvalidFormat { field, .. } => {
                assert_eq!(field, "promo_code_suffix");
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn prefix_with_spaces_returns_error() {
        let result = PromoCode::try_new("WORK SHOP-A7K9M3");
        assert!(result.is_err());
    }

    #[test]
    fn suffix_with_spaces_returns_error() {
        let result = PromoCode::try_new("WORKSHOP-A7 9M3");
        assert!(result.is_err());
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Display and Conversion Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn display_shows_full_code() {
        let code = PromoCode::try_new("WORKSHOP2026-A7K9M3").unwrap();
        assert_eq!(format!("{}", code), "WORKSHOP2026-A7K9M3");
    }

    #[test]
    fn try_from_str_works() {
        let code: PromoCode = "WORKSHOP2026-A7K9M3".try_into().unwrap();
        assert_eq!(code.as_str(), "WORKSHOP2026-A7K9M3");
    }

    #[test]
    fn try_from_string_works() {
        let code: PromoCode = "WORKSHOP2026-A7K9M3".to_string().try_into().unwrap();
        assert_eq!(code.as_str(), "WORKSHOP2026-A7K9M3");
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Serialization Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn serializes_to_json() {
        let code = PromoCode::try_new("WORKSHOP2026-A7K9M3").unwrap();
        let json = serde_json::to_string(&code).unwrap();
        // Should serialize all fields
        assert!(json.contains("WORKSHOP2026-A7K9M3"));
        assert!(json.contains("WORKSHOP2026"));
        assert!(json.contains("A7K9M3"));
    }

    #[test]
    fn deserializes_from_json() {
        let json = r#"{"code":"WORKSHOP2026-A7K9M3","prefix":"WORKSHOP2026","suffix":"A7K9M3"}"#;
        let code: PromoCode = serde_json::from_str(json).unwrap();
        assert_eq!(code.as_str(), "WORKSHOP2026-A7K9M3");
    }

    // ════════════════════════════════════════════════════════════════════════════
    // Equality Tests
    // ════════════════════════════════════════════════════════════════════════════

    #[test]
    fn same_codes_are_equal() {
        let code1 = PromoCode::try_new("WORKSHOP2026-A7K9M3").unwrap();
        let code2 = PromoCode::try_new("WORKSHOP2026-A7K9M3").unwrap();
        assert_eq!(code1, code2);
    }

    #[test]
    fn different_codes_are_not_equal() {
        let code1 = PromoCode::try_new("WORKSHOP2026-A7K9M3").unwrap();
        let code2 = PromoCode::try_new("WORKSHOP2026-B8L0N4").unwrap();
        assert_ne!(code1, code2);
    }

    #[test]
    fn normalized_codes_are_equal() {
        let code1 = PromoCode::try_new("workshop2026-a7k9m3").unwrap();
        let code2 = PromoCode::try_new("WORKSHOP2026-A7K9M3").unwrap();
        assert_eq!(code1, code2);
    }
}
