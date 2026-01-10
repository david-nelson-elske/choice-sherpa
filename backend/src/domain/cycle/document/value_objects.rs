//! Value objects for decision documents.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::domain::foundation::{ComponentType, UserId};

// ════════════════════════════════════════════════════════════════════════════════
// MarkdownContent - The actual document content with integrity checking
// ════════════════════════════════════════════════════════════════════════════════

/// The actual markdown content with checksum for change detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownContent {
    raw: String,
    checksum: String,
}

impl MarkdownContent {
    /// Creates a new MarkdownContent, computing the checksum.
    pub fn new(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let checksum = Self::compute_checksum(&raw);
        Self { raw, checksum }
    }

    /// Computes SHA-256 checksum of content.
    fn compute_checksum(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Returns the raw markdown content.
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// Returns the content checksum.
    pub fn checksum(&self) -> &str {
        &self.checksum
    }

    /// Returns the content size in bytes.
    pub fn size_bytes(&self) -> usize {
        self.raw.len()
    }

    /// Checks if content has changed compared to another string.
    pub fn has_changed(&self, other: &str) -> bool {
        self.checksum != Self::compute_checksum(other)
    }

    /// Updates the content with new raw markdown.
    pub fn update(&mut self, new_raw: impl Into<String>) {
        let new_raw = new_raw.into();
        self.checksum = Self::compute_checksum(&new_raw);
        self.raw = new_raw;
    }
}

impl Default for MarkdownContent {
    fn default() -> Self {
        Self::new("")
    }
}

impl PartialEq for MarkdownContent {
    fn eq(&self, other: &Self) -> bool {
        self.checksum == other.checksum
    }
}

impl Eq for MarkdownContent {}

// ════════════════════════════════════════════════════════════════════════════════
// DocumentVersion - Monotonically increasing version number
// ════════════════════════════════════════════════════════════════════════════════

/// Version tracking for the document.
///
/// Versions only increase, never decrease. Used for optimistic locking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DocumentVersion(u32);

impl DocumentVersion {
    /// Creates the initial version (1).
    pub fn initial() -> Self {
        Self(1)
    }

    /// Creates a version from a raw value.
    pub fn from_raw(value: u32) -> Self {
        Self(value)
    }

    /// Returns the raw version number.
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    /// Returns the next version.
    pub fn increment(&self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl Default for DocumentVersion {
    fn default() -> Self {
        Self::initial()
    }
}

impl std::fmt::Display for DocumentVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}", self.0)
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// SyncSource - What triggered the last synchronization
// ════════════════════════════════════════════════════════════════════════════════

/// Tracks what caused the last sync operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncSource {
    /// Initial document generation.
    Initial,
    /// Generated from component outputs (JSON → MD).
    ComponentUpdate,
    /// Parsed from user edits (MD → JSON).
    UserEdit,
    /// Synchronized from file system changes.
    FileSync,
}

impl SyncSource {
    /// Returns the string representation for database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            SyncSource::Initial => "initial",
            SyncSource::ComponentUpdate => "component_update",
            SyncSource::UserEdit => "user_edit",
            SyncSource::FileSync => "file_sync",
        }
    }
}

impl Default for SyncSource {
    fn default() -> Self {
        SyncSource::Initial
    }
}

impl std::fmt::Display for SyncSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for SyncSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "initial" => Ok(SyncSource::Initial),
            "component_update" => Ok(SyncSource::ComponentUpdate),
            "user_edit" => Ok(SyncSource::UserEdit),
            "file_sync" => Ok(SyncSource::FileSync),
            _ => Err(format!("Invalid sync source: {}", s)),
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// UpdatedBy - Who or what made the last update
// ════════════════════════════════════════════════════════════════════════════════

/// Who last updated the document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum UpdatedBy {
    /// System-generated update (e.g., initial generation).
    System,
    /// Updated by a specific user.
    User { user_id: UserId },
    /// Updated by an AI agent.
    Agent,
}

impl UpdatedBy {
    /// Returns the type string for database storage.
    pub fn type_str(&self) -> &'static str {
        match self {
            UpdatedBy::System => "system",
            UpdatedBy::User { .. } => "user",
            UpdatedBy::Agent => "agent",
        }
    }

    /// Returns the user ID if this is a user update.
    pub fn user_id(&self) -> Option<&UserId> {
        match self {
            UpdatedBy::User { user_id } => Some(user_id),
            _ => None,
        }
    }
}

impl Default for UpdatedBy {
    fn default() -> Self {
        UpdatedBy::System
    }
}

impl std::fmt::Display for UpdatedBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdatedBy::System => write!(f, "system"),
            UpdatedBy::User { user_id } => write!(f, "user:{}", user_id),
            UpdatedBy::Agent => write!(f, "agent"),
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// ParsedSection - Result of parsing a document section
// ════════════════════════════════════════════════════════════════════════════════

/// Represents a parsed section from the markdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSection {
    /// Which component type this section represents.
    pub component_type: ComponentType,
    /// The raw markdown content of the section.
    pub raw_content: String,
    /// The extracted structured data (if parsing succeeded).
    pub parsed_data: Option<serde_json::Value>,
    /// Any errors encountered during parsing.
    pub parse_errors: Vec<ParseError>,
}

impl ParsedSection {
    /// Creates a new successfully parsed section.
    pub fn success(
        component_type: ComponentType,
        raw_content: impl Into<String>,
        data: serde_json::Value,
    ) -> Self {
        Self {
            component_type,
            raw_content: raw_content.into(),
            parsed_data: Some(data),
            parse_errors: Vec::new(),
        }
    }

    /// Creates a section with parse errors.
    pub fn with_errors(
        component_type: ComponentType,
        raw_content: impl Into<String>,
        errors: Vec<ParseError>,
    ) -> Self {
        Self {
            component_type,
            raw_content: raw_content.into(),
            parsed_data: None,
            parse_errors: errors,
        }
    }

    /// Returns true if parsing was successful.
    pub fn is_success(&self) -> bool {
        self.parsed_data.is_some() && self.parse_errors.is_empty()
    }

    /// Returns true if there are any errors.
    pub fn has_errors(&self) -> bool {
        !self.parse_errors.is_empty()
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// ParseError - Errors encountered during markdown parsing
// ════════════════════════════════════════════════════════════════════════════════

/// Errors encountered during markdown parsing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseError {
    /// The line number where the error occurred.
    pub line: usize,
    /// The column number (if available).
    pub column: Option<usize>,
    /// A description of the error.
    pub message: String,
    /// How severe is this error.
    pub severity: ParseErrorSeverity,
}

impl ParseError {
    /// Creates a warning parse error.
    pub fn warning(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            column: None,
            message: message.into(),
            severity: ParseErrorSeverity::Warning,
        }
    }

    /// Creates an error parse error.
    pub fn error(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            column: None,
            message: message.into(),
            severity: ParseErrorSeverity::Error,
        }
    }

    /// Adds column information.
    pub fn at_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.column {
            Some(col) => write!(
                f,
                "[{}] Line {}, Column {}: {}",
                self.severity, self.line, col, self.message
            ),
            None => write!(f, "[{}] Line {}: {}", self.severity, self.line, self.message),
        }
    }
}

/// Severity level of a parse error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParseErrorSeverity {
    /// Data extracted but may be incomplete.
    Warning,
    /// Section could not be parsed.
    Error,
}

impl std::fmt::Display for ParseErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseErrorSeverity::Warning => write!(f, "WARNING"),
            ParseErrorSeverity::Error => write!(f, "ERROR"),
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// ParsedMetadata - Extracted document metadata
// ════════════════════════════════════════════════════════════════════════════════

/// Metadata extracted from the document header.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParsedMetadata {
    /// The document title (session title).
    pub title: Option<String>,
    /// The focal decision statement.
    pub focal_decision: Option<String>,
    /// Current status (In Progress, Complete).
    pub status: Option<String>,
    /// Decision quality score (0-100).
    pub dq_score: Option<u8>,
}

impl ParsedMetadata {
    /// Creates empty metadata.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Returns true if any metadata was extracted.
    pub fn has_content(&self) -> bool {
        self.title.is_some()
            || self.focal_decision.is_some()
            || self.status.is_some()
            || self.dq_score.is_some()
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ───────────────────────────────────────────────────────────────
    // MarkdownContent Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn markdown_content_computes_checksum() {
        let content = MarkdownContent::new("# Hello World");
        assert!(!content.checksum().is_empty());
        assert_eq!(content.checksum().len(), 64); // SHA-256 hex string
    }

    #[test]
    fn markdown_content_detects_changes() {
        let content = MarkdownContent::new("# Original");
        assert!(!content.has_changed("# Original"));
        assert!(content.has_changed("# Modified"));
    }

    #[test]
    fn markdown_content_equality_uses_checksum() {
        let content1 = MarkdownContent::new("# Same");
        let content2 = MarkdownContent::new("# Same");
        let content3 = MarkdownContent::new("# Different");

        assert_eq!(content1, content2);
        assert_ne!(content1, content3);
    }

    #[test]
    fn markdown_content_update_changes_checksum() {
        let mut content = MarkdownContent::new("# Original");
        let original_checksum = content.checksum().to_string();

        content.update("# Updated");

        assert_ne!(content.checksum(), original_checksum);
        assert_eq!(content.raw(), "# Updated");
    }

    // ───────────────────────────────────────────────────────────────
    // DocumentVersion Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn document_version_starts_at_one() {
        let version = DocumentVersion::initial();
        assert_eq!(version.as_u32(), 1);
    }

    #[test]
    fn document_version_increments() {
        let v1 = DocumentVersion::initial();
        let v2 = v1.increment();
        let v3 = v2.increment();

        assert_eq!(v1.as_u32(), 1);
        assert_eq!(v2.as_u32(), 2);
        assert_eq!(v3.as_u32(), 3);
    }

    #[test]
    fn document_version_ordering() {
        let v1 = DocumentVersion::from_raw(1);
        let v2 = DocumentVersion::from_raw(2);

        assert!(v1 < v2);
        assert!(v2 > v1);
    }

    #[test]
    fn document_version_displays_with_prefix() {
        let version = DocumentVersion::from_raw(42);
        assert_eq!(format!("{}", version), "v42");
    }

    // ───────────────────────────────────────────────────────────────
    // SyncSource Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn sync_source_as_str() {
        assert_eq!(SyncSource::Initial.as_str(), "initial");
        assert_eq!(SyncSource::ComponentUpdate.as_str(), "component_update");
        assert_eq!(SyncSource::UserEdit.as_str(), "user_edit");
        assert_eq!(SyncSource::FileSync.as_str(), "file_sync");
    }

    #[test]
    fn sync_source_from_str() {
        assert_eq!("initial".parse::<SyncSource>().unwrap(), SyncSource::Initial);
        assert_eq!(
            "component_update".parse::<SyncSource>().unwrap(),
            SyncSource::ComponentUpdate
        );
        assert_eq!("user_edit".parse::<SyncSource>().unwrap(), SyncSource::UserEdit);
        assert_eq!("file_sync".parse::<SyncSource>().unwrap(), SyncSource::FileSync);
        assert!("invalid".parse::<SyncSource>().is_err());
    }

    // ───────────────────────────────────────────────────────────────
    // UpdatedBy Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn updated_by_type_str() {
        assert_eq!(UpdatedBy::System.type_str(), "system");
        assert_eq!(UpdatedBy::Agent.type_str(), "agent");

        let user_id = UserId::new("test-user").unwrap();
        let updated_by = UpdatedBy::User {
            user_id: user_id.clone(),
        };
        assert_eq!(updated_by.type_str(), "user");
    }

    #[test]
    fn updated_by_user_id_extraction() {
        let user_id = UserId::new("test-user").unwrap();
        let updated_by = UpdatedBy::User {
            user_id: user_id.clone(),
        };

        assert_eq!(updated_by.user_id(), Some(&user_id));
        assert_eq!(UpdatedBy::System.user_id(), None);
        assert_eq!(UpdatedBy::Agent.user_id(), None);
    }

    // ───────────────────────────────────────────────────────────────
    // ParsedSection Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn parsed_section_success() {
        let section = ParsedSection::success(
            ComponentType::IssueRaising,
            "## Issue Raising",
            serde_json::json!({"decisions": []}),
        );

        assert!(section.is_success());
        assert!(!section.has_errors());
    }

    #[test]
    fn parsed_section_with_errors() {
        let section = ParsedSection::with_errors(
            ComponentType::Objectives,
            "## Objectives\n\nInvalid content",
            vec![ParseError::error(3, "Expected table format")],
        );

        assert!(!section.is_success());
        assert!(section.has_errors());
    }

    // ───────────────────────────────────────────────────────────────
    // ParseError Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn parse_error_display_without_column() {
        let error = ParseError::error(10, "Missing required field");
        let display = format!("{}", error);
        assert!(display.contains("Line 10"));
        assert!(display.contains("Missing required field"));
    }

    #[test]
    fn parse_error_display_with_column() {
        let error = ParseError::warning(5, "Unexpected format").at_column(15);
        let display = format!("{}", error);
        assert!(display.contains("Line 5"));
        assert!(display.contains("Column 15"));
        assert!(display.contains("WARNING"));
    }

    // ───────────────────────────────────────────────────────────────
    // ParsedMetadata Tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn parsed_metadata_empty() {
        let metadata = ParsedMetadata::empty();
        assert!(!metadata.has_content());
    }

    #[test]
    fn parsed_metadata_with_title() {
        let metadata = ParsedMetadata {
            title: Some("My Decision".to_string()),
            ..Default::default()
        };
        assert!(metadata.has_content());
    }
}
