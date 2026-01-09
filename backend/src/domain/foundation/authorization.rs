//! Authorization support types and traits.
//!
//! This module provides foundation-level authorization support:
//! - `AuthorizationContext` - Structured context for authorization decisions
//! - `AuthorizationResult` - Standard result type for authorization checks
//! - Helper functions for common authorization patterns
//!
//! # Architecture Note
//!
//! The concrete `AuthorizationHelper` service that loads resources from
//! repositories belongs in the **application layer**, not here. This module
//! provides the building blocks; the orchestration lives elsewhere.
//!
//! ```text
//! foundation/authorization.rs  <- Types and traits (this module)
//! application/authorization.rs <- AuthorizationHelper service (uses repos)
//! ```
//!
//! # DRY Pattern
//!
//! Authorization checks follow a consistent pattern:
//! 1. Load resource from repository
//! 2. Check ownership/permissions
//! 3. Log the result (success or failure)
//! 4. Return or raise appropriate error
//!
//! This module provides the types to make step 2-4 consistent across handlers.

use super::{DomainError, ErrorCode, UserId};

/// Result of an authorization check.
///
/// Contains both the decision and context for logging/auditing.
#[derive(Debug, Clone)]
pub struct AuthorizationResult {
    /// Whether access was granted.
    pub granted: bool,

    /// The resource type being accessed (e.g., "Session", "Cycle").
    pub resource_type: &'static str,

    /// The ID of the resource being accessed.
    pub resource_id: String,

    /// The user who requested access.
    pub user_id: String,

    /// Optional reason for denial (if denied).
    pub denial_reason: Option<String>,
}

impl AuthorizationResult {
    /// Creates a successful authorization result.
    pub fn granted(
        resource_type: &'static str,
        resource_id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> Self {
        Self {
            granted: true,
            resource_type,
            resource_id: resource_id.into(),
            user_id: user_id.into(),
            denial_reason: None,
        }
    }

    /// Creates a denied authorization result.
    pub fn denied(
        resource_type: &'static str,
        resource_id: impl Into<String>,
        user_id: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            granted: false,
            resource_type,
            resource_id: resource_id.into(),
            user_id: user_id.into(),
            denial_reason: Some(reason.into()),
        }
    }

    /// Converts this result to a `Result<(), DomainError>`.
    ///
    /// Returns `Ok(())` if granted, `Err(Forbidden)` if denied.
    pub fn into_result(self) -> Result<(), DomainError> {
        if self.granted {
            Ok(())
        } else {
            Err(DomainError::new(
                ErrorCode::Forbidden,
                self.denial_reason
                    .unwrap_or_else(|| "Access denied".to_string()),
            )
            .with_detail("resource_type", self.resource_type)
            .with_detail("resource_id", self.resource_id)
            .with_detail("user_id", self.user_id))
        }
    }

    /// Returns true if access was granted.
    pub fn is_granted(&self) -> bool {
        self.granted
    }

    /// Returns true if access was denied.
    pub fn is_denied(&self) -> bool {
        !self.granted
    }
}

/// Context for authorization decisions.
///
/// Captures the "who, what, why" of an authorization request for
/// consistent logging and auditing across all handlers.
#[derive(Debug, Clone)]
pub struct AuthorizationContext {
    /// The user requesting access.
    pub user_id: UserId,

    /// The action being performed (e.g., "read", "update", "delete").
    pub action: String,

    /// The type of resource (e.g., "Session", "Cycle").
    pub resource_type: &'static str,

    /// The ID of the specific resource.
    pub resource_id: String,

    /// Optional correlation ID for request tracing.
    pub correlation_id: Option<String>,
}

impl AuthorizationContext {
    /// Creates a new authorization context.
    pub fn new(
        user_id: UserId,
        action: impl Into<String>,
        resource_type: &'static str,
        resource_id: impl Into<String>,
    ) -> Self {
        Self {
            user_id,
            action: action.into(),
            resource_type,
            resource_id: resource_id.into(),
            correlation_id: None,
        }
    }

    /// Adds a correlation ID for request tracing.
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Creates an `AuthorizationResult` for granted access.
    pub fn grant(&self) -> AuthorizationResult {
        AuthorizationResult::granted(
            self.resource_type,
            &self.resource_id,
            self.user_id.to_string(),
        )
    }

    /// Creates an `AuthorizationResult` for denied access.
    pub fn deny(&self, reason: impl Into<String>) -> AuthorizationResult {
        AuthorizationResult::denied(
            self.resource_type,
            &self.resource_id,
            self.user_id.to_string(),
            reason,
        )
    }
}

/// Helper trait for resources that can generate authorization contexts.
///
/// Implemented by aggregates that support authorization checks.
/// Works together with `OwnedByUser` to provide full authorization support.
pub trait Authorizable {
    /// The type name for authorization logs (e.g., "Session", "Cycle").
    const RESOURCE_TYPE: &'static str;

    /// Returns a string representation of this resource's ID.
    fn resource_id(&self) -> String;
}

/// Helper function to check ownership and return appropriate result.
///
/// Use this in handlers for consistent authorization checking:
///
/// ```ignore
/// let ctx = AuthorizationContext::new(
///     metadata.user_id.clone(),
///     "update",
///     Session::RESOURCE_TYPE,
///     session.id().to_string(),
/// );
///
/// let result = check_ownership(&session, &metadata.user_id);
/// log_authorization(&ctx, &result); // Your logging implementation
/// result.into_result()?;
/// ```
pub fn check_ownership<T>(
    resource: &T,
    user_id: &UserId,
) -> AuthorizationResult
where
    T: super::OwnedByUser + Authorizable,
{
    if resource.is_owner(user_id) {
        AuthorizationResult::granted(
            T::RESOURCE_TYPE,
            resource.resource_id(),
            user_id.to_string(),
        )
    } else {
        AuthorizationResult::denied(
            T::RESOURCE_TYPE,
            resource.resource_id(),
            user_id.to_string(),
            format!(
                "User {} does not own {} {}",
                user_id,
                T::RESOURCE_TYPE,
                resource.resource_id()
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::OwnedByUser;
    use super::*;

    // Test resource implementing both traits
    struct TestResource {
        id: String,
        owner: UserId,
    }

    impl OwnedByUser for TestResource {
        fn owner_id(&self) -> &UserId {
            &self.owner
        }
    }

    impl Authorizable for TestResource {
        const RESOURCE_TYPE: &'static str = "TestResource";

        fn resource_id(&self) -> String {
            self.id.clone()
        }
    }

    fn test_user(id: &str) -> UserId {
        UserId::new(id).unwrap()
    }

    // ============================================================
    // AuthorizationResult Tests
    // ============================================================

    #[test]
    fn authorization_result_granted_creates_success() {
        let result = AuthorizationResult::granted("Session", "sess-123", "user-456");

        assert!(result.is_granted());
        assert!(!result.is_denied());
        assert_eq!(result.resource_type, "Session");
        assert_eq!(result.resource_id, "sess-123");
        assert_eq!(result.user_id, "user-456");
        assert!(result.denial_reason.is_none());
    }

    #[test]
    fn authorization_result_denied_creates_failure() {
        let result = AuthorizationResult::denied(
            "Cycle",
            "cycle-789",
            "user-abc",
            "Not the owner",
        );

        assert!(result.is_denied());
        assert!(!result.is_granted());
        assert_eq!(result.denial_reason, Some("Not the owner".to_string()));
    }

    #[test]
    fn authorization_result_into_result_ok_for_granted() {
        let result = AuthorizationResult::granted("Session", "s-1", "u-1");
        assert!(result.into_result().is_ok());
    }

    #[test]
    fn authorization_result_into_result_err_for_denied() {
        let result = AuthorizationResult::denied("Session", "s-1", "u-1", "Denied");
        let err = result.into_result().unwrap_err();

        assert_eq!(err.code, ErrorCode::Forbidden);
        assert_eq!(err.details.get("resource_type"), Some(&"Session".to_string()));
        assert_eq!(err.details.get("resource_id"), Some(&"s-1".to_string()));
    }

    // ============================================================
    // AuthorizationContext Tests
    // ============================================================

    #[test]
    fn authorization_context_new_creates_context() {
        let ctx = AuthorizationContext::new(
            test_user("user-123"),
            "update",
            "Session",
            "session-456",
        );

        assert_eq!(ctx.user_id.as_str(), "user-123");
        assert_eq!(ctx.action, "update");
        assert_eq!(ctx.resource_type, "Session");
        assert_eq!(ctx.resource_id, "session-456");
        assert!(ctx.correlation_id.is_none());
    }

    #[test]
    fn authorization_context_with_correlation_id() {
        let ctx = AuthorizationContext::new(test_user("u"), "read", "Cycle", "c-1")
            .with_correlation_id("corr-789");

        assert_eq!(ctx.correlation_id, Some("corr-789".to_string()));
    }

    #[test]
    fn authorization_context_grant_creates_granted_result() {
        let ctx = AuthorizationContext::new(test_user("user-1"), "delete", "Session", "s-1");

        let result = ctx.grant();

        assert!(result.is_granted());
        assert_eq!(result.resource_type, "Session");
        assert_eq!(result.user_id, "user-1");
    }

    #[test]
    fn authorization_context_deny_creates_denied_result() {
        let ctx = AuthorizationContext::new(test_user("user-2"), "update", "Cycle", "c-2");

        let result = ctx.deny("No permission");

        assert!(result.is_denied());
        assert_eq!(result.denial_reason, Some("No permission".to_string()));
    }

    // ============================================================
    // check_ownership Tests
    // ============================================================

    #[test]
    fn check_ownership_grants_for_owner() {
        let owner = test_user("owner-123");
        let resource = TestResource {
            id: "res-1".to_string(),
            owner: owner.clone(),
        };

        let result = check_ownership(&resource, &owner);

        assert!(result.is_granted());
        assert_eq!(result.resource_type, "TestResource");
        assert_eq!(result.resource_id, "res-1");
    }

    #[test]
    fn check_ownership_denies_for_non_owner() {
        let owner = test_user("owner-123");
        let other = test_user("other-456");
        let resource = TestResource {
            id: "res-2".to_string(),
            owner,
        };

        let result = check_ownership(&resource, &other);

        assert!(result.is_denied());
        assert!(result.denial_reason.unwrap().contains("does not own"));
    }
}
