# Implementation Checklist: Cross-Cutting Integrations

**Type:** Cross-Cutting Infrastructure
**Priority:** P1-P2 (Required for production deployment)
**Created:** 2026-01-09

---

## Overview

This checklist tracks implementation of cross-cutting integration features that span multiple modules. These are infrastructure concerns required for a production-ready application.

### Feature Specifications

| Feature | Specification | Tasks | Status |
|---------|---------------|-------|--------|
| Authentication & Identity | `features/integrations/authentication-identity.md` | 0/17 | Not started |
| Event Versioning | `features/integrations/event-versioning.md` | 0/35 | Not started |
| Membership Access Control | `features/integrations/membership-access-control.md` | 3/22 | In progress |
| Notification Service | `features/integrations/notification-service.md` | 0/25 | Not started |
| Observability | `features/integrations/observability.md` | 0/25 | Not started |
| Rate Limiting | `features/integrations/rate-limiting.md` | 0/25 | Not started |
| WebSocket Dashboard | `features/integrations/websocket-dashboard.md` | 0/16 | Not started |

**Total Progress: 3/165 (2%)**

---

## 1. Authentication & Identity (Zitadel OIDC)

*Specification: `features/integrations/authentication-identity.md`*

### Infrastructure Setup

- [ ] Deploy Zitadel instance
- [ ] Configure OIDC application for frontend
- [ ] Configure service account for backend
- [ ] Set up email via Resend SMTP

### Backend Implementation

- [ ] Implement SessionValidator port
- [ ] Implement ZitadelSessionValidator adapter
- [ ] Add JWT verification middleware
- [ ] Create user context extraction
- [ ] Write authentication tests

### Frontend Implementation

- [ ] Configure OIDC client
- [ ] Implement login/logout flows
- [ ] Add protected route guards
- [ ] Handle token refresh

### Integration Tests

- [ ] End-to-end authentication flow
- [ ] Token expiration handling
- [ ] Multi-device session management

---

## 2. Event Versioning

*Specification: `features/integrations/event-versioning.md`*

### Domain Types

- [ ] EventVersion value object
- [ ] Upcaster trait definition
- [ ] UpcasterRegistry struct
- [ ] VersionedEvent wrapper

### Upcaster Infrastructure

- [ ] Version detection from payload
- [ ] Upcaster chain execution
- [ ] Error handling for failed upcasts

### Per-Event Upcasters

- [ ] SessionCreatedV2 → V3 upcaster (and other event types as needed)

### Testing

- [ ] Upcaster unit tests
- [ ] Registry integration tests
- [ ] Round-trip version tests

---

## 3. Membership Access Control

*Specification: `features/integrations/membership-access-control.md`*

### Completed

- [x] AccessChecker port definition
- [x] TierLimits value object (+ MembershipTier enum)
- [x] StubAccessChecker implementation (for development/testing)

### Pending

- [ ] Session module integration (requires Session aggregate)
- [ ] PromoCode value object
- [ ] PromoCodeValidator port
- [ ] Promo code redemption logic
- [ ] Usage tracking per tier
- [ ] Limit enforcement in handlers
- [ ] Upgrade/downgrade flows

### Testing

- [ ] Access control unit tests
- [ ] Tier limit enforcement tests
- [ ] Promo code validation tests

---

## 4. Notification Service

*Specification: `features/integrations/notification-service.md`*

### Domain Layer

- [ ] Define notification ports
- [ ] Create domain types (NotificationType, NotificationChannel, etc.)
- [ ] Define notification templates

### Adapters

- [ ] Implement console email sender (for dev)
- [ ] Implement Resend adapter (for production)
- [ ] Implement in-app notification adapter

### Event Handlers

- [ ] Subscribe to relevant domain events
- [ ] Route notifications to appropriate channels
- [ ] Handle delivery failures

### Testing

- [ ] Write unit tests
- [ ] Integration tests with mock providers

---

## 5. Observability

*Specification: `features/integrations/observability.md`*

### Structured Logging

- [ ] Configure tracing-subscriber with JSON format
- [ ] Add request/response logging middleware
- [ ] Standardize log fields across modules
- [ ] Add #[instrument] to key handlers

### Metrics

- [ ] Define key metrics (request latency, error rates, etc.)
- [ ] Implement Prometheus metrics endpoint
- [ ] Add business metrics (decisions created, cycles completed)

### Distributed Tracing

- [ ] Configure trace propagation
- [ ] Add trace context to events
- [ ] Integrate with observability platform

### Testing

- [ ] Write unit tests for log format
- [ ] Verify metric collection

---

## 6. Rate Limiting

*Specification: `features/integrations/rate-limiting.md`*

### Domain Layer

- [ ] Define RateLimiter port
- [ ] Create rate limit configuration types
- [ ] Define rate limit rules per endpoint

### Adapters

- [ ] Implement InMemoryRateLimiter for testing
- [ ] Implement RedisRateLimiter with Lua scripts

### Middleware

- [ ] Create rate limit middleware
- [ ] Add bypass for internal services
- [ ] Implement retry-after headers

### Testing

- [ ] Write unit tests for token bucket algorithm
- [ ] Integration tests for Redis adapter
- [ ] Load tests for limit enforcement

---

## 7. WebSocket Dashboard

*Specification: `features/integrations/websocket-dashboard.md`*

### Backend Infrastructure

- [ ] `RoomManager` struct with join/leave
- [ ] WebSocket upgrade handler
- [ ] Connected/disconnected messages
- [ ] `WebSocketEventBridge` handler

### Event Routing

- [ ] Subscribe to dashboard-relevant events
- [ ] Route events to appropriate rooms
- [ ] Handle room lifecycle

### Frontend

- [ ] Basic `useDashboardLive` hook
- [ ] Reconnection logic
- [ ] Optimistic UI updates

### Testing

- [ ] WebSocket connection tests
- [ ] Event delivery tests
- [ ] Reconnection behavior tests

---

## Summary

| Section | Tasks | Completed | Percentage |
|---------|-------|-----------|------------|
| Authentication & Identity | 17 | 0 | 0% |
| Event Versioning | 35 | 0 | 0% |
| Membership Access Control | 22 | 3 | 14% |
| Notification Service | 25 | 0 | 0% |
| Observability | 25 | 0 | 0% |
| Rate Limiting | 25 | 0 | 0% |
| WebSocket Dashboard | 16 | 0 | 0% |
| **Total** | **165** | **3** | **2%** |

---

*Last Updated: 2026-01-09*
