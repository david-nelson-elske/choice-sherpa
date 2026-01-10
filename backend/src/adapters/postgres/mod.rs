//! PostgreSQL adapters - Database implementations of repository ports.
//!
//! Provides persistence implementations using sqlx and PostgreSQL.
//!
//! # Tables
//!
//! - `cycles` - Cycle aggregate metadata
//! - `components` - Component data with JSONB outputs
//! - `memberships` - User membership/subscription data
//! - `promo_codes` - Promotional codes for free access

mod access_checker_impl;
mod cycle_reader;
mod cycle_repository;
mod membership_reader;
mod membership_repository;

pub use access_checker_impl::PostgresAccessChecker;
pub use cycle_reader::PostgresCycleReader;
pub use cycle_repository::PostgresCycleRepository;
pub use membership_reader::PostgresMembershipReader;
pub use membership_repository::PostgresMembershipRepository;
