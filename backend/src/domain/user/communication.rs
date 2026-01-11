//! Communication preferences for agent interaction

use serde::{Deserialize, Serialize};

/// Preference level for various interaction aspects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreferenceLevel {
    Minimal,
    Low,
    Medium,
    High,
    Extensive,
}

impl Default for PreferenceLevel {
    fn default() -> Self {
        Self::Medium
    }
}

impl std::fmt::Display for PreferenceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Minimal => write!(f, "Minimal"),
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Extensive => write!(f, "Extensive"),
        }
    }
}

/// Style of challenging user assumptions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeStyle {
    Gentle,
    DevilsAdvocate,
    Socratic,
    Direct,
    Collaborative,
}

impl Default for ChallengeStyle {
    fn default() -> Self {
        Self::Collaborative
    }
}

impl std::fmt::Display for ChallengeStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gentle => write!(f, "Gentle"),
            Self::DevilsAdvocate => write!(f, "Devil's Advocate"),
            Self::Socratic => write!(f, "Socratic"),
            Self::Direct => write!(f, "Direct"),
            Self::Collaborative => write!(f, "Collaborative"),
        }
    }
}

/// Conversation pacing preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PacingPreference {
    Quick,
    Steady,
    Thorough,
    UserControlled,
}

impl Default for PacingPreference {
    fn default() -> Self {
        Self::Steady
    }
}

impl std::fmt::Display for PacingPreference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quick => write!(f, "Quick"),
            Self::Steady => write!(f, "Steady"),
            Self::Thorough => write!(f, "Thorough"),
            Self::UserControlled => write!(f, "User Controlled"),
        }
    }
}

/// How to handle uncertainty in responses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UncertaintyStyle {
    /// Say "I don't know" directly
    Explicit,
    /// Give confidence percentages
    Probabilistic,
    /// Use qualifiers
    Hedged,
    /// Turn into questions
    Exploratory,
}

impl Default for UncertaintyStyle {
    fn default() -> Self {
        Self::Explicit
    }
}

impl std::fmt::Display for UncertaintyStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Explicit => write!(f, "Explicit"),
            Self::Probabilistic => write!(f, "Probabilistic"),
            Self::Hedged => write!(f, "Hedged"),
            Self::Exploratory => write!(f, "Exploratory"),
        }
    }
}

/// Interaction style settings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteractionStyle {
    /// How much context before questions
    pub preamble_preference: PreferenceLevel,
    /// How to challenge assumptions
    pub challenge_style: ChallengeStyle,
    /// Depth of explanations
    pub explanation_depth: PreferenceLevel,
    /// Conversation pacing
    pub pacing: PacingPreference,
    /// How to handle uncertainty
    pub uncertainty_handling: UncertaintyStyle,
}

impl Default for InteractionStyle {
    fn default() -> Self {
        Self {
            preamble_preference: PreferenceLevel::default(),
            challenge_style: ChallengeStyle::default(),
            explanation_depth: PreferenceLevel::default(),
            pacing: PacingPreference::default(),
            uncertainty_handling: UncertaintyStyle::default(),
        }
    }
}

/// Complete communication preferences
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommunicationPreferences {
    /// Interaction style settings
    pub interaction_style: InteractionStyle,
    /// Language patterns that resonate
    pub positive_patterns: Vec<String>,
    /// Language patterns to avoid
    pub negative_patterns: Vec<String>,
    /// Number of sessions analyzed
    pub learned_from_sessions: u32,
}

impl CommunicationPreferences {
    pub fn new(
        interaction_style: InteractionStyle,
        positive_patterns: Vec<String>,
        negative_patterns: Vec<String>,
        learned_from_sessions: u32,
    ) -> Self {
        Self {
            interaction_style,
            positive_patterns,
            negative_patterns,
            learned_from_sessions,
        }
    }

    /// Add a positive pattern
    pub fn add_positive_pattern(&mut self, pattern: String) {
        if !self.positive_patterns.contains(&pattern) {
            self.positive_patterns.push(pattern);
        }
    }

    /// Add a negative pattern
    pub fn add_negative_pattern(&mut self, pattern: String) {
        if !self.negative_patterns.contains(&pattern) {
            self.negative_patterns.push(pattern);
        }
    }

    /// Increment sessions analyzed
    pub fn increment_sessions(&mut self) {
        self.learned_from_sessions += 1;
    }
}

impl Default for CommunicationPreferences {
    fn default() -> Self {
        Self {
            interaction_style: InteractionStyle::default(),
            positive_patterns: Vec::new(),
            negative_patterns: Vec::new(),
            learned_from_sessions: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preference_level_display() {
        assert_eq!(format!("{}", PreferenceLevel::Minimal), "Minimal");
        assert_eq!(format!("{}", PreferenceLevel::Medium), "Medium");
        assert_eq!(format!("{}", PreferenceLevel::Extensive), "Extensive");
    }

    #[test]
    fn test_challenge_style_display() {
        assert_eq!(format!("{}", ChallengeStyle::Gentle), "Gentle");
        assert_eq!(
            format!("{}", ChallengeStyle::DevilsAdvocate),
            "Devil's Advocate"
        );
        assert_eq!(format!("{}", ChallengeStyle::Socratic), "Socratic");
    }

    #[test]
    fn test_pacing_preference_display() {
        assert_eq!(format!("{}", PacingPreference::Quick), "Quick");
        assert_eq!(format!("{}", PacingPreference::Steady), "Steady");
        assert_eq!(
            format!("{}", PacingPreference::UserControlled),
            "User Controlled"
        );
    }

    #[test]
    fn test_uncertainty_style_display() {
        assert_eq!(format!("{}", UncertaintyStyle::Explicit), "Explicit");
        assert_eq!(
            format!("{}", UncertaintyStyle::Probabilistic),
            "Probabilistic"
        );
        assert_eq!(format!("{}", UncertaintyStyle::Hedged), "Hedged");
    }

    #[test]
    fn test_interaction_style_default() {
        let style = InteractionStyle::default();
        assert_eq!(style.preamble_preference, PreferenceLevel::Medium);
        assert_eq!(style.challenge_style, ChallengeStyle::Collaborative);
        assert_eq!(style.explanation_depth, PreferenceLevel::Medium);
        assert_eq!(style.pacing, PacingPreference::Steady);
        assert_eq!(style.uncertainty_handling, UncertaintyStyle::Explicit);
    }

    #[test]
    fn test_communication_preferences_default() {
        let prefs = CommunicationPreferences::default();
        assert_eq!(prefs.positive_patterns.len(), 0);
        assert_eq!(prefs.negative_patterns.len(), 0);
        assert_eq!(prefs.learned_from_sessions, 0);
    }

    #[test]
    fn test_communication_preferences_add_positive_pattern() {
        let mut prefs = CommunicationPreferences::default();

        prefs.add_positive_pattern("concrete examples".to_string());
        prefs.add_positive_pattern("data-driven".to_string());

        assert_eq!(prefs.positive_patterns.len(), 2);
        assert!(prefs.positive_patterns.contains(&"concrete examples".to_string()));
    }

    #[test]
    fn test_communication_preferences_no_duplicate_patterns() {
        let mut prefs = CommunicationPreferences::default();

        prefs.add_positive_pattern("test".to_string());
        prefs.add_positive_pattern("test".to_string());

        assert_eq!(prefs.positive_patterns.len(), 1);
    }

    #[test]
    fn test_communication_preferences_add_negative_pattern() {
        let mut prefs = CommunicationPreferences::default();

        prefs.add_negative_pattern("generic advice".to_string());
        prefs.add_negative_pattern("excessive qualifications".to_string());

        assert_eq!(prefs.negative_patterns.len(), 2);
        assert!(prefs
            .negative_patterns
            .contains(&"generic advice".to_string()));
    }

    #[test]
    fn test_communication_preferences_increment_sessions() {
        let mut prefs = CommunicationPreferences::default();

        assert_eq!(prefs.learned_from_sessions, 0);

        prefs.increment_sessions();
        assert_eq!(prefs.learned_from_sessions, 1);

        prefs.increment_sessions();
        prefs.increment_sessions();
        assert_eq!(prefs.learned_from_sessions, 3);
    }

    #[test]
    fn test_communication_preferences_creation() {
        let style = InteractionStyle {
            preamble_preference: PreferenceLevel::Minimal,
            challenge_style: ChallengeStyle::DevilsAdvocate,
            explanation_depth: PreferenceLevel::Medium,
            pacing: PacingPreference::Quick,
            uncertainty_handling: UncertaintyStyle::Probabilistic,
        };

        let prefs = CommunicationPreferences::new(
            style,
            vec!["pattern1".to_string()],
            vec!["pattern2".to_string()],
            5,
        );

        assert_eq!(prefs.interaction_style.preamble_preference, PreferenceLevel::Minimal);
        assert_eq!(prefs.learned_from_sessions, 5);
    }
}
