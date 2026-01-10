# Conversation Module Checklist

**Module:** Conversation
**Language:** Rust
**Dependencies:** foundation, proact-types
**Phase:** 3 (parallel with cycle, analysis)

---

## Overview

The Conversation module manages AI agent behavior, conversation flow, and message handling. It implements the "thoughtful decision professional" persona across all PrOACT components, handling the interaction between users and the AI assistant.

---

## Current Status

```
IN PROGRESS: conversation
Files: 13/75 (17%)
Tests: 200 passing
Frontend: Not started
```

**Note:** Core domain types, phase transitions, data extraction, and context management are implemented. Still needed: persistence layer, HTTP adapters, and frontend.

---

## File Inventory

### Domain Layer - Core (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/conversation/mod.rs` | Module exports | :white_check_mark: |
| `backend/src/domain/conversation/state.rs` | ConversationState enum with state machine | :white_check_mark: |
| `backend/src/domain/conversation/phase.rs` | AgentPhase enum (Greeting, Probing, etc.) | :white_check_mark: |
| `backend/src/domain/conversation/engine.rs` | PhaseTransitionEngine for phase progression | :white_check_mark: |
| `backend/src/domain/conversation/extractor.rs` | DataExtractor with security sanitization | :white_check_mark: |
| `backend/src/domain/conversation/context.rs` | ContextWindowManager for token budgets | :white_check_mark: |

### Domain Layer - Configuration (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/conversation/configs/mod.rs` | Config module exports | :white_check_mark: |
| `backend/src/domain/conversation/configs/agent_config.rs` | AgentConfig per component | :white_check_mark: |
| `backend/src/domain/conversation/configs/templates.rs` | Prompt templates per component | :white_check_mark: |

### Application Layer (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/mod.rs` | Application module exports | :white_check_mark: |
| `backend/src/application/handlers/mod.rs` | Handler exports | :white_check_mark: |
| `backend/src/application/handlers/stream_message.rs` | StreamingMessageHandler | :white_check_mark: |

### Documentation

| File | Description | Status |
|------|-------------|--------|
| `docs/api/streaming-protocol.md` | WebSocket streaming specification | :white_check_mark: |

---

## Implemented Features

### ConversationState (state.rs)
- [x] State enum: Initializing, Ready, InProgress, Confirmed, Complete
- [x] State machine transitions with validation
- [x] `can_transition_to()` method
- [x] `valid_transitions()` method
- [x] Helper methods: `is_active()`, `accepts_user_input()`, `can_generate_response()`
- [x] Serialization to snake_case

### AgentPhase (phase.rs)
- [x] Phase enum: Greeting, Probing, Clarifying, Synthesizing, Confirming, Extracting, Transitioning
- [x] Phase ordering and progression
- [x] Phase-specific behaviors
- [x] Serialization support

### PhaseTransitionEngine (engine.rs)
- [x] Phase progression logic
- [x] Conversation snapshots
- [x] Phase transition configuration
- [x] Integration with component types

### DataExtractor (extractor.rs)
- [x] JSON extraction from AI responses
- [x] Security sanitization
- [x] Field length limits (MAX_FIELD_LENGTH)
- [x] Response length limits (MAX_RESPONSE_LENGTH)
- [x] XSS/injection prevention
- [x] Comprehensive error handling

### ContextWindowManager (context.rs)
- [x] Token budget management
- [x] Message role handling
- [x] Context building for AI requests
- [x] Configuration options

### AgentConfig (configs/)
- [x] Component-specific configurations for all 9 PrOACT components
- [x] Phase prompts per component
- [x] Completion criteria
- [x] Opening messages per component
- [x] Extraction prompts per component

### StreamingMessageHandler (application/handlers/)
- [x] WebSocket message types
- [x] Streaming response handling
- [x] Token usage tracking
- [x] Broadcasting interface
- [x] Repository interface
- [x] AI provider interface

---

## NOT YET IMPLEMENTED

### Domain Layer

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/conversation/conversation.rs` | Conversation entity | :white_large_square: |
| `backend/src/domain/conversation/conversation_id.rs` | ConversationId value object | :white_large_square: |
| `backend/src/domain/conversation/errors.rs` | ConversationError enum | :white_large_square: |

### Ports (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/ports/conversation_repository.rs` | ConversationRepository trait | :white_large_square: |
| `backend/src/ports/conversation_reader.rs` | ConversationReader trait | :white_large_square: |

### Application Layer (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/commands/send_message.rs` | SendMessageCommand + Handler | :white_large_square: |
| `backend/src/application/commands/regenerate_response.rs` | RegenerateResponseCommand + Handler | :white_large_square: |
| `backend/src/application/queries/get_conversation.rs` | GetConversationQuery + Handler | :white_large_square: |

### Adapters - HTTP (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/http/conversation/mod.rs` | HTTP module exports | :white_large_square: |
| `backend/src/adapters/http/conversation/handlers.rs` | HTTP handlers | :white_large_square: |
| `backend/src/adapters/http/conversation/websocket_handler.rs` | WebSocket streaming | :white_large_square: |
| `backend/src/adapters/http/conversation/dto.rs` | Request/Response DTOs | :white_large_square: |
| `backend/src/adapters/http/conversation/routes.rs` | Route definitions | :white_large_square: |

### Adapters - Postgres (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/postgres/conversation_repository.rs` | Repository impl | :white_large_square: |
| `backend/src/adapters/postgres/conversation_reader.rs` | Reader impl | :white_large_square: |

### Migrations

| File | Description | Status |
|------|-------------|--------|
| `backend/migrations/XXXXXX_create_conversations.sql` | Create tables | :white_large_square: |

### Frontend Types (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/conversation/domain/conversation.ts` | Conversation type | :white_large_square: |
| `frontend/src/modules/conversation/domain/agent-state.ts` | AgentState type | :white_large_square: |
| `frontend/src/modules/conversation/index.ts` | Public exports | :white_large_square: |

### Frontend API (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/conversation/api/conversation-api.ts` | API client | :white_large_square: |
| `frontend/src/modules/conversation/api/use-conversation.ts` | Conversation hook | :white_large_square: |
| `frontend/src/modules/conversation/api/use-streaming.ts` | Streaming hook | :white_large_square: |

### Frontend Components (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/conversation/components/ChatInterface.tsx` | Main chat UI | :white_large_square: |
| `frontend/src/modules/conversation/components/MessageBubble.tsx` | Message display | :white_large_square: |
| `frontend/src/modules/conversation/components/TypingIndicator.tsx` | Typing indicator | :white_large_square: |
| `frontend/src/modules/conversation/components/InputArea.tsx` | Message input | :white_large_square: |

---

## Test Summary

### Implemented Tests (200 total)

| Category | Count | Description |
|----------|-------|-------------|
| ConversationState | ~26 | State transitions, behaviors |
| AgentPhase | ~32 | Phase definitions, ordering |
| PhaseTransitionEngine | ~48 | Engine behavior, snapshots |
| DataExtractor | ~52 | Extraction, sanitization, limits |
| ContextWindowManager | ~32 | Token budgets, context building |
| AgentConfig | ~15 | Component configs, templates |

---

## Verification Commands

```bash
# Run all conversation tests
cargo test --lib conversation

# Run specific category
cargo test --lib conversation::state
cargo test --lib conversation::phase
cargo test --lib conversation::engine
cargo test --lib conversation::extractor
cargo test --lib conversation::context
cargo test --lib conversation::configs

# Count tests
cargo test --lib conversation 2>&1 | grep -E "^test " | wc -l

# Full verification
cargo test --lib && cargo clippy -- -D warnings
```

---

## Exit Criteria (Updated)

### Current Progress: ~40%

The core domain logic for conversation management is implemented:
- [x] State machine for conversation lifecycle
- [x] Phase management for AI agent behavior
- [x] Data extraction with security measures
- [x] Context window management
- [x] Component-specific configurations
- [x] Streaming message handler skeleton

### Remaining Work: ~60%

- [ ] Conversation entity and ID
- [ ] Persistence layer (ports + adapters)
- [ ] HTTP/WebSocket adapters
- [ ] Additional application commands
- [ ] Database migrations
- [ ] Frontend implementation

---

## Notes

- Core domain types recovered from feat/conversation-lifecycle branch
- 200 tests passing covering state, phase, engine, extractor, context, configs
- AI provider ports already exist in infrastructure (ports/ai_provider.rs)
- Streaming protocol documented in docs/api/streaming-protocol.md
- Templates include prompts for all 9 PrOACT components

---

*Updated: 2026-01-09*
*Source: Recovered from feat/conversation-lifecycle branch*
*Test Count: 200 (domain) + handlers*
