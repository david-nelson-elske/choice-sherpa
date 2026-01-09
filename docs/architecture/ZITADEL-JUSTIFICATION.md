# Authentication Provider Selection: Zitadel

> **Decision:** Zitadel (self-hosted)
> **Date:** 2026-01-07

---

## Summary

Zitadel selected as authentication provider for first-class Rust support, official SvelteKit documentation, and clean hexagonal architecture integration via standard OIDC patterns.

---

## Key Factors

### 1. First-Class Rust Support

Zitadel provides an actively maintained Rust crate with dedicated axum integration:

```toml
[dependencies]
zitadel = { version = "3.4", features = ["axum", "credentials"] }
```

| Feature Flag | Purpose |
|--------------|---------|
| `axum` | Framework integration helpers |
| `credentials` | Service account authentication |
| `api` | Full gRPC/REST API access |

Crate actively maintained with issues addressed as recently as January 2025.

**Comparison:**
- Ory Kratos: Generic API client only, no framework integration
- SuperTokens: Community Rust crate unmaintained since 2022
- Authentik: No Rust support, OIDC only

### 2. Official SvelteKit Documentation

Zitadel provides [official SvelteKit integration guide](https://zitadel.com/docs/sdk-examples/sveltekit) using @auth/sveltekit (Auth.js):

- PKCE flow implementation
- Session management
- Automatic token refresh
- Federated logout with CSRF protection

### 3. Operational Simplicity

| Aspect | Zitadel | Ory Kratos |
|--------|---------|------------|
| Deployment | Single Go binary | Multiple components |
| Database | PostgreSQL | PostgreSQL |
| UI | Built-in (optional) | Requires custom UI |
| Email | Built-in | Singleton courier worker |

### 4. Standards Compliance

- OpenID Certified provider
- Standard OIDC/OAuth2 flows
- Swappable for any OIDC provider without domain changes

---

## Hexagonal Architecture Compliance

### Coupling Prevention

Zitadel integration follows strict port/adapter separation:

| Layer | Zitadel Dependency |
|-------|-------------------|
| Domain | ❌ None |
| Ports | ❌ None |
| Application | ❌ None |
| Adapters | ✅ `zitadel` crate |

### Port Definition

```rust
// ports/auth.rs

#[async_trait]
pub trait SessionValidator: Send + Sync {
    async fn validate(&self, token: &str) -> Result<AuthenticatedUser, AuthError>;
}

#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn get_user(&self, user_id: &UserID) -> Result<UserProfile, AuthError>;
}
```

### Domain Types (No External Dependencies)

```rust
// domain/foundation/user.rs

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UserID(String);

// domain/foundation/auth.rs

pub struct AuthenticatedUser {
    pub id: UserID,
    pub email: String,
    pub display_name: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid or expired token")]
    InvalidToken,
    #[error("User not found")]
    UserNotFound,
    #[error("Service unavailable")]
    ServiceUnavailable,
}
```

### Adapter Implementation

```rust
// adapters/auth/zitadel.rs

use zitadel::credentials::Application;
use crate::ports::{SessionValidator, AuthenticatedUser, AuthError};

pub struct ZitadelValidator {
    client: Application,
}

#[async_trait]
impl SessionValidator for ZitadelValidator {
    async fn validate(&self, token: &str) -> Result<AuthenticatedUser, AuthError> {
        let introspection = self.client
            .introspect(token)
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        // Map Zitadel types → Domain types
        Ok(AuthenticatedUser {
            id: UserID::new(introspection.sub),
            email: introspection.email.unwrap_or_default(),
            display_name: introspection.name,
        })
    }
}
```

### Swappability Verified

Provider can be swapped without domain changes:

```rust
// Zitadel
let validator: Arc<dyn SessionValidator> = Arc::new(ZitadelValidator::new(config));

// Hypothetical Kratos swap
let validator: Arc<dyn SessionValidator> = Arc::new(KratosValidator::new(config));

// Domain code unchanged
```

---

## Integration Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      SvelteKit Frontend                      │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ @auth/sveltekit (Auth.js)                           │    │
│  │ - OIDC PKCE flow                                    │    │
│  │ - Session cookies                                   │    │
│  │ - Token refresh                                     │    │
│  └──────────────────────────┬──────────────────────────┘    │
└─────────────────────────────┼───────────────────────────────┘
                              │ OIDC
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                        Zitadel                               │
│  - User registration/login                                  │
│  - Session management                                       │
│  - MFA (TOTP, WebAuthn)                                     │
│  - JWT issuance                                             │
│  PostgreSQL backend                                         │
└──────────────────────────────┬──────────────────────────────┘
                               │ JWT in Authorization header
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                    Rust Backend (axum)                       │
│  ┌────────────────────────────────────────────────────┐     │
│  │ Auth Middleware                                     │     │
│  │ - Extract Bearer token                             │     │
│  │ - Call SessionValidator port                       │     │
│  │ - Inject AuthenticatedUser into request           │     │
│  └────────────────────────────────────────────────────┘     │
│  ┌────────────────────────────────────────────────────┐     │
│  │ Protected Handlers                                  │     │
│  │ - Receive domain AuthenticatedUser type            │     │
│  │ - No Zitadel dependency                            │     │
│  └────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

---

## Configuration Approach

Generic OIDC configuration terms (not Zitadel-specific):

```rust
// config.rs

pub struct AuthConfig {
    pub issuer_url: String,     // Generic OIDC
    pub client_id: String,      // Generic OIDC
    pub client_secret: String,  // Generic OIDC
    pub audience: String,       // Generic OIDC
}
```

Environment variables:

```bash
AUTH_ISSUER_URL=https://auth.example.com
AUTH_CLIENT_ID=choice-sherpa-backend
AUTH_CLIENT_SECRET=<secret>
AUTH_AUDIENCE=https://api.example.com
```

---

## Testing Strategy

Port abstraction enables testing without Zitadel:

```rust
pub struct MockSessionValidator {
    users: HashMap<String, AuthenticatedUser>,
}

impl SessionValidator for MockSessionValidator {
    async fn validate(&self, token: &str) -> Result<AuthenticatedUser, AuthError> {
        self.users.get(token)
            .cloned()
            .ok_or(AuthError::InvalidToken)
    }
}

#[tokio::test]
async fn test_protected_endpoint() {
    let validator = MockSessionValidator {
        users: hashmap! {
            "valid-token" => AuthenticatedUser {
                id: UserID::new("user-123"),
                email: "test@example.com".into(),
                display_name: Some("Test User".into()),
            }
        }
    };

    // Test with mock, no Zitadel required
}
```

---

## Coupling Prevention Checklist

| Scenario | Mitigation |
|----------|------------|
| User ID format | Opaque `UserID(String)`, no format validation |
| JWT claims | Extracted in adapter, mapped to domain types |
| Zitadel errors | Mapped to domain `AuthError` at adapter boundary |
| Configuration | Generic OIDC terminology |
| Roles/permissions | Domain `Permission` enum, adapter maps from Zitadel |
| SvelteKit session | Auth.js provides abstraction layer |

---

## Trade-off Accepted

Zitadel is younger than Keycloak with smaller community, accepted in exchange for:

- First-class Rust support with axum integration
- Modern architecture (Go, PostgreSQL)
- Simpler deployment than Ory Kratos
- Official SvelteKit documentation
- OpenID Certified compliance

---

## Alternatives Considered

| Provider | Rejection Reason |
|----------|------------------|
| Ory Kratos | No Rust SDK, high deployment complexity |
| SuperTokens | No Rust support (community crate dead since 2022) |
| Keycloak | Java overhead, overkill for requirements |
| Authentik | No Rust support |

---

## Sources

- [Zitadel Rust crate (smartive)](https://github.com/smartive/zitadel-rust)
- [Zitadel crate on crates.io](https://crates.io/crates/zitadel)
- [Zitadel SvelteKit documentation](https://zitadel.com/docs/sdk-examples/sveltekit)
- [Zitadel features overview](https://zitadel.com/features)
- [Auth.js SvelteKit integration](https://authjs.dev/reference/sveltekit)
