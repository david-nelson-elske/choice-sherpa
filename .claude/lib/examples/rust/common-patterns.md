# Rust Common Patterns

## Newtype Pattern (Strong Typing)

```rust
// Wrap primitives to prevent mixing
pub struct UserId(Uuid);
pub struct SessionId(Uuid);
pub struct Money(i64);  // Cents

// Can't accidentally pass SessionId where UserId expected
fn get_user(id: UserId) -> Option<User>;

// Use macro for ID types (project convention)
declare_uuid_id!(CycleId);
declare_uuid_id!(ComponentId);
```

## Builder Pattern

```rust
pub struct SessionBuilder {
    user_id: Option<UserId>,
    title: Option<String>,
    metadata: HashMap<String, String>,
}

impl SessionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn user_id(mut self, id: UserId) -> Self {
        self.user_id = Some(id);
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn build(self) -> Result<Session, BuilderError> {
        Ok(Session {
            user_id: self.user_id.ok_or(BuilderError::MissingUserId)?,
            title: self.title.unwrap_or_default(),
            metadata: self.metadata,
        })
    }
}
```

## State Machine Pattern

```rust
// Encode valid states in the type system
pub enum CycleStatus {
    Draft,
    InProgress,
    Completed,
    Archived,
}

impl CycleStatus {
    pub fn can_transition_to(&self, target: &CycleStatus) -> bool {
        matches!(
            (self, target),
            (Self::Draft, Self::InProgress)
                | (Self::InProgress, Self::Completed)
                | (Self::Completed, Self::Archived)
        )
    }

    pub fn transition_to(self, target: CycleStatus) -> Result<CycleStatus, StatusError> {
        if self.can_transition_to(&target) {
            Ok(target)
        } else {
            Err(StatusError::InvalidTransition { from: self, to: target })
        }
    }
}
```

## Repository Pattern

```rust
#[async_trait]
pub trait Repository<T, ID> {
    async fn find(&self, id: ID) -> Result<Option<T>, RepositoryError>;
    async fn save(&self, entity: &T) -> Result<(), RepositoryError>;
    async fn delete(&self, id: ID) -> Result<(), RepositoryError>;
}

// Domain-specific repository
#[async_trait]
pub trait CycleRepository: Repository<Cycle, CycleId> {
    async fn find_by_session(&self, session_id: SessionId) -> Result<Vec<Cycle>, RepositoryError>;
}
```

## Option Combinators

```rust
// Transform Option values
let name = user.name.as_ref().map(|n| n.to_uppercase());

// Provide defaults
let name = user.name.unwrap_or_else(|| "Anonymous".to_string());

// Chain operations
let display_name = user
    .nickname
    .or(user.first_name.as_ref().map(|s| s.clone()))
    .unwrap_or_else(|| "Unknown".to_string());

// Filter
let adult = user.age.filter(|&age| age >= 18);
```

## Iterator Patterns

```rust
// Map and collect
let ids: Vec<CycleId> = cycles.iter().map(|c| c.id).collect();

// Filter and collect
let active: Vec<&Cycle> = cycles.iter().filter(|c| c.is_active()).collect();

// Find first matching
let first_draft = cycles.iter().find(|c| c.status == CycleStatus::Draft);

// Check if any/all match
let has_active = cycles.iter().any(|c| c.is_active());
let all_complete = cycles.iter().all(|c| c.is_complete());

// Fold/reduce
let total: i64 = items.iter().map(|i| i.amount).sum();
```

## Enum Dispatch

```rust
// Use enum variants instead of trait objects when types are known
pub enum ComponentVariant {
    IssueRaising(IssueRaisingComponent),
    ProblemFrame(ProblemFrameComponent),
    Objectives(ObjectivesComponent),
    // ...
}

impl ComponentVariant {
    pub fn component_type(&self) -> ComponentType {
        match self {
            Self::IssueRaising(_) => ComponentType::IssueRaising,
            Self::ProblemFrame(_) => ComponentType::ProblemFrame,
            // ...
        }
    }
}
```

## From/Into Conversions

```rust
// Implement From for easy conversions
impl From<CreateSessionInput> for Session {
    fn from(input: CreateSessionInput) -> Self {
        Session {
            id: SessionId::new(),
            title: input.title,
            user_id: input.user_id,
            status: SessionStatus::Active,
        }
    }
}

// Usage
let session: Session = input.into();
let session = Session::from(input);
```

## Smart Constructor Pattern

```rust
pub struct NonEmptyString(String);

impl NonEmptyString {
    pub fn new(s: impl Into<String>) -> Result<Self, ValidationError> {
        let s = s.into();
        if s.trim().is_empty() {
            Err(ValidationError::Empty)
        } else {
            Ok(Self(s))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

## Derive Macros

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]  // Common derives
pub struct Entity { ... }

#[derive(Serialize, Deserialize)]  // For JSON/API
pub struct ApiResponse { ... }

#[derive(sqlx::FromRow)]  // For database mapping
pub struct DbRow { ... }
```
