//! Document adapters - Implementations for document generation and storage.
//!
//! This module provides adapters for the document-related ports:
//! - `TemplateDocumentGenerator` - Generates markdown from PrOACT components

mod template_generator;

pub use template_generator::TemplateDocumentGenerator;
