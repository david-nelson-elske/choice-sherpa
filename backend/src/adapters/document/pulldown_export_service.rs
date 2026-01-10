//! Pulldown-cmark based export service adapter.
//!
//! This adapter provides document export capabilities:
//! - HTML conversion using pulldown-cmark (pure Rust, no external dependencies)
//! - PDF conversion using Pandoc (requires external Pandoc installation)
//!
//! # Architecture
//!
//! This adapter implements the `DocumentExportService` port from the hexagonal
//! architecture. The domain depends on the port trait, while this concrete
//! implementation provides the actual conversion logic.

use std::process::Stdio;

use async_trait::async_trait;
use pulldown_cmark::{html, Options, Parser};
use tokio::process::Command;

use crate::ports::{DocumentExportService, ExportError};

/// Export service using pulldown-cmark for HTML and Pandoc for PDF.
///
/// # HTML Conversion
///
/// Uses the pure Rust `pulldown-cmark` library for markdown to HTML conversion.
/// This has no external dependencies and works on all platforms.
///
/// # PDF Conversion
///
/// Uses Pandoc for PDF conversion. Pandoc must be installed on the system.
/// If Pandoc is not available, PDF conversion will return a `ServiceUnavailable` error.
///
/// # Example
///
/// ```rust,ignore
/// let service = PulldownExportService::new();
///
/// // HTML conversion (always available)
/// let html = service.to_html("# Hello\n\nWorld").await?;
///
/// // PDF conversion (requires Pandoc)
/// let pdf = service.to_pdf("# Hello\n\nWorld").await?;
/// ```
#[derive(Debug, Clone, Default)]
pub struct PulldownExportService {
    /// Path to pandoc executable. If None, will search PATH.
    pandoc_path: Option<String>,

    /// Timeout for PDF conversion in seconds.
    pdf_timeout_secs: u64,

    /// Include default CSS styling for HTML output.
    include_default_css: bool,
}

impl PulldownExportService {
    /// Create a new export service with default settings.
    pub fn new() -> Self {
        Self {
            pandoc_path: None,
            pdf_timeout_secs: 30,
            include_default_css: true,
        }
    }

    /// Set a custom path to the Pandoc executable.
    pub fn with_pandoc_path(mut self, path: impl Into<String>) -> Self {
        self.pandoc_path = Some(path.into());
        self
    }

    /// Set the timeout for PDF conversion.
    pub fn with_pdf_timeout(mut self, timeout_secs: u64) -> Self {
        self.pdf_timeout_secs = timeout_secs;
        self
    }

    /// Disable default CSS styling for HTML output.
    pub fn without_default_css(mut self) -> Self {
        self.include_default_css = false;
        self
    }

    /// Get the pandoc command path.
    fn pandoc_command(&self) -> &str {
        self.pandoc_path.as_deref().unwrap_or("pandoc")
    }

    /// Wrap HTML content in a complete document with styling.
    fn wrap_html(&self, body: String, title: &str) -> String {
        let css = if self.include_default_css {
            DEFAULT_CSS
        } else {
            ""
        };

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
{css}
    </style>
</head>
<body>
    <article class="decision-document">
{body}
    </article>
</body>
</html>"#,
            title = html_escape(title),
            css = css,
            body = body
        )
    }

    /// Extract title from markdown content (first h1 heading).
    fn extract_title(&self, markdown: &str) -> String {
        for line in markdown.lines() {
            let trimmed = line.trim();
            if let Some(title) = trimmed.strip_prefix("# ") {
                // Remove any trailing markdown/special chars
                return title.split(':').next().unwrap_or(title).trim().to_string();
            }
        }
        "Decision Document".to_string()
    }

    /// Check if Pandoc is installed and accessible.
    async fn check_pandoc(&self) -> bool {
        let output = Command::new(self.pandoc_command())
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .await;

        output.map(|o| o.status.success()).unwrap_or(false)
    }
}

#[async_trait]
impl DocumentExportService for PulldownExportService {
    async fn to_pdf(&self, markdown: &str) -> Result<Vec<u8>, ExportError> {
        // Check if Pandoc is available
        if !self.check_pandoc().await {
            return Err(ExportError::service_unavailable(
                "Pandoc is not installed. PDF export requires Pandoc. \
                 Install from https://pandoc.org/installing.html",
            ));
        }

        // Run pandoc to convert markdown to PDF
        let mut child = Command::new(self.pandoc_command())
            .args([
                "-f",
                "markdown",
                "-t",
                "pdf",
                "--pdf-engine=xelatex",
                "-V",
                "geometry:margin=1in",
                "-V",
                "fontsize=11pt",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ExportError::pdf_failed(format!("Failed to start Pandoc: {}", e)))?;

        // Write markdown to stdin
        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            stdin
                .write_all(markdown.as_bytes())
                .await
                .map_err(|e| ExportError::pdf_failed(format!("Failed to write to Pandoc: {}", e)))?;
        }

        // Wait for completion with timeout
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(self.pdf_timeout_secs),
            child.wait_with_output(),
        )
        .await
        .map_err(|_| ExportError::Timeout(self.pdf_timeout_secs))?
        .map_err(|e| ExportError::pdf_failed(format!("Pandoc execution failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ExportError::pdf_failed(format!(
                "Pandoc returned error: {}",
                stderr.trim()
            )));
        }

        Ok(output.stdout)
    }

    async fn to_html(&self, markdown: &str) -> Result<String, ExportError> {
        // Configure parser options for full GFM support
        let options = Options::ENABLE_TABLES
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_SMART_PUNCTUATION;

        // Parse markdown
        let parser = Parser::new_ext(markdown, options);

        // Render to HTML
        let mut html_body = String::new();
        html::push_html(&mut html_body, parser);

        // Extract title and wrap in full document
        let title = self.extract_title(markdown);
        let full_html = self.wrap_html(html_body, &title);

        Ok(full_html)
    }

    async fn is_available(&self) -> bool {
        // HTML conversion is always available (pure Rust)
        // This method indicates if the service can do basic operations
        true
    }
}

/// Escape HTML special characters.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Default CSS for styled HTML output.
const DEFAULT_CSS: &str = r#"
:root {
    --primary-color: #2563eb;
    --text-color: #1f2937;
    --muted-color: #6b7280;
    --border-color: #e5e7eb;
    --bg-color: #ffffff;
    --code-bg: #f3f4f6;
}

* {
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
    font-size: 16px;
    line-height: 1.6;
    color: var(--text-color);
    background-color: var(--bg-color);
    margin: 0;
    padding: 2rem;
    max-width: 900px;
    margin: 0 auto;
}

.decision-document {
    padding: 2rem;
}

h1, h2, h3, h4, h5, h6 {
    margin-top: 1.5em;
    margin-bottom: 0.5em;
    font-weight: 600;
    line-height: 1.25;
}

h1 {
    font-size: 2rem;
    border-bottom: 2px solid var(--primary-color);
    padding-bottom: 0.5rem;
}

h2 {
    font-size: 1.5rem;
    border-bottom: 1px solid var(--border-color);
    padding-bottom: 0.25rem;
}

h3 {
    font-size: 1.25rem;
}

p {
    margin: 1em 0;
}

blockquote {
    margin: 1em 0;
    padding: 0.5em 1em;
    border-left: 4px solid var(--primary-color);
    background-color: var(--code-bg);
    color: var(--muted-color);
}

blockquote p {
    margin: 0;
}

ul, ol {
    margin: 1em 0;
    padding-left: 2em;
}

li {
    margin: 0.25em 0;
}

table {
    width: 100%;
    border-collapse: collapse;
    margin: 1em 0;
}

th, td {
    padding: 0.5rem;
    text-align: left;
    border: 1px solid var(--border-color);
}

th {
    background-color: var(--code-bg);
    font-weight: 600;
}

tr:nth-child(even) {
    background-color: #f9fafb;
}

code {
    font-family: 'SF Mono', 'Fira Code', 'Consolas', monospace;
    font-size: 0.875em;
    background-color: var(--code-bg);
    padding: 0.125em 0.25em;
    border-radius: 3px;
}

pre {
    background-color: var(--code-bg);
    padding: 1em;
    border-radius: 6px;
    overflow-x: auto;
}

pre code {
    background-color: transparent;
    padding: 0;
}

hr {
    border: none;
    border-top: 1px solid var(--border-color);
    margin: 2em 0;
}

a {
    color: var(--primary-color);
    text-decoration: none;
}

a:hover {
    text-decoration: underline;
}

input[type="checkbox"] {
    margin-right: 0.5em;
}

@media print {
    body {
        font-size: 12pt;
        padding: 0;
    }

    .decision-document {
        padding: 0;
    }

    h1, h2, h3 {
        page-break-after: avoid;
    }

    table, figure {
        page-break-inside: avoid;
    }
}
"#;

// ════════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ───────────────────────────────────────────────────────────────
    // HTML conversion tests
    // ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn to_html_converts_basic_markdown() {
        let service = PulldownExportService::new();
        let markdown = "# Test Document\n\nHello, world!";

        let html = service.to_html(markdown).await.unwrap();

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Test Document</title>"));
        assert!(html.contains("<h1>Test Document</h1>"));
        assert!(html.contains("<p>Hello, world!</p>"));
    }

    #[tokio::test]
    async fn to_html_converts_tables() {
        let service = PulldownExportService::new();
        let markdown = "# Doc\n\n| A | B |\n|---|---|\n| 1 | 2 |";

        let html = service.to_html(markdown).await.unwrap();

        assert!(html.contains("<table>"));
        assert!(html.contains("<th>A</th>"));
        assert!(html.contains("<td>1</td>"));
    }

    #[tokio::test]
    async fn to_html_converts_task_lists() {
        let service = PulldownExportService::new();
        let markdown = "# Tasks\n\n- [x] Done\n- [ ] Todo";

        let html = service.to_html(markdown).await.unwrap();

        assert!(html.contains("type=\"checkbox\""));
        assert!(html.contains("checked"));
    }

    #[tokio::test]
    async fn to_html_converts_blockquotes() {
        let service = PulldownExportService::new();
        let markdown = "# Quote\n\n> Important note";

        let html = service.to_html(markdown).await.unwrap();

        assert!(html.contains("<blockquote>"));
        assert!(html.contains("Important note"));
    }

    #[tokio::test]
    async fn to_html_includes_default_css() {
        let service = PulldownExportService::new();
        let markdown = "# Test";

        let html = service.to_html(markdown).await.unwrap();

        assert!(html.contains("<style>"));
        assert!(html.contains("--primary-color"));
    }

    #[tokio::test]
    async fn to_html_without_css_excludes_styling() {
        let service = PulldownExportService::new().without_default_css();
        let markdown = "# Test";

        let html = service.to_html(markdown).await.unwrap();

        assert!(html.contains("<style>"));
        // Empty style block
        assert!(!html.contains("--primary-color"));
    }

    // ───────────────────────────────────────────────────────────────
    // Title extraction tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn extract_title_from_h1() {
        let service = PulldownExportService::new();

        assert_eq!(
            service.extract_title("# My Decision\n\nContent"),
            "My Decision"
        );
    }

    #[test]
    fn extract_title_with_colon() {
        let service = PulldownExportService::new();

        assert_eq!(
            service.extract_title("# Career Choice: Next Steps\n\nContent"),
            "Career Choice"
        );
    }

    #[test]
    fn extract_title_default_when_no_h1() {
        let service = PulldownExportService::new();

        assert_eq!(
            service.extract_title("## Not an H1\n\nContent"),
            "Decision Document"
        );
    }

    // ───────────────────────────────────────────────────────────────
    // Service availability tests
    // ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn is_available_returns_true() {
        let service = PulldownExportService::new();
        assert!(service.is_available().await);
    }

    // ───────────────────────────────────────────────────────────────
    // Builder tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn builder_sets_pandoc_path() {
        let service = PulldownExportService::new().with_pandoc_path("/usr/local/bin/pandoc");

        assert_eq!(service.pandoc_command(), "/usr/local/bin/pandoc");
    }

    #[test]
    fn builder_sets_timeout() {
        let service = PulldownExportService::new().with_pdf_timeout(60);

        assert_eq!(service.pdf_timeout_secs, 60);
    }

    // ───────────────────────────────────────────────────────────────
    // HTML escape tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn html_escape_escapes_special_chars() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
    }

    // ───────────────────────────────────────────────────────────────
    // Integration test: Decision document conversion
    // ───────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn converts_decision_document_markdown_to_html() {
        let service = PulldownExportService::new();
        let markdown = r#"# Career Decision: Job Offer

> **Status:** In Progress | **Quality Score:** 75%
> **Last Updated:** 2024-01-10 by agent

---

## 1. Issue Raising

### Potential Decisions
- [x] Accept new job offer
- [ ] Stay at current company
- [ ] Negotiate better terms

### Objectives Identified
- Maximize compensation
- Maintain work-life balance

---

## 2. Problem Frame

**Decision Maker:** John Smith (Principal Engineer)

**Focal Decision:**
> Should I accept the job offer from TechCorp?

| Level | Decision | Status |
|-------|----------|--------|
| **Focal** | Accept TechCorp offer | In Progress |

---
"#;

        let html = service.to_html(markdown).await.unwrap();

        // Verify structure
        assert!(html.contains("<title>Career Decision</title>"));
        assert!(html.contains("<h1>Career Decision: Job Offer</h1>"));
        assert!(html.contains("<h2>1. Issue Raising</h2>"));
        assert!(html.contains("<h2>2. Problem Frame</h2>"));

        // Verify blockquote
        assert!(html.contains("<blockquote>"));
        assert!(html.contains("In Progress"));

        // Verify table
        assert!(html.contains("<table>"));
        assert!(html.contains("TechCorp"));

        // Verify task list
        assert!(html.contains("type=\"checkbox\""));
    }
}
