# Hosting Selection: Self-Hosted VPS

> **Decision:** Self-hosted VPS with Docker Compose
> **Date:** 2026-01-07

---

## Summary

Self-hosted VPS selected for full infrastructure control, predictable costs, and simplified architecture. No object storage required - audio transcription is ephemeral (in-memory to Whisper API, only text persisted).

---

## Requirements

| Requirement | Priority |
|-------------|----------|
| Run Rust backend (axum) | Must |
| Run SvelteKit frontend | Must |
| PostgreSQL database | Must |
| Zitadel authentication | Must |
| Prometheus metrics | Must |
| Grafana dashboards | Should |
| SSL/TLS termination | Must |
| Automated deployments | Should |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         VPS (Single Server)                      │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    Docker Compose                            ││
│  │  ┌───────────┐  ┌───────────┐  ┌───────────┐               ││
│  │  │  Caddy    │  │  Backend  │  │ Frontend  │               ││
│  │  │ (reverse  │  │  (Rust)   │  │ (SvelteKit│               ││
│  │  │  proxy)   │  │  :8080    │  │  Node)    │               ││
│  │  │  :80/443  │  │           │  │  :3000    │               ││
│  │  └─────┬─────┘  └─────┬─────┘  └─────┬─────┘               ││
│  │        │              │              │                      ││
│  │        └──────────────┴──────────────┘                      ││
│  │                       │                                      ││
│  │  ┌───────────┐  ┌─────┴─────┐  ┌───────────┐               ││
│  │  │ Zitadel   │  │PostgreSQL │  │Prometheus │               ││
│  │  │  :8081    │  │  :5432    │  │  :9090    │               ││
│  │  └───────────┘  └───────────┘  └─────┬─────┘               ││
│  │                                      │                      ││
│  │                                ┌─────┴─────┐               ││
│  │                                │  Grafana  │               ││
│  │                                │  :3001    │               ││
│  │                                └───────────┘               ││
│  └─────────────────────────────────────────────────────────────┘│
│                                                                  │
│  Volumes: postgres_data, zitadel_data, prometheus_data,         │
│           grafana_data                                           │
└─────────────────────────────────────────────────────────────────┘
```

---

## No File Storage Required

Voice input flow eliminates object storage:

```
Browser → Backend (memory) → Whisper API → Text → PostgreSQL
                ↓
        Audio discarded after transcription
```

| Data Type | Storage | Retention |
|-----------|---------|-----------|
| Audio blobs | Memory only | Discarded after Whisper call |
| Transcription text | PostgreSQL | Permanent |
| Session data | PostgreSQL | Permanent |
| User data | PostgreSQL (via Zitadel) | Permanent |

---

## VPS Provider Options

| Provider | 4 vCPU / 8GB RAM | Location | Notes |
|----------|------------------|----------|-------|
| **Hetzner** | ~€15/mo (~$16) | EU, US | Best price/performance |
| DigitalOcean | $48/mo | Global | Good docs, higher cost |
| Linode | $48/mo | Global | Akamai-backed |
| Vultr | $48/mo | Global | Good API |
| OVH | ~€20/mo | EU, CA | Budget option |

**Recommendation:** Hetzner for cost efficiency, or DigitalOcean for US-centric users.

### Suggested Starting Spec

| Resource | Spec | Rationale |
|----------|------|-----------|
| CPU | 4 vCPU | Concurrent AI requests + Zitadel |
| RAM | 8GB | PostgreSQL + all services |
| Storage | 80GB NVMe | Database + logs |
| Bandwidth | 20TB | Generous for API traffic |

Scale vertically initially; consider splitting services at ~1000 daily active users.

---

## Docker Compose Configuration

```yaml
# docker-compose.yml

services:
  caddy:
    image: caddy:2-alpine
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      - caddy_data:/data
    depends_on:
      - backend
      - frontend

  backend:
    build: ./backend
    restart: unless-stopped
    environment:
      - DATABASE_URL=postgres://app:${DB_PASSWORD}@postgres:5432/choicesherpa
      - AUTH_ISSUER_URL=https://auth.${DOMAIN}
      - OPENAI_API_KEY=${OPENAI_API_KEY}
      - SENTRY_DSN=${SENTRY_DSN}
    depends_on:
      - postgres

  frontend:
    build: ./frontend
    restart: unless-stopped
    environment:
      - PUBLIC_API_URL=https://api.${DOMAIN}
      - AUTH_SECRET=${AUTH_SECRET}

  postgres:
    image: postgres:16-alpine
    restart: unless-stopped
    environment:
      - POSTGRES_DB=choicesherpa
      - POSTGRES_USER=app
      - POSTGRES_PASSWORD=${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data

  zitadel:
    image: ghcr.io/zitadel/zitadel:latest
    restart: unless-stopped
    command: start-from-init --masterkeyFromEnv
    environment:
      - ZITADEL_MASTERKEY=${ZITADEL_MASTERKEY}
      - ZITADEL_DATABASE_POSTGRES_HOST=postgres
      - ZITADEL_DATABASE_POSTGRES_DATABASE=zitadel
      - ZITADEL_EXTERNALDOMAIN=auth.${DOMAIN}
    depends_on:
      - postgres

  prometheus:
    image: prom/prometheus:latest
    restart: unless-stopped
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.retention.time=30d'

  grafana:
    image: grafana/grafana:latest
    restart: unless-stopped
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD}
    volumes:
      - grafana_data:/var/lib/grafana

volumes:
  caddy_data:
  postgres_data:
  prometheus_data:
  grafana_data:
```

---

## Caddy Configuration

```
# Caddyfile

{
    email admin@{$DOMAIN}
}

api.{$DOMAIN} {
    reverse_proxy backend:8080
}

app.{$DOMAIN} {
    reverse_proxy frontend:3000
}

auth.{$DOMAIN} {
    reverse_proxy zitadel:8080
}

grafana.{$DOMAIN} {
    reverse_proxy grafana:3000
    basicauth {
        admin {$GRAFANA_BASIC_AUTH_HASH}
    }
}
```

**Why Caddy:**
- Automatic HTTPS via Let's Encrypt
- Simple configuration
- No manual certificate renewal
- Lower memory than nginx

---

## Deployment Strategy

### Initial Setup

```bash
# 1. Provision VPS
# 2. Install Docker + Docker Compose
# 3. Clone repository
# 4. Configure .env file
# 5. Run docker compose up -d
```

### Updates (Manual for MVP)

```bash
ssh vps "cd /app && git pull && docker compose build && docker compose up -d"
```

### Future: GitHub Actions

```yaml
# .github/workflows/deploy.yml
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Deploy to VPS
        uses: appleboy/ssh-action@v1
        with:
          host: ${{ secrets.VPS_HOST }}
          username: deploy
          key: ${{ secrets.SSH_KEY }}
          script: |
            cd /app
            git pull
            docker compose build
            docker compose up -d --remove-orphans
```

---

## Cost Comparison

| Approach | Monthly Cost | Notes |
|----------|--------------|-------|
| **Self-hosted (Hetzner)** | ~$20 | VPS + domain |
| **Self-hosted (DO)** | ~$55 | VPS + domain |
| Fly.io | ~$30-50 | Per-service pricing adds up |
| Railway | ~$40-60 | Usage-based, less predictable |
| Render | ~$50-80 | Managed PostgreSQL expensive |
| AWS/GCP | ~$100+ | Overkill for MVP |

Self-hosted provides predictable costs and full control.

---

## Backup Strategy

### PostgreSQL

```bash
# Daily backup to local + offsite
0 3 * * * docker exec postgres pg_dump -U app choicesherpa | gzip > /backups/db-$(date +\%Y\%m\%d).sql.gz
0 4 * * * rclone copy /backups remote:choicesherpa-backups --max-age 24h
```

### Volumes

```bash
# Weekly volume snapshots (provider-dependent)
# Hetzner: Automated snapshots available
# DO: Droplet backups ($4/mo)
```

---

## Security Considerations

| Area | Approach |
|------|----------|
| SSH | Key-only, no password auth |
| Firewall | UFW: allow 80, 443, 22 only |
| Updates | Unattended-upgrades for OS |
| Secrets | `.env` file, not in repo |
| Database | Internal network only, no public port |
| Monitoring | Fail2ban for SSH brute force |

---

## Scaling Path

| Stage | Users | Infrastructure |
|-------|-------|----------------|
| MVP | 1-100 | Single VPS |
| Growth | 100-1000 | Larger VPS, managed PostgreSQL |
| Scale | 1000+ | Split services, load balancer |

Vertical scaling sufficient for considerable growth before horizontal complexity needed.

---

## Alternatives Considered

| Option | Rejection Reason |
|--------|------------------|
| **Fly.io** | Good option, but per-service costs add up |
| **Railway** | Usage-based pricing less predictable |
| **Render** | Managed DB pricing high for MVP |
| **Vercel + separate backend** | Split deployment complexity |
| **AWS/GCP** | Over-engineered for MVP stage |
| **Kubernetes** | Massive operational overhead |

---

## Trade-off Accepted

More operational responsibility accepted in exchange for:
- Full infrastructure control
- Predictable monthly costs
- No vendor lock-in
- Simple mental model (one server, one compose file)
- Easy debugging (SSH in, docker logs)

---

## Sources

- [Hetzner Cloud](https://www.hetzner.com/cloud)
- [Caddy Documentation](https://caddyserver.com/docs/)
- [Zitadel Docker Deployment](https://zitadel.com/docs/self-hosting/deploy/docker)
- [Docker Compose Specification](https://docs.docker.com/compose/compose-file/)
