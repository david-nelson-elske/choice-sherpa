# Integration: Authentication & Identity

**Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md
**Provider:** Zitadel (self-hosted OIDC)
**Type:** External Service Integration
**Priority:** P0 (Required for all authenticated features)
**Depends On:** foundation module

> Zitadel-based authentication with thin adapter layer. All auth complexity delegated to the identity provider per hexagonal architecture principles.

---

## Overview

Authentication & Identity is handled by **Zitadel**, a self-hosted OIDC provider. Choice Sherpa implements a thin adapter layer that validates JWTs and maps claims to domain types. This approach:

1. **Eliminates auth complexity** from our codebase
2. **Provides enterprise features** (MFA, audit logs, admin UI) for free
3. **Maintains swappability** via standard OIDC

### Authentication Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      SvelteKit Frontend                                      │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ @auth/sveltekit (Auth.js)                                           │    │
│  │ - OIDC PKCE flow                                                    │    │
│  │ - Session cookies                                                   │    │
│  │ - Automatic token refresh                                           │    │
│  │ - Federated logout                                                  │    │
│  └──────────────────────────┬──────────────────────────────────────────┘    │
└─────────────────────────────┼───────────────────────────────────────────────┘
                              │ OIDC (Authorization Code + PKCE)
                              ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Zitadel                                            │
│  - User registration & login                                                │
│  - Session management                                                       │
│  - MFA (TOTP, WebAuthn, Passkeys)                                           │
│  - Email verification                                                       │
│  - Password reset                                                           │
│  - JWT issuance                                                             │
│  - Admin console (user management)                                          │
│  PostgreSQL backend                                                         │
└──────────────────────────────┬──────────────────────────────────────────────┘
                               │ JWT in Authorization header
                               ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                    Rust Backend (axum)                                       │
│  ┌───────────────────────────────────────────────────────────────────┐      │
│  │ Auth Middleware                                                    │      │
│  │ - Extract Bearer token                                            │      │
│  │ - Call SessionValidator port                                      │      │
│  │ - Inject AuthenticatedUser into request extensions               │      │
│  └───────────────────────────────────────────────────────────────────┘      │
│  ┌───────────────────────────────────────────────────────────────────┐      │
│  │ Protected Handlers                                                 │      │
│  │ - Receive domain AuthenticatedUser type                           │      │
│  │ - No Zitadel dependency in domain/application layers              │      │
│  └───────────────────────────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Zitadel Features Used

| Feature | Purpose |
|---------|---------|
| **OIDC/OAuth2** | Standard authentication flow |
| **JWT Access Tokens** | Stateless API authentication |
| **User Registration** | Self-service signup |
| **Password Reset** | Email-based recovery |
| **Email Verification** | Confirm user identity |
| **MFA (TOTP, WebAuthn)** | Enhanced security |
| **Admin Console** | User management UI |
| **Audit Logs** | Security compliance |

---

## Domain Types

Domain types have **no external dependencies**—pure Rust types that any auth provider can populate:

```rust
// backend/src/domain/foundation/user.rs

use serde::{Deserialize, Serialize};

/// Opaque user identifier
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserID(String);

impl UserID {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UserID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

```rust
// backend/src/domain/foundation/auth.rs

use super::UserID;

/// Authenticated user extracted from JWT
/// Domain type with no provider dependencies
#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub id: UserID,
    pub email: String,
    pub display_name: Option<String>,
    pub email_verified: bool,
}

/// Domain auth errors
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid or expired token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("User not found")]
    UserNotFound,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("Auth service unavailable")]
    ServiceUnavailable,
}
```

---

## Port Definitions

Ports define the contract without any provider knowledge:

```rust
// backend/src/ports/auth.rs

use async_trait::async_trait;
use crate::domain::foundation::{AuthenticatedUser, AuthError, UserID};

/// Validates access tokens and extracts user identity
#[async_trait]
pub trait SessionValidator: Send + Sync {
    /// Validate JWT and return authenticated user
    async fn validate(&self, token: &str) -> Result<AuthenticatedUser, AuthError>;
}

/// Retrieves user profile information
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Get user by ID
    async fn get_user(&self, user_id: &UserID) -> Result<AuthenticatedUser, AuthError>;
}
```

---

## Zitadel Adapter

The **only** place Zitadel appears in our codebase:

```rust
// backend/src/adapters/auth/zitadel.rs

use async_trait::async_trait;
use zitadel::credentials::Application;
use crate::domain::foundation::{AuthenticatedUser, AuthError, UserID};
use crate::ports::auth::SessionValidator;

pub struct ZitadelValidator {
    client: Application,
    /// Expected issuer URL for JWT validation (REQUIRED per A07)
    expected_issuer: String,
    /// Expected audience for JWT validation (REQUIRED per A07)
    expected_audience: String,
}

impl ZitadelValidator {
    pub fn new(
        issuer_url: &str,
        client_id: &str,
        client_secret: &str,
        expected_audience: &str,
    ) -> Result<Self, AuthError> {
        let client = Application::new(issuer_url, client_id, client_secret)
            .map_err(|_| AuthError::ServiceUnavailable)?;

        Ok(Self {
            client,
            expected_issuer: issuer_url.to_string(),
            expected_audience: expected_audience.to_string(),
        })
    }
}

#[async_trait]
impl SessionValidator for ZitadelValidator {
    async fn validate(&self, token: &str) -> Result<AuthenticatedUser, AuthError> {
        let introspection = self.client
            .introspect(token)
            .await
            .map_err(|e| {
                if e.to_string().contains("expired") {
                    AuthError::TokenExpired
                } else {
                    AuthError::InvalidToken
                }
            })?;

        if !introspection.active {
            return Err(AuthError::InvalidToken);
        }

        // SECURITY: Validate issuer (REQUIRED per APPLICATION-SECURITY-STANDARD.md A07)
        // Prevents token confusion attacks where tokens from other issuers are accepted
        if introspection.iss.as_deref() != Some(&self.expected_issuer) {
            tracing::warn!(
                "JWT issuer mismatch: expected '{}', got '{:?}'",
                self.expected_issuer,
                introspection.iss
            );
            return Err(AuthError::InvalidToken);
        }

        // SECURITY: Validate audience (REQUIRED per APPLICATION-SECURITY-STANDARD.md A07)
        // Prevents tokens intended for other services from being accepted
        if !introspection.aud.as_ref()
            .map(|a| a.contains(&self.expected_audience))
            .unwrap_or(false)
        {
            tracing::warn!(
                "JWT audience mismatch: expected '{}', got '{:?}'",
                self.expected_audience,
                introspection.aud
            );
            return Err(AuthError::InvalidToken);
        }

        // SECURITY: Validate expiry explicitly (REQUIRED per APPLICATION-SECURITY-STANDARD.md A07)
        // Defense in depth - even though introspection.active should cover this,
        // we explicitly check expiry to guard against introspection endpoint bugs
        if let Some(exp) = introspection.exp {
            if exp < chrono::Utc::now().timestamp() {
                tracing::warn!("JWT expired: exp={}, now={}", exp, chrono::Utc::now().timestamp());
                return Err(AuthError::TokenExpired);
            }
        } else {
            // Tokens without expiry are rejected as a security measure
            tracing::warn!("JWT missing expiry claim");
            return Err(AuthError::InvalidToken);
        }

        // Map Zitadel claims -> Domain types
        Ok(AuthenticatedUser {
            id: UserID::new(introspection.sub),
            email: introspection.email.ok_or(AuthError::InvalidToken)?,
            display_name: introspection.name,
            email_verified: introspection.email_verified.unwrap_or(false),
        })
    }
}
```

---

## HTTP Middleware

Axum middleware extracts and validates tokens:

```rust
// backend/src/adapters/http/middleware/auth.rs

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Json,
};
use std::sync::Arc;
use crate::ports::auth::SessionValidator;
use crate::domain::foundation::AuthenticatedUser;

pub async fn auth_middleware(
    State(validator): State<Arc<dyn SessionValidator>>,
    mut request: Request,
    next: Next,
) -> Response {
    // Extract Bearer token
    let token = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    match token {
        Some(token) => {
            match validator.validate(token).await {
                Ok(user) => {
                    // Inject authenticated user into request extensions
                    request.extensions_mut().insert(user);
                    next.run(request).await
                }
                Err(e) => {
                    let status = match e {
                        AuthError::TokenExpired => StatusCode::UNAUTHORIZED,
                        AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
                        _ => StatusCode::INTERNAL_SERVER_ERROR,
                    };
                    (status, Json(serde_json::json!({
                        "error": e.to_string()
                    }))).into_response()
                }
            }
        }
        None => {
            // No token - let handler decide if auth is required
            next.run(request).await
        }
    }
}

/// Extractor that requires authentication
pub struct RequireAuth(pub AuthenticatedUser);

#[async_trait]
impl<S> axum::extract::FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .map(RequireAuth)
            .ok_or((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Authentication required" })),
            ))
    }
}
```

---

## Frontend Integration

SvelteKit uses Auth.js with Zitadel OIDC:

```typescript
// frontend/src/auth.ts

import { SvelteKitAuth } from "@auth/sveltekit";
import Zitadel from "@auth/sveltekit/providers/zitadel";

export const { handle, signIn, signOut } = SvelteKitAuth({
  providers: [
    Zitadel({
      issuer: process.env.AUTH_ZITADEL_ISSUER,
      clientId: process.env.AUTH_ZITADEL_CLIENT_ID,
      clientSecret: process.env.AUTH_ZITADEL_CLIENT_SECRET,
    }),
  ],
  callbacks: {
    async jwt({ token, account }) {
      // Store access token for API calls
      if (account) {
        token.accessToken = account.access_token;
      }
      return token;
    },
    async session({ session, token }) {
      session.accessToken = token.accessToken;
      return session;
    },
  },
});
```

```typescript
// frontend/src/hooks.server.ts

import { handle as authHandle } from "./auth";

export const handle = authHandle;
```

```svelte
<!-- frontend/src/routes/+layout.svelte -->
<script lang="ts">
  import { signIn, signOut } from "@auth/sveltekit/client";
  import { page } from "$app/stores";
</script>

{#if $page.data.session}
  <span>Welcome, {$page.data.session.user?.name}</span>
  <button on:click={() => signOut()}>Sign out</button>
{:else}
  <button on:click={() => signIn("zitadel")}>Sign in</button>
{/if}
```

### API Client with Auth Token

```typescript
// frontend/src/lib/api/client.ts

import { page } from "$app/stores";
import { get } from "svelte/store";

export async function authFetch(url: string, options: RequestInit = {}) {
  const session = get(page).data.session;

  if (!session?.accessToken) {
    throw new Error("Not authenticated");
  }

  return fetch(url, {
    ...options,
    headers: {
      ...options.headers,
      Authorization: `Bearer ${session.accessToken}`,
    },
  });
}
```

---

## Configuration

Uses **generic OIDC terminology** to maintain provider-independence:

```rust
// backend/src/config/auth.rs

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AuthConfig {
    /// OIDC issuer URL (e.g., https://auth.choicesherpa.com)
    pub issuer_url: String,

    /// Client ID for token validation
    pub client_id: String,

    /// Client secret for introspection
    pub client_secret: String,

    /// Expected audience
    pub audience: String,
}
```

### Environment Variables

```bash
# Backend (.env)
AUTH_ISSUER_URL=https://auth.choicesherpa.com
AUTH_CLIENT_ID=choice-sherpa-backend
AUTH_CLIENT_SECRET=<secret>
AUTH_AUDIENCE=https://api.choicesherpa.com

# Frontend (.env)
AUTH_ZITADEL_ISSUER=https://auth.choicesherpa.com
AUTH_ZITADEL_CLIENT_ID=choice-sherpa-frontend
AUTH_ZITADEL_CLIENT_SECRET=<secret>
AUTH_SECRET=<random-32-bytes-for-cookie-encryption>
```

---

## Testing Strategy

Port abstraction enables testing without Zitadel:

```rust
// backend/src/adapters/auth/mock.rs

use std::collections::HashMap;
use async_trait::async_trait;
use crate::domain::foundation::{AuthenticatedUser, AuthError, UserID};
use crate::ports::auth::SessionValidator;

pub struct MockSessionValidator {
    users: HashMap<String, AuthenticatedUser>,
}

impl MockSessionValidator {
    pub fn new() -> Self {
        Self { users: HashMap::new() }
    }

    pub fn with_user(mut self, token: &str, user: AuthenticatedUser) -> Self {
        self.users.insert(token.to_string(), user);
        self
    }
}

#[async_trait]
impl SessionValidator for MockSessionValidator {
    async fn validate(&self, token: &str) -> Result<AuthenticatedUser, AuthError> {
        self.users
            .get(token)
            .cloned()
            .ok_or(AuthError::InvalidToken)
    }
}

#[tokio::test]
async fn test_protected_endpoint_requires_auth() {
    let validator = MockSessionValidator::new()
        .with_user("valid-token", AuthenticatedUser {
            id: UserID::new("user-123"),
            email: "test@example.com".into(),
            display_name: Some("Test User".into()),
            email_verified: true,
        });

    // Test with mock validator - no Zitadel required
    let app = create_test_app(Arc::new(validator));

    // Unauthenticated request
    let response = app.get("/api/sessions").send().await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Authenticated request
    let response = app
        .get("/api/sessions")
        .header("Authorization", "Bearer valid-token")
        .send()
        .await;
    assert_eq!(response.status(), StatusCode::OK);
}
```

---

## Swappability

Provider can be swapped by implementing `SessionValidator`:

```rust
// Current: Zitadel
let validator: Arc<dyn SessionValidator> = Arc::new(
    ZitadelValidator::new(&config.issuer_url, &config.client_id, &config.client_secret)?
);

// Alternative: Auth0
let validator: Arc<dyn SessionValidator> = Arc::new(
    Auth0Validator::new(&config.issuer_url, &config.audience)?
);

// Alternative: Keycloak
let validator: Arc<dyn SessionValidator> = Arc::new(
    KeycloakValidator::new(&config.issuer_url, &config.realm)?
);

// Domain code unchanged - only adapter swap required
```

---

## Deployment

### Zitadel Self-Hosted

```yaml
# docker-compose.yml (excerpt)
services:
  zitadel:
    image: ghcr.io/zitadel/zitadel:latest
    command: start-from-init --masterkey "YourMasterKey" --tlsMode external
    environment:
      ZITADEL_DATABASE_POSTGRES_HOST: postgres
      ZITADEL_DATABASE_POSTGRES_PORT: 5432
      ZITADEL_DATABASE_POSTGRES_DATABASE: zitadel
      ZITADEL_DATABASE_POSTGRES_USER: zitadel
      ZITADEL_DATABASE_POSTGRES_PASSWORD: ${ZITADEL_DB_PASSWORD}
      ZITADEL_EXTERNALSECURE: true
      ZITADEL_EXTERNALDOMAIN: auth.choicesherpa.com
      ZITADEL_EXTERNALPORT: 443
    depends_on:
      - postgres
    ports:
      - "8080:8080"

  # Zitadel uses the shared PostgreSQL instance
  # with its own database
```

### Email Configuration (via Resend)

Zitadel sends emails for verification and password reset via SMTP:

```yaml
# Zitadel SMTP configuration
ZITADEL_SMTP_HOST: smtp.resend.com
ZITADEL_SMTP_PORT: 465
ZITADEL_SMTP_USER: resend
ZITADEL_SMTP_PASSWORD: ${RESEND_API_KEY}
ZITADEL_SMTP_TLS: true
ZITADEL_SMTP_FROM: noreply@choicesherpa.com
```

---

## Implementation Phases

### Phase 1: Zitadel Setup
- [ ] Deploy Zitadel instance
- [ ] Configure OIDC application for frontend
- [ ] Configure service account for backend
- [ ] Set up email via Resend SMTP

### Phase 2: Backend Integration
- [ ] Implement SessionValidator port
- [ ] Implement ZitadelValidator adapter
- [ ] Create auth middleware
- [ ] Add RequireAuth extractor
- [ ] Write unit tests with mock validator

### Phase 3: Frontend Integration
- [ ] Configure Auth.js with Zitadel
- [ ] Add sign in/out buttons
- [ ] Create authFetch helper
- [ ] Protect routes

### Phase 4: Production Hardening
- [ ] Configure MFA policies
- [ ] Set up audit logging
- [ ] Configure brute force protection (Zitadel built-in)
- [ ] Test federated logout

---

## Security Considerations

### JWT Claim Validation (OWASP A07)

Per APPLICATION-SECURITY-STANDARD.md, all JWT tokens MUST validate the following claims:

| Claim | Validation | Attack Prevented |
|-------|------------|------------------|
| **Issuer (iss)** | Must match expected Zitadel URL | Token confusion attacks |
| **Audience (aud)** | Must contain our application | Cross-service token reuse |
| **Expiry (exp)** | Must be in the future | Replay attacks with expired tokens |

**Why This Matters:**

1. **Token Confusion**: Without issuer validation, an attacker could use tokens from a different OIDC provider
2. **Cross-Service Attacks**: Without audience validation, tokens meant for other services could be accepted
3. **Replay Attacks**: Explicit expiry validation provides defense-in-depth beyond `introspection.active`

**Implementation Requirements:**

```rust
// REQUIRED: All three validations must be present
if introspection.iss.as_deref() != Some(&self.expected_issuer) {
    return Err(AuthError::InvalidToken);
}

if !introspection.aud.as_ref()
    .map(|a| a.contains(&self.expected_audience))
    .unwrap_or(false)
{
    return Err(AuthError::InvalidToken);
}

if let Some(exp) = introspection.exp {
    if exp < chrono::Utc::now().timestamp() {
        return Err(AuthError::TokenExpired);
    }
} else {
    return Err(AuthError::InvalidToken);  // Tokens without expiry are rejected
}
```

### Fail-Secure Behavior

The ZitadelValidator follows fail-secure principles:

- Missing claims result in rejection (not default values)
- Tokens without expiry are rejected (not treated as never-expiring)
- Validation errors log warnings for security monitoring
- No "allow on error" patterns in authentication code

---

## Exit Criteria

1. **Login works**: Users can sign in via Zitadel
2. **Tokens validate**: Backend correctly validates JWTs
3. **Domain clean**: No Zitadel types in domain/application layers
4. **Tests pass**: Unit tests work with mock validator
5. **Swappable**: Clear path to swap auth provider
6. **JWT claims validated**: Issuer, audience, and expiry are all explicitly checked

---

## What We DON'T Build

Per the Zitadel architecture decision, we delegate:

| Feature | Handled By |
|---------|------------|
| User registration | Zitadel |
| Password reset | Zitadel |
| Email verification | Zitadel |
| MFA (TOTP, WebAuthn) | Zitadel |
| Session management | Zitadel + Auth.js |
| Brute force protection | Zitadel |
| Audit logging | Zitadel |
| Admin user management | Zitadel Console |

Our codebase implements:
- `SessionValidator` port (~50 lines)
- `ZitadelValidator` adapter (~90 lines, includes security validations)
- Auth middleware (~60 lines)
- Frontend Auth.js config (~30 lines)

**Total: ~230 lines** instead of building OAuth from scratch.
