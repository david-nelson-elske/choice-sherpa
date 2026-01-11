//! User module - Decision Profile and Cross-Decision Intelligence
//!
//! This module implements the Decision Profile feature, which captures
//! decision-making patterns, risk tolerance, preferences, and tendencies
//! across multiple sessions to enable personalized AI guidance.
//!
//! # Architecture
//!
//! The DecisionProfile is a user-owned aggregate that persists across
//! sessions. It consists of six main components:
//!
//! - **Risk Profile** - Risk tolerance across multiple dimensions
//! - **Values & Priorities** - Consistent objectives and value tensions
//! - **Decision Style** - Analytical vs intuitive, cautious vs dynamic
//! - **Blind Spots & Growth** - Areas for improvement and observed growth
//! - **Communication Preferences** - How the user prefers to interact
//! - **Decision History** - Past decisions and outcomes for pattern analysis
//!
//! # Domain Invariants
//!
//! 1. Each profile belongs to exactly one user
//! 2. Profile cannot be created without explicit consent
//! 3. Risk classification requires at least 3 decisions
//! 4. Profile confidence increases with more decisions
//! 5. Profile version only increases
//! 6. User can disable/delete at any time

pub mod blind_spots;
pub mod communication;
pub mod decision_style;
pub mod events;
pub mod history;
pub mod profile;
pub mod risk_profile;
pub mod values;

// Re-exports for public API
pub use blind_spots::{BlindSpot, BlindSpotsGrowth, GrowthObservation};
pub use communication::{
    ChallengeStyle, CommunicationPreferences, InteractionStyle, PacingPreference,
    PreferenceLevel, UncertaintyStyle,
};
pub use decision_style::{
    CognitiveBiasType, CognitivePattern, DecisionMakingStyle, DimensionLevel, DimensionScore,
    SeverityLevel, StrengthLevel, StyleClassification, StyleDimensions,
};
pub use events::*;
pub use history::{
    DecisionDomain, DecisionHistory, DecisionRecord, DomainStats, OutcomeRecord,
    PredictionAccuracy, SatisfactionLevel,
};
pub use profile::{
    DecisionProfile, DecisionProfileId, ProfileConfidence, ProfileConsent, ProfileVersion,
};
pub use risk_profile::{
    RiskClassification, RiskDimensions, RiskEvidence, RiskIndicatorType, RiskProfile, RiskScore,
};
pub use values::{ConsistentObjective, ObjectiveWeight, ValueTension, ValuesPriorities};
