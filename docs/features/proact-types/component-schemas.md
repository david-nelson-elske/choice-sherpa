# PrOACT Component Output Schemas

**Module:** proact-types
**Type:** Cross-Module Contract
**Priority:** P0 (Required for conversation-cycle integration)
**Last Updated:** 2026-01-08

> JSON Schema definitions for each PrOACT component's structured output. These schemas enforce the contract between conversation extraction and cycle storage.

---

## Overview

Each component type has a defined output schema. The conversation module extracts data matching these schemas, and the cycle module validates before storage.

**Key Properties:**
- All schemas use JSON Schema draft-07
- UUIDs use the "uuid" format for validation
- Required fields enforce completeness before component completion
- Optional fields support incremental extraction during conversation

---

## Schema Validation Port

```rust
use serde_json::Value;
use crate::foundation::ComponentType;

/// Port for validating component outputs against their schemas
pub trait ComponentSchemaValidator: Send + Sync {
    /// Validate output against component type's schema
    /// Returns Ok(()) if valid, Err with validation errors if not
    fn validate(
        &self,
        component_type: ComponentType,
        output: &Value,
    ) -> Result<(), SchemaValidationError>;

    /// Get the JSON Schema for a component type
    fn schema_for(&self, component_type: ComponentType) -> &serde_json::Value;

    /// Validate partial output (less strict, allows missing optional fields)
    fn validate_partial(
        &self,
        component_type: ComponentType,
        output: &Value,
    ) -> Result<(), SchemaValidationError>;
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum SchemaValidationError {
    #[error("Missing required field: {field}")]
    MissingRequired { field: String },

    #[error("Invalid type for field {field}: expected {expected}, got {actual}")]
    InvalidType { field: String, expected: String, actual: String },

    #[error("Array too short for field {field}: minimum {min}, got {actual}")]
    ArrayTooShort { field: String, min: usize, actual: usize },

    #[error("Value out of range for field {field}: {value} not in [{min}, {max}]")]
    OutOfRange { field: String, value: String, min: String, max: String },

    #[error("Invalid format for field {field}: expected {format}")]
    InvalidFormat { field: String, format: String },

    #[error("Validation errors: {0:?}")]
    Multiple(Vec<SchemaValidationError>),
}
```

---

## IssueRaising Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "IssueRaisingOutput",
  "description": "Categorized initial thoughts from user's decision context",
  "type": "object",
  "required": ["potential_decisions", "objectives", "uncertainties", "considerations"],
  "properties": {
    "potential_decisions": {
      "type": "array",
      "description": "Decision questions identified from initial discussion",
      "items": {
        "type": "object",
        "required": ["id", "description"],
        "properties": {
          "id": { "type": "string", "format": "uuid" },
          "description": { "type": "string", "minLength": 1 },
          "priority": {
            "type": "string",
            "enum": ["high", "medium", "low"],
            "default": "medium"
          },
          "notes": { "type": "string" }
        }
      }
    },
    "objectives": {
      "type": "array",
      "description": "Goals and values mentioned by the user",
      "items": {
        "type": "object",
        "required": ["id", "description"],
        "properties": {
          "id": { "type": "string", "format": "uuid" },
          "description": { "type": "string", "minLength": 1 }
        }
      }
    },
    "uncertainties": {
      "type": "array",
      "description": "Unknown factors and risks identified",
      "items": {
        "type": "object",
        "required": ["id", "description"],
        "properties": {
          "id": { "type": "string", "format": "uuid" },
          "description": { "type": "string", "minLength": 1 },
          "driver": {
            "type": "string",
            "description": "What's causing this uncertainty"
          }
        }
      }
    },
    "considerations": {
      "type": "array",
      "description": "General thoughts and context not fitting other categories",
      "items": {
        "type": "object",
        "required": ["id", "text"],
        "properties": {
          "id": { "type": "string", "format": "uuid" },
          "text": { "type": "string", "minLength": 1 }
        }
      }
    }
  }
}
```

### Rust Type

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueRaisingOutput {
    pub potential_decisions: Vec<PotentialDecision>,
    pub objectives: Vec<IdentifiedObjective>,
    pub uncertainties: Vec<Uncertainty>,
    pub considerations: Vec<Consideration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PotentialDecision {
    pub id: Uuid,
    pub description: String,
    #[serde(default)]
    pub priority: Priority,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    High,
    #[default]
    Medium,
    Low,
}
```

---

## ProblemFrame Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ProblemFrameOutput",
  "description": "Structured framing of the decision context",
  "type": "object",
  "required": ["decision_maker", "focal_decision", "decision_hierarchy"],
  "properties": {
    "decision_maker": {
      "type": "object",
      "description": "Primary person making this decision",
      "required": ["name", "role"],
      "properties": {
        "name": { "type": "string", "minLength": 1 },
        "role": { "type": "string" }
      }
    },
    "focal_decision": {
      "type": "object",
      "description": "The specific decision being addressed",
      "required": ["statement", "scope"],
      "properties": {
        "statement": {
          "type": "string",
          "minLength": 10,
          "description": "Clear statement of the decision question"
        },
        "scope": {
          "type": "string",
          "description": "Boundaries of what's in/out of this decision"
        },
        "constraints": {
          "type": "array",
          "items": { "type": "string" },
          "description": "Limitations or fixed parameters"
        },
        "trigger": {
          "type": "string",
          "description": "What prompted this decision now"
        }
      }
    },
    "decision_hierarchy": {
      "type": "object",
      "description": "Linked decisions in the hierarchy",
      "required": ["already_made", "focal", "deferred"],
      "properties": {
        "already_made": {
          "type": "array",
          "items": { "$ref": "#/definitions/LinkedDecision" },
          "description": "Decisions that constrain this one"
        },
        "focal": {
          "$ref": "#/definitions/LinkedDecision",
          "description": "The current decision"
        },
        "deferred": {
          "type": "array",
          "items": { "$ref": "#/definitions/LinkedDecision" },
          "description": "Decisions that depend on this one"
        }
      }
    },
    "parties": {
      "type": "array",
      "description": "People involved in or affected by this decision",
      "items": {
        "type": "object",
        "required": ["name", "role"],
        "properties": {
          "name": { "type": "string" },
          "role": {
            "type": "string",
            "enum": ["stakeholder", "advisor", "decision_maker", "affected_party"]
          },
          "influence": { "type": "string" }
        }
      }
    }
  },
  "definitions": {
    "LinkedDecision": {
      "type": "object",
      "required": ["id", "statement"],
      "properties": {
        "id": { "type": "string", "format": "uuid" },
        "statement": { "type": "string", "minLength": 1 },
        "outcome": {
          "type": "string",
          "description": "Result of decision (for already_made only)"
        }
      }
    }
  }
}
```

---

## Objectives Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ObjectivesOutput",
  "description": "Fundamental and means objectives with performance measures",
  "type": "object",
  "required": ["fundamental_objectives", "means_objectives"],
  "properties": {
    "fundamental_objectives": {
      "type": "array",
      "minItems": 1,
      "description": "Core goals that matter for their own sake",
      "items": {
        "type": "object",
        "required": ["id", "description"],
        "properties": {
          "id": { "type": "string", "format": "uuid" },
          "description": { "type": "string", "minLength": 1 },
          "performance_measure": {
            "type": "object",
            "description": "How to measure achievement of this objective",
            "properties": {
              "metric": { "type": "string" },
              "direction": {
                "type": "string",
                "enum": ["maximize", "minimize", "target"]
              },
              "target": { "type": "string" },
              "units": { "type": "string" }
            }
          },
          "weight": {
            "type": "number",
            "minimum": 0,
            "maximum": 1,
            "description": "Relative importance (optional, must sum to 1)"
          }
        }
      }
    },
    "means_objectives": {
      "type": "array",
      "description": "Objectives that serve as means to fundamental objectives",
      "items": {
        "type": "object",
        "required": ["id", "description", "supports"],
        "properties": {
          "id": { "type": "string", "format": "uuid" },
          "description": { "type": "string", "minLength": 1 },
          "supports": {
            "type": "array",
            "items": { "type": "string", "format": "uuid" },
            "minItems": 1,
            "description": "IDs of fundamental objectives this supports"
          }
        }
      }
    }
  }
}
```

---

## Alternatives Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AlternativesOutput",
  "description": "Options being considered with status quo baseline",
  "type": "object",
  "required": ["alternatives", "status_quo_id"],
  "properties": {
    "alternatives": {
      "type": "array",
      "minItems": 2,
      "description": "All options including status quo",
      "items": {
        "type": "object",
        "required": ["id", "name", "description"],
        "properties": {
          "id": { "type": "string", "format": "uuid" },
          "name": {
            "type": "string",
            "minLength": 1,
            "maxLength": 100,
            "description": "Short name for display"
          },
          "description": {
            "type": "string",
            "description": "Detailed description of this option"
          },
          "is_status_quo": {
            "type": "boolean",
            "default": false,
            "description": "True if this is the do-nothing baseline"
          }
        }
      }
    },
    "status_quo_id": {
      "type": "string",
      "format": "uuid",
      "description": "ID of the alternative designated as status quo (baseline for Pugh)"
    },
    "strategy_table": {
      "type": "object",
      "description": "Optional strategy table for complex decisions",
      "properties": {
        "decision_columns": {
          "type": "array",
          "description": "Sub-decisions that make up each alternative",
          "items": {
            "type": "object",
            "required": ["id", "name", "options"],
            "properties": {
              "id": { "type": "string" },
              "name": { "type": "string" },
              "options": {
                "type": "array",
                "items": { "type": "string" },
                "minItems": 1
              }
            }
          }
        },
        "strategies": {
          "type": "array",
          "description": "Mapping of alternatives to strategy selections",
          "items": {
            "type": "object",
            "required": ["alternative_id", "selections"],
            "properties": {
              "alternative_id": { "type": "string", "format": "uuid" },
              "selections": {
                "type": "object",
                "description": "Map of column_id -> selected option",
                "additionalProperties": { "type": "string" }
              }
            }
          }
        }
      }
    }
  }
}
```

---

## Consequences Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ConsequencesOutput",
  "description": "Consequence table with Pugh ratings relative to status quo",
  "type": "object",
  "required": ["table"],
  "properties": {
    "table": {
      "type": "object",
      "required": ["alternative_ids", "objective_ids", "cells"],
      "properties": {
        "alternative_ids": {
          "type": "array",
          "items": { "type": "string", "format": "uuid" },
          "minItems": 2,
          "description": "All alternatives being compared"
        },
        "objective_ids": {
          "type": "array",
          "items": { "type": "string", "format": "uuid" },
          "minItems": 1,
          "description": "All fundamental objectives being evaluated"
        },
        "cells": {
          "type": "object",
          "description": "Map of 'alt_id:obj_id' -> Cell",
          "additionalProperties": {
            "type": "object",
            "required": ["alternative_id", "objective_id", "rating"],
            "properties": {
              "alternative_id": { "type": "string", "format": "uuid" },
              "objective_id": { "type": "string", "format": "uuid" },
              "rating": {
                "type": "integer",
                "minimum": -2,
                "maximum": 2,
                "description": "Pugh rating: -2 (much worse) to +2 (much better) vs status quo"
              },
              "rationale": {
                "type": "string",
                "description": "Explanation for this rating"
              },
              "uncertainty": {
                "type": "object",
                "properties": {
                  "level": { "type": "string", "enum": ["low", "medium", "high"] },
                  "driver": { "type": "string" }
                }
              }
            }
          }
        }
      }
    }
  }
}
```

### Pugh Rating Scale

| Rating | Meaning | Description |
|--------|---------|-------------|
| -2 | Much Worse | Significantly underperforms status quo |
| -1 | Worse | Somewhat underperforms status quo |
| 0 | Same | Equal to status quo |
| +1 | Better | Somewhat outperforms status quo |
| +2 | Much Better | Significantly outperforms status quo |

---

## Tradeoffs Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "TradeoffsOutput",
  "description": "Analysis results identifying dominated alternatives and tensions",
  "type": "object",
  "properties": {
    "dominated_alternatives": {
      "type": "array",
      "description": "Alternatives that are strictly worse than another",
      "items": {
        "type": "object",
        "required": ["alternative_id", "dominated_by"],
        "properties": {
          "alternative_id": { "type": "string", "format": "uuid" },
          "dominated_by": { "type": "string", "format": "uuid" },
          "explanation": { "type": "string" }
        }
      }
    },
    "irrelevant_objectives": {
      "type": "array",
      "description": "Objectives where all alternatives score the same",
      "items": {
        "type": "object",
        "required": ["objective_id", "reason"],
        "properties": {
          "objective_id": { "type": "string", "format": "uuid" },
          "reason": { "type": "string" }
        }
      }
    },
    "tensions": {
      "type": "array",
      "description": "Key tradeoffs between alternatives",
      "items": {
        "type": "object",
        "required": ["alternative_id", "gains", "losses"],
        "properties": {
          "alternative_id": { "type": "string", "format": "uuid" },
          "gains": {
            "type": "array",
            "items": { "type": "string", "format": "uuid" },
            "description": "Objective IDs where this alternative excels"
          },
          "losses": {
            "type": "array",
            "items": { "type": "string", "format": "uuid" },
            "description": "Objective IDs where this alternative is weakest"
          },
          "net_score": {
            "type": "integer",
            "description": "Sum of all Pugh ratings for this alternative"
          }
        }
      }
    }
  }
}
```

---

## Recommendation Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "RecommendationOutput",
  "description": "Synthesis of analysis - does NOT decide for user",
  "type": "object",
  "required": ["synthesis"],
  "properties": {
    "synthesis": {
      "type": "string",
      "minLength": 50,
      "description": "Summary of the analysis and potential paths forward"
    },
    "standout_option": {
      "type": "object",
      "description": "Optional: If one alternative clearly stands out",
      "properties": {
        "alternative_id": { "type": "string", "format": "uuid" },
        "rationale": { "type": "string" }
      }
    },
    "key_considerations": {
      "type": "array",
      "description": "Most important factors for the decision",
      "items": { "type": "string" }
    },
    "remaining_uncertainties": {
      "type": "array",
      "description": "Unknowns that could affect the decision",
      "items": {
        "type": "object",
        "properties": {
          "description": { "type": "string" },
          "impact": { "type": "string", "enum": ["high", "medium", "low"] },
          "resolution_path": { "type": "string" }
        }
      }
    },
    "sensitivity_notes": {
      "type": "string",
      "description": "How robust is the analysis to changes in assumptions"
    }
  }
}
```

---

## DecisionQuality Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DecisionQualityOutput",
  "description": "Assessment of decision quality across 7 elements",
  "type": "object",
  "required": ["elements"],
  "properties": {
    "elements": {
      "type": "array",
      "minItems": 7,
      "maxItems": 7,
      "description": "Scores for each DQ element",
      "items": {
        "type": "object",
        "required": ["name", "score"],
        "properties": {
          "name": {
            "type": "string",
            "enum": [
              "Helpful Problem Frame",
              "Clear Objectives",
              "Creative Alternatives",
              "Reliable Consequence Information",
              "Logically Correct Reasoning",
              "Clear Tradeoffs",
              "Commitment to Follow Through"
            ]
          },
          "score": {
            "type": "integer",
            "minimum": 0,
            "maximum": 100,
            "description": "0-100 percentage score"
          },
          "rationale": {
            "type": "string",
            "description": "Why this score was given"
          },
          "improvement_path": {
            "type": "string",
            "description": "What would increase this score"
          }
        }
      }
    },
    "overall_score": {
      "type": "integer",
      "minimum": 0,
      "maximum": 100,
      "description": "Computed as minimum of all element scores (weakest link)"
    }
  }
}
```

### DQ Element Descriptions

| Element | Description |
|---------|-------------|
| Helpful Problem Frame | Is the decision clearly defined with appropriate scope? |
| Clear Objectives | Are objectives well-articulated with performance measures? |
| Creative Alternatives | Are there diverse, creative options including status quo? |
| Reliable Consequence Information | Is the consequence analysis based on sound data? |
| Logically Correct Reasoning | Is the analysis free from bias and logical errors? |
| Clear Tradeoffs | Are the tradeoffs between alternatives understood? |
| Commitment to Follow Through | Is there willingness to act on the analysis? |

---

## NotesNextSteps Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "NotesNextStepsOutput",
  "description": "Wrap-up notes, open questions, and action items",
  "type": "object",
  "properties": {
    "notes": {
      "type": "array",
      "description": "General notes from the session",
      "items": { "type": "string" }
    },
    "open_questions": {
      "type": "array",
      "description": "Unresolved questions for future consideration",
      "items": { "type": "string" }
    },
    "planned_actions": {
      "type": "array",
      "description": "Concrete next steps",
      "items": {
        "type": "object",
        "required": ["action"],
        "properties": {
          "action": { "type": "string", "minLength": 1 },
          "owner": { "type": "string" },
          "due_date": { "type": "string", "format": "date" },
          "status": {
            "type": "string",
            "enum": ["planned", "in_progress", "completed"],
            "default": "planned"
          }
        }
      }
    },
    "decision_affirmation": {
      "type": "string",
      "description": "When DQ is 100%: affirmation that this was a good decision at time made"
    },
    "revisit_triggers": {
      "type": "array",
      "description": "Conditions that should trigger revisiting this decision",
      "items": { "type": "string" }
    }
  }
}
```

---

## Validation Integration

### In Cycle Module

```rust
impl Cycle {
    /// Updates component output with schema validation
    pub fn update_component_output(
        &mut self,
        ct: ComponentType,
        output: serde_json::Value,
        validator: &dyn ComponentSchemaValidator,
    ) -> Result<(), DomainError> {
        self.ensure_mutable()?;

        // Validate BEFORE accepting
        validator.validate(ct, &output)
            .map_err(|e| DomainError::validation("component_output", e.to_string()))?;

        let component = self.components.get_mut(&ct)
            .ok_or_else(|| DomainError::not_found("component"))?;

        component.set_output(output)?;
        self.updated_at = Timestamp::now();

        self.record_event(CycleEvent::ComponentOutputUpdated {
            cycle_id: self.id,
            component_type: ct,
        });

        Ok(())
    }
}
```

### In Conversation Module

```rust
impl DataExtractor {
    /// Extract and validate structured data from conversation
    pub async fn extract(
        &self,
        component_type: ComponentType,
        messages: &[Message],
        validator: &dyn ComponentSchemaValidator,
    ) -> Result<serde_json::Value, ExtractionError> {
        // Extract structured data from conversation via AI
        let extracted = self.ai_extractor.extract(component_type, messages).await?;

        // Validate before returning
        validator.validate(component_type, &extracted)
            .map_err(|e| ExtractionError::InvalidOutput(e.to_string()))?;

        Ok(extracted)
    }

    /// Extract partial data (during conversation, before completion)
    pub async fn extract_partial(
        &self,
        component_type: ComponentType,
        messages: &[Message],
        validator: &dyn ComponentSchemaValidator,
    ) -> Result<serde_json::Value, ExtractionError> {
        let extracted = self.ai_extractor.extract(component_type, messages).await?;

        // Less strict validation for partial data
        validator.validate_partial(component_type, &extracted)
            .map_err(|e| ExtractionError::InvalidOutput(e.to_string()))?;

        Ok(extracted)
    }
}
```

---

## Adapter Implementation

```rust
use jsonschema::{JSONSchema, ValidationError};
use once_cell::sync::Lazy;
use serde_json::Value;

/// JSON Schema-based validator implementation
pub struct JsonSchemaValidator {
    schemas: HashMap<ComponentType, JSONSchema>,
}

impl JsonSchemaValidator {
    pub fn new() -> Self {
        let mut schemas = HashMap::new();

        // Load and compile all schemas at startup
        for ct in ComponentType::all() {
            let schema_json = Self::load_schema(*ct);
            let compiled = JSONSchema::compile(&schema_json)
                .expect("Invalid schema");
            schemas.insert(*ct, compiled);
        }

        Self { schemas }
    }

    fn load_schema(ct: ComponentType) -> Value {
        match ct {
            ComponentType::IssueRaising =>
                serde_json::from_str(include_str!("schemas/issue_raising.json")).unwrap(),
            ComponentType::ProblemFrame =>
                serde_json::from_str(include_str!("schemas/problem_frame.json")).unwrap(),
            // ... etc for all 9 components
        }
    }
}

impl ComponentSchemaValidator for JsonSchemaValidator {
    fn validate(
        &self,
        component_type: ComponentType,
        output: &Value,
    ) -> Result<(), SchemaValidationError> {
        let schema = self.schemas.get(&component_type)
            .ok_or_else(|| SchemaValidationError::MissingRequired {
                field: "schema".to_string()
            })?;

        match schema.validate(output) {
            Ok(_) => Ok(()),
            Err(errors) => {
                let validation_errors: Vec<_> = errors
                    .map(|e| self.convert_error(e))
                    .collect();

                if validation_errors.len() == 1 {
                    Err(validation_errors.into_iter().next().unwrap())
                } else {
                    Err(SchemaValidationError::Multiple(validation_errors))
                }
            }
        }
    }

    fn schema_for(&self, component_type: ComponentType) -> &Value {
        // Return the raw schema JSON for introspection
        static SCHEMAS: Lazy<HashMap<ComponentType, Value>> = Lazy::new(|| {
            let mut map = HashMap::new();
            for ct in ComponentType::all() {
                map.insert(*ct, JsonSchemaValidator::load_schema(*ct));
            }
            map
        });

        SCHEMAS.get(&component_type).unwrap()
    }

    fn validate_partial(
        &self,
        component_type: ComponentType,
        output: &Value,
    ) -> Result<(), SchemaValidationError> {
        // For partial validation, we only check that present fields are valid
        // We don't require all required fields to be present

        // If empty, that's fine for partial
        if output.is_null() || (output.is_object() && output.as_object().unwrap().is_empty()) {
            return Ok(());
        }

        // Validate structure of present fields
        // This is a simplified version - real implementation would strip required constraints
        self.validate(component_type, output)
    }
}
```

---

## Tasks

- [x] Create JSON schema files for all 9 components in `backend/src/domain/proact/schemas/`
- [x] Implement ComponentSchemaValidator port in `backend/src/ports/schema_validator.rs`
- [x] Implement JsonSchemaValidator adapter in `backend/src/adapters/validation/`
- [DEFERRED] Add schema validation to Cycle.update_component_output() - requires Cycle aggregate
- [DEFERRED] Add schema validation to DataExtractor.extract() - requires Conversation module
- [x] Write unit tests for each schema with valid and invalid examples
- [DEFERRED] Write integration tests for conversation extraction → cycle storage flow - requires cycle and conversation modules

---

## Related Documents

- **Cycle Module:** `docs/modules/cycle.md`
- **Conversation Module:** `docs/modules/conversation.md`
- **PrOACT Types:** `features/proact-types/proact-types.md`

---

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Not Required (validation logic, no endpoints) |
| Authorization Model | N/A - schemas used by authenticated modules |
| Sensitive Data | Validated outputs contain Confidential user data |
| Rate Limiting | Not Required (no endpoints) |
| Audit Logging | Log validation failures (without payload details) |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| Schema definitions | Public | Safe to expose, defines contract |
| Validated output data | Confidential | User decision data - never log contents |
| Validation error details | Internal | Sanitize before returning to client |

### Error Handling Security

1. **Client-Safe Error Conversion**: Validation errors returned to clients MUST NOT expose internal schema structure or paths that could reveal implementation details:

```rust
impl SchemaValidationError {
    /// Convert to client-safe error message
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
            SchemaValidationError::Multiple(errors) => {
                // Return first error only to avoid information leakage
                errors.first()
                    .map(|e| e.to_client_message())
                    .unwrap_or_else(|| "Validation failed".to_string())
            }
        }
    }
}
```

2. **Internal vs External Errors**: Log detailed errors internally, return sanitized errors to clients:

```rust
fn validate_and_respond(
    component_type: ComponentType,
    output: &Value,
    validator: &dyn ComponentSchemaValidator,
) -> Result<(), ApiError> {
    validator.validate(component_type, output).map_err(|e| {
        // Log full error internally
        tracing::warn!(
            component = ?component_type,
            error = %e,
            "Schema validation failed"
        );

        // Return sanitized error to client
        ApiError::validation(e.to_client_message())
    })
}
```

### Security Guidelines

1. **Schema Exposure**: JSON schemas are considered Public and can be exposed via API for client-side validation. Do not embed sensitive defaults or comments in schemas.

2. **Payload Logging**: Never log the output being validated - it contains confidential user data:

```rust
// CORRECT: Log validation result without payload
tracing::debug!(
    component = ?component_type,
    valid = true,
    "Schema validation passed"
);

// INCORRECT: Never log the output
tracing::debug!("Validating output: {:?}", output); // DO NOT DO THIS
```

---

*Version: 1.0.0*
*Created: 2026-01-08*
