//! Document adapters - Implementations for document generation and storage.
//!
//! This module provides adapters for the document-related ports:
//! - `TemplateDocumentGenerator` - Generates markdown from PrOACT components
//! - `MarkdownDocumentParser` - Parses markdown back to structured data
//! - `LocalDocumentFileStorage` - Stores documents on local filesystem
//! - `PulldownExportService` - Exports documents to HTML/PDF formats

mod local_file_storage;
mod markdown_parser;
mod pulldown_export_service;
mod template_generator;

pub use local_file_storage::LocalDocumentFileStorage;
pub use markdown_parser::MarkdownDocumentParser;
pub use pulldown_export_service::PulldownExportService;
pub use template_generator::TemplateDocumentGenerator;
