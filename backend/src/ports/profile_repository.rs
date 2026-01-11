//! ProfileRepository port for profile persistence operations

use async_trait::async_trait;
use std::fmt;

use crate::domain::{
    foundation::{DomainError, UserId},
    user::{DecisionProfile, DecisionProfileId},
};

/// Export format for profile data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Markdown,
    Json,
    Pdf,
}

impl fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Markdown => write!(f, "markdown"),
            Self::Json => write!(f, "json"),
            Self::Pdf => write!(f, "pdf"),
        }
    }
}

/// Repository for managing decision profiles
#[async_trait]
pub trait ProfileRepository: Send + Sync {
    /// Create a new profile (requires valid consent)
    async fn create(&self, profile: &DecisionProfile) -> Result<(), DomainError>;

    /// Update an existing profile
    async fn update(&self, profile: &DecisionProfile) -> Result<(), DomainError>;

    /// Find profile by user ID
    async fn find_by_user(&self, user_id: &UserId) -> Result<Option<DecisionProfile>, DomainError>;

    /// Find profile by profile ID
    async fn find_by_id(
        &self,
        profile_id: DecisionProfileId,
    ) -> Result<Option<DecisionProfile>, DomainError>;

    /// Delete profile completely (for privacy compliance)
    async fn delete(&self, profile_id: DecisionProfileId) -> Result<(), DomainError>;

    /// Export profile in specified format
    async fn export(
        &self,
        profile_id: DecisionProfileId,
        format: ExportFormat,
    ) -> Result<Vec<u8>, DomainError>;

    /// Check if profile exists for user
    async fn exists_for_user(&self, user_id: &UserId) -> Result<bool, DomainError>;
}
