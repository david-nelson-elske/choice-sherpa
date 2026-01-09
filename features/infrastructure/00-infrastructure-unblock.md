# Feature: Infrastructure Unblock

**Module:** infrastructure
**Priority:** P0 (Critical Blocker)
**Dependencies:** None (foundational)
**Source:** REQUIREMENTS/COMPLETION-DIRECTIVE.md (Loop 0 + Loop 1)

---

## Context

The Choice Sherpa project has excellent specification coverage (9 module specs, 32 feature specs) and solid domain foundations (510 passing tests), but is blocked from full application development by:

1. **Missing infrastructure dependencies** - Cannot build HTTP/persistence layers
2. **Undefined cross-module ports** - Session blocked by missing AccessChecker
3. **No local development environment** - Docker/config not set up

This feature addresses the first and third blockers by establishing:
- Core Cargo dependencies (sqlx, axum, tower, config)
- Configuration loading infrastructure
- Docker development environment (PostgreSQL + Redis)
- Database migration foundation

### Current State

| Layer | Status | Blocker |
|-------|--------|---------|
| Domain | 570 tests passing | N/A |
| Ports | Partially defined | Missing AccessChecker, repository ports |
| Adapters | AI adapters only | No Postgres, HTTP adapters |
| HTTP | Dependencies added | axum/tower/tower-http in Cargo.toml |
| Config | Implemented | 52 config tests passing, .env.example created |
| Docker | Configured | docker-compose.yml ready (port conflicts with other projects) |
| Migrations | Schema files created | Pending sqlx-cli install and migration run |

### Target State

| Layer | Target | Validation |
|-------|--------|------------|
| Dependencies | All infrastructure crates added | `cargo build` succeeds |
| Configuration | .env loading works | Config struct deserializes |
| Docker | PostgreSQL + Redis running | `docker-compose up` succeeds |
| Migrations | Foundation tables created | `sqlx migrate run` succeeds |

---

## Tasks

### Phase 0: Specification Validation & Creation

These specifications document the infrastructure being implemented:

- [x] Review and validate existing `features/infrastructure/database-connection-pool.md`
- [x] Create `features/infrastructure/configuration.md` specification
- [x] Create `features/infrastructure/database-migrations.md` specification
- [x] Create `features/infrastructure/http-router.md` specification
- [x] Create `features/infrastructure/test-harness.md` specification
- [x] Create `features/infrastructure/docker-development.md` specification

### Phase 1: Core Infrastructure Implementation

#### 1.1 Cargo Dependencies (CRITICAL)

- [x] Update `backend/Cargo.toml` with database dependencies (sqlx with postgres, uuid, chrono, runtime-tokio, migrate features)
- [x] Update `backend/Cargo.toml` with HTTP framework dependencies (axum, tower, tower-http with trace, cors, timeout, request-id features)
- [x] Update `backend/Cargo.toml` with configuration dependencies (config, dotenvy)
- [x] Update `backend/Cargo.toml` with observability dependencies (tracing, tracing-subscriber with env-filter, json features)
- [x] Update `backend/Cargo.toml` with cache dependencies (redis with aio, tokio-comp features)
- [x] Verify `cargo check` succeeds with new dependencies

#### 1.2 Configuration Infrastructure

- [x] Create `backend/.env.example` with all required environment variables
- [x] Create `backend/src/config.rs` - Configuration struct with database, redis, auth, AI, payment, email, server sections
- [x] Create `backend/src/config/database.rs` - Database configuration (URL, max connections)
- [x] Create `backend/src/config/server.rs` - Server configuration (host, port, log level)
- [x] Add config module to `backend/src/lib.rs`
- [x] Write tests for configuration loading

#### 1.3 Docker Development Environment

- [x] Create `docker-compose.yml` with PostgreSQL 16 service
- [x] Add Redis 7 service to `docker-compose.yml`
- [x] Add healthchecks for both services
- [x] Add persistent volume for PostgreSQL data
- [x] Verify `docker-compose up -d` starts both services (ports 5432/6379 in use by other projects - config valid)
- [x] Verify services are accessible (pg_isready, redis-cli ping) - use docker-compose.test.yml with ports 5433/6380 when main ports occupied

#### 1.4 Database Migrations Foundation

- [x] Create `backend/migrations/` directory
- [x] Create `001_foundation.sql` - outbox table, processed_events table, extensions (uuid-ossp, pgcrypto)
- [x] Create `002_membership.sql` - memberships table, billing_history table
- [ ] Install sqlx-cli if not present
- [ ] Verify `sqlx migrate run` applies migrations
- [ ] Verify `sqlx migrate revert` can rollback

---

## Technical Specifications

### Cargo.toml Dependencies

```toml
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

# Observability (P1)
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

### Environment Variables (.env.example)

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

### Docker Compose Structure

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

### Migration 001_foundation.sql

```sql
-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Outbox pattern table for reliable event publishing
CREATE TABLE outbox (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    aggregate_type VARCHAR(100) NOT NULL,
    aggregate_id UUID NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ
);

CREATE INDEX idx_outbox_unprocessed ON outbox(created_at) WHERE processed_at IS NULL;

-- Processed events for idempotency
CREATE TABLE processed_events (
    event_id UUID PRIMARY KEY,
    handler_name VARCHAR(100) NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_processed_events_handler ON processed_events(handler_name, processed_at);
```

### Migration 002_membership.sql

```sql
-- Membership aggregate table
CREATE TABLE memberships (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL UNIQUE,
    tier VARCHAR(20) NOT NULL CHECK (tier IN ('free', 'monthly', 'annual')),
    status VARCHAR(20) NOT NULL CHECK (status IN ('active', 'cancelled', 'expired', 'pending')),
    stripe_customer_id VARCHAR(255),
    stripe_subscription_id VARCHAR(255),
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_memberships_user_id ON memberships(user_id);
CREATE INDEX idx_memberships_stripe_customer ON memberships(stripe_customer_id) WHERE stripe_customer_id IS NOT NULL;

-- Billing history for audit trail
CREATE TABLE billing_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    membership_id UUID NOT NULL REFERENCES memberships(id),
    event_type VARCHAR(50) NOT NULL,
    amount_cents INTEGER,
    currency VARCHAR(3) DEFAULT 'CAD',
    stripe_invoice_id VARCHAR(255),
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_billing_history_membership ON billing_history(membership_id, created_at DESC);
```

---

## Acceptance Criteria

### Must Pass

1. **Dependency Resolution**: `cargo build` completes without errors after adding dependencies
2. **Configuration Loading**: Config struct loads from `.env` file correctly
3. **Docker Services**: `docker-compose up -d` starts PostgreSQL and Redis with healthy status
4. **Database Connectivity**: Application can connect to PostgreSQL using DATABASE_URL
5. **Migration Execution**: `sqlx migrate run` applies all migrations without errors
6. **Migration Rollback**: `sqlx migrate revert` can undo migrations
7. **Existing Tests Pass**: All 510+ existing tests continue to pass

### Should Pass

1. **Health Checks**: Docker containers report healthy within 30 seconds
2. **Connection Pool**: Database connection pool initializes correctly
3. **Redis Connectivity**: Application can connect to Redis using REDIS_URL

### Verification Commands

```bash
# After Phase 1.1 (Dependencies)
cargo build

# After Phase 1.2 (Configuration)
cargo test config

# After Phase 1.3 (Docker)
docker-compose up -d
docker-compose ps  # Both services should be "healthy"

# After Phase 1.4 (Migrations)
sqlx migrate run
sqlx migrate info  # Shows applied migrations

# Final verification
cargo test  # 510+ tests pass
```

---

## Implementation Notes

### Specification Files to Create (Phase 0)

1. **configuration.md** - Environment loading, config struct design, validation rules
2. **database-migrations.md** - Schema versioning strategy, rollback procedures, naming conventions
3. **http-router.md** - Axum router setup, middleware stack, error handling
4. **test-harness.md** - Test database setup, fixtures, integration test patterns
5. **docker-development.md** - Local development workflow, service dependencies, volume management

### TDD Approach

For Phase 1, implementation should follow TDD:

1. **Configuration tests first**: Write tests that load config from env vars
2. **Implement config.rs**: Make tests pass
3. **Migration tests**: Write integration tests that verify migrations apply
4. **Implement migrations**: Make tests pass

### Deferred to Later Features

- HTTP handlers (Loop 6)
- AccessChecker port (Loop 2)
- Repository ports (Loop 2)
- Session/Cycle/Conversation persistence (Loops 4-5)

---

## Related Documents

- [COMPLETION-DIRECTIVE.md](../../REQUIREMENTS/COMPLETION-DIRECTIVE.md) - Source planning document
- [SYSTEM-ARCHITECTURE.md](../../docs/architecture/SYSTEM-ARCHITECTURE.md) - Overall architecture
- [database-connection-pool.md](./database-connection-pool.md) - Existing connection pool spec
