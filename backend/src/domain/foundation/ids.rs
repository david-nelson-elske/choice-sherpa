//! Strongly-typed identifier value objects.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

use super::ValidationError;

/// Unique identifier for a decision session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(Uuid);

impl SessionId {
    /// Creates a new random SessionId.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a SessionId from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the inner UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SessionId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// Unique identifier for a decision cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CycleId(Uuid);

impl CycleId {
    /// Creates a new random CycleId.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a CycleId from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the inner UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for CycleId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CycleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for CycleId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// Unique identifier for a PrOACT component within a cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ComponentId(Uuid);

impl ComponentId {
    /// Creates a new random ComponentId.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a ComponentId from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the inner UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for ComponentId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ComponentId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// User identifier (typically from auth provider).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(String);

impl UserId {
    /// Creates a new UserId, returning error if empty.
    pub fn new(id: impl Into<String>) -> Result<Self, ValidationError> {
        let id = id.into();
        if id.is_empty() {
            return Err(ValidationError::empty_field("user_id"));
        }
        Ok(Self(id))
    }

    /// Returns the inner string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_generates_unique_values() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn session_id_parses_from_valid_string() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let id: SessionId = uuid_str.parse().unwrap();
        assert_eq!(id.to_string(), uuid_str);
    }

    #[test]
    fn session_id_from_uuid_preserves_value() {
        let uuid = Uuid::new_v4();
        let id = SessionId::from_uuid(uuid);
        assert_eq!(id.as_uuid(), &uuid);
    }

    #[test]
    fn session_id_serializes_to_json() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let id: SessionId = uuid_str.parse().unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, format!("\"{}\"", uuid_str));
    }

    #[test]
    fn cycle_id_generates_unique_values() {
        let id1 = CycleId::new();
        let id2 = CycleId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn component_id_generates_unique_values() {
        let id1 = ComponentId::new();
        let id2 = ComponentId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn user_id_accepts_non_empty_string() {
        let id = UserId::new("user-123").unwrap();
        assert_eq!(id.as_str(), "user-123");
    }

    #[test]
    fn user_id_rejects_empty_string() {
        let result = UserId::new("");
        assert!(result.is_err());
        match result {
            Err(ValidationError::EmptyField { field }) => assert_eq!(field, "user_id"),
            _ => panic!("Expected EmptyField error"),
        }
    }

    #[test]
    fn user_id_displays_correctly() {
        let id = UserId::new("user-456").unwrap();
        assert_eq!(format!("{}", id), "user-456");
    }
}
