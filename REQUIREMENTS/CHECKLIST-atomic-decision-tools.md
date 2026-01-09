# Atomic Decision Tools Implementation Checklist

**Feature:** Atomic Decision Tools with Emergent Composition
**Module:** conversation
**Priority:** P1 (Phase 1 of Agent-Native Enrichments)
**Specification:** [features/conversation/atomic-decision-tools.md](../features/conversation/atomic-decision-tools.md)
**Created:** 2026-01-09

---

## Overview

This checklist tracks implementation of Atomic Decision Tools - fine-grained primitives that the AI agent can invoke to directly manipulate decision documents and component state, enabling emergent behaviors while optimizing token usage.

### Key Components

```
┌─────────────────────────────────────────────────────────────────┐
│                    ATOMIC DECISION TOOLS                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   ┌──────────────────────────────────────────────────────┐      │
│   │                  Component Tools                       │      │
│   │  Issue Raising │ Problem Frame │ Objectives │ Alts    │      │
│   │  Consequences  │ Tradeoffs     │ Recommend  │ DQ      │      │
│   └──────────────────────────────────────────────────────┘      │
│                                                                  │
│   ┌──────────────────────────────────────────────────────┐      │
│   │              Cross-Cutting Tools                       │      │
│   │  Uncertainty │ Revisit Suggestions │ Confirmations    │      │
│   └──────────────────────────────────────────────────────┘      │
│                                                                  │
│   ┌──────────────────────────────────────────────────────┐      │
│   │                 Analysis Tools                         │      │
│   │  Pugh Totals │ Dominated Detection │ Sensitivity      │      │
│   └──────────────────────────────────────────────────────┘      │
│                                                                  │
│   ┌──────────────────────────────────────────────────────┐      │
│   │                  Infrastructure                        │      │
│   │  ToolExecutor │ Registry │ Audit Log │ AI Integration │      │
│   └──────────────────────────────────────────────────────┘      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Core Infrastructure

### Domain Layer - Tool Framework

- [ ] `domain/conversation/tools/mod.rs` - Tools module setup
  - [ ] Module exports
  - [ ] Re-exports for public API

- [ ] `domain/conversation/tools/tool_invocation.rs` - Audit entity
  - [ ] ToolInvocationId value object
  - [ ] ToolInvocation entity
  - [ ] ToolResult enum (Success, ValidationError, NotFound, Conflict, InternalError)
  - [ ] Timing and context fields
  - [ ] Constructor and accessors
  - [ ] Unit tests (5+ tests)

- [ ] `domain/conversation/tools/revisit_suggestion.rs` - Revisit queue
  - [ ] RevisitSuggestionId value object
  - [ ] RevisitSuggestion entity
  - [ ] RevisitPriority enum (Low, Medium, High, Critical)
  - [ ] SuggestionStatus enum (Pending, Accepted, Dismissed, Expired)
  - [ ] Unit tests (5+ tests)

- [ ] `domain/conversation/tools/confirmation_request.rs` - User confirmations
  - [ ] ConfirmationRequestId value object
  - [ ] ConfirmationRequest entity
  - [ ] ConfirmationOption struct
  - [ ] ConfirmationStatus enum
  - [ ] Expiration logic
  - [ ] Unit tests (5+ tests)

### Domain Events

- [ ] `domain/conversation/tools/events.rs` - Tool events
  - [ ] ToolInvoked event
  - [ ] ToolCompleted event
  - [ ] ObjectiveAdded event
  - [ ] AlternativeAdded event
  - [ ] ConsequenceRated event
  - [ ] DominatedMarked event
  - [ ] UncertaintyFlagged event
  - [ ] RevisitSuggested event
  - [ ] ConfirmationRequested event
  - [ ] ConfirmationResolved event
  - [ ] Unit tests for event serialization

---

## Phase 2: Tool Definitions & Registry

### Tool Definition Types

- [ ] `domain/conversation/tools/definition.rs` - Tool definition types
  - [ ] ToolDefinition struct (name, description, parameters_schema, returns_schema)
  - [ ] JSON Schema validation support
  - [ ] to_openai_format() method
  - [ ] to_anthropic_format() method
  - [ ] Unit tests

### Tool Execution Context

- [ ] `domain/conversation/tools/context.rs` - Execution context
  - [ ] ToolExecutionContext struct (minimal context)
  - [ ] ToolCall struct (name, parameters)
  - [ ] ToolResponse struct (success, data, error, document_updated, suggestions)
  - [ ] Unit tests

### Tool Registry

- [ ] `domain/conversation/tools/registry.rs` - Tool registry
  - [ ] ToolRegistry struct
  - [ ] register_tool() method
  - [ ] tools_for_component() method
  - [ ] validate_parameters() method
  - [ ] Unit tests (10+ tests)

---

## Phase 3: Component-Specific Tools - Part 1

### Issue Raising Tools

- [ ] `domain/conversation/tools/issue_raising.rs` - Issue Raising tools
  - [ ] AddPotentialDecision tool
  - [ ] AddObjectiveIdea tool
  - [ ] AddUncertainty tool
  - [ ] AddConsideration tool
  - [ ] SetFocalDecision tool
  - [ ] Parameter validation for each
  - [ ] Unit tests (10+ tests)

### Problem Frame Tools

- [ ] `domain/conversation/tools/problem_frame.rs` - Problem Frame tools
  - [ ] SetDecisionMaker tool
  - [ ] SetFocalStatement tool
  - [ ] SetScope tool
  - [ ] AddConstraint tool
  - [ ] AddParty tool
  - [ ] SetDeadline tool
  - [ ] AddHierarchyDecision tool
  - [ ] Parameter validation for each
  - [ ] Unit tests (12+ tests)

### Objectives Tools

- [ ] `domain/conversation/tools/objectives.rs` - Objectives tools
  - [ ] AddObjective tool (with direction: higher/lower/target)
  - [ ] LinkMeansToFundamental tool
  - [ ] UpdateObjectiveMeasure tool
  - [ ] RemoveObjective tool (with logging)
  - [ ] PromoteToFundamental tool
  - [ ] Parameter validation for each
  - [ ] Unit tests (10+ tests)

### Alternatives Tools

- [ ] `domain/conversation/tools/alternatives.rs` - Alternatives tools
  - [ ] AddAlternative tool (with status_quo flag)
  - [ ] UpdateAlternative tool
  - [ ] RemoveAlternative tool (with logging)
  - [ ] AddStrategyDimension tool
  - [ ] SetAlternativeStrategy tool
  - [ ] Parameter validation for each
  - [ ] Unit tests (10+ tests)

---

## Phase 4: Component-Specific Tools - Part 2

### Consequences Tools

- [ ] `domain/conversation/tools/consequences.rs` - Consequences tools
  - [ ] RateConsequence tool (single cell)
  - [ ] BatchRateConsequences tool (multiple cells)
  - [ ] AddConsequenceUncertainty tool
  - [ ] UpdateRatingReasoning tool
  - [ ] PughRating enum (-2 to +2)
  - [ ] Parameter validation for each
  - [ ] Unit tests (10+ tests)

### Tradeoffs Tools

- [ ] `domain/conversation/tools/tradeoffs.rs` - Tradeoffs tools
  - [ ] MarkDominated tool
  - [ ] MarkIrrelevantObjective tool
  - [ ] AddTension tool
  - [ ] ClearDominated tool
  - [ ] Parameter validation for each
  - [ ] Unit tests (8+ tests)

### Recommendation Tools

- [ ] `domain/conversation/tools/recommendation.rs` - Recommendation tools
  - [ ] SetSynthesis tool
  - [ ] SetStandout tool (with optional alternative)
  - [ ] AddKeyConsideration tool
  - [ ] AddRemainingUncertainty tool
  - [ ] Parameter validation for each
  - [ ] Unit tests (8+ tests)

### Decision Quality Tools

- [ ] `domain/conversation/tools/decision_quality.rs` - DQ tools
  - [ ] RateDQElement tool
  - [ ] DQElement enum (7 elements)
  - [ ] AddQualityImprovement tool
  - [ ] CalculateOverallDQ tool (min of elements)
  - [ ] Parameter validation for each
  - [ ] Unit tests (8+ tests)

---

## Phase 5: Cross-Cutting Tools

### Uncertainty Management

- [ ] `domain/conversation/tools/uncertainty.rs` - Uncertainty tools
  - [ ] FlagUncertainty tool (any component)
  - [ ] ResolveUncertainty tool
  - [ ] ListUncertainties tool (with filter)
  - [ ] Parameter validation for each
  - [ ] Unit tests (6+ tests)

### Revisit Suggestions

- [ ] `domain/conversation/tools/revisit.rs` - Revisit tools
  - [ ] SuggestRevisit tool (queued, not immediate)
  - [ ] GetPendingRevisits tool
  - [ ] DismissRevisit tool
  - [ ] Linear flow compliance checks
  - [ ] Parameter validation for each
  - [ ] Unit tests (8+ tests)

### User Confirmation

- [ ] `domain/conversation/tools/confirmation.rs` - Confirmation tools
  - [ ] RequestConfirmation tool
  - [ ] RecordUserChoice tool
  - [ ] Expiration handling
  - [ ] Parameter validation for each
  - [ ] Unit tests (6+ tests)

### Document Operations

- [ ] `domain/conversation/tools/document_ops.rs` - Document tools
  - [ ] GetDocumentSection tool
  - [ ] GetDocumentSummary tool (minimal context)
  - [ ] AddNote tool
  - [ ] Parameter validation for each
  - [ ] Unit tests (6+ tests)

---

## Phase 6: Analysis Tools

### Pugh Matrix Analysis

- [ ] `domain/conversation/tools/analysis.rs` - Analysis tools
  - [ ] ComputePughTotals tool
  - [ ] AlternativeScore struct
  - [ ] PughTotalsResult struct
  - [ ] Unit tests (5+ tests)

### Dominance Detection

- [ ] FindDominatedAlternatives tool
  - [ ] DominatedInfo struct
  - [ ] Dominance detection algorithm
  - [ ] Unit tests (5+ tests)

### Irrelevant Objective Detection

- [ ] FindIrrelevantObjectives tool
  - [ ] Non-differentiating detection algorithm
  - [ ] Unit tests (3+ tests)

### Sensitivity Analysis

- [ ] SensitivityCheck tool
  - [ ] SensitivityResult struct
  - [ ] What-if analysis for single cell
  - [ ] Ranking change detection
  - [ ] Unit tests (5+ tests)

---

## Phase 7: Ports Layer

### ToolExecutor Port

- [ ] `ports/tool_executor.rs` - Tool executor port
  - [ ] ToolExecutor trait
  - [ ] execute() method signature
  - [ ] available_tools() method signature
  - [ ] validate() method signature
  - [ ] ToolError enum

### Repository Ports

- [ ] `ports/tool_invocation_repository.rs` - Invocation repository port
  - [ ] ToolInvocationRepository trait
  - [ ] save() method
  - [ ] find_by_cycle() method
  - [ ] find_by_component() method
  - [ ] find_by_id() method

- [ ] `ports/revisit_suggestion_repository.rs` - Revisit repository port
  - [ ] RevisitSuggestionRepository trait
  - [ ] save() method
  - [ ] update() method
  - [ ] find_pending() method
  - [ ] find_by_id() method

- [ ] `ports/confirmation_request_repository.rs` - Confirmation repository port
  - [ ] ConfirmationRequestRepository trait
  - [ ] save() method
  - [ ] update() method
  - [ ] find_pending() method
  - [ ] expire_old() method

---

## Phase 8: Application Layer

### Commands

- [ ] `application/commands/execute_tool.rs` - Execute tool command
  - [ ] ExecuteToolCommand struct
  - [ ] ExecuteToolHandler struct
  - [ ] Authorization check
  - [ ] Context building (minimal)
  - [ ] Tool validation and execution
  - [ ] Invocation logging
  - [ ] Document section update trigger
  - [ ] Event publishing
  - [ ] Unit tests (10+ tests)

- [ ] `application/commands/resolve_revisit.rs` - Resolve revisit command
  - [ ] ResolveRevisitCommand struct
  - [ ] ResolveRevisitHandler struct
  - [ ] Accept/dismiss logic
  - [ ] Event publishing
  - [ ] Unit tests (5+ tests)

- [ ] `application/commands/respond_confirmation.rs` - Respond to confirmation
  - [ ] RespondToConfirmationCommand struct
  - [ ] RespondToConfirmationHandler struct
  - [ ] Choice recording
  - [ ] Event publishing
  - [ ] Unit tests (5+ tests)

### Queries

- [ ] `application/queries/get_available_tools.rs` - Available tools query
  - [ ] GetAvailableToolsQuery struct
  - [ ] GetAvailableToolsHandler struct
  - [ ] Component-based tool filtering
  - [ ] Unit tests (3+ tests)

- [ ] `application/queries/get_tool_invocations.rs` - Invocation history query
  - [ ] GetToolInvocationsQuery struct
  - [ ] GetToolInvocationsHandler struct
  - [ ] Filtering and pagination
  - [ ] Unit tests (3+ tests)

- [ ] `application/queries/get_pending_revisits.rs` - Pending revisits query
  - [ ] GetPendingRevisitsQuery struct
  - [ ] GetPendingRevisitsHandler struct
  - [ ] Priority sorting
  - [ ] Unit tests (3+ tests)

- [ ] `application/queries/get_pending_confirmations.rs` - Pending confirmations
  - [ ] GetPendingConfirmationsQuery struct
  - [ ] GetPendingConfirmationsHandler struct
  - [ ] Expiration filtering
  - [ ] Unit tests (3+ tests)

---

## Phase 9: HTTP Adapter

### Endpoints

- [ ] `adapters/http/tools.rs` - Tool endpoints
  - [ ] POST `/api/cycles/:id/tools/:name` - Execute tool
  - [ ] GET `/api/cycles/:id/tools` - List available tools
  - [ ] GET `/api/cycles/:id/tools/invocations` - Get invocation history

- [ ] `adapters/http/revisits.rs` - Revisit endpoints
  - [ ] GET `/api/cycles/:id/revisits` - List pending revisits
  - [ ] POST `/api/cycles/:id/revisits/:id/resolve` - Accept/dismiss

- [ ] `adapters/http/confirmations.rs` - Confirmation endpoints
  - [ ] GET `/api/cycles/:id/confirmations/pending` - List pending
  - [ ] POST `/api/cycles/:id/confirmations/:id/respond` - Respond

### DTOs

- [ ] `adapters/http/tools/dto.rs` - Request/Response DTOs
  - [ ] ExecuteToolRequest
  - [ ] ExecuteToolResponse
  - [ ] AvailableToolsResponse
  - [ ] ToolDefinitionDto
  - [ ] InvocationHistoryResponse
  - [ ] PendingRevisitsResponse
  - [ ] RevisitSuggestionDto
  - [ ] PendingConfirmationsResponse
  - [ ] ConfirmationDto

### Route Configuration

- [ ] `adapters/http/routes.rs` - Route registration
  - [ ] Tool routes
  - [ ] Revisit routes
  - [ ] Confirmation routes
  - [ ] Integration tests (10+ tests)

---

## Phase 10: Database Adapter

### Migrations

- [ ] `migrations/YYYYMMDD_create_tool_invocations.sql`
  - [ ] tool_invocations table
  - [ ] Indexes for cycle, component, tool_name, time
  - [ ] Constraints for result enum

- [ ] `migrations/YYYYMMDD_create_revisit_suggestions.sql`
  - [ ] revisit_suggestions table
  - [ ] Indexes for cycle, status
  - [ ] Constraints for priority and status enums

- [ ] `migrations/YYYYMMDD_create_confirmation_requests.sql`
  - [ ] confirmation_requests table
  - [ ] Indexes for cycle, pending status
  - [ ] Constraints for status enum

### Repository Implementations

- [ ] `adapters/postgres/tool_invocation_repository.rs`
  - [ ] PostgresToolInvocationRepository struct
  - [ ] CRUD operations
  - [ ] Query with filters
  - [ ] Integration tests (5+ tests)

- [ ] `adapters/postgres/revisit_suggestion_repository.rs`
  - [ ] PostgresRevisitSuggestionRepository struct
  - [ ] CRUD operations
  - [ ] Pending query with priority sort
  - [ ] Integration tests (5+ tests)

- [ ] `adapters/postgres/confirmation_request_repository.rs`
  - [ ] PostgresConfirmationRequestRepository struct
  - [ ] CRUD operations
  - [ ] Expiration handling
  - [ ] Integration tests (5+ tests)

---

## Phase 11: AI Provider Integration

### Tool Format Conversion

- [ ] `adapters/ai/tools/openai.rs` - OpenAI format
  - [ ] Convert ToolDefinition to OpenAI function format
  - [ ] Handle tool responses
  - [ ] Unit tests

- [ ] `adapters/ai/tools/anthropic.rs` - Anthropic format
  - [ ] Convert ToolDefinition to Anthropic tool_use format
  - [ ] Handle tool responses
  - [ ] Unit tests

### Conversation Flow Integration

- [ ] `adapters/ai/tools/integration.rs` - AI integration
  - [ ] Inject tools into conversation context
  - [ ] Parse tool calls from AI response
  - [ ] Execute tools and return results
  - [ ] Handle multi-turn tool use
  - [ ] Unit tests (10+ tests)

### Token Optimization

- [ ] Context injection optimization
  - [ ] DocumentSummary generation for minimal context
  - [ ] ID-only reference passing
  - [ ] Batch operation support
  - [ ] Performance benchmarks

---

## Phase 12: ToolExecutor Implementation

### Core Implementation

- [ ] `adapters/tool_executor.rs` - Tool executor implementation
  - [ ] ToolExecutorImpl struct
  - [ ] Tool dispatch logic
  - [ ] Parameter validation
  - [ ] Document update coordination
  - [ ] Error handling

### Tool Handlers

- [ ] Issue Raising tool handlers
- [ ] Problem Frame tool handlers
- [ ] Objectives tool handlers
- [ ] Alternatives tool handlers
- [ ] Consequences tool handlers
- [ ] Tradeoffs tool handlers
- [ ] Recommendation tool handlers
- [ ] Decision Quality tool handlers
- [ ] Cross-cutting tool handlers
- [ ] Analysis tool handlers

### Integration with Document Service

- [ ] Section update triggers
  - [ ] Tool → Component state update
  - [ ] Component state → Document section regeneration
  - [ ] Delta update (not full regeneration)
  - [ ] Unit tests

---

## Phase 13: Frontend (Optional - API-First)

### Tool Visualization

- [ ] `frontend/src/lib/tools/ToolHistory.svelte`
  - [ ] Display tool invocation history
  - [ ] Show tool parameters and results
  - [ ] Filter by component

### Revisit Management

- [ ] `frontend/src/lib/tools/RevisitSuggestions.svelte`
  - [ ] Display pending revisits
  - [ ] Accept/dismiss actions
  - [ ] Priority indicators

### Confirmation UI

- [ ] `frontend/src/lib/tools/ConfirmationModal.svelte`
  - [ ] Display confirmation requests
  - [ ] Option selection
  - [ ] Custom input support

---

## Phase 14: Testing & Polish

### Unit Tests

- [ ] Domain layer tests (60+ tests)
  - [ ] All tool definitions
  - [ ] All parameter validations
  - [ ] All result types

- [ ] Application layer tests (30+ tests)
  - [ ] Command handlers
  - [ ] Query handlers

- [ ] Adapter tests (30+ tests)
  - [ ] HTTP handlers
  - [ ] Repository implementations

### Integration Tests

- [ ] End-to-end tool execution (10+ tests)
  - [ ] Tool → Document update flow
  - [ ] Multi-tool composition
  - [ ] Error handling

- [ ] AI provider integration (5+ tests)
  - [ ] Tool format conversion
  - [ ] Response parsing

### Emergent Behavior Tests

- [ ] Pattern detection scenarios
  - [ ] Repeated theme detection
  - [ ] Gap detection
  - [ ] Dominated alternative detection
  - [ ] Sensitivity analysis

### Performance Tests

- [ ] Token usage benchmarks
  - [ ] Before/after comparison
  - [ ] Context size measurement
  - [ ] Batch operation efficiency

### Documentation

- [ ] Tool catalog (all tools with examples)
- [ ] AI integration guide
- [ ] Token optimization guide

---

## Test Inventory

### Domain Layer Tests

| Test Category | Count | Status |
|--------------|-------|--------|
| ToolInvocation entity | 5 | ⬜ |
| RevisitSuggestion entity | 5 | ⬜ |
| ConfirmationRequest entity | 5 | ⬜ |
| Tool events | 10 | ⬜ |
| ToolDefinition | 5 | ⬜ |
| ToolRegistry | 10 | ⬜ |
| Issue Raising tools | 10 | ⬜ |
| Problem Frame tools | 12 | ⬜ |
| Objectives tools | 10 | ⬜ |
| Alternatives tools | 10 | ⬜ |
| Consequences tools | 10 | ⬜ |
| Tradeoffs tools | 8 | ⬜ |
| Recommendation tools | 8 | ⬜ |
| Decision Quality tools | 8 | ⬜ |
| Uncertainty tools | 6 | ⬜ |
| Revisit tools | 8 | ⬜ |
| Confirmation tools | 6 | ⬜ |
| Document ops tools | 6 | ⬜ |
| Analysis tools | 18 | ⬜ |
| **Total Domain** | **160** | ⬜ |

### Application Layer Tests

| Test Category | Count | Status |
|--------------|-------|--------|
| ExecuteToolHandler | 10 | ⬜ |
| ResolveRevisitHandler | 5 | ⬜ |
| RespondToConfirmationHandler | 5 | ⬜ |
| GetAvailableToolsHandler | 3 | ⬜ |
| GetToolInvocationsHandler | 3 | ⬜ |
| GetPendingRevisitsHandler | 3 | ⬜ |
| GetPendingConfirmationsHandler | 3 | ⬜ |
| **Total Application** | **32** | ⬜ |

### Adapter Layer Tests

| Test Category | Count | Status |
|--------------|-------|--------|
| HTTP tool endpoints | 10 | ⬜ |
| HTTP revisit endpoints | 4 | ⬜ |
| HTTP confirmation endpoints | 4 | ⬜ |
| Postgres tool invocation repo | 5 | ⬜ |
| Postgres revisit repo | 5 | ⬜ |
| Postgres confirmation repo | 5 | ⬜ |
| OpenAI format conversion | 5 | ⬜ |
| Anthropic format conversion | 5 | ⬜ |
| AI integration | 10 | ⬜ |
| **Total Adapter** | **53** | ⬜ |

### Integration Tests

| Test Category | Count | Status |
|--------------|-------|--------|
| Tool → Document flow | 5 | ⬜ |
| Multi-tool composition | 3 | ⬜ |
| Emergent behaviors | 4 | ⬜ |
| Error handling | 3 | ⬜ |
| **Total Integration** | **15** | ⬜ |

---

## Progress Summary

```
ATOMIC DECISION TOOLS: conversation
Phases: 0/14 complete
Files: 0/50+ created
Tests: 0/260 passing
```

### Completion Criteria

Module is COMPLETE when:

- [ ] All 14 phases completed
- [ ] 260+ tests passing
- [ ] All 50+ tools implemented and documented
- [ ] AI provider integration working (OpenAI + Anthropic)
- [ ] Token usage reduced by 40%+ vs text generation
- [ ] Document updates are delta-based (not full regeneration)
- [ ] Audit trail complete for all tool invocations
- [ ] Emergent behavior tests passing
- [ ] No clippy warnings

---

## Implementation Notes

### Linear Flow Compliance

Tools respect PrOACT's sequential structure:
- `suggest_revisit` queues suggestions, does not navigate
- User sees suggestions at component boundaries
- Agent cannot jump between components mid-flow

### Token Optimization Strategy

1. **Minimal Context** - ToolExecutionContext contains only IDs and counts
2. **Delta Updates** - Tools update specific fields, not entire sections
3. **Batch Operations** - Group related updates in single calls
4. **Lazy Loading** - Return summaries unless details requested

### Document Integration

Tools coordinate with Decision Document:
1. Tool modifies component state
2. Affected section regenerated (not full document)
3. Document checksum updated
4. Version incremented

---

## Dependencies

### Internal Dependencies
- `foundation` module (IDs, timestamps)
- `cycle` module (components, outputs)
- `conversation` module (messages, agent state)
- `decision-document` feature (document updates)

### External Dependencies
- `serde_json` - Parameter validation
- `jsonschema` - JSON Schema validation
- `uuid` - ID generation

---

*Checklist Version: 1.0.0*
*Created: 2026-01-09*
*Specification: features/conversation/atomic-decision-tools.md*
