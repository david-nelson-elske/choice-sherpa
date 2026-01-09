# DRY Analysis Report - Choice Sherpa

**Generated:** 2026-01-08
**Scope:** All modules, infrastructure, and cross-cutting patterns
**Method:** Multi-agent parallel analysis with consolidation

---

## Executive Summary

This analysis examined all 8 modules plus infrastructure across Choice Sherpa specifications and identified **significant DRY opportunities**. The analysis found:

| Category | Patterns Found | Estimated Lines Saved |
|----------|---------------|----------------------|
| **Foundation** | 8 patterns | ~400 lines |
| **PrOACT-Types** | 10 patterns | ~800-1000 lines |
| **Membership** | 8 patterns | ~250 lines |
| **Session** | 10 patterns | ~200 lines |
| **Cycle** | 8 patterns | ~630 lines |
| **Conversation** | 10 patterns | ~235 lines |
| **Analysis** | 8 patterns | ~150 lines |
| **Dashboard** | 7 patterns | ~710 lines |
| **Infrastructure** | 10 patterns | ~400-600 lines |
| **Cross-Module** | 10 patterns | ~1,200 lines |
| **Total** | **89 patterns** | **~5,000+ lines** |

---

## Top 10 Priority Abstractions

### Critical Priority (Implement First)

#### 1. `declare_uuid_id!` Macro

**Affects:** ALL modules (8+ ID types)
**Current:** 120 lines per ID type, identical implementations
**Saves:** ~960 lines

```rust
// Before: 120 lines × 8 IDs
// After: 1 line per ID

declare_uuid_id!(SessionId, "Unique identifier for sessions");
declare_uuid_id!(CycleId, "Unique identifier for cycles");
declare_uuid_id!(ComponentId, "Unique identifier for components");
declare_uuid_id!(ConversationId, "Unique identifier for conversations");
declare_uuid_id!(MessageId, "Unique identifier for messages");
declare_uuid_id!(MembershipId, "Unique identifier for memberships");
declare_uuid_id!(AnalysisId, "Unique identifier for analysis runs");
declare_uuid_id!(EventId, "Unique identifier for domain events");
```

**Implementation:**

```rust
// foundation/ids.rs

macro_rules! declare_uuid_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self { Self(Uuid::new_v4()) }
            pub fn from_uuid(uuid: Uuid) -> Self { Self(uuid) }
            pub fn as_uuid(&self) -> &Uuid { &self.0 }
        }

        impl Default for $name {
            fn default() -> Self { Self::new() }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(Uuid::parse_str(s)?))
            }
        }
    };
}
```

**Location:** `backend/src/domain/foundation/ids.rs`

---

#### 2. `#[derive(Component)]` Macro for PrOACT

**Affects:** proact-types (9 components)
**Current:** 60-80 lines of boilerplate per component
**Saves:** ~540-720 lines

```rust
// Before: 80 lines per component × 9 components = 720 lines
// After: Derive macro handles all boilerplate

#[derive(Component)]
pub struct IssueRaising {
    base: ComponentBase,
    output: IssueRaisingOutput,
}

#[derive(Component)]
pub struct ProblemFrame {
    base: ComponentBase,
    output: ProblemFrameOutput,
}

// ... etc for all 9 components
```

**What the macro generates:**
- `Component` trait implementation (6 accessor methods, 3 action methods, 2 serialization methods)
- `Default` trait implementation
- `output()` and `set_output()` methods with `touch()` calls
- JSON conversion via `output_as_value()` and `set_output_from_value()`

**Location:** `backend/src/domain/proact_types/macros.rs`

---

#### 3. `domain_event!` Macro

**Affects:** ALL modules (25+ events)
**Current:** 15 lines per DomainEvent impl
**Saves:** ~200 lines

```rust
// Before: 15 lines per event × 25+ events
impl DomainEvent for SessionCreated {
    fn event_type(&self) -> &'static str { "session.created" }
    fn aggregate_id(&self) -> String { self.session_id.to_string() }
    fn aggregate_type(&self) -> &'static str { "Session" }
    fn occurred_at(&self) -> Timestamp { self.created_at }
    fn event_id(&self) -> EventId { self.event_id.clone() }
}

// After: 6 lines per event
domain_event!(
    SessionCreated,
    event_type = "session.created",
    aggregate_id = session_id,
    aggregate_type = "Session",
    occurred_at = created_at,
    event_id = event_id
);
```

**Implementation:**

```rust
// foundation/events.rs

macro_rules! domain_event {
    (
        $event_name:ident,
        event_type = $event_type:expr,
        aggregate_id = $agg_id_field:ident,
        aggregate_type = $agg_type:expr,
        occurred_at = $occurred_field:ident,
        event_id = $event_id_field:ident
    ) => {
        impl DomainEvent for $event_name {
            fn event_type(&self) -> &'static str { $event_type }
            fn aggregate_id(&self) -> String { self.$agg_id_field.to_string() }
            fn aggregate_type(&self) -> &'static str { $agg_type }
            fn occurred_at(&self) -> Timestamp { self.$occurred_field }
            fn event_id(&self) -> EventId { self.$event_id_field.clone() }
        }
    };
}
```

**Location:** `backend/src/domain/foundation/events.rs`

---

### High Priority

#### 4. `StateMachine` Trait

**Affects:** 5 modules (ComponentStatus, SessionStatus, CycleStatus, MembershipStatus, ConversationStatus)
**Current:** Repeated `can_transition_to()` pattern with manual validation
**Saves:** ~100 lines

```rust
// foundation/state_machine.rs

pub trait StateMachine: Sized + Copy + PartialEq + std::fmt::Debug {
    /// Returns true if transition from self to target is valid
    fn can_transition_to(&self, target: &Self) -> bool;

    /// Performs transition with validation, returning error if invalid
    fn transition_to(&self, target: Self) -> Result<Self, ValidationError> {
        if self.can_transition_to(&target) {
            Ok(target)
        } else {
            Err(ValidationError::invalid_format(
                "state_transition",
                format!("Cannot transition from {:?} to {:?}", self, target)
            ))
        }
    }

    /// Returns all valid target states from current state
    fn valid_transitions(&self) -> Vec<Self>;
}
```

**Usage:**

```rust
impl StateMachine for ComponentStatus {
    fn can_transition_to(&self, target: &Self) -> bool {
        matches!(
            (self, target),
            (NotStarted, InProgress) |
            (InProgress, Complete) |
            (InProgress, NeedsRevision) |
            (Complete, NeedsRevision) |
            (NeedsRevision, InProgress)
        )
    }

    fn valid_transitions(&self) -> Vec<Self> {
        match self {
            NotStarted => vec![InProgress],
            InProgress => vec![Complete, NeedsRevision],
            Complete => vec![NeedsRevision],
            NeedsRevision => vec![InProgress],
        }
    }
}
```

**Location:** `backend/src/domain/foundation/state_machine.rs`

---

#### 5. `CacheRegion<ID, T>` Generic

**Affects:** dashboard (6 cache types)
**Current:** Identical CRUD + invalidation repeated 6 times
**Saves:** ~120 lines

```rust
// dashboard/cache.rs

pub struct CacheEntry<T> {
    pub data: T,
    pub updated_at: Timestamp,
    pub last_event_id: Option<EventId>,
    pub version: u64,
}

impl<T> CacheEntry<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            updated_at: Timestamp::now(),
            last_event_id: None,
            version: 1,
        }
    }

    pub fn is_stale(&self, max_age: Duration) -> bool {
        self.updated_at.elapsed() > max_age
    }

    pub fn touch(&mut self) {
        self.updated_at = Timestamp::now();
        self.version += 1;
    }
}

#[async_trait]
pub trait CacheRegion<ID, T>: Send + Sync
where
    ID: Hash + Eq + Clone + Send + Sync,
    T: Clone + Send + Sync,
{
    async fn get(&self, id: &ID) -> Option<CacheEntry<T>>;
    async fn set(&self, id: ID, entry: T);
    async fn update<F>(&self, id: &ID, f: F) -> Option<T>
    where
        F: FnOnce(&mut T) + Send;
    async fn invalidate(&self, id: &ID);
    async fn evict_stale(&self, max_age: Duration) -> usize;
    async fn count(&self) -> usize;
}
```

**Location:** `backend/src/domain/dashboard/cache.rs`

---

#### 6. Component Match Delegation Macro

**Affects:** cycle module (23 match blocks)
**Current:** 9-variant match repeated 23 times
**Saves:** ~200 lines

```rust
// cycle/macros.rs

macro_rules! delegate_to_variant {
    ($variant:expr, $method:ident) => {
        match $variant {
            ComponentVariant::IssueRaising(c) => c.$method(),
            ComponentVariant::ProblemFrame(c) => c.$method(),
            ComponentVariant::Objectives(c) => c.$method(),
            ComponentVariant::Alternatives(c) => c.$method(),
            ComponentVariant::Consequences(c) => c.$method(),
            ComponentVariant::Tradeoffs(c) => c.$method(),
            ComponentVariant::Recommendation(c) => c.$method(),
            ComponentVariant::DecisionQuality(c) => c.$method(),
            ComponentVariant::NotesNextSteps(c) => c.$method(),
        }
    };

    ($variant:expr, $method:ident, $($arg:expr),*) => {
        match $variant {
            ComponentVariant::IssueRaising(c) => c.$method($($arg),*),
            ComponentVariant::ProblemFrame(c) => c.$method($($arg),*),
            ComponentVariant::Objectives(c) => c.$method($($arg),*),
            ComponentVariant::Alternatives(c) => c.$method($($arg),*),
            ComponentVariant::Consequences(c) => c.$method($($arg),*),
            ComponentVariant::Tradeoffs(c) => c.$method($($arg),*),
            ComponentVariant::Recommendation(c) => c.$method($($arg),*),
            ComponentVariant::DecisionQuality(c) => c.$method($($arg),*),
            ComponentVariant::NotesNextSteps(c) => c.$method($($arg),*),
        }
    };
}

// Usage:
impl ComponentVariant {
    pub fn start(&mut self) -> Result<(), ComponentError> {
        delegate_to_variant!(self, start)
    }

    pub fn complete(&mut self) -> Result<(), ComponentError> {
        delegate_to_variant!(self, complete)
    }

    pub fn id(&self) -> ComponentId {
        delegate_to_variant!(self, id)
    }
}
```

**Location:** `backend/src/domain/cycle/macros.rs`

---

#### 7. `AuthorizationHelper` Service

**Affects:** session, cycle, conversation (15+ handlers)
**Current:** Load→Authorize→Log repeated everywhere
**Saves:** ~150 lines

```rust
// foundation/authorization.rs or ports/authorization.rs

pub struct AuthorizationHelper {
    session_repo: Arc<dyn SessionRepository>,
    cycle_repo: Arc<dyn CycleRepository>,
}

impl AuthorizationHelper {
    pub async fn authorize_session_access(
        &self,
        user_id: &UserId,
        session_id: SessionId,
    ) -> Result<Session, DomainError> {
        let session = self.session_repo
            .find_by_id(session_id)
            .await?
            .ok_or_else(|| DomainError::new(
                ErrorCode::SessionNotFound,
                format!("Session {} not found", session_id)
            ))?;

        if !session.is_owner(user_id) {
            warn!(
                user_id = %user_id,
                session_id = %session_id,
                owner_id = %session.user_id(),
                "Unauthorized session access attempt"
            );
            return Err(DomainError::new(
                ErrorCode::Forbidden,
                "User does not own this session"
            ));
        }

        info!(user_id = %user_id, session_id = %session_id, "Session access authorized");
        Ok(session)
    }

    pub async fn authorize_cycle_access(
        &self,
        user_id: &UserId,
        cycle_id: CycleId,
    ) -> Result<(Cycle, Session), DomainError> {
        let cycle = self.cycle_repo
            .find_by_id(cycle_id)
            .await?
            .ok_or_else(|| DomainError::new(
                ErrorCode::CycleNotFound,
                format!("Cycle {} not found", cycle_id)
            ))?;

        let session = self.authorize_session_access(user_id, cycle.session_id()).await?;

        Ok((cycle, session))
    }

    pub async fn authorize_component_access(
        &self,
        user_id: &UserId,
        cycle_id: CycleId,
        component_type: ComponentType,
    ) -> Result<(Cycle, Session), DomainError> {
        let (cycle, session) = self.authorize_cycle_access(user_id, cycle_id).await?;

        // Additional component-specific checks can go here

        Ok((cycle, session))
    }
}
```

**Location:** `backend/src/domain/foundation/authorization.rs`

---

### Medium Priority

#### 8. `TokenBucket` Algorithm

**Affects:** infrastructure (rate limiter, AI tokens, WebSocket limits)
**Current:** Same algorithm in Redis Lua + Rust + multiple places
**Saves:** ~100 lines

```rust
// infrastructure/token_bucket.rs

pub struct TokenBucket {
    limit: u32,
    window_secs: u32,
    store: Arc<dyn TokenStore>,
}

#[async_trait]
pub trait TokenStore: Send + Sync {
    async fn get_and_increment(&self, key: &str, amount: u32, window_secs: u32)
        -> Result<TokenState, TokenBucketError>;
    async fn reset(&self, key: &str) -> Result<(), TokenBucketError>;
}

pub struct TokenState {
    pub current: u32,
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: Timestamp,
}

impl TokenBucket {
    pub fn new(limit: u32, window_secs: u32, store: Arc<dyn TokenStore>) -> Self {
        Self { limit, window_secs, store }
    }

    pub async fn consume(&self, key: &str, tokens: u32) -> Result<TokenState, TokenBucketError> {
        let state = self.store.get_and_increment(key, tokens, self.window_secs).await?;

        if state.current > self.limit {
            return Err(TokenBucketError::LimitExceeded {
                limit: self.limit,
                current: state.current,
                reset_at: state.reset_at,
            });
        }

        Ok(state)
    }

    pub async fn check(&self, key: &str) -> Result<TokenState, TokenBucketError> {
        self.store.get_and_increment(key, 0, self.window_secs).await
    }
}
```

**Location:** `backend/src/adapters/infrastructure/token_bucket.rs`

---

#### 9. `Repository<T, ID>` Base Trait

**Affects:** 5 repositories
**Current:** Identical CRUD signatures repeated
**Saves:** ~90 lines

```rust
// foundation/repository.rs

#[async_trait]
pub trait Repository<T, ID>: Send + Sync
where
    T: Send + Sync,
    ID: Send + Sync,
{
    async fn find_by_id(&self, id: ID) -> Result<Option<T>, DomainError>;
    async fn save(&self, entity: &T) -> Result<(), DomainError>;
    async fn update(&self, entity: &T) -> Result<(), DomainError>;
    async fn delete(&self, id: ID) -> Result<(), DomainError>;
    async fn exists(&self, id: ID) -> Result<bool, DomainError> {
        Ok(self.find_by_id(id).await?.is_some())
    }
}

// Module-specific repositories extend the base:
#[async_trait]
pub trait SessionRepository: Repository<Session, SessionId> {
    async fn find_by_user(&self, user_id: UserId) -> Result<Vec<Session>, DomainError>;
    async fn find_active_by_user(&self, user_id: UserId) -> Result<Vec<Session>, DomainError>;
}

#[async_trait]
pub trait CycleRepository: Repository<Cycle, CycleId> {
    async fn find_by_session(&self, session_id: SessionId) -> Result<Vec<Cycle>, DomainError>;
    async fn find_children(&self, parent_id: CycleId) -> Result<Vec<Cycle>, DomainError>;
}
```

**Location:** `backend/src/domain/foundation/repository.rs`

---

#### 10. `EventEnvelope::from_event()` Helper

**Affects:** ALL command handlers (25+)
**Current:** Manual envelope construction with serialization
**Saves:** ~150 lines

```rust
// foundation/events.rs

impl EventEnvelope {
    /// Create envelope from a domain event with automatic serialization
    pub fn from_event<T>(event: &T, aggregate_type: &str) -> Self
    where
        T: DomainEvent + Serialize,
    {
        Self {
            event_id: event.event_id(),
            event_type: event.event_type().to_string(),
            aggregate_id: event.aggregate_id(),
            aggregate_type: aggregate_type.to_string(),
            occurred_at: event.occurred_at(),
            payload: serde_json::to_value(event)
                .expect("Event serialization should never fail"),
            metadata: EventMetadata::default(),
        }
    }

    /// Parse payload back to typed event
    pub fn parse_payload<T>(&self) -> Result<T, DomainError>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_value(self.payload.clone()).map_err(|e| {
            DomainError::new(
                ErrorCode::ValidationFailed,
                format!("Failed to parse {} payload: {}", self.event_type, e)
            )
        })
    }
}

// Usage in handlers:
let event = SessionCreated { /* ... */ };
let envelope = EventEnvelope::from_event(&event, "Session")
    .with_correlation_id(metadata.correlation_id.clone())
    .with_user_id(user_id.to_string());
self.event_publisher.publish(envelope).await?;
```

**Location:** `backend/src/domain/foundation/events.rs`

---

## Module-Specific Findings

### Foundation Module

| Pattern | Occurrences | Priority | Action |
|---------|-------------|----------|--------|
| UUID ID boilerplate | 3 (will grow) | Critical | `declare_uuid_id!` macro |
| Status enum state machines | 3 | High | `StateMachine` trait |
| Error code Display match | 1 (18 cases) | Medium | Use `strum` crate |
| Test assertion patterns | 10+ | Low | Test helper macros |

**Key Files:**
- `ids.rs`: Lines 10-134 contain 3 identical ID implementations
- `component_status.rs`: Lines 61-71 repeat Display pattern
- `errors.rs`: Lines 88-110 manual ErrorCode→String mapping

---

### PrOACT-Types Module

| Pattern | Occurrences | Priority | Action |
|---------|-------------|----------|--------|
| Component trait impl | 9 × 80 lines | Critical | `#[derive(Component)]` |
| Output getter/setter | 9 × 6 lines | High | Include in derive |
| JSON serialization | 9 × 8 lines | High | Include in derive |
| Add-to-vec mutations | 20+ instances | Medium | `add_to_vec!` macro |
| Test structure | 9 × 50 lines | Medium | Parametrized tests |

**Quantified Impact:**

| Abstraction | Lines Before | Lines After | Savings |
|-------------|--------------|-------------|---------|
| Component derive | 720 | 27 | 693 (96%) |
| Output methods | 54 | 0 | 54 (100%) |
| JSON conversion | 72 | 0 | 72 (100%) |
| Tests | 450 | 150 | 300 (67%) |
| **Total** | **1,296** | **177** | **1,119 (86%)** |

---

### Membership Module

| Pattern | Occurrences | Priority | Action |
|---------|-------------|----------|--------|
| Status transition validation | 8 methods | High | Use `StateMachine` trait |
| Timestamp + event recording | 7 methods | High | `record_state_change()` helper |
| Tier limits constructors | 4 definitions | Medium | Data-driven config table |
| Feature flag checking | 6 methods | Medium | Generic `check_limit()` |
| Webhook handler flow | 3+ handlers | Medium | `StripeWebhookHandler` trait |

**Recommended Helper:**

```rust
impl Membership {
    fn record_state_change(
        &mut self,
        new_status: MembershipStatus,
        event: MembershipEvent,
    ) -> Result<(), DomainError> {
        self.status.transition_to(new_status)?;
        self.status = new_status;
        self.updated_at = Timestamp::now();
        self.record_event(event);
        Ok(())
    }
}
```

---

### Session Module

| Pattern | Occurrences | Priority | Action |
|---------|-------------|----------|--------|
| Command handler structure | 3+ handlers | High | `CommandHandler` trait |
| Ownership validation | 5+ places | High | `AuthorizationHelper` |
| DomainEvent impl | 5 events | High | `domain_event!` macro |
| Event envelope creation | 3+ handlers | Medium | `EventEnvelope::from_event()` |
| Pull-and-publish pattern | 3 handlers | Medium | `publish_domain_events()` helper |

---

### Cycle Module

| Pattern | Occurrences | Priority | Action |
|---------|-------------|----------|--------|
| 9-variant match blocks | 23 instances | Critical | `delegate_to_variant!` macro |
| Authorization check | 6+ handlers | High | `AuthorizationHelper` |
| Ordering logic | 5 patterns | High | `ComponentSequence` type |
| Event publishing | 5+ handlers | Medium | `EventEnvelope::from_event()` |
| Branching component copy | 2-3 places | Medium | `CycleBranchBuilder` |

**ComponentSequence Value Object:**

```rust
// cycle/component_sequence.rs

pub struct ComponentSequence;

impl ComponentSequence {
    pub const ORDER: [ComponentType; 9] = [
        ComponentType::IssueRaising,
        ComponentType::ProblemFrame,
        ComponentType::Objectives,
        ComponentType::Alternatives,
        ComponentType::Consequences,
        ComponentType::Tradeoffs,
        ComponentType::Recommendation,
        ComponentType::DecisionQuality,
        ComponentType::NotesNextSteps,
    ];

    pub fn order_index(ct: ComponentType) -> usize {
        Self::ORDER.iter().position(|&c| c == ct).unwrap()
    }

    pub fn next(ct: ComponentType) -> Option<ComponentType> {
        let idx = Self::order_index(ct);
        Self::ORDER.get(idx + 1).copied()
    }

    pub fn previous(ct: ComponentType) -> Option<ComponentType> {
        let idx = Self::order_index(ct);
        if idx > 0 { Self::ORDER.get(idx - 1).copied() } else { None }
    }

    pub fn is_before(a: ComponentType, b: ComponentType) -> bool {
        Self::order_index(a) < Self::order_index(b)
    }

    pub fn components_up_to(ct: ComponentType) -> Vec<ComponentType> {
        let idx = Self::order_index(ct);
        Self::ORDER[..=idx].to_vec()
    }

    pub fn prerequisite(ct: ComponentType) -> Option<ComponentType> {
        Self::previous(ct)
    }
}
```

---

### Conversation Module

| Pattern | Occurrences | Priority | Action |
|---------|-------------|----------|--------|
| Handler dependencies | 5 handlers | High | `ConversationHandlerDeps` struct |
| Authorization + logging | 4+ handlers | High | `AuthorizationHelper` |
| EventEnvelope creation | 8 places | High | `EventEnvelope::from_event()` |
| AI request building | 3+ handlers | Medium | `PromptBuilder` |
| Message exchange cycle | 2+ handlers | Medium | `MessageExchangeExecutor` |

**Handler Dependencies Bundle:**

```rust
pub struct ConversationHandlerDeps {
    pub conversation_repo: Arc<dyn ConversationRepository>,
    pub cycle_repo: Arc<dyn CycleRepository>,
    pub session_repo: Arc<dyn SessionRepository>,
    pub ai_provider: Arc<dyn AIProvider>,
    pub event_publisher: Arc<dyn EventPublisher>,
    pub auth_helper: Arc<AuthorizationHelper>,
}

// All handlers receive this single struct instead of 5 separate fields
impl SendMessageHandler {
    pub fn new(deps: ConversationHandlerDeps) -> Self {
        Self { deps }
    }
}
```

---

### Analysis Module

| Pattern | Occurrences | Priority | Action |
|---------|-------------|----------|--------|
| Cell rating extraction | 5+ places | High | `CellAccessor` trait |
| Comparison loops | 3 patterns | High | `compare_across_objectives()` |
| Threshold filtering | 4+ places | Medium | `ThresholdFilter` |
| Min/max operations | 3+ places | Medium | `find_extrema()` |
| ID lookup mapping | 2+ places | Low | `IdLookup<T>` |

**CellAccessor Trait:**

```rust
pub trait CellAccessor {
    fn get_rating(&self, alt_id: &str, obj_id: &str) -> i8;
    fn get_rating_or_default(&self, alt_id: &str, obj_id: &str, default: i8) -> i8 {
        self.get_rating(alt_id, obj_id)
    }
}

impl CellAccessor for ConsequencesTable {
    fn get_rating(&self, alt_id: &str, obj_id: &str) -> i8 {
        self.get_cell(alt_id, obj_id)
            .map(|c| c.rating.value())
            .unwrap_or(0)
    }
}
```

---

### Dashboard Module

| Pattern | Occurrences | Priority | Action |
|---------|-------------|----------|--------|
| CacheEntry variants | 6 types | Critical | Generic `CacheEntry<T>` |
| Cache CRUD operations | 6 regions | Critical | `CacheRegion<ID, T>` trait |
| Event handlers | 15+ handlers | High | `EventHandlerRegistry` |
| Read-through pattern | 3+ places | Medium | `CacheLoader<T>` |
| Eviction policy | 3 regions | Medium | `EvictionPolicy<T>` trait |

**EventHandlerRegistry:**

```rust
pub struct EventHandlerRegistry {
    handlers: HashMap<String, Vec<Box<dyn Fn(&EventEnvelope) -> Result<(), DomainError> + Send + Sync>>>,
}

impl EventHandlerRegistry {
    pub fn register<F>(&mut self, event_type: &str, handler: F)
    where
        F: Fn(&EventEnvelope) -> Result<(), DomainError> + Send + Sync + 'static,
    {
        self.handlers
            .entry(event_type.to_string())
            .or_default()
            .push(Box::new(handler));
    }

    pub async fn dispatch(&self, event: &EventEnvelope) -> Result<(), DomainError> {
        if let Some(handlers) = self.handlers.get(&event.event_type) {
            for handler in handlers {
                handler(event)?;
            }
        }
        Ok(())
    }
}

// Usage:
registry.register("session.created", |event| {
    let payload: SessionCreated = event.parse_payload()?;
    cache.sessions.create(payload.into());
    Ok(())
});
```

---

### Infrastructure Layer

| Pattern | Occurrences | Priority | Action |
|---------|-------------|----------|--------|
| Service error types | 6+ enums | High | `ServiceError<T>` trait |
| Token bucket algorithm | 3 implementations | High | Shared `TokenBucket` |
| Configuration parsing | 3+ configs | Medium | `ConfigBuilder<T>` |
| Connection pooling | 3+ pools | Medium | `PooledResource<T>` trait |
| SSRF validation | 2 adapters | Medium | `SSRFValidator` utility |

**ServiceError Trait:**

```rust
pub trait ServiceError: std::error::Error + Send + Sync {
    fn code(&self) -> ErrorCode;
    fn is_retryable(&self) -> bool;
    fn retry_after_secs(&self) -> Option<u32>;

    fn to_domain_error(&self) -> DomainError {
        DomainError::new(self.code(), self.to_string())
    }
}

// Implementations for AIError, RateLimitError, CircuitBreakerError, etc.
```

---

## Cross-Module Patterns

### Pattern Summary

| Pattern | Modules Affected | Priority |
|---------|-----------------|----------|
| UUID-based IDs | ALL (8+) | Critical |
| DomainEvent impl | ALL (25+ events) | Critical |
| Status state machines | 5 modules | High |
| Command handler structure | 4 modules | High |
| Repository CRUD | 5 modules | High |
| Ownership authorization | 4 modules | High |
| Event envelope building | ALL handlers | High |
| ErrorCode HTTP mapping | ALL adapters | Medium |
| Timestamp management | ALL aggregates | Medium |

---

## Proposed Foundation Exports

After implementing abstractions, `foundation/mod.rs` should export:

```rust
// ============================================
// foundation/mod.rs - After DRY Improvements
// ============================================

// === Macros ===
pub use macros::{declare_uuid_id, domain_event};

// === Value Objects ===
pub use ids::*;           // SessionId, CycleId, ComponentId, etc.
pub use timestamp::{Timestamp, Timestamped};
pub use percentage::Percentage;
pub use rating::Rating;

// === Enums ===
pub use component_type::ComponentType;
pub use component_status::ComponentStatus;
pub use session_status::SessionStatus;
pub use cycle_status::CycleStatus;

// === Events ===
pub use events::{EventId, EventMetadata, EventEnvelope, DomainEvent};

// === Errors ===
pub use errors::{ValidationError, ErrorCode, DomainError};

// === Traits ===
pub use state_machine::StateMachine;
pub use ownership::OwnedByUser;
pub use command::{CommandHandler, CommandMetadata};
pub use repository::Repository;

// === Re-exports for convenience ===
pub use uuid::Uuid;
pub use chrono::{DateTime, Utc};
```

---

## Implementation Roadmap

### Phase 1: Foundation Macros (Week 1)

**Goal:** Establish core macros before any module implementation

1. **`declare_uuid_id!` macro** - `foundation/ids.rs`
   - Refactor existing SessionId, CycleId, ComponentId to use macro
   - Add macro documentation and examples

2. **`domain_event!` macro** - `foundation/events.rs`
   - Create macro with field mapping
   - Add `EventEnvelope::from_event()` helper
   - Add `EventEnvelope::parse_payload()` helper

3. **`StateMachine` trait** - `foundation/state_machine.rs`
   - Define trait with `can_transition_to()` and `transition_to()`
   - Implement for ComponentStatus, SessionStatus, CycleStatus

4. **`ErrorCode::to_http_status()` method** - `foundation/errors.rs`
   - Add HTTP status mapping to ErrorCode enum
   - Document mapping rationale

5. **`OwnedByUser` trait** - `foundation/ownership.rs`
   - Define trait with `owner_id()`, `is_owner()`, `check_ownership()`
   - Default implementations for common cases

### Phase 2: PrOACT & Cycle Consolidation (Week 2)

**Goal:** Eliminate massive boilerplate in component handling

1. **`#[derive(Component)]` proc macro** - `proact_types/macros.rs`
   - Generate Component trait impl
   - Generate Default, output accessors, JSON conversion
   - Test with all 9 components

2. **`delegate_to_variant!` macro** - `cycle/macros.rs`
   - Handle 9-variant ComponentVariant delegation
   - Support both `&self` and `&mut self` methods
   - Support methods with arguments

3. **`ComponentSequence` value object** - `cycle/component_sequence.rs`
   - Consolidate ordering logic
   - Provide next/previous/prerequisite queries
   - Use const array for ordering

4. **Component test macros** - `proact_types/test_helpers.rs`
   - `component_type_test!()`
   - `json_roundtrip_test!()`
   - `output_mutation_test!()`

### Phase 3: Handler Patterns (Week 3)

**Goal:** Standardize command/event handler infrastructure

1. **`AuthorizationHelper` service** - `foundation/authorization.rs`
   - Session access authorization
   - Cycle access authorization (via session)
   - Component access authorization
   - Audit logging integration

2. **`CommandMetadata` standard type** - `foundation/command.rs`
   - Define correlation_id, user_id, trace_id
   - Add builder pattern
   - Document usage in handlers

3. **`Repository<T, ID>` base trait** - `foundation/repository.rs`
   - Define CRUD interface
   - Add default `exists()` implementation
   - Module repos extend base trait

4. **`ConversationHandlerDeps` bundle** - `conversation/handlers.rs`
   - Single struct for all handler dependencies
   - Reduces constructor boilerplate

### Phase 4: Infrastructure & Caching (Week 4)

**Goal:** Consolidate infrastructure patterns

1. **`CacheEntry<T>` generic** - `dashboard/cache.rs`
   - Replace 6 specific cache entry types
   - Include staleness checking
   - Include version tracking

2. **`CacheRegion<ID, T>` trait** - `dashboard/cache.rs`
   - Generic CRUD for cache regions
   - Eviction support
   - Metrics/counting

3. **`TokenBucket` algorithm** - `adapters/infrastructure/`
   - Shared implementation
   - Pluggable storage backend
   - Used by rate limiter, AI tokens, WebSocket limits

4. **`ServiceError<T>` trait** - `foundation/errors.rs`
   - Common interface for adapter errors
   - Retry semantics
   - Domain error conversion

---

## Summary

### Key Metrics

| Metric | Value |
|--------|-------|
| Total patterns identified | 89 |
| Estimated lines saved | ~5,000+ |
| Modules affected | All 8 + infrastructure |
| Critical priority items | 3 macros |
| High priority items | 7 abstractions |

### Three Tiers of Abstraction

1. **Mechanical Boilerplate (60% of savings)**
   - UUID IDs, event impls, component traits
   - Pure repetition with zero variation
   - **Solution:** Macros eliminate entirely

2. **Structural Patterns (25% of savings)**
   - State machines, repositories, authorization
   - Same shape with different parameters
   - **Solution:** Traits provide contract, implementations vary

3. **Behavioral Patterns (15% of savings)**
   - Token buckets, caching, failover
   - Same algorithm in different contexts
   - **Solution:** Extract to shared utilities

### Architectural Insight

The codebase correctly follows hexagonal architecture - most duplication is in domain *scaffolding* (IDs, events, errors) not domain *logic*. This is healthy! The proposed abstractions maintain hexagonal boundaries by staying within the foundation/domain layers.

### Recommended Action

**Start with Phase 1 (foundation macros) before implementing any modules.** The `declare_uuid_id!` and `domain_event!` macros alone will save ~1,200 lines of repetitive code and establish consistent patterns for all subsequent development.

---

*Report generated by multi-agent DRY analysis on 2026-01-08*
