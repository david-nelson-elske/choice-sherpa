//! Document Export Service Port - Format conversion interface.
//!
//! This port defines the contract for converting markdown documents to
//! other formats (PDF, HTML). The domain depends on this trait, while
//! adapters (like PandocExportService) provide the implementation.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Port for exporting markdown documents to other formats.
///
/// # Contract
///
/// Implementations must:
/// - Convert valid markdown to the target format
/// - Preserve document structure and formatting
/// - Handle embedded metadata (YAML frontmatter)
/// - Report clear errors for conversion failures
///
/// # Usage
///
/// ```rust,ignore
/// let export_service: &dyn DocumentExportService = get_service();
///
/// // Convert to PDF
/// let pdf_bytes = export_service.to_pdf("# My Document\n\nContent here").await?;
///
/// // Convert to HTML
/// let html = export_service.to_html("# My Document\n\nContent here").await?;
/// ```
#[async_trait]
pub trait DocumentExportService: Send + Sync {
    /// Convert markdown content to PDF bytes.
    ///
    /// The returned bytes are a valid PDF document that can be
    /// written to a file or sent as an HTTP response.
    ///
    /// # Arguments
    ///
    /// * `markdown` - The markdown content to convert
    ///
    /// # Errors
    ///
    /// Returns `ExportError` if conversion fails.
    async fn to_pdf(&self, markdown: &str) -> Result<Vec<u8>, ExportError>;

    /// Convert markdown content to HTML string.
    ///
    /// Returns a complete HTML document (with `<html>`, `<head>`, `<body>` tags)
    /// suitable for viewing in a browser or embedding.
    ///
    /// # Arguments
    ///
    /// * `markdown` - The markdown content to convert
    ///
    /// # Errors
    ///
    /// Returns `ExportError` if conversion fails.
    async fn to_html(&self, markdown: &str) -> Result<String, ExportError>;

    /// Check if the export service is available.
    ///
    /// Used for health checks and to verify external dependencies
    /// (like Pandoc) are properly configured.
    async fn is_available(&self) -> bool;
}

/// Export formats supported by the service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    /// Raw markdown (no conversion needed).
    Markdown,
    /// PDF document.
    Pdf,
    /// HTML document.
    Html,
}

impl ExportFormat {
    /// Get the MIME content type for this format.
    pub fn content_type(&self) -> &'static str {
        match self {
            ExportFormat::Markdown => "text/markdown; charset=utf-8",
            ExportFormat::Pdf => "application/pdf",
            ExportFormat::Html => "text/html; charset=utf-8",
        }
    }

    /// Get the file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Markdown => "md",
            ExportFormat::Pdf => "pdf",
            ExportFormat::Html => "html",
        }
    }
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::Markdown => write!(f, "markdown"),
            ExportFormat::Pdf => write!(f, "pdf"),
            ExportFormat::Html => write!(f, "html"),
        }
    }
}

impl std::str::FromStr for ExportFormat {
    type Err = ExportError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(ExportFormat::Markdown),
            "pdf" => Ok(ExportFormat::Pdf),
            "html" | "htm" => Ok(ExportFormat::Html),
            _ => Err(ExportError::UnsupportedFormat(s.to_string())),
        }
    }
}

/// Exported document with content and metadata.
#[derive(Debug, Clone)]
pub struct ExportedDocument {
    /// The exported content as bytes.
    pub content: Vec<u8>,
    /// The MIME content type.
    pub content_type: String,
    /// Suggested filename for download.
    pub filename: String,
    /// The format that was used.
    pub format: ExportFormat,
}

impl ExportedDocument {
    /// Create a new exported document.
    pub fn new(
        content: Vec<u8>,
        format: ExportFormat,
        base_filename: &str,
    ) -> Self {
        Self {
            content,
            content_type: format.content_type().to_string(),
            filename: format!("{}.{}", base_filename, format.extension()),
            format,
        }
    }

    /// Create from markdown content (no conversion needed).
    pub fn from_markdown(markdown: String, base_filename: &str) -> Self {
        Self::new(markdown.into_bytes(), ExportFormat::Markdown, base_filename)
    }

    /// Create from HTML content.
    pub fn from_html(html: String, base_filename: &str) -> Self {
        Self::new(html.into_bytes(), ExportFormat::Html, base_filename)
    }

    /// Create from PDF bytes.
    pub fn from_pdf(pdf_bytes: Vec<u8>, base_filename: &str) -> Self {
        Self::new(pdf_bytes, ExportFormat::Pdf, base_filename)
    }
}

/// Errors that can occur during document export.
#[derive(Debug, Clone, Error)]
pub enum ExportError {
    /// Unsupported export format requested.
    #[error("Unsupported export format: {0}")]
    UnsupportedFormat(String),

    /// External converter (e.g., Pandoc) is not available.
    #[error("Export service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Conversion to PDF failed.
    #[error("PDF conversion failed: {0}")]
    PdfConversionFailed(String),

    /// Conversion to HTML failed.
    #[error("HTML conversion failed: {0}")]
    HtmlConversionFailed(String),

    /// Input markdown is invalid.
    #[error("Invalid markdown input: {0}")]
    InvalidInput(String),

    /// Timeout during conversion.
    #[error("Conversion timed out after {0} seconds")]
    Timeout(u64),

    /// I/O error during conversion.
    #[error("I/O error during export: {0}")]
    IoError(String),
}

impl ExportError {
    /// Create a service unavailable error.
    pub fn service_unavailable(reason: impl Into<String>) -> Self {
        Self::ServiceUnavailable(reason.into())
    }

    /// Create a PDF conversion error.
    pub fn pdf_failed(reason: impl Into<String>) -> Self {
        Self::PdfConversionFailed(reason.into())
    }

    /// Create an HTML conversion error.
    pub fn html_failed(reason: impl Into<String>) -> Self {
        Self::HtmlConversionFailed(reason.into())
    }

    /// Create an I/O error.
    pub fn io_error(reason: impl Into<String>) -> Self {
        Self::IoError(reason.into())
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ───────────────────────────────────────────────────────────────
    // ExportFormat tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn export_format_content_types_are_correct() {
        assert_eq!(ExportFormat::Markdown.content_type(), "text/markdown; charset=utf-8");
        assert_eq!(ExportFormat::Pdf.content_type(), "application/pdf");
        assert_eq!(ExportFormat::Html.content_type(), "text/html; charset=utf-8");
    }

    #[test]
    fn export_format_extensions_are_correct() {
        assert_eq!(ExportFormat::Markdown.extension(), "md");
        assert_eq!(ExportFormat::Pdf.extension(), "pdf");
        assert_eq!(ExportFormat::Html.extension(), "html");
    }

    #[test]
    fn export_format_parses_from_string() {
        assert_eq!("markdown".parse::<ExportFormat>().unwrap(), ExportFormat::Markdown);
        assert_eq!("md".parse::<ExportFormat>().unwrap(), ExportFormat::Markdown);
        assert_eq!("pdf".parse::<ExportFormat>().unwrap(), ExportFormat::Pdf);
        assert_eq!("html".parse::<ExportFormat>().unwrap(), ExportFormat::Html);
        assert_eq!("htm".parse::<ExportFormat>().unwrap(), ExportFormat::Html);
        assert_eq!("HTML".parse::<ExportFormat>().unwrap(), ExportFormat::Html);
    }

    #[test]
    fn export_format_parse_rejects_unknown_format() {
        let result = "docx".parse::<ExportFormat>();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExportError::UnsupportedFormat(_)));
    }

    #[test]
    fn export_format_serializes_to_snake_case() {
        assert_eq!(serde_json::to_string(&ExportFormat::Markdown).unwrap(), "\"markdown\"");
        assert_eq!(serde_json::to_string(&ExportFormat::Pdf).unwrap(), "\"pdf\"");
        assert_eq!(serde_json::to_string(&ExportFormat::Html).unwrap(), "\"html\"");
    }

    #[test]
    fn export_format_displays_correctly() {
        assert_eq!(ExportFormat::Markdown.to_string(), "markdown");
        assert_eq!(ExportFormat::Pdf.to_string(), "pdf");
        assert_eq!(ExportFormat::Html.to_string(), "html");
    }

    // ───────────────────────────────────────────────────────────────
    // ExportedDocument tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn exported_document_from_markdown_creates_correctly() {
        let doc = ExportedDocument::from_markdown("# Test".to_string(), "decision");
        assert_eq!(doc.filename, "decision.md");
        assert_eq!(doc.content_type, "text/markdown; charset=utf-8");
        assert_eq!(doc.format, ExportFormat::Markdown);
        assert_eq!(doc.content, b"# Test");
    }

    #[test]
    fn exported_document_from_html_creates_correctly() {
        let doc = ExportedDocument::from_html("<html></html>".to_string(), "decision");
        assert_eq!(doc.filename, "decision.html");
        assert_eq!(doc.content_type, "text/html; charset=utf-8");
        assert_eq!(doc.format, ExportFormat::Html);
    }

    #[test]
    fn exported_document_from_pdf_creates_correctly() {
        let doc = ExportedDocument::from_pdf(vec![0x25, 0x50, 0x44, 0x46], "decision");
        assert_eq!(doc.filename, "decision.pdf");
        assert_eq!(doc.content_type, "application/pdf");
        assert_eq!(doc.format, ExportFormat::Pdf);
    }

    // ───────────────────────────────────────────────────────────────
    // ExportError tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn export_error_displays_messages() {
        let err = ExportError::service_unavailable("Pandoc not found");
        assert!(err.to_string().contains("Pandoc not found"));

        let err = ExportError::pdf_failed("Invalid input");
        assert!(err.to_string().contains("PDF conversion failed"));

        let err = ExportError::html_failed("Parse error");
        assert!(err.to_string().contains("HTML conversion failed"));
    }

    // ───────────────────────────────────────────────────────────────
    // Trait object safety test
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn document_export_service_is_object_safe() {
        fn check<T: DocumentExportService + ?Sized>() {}
        // This compiles only if the trait is object-safe
        check::<dyn DocumentExportService>();
    }
}
