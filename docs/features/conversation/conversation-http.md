# Feature: Conversation HTTP Endpoints

**Module:** conversation | **Phase:** 4 | **Priority:** P1

> REST endpoints and WebSocket handler for conversation access and streaming.

---

## Requirements

### GET /api/components/{id}/conversation

| ID | Rule | Given | When | Then | Error |
|----|------|-------|------|------|-------|
| R1 | Returns conversation | Component has conversation | GET | 200 with ConversationView | - |
| R2 | Returns 404 if none | No conversation for component | GET | 404 NotFound | `NotFound` |
| R3 | Requires authentication | No auth token | GET | 401 Unauthorized | `Unauthorized` |
| R4 | Requires ownership | User doesn't own component | GET | 403 Forbidden | `ForbiddenError` |

### GET /api/conversations/{id}/messages

| ID | Rule | Given | When | Then | Error |
|----|------|-------|------|------|-------|
| R5 | Returns messages paginated | Conversation has messages | GET | 200 with Page<MessageView> | - |
| R6 | Respects limit param | limit=10 | GET | Max 10 messages | - |
| R7 | Respects offset param | offset=5 | GET | Skips first 5 | - |
| R8 | Default limit 50 | No limit param | GET | Returns up to 50 | - |
| R9 | Max limit 100 | limit=500 | GET | Capped at 100 | - |
| R10 | Returns total count | Any request | GET | Response includes total | - |

### POST /api/components/{id}/conversation/regenerate

| ID | Rule | Given | When | Then | Error |
|----|------|-------|------|------|-------|
| R11 | Regenerates response | Valid conversation | POST | 200 with new message | - |
| R12 | Requires ownership | User doesn't own component | POST | 403 Forbidden | `ForbiddenError` |
| R13 | Rate limited | Too many requests | POST | 429 Too Many Requests | `RateLimitExceeded` |

### WebSocket /api/components/{id}/stream

| ID | Rule | Given | When | Then | Error |
|----|------|-------|------|------|-------|
| R14 | Accepts connection | Valid auth, owns component | Connect | 101 Switching Protocols | - |
| R15 | Rejects unauthorized | No auth | Connect | 401 before upgrade | `Unauthorized` |
| R16 | Receives user message | Connected | Send UserMessage | Acknowledged | - |
| R17 | Streams assistant tokens | AI generating | - | TokenChunk events | - |
| R18 | Sends completion event | AI finishes | - | MessageComplete event | - |
| R19 | Sends error event | AI fails | - | StreamError event | - |
| R20 | Handles disconnect | Client disconnects | - | Cleanup resources | - |

---

## Tasks

### REST Endpoints
- [ ] R1, R2: Implement get_conversation handler
- [ ] R3, R4: Add auth middleware and ownership check
- [ ] R5-R10: Implement get_messages with pagination
- [ ] R11-R13: Implement regenerate endpoint with rate limiting

### WebSocket
- [ ] R14, R15: Implement WebSocket upgrade with auth
- [ ] R16: Handle incoming UserMessage frames
- [ ] R17, R18: Stream TokenChunk and MessageComplete events
- [ ] R19: Handle AI errors gracefully
- [ ] R20: Clean up on client disconnect

### DTOs
- [ ] INT: Define ConversationView response DTO
- [ ] INT: Define MessageView response DTO
- [ ] INT: Define WebSocket message types (see streaming-protocol.md)

---

## Context

- Follow axum router pattern in `adapters/http/`
- Use existing `AuthExtractor` for authentication
- WebSocket protocol defined in `docs/api/streaming-protocol.md`
- Rate limiting via `tower` middleware (per user, per endpoint)
- Pagination uses `Page<T>` wrapper from foundation
