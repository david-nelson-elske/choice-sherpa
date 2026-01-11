# Feature: Conversation Commands

**Module:** conversation | **Phase:** 4 | **Priority:** P1

> Command handlers for sending messages and regenerating AI responses.

---

## Requirements

### SendMessage Command

| ID | Rule | Given | When | Then | Error |
|----|------|-------|------|------|-------|
| R1 | Requires component ownership | User doesn't own component | SendMessage | Reject | `ForbiddenError` |
| R2 | Creates conversation if none | No conversation for component | SendMessage | Creates conversation | - |
| R3 | Rejects empty content | Empty message content | SendMessage | Reject | `ValidationError` |
| R4 | Stores user message | Valid content | SendMessage | User message persisted | - |
| R5 | Gets AI response | User message stored | SendMessage | AI response generated | - |
| R6 | Stores assistant message | AI responds | SendMessage | Assistant message persisted | - |
| R7 | Tracks token usage | Messages exchanged | SendMessage | Token counts recorded | - |
| R8 | Updates conversation state | First message in Ready | SendMessage | State -> InProgress | - |
| R9 | Rejects in Complete state | Conversation Complete | SendMessage | Reject | `ConversationComplete` |

### RegenerateResponse Command

| ID | Rule | Given | When | Then | Error |
|----|------|-------|------|------|-------|
| R10 | Requires ownership | User doesn't own conversation | Regenerate | Reject | `ForbiddenError` |
| R11 | Requires existing messages | No messages in conversation | Regenerate | Reject | `NoMessagesToRegenerate` |
| R12 | Removes last assistant message | Last message is assistant | Regenerate | Last message deleted | - |
| R13 | Generates new response | Last message removed | Regenerate | New AI response | - |
| R14 | Rejects if last is user | Last message is user | Regenerate | Reject | `LastMessageNotAssistant` |
| R15 | Rejects in Complete state | Conversation Complete | Regenerate | Reject | `ConversationComplete` |

### Streaming

| ID | Rule | Given | When | Then | Error |
|----|------|-------|------|------|-------|
| R16 | Streams tokens | AI generating | SendMessage | Tokens streamed incrementally | - |
| R17 | Final message on complete | Stream finishes | SendMessage | Complete message event | - |
| R18 | Error event on failure | AI fails mid-stream | SendMessage | Error event sent | - |

---

## Tasks

### SendMessage
- [ ] R1: Verify user owns component via session chain
- [ ] R2: Create conversation on first message if missing
- [ ] R3: Reject empty or whitespace-only content
- [ ] R4: Persist user message via repository
- [ ] R5: Call AI provider with conversation context
- [ ] R6: Persist assistant message via repository
- [ ] R7: Record token counts on messages
- [ ] R8: Transition Ready -> InProgress on first exchange
- [ ] R9: Check conversation state before processing

### RegenerateResponse
- [ ] R10: Verify conversation ownership
- [ ] R11: Check conversation has messages
- [ ] R12: Delete last assistant message from conversation
- [ ] R13: Generate new AI response with same context
- [ ] R14: Validate last message role before regenerating
- [ ] R15: Check conversation state is not Complete

### Streaming
- [ ] R16: Implement token streaming via channel
- [ ] R17: Send MessageComplete event when done
- [ ] R18: Send MessageError event on AI failure

---

## Context

- Use existing `AiProvider` port for LLM calls
- `ContextWindowManager` in `domain/conversation/context.rs` handles message windowing
- Commands return stream handle, caller decides sync/async consumption
- Authorization via session ownership chain: conversation -> component -> cycle -> session -> user
