# Development Blockers Analysis: Choice Sherpa

**Generated:** 2026-01-09
**Purpose:** Identify all blockers before starting full development push

---

## Current State Summary

| Metric | Status |
|--------|--------|
| **Tests** | 510/510 passing |
| **Domain Layer** | ~40% complete |
| **Infrastructure** | ~5% complete |
| **Configuration** | 0% - nothing configured |
| **Frontend** | Does not exist |

---

## BLOCKER CATEGORY 1: Missing Cargo Dependencies

The `Cargo.toml` is missing critical crates needed for a full-stack application:

```toml
# MISSING - Must add to backend/Cargo.toml

# Database (CRITICAL)
sqlx = { version = "0.7", features = ["postgres", "uuid", "chrono", "runtime-tokio"] }

# HTTP Framework (CRITICAL)
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["trace", "cors", "timeout"] }

# Cache/PubSub
redis = { version = "0.24", features = ["aio", "tokio-comp"] }

# Configuration
config = "0.14"
dotenvy = "0.15"

# Authentication
jsonwebtoken = "9.2"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

**Impact:** Cannot implement any HTTP handlers, database persistence, or configuration loading.

---

## BLOCKER CATEGORY 2: Missing Configuration Infrastructure

### Required `.env.example` File

```env
# Database
DATABASE_URL=postgresql://choice-sherpa:password@localhost:5432/choice_sherpa

# Redis
REDIS_URL=redis://localhost:6379

# Authentication (Zitadel)
ZITADEL_AUTHORITY=https://auth.example.com
ZITADEL_CLIENT_ID=xxx
ZITADEL_AUDIENCE=xxx

# AI Providers
OPENAI_API_KEY=sk-xxx
ANTHROPIC_API_KEY=sk-ant-xxx

# Payment (Stripe)
STRIPE_API_KEY=sk_test_xxx
STRIPE_WEBHOOK_SECRET=whsec_xxx

# Email (Resend)
RESEND_API_KEY=re_xxx

# Server
HOST=0.0.0.0
PORT=8080
RUST_LOG=info
```

**Impact:** Tests that need external services can't run; integration testing is impossible.

---

## BLOCKER CATEGORY 3: Module Dependencies

### Critical Cross-Module Blocker: AccessChecker

```
Session Module → depends on → membership::AccessChecker → NOT DEFINED
```

The session module requires access checking before session creation:

```rust
// This port must be defined BEFORE session can be fully implemented
pub trait AccessChecker: Send + Sync {
    async fn can_create_session(&self, user_id: &UserId) -> Result<bool, AccessError>;
}
```

**Files needed:**
- `backend/src/ports/access_checker.rs` - trait definition
- `backend/src/adapters/membership/stub_access_checker.rs` - dev stub (always returns true)

---

## BLOCKER CATEGORY 4: Missing Ports (9 critical ports not yet defined)

| Port | Module | Status | Blocks |
|------|--------|--------|--------|
| `access_checker.rs` | membership | ❌ | session module |
| `membership_repository.rs` | membership | ❌ | membership persistence |
| `membership_reader.rs` | membership | ❌ | membership queries |
| `payment_provider.rs` | membership | ❌ | Stripe integration |
| `session_repository.rs` | session | ❌ | session persistence |
| `session_reader.rs` | session | ❌ | session queries |
| `cycle_repository.rs` | cycle | ❌ | cycle persistence |
| `cycle_reader.rs` | cycle | ❌ | cycle queries |
| `conversation_repository.rs` | conversation | ❌ | message persistence |

**What exists (8 ports):**
- `ai_provider.rs` ✅
- `event_publisher.rs` ✅
- `event_subscriber.rs` ✅
- `outbox_writer.rs` ✅
- `processed_event_store.rs` ✅
- `schema_validator.rs` ✅
- `circuit_breaker.rs` ✅
- `connection_registry.rs` ✅

---

## BLOCKER CATEGORY 5: No Database Infrastructure

### Missing Components

1. **No `migrations/` directory** - Schema is completely undefined
2. **No `.sqlx/` cache** - Offline query checking impossible
3. **No test database harness** - Integration tests can't run

### Required Migrations (Priority Order)

| Migration | Tables | Priority |
|-----------|--------|----------|
| `001_foundation.sql` | outbox, processed_events | P0 |
| `002_membership.sql` | memberships, billing_history | P0 |
| `003_session.sql` | sessions | P1 |
| `004_cycle.sql` | cycles, components | P1 |
| `005_conversation.sql` | conversations, messages | P2 |

---

## BLOCKER CATEGORY 6: Stranded Code (Unmerged Branch)

**Branch:** `feat/conversation-lifecycle`
**Status:** 12 commits ahead of main, **NOT merged**
**Contains ~40% of conversation module:**

```
cf943e7 fix(skills): restore workflow state integration in /dev and /tdd
78e38d2 fix(skills): restore workflow state tracking library
9a02ab1 docs(api): add streaming protocol specification
7fd2c92 test(conversation): add extraction integration tests
44ca5c5 test(conversation): add comprehensive phase transition tests
09f8547 feat(conversation): add component-specific agent configs and templates
048e710 feat(application): add StreamingMessageHandler for AI conversations
563058e feat(conversation): implement ContextWindowManager
1ac597e feat(conversation): implement DataExtractor with security sanitization
3f4837f feat(conversation): implement PhaseTransitionEngine
980055b feat(conversation): implement AgentPhase enum
44aaf1b feat(conversation): implement ConversationState enum
```

**Action required:** Review and merge, or the work is effectively lost.

---

## BLOCKER CATEGORY 7: No Frontend

The `frontend/` directory does **not exist**. This will need:

1. SvelteKit project initialization
2. TypeScript configuration
3. API client setup (shared types with backend)
4. Component library structure

---

## BLOCKER CATEGORY 8: No Docker/CI-CD

| Missing | Impact |
|---------|--------|
| `docker-compose.yml` | Cannot spin up local PostgreSQL/Redis |
| `Dockerfile` | Cannot containerize app |
| `.github/workflows/` | No automated testing |

---

## Recommended Action Plan

### Phase 0: Infrastructure Setup (Do First)

- [ ] Add missing Cargo dependencies
- [ ] Create .env.example
- [ ] Create docker-compose.yml for local dev (PostgreSQL + Redis)
- [ ] Add configuration loading module
- [ ] Initialize sqlx with offline mode

### Phase 1: Unblock Module Dependencies

- [ ] Define AccessChecker port + stub implementation
- [ ] Create database migrations for outbox/foundation tables
- [ ] Add basic tracing/logging setup

### Phase 2: Merge Stranded Work

- [ ] Review feat/conversation-lifecycle branch
- [ ] Merge or rebase to main
- [ ] Update CHECKLIST-conversation.md

### Phase 3: Complete Priority Modules

- [ ] Membership module (Money value object → AccessChecker impl)
- [ ] Session module (entity → commands → queries)
- [ ] HTTP layer (axum routes + handlers)

---

## Configuration Checklist for Meaningful Tests

For tests to be meaningful beyond pure unit tests:

| Requirement | Purpose | Status |
|-------------|---------|--------|
| Mock AI providers | AI conversation tests | ✅ Exists |
| In-memory event store | Event sourcing tests | ✅ Exists |
| Test database harness | Repository tests | ❌ Missing |
| Stub AccessChecker | Session creation tests | ❌ Missing |
| .env.test file | Test configuration | ❌ Missing |
| Docker test containers | Integration tests | ❌ Missing |

---

## Quick Reference: What's Blocking What

```
┌─────────────────────────────────────────────────────────────────┐
│                    INFRASTRUCTURE BLOCKERS                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Cargo.toml missing deps ──┬──► HTTP handlers blocked           │
│                            ├──► Database persistence blocked    │
│                            └──► Configuration loading blocked   │
│                                                                  │
│  No docker-compose ────────────► Local dev environment blocked  │
│                                                                  │
│  No migrations ────────────────► All persistence blocked        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                     MODULE BLOCKERS                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  AccessChecker port missing ───► Session module blocked         │
│                                                                  │
│  feat/conversation-lifecycle ──► 40% of conversation stranded   │
│  branch not merged                                               │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                     TESTING BLOCKERS                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  No test database harness ─────► Integration tests impossible   │
│                                                                  │
│  No .env.test ─────────────────► External service tests blocked │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

*This document should be updated as blockers are resolved.*
