# Conversation Module Checklist

**Module:** Conversation
**Language:** Rust
**Dependencies:** foundation, proact-types
**Phase:** 3 (parallel with cycle, analysis)

---

## Overview

The Conversation module manages AI agent behavior, conversation flow, and message handling. It implements the "thoughtful decision professional" persona across all PrOACT components, handling the interaction between users and the AI assistant.

---

## File Inventory

### Domain Layer (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/conversation/mod.rs` | Module exports | ⬜ |
| `backend/src/domain/conversation/conversation.rs` | Conversation entity | ⬜ |
| `backend/src/domain/conversation/conversation_id.rs` | ConversationId value object | ⬜ |
| `backend/src/domain/conversation/agent_state.rs` | AgentState value object | ⬜ |
| `backend/src/domain/conversation/agent_config.rs` | AgentConfig and phases | ⬜ |
| `backend/src/domain/conversation/agent_phase.rs` | AgentPhase struct | ⬜ |
| `backend/src/domain/conversation/extraction_rule.rs` | ExtractionRule struct | ⬜ |
| `backend/src/domain/conversation/errors.rs` | ConversationError enum | ⬜ |

### Agent Configurations (Per-Component)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/conversation/agent_configs/mod.rs` | Config exports | ⬜ |
| `backend/src/domain/conversation/agent_configs/issue_raising.rs` | IssueRaising config | ⬜ |
| `backend/src/domain/conversation/agent_configs/problem_frame.rs` | ProblemFrame config | ⬜ |
| `backend/src/domain/conversation/agent_configs/objectives.rs` | Objectives config | ⬜ |
| `backend/src/domain/conversation/agent_configs/alternatives.rs` | Alternatives config | ⬜ |
| `backend/src/domain/conversation/agent_configs/consequences.rs` | Consequences config | ⬜ |
| `backend/src/domain/conversation/agent_configs/tradeoffs.rs` | Tradeoffs config | ⬜ |
| `backend/src/domain/conversation/agent_configs/recommendation.rs` | Recommendation config | ⬜ |
| `backend/src/domain/conversation/agent_configs/decision_quality.rs` | DecisionQuality config | ⬜ |
| `backend/src/domain/conversation/agent_configs/notes_next_steps.rs` | NotesNextSteps config | ⬜ |

### System Prompts

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/conversation/prompts/issue_raising.txt` | IssueRaising system prompt | ⬜ |
| `backend/src/domain/conversation/prompts/problem_frame.txt` | ProblemFrame system prompt | ⬜ |
| `backend/src/domain/conversation/prompts/objectives.txt` | Objectives system prompt | ⬜ |
| `backend/src/domain/conversation/prompts/alternatives.txt` | Alternatives system prompt | ⬜ |
| `backend/src/domain/conversation/prompts/consequences.txt` | Consequences system prompt | ⬜ |
| `backend/src/domain/conversation/prompts/tradeoffs.txt` | Tradeoffs system prompt | ⬜ |
| `backend/src/domain/conversation/prompts/recommendation.txt` | Recommendation system prompt | ⬜ |
| `backend/src/domain/conversation/prompts/decision_quality.txt` | DecisionQuality system prompt | ⬜ |
| `backend/src/domain/conversation/prompts/notes_next_steps.txt` | NotesNextSteps system prompt | ⬜ |

### Domain Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/conversation/conversation_test.rs` | Conversation entity tests | ⬜ |
| `backend/src/domain/conversation/conversation_id_test.rs` | ConversationId tests | ⬜ |
| `backend/src/domain/conversation/agent_state_test.rs` | AgentState tests | ⬜ |
| `backend/src/domain/conversation/agent_config_test.rs` | AgentConfig tests | ⬜ |

### Ports (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/ports/ai_provider.rs` | AIProvider trait | ⬜ |
| `backend/src/ports/ai_types.rs` | CompletionRequest, Response, Chunk, Error | ⬜ |
| `backend/src/ports/conversation_repository.rs` | ConversationRepository trait | ⬜ |
| `backend/src/ports/conversation_reader.rs` | ConversationReader trait | ⬜ |

### Application Layer (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/commands/send_message.rs` | SendMessageCommand + Handler | ⬜ |
| `backend/src/application/commands/stream_message.rs` | StreamMessageCommand + Handler | ⬜ |
| `backend/src/application/commands/regenerate_response.rs` | RegenerateResponseCommand + Handler | ⬜ |
| `backend/src/application/queries/get_conversation.rs` | GetConversationQuery + Handler | ⬜ |

### Application Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/commands/send_message_test.rs` | SendMessage tests | ⬜ |
| `backend/src/application/commands/stream_message_test.rs` | StreamMessage tests | ⬜ |
| `backend/src/application/commands/regenerate_response_test.rs` | RegenerateResponse tests | ⬜ |
| `backend/src/application/queries/get_conversation_test.rs` | GetConversation tests | ⬜ |

### Adapters - AI Providers (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/ai/mod.rs` | AI adapter exports | ⬜ |
| `backend/src/adapters/ai/openai_adapter.rs` | OpenAI implementation | ⬜ |
| `backend/src/adapters/ai/anthropic_adapter.rs` | Anthropic implementation | ⬜ |
| `backend/src/adapters/ai/mock_adapter.rs` | Mock for testing | ⬜ |

### Adapters - AI Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/ai/openai_adapter_test.rs` | OpenAI adapter tests | ⬜ |
| `backend/src/adapters/ai/anthropic_adapter_test.rs` | Anthropic adapter tests | ⬜ |
| `backend/src/adapters/ai/mock_adapter_test.rs` | Mock adapter tests | ⬜ |

### Adapters - HTTP (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/http/conversation/mod.rs` | HTTP module exports | ⬜ |
| `backend/src/adapters/http/conversation/handlers.rs` | HTTP handlers | ⬜ |
| `backend/src/adapters/http/conversation/websocket_handler.rs` | WebSocket streaming | ⬜ |
| `backend/src/adapters/http/conversation/dto.rs` | Request/Response DTOs | ⬜ |
| `backend/src/adapters/http/conversation/routes.rs` | Route definitions | ⬜ |

### Adapters - Postgres (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/postgres/conversation_repository.rs` | Repository impl | ⬜ |
| `backend/src/adapters/postgres/conversation_reader.rs` | Reader impl | ⬜ |

### Adapters - Postgres Tests (Rust)

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/postgres/conversation_repository_test.rs` | Repository tests | ⬜ |
| `backend/src/adapters/postgres/conversation_reader_test.rs` | Reader tests | ⬜ |

### Migrations

| File | Description | Status |
|------|-------------|--------|
| `backend/migrations/XXXXXX_create_conversations.sql` | Create tables | ⬜ |

### Frontend Types (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/conversation/domain/conversation.ts` | Conversation type | ⬜ |
| `frontend/src/modules/conversation/domain/agent-state.ts` | AgentState type | ⬜ |
| `frontend/src/modules/conversation/index.ts` | Public exports | ⬜ |

### Frontend API (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/conversation/api/conversation-api.ts` | API client | ⬜ |
| `frontend/src/modules/conversation/api/use-conversation.ts` | Conversation hook | ⬜ |
| `frontend/src/modules/conversation/api/use-streaming.ts` | Streaming hook | ⬜ |

### Frontend Components (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/conversation/components/ChatInterface.tsx` | Main chat UI | ⬜ |
| `frontend/src/modules/conversation/components/MessageBubble.tsx` | Message display | ⬜ |
| `frontend/src/modules/conversation/components/TypingIndicator.tsx` | Typing indicator | ⬜ |
| `frontend/src/modules/conversation/components/InputArea.tsx` | Message input | ⬜ |

### Frontend Tests (TypeScript)

| File | Description | Status |
|------|-------------|--------|
| `frontend/src/modules/conversation/components/ChatInterface.test.tsx` | ChatInterface tests | ⬜ |
| `frontend/src/modules/conversation/api/use-conversation.test.ts` | Hook tests | ⬜ |

---

## Test Inventory

### ConversationId Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_conversation_id_new_generates_unique` | Each call produces different ID | ⬜ |
| `test_conversation_id_from_uuid_preserves_value` | Wrapping preserves UUID | ⬜ |
| `test_conversation_id_serialize_deserialize` | JSON roundtrip preserves value | ⬜ |

### Conversation Entity Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_conversation_new_starts_empty` | New conversation has no messages | ⬜ |
| `test_conversation_new_has_default_agent_state` | Default state is set | ⬜ |
| `test_conversation_add_user_message_increments_count` | Count increases | ⬜ |
| `test_conversation_add_user_message_sets_role` | Role is User | ⬜ |
| `test_conversation_add_assistant_message_increments_count` | Count increases | ⬜ |
| `test_conversation_add_assistant_message_sets_role` | Role is Assistant | ⬜ |
| `test_conversation_add_system_message_sets_role` | Role is System | ⬜ |
| `test_conversation_last_message_returns_most_recent` | Returns last message | ⬜ |
| `test_conversation_last_message_returns_none_when_empty` | None for empty | ⬜ |
| `test_conversation_last_assistant_message_finds_correct` | Finds last assistant msg | ⬜ |
| `test_conversation_last_assistant_message_skips_user` | Skips user messages | ⬜ |
| `test_conversation_remove_last_assistant_removes_correct` | Removes right one | ⬜ |
| `test_conversation_remove_last_assistant_returns_none_if_none` | None when no assistant msg | ⬜ |
| `test_conversation_get_context_messages_limits_to_max` | Respects limit | ⬜ |
| `test_conversation_get_context_messages_returns_recent` | Returns most recent | ⬜ |
| `test_conversation_estimate_tokens_reasonable` | Token estimate works | ⬜ |
| `test_conversation_reconstitute_restores_all_fields` | Full restoration | ⬜ |
| `test_conversation_add_message_updates_timestamp` | Timestamp updates | ⬜ |

### AgentState Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_agent_state_default_has_empty_phase` | Default phase is empty | ⬜ |
| `test_agent_state_new_sets_phase` | Constructor sets phase | ⬜ |
| `test_agent_state_add_question_appends` | Questions append | ⬜ |
| `test_agent_state_next_question_pops_first` | FIFO order | ⬜ |
| `test_agent_state_next_question_returns_none_when_empty` | None for empty | ⬜ |
| `test_agent_state_increment_extracted_increases` | Count increases | ⬜ |
| `test_agent_state_serialize_roundtrip` | JSON roundtrip | ⬜ |
| `test_agent_state_phase_data_accepts_any_json` | Flexible phase data | ⬜ |

### AgentConfig Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_agent_config_for_component_returns_correct_type` | Returns matching type | ⬜ |
| `test_agent_config_for_all_nine_components_exists` | All 9 have configs | ⬜ |
| `test_agent_config_has_system_prompt` | System prompt is set | ⬜ |
| `test_agent_config_has_phases` | Phases are defined | ⬜ |
| `test_agent_config_has_extraction_rules` | Rules are defined | ⬜ |
| `test_agent_config_max_context_positive` | Context > 0 | ⬜ |
| `test_agent_config_temperature_in_range` | 0.0-2.0 range | ⬜ |

### AIProvider Port Tests (Mock Adapter)

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_mock_adapter_complete_returns_queued_response` | Returns queued | ⬜ |
| `test_mock_adapter_complete_returns_default_if_empty` | Default response | ⬜ |
| `test_mock_adapter_stream_returns_single_chunk` | Single chunk with full content | ⬜ |
| `test_mock_adapter_queue_multiple_responses` | Multiple responses work | ⬜ |

### OpenAI Adapter Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_openai_adapter_new_sets_defaults` | Default model, url | ⬜ |
| `test_openai_adapter_with_model_overrides` | Model override | ⬜ |
| `test_openai_adapter_formats_messages_correctly` | Message format | ⬜ |
| `test_openai_adapter_handles_rate_limit_response` | 429 handling | ⬜ |
| `test_openai_adapter_parses_response_content` | Content extraction | ⬜ |
| `test_openai_adapter_parses_token_usage` | Token counting | ⬜ |

### Anthropic Adapter Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_anthropic_adapter_new_sets_defaults` | Default model | ⬜ |
| `test_anthropic_adapter_formats_messages_correctly` | Message format | ⬜ |
| `test_anthropic_adapter_handles_rate_limit` | Rate limit handling | ⬜ |

### ConversationRepository Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_repository_save_persists_new` | New conversation persisted | ⬜ |
| `test_repository_save_generates_id_on_insert` | ID created | ⬜ |
| `test_repository_update_modifies_existing` | Updates work | ⬜ |
| `test_repository_find_by_component_returns_some` | Finds existing | ⬜ |
| `test_repository_find_by_component_returns_none` | None for missing | ⬜ |
| `test_repository_append_message_adds_to_existing` | Append works | ⬜ |
| `test_repository_append_message_preserves_order` | Order maintained | ⬜ |

### ConversationReader Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_reader_get_by_component_returns_view` | Returns ConversationView | ⬜ |
| `test_reader_get_by_component_returns_none` | None for missing | ⬜ |
| `test_reader_get_message_count_accurate` | Count is correct | ⬜ |
| `test_reader_get_recent_messages_respects_limit` | Limit respected | ⬜ |

### SendMessage Command Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_send_message_creates_conversation_if_none` | Auto-creates conversation | ⬜ |
| `test_send_message_adds_user_message` | User message added | ⬜ |
| `test_send_message_calls_ai_provider` | AI called | ⬜ |
| `test_send_message_adds_assistant_response` | Assistant message added | ⬜ |
| `test_send_message_persists_conversation` | Conversation saved | ⬜ |
| `test_send_message_returns_result_with_message` | Result has message | ⬜ |
| `test_send_message_returns_tokens_used` | Token count in result | ⬜ |
| `test_send_message_handles_ai_error` | AI errors propagate | ⬜ |
| `test_send_message_includes_component_state_in_prompt` | Context included | ⬜ |

### StreamMessage Command Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_stream_message_returns_stream` | Returns stream | ⬜ |
| `test_stream_message_chunks_accumulate` | Chunks build up | ⬜ |
| `test_stream_message_saves_final_message` | Final message saved | ⬜ |
| `test_stream_message_handles_ai_error` | Errors handled | ⬜ |

### RegenerateResponse Command Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_regenerate_removes_last_assistant` | Last assistant removed | ⬜ |
| `test_regenerate_calls_ai_provider` | AI called | ⬜ |
| `test_regenerate_adds_new_response` | New response added | ⬜ |
| `test_regenerate_fails_if_no_conversation` | Error if missing | ⬜ |

### GetConversation Query Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_get_conversation_returns_view` | Returns view | ⬜ |
| `test_get_conversation_returns_error_if_not_found` | Error for missing | ⬜ |

---

## Error Codes

| Error Code | Condition |
|------------|-----------|
| `CONVERSATION_NOT_FOUND` | Conversation does not exist for component |
| `AI_RATE_LIMITED` | AI provider rate limit exceeded |
| `AI_MODEL_UNAVAILABLE` | Requested model not available |
| `AI_CONTENT_FILTERED` | Response filtered by provider |
| `AI_TOKEN_LIMIT` | Token limit exceeded |
| `AI_NETWORK_ERROR` | Network error calling AI |
| `AI_PROVIDER_ERROR` | Generic provider error |
| `INVALID_MESSAGE_ROLE` | Unknown message role |
| `CONVERSATION_EMPTY` | No messages to regenerate |

---

## Business Rules

| Rule | Implementation | Test | Status |
|------|----------------|------|--------|
| One conversation per component | UNIQUE constraint | `test_repository_save_persists_new` | ⬜ |
| Messages are append-only | `append_message()` method | `test_repository_append_message_adds_to_existing` | ⬜ |
| Messages ordered by time | sequence_num column | `test_repository_append_message_preserves_order` | ⬜ |
| Valid message roles only | CHECK constraint | `test_conversation_add_user_message_sets_role` | ⬜ |
| Context window is limited | `get_context_messages(max)` | `test_conversation_get_context_messages_limits_to_max` | ⬜ |
| All 9 components have configs | `for_component()` match | `test_agent_config_for_all_nine_components_exists` | ⬜ |
| Rate limit errors are recoverable | `AIError::RateLimited` | `test_openai_adapter_handles_rate_limit_response` | ⬜ |

---

## Verification Commands

```bash
# Run all conversation tests
cargo test --package conversation -- --nocapture

# Run specific test category
cargo test --package conversation conversation:: -- --nocapture
cargo test --package conversation agent_state:: -- --nocapture
cargo test --package conversation agent_config:: -- --nocapture
cargo test --package conversation repository:: -- --nocapture
cargo test --package conversation commands:: -- --nocapture

# Run AI adapter tests
cargo test --package conversation openai:: -- --nocapture
cargo test --package conversation anthropic:: -- --nocapture

# Coverage check (target: 85%+)
cargo tarpaulin --package conversation --out Html

# Full verification
cargo test --package conversation -- --nocapture && cargo clippy --package conversation

# Frontend tests
cd frontend && npm test -- --testPathPattern="modules/conversation"
```

---

## Exit Criteria

### Module is COMPLETE when:

- [ ] All 75 files in File Inventory exist
- [ ] All 82 tests in Test Inventory pass
- [ ] Rust coverage >= 85%
- [ ] All 9 system prompts written
- [ ] OpenAI adapter handles rate limits
- [ ] Anthropic adapter handles rate limits
- [ ] WebSocket streaming works end-to-end
- [ ] Message persistence verified
- [ ] No clippy warnings
- [ ] No TypeScript lint errors

### Exit Signal

```
MODULE COMPLETE: conversation
Files: 75/75
Tests: 82/82 passing
Coverage: 87%
```

---

## Implementation Phases

### Phase 1: Core Domain
- [ ] ConversationId value object
- [ ] AgentState value object
- [ ] Conversation entity
- [ ] Core entity tests

### Phase 2: Agent Configuration
- [ ] AgentConfig struct
- [ ] AgentPhase, ExtractionRule
- [ ] Issue Raising config (first component)
- [ ] Config tests

### Phase 3: Ports
- [ ] AIProvider trait
- [ ] CompletionRequest, Response, Chunk
- [ ] AIError enum
- [ ] ConversationRepository trait
- [ ] ConversationReader trait

### Phase 4: Mock AI Adapter
- [ ] MockAIAdapter implementation
- [ ] Queue-based response system
- [ ] Mock adapter tests

### Phase 5: Commands & Queries
- [ ] SendMessageCommand + Handler
- [ ] GetConversationQuery + Handler
- [ ] RegenerateResponseCommand + Handler
- [ ] Command/Query tests (with mocks)

### Phase 6: Postgres Adapters
- [ ] Database migration
- [ ] PostgresConversationRepository
- [ ] PostgresConversationReader
- [ ] Integration tests

### Phase 7: OpenAI Adapter
- [ ] OpenAI HTTP client
- [ ] Message formatting
- [ ] Error handling (rate limits)
- [ ] OpenAI adapter tests

### Phase 8: Anthropic Adapter
- [ ] Anthropic HTTP client
- [ ] Message formatting
- [ ] Error handling
- [ ] Anthropic adapter tests

### Phase 9: Streaming
- [ ] StreamMessageCommand + Handler
- [ ] WebSocket handler
- [ ] Chunk accumulation
- [ ] Streaming tests

### Phase 10: Remaining Agent Configs
- [ ] ProblemFrame config + prompt
- [ ] Objectives config + prompt
- [ ] Alternatives config + prompt
- [ ] Consequences config + prompt
- [ ] Tradeoffs config + prompt
- [ ] Recommendation config + prompt
- [ ] DecisionQuality config + prompt
- [ ] NotesNextSteps config + prompt

### Phase 11: HTTP Layer
- [ ] Request/Response DTOs
- [ ] HTTP handlers
- [ ] Route definitions
- [ ] Handler tests

### Phase 12: Frontend
- [ ] TypeScript types
- [ ] API client
- [ ] Hooks (useConversation, useStreaming)
- [ ] React components
- [ ] Component tests

---

## Notes

- AI providers are behind a port for easy swapping/testing
- MockAIAdapter essential for fast unit tests
- System prompts should be refined iteratively
- Streaming uses WebSocket for real-time UX
- Token estimation is approximate (4 chars/token)
- Rate limit handling enables retry logic
- Each component has distinct prompts and extraction rules

---

*Generated: 2026-01-07*
*Specification: docs/modules/conversation.md*
