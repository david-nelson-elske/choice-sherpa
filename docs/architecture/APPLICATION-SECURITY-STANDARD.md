# Application Security Standard

> **Purpose**: Define security requirements and coding standards to prevent known vulnerabilities.
> **Scope**: All code in the Choice Sherpa application (backend, frontend, infrastructure).
> **Compliance**: All PRs must pass security review before merge.

---

## Table of Contents

1. [Security Principles](#security-principles)
2. [OWASP Top 10 Prevention](#owasp-top-10-prevention)
3. [Backend Security (Rust/Axum)](#backend-security-rustaxum)
4. [Frontend Security (SvelteKit)](#frontend-security-sveltekit)
5. [Authentication & Authorization](#authentication--authorization)
6. [Data Protection](#data-protection)
7. [API Security](#api-security)
8. [Infrastructure Security](#infrastructure-security)
9. [Dependency Management](#dependency-management)
10. [Security Review Checklist](#security-review-checklist)

---

## Security Principles

### Defense in Depth
Multiple layers of security controls. Never rely on a single defense mechanism.

### Least Privilege
Grant minimum permissions necessary. Applies to:
- Database connections (read-only where possible)
- API scopes
- User roles
- Service accounts

### Fail Secure
On error, default to denying access rather than granting it.

```rust
// GOOD: Fail secure
fn check_access(user: &User, resource: &Resource) -> Result<(), AccessDenied> {
    if !user.has_permission(resource) {
        return Err(AccessDenied);  // Explicit denial
    }
    Ok(())
}

// BAD: Fail open
fn check_access(user: &User, resource: &Resource) -> bool {
    user.has_permission(resource).unwrap_or(true)  // Defaults to allowing!
}
```

### Input Validation at Boundaries
Validate all external input at system boundaries. Internal code trusts validated types.

### Secure by Default
Security features enabled by default. Insecure options require explicit opt-in.

---

## OWASP Top 10 Prevention

### A01: Broken Access Control

**Risk**: Users accessing resources they shouldn't.

**Prevention**:

1. **Enforce ownership checks on all resource access**:
```rust
// REQUIRED: Always verify ownership
pub async fn get_session(
    user_id: UserId,
    session_id: SessionId,
    repo: &dyn SessionRepository,
) -> Result<Session, Error> {
    let session = repo.find(session_id).await?;

    // CRITICAL: Verify ownership
    if session.owner_id != user_id {
        return Err(Error::AccessDenied);
    }

    Ok(session)
}
```

2. **Deny by default**:
```rust
// Access checker must explicitly grant, not check for denial
pub trait AccessChecker {
    async fn can_access(&self, user: &UserId, resource: &ResourceId) -> bool;
}
```

3. **IDOR (Insecure Direct Object Reference) prevention**:
```rust
// BAD: Sequential IDs expose enumeration
struct SessionId(i64);  // Can guess other IDs

// GOOD: UUIDs prevent enumeration
struct SessionId(Uuid);  // Cannot guess other IDs
```

4. **Rate limiting on resource access**:
```rust
// Limit requests to prevent enumeration attacks
#[middleware(rate_limit(requests = 100, window = "1m"))]
async fn get_resource(/* ... */) { }
```

### A02: Cryptographic Failures

**Risk**: Sensitive data exposed through weak or missing encryption.

**Prevention**:

1. **Use strong algorithms only**:
```rust
// ALLOWED algorithms:
// - Hashing: Argon2id, bcrypt (cost >= 12), SHA-256/384/512
// - Encryption: AES-256-GCM, ChaCha20-Poly1305
// - Signing: Ed25519, ECDSA P-256/P-384

// BANNED algorithms:
// - MD5, SHA-1 (for security purposes)
// - DES, 3DES, RC4
// - RSA < 2048 bits
```

2. **Password storage**:
```rust
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

// REQUIRED: Use Argon2id for passwords
pub fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();  // Argon2id with secure defaults
    Ok(argon2.hash_password(password.as_bytes(), &salt)?.to_string())
}
```

3. **Secrets management**:
```rust
// NEVER hardcode secrets
const API_KEY: &str = "sk-1234...";  // BANNED

// REQUIRED: Load from environment
let api_key = std::env::var("API_KEY")
    .expect("API_KEY must be set");
```

4. **TLS enforcement**:
```rust
// REQUIRED: All external connections use TLS
// - Database: sslmode=require
// - Redis: TLS enabled
// - HTTP clients: HTTPS only
```

### A03: Injection

**Risk**: Untrusted data interpreted as code/commands.

**Prevention**:

1. **SQL Injection - Use parameterized queries**:
```rust
// GOOD: Parameterized query (sqlx)
sqlx::query_as!(
    Session,
    "SELECT * FROM sessions WHERE id = $1 AND owner_id = $2",
    session_id,
    owner_id
)
.fetch_one(&pool)
.await?;

// BAD: String interpolation
let query = format!("SELECT * FROM sessions WHERE id = '{}'", session_id);
sqlx::query(&query).fetch_one(&pool).await?;  // VULNERABLE
```

2. **Command Injection - Avoid shell execution**:
```rust
// BAD: Shell command with user input
std::process::Command::new("sh")
    .arg("-c")
    .arg(format!("convert {} output.png", user_filename))  // VULNERABLE
    .output()?;

// GOOD: Direct execution without shell
std::process::Command::new("convert")
    .arg(&validated_filename)  // Validated path
    .arg("output.png")
    .output()?;
```

3. **XSS Prevention** (see Frontend Security section)

4. **LDAP/XML/Other Injection**:
```rust
// Use typed builders, never string concatenation
// Validate and sanitize all inputs at boundaries
```

### A04: Insecure Design

**Risk**: Architectural flaws that can't be fixed by implementation.

**Prevention**:

1. **Threat modeling during design**:
   - Identify trust boundaries
   - Document data flows
   - Consider abuse cases

2. **Security requirements in feature specs**:
```markdown
## Security Requirements
- [ ] Authentication required: Yes/No
- [ ] Authorization model: [describe]
- [ ] Sensitive data handled: [list]
- [ ] Rate limiting required: Yes/No
```

3. **Secure defaults**:
```rust
// GOOD: Secure by default
pub struct SessionConfig {
    pub require_auth: bool,     // Default: true
    pub max_lifetime: Duration, // Default: 24 hours
    pub audit_logging: bool,    // Default: true
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            require_auth: true,      // Secure default
            max_lifetime: Duration::hours(24),
            audit_logging: true,
        }
    }
}
```

### A05: Security Misconfiguration

**Risk**: Insecure default configurations or missing hardening.

**Prevention**:

1. **Environment-specific configs**:
```rust
// REQUIRED: Different configs per environment
pub struct Config {
    pub debug_mode: bool,        // false in production
    pub detailed_errors: bool,   // false in production
    pub cors_origins: Vec<String>, // Restricted in production
}
```

2. **HTTP security headers** (see API Security section)

3. **Disable unnecessary features**:
```rust
// Remove debug endpoints in production
#[cfg(debug_assertions)]
fn debug_routes() -> Router { /* ... */ }
```

4. **Error messages**:
```rust
// BAD: Leaks internal details
Err(format!("Database error: {}", sql_error))

// GOOD: Generic message, log details
error!("Database error: {}", sql_error);
Err(Error::InternalError)  // Generic response
```

### A06: Vulnerable and Outdated Components

**Risk**: Using components with known vulnerabilities.

**Prevention**:

1. **Automated dependency scanning**:
```bash
# REQUIRED: Run before each PR
cargo audit
npm audit
```

2. **Dependency pinning**:
```toml
# Cargo.toml - Pin major versions
[dependencies]
axum = "0.7"
sqlx = "0.8"
```

3. **Regular updates**:
   - Review security advisories weekly
   - Update dependencies monthly (minimum)
   - Emergency patches within 24 hours for critical CVEs

### A07: Identification and Authentication Failures

**Risk**: Authentication bypass or weak authentication.

**Prevention**:

1. **Use established auth provider** (Zitadel):
```rust
// REQUIRED: Validate JWT on every authenticated request
pub async fn validate_token(token: &str) -> Result<Claims, AuthError> {
    let validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&["https://auth.example.com"]);
    validation.set_audience(&["choice-sherpa"]);

    decode::<Claims>(token, &decoding_key, &validation)?
}
```

2. **Session management**:
```rust
// REQUIRED session properties
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,  // REQUIRED: Expiration
    pub ip_address: IpAddr,         // For anomaly detection
}
```

3. **Multi-factor authentication**: Support MFA via Zitadel

4. **Account lockout**:
```rust
// Lock after N failed attempts
const MAX_FAILED_ATTEMPTS: u32 = 5;
const LOCKOUT_DURATION: Duration = Duration::minutes(15);
```

### A08: Software and Data Integrity Failures

**Risk**: Code or data modified without verification.

**Prevention**:

1. **CI/CD pipeline security**:
   - Signed commits required
   - Protected branches
   - Code review required

2. **Dependency integrity**:
```toml
# Cargo.toml - Use Cargo.lock
# REQUIRED: Commit Cargo.lock to repository
```

3. **Verify external data**:
```rust
// Validate webhooks with signatures
pub fn verify_webhook(payload: &[u8], signature: &str, secret: &str) -> bool {
    let expected = hmac_sha256(secret, payload);
    constant_time_eq(signature.as_bytes(), &expected)
}
```

### A09: Security Logging and Monitoring Failures

**Risk**: Attacks go undetected due to insufficient logging.

**Prevention**:

1. **Log security events**:
```rust
// REQUIRED: Log these events
// - Authentication success/failure
// - Authorization failures
// - Input validation failures
// - Rate limit triggers
// - Admin actions

#[instrument(skip(password))]
pub async fn login(username: &str, password: &str) -> Result<Token, AuthError> {
    match authenticate(username, password).await {
        Ok(token) => {
            info!(username = %username, "Login successful");
            Ok(token)
        }
        Err(e) => {
            warn!(username = %username, error = %e, "Login failed");
            Err(e)
        }
    }
}
```

2. **Structured logging**:
```rust
// REQUIRED: Use structured logging with tracing
use tracing::{info, warn, error, instrument};

#[instrument(fields(user_id = %user_id, resource = %resource_id))]
pub async fn access_resource(user_id: UserId, resource_id: ResourceId) {
    // ...
}
```

3. **Never log sensitive data**:
```rust
// BAD
info!("User login: {} with password {}", username, password);

// GOOD
info!("User login: {}", username);  // No password
```

### A10: Server-Side Request Forgery (SSRF)

**Risk**: Server makes requests to unintended locations.

**Prevention**:

1. **Validate and sanitize URLs**:
```rust
// REQUIRED: Allowlist for external requests
const ALLOWED_HOSTS: &[&str] = &[
    "api.openai.com",
    "api.anthropic.com",
];

pub fn validate_url(url: &Url) -> Result<(), Error> {
    let host = url.host_str().ok_or(Error::InvalidUrl)?;
    if !ALLOWED_HOSTS.contains(&host) {
        return Err(Error::HostNotAllowed);
    }
    Ok(())
}
```

2. **Block internal network access**:
```rust
// REQUIRED: Block requests to internal IPs
fn is_internal_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_private() || ip.is_loopback() || ip.is_link_local()
        }
        IpAddr::V6(ip) => {
            ip.is_loopback() || /* other checks */
        }
    }
}
```

---

## Backend Security (Rust/Axum)

### Type-Safe Security

Leverage Rust's type system for security:

```rust
// Use newtypes to prevent parameter confusion
pub struct UserId(Uuid);
pub struct SessionId(Uuid);

// Compiler prevents mixing these up
fn get_session(user_id: UserId, session_id: SessionId) { /* ... */ }

// Use validated types
pub struct Email(String);  // Only valid emails can be constructed

impl Email {
    pub fn new(value: &str) -> Result<Self, ValidationError> {
        if is_valid_email(value) {
            Ok(Self(value.to_string()))
        } else {
            Err(ValidationError::InvalidEmail)
        }
    }
}
```

### Memory Safety

Rust provides memory safety, but be careful with:

```rust
// CAREFUL: unsafe blocks
unsafe {
    // Document why this is safe
    // Minimize unsafe scope
}

// CAREFUL: FFI
extern "C" {
    fn external_function(data: *const u8, len: usize);
}

// REQUIRED: Validate before FFI calls
pub fn safe_wrapper(data: &[u8]) {
    // Validate data first
    unsafe {
        external_function(data.as_ptr(), data.len());
    }
}
```

### Error Handling

```rust
// REQUIRED: Use Result for fallible operations
// BANNED: panic! in library code (except truly unrecoverable)

// GOOD: Propagate errors
pub fn process_request(req: Request) -> Result<Response, Error> {
    let data = validate_input(&req)?;
    let result = do_work(data)?;
    Ok(result)
}

// BAD: Unwrap in production code
pub fn process_request(req: Request) -> Response {
    let data = validate_input(&req).unwrap();  // Can panic!
    // ...
}
```

### Axum-Specific Security

```rust
use axum::{
    extract::{Path, Query, Json},
    http::StatusCode,
};

// REQUIRED: Use extractors for type-safe parsing
async fn handler(
    Path(id): Path<Uuid>,           // Validated UUID
    Query(params): Query<Params>,   // Validated query params
    Json(body): Json<CreateRequest>, // Validated JSON body
) -> Result<Json<Response>, StatusCode> {
    // id, params, body are already validated
}

// REQUIRED: Limit request body size
let app = Router::new()
    .route("/api/upload", post(upload_handler))
    .layer(DefaultBodyLimit::max(10 * 1024 * 1024)); // 10MB max

// REQUIRED: Add timeout layer
let app = app.layer(TimeoutLayer::new(Duration::from_secs(30)));
```

---

## Frontend Security (SvelteKit)

### XSS Prevention

```svelte
<!-- GOOD: Svelte auto-escapes by default -->
<p>{userInput}</p>

<!-- DANGEROUS: Raw HTML - avoid if possible -->
{@html userContent}  <!-- Only use with sanitized content -->

<!-- If @html is needed, sanitize first -->
<script>
    import DOMPurify from 'dompurify';
    $: sanitizedContent = DOMPurify.sanitize(userContent);
</script>
{@html sanitizedContent}
```

### Content Security Policy

```typescript
// svelte.config.js or hooks.server.ts
const csp = {
    'default-src': ["'self'"],
    'script-src': ["'self'"],  // No 'unsafe-inline' or 'unsafe-eval'
    'style-src': ["'self'", "'unsafe-inline'"],  // Required for Svelte
    'img-src': ["'self'", 'data:', 'https:'],
    'connect-src': ["'self'", 'https://api.example.com'],
    'frame-ancestors': ["'none'"],
    'form-action': ["'self'"],
};
```

### Secure Data Handling

```typescript
// NEVER store sensitive data in localStorage
localStorage.setItem('token', jwt);  // BAD

// Use httpOnly cookies for tokens (handled server-side)
// For client state, use secure session storage with encryption

// REQUIRED: Validate all external data
import { z } from 'zod';

const UserSchema = z.object({
    id: z.string().uuid(),
    email: z.string().email(),
    name: z.string().min(1).max(100),
});

async function fetchUser(id: string) {
    const response = await fetch(`/api/users/${id}`);
    const data = await response.json();
    return UserSchema.parse(data);  // Validates response
}
```

### CSRF Protection

```typescript
// SvelteKit provides CSRF protection for form actions
// REQUIRED: Use form actions for mutations

// +page.server.ts
export const actions = {
    default: async ({ request, cookies }) => {
        // SvelteKit validates CSRF token automatically
        const formData = await request.formData();
        // ...
    }
};
```

---

## Authentication & Authorization

### Authentication Flow

```
User → Frontend → Auth Provider (Zitadel) → JWT → Backend

1. User initiates login via OIDC flow
2. Zitadel handles credentials
3. JWT issued with claims
4. Backend validates JWT on each request
5. Claims used for authorization decisions
```

### JWT Validation

```rust
// REQUIRED: Validate all claims
pub struct Claims {
    pub sub: String,      // Subject (user ID)
    pub iss: String,      // Issuer (must match Zitadel)
    pub aud: Vec<String>, // Audience (must include our app)
    pub exp: i64,         // Expiration (must be in future)
    pub iat: i64,         // Issued at
    pub roles: Vec<String>,
}

pub fn validate_jwt(token: &str) -> Result<Claims, AuthError> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[ZITADEL_ISSUER]);
    validation.set_audience(&[APP_AUDIENCE]);

    let token_data = decode::<Claims>(token, &DECODING_KEY, &validation)?;

    // Additional validation
    if token_data.claims.exp < Utc::now().timestamp() {
        return Err(AuthError::TokenExpired);
    }

    Ok(token_data.claims)
}
```

### Authorization Patterns

```rust
// REQUIRED: Use declarative authorization
#[derive(Debug, Clone)]
pub enum Permission {
    SessionRead,
    SessionWrite,
    SessionDelete,
    AdminAccess,
}

pub trait Authorizer {
    fn has_permission(&self, user: &Claims, permission: Permission) -> bool;
    fn can_access_resource(&self, user: &Claims, resource: &dyn Resource) -> bool;
}

// REQUIRED: Check authorization in handlers
async fn delete_session(
    claims: Claims,
    Path(session_id): Path<SessionId>,
    State(state): State<AppState>,
) -> Result<(), Error> {
    let session = state.repo.find(session_id).await?;

    // Check ownership
    if session.owner_id != claims.sub {
        return Err(Error::Forbidden);
    }

    state.repo.delete(session_id).await
}
```

---

## Data Protection

### Sensitive Data Classification

| Classification | Examples | Requirements |
|----------------|----------|--------------|
| **Public** | Marketing content | None |
| **Internal** | User preferences | Authentication required |
| **Confidential** | User data, decisions | Auth + encryption at rest |
| **Restricted** | Passwords, tokens | Never logged, encrypted |

### Encryption at Rest

```rust
// Database: Use PostgreSQL encryption
// - Column-level encryption for PII
// - TDE (Transparent Data Encryption) for disk

// Application-level for specific fields
pub struct EncryptedField {
    ciphertext: Vec<u8>,
    nonce: [u8; 12],
}

impl EncryptedField {
    pub fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Self {
        let cipher = ChaCha20Poly1305::new(key.into());
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = cipher.encrypt(&nonce, plaintext).unwrap();
        Self { ciphertext, nonce: nonce.into() }
    }
}
```

### Data Retention

```rust
// REQUIRED: Define retention policies
pub struct RetentionPolicy {
    pub session_data: Duration,      // 1 year after last activity
    pub audit_logs: Duration,        // 7 years (legal requirement)
    pub deleted_user_data: Duration, // 30 days, then permanent delete
}

// REQUIRED: Implement data deletion
pub async fn delete_user_data(user_id: UserId, repo: &dyn Repository) {
    // Soft delete first
    repo.soft_delete_user(user_id).await?;

    // Schedule permanent deletion
    scheduler.schedule_deletion(user_id, Duration::days(30)).await?;
}
```

---

## API Security

### HTTP Security Headers

```rust
// REQUIRED: Add security headers middleware
pub fn security_headers() -> SetResponseHeaderLayer {
    // Applied to all responses
    headers![
        (STRICT_TRANSPORT_SECURITY, "max-age=31536000; includeSubDomains"),
        (X_CONTENT_TYPE_OPTIONS, "nosniff"),
        (X_FRAME_OPTIONS, "DENY"),
        (X_XSS_PROTECTION, "0"),  // Disabled - rely on CSP
        (CONTENT_SECURITY_POLICY, "default-src 'self'"),
        (REFERRER_POLICY, "strict-origin-when-cross-origin"),
        (PERMISSIONS_POLICY, "geolocation=(), microphone=(), camera=()"),
    ]
}
```

### Rate Limiting

```rust
// REQUIRED: Rate limit all endpoints
use governor::{Quota, RateLimiter};

pub struct RateLimitConfig {
    // Per-IP limits
    pub anonymous: Quota,      // 100 req/min
    pub authenticated: Quota,  // 1000 req/min

    // Per-endpoint limits
    pub login: Quota,          // 5 req/min (prevent brute force)
    pub password_reset: Quota, // 3 req/hour
}

// Apply different limits based on sensitivity
#[rate_limit(quota = "login")]
async fn login_handler(/* ... */) { }

#[rate_limit(quota = "authenticated")]
async fn api_handler(/* ... */) { }
```

### Input Validation

```rust
// REQUIRED: Validate all input at API boundary
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CreateSessionRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: String,

    #[validate(length(max = 2000))]
    pub description: Option<String>,

    #[validate(range(min = 1, max = 100))]
    pub priority: u8,
}

async fn create_session(
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<Session>, Error> {
    req.validate()?;  // Validate before processing
    // ...
}
```

### CORS Configuration

```rust
// REQUIRED: Restrict CORS in production
let cors = CorsLayer::new()
    .allow_origin([
        "https://app.choicesherpa.com".parse().unwrap(),
    ])
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([AUTHORIZATION, CONTENT_TYPE])
    .allow_credentials(true);

// Development can be more permissive
#[cfg(debug_assertions)]
let cors = CorsLayer::permissive();
```

---

## Infrastructure Security

### Database Security

```sql
-- REQUIRED: Least privilege database users
CREATE USER app_readonly WITH PASSWORD '...';
GRANT SELECT ON ALL TABLES IN SCHEMA public TO app_readonly;

CREATE USER app_write WITH PASSWORD '...';
GRANT SELECT, INSERT, UPDATE ON ALL TABLES IN SCHEMA public TO app_write;
-- No DELETE or TRUNCATE - use soft deletes

-- REQUIRED: Enable row-level security
ALTER TABLE sessions ENABLE ROW LEVEL SECURITY;

CREATE POLICY session_owner_policy ON sessions
    USING (owner_id = current_setting('app.current_user_id')::uuid);
```

### Secrets Management

```yaml
# REQUIRED: Use environment variables or secret manager
# NEVER commit secrets to repository

# docker-compose.yml (development only)
services:
  app:
    environment:
      - DATABASE_URL=${DATABASE_URL}  # From .env (gitignored)

# Production: Use secret manager (AWS Secrets Manager, Vault, etc.)
```

### Container Security

```dockerfile
# REQUIRED: Non-root user
FROM rust:1.75-slim as runtime
RUN useradd -m -u 1001 appuser
USER appuser

# REQUIRED: Minimal base image
FROM gcr.io/distroless/cc-debian12

# REQUIRED: No sensitive data in image
# Use runtime secrets injection
```

---

## Dependency Management

### Rust Dependencies

```bash
# REQUIRED: Run before each PR
cargo audit

# REQUIRED: Check for outdated dependencies monthly
cargo outdated

# REQUIRED: Review new dependencies
# - Check crate popularity and maintenance
# - Review source code for red flags
# - Prefer well-known, audited crates
```

### Approved Security Crates

| Purpose | Approved Crate |
|---------|---------------|
| Password hashing | `argon2` |
| JWT | `jsonwebtoken` |
| Encryption | `chacha20poly1305`, `aes-gcm` |
| Random | `rand`, `getrandom` |
| TLS | `rustls` |
| HTTP client | `reqwest` (with `rustls-tls`) |

### Frontend Dependencies

```bash
# REQUIRED: Run before each PR
npm audit

# REQUIRED: Use lockfile
npm ci  # Not npm install

# REQUIRED: Review new packages
# - Check npm audit advisory
# - Review GitHub issues
# - Check download trends
```

---

## Security Review Checklist

Use this checklist for every PR:

### Input Handling
- [ ] All user input validated at boundaries
- [ ] No string interpolation in SQL queries
- [ ] No shell command construction from user input
- [ ] URLs validated against allowlist for external requests

### Authentication & Authorization
- [ ] All endpoints require authentication (unless public)
- [ ] Authorization checked for every resource access
- [ ] Ownership verified for resource modifications
- [ ] No privilege escalation paths

### Data Protection
- [ ] Sensitive data not logged
- [ ] Secrets loaded from environment, not hardcoded
- [ ] PII encrypted at rest if applicable
- [ ] Data retention policy followed

### Error Handling
- [ ] No sensitive details in error responses
- [ ] Errors logged with appropriate level
- [ ] Fail secure (deny on error)

### Dependencies
- [ ] `cargo audit` passes
- [ ] `npm audit` passes (if frontend changes)
- [ ] New dependencies reviewed

### API Security
- [ ] Rate limiting configured
- [ ] Security headers present
- [ ] CORS properly configured
- [ ] Request size limits set

### Code Quality
- [ ] No `unwrap()` in production paths
- [ ] No `unsafe` without justification
- [ ] No TODO/FIXME for security items

---

## Security Contacts

**Security Issues**: Report to security@choicesherpa.com

**Responsible Disclosure**: We follow responsible disclosure. Please allow 90 days before public disclosure.

---

*Document Version: 1.0.0*
*Created: 2026-01-08*
*Review Cycle: Quarterly*
