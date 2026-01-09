# Streaming Protocol Specification

**Version:** 1.0.0
**Module:** conversation
**Last Updated:** 2026-01-08

> WebSocket streaming protocol for real-time AI conversation responses.

---

## Overview

Choice Sherpa uses WebSocket connections to stream AI responses in real-time. This document specifies the message format, connection lifecycle, and error handling for the streaming protocol.

---

## Connection Establishment

### WebSocket Endpoint

```
ws://{host}/api/components/{componentId}/stream
```

### Authentication

WebSocket connections authenticate via the initial HTTP upgrade request:

1. Include `Authorization: Bearer {token}` header in upgrade request
2. Server validates JWT token before accepting upgrade
3. Invalid tokens receive HTTP 401 before WebSocket upgrade

### Connection Flow

```
Client                                Server
  │                                     │
  │ ──── HTTP Upgrade Request ─────────►│
  │      Authorization: Bearer {jwt}    │
  │                                     │
  │ ◄─── 101 Switching Protocols ────── │
  │                                     │
  │ ══════ WebSocket Established ══════ │
  │                                     │
```

---

## Message Types

All messages are JSON-encoded with a `type` discriminator field.

### Client to Server

#### SendMessage

Request the AI to respond to a user message.

```typescript
interface SendMessageRequest {
  type: 'send_message';
  message_id: string;   // Client-generated UUID for tracking
  content: string;      // User's message text (max 10,000 chars)
}
```

**Example:**
```json
{
  "type": "send_message",
  "message_id": "550e8400-e29b-41d4-a716-446655440000",
  "content": "I'm considering whether to change careers or stay in my current job."
}
```

#### CancelStream

Cancel an in-progress streaming response.

```typescript
interface CancelStreamRequest {
  type: 'cancel_stream';
  message_id: string;   // Message ID of the stream to cancel
}
```

**Example:**
```json
{
  "type": "cancel_stream",
  "message_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

#### Ping

Keep-alive heartbeat (recommended every 30 seconds).

```typescript
interface PingRequest {
  type: 'ping';
}
```

### Server to Client

#### StreamChunk

Partial AI response content delivered incrementally.

```typescript
interface StreamChunkMessage {
  type: 'stream_chunk';
  message_id: string;   // Matches request message_id
  delta: string;        // Incremental text content
  is_final: boolean;    // True if this is the last chunk
}
```

**Example:**
```json
{
  "type": "stream_chunk",
  "message_id": "550e8400-e29b-41d4-a716-446655440000",
  "delta": "That's an important decision. Let me help you ",
  "is_final": false
}
```

#### StreamComplete

Sent after the final chunk with usage statistics.

```typescript
interface StreamCompleteMessage {
  type: 'stream_complete';
  message_id: string;
  full_content: string;          // Complete assembled response
  usage: TokenUsage;
  phase_transition?: PhaseTransition;  // If agent phase changed
}

interface TokenUsage {
  prompt_tokens: number;
  completion_tokens: number;
  total_tokens: number;
  estimated_cost_cents: number;  // Estimated cost in USD cents
}

interface PhaseTransition {
  from_phase: AgentPhase;
  to_phase: AgentPhase;
}

type AgentPhase = 'intro' | 'gather' | 'clarify' | 'extract' | 'confirm';
```

**Example:**
```json
{
  "type": "stream_complete",
  "message_id": "550e8400-e29b-41d4-a716-446655440000",
  "full_content": "That's an important decision. Let me help you think through this systematically...",
  "usage": {
    "prompt_tokens": 1250,
    "completion_tokens": 347,
    "total_tokens": 1597,
    "estimated_cost_cents": 4
  },
  "phase_transition": {
    "from_phase": "intro",
    "to_phase": "gather"
  }
}
```

#### StreamError

Error during streaming (partial content may have been sent).

```typescript
interface StreamErrorMessage {
  type: 'stream_error';
  message_id: string;
  error_code: StreamErrorCode;
  error: string;          // Human-readable error message
  partial_content?: string;  // Content received before error
  recoverable: boolean;   // Whether retry is recommended
}

type StreamErrorCode =
  | 'rate_limited'        // AI provider rate limit
  | 'context_too_long'    // Conversation exceeds token limit
  | 'content_filtered'    // AI response blocked by safety filter
  | 'provider_error'      // AI provider unavailable
  | 'cancelled'           // User cancelled the stream
  | 'timeout'             // Stream timed out
  | 'internal_error';     // Unexpected server error
```

**Example:**
```json
{
  "type": "stream_error",
  "message_id": "550e8400-e29b-41d4-a716-446655440000",
  "error_code": "rate_limited",
  "error": "AI provider rate limit exceeded. Please retry in 30 seconds.",
  "recoverable": true
}
```

#### Pong

Response to client ping.

```typescript
interface PongMessage {
  type: 'pong';
  timestamp: string;  // ISO 8601 timestamp
}
```

#### DataExtracted

Notifies client that structured data was extracted from the conversation.

```typescript
interface DataExtractedMessage {
  type: 'data_extracted';
  component_type: ComponentType;
  data: Record<string, unknown>;  // Extracted structured data
  extracted_at: string;           // ISO 8601 timestamp
}

type ComponentType =
  | 'issue_raising'
  | 'problem_frame'
  | 'objectives'
  | 'alternatives'
  | 'consequences'
  | 'tradeoffs'
  | 'recommendation'
  | 'decision_quality'
  | 'notes_next_steps';
```

**Example:**
```json
{
  "type": "data_extracted",
  "component_type": "issue_raising",
  "data": {
    "potential_decisions": [
      {"id": "d1", "description": "Whether to change careers"}
    ],
    "objectives": [
      {"id": "o1", "description": "Financial stability"}
    ],
    "uncertainties": [
      {"id": "u1", "description": "Job market conditions"}
    ]
  },
  "extracted_at": "2026-01-08T14:30:00Z"
}
```

---

## Message Sequence Diagrams

### Normal Flow

```
Client                                Server
  │                                     │
  │ ── SendMessage ────────────────────►│
  │    {message_id: "abc"}              │
  │                                     │
  │ ◄─────────────── StreamChunk ────── │
  │    {delta: "Hello"}                 │
  │ ◄─────────────── StreamChunk ────── │
  │    {delta: ", I can"}               │
  │ ◄─────────────── StreamChunk ────── │
  │    {delta: " help.", is_final: true}│
  │                                     │
  │ ◄─────────────── StreamComplete ─── │
  │    {full_content: "Hello, I can..."}│
  │                                     │
```

### Cancellation Flow

```
Client                                Server
  │                                     │
  │ ── SendMessage ────────────────────►│
  │    {message_id: "abc"}              │
  │                                     │
  │ ◄─────────────── StreamChunk ────── │
  │    {delta: "Hello"}                 │
  │                                     │
  │ ── CancelStream ───────────────────►│
  │    {message_id: "abc"}              │
  │                                     │
  │ ◄─────────────── StreamError ────── │
  │    {error_code: "cancelled"}        │
  │                                     │
```

### Error Flow

```
Client                                Server
  │                                     │
  │ ── SendMessage ────────────────────►│
  │    {message_id: "abc"}              │
  │                                     │
  │ ◄─────────────── StreamChunk ────── │
  │    {delta: "Processing"}            │
  │                                     │
  │ ◄─────────────── StreamError ────── │
  │    {error_code: "provider_error",   │
  │     partial_content: "Processing",  │
  │     recoverable: true}              │
  │                                     │
```

---

## Rate Limiting

| Limit | Value | Scope |
|-------|-------|-------|
| Messages per minute | 20 | Per user |
| Concurrent streams | 1 | Per connection |
| Message size | 10,000 chars | Per message |
| Connection idle timeout | 5 minutes | Per connection |
| Stream timeout | 2 minutes | Per stream |

### Rate Limit Response

When rate limited, server sends:

```json
{
  "type": "stream_error",
  "message_id": "abc",
  "error_code": "rate_limited",
  "error": "Rate limit exceeded. Retry after 30 seconds.",
  "recoverable": true
}
```

---

## Error Codes

| Code | Description | Recoverable | Action |
|------|-------------|-------------|--------|
| `rate_limited` | Too many requests | Yes | Wait and retry |
| `context_too_long` | Conversation too long | No | Start new conversation |
| `content_filtered` | Safety filter triggered | No | Rephrase message |
| `provider_error` | AI service unavailable | Yes | Retry with backoff |
| `cancelled` | User cancelled stream | N/A | No action needed |
| `timeout` | Stream exceeded time limit | Yes | Retry |
| `internal_error` | Server error | Maybe | Report to support |

---

## Client Implementation Guidelines

### Reconnection Strategy

```typescript
class ReconnectingWebSocket {
  private reconnectAttempts = 0;
  private maxAttempts = 10;
  private baseDelay = 1000;  // 1 second
  private maxDelay = 30000;  // 30 seconds

  private scheduleReconnect(): void {
    const delay = Math.min(
      this.baseDelay * Math.pow(2, this.reconnectAttempts),
      this.maxDelay
    );
    // Add jitter (+-25%)
    const jitter = delay * (0.75 + Math.random() * 0.5);

    this.reconnectAttempts++;
    setTimeout(() => this.connect(), jitter);
  }
}
```

### Stream Assembly

```typescript
class StreamAssembler {
  private chunks: Map<string, string[]> = new Map();

  handleChunk(message: StreamChunkMessage): string {
    const existing = this.chunks.get(message.message_id) || [];
    existing.push(message.delta);
    this.chunks.set(message.message_id, existing);

    // Return assembled content so far
    return existing.join('');
  }

  handleComplete(message: StreamCompleteMessage): void {
    this.chunks.delete(message.message_id);
  }
}
```

### Accessibility: Screen Reader Announcements

Stream content character-by-character, but announce complete sentences:

```typescript
class AccessibleStreamHandler {
  private buffer = '';
  private lastAnnouncedLength = 0;

  handleChunk(delta: string): void {
    this.buffer += delta;

    // Find complete sentences
    const sentences = this.buffer.match(/[^.!?]+[.!?]+/g) || [];
    const complete = sentences.join(' ');

    // Announce new sentences (not individual characters)
    if (complete.length > this.lastAnnouncedLength + 50) {
      const newContent = complete.slice(this.lastAnnouncedLength);
      this.announce(`AI says: ${newContent}`);
      this.lastAnnouncedLength = complete.length;
    }
  }

  private announce(text: string): void {
    // Use ARIA live region
    document.getElementById('announcer')!.textContent = text;
  }
}
```

---

## Security Considerations

### Authentication

- WebSocket upgrade requires valid JWT in Authorization header
- Session ownership verified for each message
- Tokens expire after 1 hour; reconnect with fresh token

### Input Validation

- Maximum message length: 10,000 characters
- Messages sanitized before storage
- Prompt injection patterns detected and blocked

### Response Sanitization

All AI responses are sanitized before delivery:
- HTML tags stripped
- Prompt injection markers removed
- Response length validated
- UTF-8 encoding verified

---

## Related Documents

- **Conversation Module:** `docs/modules/conversation.md`
- **Feature Specification:** `features/conversation/conversation-lifecycle.md`
- **Security Standard:** `docs/architecture/APPLICATION-SECURITY-STANDARD.md`
- **Accessibility Standard:** `docs/architecture/ACCESSIBILITY-STANDARD.md`

---

*Version: 1.0.0*
*Created: 2026-01-08*
