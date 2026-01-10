//! Document adapters - Implementations for document generation and storage.
//!
//! This module provides adapters for the document-related ports:
//! - `TemplateDocumentGenerator` - Generates markdown from PrOACT components
//! - `LocalDocumentFileStorage` - Stores documents on local filesystem

mod local_file_storage;
mod template_generator;

pub use local_file_storage::LocalDocumentFileStorage;
pub use template_generator::TemplateDocumentGenerator;
