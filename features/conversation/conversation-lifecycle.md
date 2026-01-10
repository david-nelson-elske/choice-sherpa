# Conversation Lifecycle & Agent Phases

**Module:** conversation
**Type:** Feature Specification
**Priority:** P0 (Core functionality)
**Last Updated:** 2026-01-08

> Complete specification of conversation states, agent phases, and component-specific behavior patterns.

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Required |
| Authorization Model | User must own parent session; conversation bound to component |
| Sensitive Data | User messages (Confidential), AI responses (Confidential), extracted structured data (Confidential) |
| Rate Limiting | Required - message send rate, AI API call rate |
| Audit Logging | State transitions, phase changes, extraction events |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| User message content | Confidential | Encrypt at rest, do not log |
| AI response content | Confidential | Sanitize before storage, encrypt at rest |
| Extracted structured data | Confidential | Encrypt at rest, validate schema before storage |
| System prompts | Internal | Version controlled, do not expose to users |
| Agent phase | Internal | Safe to log |
| Conversation state | Internal | Safe to log |
| Token counts | Internal | Safe to log |

### Security Events to Log

- State transitions (Initializing -> Ready -> InProgress -> Complete)
- Phase transitions (Intro -> Gather -> Extract -> Confirm)
- Extraction attempts (success/failure, item count, no raw data)
- Stream cancellation requests
- Authorization failures

### AI Response Sanitization

Before storing AI responses:
1. Validate response length within bounds
2. Strip any potential prompt injection artifacts
3. Validate JSON structure for extraction responses
4. Log extraction errors without exposing raw content

---

## AI Response Security

### Response Sanitization Pipeline

All AI responses MUST be sanitized before storage or display:

```rust
impl ResponseSanitizer {
    /// Sanitize AI response before storage
    pub fn sanitize(&self, response: &str) -> Result<String, SanitizationError> {
        let sanitized = response
            // 1. Validate length
            .pipe(|s| self.validate_length(s))
            // 2. Remove control characters (except newlines/tabs)
            .map(|s| self.remove_control_chars(&s))?
            // 3. Strip potential prompt injection markers
            .pipe(|s| self.strip_injection_markers(&s))
            // 4. Validate UTF-8 encoding
            .pipe(|s| self.validate_utf8(&s))?;

        Ok(sanitized)
    }

    fn validate_length(&self, s: &str) -> Result<&str, SanitizationError> {
        if s.len() > Self::MAX_RESPONSE_LENGTH {
            return Err(SanitizationError::TooLong {
                max: Self::MAX_RESPONSE_LENGTH,
                actual: s.len(),
            });
        }
        Ok(s)
    }

    fn remove_control_chars(&self, s: &str) -> String {
        s.chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
            .collect()
    }

    fn strip_injection_markers(&self, s: &str) -> String {
        // Remove common prompt injection patterns
        let patterns = [
            "```system",
            "```assistant",
            "[INST]",
            "[/INST]",
            "<|system|>",
            "<|assistant|>",
        ];

        let mut result = s.to_string();
        for pattern in patterns {
            result = result.replace(pattern, "");
        }
        result
    }

    const MAX_RESPONSE_LENGTH: usize = 100_000; // 100KB
}
```

### JSON Extraction Security

When extracting structured data from AI responses:

```rust
impl DataExtractor {
    pub async fn extract(&self, response: &str) -> Result<ExtractedData, ExtractionError> {
        // 1. Sanitize the raw response first
        let sanitized = self.sanitizer.sanitize(response)?;

        // 2. Parse JSON with size limits
        let value: serde_json::Value = serde_json::from_str(&sanitized)
            .map_err(|e| ExtractionError::ParseError(e.to_string()))?;

        // 3. Validate against schema
        self.schema_validator.validate(&value)?;

        // 4. Recursively sanitize string fields
        let sanitized_value = self.sanitize_json_strings(&value)?;

        Ok(ExtractedData {
            data: sanitized_value,
            extracted_at: Timestamp::now(),
        })
    }

    /// Recursively sanitize all string values in JSON
    fn sanitize_json_strings(&self, value: &Value) -> Result<Value, ExtractionError> {
        match value {
            Value::String(s) => {
                // Remove HTML/script tags
                let clean = ammonia::clean(s);
                // Truncate if too long
                let truncated = if clean.len() > 10_000 {
                    format!("{}...[truncated]", &clean[..10_000])
                } else {
                    clean
                };
                Ok(Value::String(truncated))
            }
            Value::Array(arr) => {
                let sanitized: Result<Vec<_>, _> = arr.iter()
                    .map(|v| self.sanitize_json_strings(v))
                    .collect();
                Ok(Value::Array(sanitized?))
            }
            Value::Object(obj) => {
                let sanitized: Result<serde_json::Map<_, _>, _> = obj.iter()
                    .map(|(k, v)| {
                        self.sanitize_json_strings(v).map(|sv| (k.clone(), sv))
                    })
                    .collect();
                Ok(Value::Object(sanitized?))
            }
            other => Ok(other.clone()),
        }
    }
}
```

### Security Checklist for AI Responses

- [ ] Response length validated (< 100KB)
- [ ] Control characters removed
- [ ] Prompt injection markers stripped
- [ ] UTF-8 encoding validated
- [ ] JSON string fields sanitized (HTML stripped)
- [ ] Schema validation passed
- [ ] Extracted data does not exceed field limits

### WebSocket Streaming Security

- Authenticate WebSocket connection on handshake
- Verify session ownership for each stream
- Rate limit stream chunks per connection
- Timeout inactive streams after configurable duration

---

## Overview

The conversation module orchestrates AI-guided dialogues within each PrOACT component. This document specifies the complete lifecycle from initialization through completion, including agent phases, data extraction, and streaming behavior.

---

## Conversation State Machine

```
                               ┌─────────────────┐
                               │   NOT_CREATED   │
                               │  (no record)    │
                               └────────┬────────┘
                                        │
                                        │ ComponentStarted event
                                        ▼
                               ┌─────────────────┐
                               │  INITIALIZING   │
                               │                 │
                               │ • Create record │
                               │ • Load config   │
                               │ • Build prompt  │
                               └────────┬────────┘
                                        │
                                        │ System + opening message added
                                        ▼
         ┌──────────────────────────────┴──────────────────────────────┐
         │                            READY                             │
         │                                                              │
         │  • System message present                                    │
         │  • Opening assistant message present                         │
         │  • Waiting for first user input                              │
         └────────┬─────────────────────────────────────────────────────┘
                  │
                  │ User sends first message
                  ▼
         ┌───────────────────────────────────────────────────────────────┐
         │                        IN_PROGRESS                            │
         │                                                               │
         │  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐       │
         │  │ PHASE_INTRO │───►│ PHASE_GATHER│───►│PHASE_EXTRACT│       │
         │  └─────────────┘    └──────┬──────┘    └──────┬──────┘       │
         │                            │                  │              │
         │                            │ (loops until    │              │
         │                            │  sufficient)    │              │
         │                            ▼                  ▼              │
         │                     ┌─────────────┐   ┌─────────────┐       │
         │                     │PHASE_CLARIFY│   │PHASE_CONFIRM│       │
         │                     └─────────────┘   └─────────────┘       │
         └────────┬───────────────────────────────────────┬─────────────┘
                  │                                       │
                  │ Component completed                   │ DataExtracted event
                  ▼                                       ▼
         ┌─────────────────┐                    ┌─────────────────┐
         │    COMPLETE     │                    │    CONFIRMED    │
         │   (read-only)   │◄───────────────────│ (awaiting save) │
         └─────────────────┘                    └─────────────────┘
```

### State Definitions

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationState {
    /// Conversation created, loading configuration
    Initializing,

    /// System prompt set, opening message added, awaiting first user input
    Ready,

    /// Active conversation with user
    InProgress,

    /// Data extracted, awaiting user confirmation
    Confirmed,

    /// Component completed, conversation is read-only
    Complete,
}

impl ConversationState {
    /// Can user send messages in this state?
    pub fn accepts_user_input(&self) -> bool {
        matches!(self, Self::Ready | Self::InProgress | Self::Confirmed)
    }

    /// Can AI respond in this state?
    pub fn can_generate_response(&self) -> bool {
        matches!(self, Self::Ready | Self::InProgress | Self::Confirmed)
    }

    /// Is conversation still modifiable?
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::Complete)
    }
}
```

---

## Agent Phases

Within the `InProgress` state, conversations move through distinct phases that guide the AI's behavior.

### Phase Definitions

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentPhase {
    /// Initial greeting and context setting
    Intro,

    /// Actively gathering information through questions
    Gather,

    /// Clarifying ambiguities or inconsistencies
    Clarify,

    /// Extracting structured data from conversation
    Extract,

    /// Confirming extracted data with user
    Confirm,
}

impl AgentPhase {
    /// Returns the AI's primary directive in this phase
    pub fn directive(&self) -> &'static str {
        match self {
            Self::Intro => "Set context and make user comfortable. Explain what this step covers.",
            Self::Gather => "Ask probing questions to elicit information. Listen actively.",
            Self::Clarify => "Resolve ambiguities. Ask follow-up questions on unclear points.",
            Self::Extract => "Synthesize conversation into structured output format.",
            Self::Confirm => "Present extracted data to user. Ask for corrections or approval.",
        }
    }

    /// Returns true if this phase typically generates AI responses
    pub fn is_ai_speaking(&self) -> bool {
        !matches!(self, Self::Extract)
    }
}
```

### Phase Transition Rules

```
┌─────────────────────────────────────────────────────────────────────┐
│                     PHASE TRANSITIONS                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  INTRO ──────────► GATHER                                           │
│         (after 1-2 exchanges)                                       │
│                                                                      │
│  GATHER ────┬────► GATHER   (loop while gathering)                  │
│             │                                                        │
│             ├────► CLARIFY  (when inconsistency detected)           │
│             │                                                        │
│             └────► EXTRACT  (when sufficient info gathered)         │
│                                                                      │
│  CLARIFY ───┬────► GATHER   (return to gathering)                   │
│             └────► EXTRACT  (if clarification sufficient)           │
│                                                                      │
│  EXTRACT ─────────► CONFIRM                                          │
│                                                                      │
│  CONFIRM ───┬────► GATHER   (user requests changes)                 │
│             └────► [DONE]   (user approves)                          │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Phase Transition Logic

```rust
pub struct PhaseTransitionEngine {
    extraction_detector: Arc<dyn ExtractionReadinessDetector>,
}

impl PhaseTransitionEngine {
    /// Determine next phase based on current state and conversation
    pub fn next_phase(
        &self,
        current: AgentPhase,
        conversation: &Conversation,
        latest_message: &Message,
    ) -> AgentPhase {
        match current {
            AgentPhase::Intro => {
                // Move to gather after 1-2 exchanges
                if conversation.user_message_count() >= 1 {
                    AgentPhase::Gather
                } else {
                    AgentPhase::Intro
                }
            }

            AgentPhase::Gather => {
                // Check if we have enough information to extract
                if self.extraction_detector.is_ready(conversation) {
                    AgentPhase::Extract
                } else if self.has_inconsistency(latest_message) {
                    AgentPhase::Clarify
                } else {
                    AgentPhase::Gather
                }
            }

            AgentPhase::Clarify => {
                // Return to gather or proceed to extract
                if self.extraction_detector.is_ready(conversation) {
                    AgentPhase::Extract
                } else {
                    AgentPhase::Gather
                }
            }

            AgentPhase::Extract => {
                // Always move to confirm after extraction
                AgentPhase::Confirm
            }

            AgentPhase::Confirm => {
                // Check user response
                if self.user_requests_changes(latest_message) {
                    AgentPhase::Gather
                } else {
                    // Stay in confirm - component completion happens separately
                    AgentPhase::Confirm
                }
            }
        }
    }
}
```

---

## Component-Specific Agent Behavior

Each PrOACT component has tailored agent behavior.

### IssueRaising

```yaml
Purpose: Categorize initial thoughts into decisions, objectives, uncertainties

Phases:
  Intro:
    - Welcome user
    - Explain that we'll capture their initial thoughts
    - Ask what situation they're thinking about

  Gather:
    - Listen for: decisions, goals, concerns, uncertainties
    - Categorize internally as conversation progresses
    - Ask clarifying questions to distinguish categories
    - Prompt: "Is that something you need to decide, something you want to achieve, or something you're uncertain about?"

  Extract:
    - Parse conversation for potential_decisions, objectives, uncertainties, considerations
    - Generate unique IDs for each item

  Confirm:
    - Present categorized list
    - Ask: "Have I captured everything? Should any items move between categories?"

Completion_Criteria:
  - At least 1 potential_decision identified
  - User confirms categorization
```

### ProblemFrame

```yaml
Purpose: Define decision architecture, constraints, stakeholders

Phases:
  Intro:
    - Review potential decisions from IssueRaising (if available)
    - Ask: "Which decision should we focus on?"

  Gather:
    - Identify primary decision maker
    - Clarify scope and constraints
    - Discover stakeholders and their influence
    - Map decision hierarchy (already made → focal → deferred)

  Extract:
    - Build decision_maker object
    - Build focal_decision with statement, scope, constraints
    - Build decision_hierarchy
    - Build parties array

  Confirm:
    - Present problem frame summary
    - Verify decision statement is actionable

Completion_Criteria:
  - Decision maker identified
  - Focal decision statement defined (min 10 chars)
  - Scope clarified
  - User confirms frame is accurate
```

### Objectives

```yaml
Purpose: Identify fundamental vs means objectives with measures

Phases:
  Intro:
    - Reference focal decision from ProblemFrame
    - Ask: "What outcomes matter most to you in this decision?"

  Gather:
    - Capture objectives as stated
    - Distinguish fundamental (end goals) from means (how to get there)
    - Probe for performance measures
    - Ask: "How would you know if you achieved this?"

  Extract:
    - Separate into fundamental_objectives and means_objectives
    - Link means to their supporting fundamental objectives
    - Capture performance measures where stated

  Confirm:
    - Present objective hierarchy
    - Verify fundamental objectives truly matter for their own sake

Completion_Criteria:
  - At least 1 fundamental objective
  - User confirms objectives capture what matters
```

### Alternatives

```yaml
Purpose: Capture options, strategy tables, status quo baseline

Phases:
  Intro:
    - Reference objectives from prior step
    - Ask: "What options are you considering? Include doing nothing (status quo)."

  Gather:
    - Capture each alternative with name and description
    - Ensure status quo is explicitly stated
    - Probe for creative alternatives: "What else could you do?"
    - For complex decisions, build strategy table

  Extract:
    - Build alternatives array
    - Designate status_quo_id
    - Build strategy_table if applicable

  Confirm:
    - Present all alternatives
    - Verify status quo is captured
    - Ask: "Are there any other options we should consider?"

Completion_Criteria:
  - At least 2 alternatives (including status quo)
  - Status quo explicitly identified
  - User confirms completeness
```

### Consequences

```yaml
Purpose: Build consequence table with Pugh ratings (-2 to +2)

Phases:
  Intro:
    - Load alternatives and objectives from prior steps
    - Explain Pugh rating scale
    - Start with first objective

  Gather:
    - For each objective, evaluate each alternative vs status quo
    - Ask: "How does [alternative] compare to status quo on [objective]?"
    - Capture rationale for each rating
    - Note uncertainty levels

  Extract:
    - Build consequence table with all cells populated
    - Format: cells map of "alt_id:obj_id" -> rating, rationale, uncertainty

  Confirm:
    - Present consequence table
    - Highlight any missing cells
    - Verify ratings make sense

Completion_Criteria:
  - All cells in consequence table filled
  - User confirms ratings are reasonable
```

### Tradeoffs

```yaml
Purpose: Surface dominated alternatives, tensions, irrelevant objectives

Phases:
  Intro:
    - Load consequences table
    - Run dominance analysis automatically
    - Present initial findings

  Gather:
    - Discuss dominated alternatives
    - Explore tensions between remaining alternatives
    - Identify irrelevant objectives (all same rating)
    - Ask: "Does this analysis surprise you?"

  Extract:
    - Build dominated_alternatives array
    - Build irrelevant_objectives array
    - Build tensions array with gains/losses for each alternative

  Confirm:
    - Present tradeoff summary
    - Verify dominance conclusions

Completion_Criteria:
  - Dominance analysis complete
  - User understands key tradeoffs
```

### Recommendation

```yaml
Purpose: Synthesize analysis - does NOT decide for user

Phases:
  Intro:
    - Reference full analysis path
    - Explain: "I'll summarize what we've found, but the decision is yours."

  Gather:
    - Discuss key considerations
    - Surface remaining uncertainties
    - Explore if any alternative stands out
    - Ask: "What else would help you decide?"

  Extract:
    - Write synthesis text
    - Identify standout_option if applicable
    - List key_considerations
    - List remaining_uncertainties with resolution paths

  Confirm:
    - Present recommendation summary
    - Emphasize user retains decision authority

Completion_Criteria:
  - Synthesis written (min 50 chars)
  - User acknowledges summary
```

### DecisionQuality

```yaml
Purpose: Rate 7 DQ elements, compute overall score

Phases:
  Intro:
    - Explain Decision Quality framework
    - Present 7 elements to rate

  Gather:
    - For each element, ask user to rate 0-100%
    - Discuss rationale for each rating
    - Identify improvement paths for low scores

  Extract:
    - Build elements array with scores and rationale
    - Compute overall_score as MIN of all element scores

  Confirm:
    - Present DQ scorecard
    - If overall < 100%, discuss what would improve it

Completion_Criteria:
  - All 7 elements scored
  - User confirms scores reflect their confidence
```

### NotesNextSteps

```yaml
Purpose: Wrap-up notes, open questions, action items

Phases:
  Intro:
    - Ask: "What questions or thoughts remain?"
    - Probe for planned actions

  Gather:
    - Capture notes
    - List open questions
    - Define action items with owners and due dates
    - If DQ = 100%, capture decision affirmation

  Extract:
    - Build notes array
    - Build open_questions array
    - Build planned_actions array

  Confirm:
    - Present summary
    - Verify next steps are clear

Completion_Criteria:
  - User confirms they're ready to wrap up
```

---

## Data Extraction Specification

### Extraction Trigger

Data extraction happens when:
1. Agent enters `Extract` phase
2. Sufficient information gathered (per component rules)
3. User explicitly requests extraction ("Show me what you've captured")

### Extraction Process

```rust
pub struct DataExtractor {
    ai_provider: Arc<dyn AIProvider>,
    schema_validator: Arc<dyn ComponentSchemaValidator>,
}

impl DataExtractor {
    pub async fn extract(
        &self,
        component_type: ComponentType,
        conversation: &Conversation,
    ) -> Result<ExtractedData, ExtractionError> {
        // 1. Build extraction prompt
        let prompt = self.build_extraction_prompt(component_type, conversation);

        // 2. Call AI with low temperature for determinism
        let response = self.ai_provider.complete(CompletionRequest {
            messages: vec![Message::user(prompt)],
            system_prompt: Some(self.extraction_system_prompt()),
            temperature: Some(0.0),
            max_tokens: Some(4000),
            ..Default::default()
        }).await?;

        // 3. Parse JSON from response
        let extracted: serde_json::Value = serde_json::from_str(&response.content)
            .map_err(|e| ExtractionError::ParseError(e.to_string()))?;

        // 4. Validate against schema
        self.schema_validator.validate(component_type, &extracted)?;

        // 5. Return validated data
        Ok(ExtractedData {
            component_type,
            data: extracted,
            extracted_at: Timestamp::now(),
        })
    }

    fn extraction_system_prompt(&self) -> String {
        r#"
You are a structured data extractor. Your task is to extract structured data
from a conversation transcript.

RULES:
1. Output ONLY valid JSON matching the provided schema
2. Do not include any text before or after the JSON
3. Generate UUIDs for any new items that need IDs
4. Preserve existing IDs if they were mentioned
5. If information is not present, use null or empty arrays as appropriate
6. Be conservative - only extract what was clearly stated
        "#.to_string()
    }

    fn build_extraction_prompt(
        &self,
        component_type: ComponentType,
        conversation: &Conversation,
    ) -> String {
        let schema = self.schema_validator.schema_for(component_type);
        let transcript = conversation.format_for_extraction();

        format!(
            "Extract structured data from this conversation.\n\n\
             SCHEMA:\n{}\n\n\
             CONVERSATION:\n{}\n\n\
             Output the JSON now:",
            serde_json::to_string_pretty(schema).unwrap(),
            transcript
        )
    }
}
```

### Extraction Readiness Detection

```rust
pub trait ExtractionReadinessDetector: Send + Sync {
    fn is_ready(&self, conversation: &Conversation) -> bool;
}

pub struct ComponentSpecificDetector;

impl ExtractionReadinessDetector for ComponentSpecificDetector {
    fn is_ready(&self, conversation: &Conversation) -> bool {
        let component_type = conversation.component_type();
        let message_count = conversation.user_message_count();

        match component_type {
            ComponentType::IssueRaising => {
                // Ready after 3+ user messages or if user says "done"
                message_count >= 3 || conversation.contains_completion_signal()
            }
            ComponentType::ProblemFrame => {
                // Need decision maker and focal decision
                conversation.mentions_all(&["decision maker", "decide", "choice"])
            }
            ComponentType::Objectives => {
                // Need at least one clear objective
                message_count >= 2
            }
            ComponentType::Alternatives => {
                // Need at least 2 alternatives mentioned
                message_count >= 2
            }
            ComponentType::Consequences => {
                // Need discussion of at least half the matrix
                message_count >= 4
            }
            _ => message_count >= 3,
        }
    }
}
```

---

## Message Context Window Management

### Context Window Strategy

```rust
pub struct ContextWindowManager {
    max_tokens: u32,
    reserved_for_response: u32,
}

impl ContextWindowManager {
    /// Build messages array for AI request, managing context window
    pub fn build_context(
        &self,
        conversation: &Conversation,
        system_prompt: &str,
    ) -> Vec<Message> {
        let mut messages = Vec::new();
        let mut token_count = self.estimate_tokens(system_prompt);

        // Always include system message
        messages.push(Message::system(system_prompt.to_string()));

        // Get all conversation messages
        let all_messages: Vec<_> = conversation.messages().collect();

        // Include messages from most recent, working backward
        for msg in all_messages.iter().rev() {
            let msg_tokens = self.estimate_tokens(&msg.content);

            if token_count + msg_tokens + self.reserved_for_response > self.max_tokens {
                // Would exceed limit - add summarization
                let summary = self.summarize_truncated(&all_messages, messages.len());
                messages.insert(1, Message::system(format!(
                    "[Earlier conversation summarized: {}]",
                    summary
                )));
                break;
            }

            token_count += msg_tokens;
            messages.insert(1, msg.clone());
        }

        messages
    }

    fn estimate_tokens(&self, text: &str) -> u32 {
        // Rough estimate: ~4 chars per token
        (text.len() / 4) as u32
    }

    fn summarize_truncated(&self, messages: &[Message], included_count: usize) -> String {
        let truncated: Vec<_> = messages.iter()
            .take(messages.len() - included_count)
            .collect();

        // Simple summary - in production, could use AI for better summary
        format!(
            "{} earlier messages discussed: {}",
            truncated.len(),
            truncated.iter()
                .filter(|m| m.role == MessageRole::User)
                .take(3)
                .map(|m| m.content.chars().take(50).collect::<String>())
                .collect::<Vec<_>>()
                .join("; ")
        )
    }
}
```

### Per-Component Token Budgets

| Component | Max Context Tokens | Reserved for Response |
|-----------|--------------------|-----------------------|
| IssueRaising | 16,000 | 2,000 |
| ProblemFrame | 16,000 | 2,000 |
| Objectives | 16,000 | 2,000 |
| Alternatives | 16,000 | 2,000 |
| Consequences | 32,000 | 4,000 |
| Tradeoffs | 32,000 | 4,000 |
| Recommendation | 32,000 | 4,000 |
| DecisionQuality | 16,000 | 2,000 |
| NotesNextSteps | 8,000 | 1,000 |

---

## Streaming Specification

### Streaming Message Handler

```rust
pub struct StreamingMessageHandler {
    ai_provider: Arc<dyn AIProvider>,
    conversation_repo: Arc<dyn ConversationRepository>,
    context_manager: ContextWindowManager,
    ws_broadcaster: Arc<dyn WebSocketBroadcaster>,
}

impl StreamingMessageHandler {
    pub async fn handle(
        &self,
        cmd: StreamMessageCommand,
    ) -> Result<MessageId, CommandError> {
        // 1. Load conversation
        let mut conversation = self.conversation_repo
            .find_by_component(&cmd.component_id)
            .await?
            .ok_or(CommandError::NotFound("Conversation"))?;

        // 2. Add user message
        let user_message = conversation.add_user_message(cmd.content.clone());

        // 3. Build context for AI
        let system_prompt = self.get_system_prompt(&conversation);
        let messages = self.context_manager.build_context(&conversation, &system_prompt);

        // 4. Start streaming response
        let assistant_message_id = MessageId::new();
        let mut stream = self.ai_provider.stream_complete(CompletionRequest {
            messages,
            system_prompt: Some(system_prompt),
            ..Default::default()
        }).await?;

        // 5. Stream chunks to WebSocket
        let mut full_response = String::new();
        let session_id = cmd.session_id;

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    full_response.push_str(&chunk.delta);

                    // Broadcast chunk to client
                    self.ws_broadcaster.broadcast_to_session(
                        &session_id,
                        WebSocketMessage::StreamChunk {
                            message_id: assistant_message_id.clone(),
                            delta: chunk.delta,
                            is_final: chunk.finish_reason.is_some(),
                        },
                    ).await?;
                }
                Err(e) => {
                    // Broadcast error
                    self.ws_broadcaster.broadcast_to_session(
                        &session_id,
                        WebSocketMessage::StreamError {
                            message_id: assistant_message_id.clone(),
                            error: e.to_string(),
                        },
                    ).await?;

                    return Err(CommandError::AIError(e.to_string()));
                }
            }
        }

        // 6. Add complete assistant message
        conversation.add_assistant_message_with_id(
            assistant_message_id.clone(),
            full_response,
        );

        // 7. Update conversation state if needed
        let new_phase = self.determine_phase(&conversation);
        conversation.set_phase(new_phase);

        // 8. Persist
        self.conversation_repo.update(&conversation).await?;

        Ok(assistant_message_id)
    }
}
```

### Stream Chunk Format

```typescript
// WebSocket message types for streaming
interface StreamChunkMessage {
  type: 'stream_chunk';
  message_id: string;
  delta: string;
  is_final: boolean;
}

interface StreamErrorMessage {
  type: 'stream_error';
  message_id: string;
  error: string;
}

interface StreamCompleteMessage {
  type: 'stream_complete';
  message_id: string;
  full_content: string;
  usage: {
    prompt_tokens: number;
    completion_tokens: number;
    estimated_cost_cents: number;
  };
}
```

### Stream Cancellation

```rust
pub struct StreamCancelCommand {
    pub session_id: SessionId,
    pub message_id: MessageId,
    pub user_id: UserId,
}

pub struct StreamCancelHandler {
    active_streams: Arc<RwLock<HashMap<MessageId, CancellationToken>>>,
}

impl StreamCancelHandler {
    pub async fn handle(&self, cmd: StreamCancelCommand) -> Result<(), CommandError> {
        let streams = self.active_streams.read().await;

        if let Some(token) = streams.get(&cmd.message_id) {
            token.cancel();
            Ok(())
        } else {
            Err(CommandError::NotFound("Active stream not found"))
        }
    }
}
```

---

## Component Revision Workflow

### Revision vs. New Message

```rust
pub enum RevisionAction {
    /// User wants to modify extracted data
    EditExtraction,
    /// User wants to continue conversation
    ContinueConversation,
    /// User wants to restart from a phase
    RestartFromPhase(AgentPhase),
}

impl Conversation {
    /// Revise extracted data without losing conversation history
    pub fn revise_extraction(
        &mut self,
        revised_data: serde_json::Value,
    ) -> Result<(), DomainError> {
        if !self.state().is_active() {
            return Err(DomainError::invalid_state(
                "Cannot revise completed conversation"
            ));
        }

        // Store revision as a new message
        self.add_system_message(format!(
            "User revised extraction: {}",
            serde_json::to_string_pretty(&revised_data).unwrap()
        ));

        // Update current extraction
        self.pending_extraction = Some(revised_data);

        // Return to confirm phase
        self.phase = AgentPhase::Confirm;

        Ok(())
    }

    /// Reopen conversation from complete state (with authorization)
    pub fn reopen(&mut self) -> Result<(), DomainError> {
        if self.state != ConversationState::Complete {
            return Err(DomainError::invalid_state(
                "Can only reopen completed conversations"
            ));
        }

        self.state = ConversationState::InProgress;
        self.phase = AgentPhase::Gather;

        self.add_system_message(
            "Conversation reopened for additional discussion."
        );

        Ok(())
    }
}
```

---

## Tasks

- [x] Implement ConversationState enum in `backend/src/domain/conversation/state.rs`
- [x] Implement AgentPhase enum in `backend/src/domain/conversation/phase.rs`
- [x] Implement PhaseTransitionEngine in `backend/src/domain/conversation/engine.rs`
- [x] Implement DataExtractor in `backend/src/domain/conversation/extractor.rs`
- [x] Implement ContextWindowManager in `backend/src/domain/conversation/context.rs`
- [x] Implement StreamingMessageHandler in `backend/src/application/handlers/stream_message.rs`
- [x] Add component-specific agent configs in `backend/src/domain/conversation/configs/`
- [x] Create opening message templates for all 9 components
- [x] Write unit tests for phase transitions
- [x] Write integration tests for extraction
- [x] Document streaming protocol in API docs

---

## Related Documents

- **Conversation Module:** `docs/modules/conversation.md`
- **Component Schemas:** `features/proact-types/component-schemas.md`
- **AI Provider Integration:** `features/integrations/ai-provider-integration.md`
- **WebSocket Bridge:** `features/infrastructure/websocket-event-bridge.md`

---

*Version: 1.0.0*
*Created: 2026-01-08*
