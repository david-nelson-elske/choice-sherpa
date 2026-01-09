# Infrastructure: Database Migrations

**Type:** Cross-Cutting Infrastructure
**Priority:** P0 (Required for persistence)
**Last Updated:** 2026-01-09

> Complete specification for database schema management using sqlx migrations.

---

## Overview

Choice Sherpa uses sqlx for database migrations. This specification defines:
1. Migration file structure and naming conventions
2. Schema versioning strategy
3. Migration execution and rollback procedures
4. Schema design patterns for all modules
5. CI/CD integration

---

## Migration Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Migration Lifecycle                                  │
│                                                                              │
│   ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                 │
│   │   Write      │───▶│    Test      │───▶│    Apply     │                 │
│   │  Migration   │    │  Locally     │    │  Production  │                 │
│   └──────────────┘    └──────────────┘    └──────────────┘                 │
│          │                   │                   │                          │
│          ▼                   ▼                   ▼                          │
│   migrations/         sqlx migrate run    sqlx migrate run                  │
│   YYYYMMDDHHMMSS_     (local DB)         (prod via CI/CD)                  │
│   description.sql                                                           │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                    Migration Directory                               │   │
│   │                                                                      │   │
│   │   backend/migrations/                                                │   │
│   │   ├── 20260109000000_create_extensions.sql                          │   │
│   │   ├── 20260109000001_create_outbox.sql                              │   │
│   │   ├── 20260109000002_create_memberships.sql                         │   │
│   │   ├── 20260109000003_create_sessions.sql                            │   │
│   │   ├── 20260109000004_create_cycles.sql                              │   │
│   │   ├── 20260109000005_create_components.sql                          │   │
│   │   ├── 20260109000006_create_conversations.sql                       │   │
│   │   └── 20260109000007_create_dashboard_views.sql                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Naming Convention

### Format

```
{YYYYMMDDHHMMSS}_{description}.sql
```

- **Timestamp**: UTC timestamp when migration was created
- **Description**: Snake_case description of what the migration does

### Examples

```
20260109120000_create_extensions.sql
20260109120001_create_outbox_table.sql
20260109120002_create_memberships_table.sql
20260109120003_add_stripe_customer_id_to_memberships.sql
20260109120004_create_sessions_table.sql
```

### Naming Guidelines

| Action | Prefix | Example |
|--------|--------|---------|
| Create table | `create_` | `create_sessions_table.sql` |
| Add column | `add_` | `add_email_to_users.sql` |
| Remove column | `remove_` | `remove_legacy_field.sql` |
| Create index | `create_index_` | `create_index_sessions_user_id.sql` |
| Add constraint | `add_constraint_` | `add_constraint_unique_email.sql` |
| Enable feature | `enable_` | `enable_row_level_security.sql` |

---

## Migration Templates

### Foundation Migration (001)

```sql
-- 20260109000000_create_extensions.sql
-- Foundation: Required PostgreSQL extensions

-- Enable UUID generation
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Enable pgcrypto for encryption functions
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
```

### Outbox Pattern Migration (002)

```sql
-- 20260109000001_create_outbox.sql
-- Event sourcing outbox pattern for reliable event publishing

CREATE TABLE outbox (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    aggregate_type VARCHAR(100) NOT NULL,
    aggregate_id UUID NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ
);

-- Index for polling unprocessed events
CREATE INDEX idx_outbox_unprocessed
    ON outbox(created_at)
    WHERE processed_at IS NULL;

-- Index for aggregate lookup
CREATE INDEX idx_outbox_aggregate
    ON outbox(aggregate_type, aggregate_id);

-- Processed events for idempotency
CREATE TABLE processed_events (
    event_id UUID PRIMARY KEY,
    handler_name VARCHAR(100) NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for cleanup of old processed events
CREATE INDEX idx_processed_events_handler
    ON processed_events(handler_name, processed_at);

COMMENT ON TABLE outbox IS 'Transactional outbox for reliable event publishing';
COMMENT ON TABLE processed_events IS 'Idempotency tracking for event handlers';
```

### Membership Migration (003)

```sql
-- 20260109000002_create_memberships.sql
-- Membership aggregate and billing history

CREATE TABLE memberships (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL UNIQUE,
    tier VARCHAR(20) NOT NULL CHECK (tier IN ('free', 'monthly', 'annual')),
    status VARCHAR(20) NOT NULL CHECK (status IN ('active', 'cancelled', 'expired', 'pending')),
    stripe_customer_id VARCHAR(255),
    stripe_subscription_id VARCHAR(255),
    promo_code VARCHAR(50),
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version INTEGER NOT NULL DEFAULT 1
);

-- Indexes
CREATE INDEX idx_memberships_user_id ON memberships(user_id);
CREATE INDEX idx_memberships_stripe_customer
    ON memberships(stripe_customer_id)
    WHERE stripe_customer_id IS NOT NULL;
CREATE INDEX idx_memberships_status ON memberships(status);

-- Billing history for audit trail
CREATE TABLE billing_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    membership_id UUID NOT NULL REFERENCES memberships(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,
    amount_cents INTEGER,
    currency VARCHAR(3) DEFAULT 'CAD',
    stripe_invoice_id VARCHAR(255),
    stripe_payment_intent_id VARCHAR(255),
    description TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_billing_history_membership
    ON billing_history(membership_id, created_at DESC);

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_memberships_updated_at
    BEFORE UPDATE ON memberships
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE memberships IS 'User subscription memberships';
COMMENT ON TABLE billing_history IS 'Payment and billing event audit trail';
COMMENT ON COLUMN memberships.version IS 'Optimistic locking version';
```

### Session Migration (004)

```sql
-- 20260109000003_create_sessions.sql
-- Session aggregate for decision contexts

CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'archived', 'deleted')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    archived_at TIMESTAMPTZ,
    version INTEGER NOT NULL DEFAULT 1
);

-- Indexes
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_user_status ON sessions(user_id, status);
CREATE INDEX idx_sessions_created_at ON sessions(created_at DESC);

-- Trigger for updated_at
CREATE TRIGGER update_sessions_updated_at
    BEFORE UPDATE ON sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE sessions IS 'Decision-making session containers';
```

### Cycle Migration (005)

```sql
-- 20260109000004_create_cycles.sql
-- Cycle aggregate with branching support

CREATE TABLE cycles (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    parent_cycle_id UUID REFERENCES cycles(id),
    branch_point_component VARCHAR(50),
    status VARCHAR(20) NOT NULL DEFAULT 'in_progress'
        CHECK (status IN ('in_progress', 'completed', 'abandoned')),
    current_component VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    version INTEGER NOT NULL DEFAULT 1
);

-- Indexes
CREATE INDEX idx_cycles_session_id ON cycles(session_id);
CREATE INDEX idx_cycles_parent ON cycles(parent_cycle_id) WHERE parent_cycle_id IS NOT NULL;
CREATE INDEX idx_cycles_status ON cycles(status);

-- Trigger for updated_at
CREATE TRIGGER update_cycles_updated_at
    BEFORE UPDATE ON cycles
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE cycles IS 'PrOACT cycle instances with branching support';
COMMENT ON COLUMN cycles.parent_cycle_id IS 'Parent cycle for branched cycles';
COMMENT ON COLUMN cycles.branch_point_component IS 'Component where branch occurred';
```

### Component Migration (006)

```sql
-- 20260109000005_create_components.sql
-- PrOACT component data storage

CREATE TABLE components (
    id UUID PRIMARY KEY,
    cycle_id UUID NOT NULL REFERENCES cycles(id) ON DELETE CASCADE,
    component_type VARCHAR(50) NOT NULL
        CHECK (component_type IN (
            'issue_raising', 'problem_frame', 'objectives',
            'alternatives', 'consequences', 'tradeoffs',
            'recommendation', 'decision_quality'
        )),
    status VARCHAR(20) NOT NULL DEFAULT 'not_started'
        CHECK (status IN ('not_started', 'in_progress', 'completed', 'skipped')),
    data JSONB NOT NULL DEFAULT '{}',
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version INTEGER NOT NULL DEFAULT 1,

    -- Each cycle can have only one of each component type
    UNIQUE(cycle_id, component_type)
);

-- Indexes
CREATE INDEX idx_components_cycle_id ON components(cycle_id);
CREATE INDEX idx_components_type_status ON components(component_type, status);

-- GIN index for JSONB data queries
CREATE INDEX idx_components_data ON components USING GIN (data);

-- Trigger for updated_at
CREATE TRIGGER update_components_updated_at
    BEFORE UPDATE ON components
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE components IS 'PrOACT component instances with structured data';
COMMENT ON COLUMN components.data IS 'Component-specific structured data (JSON)';
```

### Conversation Migration (007)

```sql
-- 20260109000006_create_conversations.sql
-- AI conversation storage

CREATE TABLE conversations (
    id UUID PRIMARY KEY,
    component_id UUID NOT NULL REFERENCES components(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'paused', 'completed')),
    agent_phase VARCHAR(50) NOT NULL DEFAULT 'opening',
    context_summary TEXT,
    token_count INTEGER NOT NULL DEFAULT 0,
    message_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    version INTEGER NOT NULL DEFAULT 1
);

-- Messages within conversations
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    token_count INTEGER,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sequence_number INTEGER NOT NULL
);

-- Indexes
CREATE INDEX idx_conversations_component ON conversations(component_id);
CREATE INDEX idx_messages_conversation ON messages(conversation_id, sequence_number);
CREATE INDEX idx_messages_created_at ON messages(created_at);

-- Trigger for updated_at
CREATE TRIGGER update_conversations_updated_at
    BEFORE UPDATE ON conversations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE conversations IS 'AI-guided conversations for components';
COMMENT ON TABLE messages IS 'Individual messages in conversations';
```

---

## Rollback Strategy

### Reversible Migrations

Each migration should have a corresponding down migration when possible:

```sql
-- 20260109000003_create_sessions.sql

-- Up migration (sqlx runs this)
CREATE TABLE sessions (...);

-- Down migration (manual or future sqlx support)
-- Save as: 20260109000003_create_sessions.down.sql
DROP TABLE IF EXISTS sessions;
```

### Non-Reversible Migrations

Some migrations cannot be safely reversed:
- Data migrations (transformed data)
- Column drops (data loss)
- Type changes (potential data loss)

For these, document the limitation:

```sql
-- 20260115000000_migrate_user_data.sql
-- WARNING: Non-reversible migration - data transformation

-- Document what this migration does and why it can't be reversed
-- ...migration SQL...
```

### Manual Rollback Procedure

```bash
# 1. Connect to database
psql $DATABASE_URL

# 2. Check current state
SELECT * FROM _sqlx_migrations ORDER BY version DESC LIMIT 5;

# 3. Manually reverse last migration
\i backend/migrations/20260109000003_create_sessions.down.sql

# 4. Remove migration record
DELETE FROM _sqlx_migrations WHERE version = 20260109000003;
```

---

## CLI Commands

### sqlx-cli Installation

```bash
# Install sqlx-cli with Postgres support
cargo install sqlx-cli --no-default-features --features postgres
```

### Common Commands

```bash
# Create new migration
sqlx migrate add -r create_sessions_table

# Run all pending migrations
sqlx migrate run

# Revert last migration (requires down.sql)
sqlx migrate revert

# Show migration status
sqlx migrate info

# Build sqlx query cache (for compile-time verification)
cargo sqlx prepare
```

### Migration Info Output

```
$ sqlx migrate info

Applied migrations:
  20260109000000 create_extensions (applied: 2026-01-09 12:00:00 UTC)
  20260109000001 create_outbox (applied: 2026-01-09 12:00:01 UTC)
  20260109000002 create_memberships (applied: 2026-01-09 12:00:02 UTC)

Pending migrations:
  20260109000003 create_sessions
  20260109000004 create_cycles
```

---

## CI/CD Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/migrations.yml
name: Database Migrations

on:
  push:
    paths:
      - 'backend/migrations/**'

jobs:
  validate:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16-alpine
        env:
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
          POSTGRES_DB: test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4

      - name: Install sqlx-cli
        run: cargo install sqlx-cli --no-default-features --features postgres

      - name: Run migrations
        env:
          DATABASE_URL: postgres://test:test@localhost:5432/test
        run: sqlx migrate run

      - name: Verify rollback
        env:
          DATABASE_URL: postgres://test:test@localhost:5432/test
        run: |
          sqlx migrate revert || echo "Rollback not available"
          sqlx migrate run
```

### Production Deployment

```bash
#!/bin/bash
# deploy-migrations.sh

set -e

echo "Running database migrations..."

# Run migrations with timeout
timeout 300 sqlx migrate run

echo "Migration complete. Current status:"
sqlx migrate info
```

---

## Schema Design Patterns

### Optimistic Locking

All aggregates include a `version` column:

```sql
version INTEGER NOT NULL DEFAULT 1
```

Application code checks version on update:

```rust
UPDATE sessions
SET title = $1, version = version + 1
WHERE id = $2 AND version = $3
RETURNING version;
```

### Soft Deletes

Use status columns instead of DELETE:

```sql
status VARCHAR(20) NOT NULL DEFAULT 'active'
    CHECK (status IN ('active', 'archived', 'deleted'))
```

### Timestamps

All tables include:
- `created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()`
- `updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()` (with trigger)

### JSONB for Flexible Data

Use JSONB for semi-structured data with GIN indexes:

```sql
data JSONB NOT NULL DEFAULT '{}'
CREATE INDEX idx_components_data ON components USING GIN (data);
```

---

## File Structure

```
backend/
├── migrations/
│   ├── 20260109000000_create_extensions.sql
│   ├── 20260109000001_create_outbox.sql
│   ├── 20260109000002_create_memberships.sql
│   ├── 20260109000003_create_sessions.sql
│   ├── 20260109000004_create_cycles.sql
│   ├── 20260109000005_create_components.sql
│   ├── 20260109000006_create_conversations.sql
│   └── 20260109000007_create_dashboard_views.sql
└── .sqlx/
    └── query-*.json  # Compile-time query verification cache
```

---

## Testing Migrations

### Local Testing

```bash
# 1. Start fresh database
docker-compose down -v
docker-compose up -d postgres

# 2. Run migrations
sqlx migrate run

# 3. Verify schema
psql $DATABASE_URL -c "\dt"

# 4. Test rollback
sqlx migrate revert
sqlx migrate run
```

### Integration Test Setup

```rust
pub async fn setup_test_db() -> PgPool {
    let url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://test:test@localhost:5432/choice_sherpa_test".into());

    let pool = PgPool::connect(&url).await.unwrap();

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Migrations failed");

    // Clean tables
    sqlx::query("TRUNCATE sessions, cycles, components, conversations, memberships CASCADE")
        .execute(&pool)
        .await
        .unwrap();

    pool
}
```

---

## Related Documents

- **Database Connection Pool**: `features/infrastructure/database-connection-pool.md`
- **Configuration**: `features/infrastructure/configuration.md`
- **System Architecture**: `docs/architecture/SYSTEM-ARCHITECTURE.md`

---

*Version: 1.0.0*
*Created: 2026-01-09*
