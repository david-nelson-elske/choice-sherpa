//! Document Parser Port - Markdown parsing interface.
//!
//! This port defines the contract for parsing markdown documents back into
//! structured PrOACT component data. The domain depends on this trait, while
//! adapters (like RegexDocumentParser) provide the implementation.

use crate::domain::cycle::{ParseError, ParsedMetadata, ParsedSection};
use crate::domain::foundation::ComponentType;

use super::DocumentError;

/// Port for parsing markdown documents back to structured data.
///
/// # Contract
///
/// Implementations must:
/// - Parse full documents into section-by-section output
/// - Handle all 8 PrOACT section formats
/// - Preserve data integrity during round-trip (generate → parse → generate)
/// - Report parse errors without failing on recoverable issues
///
/// # Usage
///
/// ```rust,ignore
/// let parser: &dyn DocumentParser = get_parser();
///
/// // Parse full document
/// let result = parser.parse(markdown_content)?;
/// for section in result.sections {
///     if section.parse_errors.is_empty() {
///         process_output(section.component_type, section.parsed_data);
///     }
/// }
///
/// // Validate structure only
/// let errors = parser.validate_structure(markdown_content)?;
/// ```
pub trait DocumentParser: Send + Sync {
    /// Parse full document into component outputs.
    ///
    /// Extracts all PrOACT sections from the markdown document,
    /// converting structured tables and lists back to JSON.
    ///
    /// # Arguments
    ///
    /// * `content` - The full markdown document content
    ///
    /// # Returns
    ///
    /// A `ParseResult` containing:
    /// - Parsed sections with their structured data
    /// - Document metadata (title, status, etc.)
    /// - Any errors or warnings encountered
    ///
    /// # Errors
    ///
    /// Returns `DocumentError` only for catastrophic failures.
    /// Section-level parse errors are returned in the result.
    fn parse(&self, content: &str) -> Result<ParseResult, DocumentError>;

    /// Parse a single section for validation.
    ///
    /// Used for targeted parsing when only one section has been edited.
    ///
    /// # Arguments
    ///
    /// * `section_content` - The raw markdown for just this section
    /// * `expected_type` - The PrOACT component type expected
    ///
    /// # Errors
    ///
    /// Returns `DocumentError` if the section cannot be parsed at all.
    fn parse_section(
        &self,
        section_content: &str,
        expected_type: ComponentType,
    ) -> Result<ParsedSection, DocumentError>;

    /// Validate document structure without extracting data.
    ///
    /// Fast check for document validity without full parsing.
    /// Returns a list of structural issues found.
    ///
    /// # Arguments
    ///
    /// * `content` - The full markdown document content
    ///
    /// # Returns
    ///
    /// A list of `ParseError` describing any structural issues.
    /// Empty list means the structure is valid.
    fn validate_structure(&self, content: &str) -> Result<Vec<ParseError>, DocumentError>;

    /// Extract section boundaries from document.
    ///
    /// Returns the line ranges for each detected section.
    /// Useful for targeted updates and diff operations.
    fn extract_section_boundaries(&self, content: &str) -> Vec<SectionBoundary>;
}

/// Result of parsing a full document.
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// Parsed sections with their structured data.
    pub sections: Vec<ParsedSection>,

    /// Document-level metadata extracted from header/footer.
    pub metadata: ParsedMetadata,

    /// Critical errors that prevented section parsing.
    pub errors: Vec<ParseError>,

    /// Non-critical issues (data may be incomplete).
    pub warnings: Vec<ParseError>,
}

impl ParseResult {
    /// Creates an empty parse result.
    pub fn empty() -> Self {
        Self {
            sections: Vec::new(),
            metadata: ParsedMetadata::empty(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Returns true if parsing succeeded with no errors.
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns true if there are any parse errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns true if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Returns the total number of issues (errors + warnings).
    pub fn issue_count(&self) -> usize {
        self.errors.len() + self.warnings.len()
    }

    /// Returns the number of successfully parsed sections.
    pub fn successful_section_count(&self) -> usize {
        self.sections
            .iter()
            .filter(|s| s.is_successful())
            .count()
    }

    /// Gets a parsed section by component type.
    pub fn section(&self, component_type: ComponentType) -> Option<&ParsedSection> {
        self.sections
            .iter()
            .find(|s| s.component_type == component_type)
    }

    /// Collects all parse errors from all sections.
    pub fn all_section_errors(&self) -> Vec<&ParseError> {
        self.sections
            .iter()
            .flat_map(|s| s.parse_errors.iter())
            .collect()
    }
}

/// Boundary information for a section in the document.
#[derive(Debug, Clone)]
pub struct SectionBoundary {
    /// The component type this section represents.
    pub component_type: ComponentType,

    /// Starting line number (1-based).
    pub start_line: usize,

    /// Ending line number (1-based, inclusive).
    pub end_line: usize,

    /// The section heading text.
    pub heading: String,
}

impl SectionBoundary {
    /// Creates a new section boundary.
    pub fn new(
        component_type: ComponentType,
        start_line: usize,
        end_line: usize,
        heading: impl Into<String>,
    ) -> Self {
        Self {
            component_type,
            start_line,
            end_line,
            heading: heading.into(),
        }
    }

    /// Returns the number of lines in this section.
    pub fn line_count(&self) -> usize {
        self.end_line.saturating_sub(self.start_line) + 1
    }

    /// Returns true if the given line is within this section.
    pub fn contains_line(&self, line: usize) -> bool {
        line >= self.start_line && line <= self.end_line
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cycle::ParseErrorSeverity;

    // ───────────────────────────────────────────────────────────────
    // ParseResult tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn empty_result_has_no_issues() {
        let result = ParseResult::empty();
        assert!(result.is_ok());
        assert!(!result.has_errors());
        assert!(!result.has_warnings());
        assert_eq!(result.issue_count(), 0);
    }

    #[test]
    fn result_with_errors_is_not_ok() {
        let mut result = ParseResult::empty();
        result.errors.push(ParseError::new(
            1,
            None,
            "Missing header",
            ParseErrorSeverity::Error,
        ));

        assert!(!result.is_ok());
        assert!(result.has_errors());
        assert_eq!(result.issue_count(), 1);
    }

    #[test]
    fn result_with_warnings_is_still_ok() {
        let mut result = ParseResult::empty();
        result.warnings.push(ParseError::new(
            5,
            None,
            "Incomplete data",
            ParseErrorSeverity::Warning,
        ));

        assert!(result.is_ok()); // Warnings don't affect is_ok
        assert!(result.has_warnings());
        assert_eq!(result.issue_count(), 1);
    }

    #[test]
    fn issue_count_sums_errors_and_warnings() {
        let mut result = ParseResult::empty();
        result.errors.push(ParseError::new(
            1,
            None,
            "Error 1",
            ParseErrorSeverity::Error,
        ));
        result.errors.push(ParseError::new(
            2,
            None,
            "Error 2",
            ParseErrorSeverity::Error,
        ));
        result.warnings.push(ParseError::new(
            3,
            None,
            "Warning 1",
            ParseErrorSeverity::Warning,
        ));

        assert_eq!(result.issue_count(), 3);
    }

    #[test]
    fn successful_section_count_excludes_failed() {
        let mut result = ParseResult::empty();

        // Add a successful section
        result.sections.push(ParsedSection::success(
            ComponentType::IssueRaising,
            "# Issue Raising".to_string(),
            serde_json::json!({"synthesis": "test"}),
        ));

        // Add a failed section
        result.sections.push(ParsedSection::with_errors(
            ComponentType::Objectives,
            "# Objectives".to_string(),
            vec![ParseError::new(
                10,
                None,
                "Parse failed",
                ParseErrorSeverity::Error,
            )],
        ));

        assert_eq!(result.successful_section_count(), 1);
    }

    #[test]
    fn section_returns_matching_component() {
        let mut result = ParseResult::empty();
        result.sections.push(ParsedSection::success(
            ComponentType::Alternatives,
            "# Alternatives".to_string(),
            serde_json::json!({"alternatives": []}),
        ));

        assert!(result.section(ComponentType::Alternatives).is_some());
        assert!(result.section(ComponentType::Objectives).is_none());
    }

    // ───────────────────────────────────────────────────────────────
    // SectionBoundary tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn section_boundary_line_count() {
        let boundary = SectionBoundary::new(ComponentType::IssueRaising, 10, 25, "Issue Raising");
        assert_eq!(boundary.line_count(), 16); // 25 - 10 + 1
    }

    #[test]
    fn section_boundary_contains_line() {
        let boundary = SectionBoundary::new(ComponentType::Objectives, 50, 100, "Objectives");

        assert!(boundary.contains_line(50)); // Start
        assert!(boundary.contains_line(75)); // Middle
        assert!(boundary.contains_line(100)); // End
        assert!(!boundary.contains_line(49)); // Before
        assert!(!boundary.contains_line(101)); // After
    }

    #[test]
    fn section_boundary_single_line() {
        let boundary = SectionBoundary::new(ComponentType::Tradeoffs, 42, 42, "Tradeoffs");
        assert_eq!(boundary.line_count(), 1);
        assert!(boundary.contains_line(42));
        assert!(!boundary.contains_line(41));
        assert!(!boundary.contains_line(43));
    }

    // ───────────────────────────────────────────────────────────────
    // Trait object safety test
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn document_parser_is_object_safe() {
        fn check<T: DocumentParser + ?Sized>() {}
        // This compiles only if the trait is object-safe
        check::<dyn DocumentParser>();
    }
}
