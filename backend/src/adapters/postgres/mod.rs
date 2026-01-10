//! PostgreSQL adapters - Database implementations for repository ports.
//!
//! This module provides adapters for PostgreSQL-backed persistence:
//! - `PostgresDocumentRepository` - Coordinates DB metadata with filesystem content
//! - `PostgresDocumentReader` - Read-optimized document queries

mod document_reader;
mod document_repository;

pub use document_reader::PostgresDocumentReader;
pub use document_repository::PostgresDocumentRepository;
