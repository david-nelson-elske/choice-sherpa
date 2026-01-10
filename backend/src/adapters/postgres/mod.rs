//! PostgreSQL adapters - Database implementations of repository ports.
//!
//! Provides persistence implementations using sqlx and PostgreSQL.
//!
//! # Tables
//!
//! - `cycles` - Cycle aggregate metadata
//! - `components` - Component data with JSONB outputs

mod cycle_repository;
mod cycle_reader;

pub use cycle_repository::PostgresCycleRepository;
pub use cycle_reader::PostgresCycleReader;
