//! Authentication adapters.
//!
//! Implementations of the `SessionValidator` and `AuthProvider` ports:
//!
//! - `mock` - Test implementations that don't require external services
//! - `zitadel` - Production Zitadel OIDC implementation

mod mock;
mod zitadel;

pub use mock::{MockAuthProvider, MockSessionValidator};
pub use zitadel::{ZitadelConfig, ZitadelSessionValidator};
