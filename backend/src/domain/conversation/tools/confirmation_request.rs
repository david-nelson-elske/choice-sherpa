//! Confirmation request entity - tracks user confirmations requested by agent.
//!
//! When the AI agent needs explicit user confirmation before proceeding
//! (e.g., making assumptions, significant changes), it creates a ConfirmationRequest.
//! The conversation pauses until the user responds.

use serde::{Deserialize, Serialize};

use crate::domain::foundation::{
    ConfirmationRequestId, CycleId, Timestamp,
};

/// Status of a confirmation request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationStatus {
    /// Waiting for user response
    Pending,
    /// User confirmed one of the options
    Confirmed,
    /// User explicitly rejected
    Rejected,
    /// Request expired without response
    Expired,
}

impl ConfirmationStatus {
    /// Returns true if still waiting for response.
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }

    /// Returns true if user confirmed.
    pub fn is_confirmed(&self) -> bool {
        matches!(self, Self::Confirmed)
    }
}

impl std::fmt::Display for ConfirmationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Confirmed => write!(f, "Confirmed"),
            Self::Rejected => write!(f, "Rejected"),
            Self::Expired => write!(f, "Expired"),
        }
    }
}

/// An option presented to the user for confirmation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfirmationOption {
    /// Display label for the option
    pub label: String,
    /// Description of what this option means
    pub description: String,
}

impl ConfirmationOption {
    /// Creates a new confirmation option.
    pub fn new(label: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: description.into(),
        }
    }
}

/// A request for user confirmation.
///
/// Created by the AI agent when it needs explicit user input before
/// proceeding. Common uses:
///
/// - Confirming assumptions about the decision
/// - Approving significant changes to the document
/// - Choosing between multiple interpretations
/// - Validating extracted data
///
/// # Lifecycle
///
/// 1. Agent creates request via `request_confirmation` tool
/// 2. Conversation pauses, user sees options
/// 3. User selects an option or provides custom input
/// 4. Request is marked confirmed/rejected
/// 5. Agent continues based on user's choice
///
/// # Example
///
/// ```ignore
/// let request = ConfirmationRequest::new(
///     cycle_id,
///     5, // conversation turn
///     "I understood your primary objective as 'minimize cost'. Is this correct?",
///     vec![
///         ConfirmationOption::new("Yes", "Cost minimization is the primary objective"),
///         ConfirmationOption::new("No", "Let me clarify the objective"),
///     ],
///     Some(0), // Default to first option
///     Duration::minutes(30), // Expire after 30 minutes
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationRequest {
    /// Unique identifier
    id: ConfirmationRequestId,

    /// The cycle this request belongs to
    cycle_id: CycleId,

    /// Which conversation turn triggered this request
    conversation_turn: u32,

    /// Summary of what needs confirmation
    summary: String,

    /// Available options for the user
    options: Vec<ConfirmationOption>,

    /// Default option index (if any)
    default_option: Option<usize>,

    /// Current status
    status: ConfirmationStatus,

    /// Which option the user chose (if confirmed)
    chosen_option: Option<usize>,

    /// Custom user input (if they provided text instead of choosing)
    user_input: Option<String>,

    /// When the request was created
    requested_at: Timestamp,

    /// When the user responded (if responded)
    responded_at: Option<Timestamp>,

    /// When this request expires
    expires_at: Timestamp,
}

impl ConfirmationRequest {
    /// Creates a new pending confirmation request.
    ///
    /// # Arguments
    ///
    /// * `cycle_id` - The cycle this belongs to
    /// * `conversation_turn` - Which turn created this request
    /// * `summary` - What needs confirmation
    /// * `options` - Available options
    /// * `default_option` - Index of default option (if any)
    /// * `ttl_minutes` - How long until this expires
    pub fn new(
        cycle_id: CycleId,
        conversation_turn: u32,
        summary: impl Into<String>,
        options: Vec<ConfirmationOption>,
        default_option: Option<usize>,
        ttl_minutes: i64,
    ) -> Self {
        let now = Timestamp::now();
        // Add minutes by using days (1 day = 1440 minutes)
        let expires_at = if ttl_minutes >= 1440 {
            now.add_days(ttl_minutes / 1440)
        } else {
            // For shorter durations, just add a day as minimum
            now.add_days(1)
        };

        Self {
            id: ConfirmationRequestId::new(),
            cycle_id,
            conversation_turn,
            summary: summary.into(),
            options,
            default_option,
            status: ConfirmationStatus::Pending,
            chosen_option: None,
            user_input: None,
            requested_at: now,
            responded_at: None,
            expires_at,
        }
    }

    /// User confirms by selecting an option.
    pub fn confirm(&mut self, option_index: usize) {
        debug_assert!(self.status.is_pending(), "Cannot confirm resolved request");
        debug_assert!(option_index < self.options.len(), "Invalid option index");
        self.status = ConfirmationStatus::Confirmed;
        self.chosen_option = Some(option_index);
        self.responded_at = Some(Timestamp::now());
    }

    /// User confirms with custom input instead of predefined option.
    pub fn confirm_with_input(&mut self, input: impl Into<String>) {
        debug_assert!(self.status.is_pending(), "Cannot confirm resolved request");
        self.status = ConfirmationStatus::Confirmed;
        self.user_input = Some(input.into());
        self.responded_at = Some(Timestamp::now());
    }

    /// User rejects the request.
    pub fn reject(&mut self) {
        debug_assert!(self.status.is_pending(), "Cannot reject resolved request");
        self.status = ConfirmationStatus::Rejected;
        self.responded_at = Some(Timestamp::now());
    }

    /// Expires the request (no response received in time).
    pub fn expire(&mut self) {
        debug_assert!(self.status.is_pending(), "Cannot expire resolved request");
        self.status = ConfirmationStatus::Expired;
    }

    /// Checks if this request has expired based on current time.
    pub fn is_expired(&self) -> bool {
        if !self.status.is_pending() {
            return false;
        }
        Timestamp::now().is_after(&self.expires_at)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Getters
    // ═══════════════════════════════════════════════════════════════════════

    /// Returns the unique identifier.
    pub fn id(&self) -> ConfirmationRequestId {
        self.id
    }

    /// Returns the cycle ID.
    pub fn cycle_id(&self) -> CycleId {
        self.cycle_id
    }

    /// Returns the conversation turn.
    pub fn conversation_turn(&self) -> u32 {
        self.conversation_turn
    }

    /// Returns the summary.
    pub fn summary(&self) -> &str {
        &self.summary
    }

    /// Returns the options.
    pub fn options(&self) -> &[ConfirmationOption] {
        &self.options
    }

    /// Returns the default option index.
    pub fn default_option(&self) -> Option<usize> {
        self.default_option
    }

    /// Returns the current status.
    pub fn status(&self) -> ConfirmationStatus {
        self.status
    }

    /// Returns the chosen option index (if confirmed with option).
    pub fn chosen_option(&self) -> Option<usize> {
        self.chosen_option
    }

    /// Returns custom user input (if provided).
    pub fn user_input(&self) -> Option<&str> {
        self.user_input.as_deref()
    }

    /// Returns when the request was created.
    pub fn requested_at(&self) -> Timestamp {
        self.requested_at
    }

    /// Returns when the user responded (if responded).
    pub fn responded_at(&self) -> Option<Timestamp> {
        self.responded_at
    }

    /// Returns when this request expires.
    pub fn expires_at(&self) -> Timestamp {
        self.expires_at
    }

    /// Returns true if still pending.
    pub fn is_pending(&self) -> bool {
        self.status.is_pending()
    }

    /// Returns the chosen option label (if confirmed with option).
    pub fn chosen_option_label(&self) -> Option<&str> {
        self.chosen_option
            .and_then(|idx| self.options.get(idx))
            .map(|opt| opt.label.as_str())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Reconstitution (for loading from storage)
    // ═══════════════════════════════════════════════════════════════════════

    /// Reconstitutes a ConfirmationRequest from stored data.
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub fn reconstitute(
        id: ConfirmationRequestId,
        cycle_id: CycleId,
        conversation_turn: u32,
        summary: String,
        options: Vec<ConfirmationOption>,
        default_option: Option<usize>,
        status: ConfirmationStatus,
        chosen_option: Option<usize>,
        user_input: Option<String>,
        requested_at: Timestamp,
        responded_at: Option<Timestamp>,
        expires_at: Timestamp,
    ) -> Self {
        Self {
            id,
            cycle_id,
            conversation_turn,
            summary,
            options,
            default_option,
            status,
            chosen_option,
            user_input,
            requested_at,
            responded_at,
            expires_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cycle_id() -> CycleId {
        CycleId::new()
    }

    fn test_options() -> Vec<ConfirmationOption> {
        vec![
            ConfirmationOption::new("Yes", "Confirm this action"),
            ConfirmationOption::new("No", "Cancel this action"),
        ]
    }

    #[test]
    fn new_creates_pending_request() {
        let request = ConfirmationRequest::new(
            test_cycle_id(),
            5,
            "Is this correct?",
            test_options(),
            Some(0),
            30,
        );

        assert_eq!(request.status(), ConfirmationStatus::Pending);
        assert!(request.is_pending());
        assert_eq!(request.conversation_turn(), 5);
        assert_eq!(request.options().len(), 2);
    }

    #[test]
    fn confirm_with_option_updates_state() {
        let mut request = ConfirmationRequest::new(
            test_cycle_id(),
            1,
            "Confirm?",
            test_options(),
            None,
            30,
        );

        request.confirm(0);

        assert_eq!(request.status(), ConfirmationStatus::Confirmed);
        assert!(!request.is_pending());
        assert_eq!(request.chosen_option(), Some(0));
        assert!(request.responded_at().is_some());
        assert_eq!(request.chosen_option_label(), Some("Yes"));
    }

    #[test]
    fn confirm_with_input_updates_state() {
        let mut request = ConfirmationRequest::new(
            test_cycle_id(),
            1,
            "What is your objective?",
            test_options(),
            None,
            30,
        );

        request.confirm_with_input("Minimize environmental impact");

        assert_eq!(request.status(), ConfirmationStatus::Confirmed);
        assert_eq!(request.user_input(), Some("Minimize environmental impact"));
        assert!(request.chosen_option().is_none());
    }

    #[test]
    fn reject_updates_state() {
        let mut request = ConfirmationRequest::new(
            test_cycle_id(),
            1,
            "Confirm?",
            test_options(),
            None,
            30,
        );

        request.reject();

        assert_eq!(request.status(), ConfirmationStatus::Rejected);
        assert!(!request.is_pending());
    }

    #[test]
    fn expire_updates_state() {
        let mut request = ConfirmationRequest::new(
            test_cycle_id(),
            1,
            "Confirm?",
            test_options(),
            None,
            30,
        );

        request.expire();

        assert_eq!(request.status(), ConfirmationStatus::Expired);
    }

    #[test]
    fn confirmation_option_creates_correctly() {
        let option = ConfirmationOption::new("Accept", "Accept and continue");

        assert_eq!(option.label, "Accept");
        assert_eq!(option.description, "Accept and continue");
    }

    #[test]
    fn serializes_to_json() {
        let request = ConfirmationRequest::new(
            test_cycle_id(),
            3,
            "Is this the primary decision?",
            test_options(),
            Some(0),
            60,
        );

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("pending"));
        assert!(json.contains("primary decision"));
    }

    #[test]
    fn id_is_unique() {
        let r1 = ConfirmationRequest::new(
            test_cycle_id(),
            1,
            "q",
            test_options(),
            None,
            30,
        );
        let r2 = ConfirmationRequest::new(
            test_cycle_id(),
            1,
            "q",
            test_options(),
            None,
            30,
        );

        assert_ne!(r1.id(), r2.id());
    }
}
