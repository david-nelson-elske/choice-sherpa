//! JSON Schema Validator - Implementation of ComponentSchemaValidator.
//!
//! Uses manual validation against embedded JSON Schema definitions.
//! Validates component outputs without external schema validation dependencies.

use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde_json::Value;
use uuid::Uuid;

use crate::domain::foundation::ComponentType;
use crate::ports::{ComponentSchemaValidator, SchemaValidationError};

/// JSON Schema-based validator implementation.
///
/// Loads all 9 component schemas and validates outputs manually.
/// Schemas are embedded in the binary via `include_str!` for reliability.
///
/// # Thread Safety
///
/// This struct is `Send + Sync` and can be shared across threads.
pub struct JsonSchemaValidator {
    // No runtime state needed - all validation is based on static schema definitions
}

impl Default for JsonSchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonSchemaValidator {
    /// Create a new validator with all schemas loaded.
    pub fn new() -> Self {
        Self {}
    }

    /// Load raw schema JSON for a component type.
    fn load_raw_schema(ct: ComponentType) -> Value {
        let schema_str = match ct {
            ComponentType::IssueRaising => {
                include_str!("../../domain/proact/schemas/issue_raising.json")
            }
            ComponentType::ProblemFrame => {
                include_str!("../../domain/proact/schemas/problem_frame.json")
            }
            ComponentType::Objectives => {
                include_str!("../../domain/proact/schemas/objectives.json")
            }
            ComponentType::Alternatives => {
                include_str!("../../domain/proact/schemas/alternatives.json")
            }
            ComponentType::Consequences => {
                include_str!("../../domain/proact/schemas/consequences.json")
            }
            ComponentType::Tradeoffs => {
                include_str!("../../domain/proact/schemas/tradeoffs.json")
            }
            ComponentType::Recommendation => {
                include_str!("../../domain/proact/schemas/recommendation.json")
            }
            ComponentType::DecisionQuality => {
                include_str!("../../domain/proact/schemas/decision_quality.json")
            }
            ComponentType::NotesNextSteps => {
                include_str!("../../domain/proact/schemas/notes_next_steps.json")
            }
        };

        serde_json::from_str(schema_str)
            .unwrap_or_else(|e| panic!("Failed to parse schema for {:?}: {}", ct, e))
    }

    /// Validate a value against a component type's schema.
    fn validate_component(
        &self,
        ct: ComponentType,
        output: &Value,
    ) -> Result<(), SchemaValidationError> {
        match ct {
            ComponentType::IssueRaising => self.validate_issue_raising(output),
            ComponentType::ProblemFrame => self.validate_problem_frame(output),
            ComponentType::Objectives => self.validate_objectives(output),
            ComponentType::Alternatives => self.validate_alternatives(output),
            ComponentType::Consequences => self.validate_consequences(output),
            ComponentType::Tradeoffs => self.validate_tradeoffs(output),
            ComponentType::Recommendation => self.validate_recommendation(output),
            ComponentType::DecisionQuality => self.validate_decision_quality(output),
            ComponentType::NotesNextSteps => self.validate_notes_next_steps(output),
        }
    }

    // =========================================================================
    // Component-specific validators
    // =========================================================================

    fn validate_issue_raising(&self, output: &Value) -> Result<(), SchemaValidationError> {
        let obj = self.require_object(output, "root")?;
        let mut errors = Vec::new();

        // Required fields
        for field in &[
            "potential_decisions",
            "objectives",
            "uncertainties",
            "considerations",
        ] {
            if !obj.contains_key(*field) {
                errors.push(SchemaValidationError::MissingRequired {
                    field: field.to_string(),
                });
            }
        }

        if !errors.is_empty() {
            return Err(Self::collect_errors(errors));
        }

        // Validate arrays
        if let Some(arr) = obj.get("potential_decisions").and_then(|v| v.as_array()) {
            for (i, item) in arr.iter().enumerate() {
                self.validate_potential_decision(item, &format!("potential_decisions[{}]", i))?;
            }
        }

        if let Some(arr) = obj.get("objectives").and_then(|v| v.as_array()) {
            for (i, item) in arr.iter().enumerate() {
                self.validate_identified_objective(item, &format!("objectives[{}]", i))?;
            }
        }

        if let Some(arr) = obj.get("uncertainties").and_then(|v| v.as_array()) {
            for (i, item) in arr.iter().enumerate() {
                self.validate_uncertainty_item(item, &format!("uncertainties[{}]", i))?;
            }
        }

        if let Some(arr) = obj.get("considerations").and_then(|v| v.as_array()) {
            for (i, item) in arr.iter().enumerate() {
                self.validate_consideration(item, &format!("considerations[{}]", i))?;
            }
        }

        Ok(())
    }

    fn validate_potential_decision(
        &self,
        value: &Value,
        path: &str,
    ) -> Result<(), SchemaValidationError> {
        let obj = self.require_object(value, path)?;
        self.require_uuid_field(obj, "id", path)?;
        self.require_non_empty_string(obj, "description", path)?;
        if let Some(priority) = obj.get("priority") {
            self.validate_enum(priority, &["high", "medium", "low"], &format!("{}.priority", path))?;
        }
        Ok(())
    }

    fn validate_identified_objective(
        &self,
        value: &Value,
        path: &str,
    ) -> Result<(), SchemaValidationError> {
        let obj = self.require_object(value, path)?;
        self.require_uuid_field(obj, "id", path)?;
        self.require_non_empty_string(obj, "description", path)?;
        Ok(())
    }

    fn validate_uncertainty_item(
        &self,
        value: &Value,
        path: &str,
    ) -> Result<(), SchemaValidationError> {
        let obj = self.require_object(value, path)?;
        self.require_uuid_field(obj, "id", path)?;
        self.require_non_empty_string(obj, "description", path)?;
        Ok(())
    }

    fn validate_consideration(
        &self,
        value: &Value,
        path: &str,
    ) -> Result<(), SchemaValidationError> {
        let obj = self.require_object(value, path)?;
        self.require_uuid_field(obj, "id", path)?;
        self.require_non_empty_string(obj, "text", path)?;
        Ok(())
    }

    fn validate_problem_frame(&self, output: &Value) -> Result<(), SchemaValidationError> {
        let obj = self.require_object(output, "root")?;

        // Required fields
        self.require_field(obj, "decision_maker", "root")?;
        self.require_field(obj, "focal_decision", "root")?;
        self.require_field(obj, "decision_hierarchy", "root")?;

        // Validate decision_maker
        if let Some(dm) = obj.get("decision_maker") {
            let dm_obj = self.require_object(dm, "decision_maker")?;
            self.require_non_empty_string(dm_obj, "name", "decision_maker")?;
            self.require_string_field(dm_obj, "role", "decision_maker")?;
        }

        // Validate focal_decision
        if let Some(fd) = obj.get("focal_decision") {
            let fd_obj = self.require_object(fd, "focal_decision")?;
            self.require_string_min_length(fd_obj, "statement", 10, "focal_decision")?;
            self.require_string_field(fd_obj, "scope", "focal_decision")?;
        }

        // Validate decision_hierarchy
        if let Some(dh) = obj.get("decision_hierarchy") {
            let dh_obj = self.require_object(dh, "decision_hierarchy")?;
            self.require_field(dh_obj, "already_made", "decision_hierarchy")?;
            self.require_field(dh_obj, "focal", "decision_hierarchy")?;
            self.require_field(dh_obj, "deferred", "decision_hierarchy")?;
        }

        Ok(())
    }

    fn validate_objectives(&self, output: &Value) -> Result<(), SchemaValidationError> {
        let obj = self.require_object(output, "root")?;

        // Required fields
        self.require_field(obj, "fundamental_objectives", "root")?;
        self.require_field(obj, "means_objectives", "root")?;

        // fundamental_objectives must have at least 1 item
        if let Some(arr) = obj.get("fundamental_objectives").and_then(|v| v.as_array()) {
            if arr.is_empty() {
                return Err(SchemaValidationError::ArrayTooShort {
                    field: "fundamental_objectives".to_string(),
                    min: 1,
                    actual: 0,
                });
            }
            for (i, item) in arr.iter().enumerate() {
                self.validate_fundamental_objective(item, &format!("fundamental_objectives[{}]", i))?;
            }
        }

        if let Some(arr) = obj.get("means_objectives").and_then(|v| v.as_array()) {
            for (i, item) in arr.iter().enumerate() {
                self.validate_means_objective(item, &format!("means_objectives[{}]", i))?;
            }
        }

        Ok(())
    }

    fn validate_fundamental_objective(
        &self,
        value: &Value,
        path: &str,
    ) -> Result<(), SchemaValidationError> {
        let obj = self.require_object(value, path)?;
        self.require_uuid_field(obj, "id", path)?;
        self.require_non_empty_string(obj, "description", path)?;
        if let Some(weight) = obj.get("weight") {
            if let Some(w) = weight.as_f64() {
                if !(0.0..=1.0).contains(&w) {
                    return Err(SchemaValidationError::OutOfRange {
                        field: format!("{}.weight", path),
                        value: w.to_string(),
                        min: "0".to_string(),
                        max: "1".to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    fn validate_means_objective(
        &self,
        value: &Value,
        path: &str,
    ) -> Result<(), SchemaValidationError> {
        let obj = self.require_object(value, path)?;
        self.require_uuid_field(obj, "id", path)?;
        self.require_non_empty_string(obj, "description", path)?;
        let supports_path = format!("{}.supports", path);
        self.require_field(obj, "supports", path)?;
        if let Some(arr) = obj.get("supports").and_then(|v| v.as_array()) {
            if arr.is_empty() {
                return Err(SchemaValidationError::ArrayTooShort {
                    field: supports_path,
                    min: 1,
                    actual: 0,
                });
            }
        }
        Ok(())
    }

    fn validate_alternatives(&self, output: &Value) -> Result<(), SchemaValidationError> {
        let obj = self.require_object(output, "root")?;

        // Required fields
        self.require_field(obj, "alternatives", "root")?;
        self.require_uuid_field(obj, "status_quo_id", "root")?;

        // alternatives must have at least 2 items
        if let Some(arr) = obj.get("alternatives").and_then(|v| v.as_array()) {
            if arr.len() < 2 {
                return Err(SchemaValidationError::ArrayTooShort {
                    field: "alternatives".to_string(),
                    min: 2,
                    actual: arr.len(),
                });
            }
            for (i, item) in arr.iter().enumerate() {
                self.validate_alternative(item, &format!("alternatives[{}]", i))?;
            }
        }

        Ok(())
    }

    fn validate_alternative(
        &self,
        value: &Value,
        path: &str,
    ) -> Result<(), SchemaValidationError> {
        let obj = self.require_object(value, path)?;
        self.require_uuid_field(obj, "id", path)?;
        self.require_non_empty_string(obj, "name", path)?;
        self.require_string_field(obj, "description", path)?;
        Ok(())
    }

    fn validate_consequences(&self, output: &Value) -> Result<(), SchemaValidationError> {
        let obj = self.require_object(output, "root")?;

        // Required: table
        self.require_field(obj, "table", "root")?;

        if let Some(table) = obj.get("table") {
            let table_obj = self.require_object(table, "table")?;

            // Required fields in table
            self.require_field(table_obj, "alternative_ids", "table")?;
            self.require_field(table_obj, "objective_ids", "table")?;
            self.require_field(table_obj, "cells", "table")?;

            // alternative_ids must have at least 2
            if let Some(arr) = table_obj.get("alternative_ids").and_then(|v| v.as_array()) {
                if arr.len() < 2 {
                    return Err(SchemaValidationError::ArrayTooShort {
                        field: "table.alternative_ids".to_string(),
                        min: 2,
                        actual: arr.len(),
                    });
                }
            }

            // objective_ids must have at least 1
            if let Some(arr) = table_obj.get("objective_ids").and_then(|v| v.as_array()) {
                if arr.is_empty() {
                    return Err(SchemaValidationError::ArrayTooShort {
                        field: "table.objective_ids".to_string(),
                        min: 1,
                        actual: 0,
                    });
                }
            }

            // Validate cells
            if let Some(cells) = table_obj.get("cells").and_then(|v| v.as_object()) {
                for (key, cell) in cells {
                    self.validate_consequence_cell(cell, &format!("table.cells.{}", key))?;
                }
            }
        }

        Ok(())
    }

    fn validate_consequence_cell(
        &self,
        value: &Value,
        path: &str,
    ) -> Result<(), SchemaValidationError> {
        let obj = self.require_object(value, path)?;
        self.require_uuid_field(obj, "alternative_id", path)?;
        self.require_uuid_field(obj, "objective_id", path)?;

        // rating is required and must be -2 to 2
        self.require_field(obj, "rating", path)?;
        if let Some(rating) = obj.get("rating") {
            if let Some(r) = rating.as_i64() {
                if !(-2..=2).contains(&r) {
                    return Err(SchemaValidationError::OutOfRange {
                        field: format!("{}.rating", path),
                        value: r.to_string(),
                        min: "-2".to_string(),
                        max: "2".to_string(),
                    });
                }
            } else {
                return Err(SchemaValidationError::InvalidType {
                    field: format!("{}.rating", path),
                    expected: "integer".to_string(),
                    actual: Self::type_name(rating),
                });
            }
        }

        Ok(())
    }

    fn validate_tradeoffs(&self, output: &Value) -> Result<(), SchemaValidationError> {
        // All fields are optional in tradeoffs
        let _obj = self.require_object(output, "root")?;
        Ok(())
    }

    fn validate_recommendation(&self, output: &Value) -> Result<(), SchemaValidationError> {
        let obj = self.require_object(output, "root")?;

        // synthesis is required with minLength 50
        self.require_string_min_length(obj, "synthesis", 50, "root")?;

        Ok(())
    }

    fn validate_decision_quality(&self, output: &Value) -> Result<(), SchemaValidationError> {
        let obj = self.require_object(output, "root")?;

        // elements is required
        self.require_field(obj, "elements", "root")?;

        // elements must have exactly 7 items
        if let Some(arr) = obj.get("elements").and_then(|v| v.as_array()) {
            if arr.len() < 7 {
                return Err(SchemaValidationError::ArrayTooShort {
                    field: "elements".to_string(),
                    min: 7,
                    actual: arr.len(),
                });
            }
            if arr.len() > 7 {
                return Err(SchemaValidationError::Generic {
                    message: format!("elements must have exactly 7 items, got {}", arr.len()),
                });
            }
            for (i, item) in arr.iter().enumerate() {
                self.validate_dq_element(item, &format!("elements[{}]", i))?;
            }
        }

        // overall_score if present must be 0-100
        if let Some(score) = obj.get("overall_score") {
            if let Some(s) = score.as_i64() {
                if !(0..=100).contains(&s) {
                    return Err(SchemaValidationError::OutOfRange {
                        field: "overall_score".to_string(),
                        value: s.to_string(),
                        min: "0".to_string(),
                        max: "100".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    fn validate_dq_element(
        &self,
        value: &Value,
        path: &str,
    ) -> Result<(), SchemaValidationError> {
        let obj = self.require_object(value, path)?;

        // name is required and must be one of the valid element names
        self.require_field(obj, "name", path)?;
        if let Some(name) = obj.get("name") {
            self.validate_enum(
                name,
                &[
                    "Helpful Problem Frame",
                    "Clear Objectives",
                    "Creative Alternatives",
                    "Reliable Consequence Information",
                    "Logically Correct Reasoning",
                    "Clear Tradeoffs",
                    "Commitment to Follow Through",
                ],
                &format!("{}.name", path),
            )?;
        }

        // score is required and must be 0-100
        self.require_field(obj, "score", path)?;
        if let Some(score) = obj.get("score") {
            if let Some(s) = score.as_i64() {
                if !(0..=100).contains(&s) {
                    return Err(SchemaValidationError::OutOfRange {
                        field: format!("{}.score", path),
                        value: s.to_string(),
                        min: "0".to_string(),
                        max: "100".to_string(),
                    });
                }
            } else {
                return Err(SchemaValidationError::InvalidType {
                    field: format!("{}.score", path),
                    expected: "integer".to_string(),
                    actual: Self::type_name(score),
                });
            }
        }

        Ok(())
    }

    fn validate_notes_next_steps(&self, output: &Value) -> Result<(), SchemaValidationError> {
        // All fields are optional in notes_next_steps
        let _obj = self.require_object(output, "root")?;

        // Validate planned_actions if present
        if let Some(obj) = output.as_object() {
            if let Some(arr) = obj.get("planned_actions").and_then(|v| v.as_array()) {
                for (i, item) in arr.iter().enumerate() {
                    self.validate_planned_action(item, &format!("planned_actions[{}]", i))?;
                }
            }
        }

        Ok(())
    }

    fn validate_planned_action(
        &self,
        value: &Value,
        path: &str,
    ) -> Result<(), SchemaValidationError> {
        let obj = self.require_object(value, path)?;
        self.require_non_empty_string(obj, "action", path)?;
        if let Some(status) = obj.get("status") {
            self.validate_enum(status, &["planned", "in_progress", "completed"], &format!("{}.status", path))?;
        }
        Ok(())
    }

    // =========================================================================
    // Helper methods
    // =========================================================================

    fn require_object<'a>(
        &self,
        value: &'a Value,
        path: &str,
    ) -> Result<&'a serde_json::Map<String, Value>, SchemaValidationError> {
        value.as_object().ok_or_else(|| SchemaValidationError::InvalidType {
            field: path.to_string(),
            expected: "object".to_string(),
            actual: Self::type_name(value),
        })
    }

    fn require_field(
        &self,
        obj: &serde_json::Map<String, Value>,
        field: &str,
        parent: &str,
    ) -> Result<(), SchemaValidationError> {
        if !obj.contains_key(field) {
            Err(SchemaValidationError::MissingRequired {
                field: if parent == "root" {
                    field.to_string()
                } else {
                    format!("{}.{}", parent, field)
                },
            })
        } else {
            Ok(())
        }
    }

    fn require_string_field(
        &self,
        obj: &serde_json::Map<String, Value>,
        field: &str,
        parent: &str,
    ) -> Result<(), SchemaValidationError> {
        self.require_field(obj, field, parent)?;
        if let Some(val) = obj.get(field) {
            if !val.is_string() {
                return Err(SchemaValidationError::InvalidType {
                    field: format!("{}.{}", parent, field),
                    expected: "string".to_string(),
                    actual: Self::type_name(val),
                });
            }
        }
        Ok(())
    }

    fn require_non_empty_string(
        &self,
        obj: &serde_json::Map<String, Value>,
        field: &str,
        parent: &str,
    ) -> Result<(), SchemaValidationError> {
        self.require_string_field(obj, field, parent)?;
        if let Some(val) = obj.get(field).and_then(|v| v.as_str()) {
            if val.is_empty() {
                return Err(SchemaValidationError::Generic {
                    message: format!("{}.{} must not be empty", parent, field),
                });
            }
        }
        Ok(())
    }

    fn require_string_min_length(
        &self,
        obj: &serde_json::Map<String, Value>,
        field: &str,
        min_length: usize,
        parent: &str,
    ) -> Result<(), SchemaValidationError> {
        self.require_string_field(obj, field, parent)?;
        if let Some(val) = obj.get(field).and_then(|v| v.as_str()) {
            if val.len() < min_length {
                return Err(SchemaValidationError::Generic {
                    message: format!(
                        "{}.{} must be at least {} characters, got {}",
                        parent,
                        field,
                        min_length,
                        val.len()
                    ),
                });
            }
        }
        Ok(())
    }

    fn require_uuid_field(
        &self,
        obj: &serde_json::Map<String, Value>,
        field: &str,
        parent: &str,
    ) -> Result<(), SchemaValidationError> {
        self.require_string_field(obj, field, parent)?;
        if let Some(val) = obj.get(field).and_then(|v| v.as_str()) {
            if Uuid::parse_str(val).is_err() {
                return Err(SchemaValidationError::InvalidFormat {
                    field: format!("{}.{}", parent, field),
                    format: "uuid".to_string(),
                });
            }
        }
        Ok(())
    }

    fn validate_enum(
        &self,
        value: &Value,
        valid_values: &[&str],
        path: &str,
    ) -> Result<(), SchemaValidationError> {
        if let Some(s) = value.as_str() {
            if !valid_values.contains(&s) {
                return Err(SchemaValidationError::Generic {
                    message: format!(
                        "{} must be one of: {:?}, got '{}'",
                        path, valid_values, s
                    ),
                });
            }
        } else {
            return Err(SchemaValidationError::InvalidType {
                field: path.to_string(),
                expected: "string".to_string(),
                actual: Self::type_name(value),
            });
        }
        Ok(())
    }

    fn type_name(value: &Value) -> String {
        match value {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
        .to_string()
    }

    fn collect_errors(errors: Vec<SchemaValidationError>) -> SchemaValidationError {
        if errors.len() == 1 {
            errors.into_iter().next().unwrap()
        } else {
            SchemaValidationError::Multiple(errors)
        }
    }
}

/// Static storage for raw schemas (for `schema_for` method).
static RAW_SCHEMAS: Lazy<HashMap<ComponentType, Value>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for ct in ComponentType::all() {
        map.insert(*ct, JsonSchemaValidator::load_raw_schema(*ct));
    }
    map
});

impl ComponentSchemaValidator for JsonSchemaValidator {
    fn validate(
        &self,
        component_type: ComponentType,
        output: &Value,
    ) -> Result<(), SchemaValidationError> {
        self.validate_component(component_type, output)
    }

    fn schema_for(&self, component_type: ComponentType) -> &Value {
        RAW_SCHEMAS
            .get(&component_type)
            .expect("Schema must exist for all component types")
    }

    fn validate_partial(
        &self,
        component_type: ComponentType,
        output: &Value,
    ) -> Result<(), SchemaValidationError> {
        // For partial validation, accept null or empty objects
        if output.is_null() {
            return Ok(());
        }

        if let Some(obj) = output.as_object() {
            if obj.is_empty() {
                return Ok(());
            }
        }

        // For non-empty values, do full validation
        // The caller can catch MissingRequired errors for partial data
        self.validate(component_type, output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn validator() -> JsonSchemaValidator {
        JsonSchemaValidator::new()
    }

    // =============================================================
    // IssueRaising Tests
    // =============================================================

    #[test]
    fn issue_raising_valid_full() {
        let v = validator();
        let output = json!({
            "potential_decisions": [{
                "id": "550e8400-e29b-41d4-a716-446655440001",
                "description": "Should we relocate?",
                "priority": "high"
            }],
            "objectives": [{
                "id": "550e8400-e29b-41d4-a716-446655440002",
                "description": "Minimize costs"
            }],
            "uncertainties": [{
                "id": "550e8400-e29b-41d4-a716-446655440003",
                "description": "Market conditions",
                "driver": "Economic factors"
            }],
            "considerations": [{
                "id": "550e8400-e29b-41d4-a716-446655440004",
                "text": "Team morale is important"
            }]
        });

        assert!(v.validate(ComponentType::IssueRaising, &output).is_ok());
    }

    #[test]
    fn issue_raising_valid_empty_arrays() {
        let v = validator();
        let output = json!({
            "potential_decisions": [],
            "objectives": [],
            "uncertainties": [],
            "considerations": []
        });

        assert!(v.validate(ComponentType::IssueRaising, &output).is_ok());
    }

    #[test]
    fn issue_raising_missing_required_field() {
        let v = validator();
        let output = json!({
            "potential_decisions": [],
            "objectives": [],
            "uncertainties": []
            // missing: considerations
        });

        let result = v.validate(ComponentType::IssueRaising, &output);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SchemaValidationError::MissingRequired { .. }));
    }

    // =============================================================
    // Objectives Tests
    // =============================================================

    #[test]
    fn objectives_valid_with_measures() {
        let v = validator();
        let output = json!({
            "fundamental_objectives": [{
                "id": "550e8400-e29b-41d4-a716-446655440001",
                "description": "Maximize revenue",
                "performance_measure": {
                    "metric": "Annual Revenue",
                    "direction": "maximize",
                    "target": "$10M",
                    "units": "USD"
                },
                "weight": 0.5
            }],
            "means_objectives": [{
                "id": "550e8400-e29b-41d4-a716-446655440002",
                "description": "Increase market share",
                "supports": ["550e8400-e29b-41d4-a716-446655440001"]
            }]
        });

        assert!(v.validate(ComponentType::Objectives, &output).is_ok());
    }

    #[test]
    fn objectives_requires_at_least_one_fundamental() {
        let v = validator();
        let output = json!({
            "fundamental_objectives": [], // requires minItems: 1
            "means_objectives": []
        });

        let result = v.validate(ComponentType::Objectives, &output);
        assert!(result.is_err());
    }

    // =============================================================
    // Alternatives Tests
    // =============================================================

    #[test]
    fn alternatives_valid_with_status_quo() {
        let v = validator();
        let output = json!({
            "alternatives": [
                {
                    "id": "550e8400-e29b-41d4-a716-446655440001",
                    "name": "Status Quo",
                    "description": "Do nothing",
                    "is_status_quo": true
                },
                {
                    "id": "550e8400-e29b-41d4-a716-446655440002",
                    "name": "Option A",
                    "description": "Make the change"
                }
            ],
            "status_quo_id": "550e8400-e29b-41d4-a716-446655440001"
        });

        assert!(v.validate(ComponentType::Alternatives, &output).is_ok());
    }

    #[test]
    fn alternatives_requires_at_least_two() {
        let v = validator();
        let output = json!({
            "alternatives": [{
                "id": "550e8400-e29b-41d4-a716-446655440001",
                "name": "Only Option",
                "description": "Just one"
            }],
            "status_quo_id": "550e8400-e29b-41d4-a716-446655440001"
        });

        let result = v.validate(ComponentType::Alternatives, &output);
        assert!(result.is_err());
    }

    // =============================================================
    // Consequences Tests
    // =============================================================

    #[test]
    fn consequences_valid_with_ratings() {
        let v = validator();
        let output = json!({
            "table": {
                "alternative_ids": [
                    "550e8400-e29b-41d4-a716-446655440001",
                    "550e8400-e29b-41d4-a716-446655440002"
                ],
                "objective_ids": [
                    "550e8400-e29b-41d4-a716-446655440003"
                ],
                "cells": {
                    "550e8400-e29b-41d4-a716-446655440001:550e8400-e29b-41d4-a716-446655440003": {
                        "alternative_id": "550e8400-e29b-41d4-a716-446655440001",
                        "objective_id": "550e8400-e29b-41d4-a716-446655440003",
                        "rating": 0,
                        "rationale": "Baseline"
                    },
                    "550e8400-e29b-41d4-a716-446655440002:550e8400-e29b-41d4-a716-446655440003": {
                        "alternative_id": "550e8400-e29b-41d4-a716-446655440002",
                        "objective_id": "550e8400-e29b-41d4-a716-446655440003",
                        "rating": 2,
                        "rationale": "Much better"
                    }
                }
            }
        });

        assert!(v.validate(ComponentType::Consequences, &output).is_ok());
    }

    #[test]
    fn consequences_rating_out_of_range() {
        let v = validator();
        let output = json!({
            "table": {
                "alternative_ids": [
                    "550e8400-e29b-41d4-a716-446655440001",
                    "550e8400-e29b-41d4-a716-446655440002"
                ],
                "objective_ids": [
                    "550e8400-e29b-41d4-a716-446655440003"
                ],
                "cells": {
                    "key": {
                        "alternative_id": "550e8400-e29b-41d4-a716-446655440001",
                        "objective_id": "550e8400-e29b-41d4-a716-446655440003",
                        "rating": 5 // Out of range: max is 2
                    }
                }
            }
        });

        let result = v.validate(ComponentType::Consequences, &output);
        assert!(result.is_err());
    }

    // =============================================================
    // DecisionQuality Tests
    // =============================================================

    #[test]
    fn decision_quality_valid_complete() {
        let v = validator();
        let output = json!({
            "elements": [
                { "name": "Helpful Problem Frame", "score": 80, "rationale": "Well defined" },
                { "name": "Clear Objectives", "score": 75 },
                { "name": "Creative Alternatives", "score": 90 },
                { "name": "Reliable Consequence Information", "score": 70 },
                { "name": "Logically Correct Reasoning", "score": 85 },
                { "name": "Clear Tradeoffs", "score": 65 },
                { "name": "Commitment to Follow Through", "score": 95 }
            ],
            "overall_score": 65
        });

        assert!(v.validate(ComponentType::DecisionQuality, &output).is_ok());
    }

    #[test]
    fn decision_quality_requires_exactly_seven_elements() {
        let v = validator();
        let output = json!({
            "elements": [
                { "name": "Helpful Problem Frame", "score": 80 },
                { "name": "Clear Objectives", "score": 75 },
                { "name": "Creative Alternatives", "score": 90 }
                // Missing 4 more elements
            ]
        });

        let result = v.validate(ComponentType::DecisionQuality, &output);
        assert!(result.is_err());
    }

    #[test]
    fn decision_quality_score_must_be_0_to_100() {
        let v = validator();
        let output = json!({
            "elements": [
                { "name": "Helpful Problem Frame", "score": 150 }, // Out of range
                { "name": "Clear Objectives", "score": 75 },
                { "name": "Creative Alternatives", "score": 90 },
                { "name": "Reliable Consequence Information", "score": 70 },
                { "name": "Logically Correct Reasoning", "score": 85 },
                { "name": "Clear Tradeoffs", "score": 65 },
                { "name": "Commitment to Follow Through", "score": 95 }
            ]
        });

        let result = v.validate(ComponentType::DecisionQuality, &output);
        assert!(result.is_err());
    }

    // =============================================================
    // Recommendation Tests
    // =============================================================

    #[test]
    fn recommendation_valid_minimal() {
        let v = validator();
        let output = json!({
            "synthesis": "Based on the analysis, Option B provides the best balance of cost and quality while meeting all fundamental objectives."
        });

        assert!(v.validate(ComponentType::Recommendation, &output).is_ok());
    }

    #[test]
    fn recommendation_synthesis_too_short() {
        let v = validator();
        let output = json!({
            "synthesis": "Pick Option B" // Less than 50 characters
        });

        let result = v.validate(ComponentType::Recommendation, &output);
        assert!(result.is_err());
    }

    // =============================================================
    // NotesNextSteps Tests
    // =============================================================

    #[test]
    fn notes_next_steps_valid_complete() {
        let v = validator();
        let output = json!({
            "notes": ["Key insight about timing"],
            "open_questions": ["What about market changes?"],
            "planned_actions": [{
                "action": "Schedule meeting with team",
                "owner": "Jane",
                "due_date": "2026-02-15",
                "status": "planned"
            }],
            "decision_affirmation": "This was a well-considered decision",
            "revisit_triggers": ["Market conditions change significantly"]
        });

        assert!(v.validate(ComponentType::NotesNextSteps, &output).is_ok());
    }

    #[test]
    fn notes_next_steps_valid_empty() {
        let v = validator();
        // All fields are optional
        let output = json!({});

        assert!(v.validate(ComponentType::NotesNextSteps, &output).is_ok());
    }

    // =============================================================
    // Partial Validation Tests
    // =============================================================

    #[test]
    fn partial_validation_accepts_null() {
        let v = validator();
        assert!(v
            .validate_partial(ComponentType::IssueRaising, &Value::Null)
            .is_ok());
    }

    #[test]
    fn partial_validation_accepts_empty_object() {
        let v = validator();
        let output = json!({});
        let result = v.validate_partial(ComponentType::NotesNextSteps, &output);
        assert!(result.is_ok());
    }

    // =============================================================
    // Schema Access Tests
    // =============================================================

    #[test]
    fn schema_for_returns_valid_json() {
        let v = validator();

        for ct in ComponentType::all() {
            let schema = v.schema_for(*ct);
            assert!(schema.is_object());
            assert!(schema.get("$schema").is_some());
            assert!(schema.get("title").is_some());
        }
    }

    // =============================================================
    // Problem Frame Tests
    // =============================================================

    #[test]
    fn problem_frame_valid_complete() {
        let v = validator();
        let output = json!({
            "decision_maker": {
                "name": "John Doe",
                "role": "CEO"
            },
            "focal_decision": {
                "statement": "Should we expand into the European market within the next fiscal year?",
                "scope": "Geographic expansion decisions for 2026",
                "constraints": ["Budget limited to $5M", "Must use existing product line"],
                "trigger": "Board directive to explore growth opportunities"
            },
            "decision_hierarchy": {
                "already_made": [{
                    "id": "550e8400-e29b-41d4-a716-446655440001",
                    "statement": "We will pursue international growth",
                    "outcome": "Approved by board"
                }],
                "focal": {
                    "id": "550e8400-e29b-41d4-a716-446655440002",
                    "statement": "Which region to expand into first"
                },
                "deferred": [{
                    "id": "550e8400-e29b-41d4-a716-446655440003",
                    "statement": "Which cities within the region"
                }]
            },
            "parties": [{
                "name": "Marketing Team",
                "role": "advisor",
                "influence": "Provides market research"
            }]
        });

        assert!(v.validate(ComponentType::ProblemFrame, &output).is_ok());
    }

    // =============================================================
    // Tradeoffs Tests
    // =============================================================

    #[test]
    fn tradeoffs_valid_with_dominated_alternatives() {
        let v = validator();
        let output = json!({
            "dominated_alternatives": [{
                "alternative_id": "550e8400-e29b-41d4-a716-446655440001",
                "dominated_by": "550e8400-e29b-41d4-a716-446655440002",
                "explanation": "Option A is strictly worse on all dimensions"
            }],
            "irrelevant_objectives": [{
                "objective_id": "550e8400-e29b-41d4-a716-446655440003",
                "reason": "All alternatives score the same"
            }],
            "tensions": [{
                "alternative_id": "550e8400-e29b-41d4-a716-446655440002",
                "gains": ["550e8400-e29b-41d4-a716-446655440004"],
                "losses": ["550e8400-e29b-41d4-a716-446655440005"],
                "net_score": 1
            }]
        });

        assert!(v.validate(ComponentType::Tradeoffs, &output).is_ok());
    }

    #[test]
    fn tradeoffs_valid_empty() {
        let v = validator();
        // All fields are optional
        let output = json!({});

        assert!(v.validate(ComponentType::Tradeoffs, &output).is_ok());
    }
}
