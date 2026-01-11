//! Rate limiter adapters.
//!
//! Implementations of the RateLimiter port for different backends.
//!
//! ## Available Adapters
//!
//! - `InMemoryRateLimiter` - In-memory for testing and single-server
//! - `RedisRateLimiter` - Redis-backed for production multi-server
//!
//! ## Usage
//!
//! ```ignore
//! use choice_sherpa::adapters::rate_limiter::{
//!     InMemoryRateLimiter, RateLimitConfig
//! };
//!
//! // For testing
//! let limiter = InMemoryRateLimiter::with_defaults();
//!
//! // For production
//! let limiter = RedisRateLimiter::new(redis_client, RateLimitConfig::default());
//! ```

mod config;
mod in_memory;
mod redis;

pub use config::{GlobalLimits, IpLimits, RateLimitConfig, ResourceLimits, TierRateLimits};
pub use in_memory::{InMemoryRateLimiter, TierAwareRateLimiter};
pub use redis::RedisRateLimiter;
