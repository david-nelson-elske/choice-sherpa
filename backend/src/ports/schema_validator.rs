//! Schema Validator Port - Component output validation interface.
//!
//! This port defines the contract for validating PrOACT component outputs
//! against their JSON Schemas. The domain depends on this trait, while
//! adapters (like JsonSchemaValidator) provide the implementation.

use serde_json::Value;
use thiserror::Error;

use crate::domain::foundation::ComponentType;

/// Port for validating component outputs against their schemas.
///
/// # Contract
///
/// Implementations must:
/// - Load and compile JSON Schemas for all 9 component types
/// - Validate full outputs against schema requirements
/// - Support partial validation for incremental extraction
/// - Provide schema access for introspection
///
/// # Usage
///
/// ```rust,ignore
/// let validator: &dyn ComponentSchemaValidator = get_validator();
///
/// // Full validation (all required fields must be present)
/// validator.validate(ComponentType::IssueRaising, &output)?;
///
/// // Partial validation (allows missing required fields)
/// validator.validate_partial(ComponentType::Objectives, &partial_output)?;
///
/// // Get raw schema for client-side validation
/// let schema = validator.schema_for(ComponentType::Alternatives);
/// ```
pub trait ComponentSchemaValidator: Send + Sync {
    /// Validate output against component type's schema.
    ///
    /// Returns `Ok(())` if valid, `Err` with validation errors if not.
    /// All required fields must be present for validation to pass.
    fn validate(
        &self,
        component_type: ComponentType,
        output: &Value,
    ) -> Result<(), SchemaValidationError>;

    /// Get the JSON Schema for a component type.
    ///
    /// Returns the raw schema JSON for introspection or client-side validation.
    /// Schemas are considered public and safe to expose via API.
    fn schema_for(&self, component_type: ComponentType) -> &Value;

    /// Validate partial output (less strict, allows missing optional fields).
    ///
    /// Used during incremental extraction while conversations are in progress.
    /// Empty objects pass validation; present fields are validated for correctness.
    fn validate_partial(
        &self,
        component_type: ComponentType,
        output: &Value,
    ) -> Result<(), SchemaValidationError>;
}

/// Errors that can occur during schema validation.
///
/// # Security
///
/// These errors contain detailed information for debugging. When returning
/// errors to clients, use `to_client_message()` to get sanitized versions
/// that don't expose internal schema structure.
#[derive(Debug, Clone, Error)]
pub enum SchemaValidationError {
    #[error("Missing required field: {field}")]
    MissingRequired { field: String },

    #[error("Invalid type for field {field}: expected {expected}, got {actual}")]
    InvalidType {
        field: String,
        expected: String,
        actual: String,
    },

    #[error("Array too short for field {field}: minimum {min}, got {actual}")]
    ArrayTooShort {
        field: String,
        min: usize,
        actual: usize,
    },

    #[error("Value out of range for field {field}: {value} not in [{min}, {max}]")]
    OutOfRange {
        field: String,
        value: String,
        min: String,
        max: String,
    },

    #[error("Invalid format for field {field}: expected {format}")]
    InvalidFormat { field: String, format: String },

    #[error("Schema validation failed: {message}")]
    Generic { message: String },

    #[error("Validation errors: {0:?}")]
    Multiple(Vec<SchemaValidationError>),
}

impl SchemaValidationError {
    /// Convert to client-safe error message.
    ///
    /// Sanitizes error details to avoid exposing internal schema structure
    /// or implementation details that could aid in exploitation.
    pub fn to_client_message(&self) -> String {
        match self {
            SchemaValidationError::MissingRequired { field } => {
                format!("Missing required field: {}", field)
            }
            SchemaValidationError::InvalidType { field, expected, .. } => {
                format!("Invalid type for field '{}': expected {}", field, expected)
            }
            SchemaValidationError::ArrayTooShort { field, min, .. } => {
                format!("Field '{}' requires at least {} items", field, min)
            }
            SchemaValidationError::OutOfRange { field, min, max, .. } => {
                format!("Field '{}' must be between {} and {}", field, min, max)
            }
            SchemaValidationError::InvalidFormat { field, format } => {
                format!("Field '{}' must be a valid {}", field, format)
            }
            SchemaValidationError::Generic { message } => {
                // Truncate potentially long messages
                if message.len() > 100 {
                    format!("Validation failed: {}...", &message[..97])
                } else {
                    format!("Validation failed: {}", message)
                }
            }
            SchemaValidationError::Multiple(errors) => {
                // Return first error only to avoid information leakage
                errors
                    .first()
                    .map(|e| e.to_client_message())
                    .unwrap_or_else(|| "Validation failed".to_string())
            }
        }
    }

    /// Returns true if this error contains multiple validation failures.
    pub fn is_multiple(&self) -> bool {
        matches!(self, SchemaValidationError::Multiple(_))
    }

    /// Get the count of validation errors.
    pub fn error_count(&self) -> usize {
        match self {
            SchemaValidationError::Multiple(errors) => errors.len(),
            _ => 1,
        }
    }
}

impl PartialEq for SchemaValidationError {
    fn eq(&self, other: &Self) -> bool {
        // Compare by error message for testing purposes
        self.to_string() == other.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_required_displays_field_name() {
        let err = SchemaValidationError::MissingRequired {
            field: "synthesis".to_string(),
        };
        assert_eq!(err.to_string(), "Missing required field: synthesis");
        assert_eq!(err.to_client_message(), "Missing required field: synthesis");
    }

    #[test]
    fn invalid_type_displays_expected_and_actual() {
        let err = SchemaValidationError::InvalidType {
            field: "score".to_string(),
            expected: "integer".to_string(),
            actual: "string".to_string(),
        };
        assert!(err.to_string().contains("expected integer"));
        assert!(err.to_string().contains("got string"));
    }

    #[test]
    fn client_message_for_invalid_type_hides_actual() {
        let err = SchemaValidationError::InvalidType {
            field: "score".to_string(),
            expected: "integer".to_string(),
            actual: "string".to_string(),
        };
        let msg = err.to_client_message();
        assert!(msg.contains("expected integer"));
        assert!(!msg.contains("got")); // Don't expose actual type
    }

    #[test]
    fn array_too_short_shows_minimum() {
        let err = SchemaValidationError::ArrayTooShort {
            field: "alternatives".to_string(),
            min: 2,
            actual: 1,
        };
        assert_eq!(
            err.to_client_message(),
            "Field 'alternatives' requires at least 2 items"
        );
    }

    #[test]
    fn out_of_range_shows_bounds() {
        let err = SchemaValidationError::OutOfRange {
            field: "score".to_string(),
            value: "150".to_string(),
            min: "0".to_string(),
            max: "100".to_string(),
        };
        assert_eq!(
            err.to_client_message(),
            "Field 'score' must be between 0 and 100"
        );
    }

    #[test]
    fn invalid_format_shows_expected_format() {
        let err = SchemaValidationError::InvalidFormat {
            field: "id".to_string(),
            format: "uuid".to_string(),
        };
        assert_eq!(err.to_client_message(), "Field 'id' must be a valid uuid");
    }

    #[test]
    fn multiple_errors_returns_first_in_client_message() {
        let errors = vec![
            SchemaValidationError::MissingRequired {
                field: "first".to_string(),
            },
            SchemaValidationError::MissingRequired {
                field: "second".to_string(),
            },
        ];
        let err = SchemaValidationError::Multiple(errors);
        assert_eq!(err.to_client_message(), "Missing required field: first");
    }

    #[test]
    fn error_count_returns_correct_values() {
        let single = SchemaValidationError::MissingRequired {
            field: "test".to_string(),
        };
        assert_eq!(single.error_count(), 1);

        let multiple = SchemaValidationError::Multiple(vec![
            SchemaValidationError::MissingRequired {
                field: "a".to_string(),
            },
            SchemaValidationError::MissingRequired {
                field: "b".to_string(),
            },
            SchemaValidationError::MissingRequired {
                field: "c".to_string(),
            },
        ]);
        assert_eq!(multiple.error_count(), 3);
    }

    #[test]
    fn generic_error_truncates_long_messages() {
        let long_message = "x".repeat(200);
        let err = SchemaValidationError::Generic {
            message: long_message,
        };
        let client_msg = err.to_client_message();
        assert!(client_msg.len() < 150);
        assert!(client_msg.ends_with("..."));
    }
}
