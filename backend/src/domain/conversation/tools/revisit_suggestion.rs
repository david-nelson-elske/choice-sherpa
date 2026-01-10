//! Revisit suggestion entity - queued suggestions for component revisits.
//!
//! When the AI agent notices something in the conversation that suggests
//! an earlier component should be revisited, it creates a RevisitSuggestion.
//! These are queued (not immediately acted upon) to respect linear PrOACT flow.

use serde::{Deserialize, Serialize};

use crate::domain::foundation::{
    ComponentType, CycleId, RevisitSuggestionId, Timestamp,
};

/// Priority level for revisit suggestions.
///
/// Helps the user triage which revisits to address first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevisitPriority {
    /// Nice to have - minor refinement opportunity
    Low,
    /// Recommended - would improve decision quality
    Medium,
    /// Important - significant gap identified
    High,
    /// Decision quality at risk - should address before proceeding
    Critical,
}

impl RevisitPriority {
    /// Returns true if this priority is high or critical.
    pub fn is_urgent(&self) -> bool {
        matches!(self, Self::High | Self::Critical)
    }

    /// Returns a numeric weight for sorting (higher = more urgent).
    pub fn weight(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }
}

impl std::fmt::Display for RevisitPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Status of a revisit suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionStatus {
    /// Awaiting user decision
    Pending,
    /// User chose to revisit the component
    Accepted,
    /// User chose not to revisit
    Dismissed,
    /// Decision completed without addressing this suggestion
    Expired,
}

impl SuggestionStatus {
    /// Returns true if the suggestion is still actionable.
    pub fn is_open(&self) -> bool {
        matches!(self, Self::Pending)
    }

    /// Returns true if the suggestion has been resolved (any outcome).
    pub fn is_resolved(&self) -> bool {
        !self.is_open()
    }
}

impl std::fmt::Display for SuggestionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Accepted => write!(f, "Accepted"),
            Self::Dismissed => write!(f, "Dismissed"),
            Self::Expired => write!(f, "Expired"),
        }
    }
}

/// A suggestion to revisit an earlier component.
///
/// Created by the AI agent when conversation reveals that an earlier
/// component might need refinement. Suggestions are queued rather than
/// immediately acted upon to preserve linear PrOACT flow.
///
/// # Lifecycle
///
/// 1. Agent creates suggestion via `suggest_revisit` tool
/// 2. Suggestion is stored with Pending status
/// 3. User sees suggestions after completing current component
/// 4. User accepts, dismisses, or lets it expire
///
/// # Example
///
/// ```ignore
/// let suggestion = RevisitSuggestion::new(
///     cycle_id,
///     ComponentType::Objectives,
///     "User mentioned 'reliability' which isn't captured as an objective",
///     "User said: 'I really care about reliability'",
///     RevisitPriority::High,
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisitSuggestion {
    /// Unique identifier
    id: RevisitSuggestionId,

    /// The cycle this suggestion belongs to
    cycle_id: CycleId,

    /// The component to revisit
    target_component: ComponentType,

    /// Why this component should be revisited
    reason: String,

    /// What in the conversation triggered this suggestion
    trigger: String,

    /// How important is this revisit
    priority: RevisitPriority,

    /// Current status
    status: SuggestionStatus,

    /// When the suggestion was created
    created_at: Timestamp,

    /// When the suggestion was resolved (if resolved)
    resolved_at: Option<Timestamp>,

    /// User's reason for accepting/dismissing (if any)
    resolution: Option<String>,
}

impl RevisitSuggestion {
    /// Creates a new pending revisit suggestion.
    pub fn new(
        cycle_id: CycleId,
        target_component: ComponentType,
        reason: impl Into<String>,
        trigger: impl Into<String>,
        priority: RevisitPriority,
    ) -> Self {
        Self {
            id: RevisitSuggestionId::new(),
            cycle_id,
            target_component,
            reason: reason.into(),
            trigger: trigger.into(),
            priority,
            status: SuggestionStatus::Pending,
            created_at: Timestamp::now(),
            resolved_at: None,
            resolution: None,
        }
    }

    /// Accepts the suggestion (user will revisit the component).
    pub fn accept(&mut self, resolution: Option<String>) {
        debug_assert!(self.status.is_open(), "Cannot accept resolved suggestion");
        self.status = SuggestionStatus::Accepted;
        self.resolved_at = Some(Timestamp::now());
        self.resolution = resolution;
    }

    /// Dismisses the suggestion (user chose not to revisit).
    pub fn dismiss(&mut self, reason: impl Into<String>) {
        debug_assert!(self.status.is_open(), "Cannot dismiss resolved suggestion");
        self.status = SuggestionStatus::Dismissed;
        self.resolved_at = Some(Timestamp::now());
        self.resolution = Some(reason.into());
    }

    /// Expires the suggestion (decision completed without addressing).
    pub fn expire(&mut self) {
        debug_assert!(self.status.is_open(), "Cannot expire resolved suggestion");
        self.status = SuggestionStatus::Expired;
        self.resolved_at = Some(Timestamp::now());
        self.resolution = Some("Decision completed without addressing".into());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Getters
    // ═══════════════════════════════════════════════════════════════════════

    /// Returns the unique identifier.
    pub fn id(&self) -> RevisitSuggestionId {
        self.id
    }

    /// Returns the cycle ID.
    pub fn cycle_id(&self) -> CycleId {
        self.cycle_id
    }

    /// Returns the target component to revisit.
    pub fn target_component(&self) -> ComponentType {
        self.target_component
    }

    /// Returns why this component should be revisited.
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Returns what triggered this suggestion.
    pub fn trigger(&self) -> &str {
        &self.trigger
    }

    /// Returns the priority level.
    pub fn priority(&self) -> RevisitPriority {
        self.priority
    }

    /// Returns the current status.
    pub fn status(&self) -> SuggestionStatus {
        self.status
    }

    /// Returns when the suggestion was created.
    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    /// Returns when the suggestion was resolved (if any).
    pub fn resolved_at(&self) -> Option<Timestamp> {
        self.resolved_at
    }

    /// Returns the resolution reason (if any).
    pub fn resolution(&self) -> Option<&str> {
        self.resolution.as_deref()
    }

    /// Returns true if this is a pending suggestion.
    pub fn is_pending(&self) -> bool {
        self.status.is_open()
    }

    /// Returns true if this is an urgent suggestion.
    pub fn is_urgent(&self) -> bool {
        self.priority.is_urgent()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Reconstitution (for loading from storage)
    // ═══════════════════════════════════════════════════════════════════════

    /// Reconstitutes a RevisitSuggestion from stored data.
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: RevisitSuggestionId,
        cycle_id: CycleId,
        target_component: ComponentType,
        reason: String,
        trigger: String,
        priority: RevisitPriority,
        status: SuggestionStatus,
        created_at: Timestamp,
        resolved_at: Option<Timestamp>,
        resolution: Option<String>,
    ) -> Self {
        Self {
            id,
            cycle_id,
            target_component,
            reason,
            trigger,
            priority,
            status,
            created_at,
            resolved_at,
            resolution,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cycle_id() -> CycleId {
        CycleId::new()
    }

    #[test]
    fn new_creates_pending_suggestion() {
        let suggestion = RevisitSuggestion::new(
            test_cycle_id(),
            ComponentType::Objectives,
            "Missing objective",
            "User mentioned reliability",
            RevisitPriority::High,
        );

        assert_eq!(suggestion.status(), SuggestionStatus::Pending);
        assert!(suggestion.is_pending());
        assert!(suggestion.resolved_at().is_none());
    }

    #[test]
    fn accept_changes_status() {
        let mut suggestion = RevisitSuggestion::new(
            test_cycle_id(),
            ComponentType::Objectives,
            "reason",
            "trigger",
            RevisitPriority::Medium,
        );

        suggestion.accept(Some("Will address".into()));

        assert_eq!(suggestion.status(), SuggestionStatus::Accepted);
        assert!(!suggestion.is_pending());
        assert!(suggestion.resolved_at().is_some());
        assert_eq!(suggestion.resolution(), Some("Will address"));
    }

    #[test]
    fn dismiss_changes_status() {
        let mut suggestion = RevisitSuggestion::new(
            test_cycle_id(),
            ComponentType::ProblemFrame,
            "reason",
            "trigger",
            RevisitPriority::Low,
        );

        suggestion.dismiss("Not relevant");

        assert_eq!(suggestion.status(), SuggestionStatus::Dismissed);
        assert!(!suggestion.is_pending());
        assert_eq!(suggestion.resolution(), Some("Not relevant"));
    }

    #[test]
    fn expire_changes_status() {
        let mut suggestion = RevisitSuggestion::new(
            test_cycle_id(),
            ComponentType::Alternatives,
            "reason",
            "trigger",
            RevisitPriority::Low,
        );

        suggestion.expire();

        assert_eq!(suggestion.status(), SuggestionStatus::Expired);
        assert!(!suggestion.is_pending());
    }

    #[test]
    fn is_urgent_for_high_and_critical() {
        let high = RevisitSuggestion::new(
            test_cycle_id(),
            ComponentType::Objectives,
            "r",
            "t",
            RevisitPriority::High,
        );
        let critical = RevisitSuggestion::new(
            test_cycle_id(),
            ComponentType::Objectives,
            "r",
            "t",
            RevisitPriority::Critical,
        );
        let medium = RevisitSuggestion::new(
            test_cycle_id(),
            ComponentType::Objectives,
            "r",
            "t",
            RevisitPriority::Medium,
        );

        assert!(high.is_urgent());
        assert!(critical.is_urgent());
        assert!(!medium.is_urgent());
    }

    #[test]
    fn priority_weight_ordering() {
        assert!(RevisitPriority::Critical.weight() > RevisitPriority::High.weight());
        assert!(RevisitPriority::High.weight() > RevisitPriority::Medium.weight());
        assert!(RevisitPriority::Medium.weight() > RevisitPriority::Low.weight());
    }

    #[test]
    fn serializes_to_json() {
        let suggestion = RevisitSuggestion::new(
            test_cycle_id(),
            ComponentType::Consequences,
            "Missing ratings",
            "User skipped cells",
            RevisitPriority::Medium,
        );

        let json = serde_json::to_string(&suggestion).unwrap();
        assert!(json.contains("consequences"));
        assert!(json.contains("pending"));
        assert!(json.contains("medium"));
    }

    #[test]
    fn id_is_unique() {
        let s1 = RevisitSuggestion::new(
            test_cycle_id(),
            ComponentType::Objectives,
            "r",
            "t",
            RevisitPriority::Low,
        );
        let s2 = RevisitSuggestion::new(
            test_cycle_id(),
            ComponentType::Objectives,
            "r",
            "t",
            RevisitPriority::Low,
        );

        assert_ne!(s1.id(), s2.id());
    }
}
