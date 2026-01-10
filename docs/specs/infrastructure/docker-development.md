# Infrastructure: Docker Development Environment

**Type:** Cross-Cutting Infrastructure
**Priority:** P1 (Required for local development)
**Last Updated:** 2026-01-09

> Complete specification for Docker-based local development environment.

---

## Overview

Choice Sherpa uses Docker Compose for local development services. This specification defines:
1. Service definitions (PostgreSQL, Redis)
2. Volume management for persistence
3. Health checks and dependencies
4. Development workflow commands
5. Test environment configuration

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     Docker Compose Development Stack                         │
│                                                                              │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                        Host Machine                                  │   │
│   │                                                                      │   │
│   │   ┌─────────────────┐    ┌─────────────────┐                        │   │
│   │   │  Backend (Rust) │    │ Frontend (Svelte)│                       │   │
│   │   │  cargo run      │    │ npm run dev      │                       │   │
│   │   │  localhost:8080 │    │ localhost:5173   │                       │   │
│   │   └────────┬────────┘    └─────────────────┘                        │   │
│   │            │                                                         │   │
│   └────────────┼─────────────────────────────────────────────────────────┘   │
│                │                                                              │
│   ┌────────────┼─────────────────────────────────────────────────────────┐   │
│   │            ▼           Docker Network (choice-sherpa)                │   │
│   │                                                                      │   │
│   │   ┌─────────────────┐    ┌─────────────────┐                        │   │
│   │   │    PostgreSQL   │    │      Redis      │                        │   │
│   │   │    postgres:16  │    │    redis:7      │                        │   │
│   │   │    Port: 5432   │    │    Port: 6379   │                        │   │
│   │   │                 │    │                 │                        │   │
│   │   │   ┌─────────┐   │    │   ┌─────────┐   │                        │   │
│   │   │   │ Volume  │   │    │   │ Ephemeral│   │                        │   │
│   │   │   │postgres_│   │    │   │ (no vol) │   │                        │   │
│   │   │   │ data    │   │    │   └─────────┘   │                        │   │
│   │   │   └─────────┘   │    │                 │                        │   │
│   │   └─────────────────┘    └─────────────────┘                        │   │
│   │                                                                      │   │
│   └──────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Docker Compose Configuration

### Main Development File

```yaml
# docker-compose.yml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    container_name: choice-sherpa-postgres
    environment:
      POSTGRES_USER: choice-sherpa
      POSTGRES_PASSWORD: password
      POSTGRES_DB: choice_sherpa
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./backend/migrations:/docker-entrypoint-initdb.d:ro
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U choice-sherpa -d choice_sherpa"]
      interval: 5s
      timeout: 5s
      retries: 5
      start_period: 10s
    networks:
      - choice-sherpa

  redis:
    image: redis:7-alpine
    container_name: choice-sherpa-redis
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 5s
      retries: 5
    networks:
      - choice-sherpa
    # No volume - data is ephemeral for development
    # Add volume for persistence if needed:
    # volumes:
    #   - redis_data:/data

volumes:
  postgres_data:
    driver: local

networks:
  choice-sherpa:
    driver: bridge
```

### Test Environment Override

```yaml
# docker-compose.test.yml
version: '3.8'

services:
  postgres:
    environment:
      POSTGRES_DB: choice_sherpa_test
    ports:
      - "5433:5432"  # Different port to avoid conflicts
    volumes:
      - postgres_test_data:/var/lib/postgresql/data

  redis:
    ports:
      - "6380:6379"  # Different port to avoid conflicts

volumes:
  postgres_test_data:
    driver: local
```

### CI Environment

```yaml
# docker-compose.ci.yml
version: '3.8'

services:
  postgres:
    environment:
      POSTGRES_DB: choice_sherpa_ci
    # No port mapping - use Docker network
    ports: []
    healthcheck:
      interval: 2s
      timeout: 2s
      retries: 10

  redis:
    ports: []
    healthcheck:
      interval: 2s
      timeout: 2s
      retries: 10
```

---

## Service Details

### PostgreSQL

| Setting | Value | Notes |
|---------|-------|-------|
| Image | `postgres:16-alpine` | Latest stable, minimal size |
| User | `choice-sherpa` | Application-specific user |
| Database | `choice_sherpa` | Main database |
| Port | `5432` | Standard PostgreSQL port |
| Volume | `postgres_data` | Persistent storage |

**Connection URL:**
```
postgresql://choice-sherpa:password@localhost:5432/choice_sherpa
```

**Extensions available:**
- uuid-ossp (auto-enabled in migrations)
- pgcrypto

### Redis

| Setting | Value | Notes |
|---------|-------|-------|
| Image | `redis:7-alpine` | Latest stable, minimal size |
| Port | `6379` | Standard Redis port |
| Volume | None | Ephemeral for dev |

**Connection URL:**
```
redis://localhost:6379
```

---

## Commands Reference

### Starting Services

```bash
# Start all services
docker-compose up -d

# Start specific service
docker-compose up -d postgres

# Start with logs attached
docker-compose up

# Start test environment
docker-compose -f docker-compose.yml -f docker-compose.test.yml up -d
```

### Stopping Services

```bash
# Stop all services
docker-compose down

# Stop and remove volumes (clean slate)
docker-compose down -v

# Stop specific service
docker-compose stop postgres
```

### Service Management

```bash
# View service status
docker-compose ps

# View logs
docker-compose logs

# Follow logs for specific service
docker-compose logs -f postgres

# Restart service
docker-compose restart redis

# Execute command in container
docker-compose exec postgres psql -U choice-sherpa -d choice_sherpa
```

### Database Operations

```bash
# Connect to PostgreSQL
docker-compose exec postgres psql -U choice-sherpa -d choice_sherpa

# Run SQL file
docker-compose exec -T postgres psql -U choice-sherpa -d choice_sherpa < script.sql

# Backup database
docker-compose exec postgres pg_dump -U choice-sherpa choice_sherpa > backup.sql

# Restore database
docker-compose exec -T postgres psql -U choice-sherpa -d choice_sherpa < backup.sql
```

### Redis Operations

```bash
# Connect to Redis CLI
docker-compose exec redis redis-cli

# Check Redis info
docker-compose exec redis redis-cli info

# Flush all Redis data
docker-compose exec redis redis-cli FLUSHALL
```

---

## Development Workflow

### Initial Setup

```bash
# 1. Clone repository
git clone <repo-url>
cd choice-sherpa

# 2. Copy environment file
cp backend/.env.example backend/.env

# 3. Start Docker services
docker-compose up -d

# 4. Wait for services to be healthy
docker-compose ps
# Should show both services as "healthy"

# 5. Run database migrations
cd backend
sqlx migrate run

# 6. Verify setup
cargo test
```

### Daily Development

```bash
# Start services (if not running)
docker-compose up -d

# Check services are healthy
docker-compose ps

# Start backend
cd backend && cargo watch -x run

# In another terminal, start frontend
cd frontend && npm run dev
```

### Resetting Environment

```bash
# Full reset (removes all data)
docker-compose down -v
docker-compose up -d

# Wait for healthy
sleep 10

# Re-run migrations
cd backend && sqlx migrate run
```

---

## Makefile Integration

```makefile
# Makefile

.PHONY: up down reset db-shell redis-shell migrate test

# Start all services
up:
	docker-compose up -d
	@echo "Waiting for services..."
	@sleep 5
	@docker-compose ps

# Stop all services
down:
	docker-compose down

# Full reset
reset:
	docker-compose down -v
	docker-compose up -d
	@echo "Waiting for services..."
	@sleep 10
	cd backend && sqlx migrate run

# Database shell
db-shell:
	docker-compose exec postgres psql -U choice-sherpa -d choice_sherpa

# Redis shell
redis-shell:
	docker-compose exec redis redis-cli

# Run migrations
migrate:
	cd backend && sqlx migrate run

# Run tests
test: up
	cd backend && cargo test

# Check service health
health:
	@docker-compose ps
	@echo ""
	@echo "PostgreSQL:"
	@docker-compose exec postgres pg_isready -U choice-sherpa || true
	@echo ""
	@echo "Redis:"
	@docker-compose exec redis redis-cli ping || true
```

---

## Environment Files

### Backend .env.example

```env
# .env.example - Copy to .env and customize

# ============================================
# Database
# ============================================
DATABASE_URL=postgresql://choice-sherpa:password@localhost:5432/choice_sherpa
DATABASE_MAX_CONNECTIONS=10

# ============================================
# Redis
# ============================================
REDIS_URL=redis://localhost:6379

# ============================================
# Server
# ============================================
HOST=127.0.0.1
PORT=8080
RUST_LOG=info,choice_sherpa=debug,sqlx=warn

# ============================================
# Authentication (Zitadel)
# ============================================
# For local development, use mock auth or local Zitadel
ZITADEL_AUTHORITY=https://localhost:8443
ZITADEL_CLIENT_ID=choice-sherpa-dev
ZITADEL_AUDIENCE=choice-sherpa-api

# ============================================
# AI Providers
# ============================================
# At least one is required
OPENAI_API_KEY=sk-xxx
ANTHROPIC_API_KEY=sk-ant-xxx
AI_PRIMARY_PROVIDER=anthropic
AI_FALLBACK_PROVIDER=openai

# ============================================
# Payment (Stripe Test Mode)
# ============================================
STRIPE_API_KEY=sk_test_xxx
STRIPE_WEBHOOK_SECRET=whsec_xxx

# ============================================
# Email (Resend)
# ============================================
RESEND_API_KEY=re_xxx
```

### Test Environment

```env
# .env.test

DATABASE_URL=postgresql://choice-sherpa:password@localhost:5433/choice_sherpa_test
REDIS_URL=redis://localhost:6380
RUST_LOG=warn
```

---

## Health Check Verification

### Shell Script

```bash
#!/bin/bash
# scripts/check-services.sh

set -e

echo "Checking Docker services..."

# Check Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "ERROR: Docker is not running"
    exit 1
fi

# Check services are up
if ! docker-compose ps | grep -q "Up"; then
    echo "ERROR: Services are not running"
    echo "Run: docker-compose up -d"
    exit 1
fi

# Check PostgreSQL
echo "Checking PostgreSQL..."
if docker-compose exec -T postgres pg_isready -U choice-sherpa > /dev/null 2>&1; then
    echo "  PostgreSQL: OK"
else
    echo "  PostgreSQL: FAILED"
    exit 1
fi

# Check Redis
echo "Checking Redis..."
if docker-compose exec -T redis redis-cli ping > /dev/null 2>&1; then
    echo "  Redis: OK"
else
    echo "  Redis: FAILED"
    exit 1
fi

echo ""
echo "All services healthy!"
```

---

## Troubleshooting

### Common Issues

#### Port Already in Use

```bash
# Find process using port
lsof -i :5432
# or
netstat -tulpn | grep 5432

# Kill if needed, or change port in docker-compose.yml
```

#### Container Won't Start

```bash
# Check container logs
docker-compose logs postgres

# Common fixes:
# - Remove corrupted volume
docker-compose down -v
docker-compose up -d
```

#### Permission Issues on Volume

```bash
# Fix PostgreSQL data directory permissions
docker-compose down
sudo chown -R 999:999 ./postgres_data  # PostgreSQL UID
docker-compose up -d
```

#### Migration Failures

```bash
# Check if migrations applied
docker-compose exec postgres psql -U choice-sherpa -d choice_sherpa \
  -c "SELECT * FROM _sqlx_migrations ORDER BY version DESC LIMIT 5;"

# Manual migration reset
docker-compose exec postgres psql -U choice-sherpa -d choice_sherpa \
  -c "DROP TABLE IF EXISTS _sqlx_migrations CASCADE;"

sqlx migrate run
```

---

## Resource Limits (Optional)

For machines with limited resources:

```yaml
# docker-compose.override.yml
version: '3.8'

services:
  postgres:
    deploy:
      resources:
        limits:
          memory: 512M
        reservations:
          memory: 256M

  redis:
    deploy:
      resources:
        limits:
          memory: 128M
        reservations:
          memory: 64M
```

---

## Related Documents

- **Configuration**: `features/infrastructure/configuration.md`
- **Database Migrations**: `features/infrastructure/database-migrations.md`
- **Test Harness**: `features/infrastructure/test-harness.md`

---

*Version: 1.0.0*
*Created: 2026-01-09*
