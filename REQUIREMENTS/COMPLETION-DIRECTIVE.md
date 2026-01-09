# Choice Sherpa: Completion Directive

**Generated:** 2026-01-09
**Purpose:** Prioritized action plan to unblock full application development
**Source Documents:** MODULE-STATUS-conversation-membership.md, DEVELOPMENT-BLOCKERS-ANALYSIS.md

---

## Executive Summary

The project has **excellent specification coverage** (9 module specs, 32 feature specs) and **solid domain foundations** (510 passing tests), but is blocked from full development by:

1. **Missing infrastructure dependencies** - Cannot build HTTP/persistence layers
2. **Undefined cross-module ports** - Session blocked by missing AccessChecker
3. **Stranded code** - 40% of conversation module on unmerged branch
4. **No local development environment** - Docker/config not set up

This directive provides a **sequenced action plan** organized into development loops.

---

## Current Asset Inventory

### Specifications (Complete)

| Level | Type | Count | Status |
|-------|------|-------|--------|
| L0 | System Architecture | 1 | ✅ Complete |
| L1 | Module Specifications | 9 | ✅ Complete |
| L2 | Feature Specifications | 32 | ✅ Complete |
| L3 | Implementation Checklists | 10 | ✅ Complete |

### Implementation Status

| Module | Spec | Domain | Ports | Adapters | HTTP | Tests |
|--------|------|--------|-------|----------|------|-------|
| foundation | ✅ | ✅ 100% | N/A | N/A | N/A | 96/96 |
| proact-types | ✅ | ✅ 100% | N/A | N/A | N/A | 95/95 |
| analysis | ✅ | ✅ 100% | N/A | Optional | N/A | 61/61 |
| membership | ✅ | ⚠️ 5% | ❌ 0% | ❌ 0% | ❌ 0% | 18/95 |
| session | ✅ | ⚠️ 4% | ❌ 0% | ❌ 0% | ❌ 0% | 13/85 |
| cycle | ✅ | ⚠️ 5% | ❌ 0% | ❌ 0% | ❌ 0% | 38/82 |
| conversation | ✅ | ⚠️ 10%* | ✅ AI | ✅ AI | ❌ 0% | 87+ |
| ai-engine | ✅ | ✅ | ✅ 100% | ✅ 100% | ❌ 0% | 21/21 |
| dashboard | ✅ | ❌ 0% | ❌ 0% | ❌ 0% | ❌ 0% | 0/53 |
| events | ✅ | ✅ | ✅ 100% | ✅ 100% | N/A | 15+ |

*Conversation has 40% complete on unmerged branch `feat/conversation-lifecycle`

---

## LOOP 0: Specification Validation & Gap Analysis

**Objective:** Ensure all specifications are complete before implementation push

### 0.1 Validate Existing Specifications

| Spec File | Validation Task | Priority |
|-----------|-----------------|----------|
| `docs/modules/membership.md` | Verify Money, AccessChecker, PaymentProvider port definitions | P0 |
| `docs/modules/session.md` | Verify AccessChecker dependency documented | P0 |
| `features/infrastructure/database-connection-pool.md` | Verify schema design for all modules | P0 |
| `features/integrations/membership-access-control.md` | Verify AccessChecker interface complete | P0 |
| `features/membership/stripe-webhook-handling.md` | Verify webhook event types complete | P1 |

### 0.2 Create Missing Specifications

| Spec Needed | Location | Priority | Rationale |
|-------------|----------|----------|-----------|
| **Configuration Loading** | `features/infrastructure/configuration.md` | P0 | No spec for .env, config loading |
| **Database Migrations** | `features/infrastructure/database-migrations.md` | P0 | Schema design undocumented |
| **HTTP Router Setup** | `features/infrastructure/http-router.md` | P0 | Axum configuration undocumented |
| **Test Infrastructure** | `features/infrastructure/test-harness.md` | P1 | No spec for test database, fixtures |
| **Docker Development** | `features/infrastructure/docker-development.md` | P1 | Local dev setup undocumented |

### 0.3 Loop 0 Deliverables

```
[ ] Review and validate 5 critical existing specs
[ ] Create features/infrastructure/configuration.md
[ ] Create features/infrastructure/database-migrations.md
[ ] Create features/infrastructure/http-router.md
[ ] Create features/infrastructure/test-harness.md
[ ] Create features/infrastructure/docker-development.md
```

---

## LOOP 1: Infrastructure Unblock

**Objective:** Enable HTTP handlers, database persistence, and local development

**Dependencies:** Loop 0 (specifications complete)

### 1.1 Cargo Dependencies (CRITICAL BLOCKER)

**File:** `backend/Cargo.toml`

```toml
# ADD THESE DEPENDENCIES

# Database (P0)
sqlx = { version = "0.7", features = ["postgres", "uuid", "chrono", "runtime-tokio", "migrate"] }

# HTTP Framework (P0)
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["trace", "cors", "timeout", "request-id"] }

# Cache/PubSub (P1)
redis = { version = "0.24", features = ["aio", "tokio-comp"] }

# Configuration (P0)
config = "0.14"
dotenvy = "0.15"

# Authentication (P2)
jsonwebtoken = "9.2"

# Observability (P1)
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

### 1.2 Configuration Infrastructure

**Create:** `backend/.env.example`

```env
# Database
DATABASE_URL=postgresql://choice-sherpa:password@localhost:5432/choice_sherpa
DATABASE_MAX_CONNECTIONS=10

# Redis
REDIS_URL=redis://localhost:6379

# Authentication (Zitadel)
ZITADEL_AUTHORITY=https://auth.example.com
ZITADEL_CLIENT_ID=choice-sherpa
ZITADEL_AUDIENCE=choice-sherpa-api

# AI Providers
OPENAI_API_KEY=sk-xxx
ANTHROPIC_API_KEY=sk-ant-xxx
AI_PRIMARY_PROVIDER=anthropic
AI_FALLBACK_PROVIDER=openai

# Payment (Stripe)
STRIPE_API_KEY=sk_test_xxx
STRIPE_WEBHOOK_SECRET=whsec_xxx

# Email (Resend)
RESEND_API_KEY=re_xxx

# Server
HOST=0.0.0.0
PORT=8080
RUST_LOG=info,choice_sherpa=debug,sqlx=warn
```

**Create:** `backend/src/config.rs` - Configuration loading module

### 1.3 Docker Development Environment

**Create:** `docker-compose.yml`

```yaml
version: '3.8'
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: choice-sherpa
      POSTGRES_PASSWORD: password
      POSTGRES_DB: choice_sherpa
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U choice-sherpa"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
```

### 1.4 Database Migrations

**Create:** `backend/migrations/` directory with:

| Migration | Purpose | Priority |
|-----------|---------|----------|
| `001_foundation.sql` | outbox, processed_events tables | P0 |
| `002_membership.sql` | memberships, billing_history | P0 |
| `003_session.sql` | sessions table | P1 |
| `004_cycle.sql` | cycles, components tables | P1 |
| `005_conversation.sql` | conversations, messages | P2 |

### 1.5 Loop 1 Deliverables

```
[ ] Update backend/Cargo.toml with all dependencies
[ ] Create backend/.env.example
[ ] Create backend/src/config.rs
[ ] Create docker-compose.yml
[ ] Create backend/migrations/001_foundation.sql
[ ] Create backend/migrations/002_membership.sql
[ ] Verify `cargo build` succeeds
[ ] Verify `docker-compose up` succeeds
[ ] Verify `sqlx migrate run` succeeds
```

---

## LOOP 2: Cross-Module Dependency Unblock

**Objective:** Define ports that enable module integration

**Dependencies:** Loop 1 (infrastructure ready)

### 2.1 AccessChecker Port (CRITICAL - Unblocks Session)

**Create:** `backend/src/ports/access_checker.rs`

```rust
//! Access control port for membership-gated operations
//!
//! This port defines the contract for checking user access to platform features.
//! The Session module depends on this to gate session creation.

use crate::domain::foundation::{UserId, MembershipId};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AccessError {
    #[error("User not found: {0}")]
    UserNotFound(UserId),
    #[error("No active membership for user: {0}")]
    NoActiveMembership(UserId),
    #[error("Feature not available for membership tier")]
    TierRestricted,
    #[error("Internal error: {0}")]
    Internal(String),
}

#[async_trait]
pub trait AccessChecker: Send + Sync {
    /// Check if user can create a new session
    async fn can_create_session(&self, user_id: &UserId) -> Result<bool, AccessError>;

    /// Check if user can access a specific session
    async fn can_access_session(&self, user_id: &UserId, session_owner: &UserId) -> Result<bool, AccessError>;

    /// Get user's current membership ID (if any)
    async fn get_membership_id(&self, user_id: &UserId) -> Result<Option<MembershipId>, AccessError>;
}
```

**Create:** `backend/src/adapters/membership/stub_access_checker.rs`

```rust
//! Stub implementation for development and testing
//! Always grants access - replace with real implementation for production

use crate::ports::access_checker::{AccessChecker, AccessError};
use crate::domain::foundation::{UserId, MembershipId};
use async_trait::async_trait;

pub struct StubAccessChecker;

#[async_trait]
impl AccessChecker for StubAccessChecker {
    async fn can_create_session(&self, _user_id: &UserId) -> Result<bool, AccessError> {
        Ok(true) // Always allow in development
    }

    async fn can_access_session(&self, _user_id: &UserId, _session_owner: &UserId) -> Result<bool, AccessError> {
        Ok(true) // Always allow in development
    }

    async fn get_membership_id(&self, _user_id: &UserId) -> Result<Option<MembershipId>, AccessError> {
        Ok(None) // No membership tracking in stub
    }
}
```

### 2.2 Repository Ports (CQRS Pattern)

**Create these port files:**

| Port File | Purpose | Module |
|-----------|---------|--------|
| `ports/membership_repository.rs` | Write operations for Membership | membership |
| `ports/membership_reader.rs` | Read operations for Membership | membership |
| `ports/session_repository.rs` | Write operations for Session | session |
| `ports/session_reader.rs` | Read operations for Session | session |
| `ports/cycle_repository.rs` | Write operations for Cycle | cycle |
| `ports/cycle_reader.rs` | Read operations for Cycle | cycle |
| `ports/payment_provider.rs` | Stripe abstraction | membership |

### 2.3 Loop 2 Deliverables

```
[ ] Create backend/src/ports/access_checker.rs
[ ] Create backend/src/adapters/membership/stub_access_checker.rs
[ ] Create backend/src/ports/membership_repository.rs
[ ] Create backend/src/ports/membership_reader.rs
[ ] Create backend/src/ports/session_repository.rs
[ ] Create backend/src/ports/session_reader.rs
[ ] Create backend/src/ports/payment_provider.rs
[ ] Update backend/src/ports/mod.rs to export all ports
[ ] Verify `cargo test` passes (510+ tests)
```

---

## LOOP 3: Merge Stranded Code

**Objective:** Recover 40% of conversation module from feature branch

**Dependencies:** Loop 2 (ports defined)

### 3.1 Branch Analysis

**Branch:** `feat/conversation-lifecycle`
**Commits:** 12 ahead of main
**Content:** ~10,000 lines of conversation domain code

| Component | Status | Action |
|-----------|--------|--------|
| ConversationState enum | Complete | Merge |
| AgentPhase enum | Complete | Merge |
| PhaseTransitionEngine | Complete | Merge |
| DataExtractor | Complete | Merge |
| ContextWindowManager | Complete | Merge |
| Component configs (9) | Complete | Merge |
| StreamingMessageHandler | Complete | Merge |
| JSON schema validators | Complete | Merge |
| Streaming protocol spec | Complete | Merge |

### 3.2 Merge Process

```bash
# 1. Ensure main is up to date
git checkout main
git pull origin main

# 2. Rebase feature branch onto main
git checkout feat/conversation-lifecycle
git rebase main

# 3. Resolve any conflicts, then verify tests pass
cargo test --lib

# 4. If tests pass, merge to main
git checkout main
git merge feat/conversation-lifecycle

# 5. Push merged main
git push origin main
```

### 3.3 Post-Merge Verification

```
[ ] All 510+ tests still pass
[ ] Conversation domain code compiles
[ ] CHECKLIST-conversation.md updated to reflect merged state
[ ] Delete feature branch after merge
```

### 3.4 Loop 3 Deliverables

```
[ ] Rebase feat/conversation-lifecycle onto main
[ ] Resolve any merge conflicts
[ ] Verify cargo test passes
[ ] Merge to main
[ ] Update CHECKLIST-conversation.md
[ ] Delete merged branch
```

---

## LOOP 4: Membership Module Completion

**Objective:** Complete membership module to unblock session and enable access control

**Dependencies:** Loop 3 (stranded code merged)

### 4.1 Value Objects (P0)

| File | Description | Tests |
|------|-------------|-------|
| `domain/membership/value_objects/money.rs` | Cents-based integer currency | 8 |
| `domain/membership/value_objects/tier.rs` | Free/Monthly/Annual enum | 4 |
| `domain/membership/value_objects/billing_period.rs` | Monthly/Annual enum | 3 |
| `domain/membership/value_objects/promo_code.rs` | Workshop/beta access codes | 5 |
| `domain/membership/value_objects/plan_price.rs` | Tier pricing configuration | 4 |

### 4.2 Aggregate (P1)

| File | Description | Tests |
|------|-------------|-------|
| `domain/membership/membership.rs` | Membership aggregate root | 16 |
| `domain/membership/events.rs` | MembershipCreated, StatusChanged, etc. | 8 |

### 4.3 Commands & Queries (P2)

| File | Type | Description |
|------|------|-------------|
| `application/commands/create_free_membership.rs` | Command | Workshop/promo signup |
| `application/commands/create_paid_membership.rs` | Command | Stripe checkout |
| `application/commands/cancel_membership.rs` | Command | User cancellation |
| `application/queries/get_membership.rs` | Query | Get user's membership |
| `application/queries/check_access.rs` | Query | Access control check |

### 4.4 Postgres Adapters (P3)

| File | Description |
|------|-------------|
| `adapters/postgres/membership_repository.rs` | PostgresMembershipRepository |
| `adapters/postgres/membership_reader.rs` | PostgresMembershipReader |
| `adapters/postgres/access_checker.rs` | PostgresAccessChecker (real impl) |

### 4.5 Loop 4 Deliverables

```
[ ] Implement Money value object (TDD: red → green → refactor)
[ ] Implement Tier value object
[ ] Implement BillingPeriod value object
[ ] Implement PromoCode value object
[ ] Implement PlanPrice value object
[ ] Implement Membership aggregate
[ ] Implement MembershipEvent enum
[ ] Implement CreateFreeMembership command
[ ] Implement CreatePaidMembership command
[ ] Implement GetMembership query
[ ] Implement PostgresMembershipRepository
[ ] Implement PostgresMembershipReader
[ ] Implement PostgresAccessChecker
[ ] Update CHECKLIST-membership.md
```

---

## LOOP 5: Session Module Completion

**Objective:** Complete session module now that AccessChecker exists

**Dependencies:** Loop 4 (membership provides AccessChecker)

### 5.1 Domain Layer

| File | Description | Tests |
|------|-------------|-------|
| `domain/session/session.rs` | Session aggregate | 12 |
| `domain/session/errors.rs` | SessionError enum | 4 |

### 5.2 Commands & Queries

| File | Type | Description |
|------|------|-------------|
| `application/commands/create_session.rs` | Command | Create new session (uses AccessChecker) |
| `application/commands/rename_session.rs` | Command | Rename session title |
| `application/commands/archive_session.rs` | Command | Archive completed session |
| `application/queries/get_session.rs` | Query | Get session by ID |
| `application/queries/list_sessions.rs` | Query | List user's sessions |

### 5.3 Loop 5 Deliverables

```
[ ] Implement Session aggregate (TDD)
[ ] Implement SessionError enum
[ ] Implement CreateSession command (inject AccessChecker)
[ ] Implement RenameSession command
[ ] Implement ArchiveSession command
[ ] Implement GetSession query
[ ] Implement ListSessions query
[ ] Implement PostgresSessionRepository
[ ] Implement PostgresSessionReader
[ ] Update CHECKLIST-session.md
```

---

## LOOP 6: HTTP Layer

**Objective:** Expose REST API for frontend integration

**Dependencies:** Loop 5 (session complete)

### 6.1 HTTP Infrastructure

| File | Description |
|------|-------------|
| `adapters/http/mod.rs` | HTTP adapter module |
| `adapters/http/router.rs` | Axum router setup |
| `adapters/http/error.rs` | HTTP error responses |
| `adapters/http/middleware.rs` | Auth, logging, CORS |

### 6.2 Module Handlers

| Module | Endpoints |
|--------|-----------|
| Membership | `POST /api/memberships`, `GET /api/memberships/me` |
| Session | `POST /api/sessions`, `GET /api/sessions`, `GET /api/sessions/:id` |
| Cycle | `POST /api/sessions/:id/cycles`, `GET /api/cycles/:id` |
| Conversation | `POST /api/conversations`, WebSocket `/ws/conversations/:id` |

### 6.3 Loop 6 Deliverables

```
[ ] Create HTTP router infrastructure
[ ] Implement membership HTTP handlers
[ ] Implement session HTTP handlers
[ ] Implement cycle HTTP handlers
[ ] Implement conversation HTTP handlers
[ ] Implement WebSocket streaming
[ ] Add integration tests for HTTP layer
[ ] Update main.rs to start HTTP server
```

---

## Priority Summary

| Loop | Focus | Unblocks | Est. Effort |
|------|-------|----------|-------------|
| **0** | Specification validation & creation | All implementation | 2-4 hours |
| **1** | Infrastructure setup | HTTP, DB, config | 4-6 hours |
| **2** | Cross-module ports | Session, persistence | 2-3 hours |
| **3** | Merge stranded code | 40% of conversation | 1-2 hours |
| **4** | Membership module | Access control, payments | 8-12 hours |
| **5** | Session module | Core user workflow | 4-6 hours |
| **6** | HTTP layer | Frontend integration | 8-12 hours |

**Total estimated effort:** 29-45 hours

---

## Success Criteria

After completing all loops:

1. ✅ `cargo build --release` succeeds
2. ✅ `cargo test` passes 700+ tests
3. ✅ `docker-compose up` starts PostgreSQL + Redis
4. ✅ `cargo run` starts HTTP server on port 8080
5. ✅ `curl http://localhost:8080/health` returns 200
6. ✅ Session creation works end-to-end with access control
7. ✅ Conversation streaming works via WebSocket

---

## Next Action

**Start with Loop 0:** Validate and create missing infrastructure specifications before any code changes. This ensures implementation matches design intent.

```bash
# First command to run:
/dev features/infrastructure/
```

---

*This directive should be updated as loops are completed.*
