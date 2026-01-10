//! HTTP adapters - REST API implementations.
//!
//! Each domain module has its own HTTP adapter for endpoint exposure.

pub mod membership;

pub use membership::*;
