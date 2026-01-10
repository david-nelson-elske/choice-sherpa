//! Authentication adapters.
//!
//! Implementations of the `SessionValidator` and `AuthProvider` ports:
//!
//! - `mock` - Test implementations that don't require external services
//! - (future) `zitadel` - Production Zitadel OIDC implementation

mod mock;

pub use mock::{MockAuthProvider, MockSessionValidator};
