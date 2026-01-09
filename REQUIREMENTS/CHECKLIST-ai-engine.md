# AI Engine Module Checklist

**Module:** AI Engine
**Language:** Rust
**Dependencies:** foundation, proact-types, session, cycle
**Phase:** 3 (parallel with cycle, conversation, analysis)

---

## Overview

The AI Engine module provides conversational AI capabilities for guiding users through PrOACT decision components. Designed as a port-based abstraction enabling multiple AI backends (Claude Code, OpenAI API, Anthropic API) to be swapped without affecting domain logic.

---

## File Inventory

### Domain Layer

| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/ai_engine/mod.rs` | Module exports | ⬜ |
| `backend/src/domain/ai_engine/orchestrator.rs` | PrOACT flow management | ⬜ |
| `backend/src/domain/ai_engine/step_agent.rs` | Step agent specifications | ⬜ |
| `backend/src/domain/ai_engine/conversation_state.rs` | Conversation state tracking | ⬜ |
| `backend/src/domain/ai_engine/values.rs` | Value objects (UserIntent, StepSummary, etc.) | ⬜ |
| `backend/src/domain/ai_engine/services.rs` | Domain services (IntentClassifier, etc.) | ⬜ |
| `backend/src/domain/ai_engine/errors.rs` | Domain errors | ⬜ |

### Ports

| File | Description | Status |
|------|-------------|--------|
| `backend/src/ports/ai_engine.rs` | AIEnginePort trait definition | ⬜ |
| `backend/src/ports/step_agent.rs` | StepAgentPort trait definition | ⬜ |
| `backend/src/ports/state_storage.rs` | StateStoragePort trait definition | ⬜ |

### Application Layer

| File | Description | Status |
|------|-------------|--------|
| `backend/src/application/commands/start_conversation.rs` | Start conversation command | ⬜ |
| `backend/src/application/commands/send_message.rs` | Send message command | ⬜ |
| `backend/src/application/commands/transition_step.rs` | Transition step command | ⬜ |
| `backend/src/application/commands/complete_step.rs` | Complete step command | ⬜ |
| `backend/src/application/commands/branch_cycle.rs` | Branch cycle command | ⬜ |
| `backend/src/application/queries/get_conversation_state.rs` | Get state query | ⬜ |
| `backend/src/application/queries/get_step_output.rs` | Get step output query | ⬜ |
| `backend/src/application/queries/search_decisions.rs` | Search decisions query | ⬜ |

### Adapters - Claude Code

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/claude_code/mod.rs` | Module exports | ⬜ |
| `backend/src/adapters/claude_code/adapter.rs` | ClaudeCodeAdapter implementation | ⬜ |
| `backend/src/adapters/claude_code/process_manager.rs` | Process lifecycle management | ⬜ |
| `backend/src/adapters/claude_code/stream_parser.rs` | Parse stdout into StreamChunks | ⬜ |

### Adapters - OpenAI

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/openai/mod.rs` | Module exports | ⬜ |
| `backend/src/adapters/openai/adapter.rs` | OpenAIAdapter implementation | ⬜ |
| `backend/src/adapters/openai/message_builder.rs` | Build chat completion messages | ⬜ |
| `backend/src/adapters/openai/stream_transformer.rs` | Transform SSE to StreamChunks | ⬜ |

### Adapters - Anthropic

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/anthropic/mod.rs` | Module exports | ⬜ |
| `backend/src/adapters/anthropic/adapter.rs` | AnthropicAdapter implementation | ⬜ |
| `backend/src/adapters/anthropic/message_builder.rs` | Build Anthropic messages | ⬜ |
| `backend/src/adapters/anthropic/stream_transformer.rs` | Transform SSE to StreamChunks | ⬜ |

### Adapters - Storage

| File | Description | Status |
|------|-------------|--------|
| `backend/src/adapters/storage/mod.rs` | Module exports | ⬜ |
| `backend/src/adapters/storage/file_storage.rs` | FileStorageAdapter implementation | ⬜ |
| `backend/src/adapters/storage/hybrid_storage.rs` | HybridStorageAdapter implementation | ⬜ |
| `backend/src/adapters/storage/yaml_schemas.rs` | Serde types for YAML files | ⬜ |

### Claude Code Skills

| File | Description | Status |
|------|-------------|--------|
| `.claude/skills/decision-orchestrate.md` | Main orchestrator skill | ⬜ |
| `.claude/agents/proact-issue-raising.md` | Issue raising agent | ⬜ |
| `.claude/agents/proact-problem-frame.md` | Problem frame agent | ⬜ |
| `.claude/agents/proact-objectives.md` | Objectives agent | ⬜ |
| `.claude/agents/proact-alternatives.md` | Alternatives agent | ⬜ |
| `.claude/agents/proact-consequences.md` | Consequences agent | ⬜ |
| `.claude/agents/proact-tradeoffs.md` | Tradeoffs agent | ⬜ |
| `.claude/agents/proact-recommendation.md` | Recommendation agent | ⬜ |
| `.claude/agents/proact-decision-quality.md` | Decision quality agent | ⬜ |

### Configuration

| File | Description | Status |
|------|-------------|--------|
| `config/ai-engine.yaml` | AI engine configuration | ⬜ |
| `backend/src/config/ai_engine.rs` | Config structs and parsing | ⬜ |

---

## Test Inventory

### Domain Layer Tests

#### Orchestrator Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_orchestrator_new_creates_with_initial_step` | Create new orchestrator | ⬜ |
| `test_orchestrator_from_state_restores_correctly` | Resume from persisted state | ⬜ |
| `test_orchestrator_route_continue_returns_current` | Continue intent returns current step | ⬜ |
| `test_orchestrator_route_navigate_returns_target` | Navigate intent returns target step | ⬜ |
| `test_orchestrator_can_transition_valid_progression` | Valid forward transitions allowed | ⬜ |
| `test_orchestrator_can_transition_invalid_skip` | Cannot skip multiple steps | ⬜ |
| `test_orchestrator_transition_to_updates_current` | Transition updates current step | ⬜ |
| `test_orchestrator_transition_to_invalid_returns_error` | Invalid transition returns error | ⬜ |
| `test_orchestrator_record_completion_advances_step` | Completion advances to next step | ⬜ |
| `test_orchestrator_record_completion_stores_summary` | Completion stores summary | ⬜ |
| `test_orchestrator_context_for_step_includes_prior` | Context includes prior summaries | ⬜ |
| `test_orchestrator_to_state_exports_correctly` | State export is complete | ⬜ |

#### StepAgent Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_step_agent_spec_all_components_defined` | All 8 PrOACT agents defined | ⬜ |
| `test_step_agent_spec_has_required_fields` | Specs have role, objectives, etc. | ⬜ |
| `test_transition_rules_validate_correctly` | Transition rules work | ⬜ |

#### ConversationState Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_conversation_state_new_initializes_correctly` | New state has defaults | ⬜ |
| `test_conversation_state_serializes_to_yaml` | YAML serialization works | ⬜ |
| `test_conversation_state_deserializes_from_yaml` | YAML deserialization works | ⬜ |
| `test_step_state_status_transitions` | Status transitions valid | ⬜ |
| `test_message_creates_with_timestamp` | Messages have timestamps | ⬜ |
| `test_branch_info_stores_parent_correctly` | Branch info preserved | ⬜ |

#### Value Objects Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_user_intent_variants` | All intent variants work | ⬜ |
| `test_step_summary_fields` | Summary has all fields | ⬜ |
| `test_cycle_status_enum` | Cycle status transitions | ⬜ |

### Port Contract Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_ai_engine_port_contract_start_session` | Start session works | ⬜ |
| `test_ai_engine_port_contract_send_message` | Send message streams | ⬜ |
| `test_ai_engine_port_contract_get_state` | Get state returns current | ⬜ |
| `test_ai_engine_port_contract_end_session` | End session cleans up | ⬜ |
| `test_state_storage_port_contract_save_load` | Save/load roundtrip | ⬜ |
| `test_state_storage_port_contract_step_output` | Step output persistence | ⬜ |
| `test_state_storage_port_contract_messages` | Message history | ⬜ |
| `test_step_agent_port_contract_prompts` | Prompt generation | ⬜ |
| `test_step_agent_port_contract_tools` | Tool definitions | ⬜ |
| `test_step_agent_port_contract_parsing` | Response parsing | ⬜ |

### Adapter Tests - Claude Code

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_claude_code_adapter_start_session` | Spawns process correctly | ⬜ |
| `test_claude_code_adapter_send_message` | Writes to stdin | ⬜ |
| `test_claude_code_adapter_stream_output` | Streams stdout | ⬜ |
| `test_claude_code_adapter_end_session` | Kills process | ⬜ |
| `test_process_manager_spawn` | Spawns with correct args | ⬜ |
| `test_process_manager_send` | Sends to stdin | ⬜ |
| `test_process_manager_kill` | Terminates process | ⬜ |
| `test_process_manager_cleanup_idle` | Removes idle sessions | ⬜ |
| `test_stream_parser_text_chunks` | Parses text output | ⬜ |
| `test_stream_parser_tool_calls` | Parses tool calls | ⬜ |
| `test_stream_parser_step_complete` | Detects step completion | ⬜ |

### Adapter Tests - OpenAI

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_openai_adapter_start_session` | Creates session handle | ⬜ |
| `test_openai_adapter_build_messages` | Builds message array | ⬜ |
| `test_openai_adapter_send_message_streams` | Streams SSE response | ⬜ |
| `test_openai_adapter_handles_function_calls` | Processes function calls | ⬜ |
| `test_message_builder_system_prompt` | System prompt included | ⬜ |
| `test_message_builder_history` | History included | ⬜ |
| `test_stream_transformer_text` | Transforms text deltas | ⬜ |
| `test_stream_transformer_function` | Transforms function calls | ⬜ |

### Adapter Tests - Anthropic

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_anthropic_adapter_start_session` | Creates session handle | ⬜ |
| `test_anthropic_adapter_build_messages` | Builds Anthropic format | ⬜ |
| `test_anthropic_adapter_send_message_streams` | Streams response | ⬜ |
| `test_anthropic_adapter_handles_tool_use` | Processes tool_use blocks | ⬜ |
| `test_message_builder_system` | System block correct | ⬜ |
| `test_message_builder_alternating` | User/assistant alternating | ⬜ |

### Adapter Tests - Storage

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_file_storage_save_state` | Saves state.yaml | ⬜ |
| `test_file_storage_load_state` | Loads state.yaml | ⬜ |
| `test_file_storage_state_not_found` | Returns NotFound error | ⬜ |
| `test_file_storage_save_step_output` | Saves numbered yaml | ⬜ |
| `test_file_storage_load_step_output` | Loads step output | ⬜ |
| `test_file_storage_append_message` | Appends to history | ⬜ |
| `test_file_storage_get_messages_limit` | Respects limit | ⬜ |
| `test_file_storage_creates_directories` | Creates parent dirs | ⬜ |
| `test_hybrid_storage_writes_both` | Writes file and indexes | ⬜ |
| `test_hybrid_storage_search` | Full-text search works | ⬜ |
| `test_hybrid_storage_sync_from_files` | Rebuilds index | ⬜ |

### Application Layer Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_start_conversation_handler_success` | Happy path | ⬜ |
| `test_start_conversation_handler_cycle_not_found` | 404 case | ⬜ |
| `test_start_conversation_handler_provider_unavailable` | Provider error | ⬜ |
| `test_send_message_handler_streams_response` | Streams chunks | ⬜ |
| `test_send_message_handler_saves_history` | Saves messages | ⬜ |
| `test_transition_step_handler_valid` | Valid transition | ⬜ |
| `test_transition_step_handler_invalid` | Invalid transition error | ⬜ |
| `test_complete_step_handler_advances` | Advances step | ⬜ |
| `test_complete_step_handler_saves_output` | Saves structured output | ⬜ |
| `test_branch_cycle_handler_creates_branch` | Creates new cycle | ⬜ |
| `test_get_conversation_state_returns_current` | Returns state | ⬜ |
| `test_get_step_output_returns_output` | Returns output | ⬜ |
| `test_search_decisions_returns_results` | Search works | ⬜ |

### Integration Tests

| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_claude_code_integration_full_cycle` | Full session with CLI | ⬜ |
| `test_openai_integration_full_cycle` | Full session with OpenAI | ⬜ |
| `test_anthropic_integration_full_cycle` | Full session with Anthropic | ⬜ |
| `test_provider_switch_mid_cycle` | Switch providers works | ⬜ |
| `test_file_storage_integration` | File operations work | ⬜ |
| `test_hybrid_storage_integration` | Hybrid with Postgres works | ⬜ |

---

## Error Codes

| Error Code | HTTP Status | Condition |
|------------|-------------|-----------|
| `SESSION_NOT_FOUND` | 404 | Session does not exist |
| `PROVIDER_ERROR` | 502 | AI provider returned error |
| `CONNECTION_FAILED` | 503 | Cannot connect to provider |
| `TIMEOUT` | 504 | Provider request timed out |
| `INVALID_STATE` | 400 | Invalid state transition |
| `CYCLE_NOT_FOUND` | 404 | Cycle does not exist |
| `PROVIDER_UNAVAILABLE` | 503 | Requested provider not enabled |
| `STATE_NOT_FOUND` | 404 | No state for cycle |
| `SERIALIZATION_ERROR` | 500 | YAML/JSON serialization failed |
| `DATABASE_ERROR` | 500 | Database operation failed |

---

## Business Rules

| Rule | Implementation | Test | Status |
|------|----------------|------|--------|
| PrOACT steps must progress in order | `Orchestrator::can_transition` | `test_orchestrator_can_transition_valid_progression` | ⬜ |
| Only one step can be in_progress | `Orchestrator::transition_to` | `test_orchestrator_transition_to_updates_current` | ⬜ |
| Completed steps preserve summaries | `Orchestrator::record_completion` | `test_orchestrator_record_completion_stores_summary` | ⬜ |
| Branch inherits parent history | `BranchInfo::parent_cycle` | `test_branch_info_stores_parent_correctly` | ⬜ |
| File storage is source of truth | `HybridStorageAdapter::save_step_output` | `test_hybrid_storage_writes_both` | ⬜ |
| Context compression at threshold | `ContextCompressor::compress` | TBD | ⬜ |
| Idle sessions cleaned up | `ProcessManager::cleanup_idle` | `test_process_manager_cleanup_idle` | ⬜ |

---

## Verification Commands

```bash
# Domain tests
cargo test --package ai-engine domain:: -- --nocapture

# Port contract tests
cargo test --package ai-engine ports:: -- --nocapture

# Adapter tests (unit)
cargo test --package ai-engine adapters:: -- --nocapture

# Claude Code adapter (requires CLI)
cargo test --package ai-engine adapters::claude_code:: --features claude-code -- --ignored

# OpenAI adapter (requires API key)
cargo test --package ai-engine adapters::openai:: --features openai -- --ignored

# Anthropic adapter (requires API key)
cargo test --package ai-engine adapters::anthropic:: --features anthropic -- --ignored

# File storage integration
cargo test --package ai-engine adapters::storage::file:: -- --nocapture

# Hybrid storage integration (requires Postgres)
cargo test --package ai-engine adapters::storage::hybrid:: -- --ignored

# Application layer tests
cargo test --package ai-engine application:: -- --nocapture

# Coverage check (target: 85%+)
cargo tarpaulin --package ai-engine --out Html

# Full verification
cargo test --package ai-engine -- --nocapture && cargo clippy --package ai-engine
```

---

## Exit Criteria

### Module is COMPLETE when:

- [ ] All 43 files in File Inventory exist
- [ ] All 83 tests in Test Inventory pass
- [ ] Domain layer coverage >= 90%
- [ ] Application layer coverage >= 85%
- [ ] Adapter layer coverage >= 80%
- [ ] All port contracts verified with at least one adapter
- [ ] Claude Code adapter works with real CLI
- [ ] At least one API adapter (OpenAI or Anthropic) works
- [ ] File storage works end-to-end
- [ ] Configuration loading works
- [ ] No clippy warnings

### Exit Signal

```
MODULE COMPLETE: ai-engine
Files: 43/43
Tests: 83/83 passing
Coverage: Domain 92%, Application 87%, Adapters 82%
```

---

## Implementation Phases

### Phase 1: Domain & Ports (Foundation)
- [ ] Domain types (Orchestrator, StepAgent, ConversationState)
- [ ] Port traits (AIEnginePort, StepAgentPort, StateStoragePort)
- [ ] Domain services (IntentClassifier, ContextCompressor)
- [ ] Domain layer tests

### Phase 2: File Storage Adapter
- [ ] FileStorageAdapter implementation
- [ ] YAML schema serde types
- [ ] Storage contract tests
- [ ] File integration tests

### Phase 3: Claude Code Adapter
- [ ] ProcessManager implementation
- [ ] Orchestrator skill definition
- [ ] Step agent definitions (8 agents)
- [ ] Stream parser
- [ ] Claude Code integration tests

### Phase 4: API Adapters
- [ ] OpenAI adapter implementation
- [ ] Anthropic adapter implementation
- [ ] Adapter factory pattern
- [ ] API contract tests

### Phase 5: Hybrid Storage
- [ ] Database schema for indexing
- [ ] HybridStorageAdapter implementation
- [ ] Sync mechanism
- [ ] Search capability
- [ ] Database integration tests

### Phase 6: Application Layer
- [ ] Command handlers
- [ ] Query handlers
- [ ] Dependency injection wiring
- [ ] End-to-end tests

---

## Notes

- Claude Code adapter uses `tokio::process` for async process management
- OpenAI adapter uses `async-openai` crate
- Anthropic adapter may need custom implementation (no official Rust SDK)
- Feature flags control which adapters are compiled
- File storage uses YAML for human readability
- Database indexing uses PostgreSQL full-text search

---

*Generated: 2026-01-07*
*Specification: docs/modules/ai-engine.md*
