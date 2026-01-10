//! PostgreSQL adapters - Database implementations for repository ports.
//!
//! This module provides adapters for PostgreSQL-backed persistence:
//! - `PostgresDocumentRepository` - Coordinates DB metadata with filesystem content

mod document_repository;

pub use document_repository::PostgresDocumentRepository;
