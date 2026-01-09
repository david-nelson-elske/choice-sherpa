//! Base repository trait for persistence operations.
//!
//! This module provides the generic `Repository<T, ID>` trait that defines
//! the standard CRUD interface for all aggregate repositories.
//!
//! # DRY Pattern
//!
//! Instead of each repository defining its own `find_by_id`, `save`, `update`,
//! `delete` methods with identical signatures, they inherit from this base trait
//! and only add domain-specific query methods.
//!
//! # Example
//!
//! ```ignore
//! // Domain-specific repository extends the base trait
//! #[async_trait]
//! pub trait SessionRepository: Repository<Session, SessionId> {
//!     async fn find_by_user(&self, user_id: &UserId) -> Result<Vec<Session>, DomainError>;
//!     async fn find_active_by_user(&self, user_id: &UserId) -> Result<Vec<Session>, DomainError>;
//! }
//!
//! // The handler only needs to know about the trait
//! pub struct CreateSessionHandler {
//!     repo: Arc<dyn SessionRepository>,
//! }
//! ```

use async_trait::async_trait;
use std::fmt::Debug;

use super::DomainError;

/// Base trait for aggregate repositories.
///
/// Provides standard CRUD operations that all repositories share.
/// Domain-specific repositories should extend this trait with
/// additional query methods.
///
/// # Type Parameters
///
/// - `T`: The aggregate root type being persisted
/// - `ID`: The identifier type for the aggregate (e.g., `SessionId`, `CycleId`)
///
/// # Default Implementation
///
/// The `exists` method has a default implementation that uses `find_by_id`.
/// Implementors may override this if a more efficient check is available.
///
/// # Error Handling
///
/// All methods return `Result<_, DomainError>` to maintain consistency
/// with the domain error system. Implementations should convert adapter-specific
/// errors (e.g., database errors) into appropriate `DomainError` variants.
#[async_trait]
pub trait Repository<T, ID>: Send + Sync
where
    T: Send + Sync,
    ID: Send + Sync + Debug + 'static,
{
    /// Finds an aggregate by its unique identifier.
    ///
    /// Returns `Ok(None)` if the aggregate doesn't exist.
    /// Returns `Err` only for infrastructure failures.
    async fn find_by_id(&self, id: ID) -> Result<Option<T>, DomainError>;

    /// Persists a new aggregate.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The aggregate already exists (duplicate ID)
    /// - Infrastructure failure (database unavailable)
    async fn save(&self, entity: &T) -> Result<(), DomainError>;

    /// Updates an existing aggregate.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The aggregate doesn't exist
    /// - Optimistic locking conflict (if implemented)
    /// - Infrastructure failure
    async fn update(&self, entity: &T) -> Result<(), DomainError>;

    /// Deletes an aggregate by its identifier.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The aggregate doesn't exist
    /// - Referential integrity violation
    /// - Infrastructure failure
    async fn delete(&self, id: ID) -> Result<(), DomainError>;

    /// Checks if an aggregate with the given ID exists.
    ///
    /// Default implementation uses `find_by_id`. Override if a more
    /// efficient existence check is available (e.g., COUNT query).
    async fn exists(&self, id: ID) -> Result<bool, DomainError> {
        Ok(self.find_by_id(id).await?.is_some())
    }
}

/// Extension trait for repositories that support batch operations.
///
/// Not all repositories need batch operations, so this is a separate trait
/// that repositories can optionally implement.
#[async_trait]
pub trait BatchRepository<T, ID>: Repository<T, ID>
where
    T: Send + Sync,
    ID: Send + Sync + Debug + 'static,
{
    /// Finds multiple aggregates by their IDs.
    ///
    /// Returns only the aggregates that exist. Missing IDs are silently skipped.
    /// The order of returned aggregates may not match the order of input IDs.
    async fn find_by_ids(&self, ids: Vec<ID>) -> Result<Vec<T>, DomainError>;

    /// Saves multiple aggregates in a single operation.
    ///
    /// Implementations should make this atomic where possible.
    async fn save_all(&self, entities: &[T]) -> Result<(), DomainError>;

    /// Deletes multiple aggregates in a single operation.
    async fn delete_all(&self, ids: Vec<ID>) -> Result<(), DomainError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Test aggregate and ID types
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestEntity {
        id: TestId,
        name: String,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct TestId(u32);

    // In-memory repository for testing trait behavior
    struct InMemoryTestRepo {
        data: Mutex<HashMap<TestId, TestEntity>>,
    }

    impl InMemoryTestRepo {
        fn new() -> Self {
            Self {
                data: Mutex::new(HashMap::new()),
            }
        }

        fn with_entity(entity: TestEntity) -> Self {
            let repo = Self::new();
            repo.data.lock().unwrap().insert(entity.id, entity);
            repo
        }
    }

    #[async_trait]
    impl Repository<TestEntity, TestId> for InMemoryTestRepo {
        async fn find_by_id(&self, id: TestId) -> Result<Option<TestEntity>, DomainError> {
            Ok(self.data.lock().unwrap().get(&id).cloned())
        }

        async fn save(&self, entity: &TestEntity) -> Result<(), DomainError> {
            let mut data = self.data.lock().unwrap();
            if data.contains_key(&entity.id) {
                return Err(DomainError::new(
                    super::super::ErrorCode::ValidationFailed,
                    "Entity already exists",
                ));
            }
            data.insert(entity.id, entity.clone());
            Ok(())
        }

        async fn update(&self, entity: &TestEntity) -> Result<(), DomainError> {
            let mut data = self.data.lock().unwrap();
            if !data.contains_key(&entity.id) {
                return Err(DomainError::new(
                    super::super::ErrorCode::SessionNotFound,
                    "Entity not found",
                ));
            }
            data.insert(entity.id, entity.clone());
            Ok(())
        }

        async fn delete(&self, id: TestId) -> Result<(), DomainError> {
            let mut data = self.data.lock().unwrap();
            if data.remove(&id).is_none() {
                return Err(DomainError::new(
                    super::super::ErrorCode::SessionNotFound,
                    "Entity not found",
                ));
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn find_by_id_returns_entity_when_exists() {
        let entity = TestEntity {
            id: TestId(1),
            name: "Test".to_string(),
        };
        let repo = InMemoryTestRepo::with_entity(entity.clone());

        let result = repo.find_by_id(TestId(1)).await.unwrap();
        assert_eq!(result, Some(entity));
    }

    #[tokio::test]
    async fn find_by_id_returns_none_when_not_exists() {
        let repo = InMemoryTestRepo::new();

        let result = repo.find_by_id(TestId(999)).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn save_adds_new_entity() {
        let repo = InMemoryTestRepo::new();
        let entity = TestEntity {
            id: TestId(1),
            name: "New".to_string(),
        };

        repo.save(&entity).await.unwrap();

        let found = repo.find_by_id(TestId(1)).await.unwrap();
        assert_eq!(found, Some(entity));
    }

    #[tokio::test]
    async fn save_fails_when_entity_exists() {
        let entity = TestEntity {
            id: TestId(1),
            name: "Existing".to_string(),
        };
        let repo = InMemoryTestRepo::with_entity(entity.clone());

        let result = repo.save(&entity).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn update_modifies_existing_entity() {
        let entity = TestEntity {
            id: TestId(1),
            name: "Original".to_string(),
        };
        let repo = InMemoryTestRepo::with_entity(entity);

        let updated = TestEntity {
            id: TestId(1),
            name: "Updated".to_string(),
        };
        repo.update(&updated).await.unwrap();

        let found = repo.find_by_id(TestId(1)).await.unwrap();
        assert_eq!(found.unwrap().name, "Updated");
    }

    #[tokio::test]
    async fn update_fails_when_entity_not_exists() {
        let repo = InMemoryTestRepo::new();
        let entity = TestEntity {
            id: TestId(999),
            name: "Ghost".to_string(),
        };

        let result = repo.update(&entity).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn delete_removes_entity() {
        let entity = TestEntity {
            id: TestId(1),
            name: "ToDelete".to_string(),
        };
        let repo = InMemoryTestRepo::with_entity(entity);

        repo.delete(TestId(1)).await.unwrap();

        let found = repo.find_by_id(TestId(1)).await.unwrap();
        assert_eq!(found, None);
    }

    #[tokio::test]
    async fn delete_fails_when_entity_not_exists() {
        let repo = InMemoryTestRepo::new();

        let result = repo.delete(TestId(999)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn exists_returns_true_when_entity_exists() {
        let entity = TestEntity {
            id: TestId(1),
            name: "Exists".to_string(),
        };
        let repo = InMemoryTestRepo::with_entity(entity);

        let exists = repo.exists(TestId(1)).await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn exists_returns_false_when_entity_not_exists() {
        let repo = InMemoryTestRepo::new();

        let exists = repo.exists(TestId(999)).await.unwrap();
        assert!(!exists);
    }

    // Compile-time checks
    #[allow(dead_code)]
    fn assert_object_safe(_: &dyn Repository<TestEntity, TestId>) {}

    #[allow(dead_code)]
    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn repository_is_send_sync() {
        fn check<T: Repository<TestEntity, TestId>>() {
            assert_send_sync::<T>();
        }
    }
}
