//! Document Generator Port - Markdown generation interface.
//!
//! This port defines the contract for generating markdown documents from
//! PrOACT component outputs. The domain depends on this trait, while
//! adapters (like TemplateDocumentGenerator) provide the implementation.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::domain::cycle::Cycle;
use crate::domain::foundation::ComponentType;

/// Port for generating markdown documents from component outputs.
///
/// # Contract
///
/// Implementations must:
/// - Generate consistent markdown following the decision document template
/// - Handle all 8 PrOACT component types
/// - Support both full document and incremental section generation
/// - Preserve markdown structure for round-trip parsing
///
/// # Usage
///
/// ```rust,ignore
/// let generator: &dyn DocumentGenerator = get_generator();
///
/// // Generate full document from cycle state
/// let markdown = generator.generate("Career Decision", &cycle, options)?;
///
/// // Generate single section for incremental updates
/// let section = generator.generate_section(ComponentType::Objectives, &output)?;
/// ```
pub trait DocumentGenerator: Send + Sync {
    /// Generate full markdown document from cycle state.
    ///
    /// Creates a complete decision document including:
    /// - Header with metadata
    /// - All 8 PrOACT sections (with placeholders for unstarted components)
    /// - Footer with version and quality info
    ///
    /// # Arguments
    ///
    /// * `session_title` - Title to display in the document header
    /// * `cycle` - The cycle containing component outputs
    /// * `options` - Generation options (format, what to include)
    ///
    /// # Errors
    ///
    /// Returns `DocumentError` if generation fails due to invalid component data.
    fn generate(
        &self,
        session_title: &str,
        cycle: &Cycle,
        options: GenerationOptions,
    ) -> Result<String, DocumentError>;

    /// Generate a single section for incremental updates.
    ///
    /// Used when only one component has changed and the document needs
    /// a surgical update rather than full regeneration.
    ///
    /// # Arguments
    ///
    /// * `component_type` - The PrOACT component being generated
    /// * `output` - The component's structured output data
    ///
    /// # Errors
    ///
    /// Returns `DocumentError` if the output doesn't match the expected schema.
    fn generate_section(
        &self,
        component_type: ComponentType,
        output: &Value,
    ) -> Result<String, DocumentError>;

    /// Generate the document header with metadata.
    ///
    /// Creates the YAML frontmatter and title section.
    fn generate_header(
        &self,
        session_title: &str,
        options: &GenerationOptions,
    ) -> Result<String, DocumentError>;

    /// Generate the document footer with version info.
    ///
    /// Creates the footer section with document version, DQ score, and timestamps.
    fn generate_footer(&self, cycle: &Cycle, options: &GenerationOptions)
        -> Result<String, DocumentError>;
}

/// Options for document generation.
#[derive(Debug, Clone, Default)]
pub struct GenerationOptions {
    /// Include metadata section (cycle ID, timestamps).
    pub include_metadata: bool,

    /// Include document version information in footer.
    pub include_version_info: bool,

    /// Include empty section placeholders for unstarted components.
    pub include_empty_sections: bool,

    /// Output format variant.
    pub format: DocumentFormat,
}

impl GenerationOptions {
    /// Default options for full document generation.
    pub fn full() -> Self {
        Self {
            include_metadata: true,
            include_version_info: true,
            include_empty_sections: true,
            format: DocumentFormat::Full,
        }
    }

    /// Options for summary/dashboard view.
    pub fn summary() -> Self {
        Self {
            include_metadata: false,
            include_version_info: false,
            include_empty_sections: false,
            format: DocumentFormat::Summary,
        }
    }

    /// Options for exporting (sharing externally).
    pub fn export() -> Self {
        Self {
            include_metadata: false,
            include_version_info: false,
            include_empty_sections: false,
            format: DocumentFormat::Export,
        }
    }
}

/// Document format variants.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentFormat {
    /// Complete document with all sections and metadata.
    #[default]
    Full,

    /// Key sections only (for dashboard views).
    Summary,

    /// Clean format for sharing (no internal IDs or system info).
    Export,
}

/// Errors that can occur during document generation.
#[derive(Debug, Clone, Error)]
pub enum DocumentError {
    /// Component output doesn't match expected schema.
    #[error("Invalid component output for {component_type}: {reason}")]
    InvalidOutput {
        component_type: ComponentType,
        reason: String,
    },

    /// Template rendering failed.
    #[error("Template rendering failed: {0}")]
    TemplateError(String),

    /// Missing required data for generation.
    #[error("Missing required data: {field}")]
    MissingData { field: String },

    /// Internal generation error.
    #[error("Generation failed: {0}")]
    Internal(String),
}

impl DocumentError {
    /// Creates an invalid output error.
    pub fn invalid_output(component_type: ComponentType, reason: impl Into<String>) -> Self {
        Self::InvalidOutput {
            component_type,
            reason: reason.into(),
        }
    }

    /// Creates a template error.
    pub fn template(message: impl Into<String>) -> Self {
        Self::TemplateError(message.into())
    }

    /// Creates a missing data error.
    pub fn missing_data(field: impl Into<String>) -> Self {
        Self::MissingData {
            field: field.into(),
        }
    }

    /// Creates an internal error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ───────────────────────────────────────────────────────────────
    // GenerationOptions tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn default_options_has_full_format() {
        let opts = GenerationOptions::default();
        assert_eq!(opts.format, DocumentFormat::Full);
        assert!(!opts.include_metadata);
        assert!(!opts.include_version_info);
        assert!(!opts.include_empty_sections);
    }

    #[test]
    fn full_options_includes_everything() {
        let opts = GenerationOptions::full();
        assert!(opts.include_metadata);
        assert!(opts.include_version_info);
        assert!(opts.include_empty_sections);
        assert_eq!(opts.format, DocumentFormat::Full);
    }

    #[test]
    fn summary_options_excludes_metadata() {
        let opts = GenerationOptions::summary();
        assert!(!opts.include_metadata);
        assert!(!opts.include_version_info);
        assert!(!opts.include_empty_sections);
        assert_eq!(opts.format, DocumentFormat::Summary);
    }

    #[test]
    fn export_options_excludes_internal_info() {
        let opts = GenerationOptions::export();
        assert!(!opts.include_metadata);
        assert!(!opts.include_version_info);
        assert_eq!(opts.format, DocumentFormat::Export);
    }

    // ───────────────────────────────────────────────────────────────
    // DocumentFormat tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn document_format_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&DocumentFormat::Full).unwrap(),
            "\"full\""
        );
        assert_eq!(
            serde_json::to_string(&DocumentFormat::Summary).unwrap(),
            "\"summary\""
        );
        assert_eq!(
            serde_json::to_string(&DocumentFormat::Export).unwrap(),
            "\"export\""
        );
    }

    #[test]
    fn document_format_deserializes_from_snake_case() {
        let full: DocumentFormat = serde_json::from_str("\"full\"").unwrap();
        assert_eq!(full, DocumentFormat::Full);

        let summary: DocumentFormat = serde_json::from_str("\"summary\"").unwrap();
        assert_eq!(summary, DocumentFormat::Summary);
    }

    // ───────────────────────────────────────────────────────────────
    // DocumentError tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn invalid_output_error_displays_component_and_reason() {
        let err = DocumentError::invalid_output(ComponentType::Objectives, "missing measures");
        assert!(err.to_string().contains("Objectives"));
        assert!(err.to_string().contains("missing measures"));
    }

    #[test]
    fn template_error_displays_message() {
        let err = DocumentError::template("failed to render section");
        assert!(err.to_string().contains("failed to render section"));
    }

    #[test]
    fn missing_data_error_displays_field() {
        let err = DocumentError::missing_data("session_title");
        assert!(err.to_string().contains("session_title"));
    }

    #[test]
    fn internal_error_displays_message() {
        let err = DocumentError::internal("unexpected state");
        assert!(err.to_string().contains("unexpected state"));
    }

    // ───────────────────────────────────────────────────────────────
    // Trait object safety test
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn document_generator_is_object_safe() {
        fn check<T: DocumentGenerator + ?Sized>() {}
        // This compiles only if the trait is object-safe
        check::<dyn DocumentGenerator>();
    }
}
