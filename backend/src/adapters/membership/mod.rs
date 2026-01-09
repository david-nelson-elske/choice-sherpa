//! Membership adapters - implementations of membership-related ports.
//!
//! - `StubAccessChecker` - Development/testing stub that always allows access

mod stub_access_checker;

pub use stub_access_checker::StubAccessChecker;
