//! CircuitBreaker port - Interface for external service resilience.
//!
//! The circuit breaker pattern prevents cascading failures when external
//! services (AI providers, payment processors) become unavailable or slow.
//!
//! ## States
//!
//! - **Closed**: Normal operation, requests flow through
//! - **Open**: Too many failures, requests rejected immediately
//! - **Half-Open**: Testing if service recovered, limited requests allowed
//!
//! ## Transitions
//!
//! ```text
//! Closed --[failure_threshold exceeded]--> Open
//! Open --[recovery_timeout elapsed]--> Half-Open
//! Half-Open --[success_threshold reached]--> Closed
//! Half-Open --[any failure]--> Open
//! ```
//!
//! See `docs/architecture/SCALING-READINESS.md` for full details.

use std::time::Duration;

/// Circuit breaker states for external service protection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation - requests flow through to the service.
    Closed,

    /// Too many failures - requests rejected immediately without calling service.
    /// The circuit will transition to HalfOpen after recovery_timeout.
    Open,

    /// Testing if service recovered - limited requests allowed through.
    /// Success → Closed, Failure → Open.
    HalfOpen,
}

impl CircuitState {
    /// Check if the circuit allows requests through.
    pub fn allows_requests(&self) -> bool {
        matches!(self, CircuitState::Closed | CircuitState::HalfOpen)
    }
}

/// Configuration for circuit breaker behavior.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening circuit.
    ///
    /// Default: 5 failures
    pub failure_threshold: u32,

    /// Time to wait before testing recovery (moving to half-open).
    ///
    /// Default: 30 seconds
    pub recovery_timeout: Duration,

    /// Number of successes in half-open state needed to close circuit.
    ///
    /// Default: 3 successes
    pub success_threshold: u32,

    /// Optional: Maximum concurrent requests in half-open state.
    ///
    /// Default: 1 request at a time
    pub half_open_max_requests: u32,

    /// Optional: Time window for counting failures (sliding window).
    ///
    /// If set, only failures within this window count toward threshold.
    /// Default: None (all failures count until reset)
    pub failure_window: Option<Duration>,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            success_threshold: 3,
            half_open_max_requests: 1,
            failure_window: None,
        }
    }
}

impl CircuitBreakerConfig {
    /// Create a config optimized for AI providers (longer timeouts, lower threshold).
    pub fn for_ai_provider() -> Self {
        Self {
            failure_threshold: 3,
            recovery_timeout: Duration::from_secs(60),
            success_threshold: 2,
            half_open_max_requests: 1,
            failure_window: Some(Duration::from_secs(120)),
        }
    }

    /// Create a config optimized for payment providers (higher threshold, shorter timeout).
    pub fn for_payment_provider() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(15),
            success_threshold: 3,
            half_open_max_requests: 1,
            failure_window: Some(Duration::from_secs(60)),
        }
    }
}

/// Port for circuit breaker functionality.
///
/// Protects against cascading failures when external services become unavailable.
///
/// # Example
///
/// ```ignore
/// struct ResilientAIProvider {
///     inner: Arc<dyn AIProvider>,
///     circuit_breaker: Arc<dyn CircuitBreaker>,
/// }
///
/// impl AIProvider for ResilientAIProvider {
///     async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, AIError> {
///         // Check circuit breaker before calling service
///         if !self.circuit_breaker.should_allow() {
///             return Err(AIError::ModelUnavailable("Circuit breaker open".into()));
///         }
///
///         match self.inner.complete(request).await {
///             Ok(response) => {
///                 self.circuit_breaker.record_success();
///                 Ok(response)
///             }
///             Err(e) => {
///                 self.circuit_breaker.record_failure();
///                 Err(e)
///             }
///         }
///     }
/// }
/// ```
pub trait CircuitBreaker: Send + Sync {
    /// Get the current state of the circuit.
    fn state(&self) -> CircuitState;

    /// Check if a request should be allowed through.
    ///
    /// Returns `true` if the circuit is closed or half-open with capacity.
    /// Returns `false` if the circuit is open.
    ///
    /// In half-open state, this may limit concurrent requests.
    fn should_allow(&self) -> bool;

    /// Record a successful request.
    ///
    /// In half-open state, this counts toward the success threshold.
    /// In closed state, this may reset failure counts.
    fn record_success(&self);

    /// Record a failed request.
    ///
    /// In closed state, this counts toward the failure threshold.
    /// In half-open state, this immediately reopens the circuit.
    fn record_failure(&self);

    /// Force reset the circuit to closed state.
    ///
    /// Use sparingly - typically for administrative intervention.
    fn reset(&self);

    /// Get metrics about the circuit breaker.
    fn metrics(&self) -> CircuitBreakerMetrics;
}

/// Metrics about circuit breaker behavior.
#[derive(Debug, Clone, Default)]
pub struct CircuitBreakerMetrics {
    /// Current state
    pub state: Option<CircuitState>,

    /// Total successful requests since creation
    pub total_successes: u64,

    /// Total failed requests since creation
    pub total_failures: u64,

    /// Times the circuit has opened
    pub times_opened: u64,

    /// Current failure count (in closed state)
    pub current_failures: u32,

    /// Current success count (in half-open state)
    pub current_successes: u32,

    /// Time until circuit transitions to half-open (when open)
    pub time_until_half_open: Option<Duration>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circuit_state_allows_requests() {
        assert!(CircuitState::Closed.allows_requests());
        assert!(CircuitState::HalfOpen.allows_requests());
        assert!(!CircuitState::Open.allows_requests());
    }

    #[test]
    fn default_config_values() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.recovery_timeout, Duration::from_secs(30));
        assert_eq!(config.success_threshold, 3);
    }

    #[test]
    fn ai_provider_config() {
        let config = CircuitBreakerConfig::for_ai_provider();
        assert_eq!(config.failure_threshold, 3);
        assert_eq!(config.recovery_timeout, Duration::from_secs(60));
    }

    #[test]
    fn payment_provider_config() {
        let config = CircuitBreakerConfig::for_payment_provider();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.recovery_timeout, Duration::from_secs(15));
    }
}
