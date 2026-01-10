# Infrastructure: Test Harness

**Type:** Cross-Cutting Infrastructure
**Priority:** P1 (Required for TDD)
**Last Updated:** 2026-01-09

> Complete specification for test infrastructure including database setup, fixtures, and integration test patterns.

---

## Overview

Choice Sherpa follows TDD with a comprehensive test harness. This specification defines:
1. Test database setup and isolation
2. Test fixtures and factories
3. Integration test patterns
4. Mock implementations for external services
5. Test utility helpers

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Test Infrastructure                                │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                        Test Categories                               │   │
│   │                                                                      │   │
│   │   Unit Tests          Integration Tests       E2E Tests             │   │
│   │   ──────────          ─────────────────       ─────────             │   │
│   │   - No I/O            - Real database         - Full stack          │   │
│   │   - Pure domain       - Real Redis            - Docker compose      │   │
│   │   - Fast (<1ms)       - Mocked externals      - HTTP client         │   │
│   │   - #[test]           - #[sqlx::test]         - testcontainers      │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                         Test Database                                │   │
│   │                                                                      │   │
│   │   PostgreSQL (test instance)                                        │   │
│   │   ├── Migrations applied automatically                              │   │
│   │   ├── Each test gets isolated transaction                           │   │
│   │   └── Rolled back after each test                                   │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                         Mock Services                                │   │
│   │                                                                      │   │
│   │   MockAiProvider      MockPaymentProvider    MockEmailProvider      │   │
│   │   ──────────────      ───────────────────    ─────────────────      │   │
│   │   - Canned responses  - Test mode Stripe     - In-memory capture    │   │
│   │   - Configurable      - Deterministic        - Assertion helpers    │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Test Database Setup

### Configuration

```rust
// tests/common/database.rs

use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::OnceLock;

static TEST_DB_URL: OnceLock<String> = OnceLock::new();

pub fn get_test_database_url() -> &'static str {
    TEST_DB_URL.get_or_init(|| {
        std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://test:test@localhost:5432/choice_sherpa_test".to_string())
    })
}

/// Create a connection pool for integration tests
pub async fn create_test_pool() -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(get_test_database_url())
        .await
        .expect("Failed to connect to test database")
}

/// Setup test database with migrations
pub async fn setup_test_database() -> PgPool {
    let pool = create_test_pool().await;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}
```

### Transactional Tests

```rust
// tests/common/transaction.rs

use sqlx::{PgPool, Postgres, Transaction};

/// Wrapper for transactional test isolation
pub struct TestTransaction<'a> {
    tx: Transaction<'a, Postgres>,
}

impl<'a> TestTransaction<'a> {
    pub async fn begin(pool: &'a PgPool) -> Self {
        let tx = pool.begin().await.expect("Failed to begin transaction");
        Self { tx }
    }

    pub fn transaction(&mut self) -> &mut Transaction<'a, Postgres> {
        &mut self.tx
    }

    /// Rollback transaction (called automatically on drop)
    pub async fn rollback(self) {
        self.tx.rollback().await.expect("Failed to rollback");
    }
}

/// Run a test function within a transaction that rolls back
pub async fn with_test_transaction<F, Fut, T>(pool: &PgPool, test: F) -> T
where
    F: FnOnce(Transaction<'_, Postgres>) -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let tx = pool.begin().await.expect("Failed to begin transaction");
    let result = test(tx).await;
    // Transaction is dropped here and rolls back
    result
}
```

### Using sqlx::test Macro

```rust
// tests/integration/session_tests.rs

use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn test_create_session(pool: PgPool) {
    // Pool is automatically connected to a fresh test database
    // Migrations are applied
    // Everything is cleaned up after the test

    let session = create_session(&pool, CreateSessionCommand {
        user_id: UserId::new(),
        title: "Test Decision".to_string(),
    }).await.expect("Should create session");

    assert_eq!(session.title, "Test Decision");
}

#[sqlx::test(migrations = "./migrations", fixtures("users", "sessions"))]
async fn test_list_user_sessions(pool: PgPool) {
    // Fixtures are loaded from tests/fixtures/
    let sessions = list_sessions(&pool, &user_id()).await.unwrap();
    assert_eq!(sessions.len(), 3);
}
```

---

## Test Fixtures

### Fixture Files

```sql
-- tests/fixtures/users.sql
INSERT INTO users (id, email, created_at)
VALUES
    ('11111111-1111-1111-1111-111111111111', 'alice@test.com', NOW()),
    ('22222222-2222-2222-2222-222222222222', 'bob@test.com', NOW());
```

```sql
-- tests/fixtures/sessions.sql
INSERT INTO sessions (id, user_id, title, status, created_at)
VALUES
    ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', '11111111-1111-1111-1111-111111111111', 'Career Decision', 'active', NOW()),
    ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', '11111111-1111-1111-1111-111111111111', 'Investment Choice', 'active', NOW()),
    ('cccccccc-cccc-cccc-cccc-cccccccccccc', '11111111-1111-1111-1111-111111111111', 'Old Decision', 'archived', NOW());
```

```sql
-- tests/fixtures/memberships.sql
INSERT INTO memberships (id, user_id, tier, status, created_at)
VALUES
    ('dddddddd-dddd-dddd-dddd-dddddddddddd', '11111111-1111-1111-1111-111111111111', 'monthly', 'active', NOW()),
    ('eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', '22222222-2222-2222-2222-222222222222', 'free', 'active', NOW());
```

### Factory Pattern

```rust
// tests/common/factories.rs

use crate::domain::*;
use uuid::Uuid;

/// Factory for creating test entities
pub struct TestFactory;

impl TestFactory {
    pub fn user_id() -> UserId {
        UserId::from(Uuid::new_v4())
    }

    pub fn session_id() -> SessionId {
        SessionId::from(Uuid::new_v4())
    }

    pub fn session() -> Session {
        Session::new(Self::user_id(), "Test Session".to_string())
    }

    pub fn session_with_user(user_id: UserId) -> Session {
        Session::new(user_id, "Test Session".to_string())
    }

    pub fn membership() -> Membership {
        Membership::create_free(Self::user_id(), None)
    }

    pub fn membership_with_tier(tier: Tier) -> Membership {
        let user_id = Self::user_id();
        match tier {
            Tier::Free => Membership::create_free(user_id, None),
            Tier::Monthly | Tier::Annual => {
                Membership::create_paid(user_id, tier, StripeCustomerId::new("cus_test"))
            }
        }
    }

    pub fn cycle(session_id: SessionId) -> Cycle {
        Cycle::new(session_id)
    }

    pub fn component(cycle_id: CycleId, component_type: ComponentType) -> Component {
        Component::new(cycle_id, component_type)
    }
}

/// Builder pattern for complex entities
pub struct SessionBuilder {
    user_id: UserId,
    title: String,
    status: SessionStatus,
}

impl SessionBuilder {
    pub fn new() -> Self {
        Self {
            user_id: TestFactory::user_id(),
            title: "Test Session".to_string(),
            status: SessionStatus::Active,
        }
    }

    pub fn with_user(mut self, user_id: UserId) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn archived(mut self) -> Self {
        self.status = SessionStatus::Archived;
        self
    }

    pub fn build(self) -> Session {
        let mut session = Session::new(self.user_id, self.title);
        if self.status == SessionStatus::Archived {
            session.archive();
        }
        session
    }
}

impl Default for SessionBuilder {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Mock Services

### Mock AI Provider

```rust
// tests/common/mocks/ai_provider.rs

use std::sync::{Arc, Mutex};
use async_trait::async_trait;

pub struct MockAiProvider {
    responses: Arc<Mutex<Vec<AiResponse>>>,
    calls: Arc<Mutex<Vec<AiRequest>>>,
}

impl MockAiProvider {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(vec![])),
            calls: Arc::new(Mutex::new(vec![])),
        }
    }

    /// Queue a response to be returned
    pub fn with_response(self, response: AiResponse) -> Self {
        self.responses.lock().unwrap().push(response);
        self
    }

    /// Queue multiple responses
    pub fn with_responses(self, responses: Vec<AiResponse>) -> Self {
        self.responses.lock().unwrap().extend(responses);
        self
    }

    /// Get all recorded calls
    pub fn calls(&self) -> Vec<AiRequest> {
        self.calls.lock().unwrap().clone()
    }

    /// Assert a call was made with specific content
    pub fn assert_called_with(&self, expected: &str) {
        let calls = self.calls.lock().unwrap();
        assert!(
            calls.iter().any(|c| c.content.contains(expected)),
            "Expected call containing '{}', got: {:?}",
            expected,
            calls
        );
    }
}

#[async_trait]
impl AiProvider for MockAiProvider {
    async fn generate(&self, request: AiRequest) -> Result<AiResponse, AiError> {
        // Record the call
        self.calls.lock().unwrap().push(request);

        // Return next queued response
        let mut responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            Ok(AiResponse::default())
        } else {
            Ok(responses.remove(0))
        }
    }

    async fn stream_generate(
        &self,
        request: AiRequest,
    ) -> Result<impl Stream<Item = AiChunk>, AiError> {
        self.calls.lock().unwrap().push(request);

        let responses = self.responses.lock().unwrap();
        let chunks = if responses.is_empty() {
            vec![AiChunk::text("Mock response")]
        } else {
            responses[0].as_chunks()
        };

        Ok(futures::stream::iter(chunks))
    }
}
```

### Mock Payment Provider

```rust
// tests/common/mocks/payment_provider.rs

use std::sync::{Arc, Mutex};

pub struct MockPaymentProvider {
    customers: Arc<Mutex<HashMap<String, MockCustomer>>>,
    subscriptions: Arc<Mutex<HashMap<String, MockSubscription>>>,
    should_fail: Arc<Mutex<bool>>,
}

impl MockPaymentProvider {
    pub fn new() -> Self {
        Self {
            customers: Arc::new(Mutex::new(HashMap::new())),
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            should_fail: Arc::new(Mutex::new(false)),
        }
    }

    /// Make all operations fail
    pub fn set_should_fail(&self, should_fail: bool) {
        *self.should_fail.lock().unwrap() = should_fail;
    }

    /// Get created customers
    pub fn customers(&self) -> Vec<MockCustomer> {
        self.customers.lock().unwrap().values().cloned().collect()
    }

    /// Get created subscriptions
    pub fn subscriptions(&self) -> Vec<MockSubscription> {
        self.subscriptions.lock().unwrap().values().cloned().collect()
    }
}

#[async_trait]
impl PaymentProvider for MockPaymentProvider {
    async fn create_customer(&self, email: &str) -> Result<StripeCustomerId, PaymentError> {
        if *self.should_fail.lock().unwrap() {
            return Err(PaymentError::ApiError("Mock failure".into()));
        }

        let id = format!("cus_mock_{}", Uuid::new_v4().to_string()[..8].to_string());
        let customer = MockCustomer {
            id: id.clone(),
            email: email.to_string(),
        };

        self.customers.lock().unwrap().insert(id.clone(), customer);
        Ok(StripeCustomerId::new(id))
    }

    async fn create_subscription(
        &self,
        customer_id: &StripeCustomerId,
        price_id: &str,
    ) -> Result<StripeSubscriptionId, PaymentError> {
        if *self.should_fail.lock().unwrap() {
            return Err(PaymentError::ApiError("Mock failure".into()));
        }

        let id = format!("sub_mock_{}", Uuid::new_v4().to_string()[..8].to_string());
        let subscription = MockSubscription {
            id: id.clone(),
            customer_id: customer_id.as_str().to_string(),
            price_id: price_id.to_string(),
            status: "active".to_string(),
        };

        self.subscriptions.lock().unwrap().insert(id.clone(), subscription);
        Ok(StripeSubscriptionId::new(id))
    }
}
```

### Mock Access Checker

```rust
// tests/common/mocks/access_checker.rs

pub struct MockAccessChecker {
    can_create: Arc<Mutex<bool>>,
    can_access: Arc<Mutex<bool>>,
}

impl MockAccessChecker {
    pub fn allowing_all() -> Self {
        Self {
            can_create: Arc::new(Mutex::new(true)),
            can_access: Arc::new(Mutex::new(true)),
        }
    }

    pub fn denying_creation() -> Self {
        Self {
            can_create: Arc::new(Mutex::new(false)),
            can_access: Arc::new(Mutex::new(true)),
        }
    }

    pub fn set_can_create(&self, value: bool) {
        *self.can_create.lock().unwrap() = value;
    }
}

#[async_trait]
impl AccessChecker for MockAccessChecker {
    async fn can_create_session(&self, _user_id: &UserId) -> Result<bool, AccessError> {
        Ok(*self.can_create.lock().unwrap())
    }

    async fn can_access_session(
        &self,
        _user_id: &UserId,
        _session_owner: &UserId,
    ) -> Result<bool, AccessError> {
        Ok(*self.can_access.lock().unwrap())
    }
}
```

---

## HTTP Testing

### Test Client

```rust
// tests/common/http.rs

use axum::Router;
use axum_test::TestServer;

pub struct TestClient {
    server: TestServer,
    auth_token: Option<String>,
}

impl TestClient {
    pub async fn new(app: Router) -> Self {
        let server = TestServer::new(app).unwrap();
        Self {
            server,
            auth_token: None,
        }
    }

    pub fn with_auth(mut self, user_id: &UserId) -> Self {
        self.auth_token = Some(create_test_token(user_id));
        self
    }

    pub async fn get(&self, path: &str) -> TestResponse {
        let mut req = self.server.get(path);
        if let Some(token) = &self.auth_token {
            req = req.add_header("Authorization", &format!("Bearer {}", token));
        }
        req.await
    }

    pub async fn post<T: Serialize>(&self, path: &str, body: &T) -> TestResponse {
        let mut req = self.server.post(path).json(body);
        if let Some(token) = &self.auth_token {
            req = req.add_header("Authorization", &format!("Bearer {}", token));
        }
        req.await
    }

    pub async fn put<T: Serialize>(&self, path: &str, body: &T) -> TestResponse {
        let mut req = self.server.put(path).json(body);
        if let Some(token) = &self.auth_token {
            req = req.add_header("Authorization", &format!("Bearer {}", token));
        }
        req.await
    }

    pub async fn delete(&self, path: &str) -> TestResponse {
        let mut req = self.server.delete(path);
        if let Some(token) = &self.auth_token {
            req = req.add_header("Authorization", &format!("Bearer {}", token));
        }
        req.await
    }
}

fn create_test_token(user_id: &UserId) -> String {
    // Create a valid JWT for testing
    // In tests, we can use a known secret
    let claims = TestClaims {
        sub: user_id.to_string(),
        exp: chrono::Utc::now().timestamp() as usize + 3600,
    };

    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(b"test_secret"),
    ).unwrap()
}
```

### Integration Test Example

```rust
// tests/integration/http/session_api_tests.rs

use crate::common::{http::TestClient, factories::TestFactory};

#[tokio::test]
async fn test_create_session_requires_auth() {
    let app = create_test_app().await;
    let client = TestClient::new(app).await;

    let response = client.post("/api/v1/sessions", &json!({
        "title": "Test Session"
    })).await;

    assert_eq!(response.status_code(), 401);
}

#[tokio::test]
async fn test_create_session_success() {
    let app = create_test_app().await;
    let user_id = TestFactory::user_id();
    let client = TestClient::new(app).await.with_auth(&user_id);

    let response = client.post("/api/v1/sessions", &json!({
        "title": "Career Decision"
    })).await;

    assert_eq!(response.status_code(), 201);

    let body: SessionResponse = response.json();
    assert_eq!(body.title, "Career Decision");
    assert_eq!(body.user_id, user_id.to_string());
}

#[tokio::test]
async fn test_list_sessions_only_returns_own() {
    let pool = setup_test_database().await;

    // Create sessions for two users
    let user1 = TestFactory::user_id();
    let user2 = TestFactory::user_id();

    insert_test_session(&pool, &user1, "User1 Session").await;
    insert_test_session(&pool, &user2, "User2 Session").await;

    let app = create_test_app_with_db(pool).await;
    let client = TestClient::new(app).await.with_auth(&user1);

    let response = client.get("/api/v1/sessions").await;
    let body: ListSessionsResponse = response.json();

    assert_eq!(body.data.len(), 1);
    assert_eq!(body.data[0].title, "User1 Session");
}
```

---

## Test Utilities

### Assertion Helpers

```rust
// tests/common/assertions.rs

/// Assert that a result is Ok and return the value
#[macro_export]
macro_rules! assert_ok {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    };
}

/// Assert that a result is Err and matches a pattern
#[macro_export]
macro_rules! assert_err {
    ($expr:expr, $pattern:pat) => {
        match $expr {
            Err($pattern) => (),
            Err(e) => panic!("Expected error matching {}, got: {:?}", stringify!($pattern), e),
            Ok(v) => panic!("Expected Err, got Ok: {:?}", v),
        }
    };
}

/// Assert JSON response matches expected structure
pub fn assert_json_includes<T: Serialize>(response: &TestResponse, expected: T) {
    let response_json: serde_json::Value = response.json();
    let expected_json = serde_json::to_value(expected).unwrap();

    for (key, expected_value) in expected_json.as_object().unwrap() {
        let actual_value = &response_json[key];
        assert_eq!(
            actual_value, expected_value,
            "Mismatch for key '{}': expected {:?}, got {:?}",
            key, expected_value, actual_value
        );
    }
}
```

### Time Helpers

```rust
// tests/common/time.rs

use chrono::{DateTime, Utc, Duration};

/// Freeze time for deterministic tests
pub struct FrozenTime {
    time: DateTime<Utc>,
}

impl FrozenTime {
    pub fn now() -> Self {
        Self { time: Utc::now() }
    }

    pub fn at(time: DateTime<Utc>) -> Self {
        Self { time }
    }

    pub fn advance(&mut self, duration: Duration) {
        self.time = self.time + duration;
    }

    pub fn current(&self) -> DateTime<Utc> {
        self.time
    }
}

/// Create a timestamp relative to now
pub fn minutes_ago(minutes: i64) -> DateTime<Utc> {
    Utc::now() - Duration::minutes(minutes)
}

pub fn hours_ago(hours: i64) -> DateTime<Utc> {
    Utc::now() - Duration::hours(hours)
}

pub fn days_ago(days: i64) -> DateTime<Utc> {
    Utc::now() - Duration::days(days)
}
```

---

## Test Organization

### File Structure

```
backend/
├── tests/
│   ├── common/
│   │   ├── mod.rs              # Re-exports
│   │   ├── database.rs         # Database setup
│   │   ├── transaction.rs      # Transaction helpers
│   │   ├── factories.rs        # Test data factories
│   │   ├── http.rs             # HTTP test client
│   │   ├── assertions.rs       # Custom assertions
│   │   ├── time.rs             # Time utilities
│   │   └── mocks/
│   │       ├── mod.rs
│   │       ├── ai_provider.rs
│   │       ├── payment_provider.rs
│   │       └── access_checker.rs
│   │
│   ├── fixtures/
│   │   ├── users.sql
│   │   ├── sessions.sql
│   │   ├── memberships.sql
│   │   └── cycles.sql
│   │
│   ├── integration/
│   │   ├── mod.rs
│   │   ├── session_tests.rs
│   │   ├── cycle_tests.rs
│   │   ├── membership_tests.rs
│   │   └── http/
│   │       ├── mod.rs
│   │       ├── session_api_tests.rs
│   │       └── membership_api_tests.rs
│   │
│   └── e2e/
│       ├── mod.rs
│       └── full_workflow_test.rs
│
└── src/
    └── domain/
        └── */tests.rs          # Unit tests alongside code
```

### Running Tests

```bash
# Run all tests
cargo test

# Run unit tests only (fast)
cargo test --lib

# Run integration tests
cargo test --test '*'

# Run specific test file
cargo test --test session_tests

# Run with output
cargo test -- --nocapture

# Run tests matching pattern
cargo test session::create

# Generate coverage report
cargo tarpaulin --out Html
```

---

## CI Configuration

### GitHub Actions

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

env:
  DATABASE_URL: postgres://test:test@localhost:5432/choice_sherpa_test

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:16-alpine
        env:
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
          POSTGRES_DB: choice_sherpa_test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

      redis:
        image: redis:7-alpine
        ports:
          - 6379:6379

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Cache cargo
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run migrations
        run: cargo sqlx migrate run

      - name: Run tests
        run: cargo test --all-features

      - name: Generate coverage
        run: cargo tarpaulin --out Xml

      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

---

## Related Documents

- **Configuration**: `features/infrastructure/configuration.md`
- **Database Connection Pool**: `features/infrastructure/database-connection-pool.md`
- **HTTP Router**: `features/infrastructure/http-router.md`

---

*Version: 1.0.0*
*Created: 2026-01-09*
