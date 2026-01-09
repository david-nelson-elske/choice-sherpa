# Cross-Decision Intelligence: Decision Profile

**Module:** user (new)
**Type:** Feature Enhancement
**Priority:** P3 (Phase 4 of Agent-Native Enrichments)
**Status:** Specification
**Version:** 1.0.0
**Created:** 2026-01-09
**Based on:** [Agent-Native Enrichments](../../docs/architecture/AGENT-NATIVE-ENRICHMENTS.md) - Suggestion 5

---

## Executive Summary

The Decision Profile is a persistent, user-owned artifact that captures decision-making patterns, preferences, and tendencies across multiple sessions. It enables the AI agent to provide increasingly personalized guidance while maintaining strict privacy controls.

### Key Benefits

| Benefit | Description |
|---------|-------------|
| **Personalization** | Agent adapts to user's decision-making style over time |
| **Pattern Recognition** | Surface blind spots and recurring themes |
| **Continuity** | Context persists across sessions without re-explanation |
| **Self-Awareness** | Users gain insight into their own decision patterns |
| **Privacy-First** | User owns, controls, and can delete their data |

---

## Core Concepts

### The Decision Profile

A markdown file that evolves with each completed decision cycle:

```markdown
# Decision Profile: david@example.com

> Last Updated: 2026-01-09 | Decisions Analyzed: 12 | Profile Confidence: High

---

## Risk Profile

**Classification:** Risk-Averse (78% confidence)

| Dimension | Score | Evidence |
|-----------|-------|----------|
| Financial Risk Tolerance | Low (2/5) | Consistently chooses stable income over equity upside |
| Career Risk Tolerance | Medium (3/5) | Willing to change roles but prefers established companies |
| Time Risk Tolerance | Low (2/5) | Prioritizes near-term certainty over long-term potential |
| Relationship Risk Tolerance | High (4/5) | Values authentic expression over conflict avoidance |

**Behavioral Indicators:**
- Uses phrases like "play it safe", "what's the downside?" frequently
- Requests more information before high-stakes decisions
- Tends to overweight worst-case scenarios in consequence ratings
- 8/12 decisions chose the lower-variance option

---

## Values & Priorities

### Consistent Objectives (appear in 60%+ of decisions)

| Objective | Frequency | Typical Weight |
|-----------|-----------|----------------|
| Work-life balance | 83% | High |
| Financial security | 75% | High |
| Family impact | 67% | High |
| Professional growth | 58% | Medium |
| Personal autonomy | 50% | Medium |

### Value Tensions Observed

| Tension | Resolution Pattern |
|---------|-------------------|
| Growth vs Stability | Usually chooses stability unless growth gap is large (>2x) |
| Income vs Time | Willing to trade 15-20% income for significant time gains |
| Short-term vs Long-term | Defaults to short-term unless explicitly prompted |

---

## Decision-Making Style

**Primary Style:** Analytical-Cautious

| Dimension | Tendency | Strength |
|-----------|----------|----------|
| Information Gathering | Thorough | Strong |
| Analysis Paralysis Risk | Moderate | Watch for this |
| Intuition Trust | Low | Relies heavily on data |
| Stakeholder Consideration | High | Always considers family |
| Reversibility Weighting | High | Strongly prefers reversible choices |

**Cognitive Patterns:**
- Anchoring: Tends to anchor on first number mentioned (salary, price)
- Loss Aversion: ~2.5x more sensitive to losses than equivalent gains
- Status Quo Bias: Moderate - needs clear evidence to change
- Confirmation Bias: Low - actively seeks disconfirming evidence

---

## Blind Spots & Growth Areas

### Identified Blind Spots

| Blind Spot | Evidence | Agent Behavior |
|------------|----------|----------------|
| Underweights long-term compounding | Career decisions focus on 1-2 year horizon | Prompt: "What does this look like in 10 years?" |
| Overconfident in familiar domains | Higher DQ variance in "comfort zone" decisions | Challenge assumptions more in familiar areas |
| Neglects opportunity cost | Focuses on direct consequences | Ask: "What are you giving up by choosing this?" |

### Growth Observed

| Area | Before | After | Trigger |
|------|--------|-------|---------|
| Considering stakeholders | Often forgot spouse input | Now automatic | Explicit prompt after job decision |
| Quantifying objectives | Vague measures | Specific metrics | DQ feedback on measurement |

---

## Communication Preferences

**Interaction Style:**

| Preference | Setting | Notes |
|------------|---------|-------|
| Preamble length | Minimal | Prefers direct questions over context-setting |
| Challenge style | Devil's advocate | Responds well to pushback |
| Explanation depth | Medium | Wants reasoning but not exhaustive |
| Pacing | Steady | Doesn't like being rushed, but also not drawn out |
| Uncertainty handling | Explicit | Prefers "I don't know" over hedging |

**Language Patterns:**
- Responds positively to: concrete examples, data, trade-off framing
- Responds negatively to: generic advice, excessive qualifications, emotional appeals

---

## Decision History

### Recent Decisions

| Date | Decision | Domain | DQ Score | Key Tradeoff | Outcome |
|------|----------|--------|----------|--------------|---------|
| 2026-01 | VP role at StartupCo | Career | 82% | Growth vs Stability | Pending |
| 2025-11 | New car purchase | Financial | 75% | Cost vs Features | Satisfied |
| 2025-08 | School district move | Family | 88% | Schools vs Commute | Very satisfied |
| 2025-05 | Accept tech lead role | Career | 85% | Growth vs Work-life | Satisfied |

### Decision Patterns by Domain

| Domain | Decisions | Avg DQ | Success Rate | Notes |
|--------|-----------|--------|--------------|-------|
| Career | 5 | 84% | 80% | Strongest analysis |
| Financial | 4 | 72% | 75% | Tends to overthink |
| Family | 2 | 86% | 100% | Clear values help |
| Health | 1 | 68% | -- | Limited data |

### Outcome Tracking

| Prediction Accuracy | Value |
|---------------------|-------|
| Consequence predictions | 73% accurate |
| Satisfaction predictions | 81% accurate |
| Timeline predictions | 65% accurate |

---

## Agent Instructions

Based on this profile, the agent should:

1. **Challenge risk aversion** when potential upside is significant
2. **Prompt for long-term thinking** explicitly in career/financial decisions
3. **Skip lengthy preambles** - get to questions quickly
4. **Always ask about family impact** - it's a consistent priority
5. **Use devil's advocate** approach - user responds well
6. **Watch for analysis paralysis** - may need to push toward decision
7. **Highlight opportunity costs** - user tends to miss these

---

*Profile Version: 3.2*
*Last Analysis: 2026-01-09*
*Next Suggested Review: After 3 more decisions*
```

---

## Profile Components

### 1. Risk Profile

The Risk Profile categorizes the user's risk tolerance across multiple dimensions.

#### Risk Classification

| Classification | Description | Behavioral Indicators |
|---------------|-------------|----------------------|
| **Risk-Seeking** | Actively pursues high-variance options | Chooses options with higher potential upside despite uncertainty |
| **Risk-Neutral** | Evaluates options purely on expected value | No systematic preference for or against variance |
| **Risk-Averse** | Prefers certainty over equivalent expected value | Chooses lower-variance options, asks about downsides |

#### Risk Dimensions

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskProfile {
    /// Overall risk classification
    pub classification: RiskClassification,

    /// Confidence in the classification (0.0 - 1.0)
    pub confidence: f32,

    /// Domain-specific risk tolerances
    pub dimensions: RiskDimensions,

    /// Behavioral evidence supporting classification
    pub evidence: Vec<RiskEvidence>,

    /// Last updated timestamp
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskClassification {
    RiskSeeking,
    RiskNeutral,
    RiskAverse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDimensions {
    /// Tolerance for financial uncertainty (1-5 scale)
    pub financial: RiskScore,

    /// Tolerance for career/professional uncertainty
    pub career: RiskScore,

    /// Tolerance for time-horizon uncertainty
    pub temporal: RiskScore,

    /// Tolerance for relationship/social uncertainty
    pub relational: RiskScore,

    /// Tolerance for health/safety uncertainty
    pub health: RiskScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    pub value: u8,  // 1-5 scale
    pub confidence: f32,
    pub sample_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEvidence {
    pub decision_id: CycleId,
    pub indicator_type: RiskIndicatorType,
    pub description: String,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskIndicatorType {
    OptionChoice,       // Which alternative was chosen
    LanguagePattern,    // Words used in conversation
    InformationSeeking, // How much info requested before deciding
    ConsequenceRating,  // How they rated uncertain outcomes
    TimePreference,     // Near-term vs long-term focus
}
```

#### Risk Assessment Methods

| Method | Description | Weight |
|--------|-------------|--------|
| **Choice Analysis** | Which alternatives were chosen (high vs low variance) | 40% |
| **Language Patterns** | "What's the downside?", "play it safe", "go for it" | 25% |
| **Consequence Ratings** | How they rate uncertain vs certain outcomes | 20% |
| **Information Seeking** | How much info requested before high-stakes decisions | 15% |

---

### 2. Values & Priorities

Tracks objectives that appear consistently across decisions.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuesPriorities {
    /// Objectives that appear frequently
    pub consistent_objectives: Vec<ConsistentObjective>,

    /// Observed tensions between values
    pub value_tensions: Vec<ValueTension>,

    /// Domain-specific value patterns
    pub domain_patterns: HashMap<DecisionDomain, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistentObjective {
    pub name: String,
    pub frequency: f32,  // 0.0 - 1.0, percentage of decisions
    pub typical_weight: ObjectiveWeight,
    pub first_seen: Timestamp,
    pub last_seen: Timestamp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveWeight {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueTension {
    pub value_a: String,
    pub value_b: String,
    pub resolution_pattern: String,
    pub examples: Vec<CycleId>,
}
```

---

### 3. Decision-Making Style

Captures how the user approaches decisions.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionMakingStyle {
    /// Primary decision-making approach
    pub primary_style: StyleClassification,

    /// Dimensional tendencies
    pub dimensions: StyleDimensions,

    /// Identified cognitive biases
    pub cognitive_patterns: Vec<CognitivePattern>,

    /// Confidence level
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StyleClassification {
    AnalyticalCautious,    // Data-driven, careful
    AnalyticalDynamic,     // Data-driven, action-oriented
    IntuitiveCautious,     // Gut-feel, careful
    IntuitiveDynamic,      // Gut-feel, action-oriented
    Balanced,              // Mix of approaches
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleDimensions {
    /// How much information is gathered before deciding
    pub information_gathering: DimensionScore,

    /// Tendency to over-analyze
    pub analysis_paralysis_risk: DimensionScore,

    /// Trust in gut feelings
    pub intuition_trust: DimensionScore,

    /// Consideration of others affected
    pub stakeholder_consideration: DimensionScore,

    /// Weight given to reversibility
    pub reversibility_weighting: DimensionScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionScore {
    pub level: DimensionLevel,
    pub strength: StrengthLevel,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DimensionLevel {
    VeryLow,
    Low,
    Moderate,
    High,
    VeryHigh,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrengthLevel {
    Weak,
    Moderate,
    Strong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitivePattern {
    pub bias_type: CognitiveBiasType,
    pub severity: SeverityLevel,
    pub evidence: String,
    pub mitigation_prompt: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CognitiveBiasType {
    Anchoring,
    LossAversion,
    StatusQuoBias,
    ConfirmationBias,
    OverconfidenceBias,
    AvailabilityBias,
    SunkCostFallacy,
    PlanningFallacy,
}
```

---

### 4. Blind Spots & Growth Areas

Tracks areas for improvement and observed growth.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindSpotsGrowth {
    /// Identified blind spots
    pub blind_spots: Vec<BlindSpot>,

    /// Observed improvements over time
    pub growth_areas: Vec<GrowthObservation>,

    /// Suggested focus areas
    pub suggested_focus: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindSpot {
    pub name: String,
    pub description: String,
    pub evidence: Vec<String>,
    pub agent_behavior: String,  // What the agent should do
    pub identified_at: Timestamp,
    pub still_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthObservation {
    pub area: String,
    pub before_behavior: String,
    pub after_behavior: String,
    pub trigger: String,
    pub observed_at: Timestamp,
}
```

---

### 5. Communication Preferences

Captures how the user prefers to interact.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationPreferences {
    /// Interaction style settings
    pub interaction_style: InteractionStyle,

    /// Language patterns that resonate
    pub positive_patterns: Vec<String>,

    /// Language patterns to avoid
    pub negative_patterns: Vec<String>,

    /// Learned from conversation history
    pub learned_from_sessions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreferenceLevel {
    Minimal,
    Low,
    Medium,
    High,
    Extensive,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChallengeStyle {
    Gentle,
    DevilsAdvocate,
    Socratic,
    Direct,
    Collaborative,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PacingPreference {
    Quick,
    Steady,
    Thorough,
    UserControlled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UncertaintyStyle {
    Explicit,        // Say "I don't know" directly
    Probabilistic,   // Give confidence percentages
    Hedged,          // Use qualifiers
    Exploratory,     // Turn into questions
}
```

---

### 6. Decision History

Tracks past decisions and outcomes.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionHistory {
    /// Individual decision records
    pub decisions: Vec<DecisionRecord>,

    /// Aggregated patterns by domain
    pub domain_patterns: HashMap<DecisionDomain, DomainStats>,

    /// Outcome tracking accuracy
    pub prediction_accuracy: PredictionAccuracy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub cycle_id: CycleId,
    pub date: Timestamp,
    pub title: String,
    pub domain: DecisionDomain,
    pub dq_score: u8,
    pub key_tradeoff: String,
    pub chosen_alternative: String,
    pub outcome: Option<OutcomeRecord>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionDomain {
    Career,
    Financial,
    Family,
    Health,
    Relationship,
    Education,
    Housing,
    Lifestyle,
    Business,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeRecord {
    pub recorded_at: Timestamp,
    pub satisfaction: SatisfactionLevel,
    pub actual_consequences: String,
    pub surprises: Vec<String>,
    pub would_decide_same: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SatisfactionLevel {
    VeryDissatisfied,
    Dissatisfied,
    Neutral,
    Satisfied,
    VerySatisfied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainStats {
    pub decision_count: u32,
    pub average_dq: f32,
    pub success_rate: f32,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionAccuracy {
    pub consequence_accuracy: f32,
    pub satisfaction_accuracy: f32,
    pub timeline_accuracy: f32,
    pub sample_size: u32,
}
```

---

## Domain Model

### DecisionProfile Entity

```rust
use crate::foundation::{UserId, Timestamp};
use serde::{Deserialize, Serialize};

/// DecisionProfile is a user-owned aggregate that persists across sessions
#[derive(Debug, Clone)]
pub struct DecisionProfile {
    // Identity
    id: DecisionProfileId,
    user_id: UserId,

    // Profile components
    risk_profile: RiskProfile,
    values_priorities: ValuesPriorities,
    decision_style: DecisionMakingStyle,
    blind_spots_growth: BlindSpotsGrowth,
    communication_prefs: CommunicationPreferences,
    decision_history: DecisionHistory,

    // Metadata
    version: ProfileVersion,
    created_at: Timestamp,
    updated_at: Timestamp,
    decisions_analyzed: u32,
    profile_confidence: ProfileConfidence,

    // Privacy
    consent: ProfileConsent,

    // Domain events
    domain_events: Vec<DomainEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DecisionProfileId(uuid::Uuid);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProfileVersion(pub u32);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileConfidence {
    Low,       // < 3 decisions analyzed
    Medium,    // 3-7 decisions
    High,      // 8-15 decisions
    VeryHigh,  // 15+ decisions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConsent {
    pub collection_enabled: bool,
    pub analysis_enabled: bool,
    pub agent_access_enabled: bool,
    pub consented_at: Timestamp,
    pub last_reviewed: Timestamp,
}
```

### Domain Invariants

1. **User Ownership**: Each profile belongs to exactly one user
2. **Consent Required**: Profile cannot be created without explicit consent
3. **Minimum Data**: Risk classification requires at least 3 decisions
4. **Confidence Scaling**: Profile confidence increases with more decisions
5. **Version Monotonicity**: Profile version only increases
6. **Privacy Control**: User can disable/delete at any time

---

## Domain Events

| Event | Trigger | Data |
|-------|---------|------|
| `DecisionProfileCreated` | User enables profiling | user_id, profile_id, consent |
| `DecisionProfileUpdated` | Cycle completed | profile_id, version, changes_summary |
| `RiskProfileRecalculated` | New decision data | profile_id, old_classification, new_classification |
| `BlindSpotIdentified` | Pattern detected | profile_id, blind_spot_name, evidence |
| `GrowthObserved` | Improvement detected | profile_id, area, before, after |
| `OutcomeRecorded` | User provides feedback | profile_id, decision_id, outcome |
| `ProfileDeleted` | User requests deletion | user_id, profile_id |
| `ProfileExported` | User exports data | profile_id, format |

---

## Storage Architecture

### Dual Storage (Consistent with Decision Document)

```
┌─────────────────────────────────────────────────────────────────┐
│                     PROFILE STORAGE                              │
├─────────────────────────────────────────────────────────────────┤
│   FILESYSTEM (Human-Readable)       DATABASE (Structured Query) │
│   ════════════════════════════      ════════════════════════════│
│                                                                  │
│   /profiles/{user_id}/              decision_profiles table     │
│   └── profile.md                    ├── risk_profile JSONB      │
│                                     ├── values_priorities JSONB │
│   Markdown format                   ├── decision_style JSONB    │
│   User can read/edit                ├── history (separate table)│
│   Exportable                        └── indexes for queries     │
│                                                                  │
│   Benefits:                         Benefits:                    │
│   • User transparency               • Fast queries              │
│   • External backup                 • Cross-user analytics      │
│   • Git versioning                  • Pattern detection         │
│   • Portable                        • Search                    │
└─────────────────────────────────────────────────────────────────┘
```

### Database Schema

```sql
-- User decision profiles
CREATE TABLE decision_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL UNIQUE,

    -- File reference
    file_path VARCHAR(500) NOT NULL,
    content_checksum VARCHAR(64) NOT NULL,

    -- Profile version
    version INTEGER NOT NULL DEFAULT 1,

    -- Core profile data (JSONB for flexibility)
    risk_profile JSONB NOT NULL DEFAULT '{}',
    values_priorities JSONB NOT NULL DEFAULT '{}',
    decision_style JSONB NOT NULL DEFAULT '{}',
    blind_spots_growth JSONB NOT NULL DEFAULT '{}',
    communication_prefs JSONB NOT NULL DEFAULT '{}',

    -- Aggregates
    decisions_analyzed INTEGER NOT NULL DEFAULT 0,
    profile_confidence VARCHAR(20) NOT NULL DEFAULT 'low',

    -- Privacy consent
    consent JSONB NOT NULL,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT valid_confidence CHECK (profile_confidence IN ('low', 'medium', 'high', 'very_high'))
);

-- Decision history (separate for efficient queries)
CREATE TABLE profile_decision_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id UUID NOT NULL REFERENCES decision_profiles(id) ON DELETE CASCADE,
    cycle_id UUID NOT NULL REFERENCES cycles(id),

    -- Decision metadata
    decision_date TIMESTAMPTZ NOT NULL,
    title VARCHAR(500) NOT NULL,
    domain VARCHAR(50) NOT NULL,
    dq_score INTEGER,
    key_tradeoff TEXT,
    chosen_alternative VARCHAR(500),

    -- Outcome tracking (filled in later)
    outcome_recorded_at TIMESTAMPTZ,
    satisfaction VARCHAR(20),
    actual_consequences TEXT,
    would_decide_same BOOLEAN,

    -- Analysis data
    risk_indicators JSONB,
    objectives_used JSONB,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(profile_id, cycle_id)
);

-- Indexes
CREATE INDEX idx_profiles_user ON decision_profiles(user_id);
CREATE INDEX idx_profile_history_profile ON profile_decision_history(profile_id);
CREATE INDEX idx_profile_history_domain ON profile_decision_history(domain);
CREATE INDEX idx_profile_history_date ON profile_decision_history(decision_date DESC);

-- Risk profile specific index (for aggregate queries)
CREATE INDEX idx_profiles_risk ON decision_profiles
    USING GIN ((risk_profile->'classification'));
```

---

## Ports

### ProfileRepository Port

```rust
#[async_trait]
pub trait ProfileRepository: Send + Sync {
    /// Create new profile (requires consent)
    async fn create(&self, profile: &DecisionProfile) -> Result<(), DomainError>;

    /// Update existing profile
    async fn update(&self, profile: &DecisionProfile) -> Result<(), DomainError>;

    /// Find by user ID
    async fn find_by_user(&self, user_id: &UserId) -> Result<Option<DecisionProfile>, DomainError>;

    /// Delete profile (user request)
    async fn delete(&self, profile_id: DecisionProfileId) -> Result<(), DomainError>;

    /// Export profile in various formats
    async fn export(&self, profile_id: DecisionProfileId, format: ExportFormat) -> Result<Vec<u8>, DomainError>;
}
```

### ProfileAnalyzer Port

```rust
#[async_trait]
pub trait ProfileAnalyzer: Send + Sync {
    /// Analyze a completed cycle and update profile
    async fn analyze_decision(
        &self,
        profile: &mut DecisionProfile,
        cycle: &Cycle,
        conversation_history: &[Message],
    ) -> Result<AnalysisResult, DomainError>;

    /// Recalculate risk profile from history
    fn recalculate_risk_profile(
        &self,
        history: &DecisionHistory,
    ) -> Result<RiskProfile, DomainError>;

    /// Detect cognitive biases from patterns
    fn detect_cognitive_patterns(
        &self,
        history: &DecisionHistory,
        conversations: &[ConversationSummary],
    ) -> Result<Vec<CognitivePattern>, DomainError>;

    /// Identify blind spots
    fn identify_blind_spots(
        &self,
        profile: &DecisionProfile,
    ) -> Result<Vec<BlindSpot>, DomainError>;

    /// Generate agent instructions from profile
    fn generate_agent_instructions(
        &self,
        profile: &DecisionProfile,
    ) -> Result<AgentInstructions, DomainError>;
}

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub risk_profile_changed: bool,
    pub new_patterns_detected: Vec<String>,
    pub blind_spots_identified: Vec<BlindSpot>,
    pub growth_observed: Vec<GrowthObservation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInstructions {
    pub risk_guidance: String,
    pub blind_spot_prompts: Vec<String>,
    pub communication_adjustments: Vec<String>,
    pub suggested_questions: Vec<String>,
}
```

### ProfileFileStorage Port

```rust
#[async_trait]
pub trait ProfileFileStorage: Send + Sync {
    /// Write profile to filesystem as markdown
    async fn write(&self, user_id: &UserId, content: &str) -> Result<FilePath, StorageError>;

    /// Read profile markdown from filesystem
    async fn read(&self, user_id: &UserId) -> Result<String, StorageError>;

    /// Check if profile file exists
    async fn exists(&self, user_id: &UserId) -> Result<bool, StorageError>;

    /// Delete profile file
    async fn delete(&self, user_id: &UserId) -> Result<(), StorageError>;
}
```

---

## Application Layer

### Commands

#### CreateProfile

```rust
#[derive(Debug, Clone)]
pub struct CreateProfileCommand {
    pub user_id: UserId,
    pub consent: ProfileConsent,
}

pub struct CreateProfileHandler {
    profile_repo: Arc<dyn ProfileRepository>,
    file_storage: Arc<dyn ProfileFileStorage>,
    generator: Arc<dyn ProfileGenerator>,
    publisher: Arc<dyn DomainEventPublisher>,
}

impl CreateProfileHandler {
    pub async fn handle(&self, cmd: CreateProfileCommand) -> Result<DecisionProfileId, DomainError> {
        // 1. Verify consent is valid
        if !cmd.consent.collection_enabled {
            return Err(DomainError::validation("Consent required for profile creation"));
        }

        // 2. Check if profile already exists
        if self.profile_repo.find_by_user(&cmd.user_id).await?.is_some() {
            return Err(DomainError::conflict("Profile already exists"));
        }

        // 3. Create empty profile
        let profile = DecisionProfile::new(cmd.user_id.clone(), cmd.consent)?;

        // 4. Generate initial markdown
        let content = self.generator.generate_markdown(&profile)?;

        // 5. Save to filesystem and database
        self.file_storage.write(&cmd.user_id, &content).await?;
        self.profile_repo.create(&profile).await?;

        // 6. Publish event
        self.publisher.publish(vec![
            DomainEvent::DecisionProfileCreated {
                user_id: cmd.user_id,
                profile_id: profile.id(),
            }
        ]).await?;

        Ok(profile.id())
    }
}
```

#### UpdateProfileFromDecision

```rust
#[derive(Debug, Clone)]
pub struct UpdateProfileFromDecisionCommand {
    pub user_id: UserId,
    pub cycle_id: CycleId,
}

pub struct UpdateProfileFromDecisionHandler {
    profile_repo: Arc<dyn ProfileRepository>,
    cycle_reader: Arc<dyn CycleReader>,
    conversation_reader: Arc<dyn ConversationReader>,
    analyzer: Arc<dyn ProfileAnalyzer>,
    file_storage: Arc<dyn ProfileFileStorage>,
    generator: Arc<dyn ProfileGenerator>,
    publisher: Arc<dyn DomainEventPublisher>,
}

impl UpdateProfileFromDecisionHandler {
    pub async fn handle(&self, cmd: UpdateProfileFromDecisionCommand) -> Result<AnalysisResult, DomainError> {
        // 1. Load profile (must exist and have consent)
        let mut profile = self.profile_repo.find_by_user(&cmd.user_id).await?
            .ok_or_else(|| DomainError::not_found("profile"))?;

        if !profile.consent().analysis_enabled {
            return Err(DomainError::forbidden("Analysis consent not granted"));
        }

        // 2. Load completed cycle
        let cycle = self.cycle_reader.get_by_id(cmd.cycle_id).await?
            .ok_or_else(|| DomainError::not_found("cycle"))?;

        // 3. Load conversation history
        let conversations = self.conversation_reader
            .get_by_cycle(cmd.cycle_id).await?;

        // 4. Analyze and update profile
        let result = self.analyzer.analyze_decision(
            &mut profile,
            &cycle,
            &conversations,
        ).await?;

        // 5. Regenerate markdown
        let content = self.generator.generate_markdown(&profile)?;

        // 6. Save updates
        self.file_storage.write(&cmd.user_id, &content).await?;
        self.profile_repo.update(&profile).await?;

        // 7. Publish events
        let mut events = profile.pull_domain_events();
        self.publisher.publish(events).await?;

        Ok(result)
    }
}
```

#### RecordOutcome

```rust
#[derive(Debug, Clone)]
pub struct RecordOutcomeCommand {
    pub user_id: UserId,
    pub cycle_id: CycleId,
    pub satisfaction: SatisfactionLevel,
    pub actual_consequences: String,
    pub surprises: Vec<String>,
    pub would_decide_same: bool,
}

// Handler updates decision history with actual outcome
```

#### DeleteProfile

```rust
#[derive(Debug, Clone)]
pub struct DeleteProfileCommand {
    pub user_id: UserId,
    pub confirmation: String,  // Must match "DELETE MY PROFILE"
}

// Handler removes all profile data from filesystem and database
```

### Queries

#### GetProfileSummary

```rust
#[derive(Debug)]
pub struct GetProfileSummaryQuery {
    pub user_id: UserId,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfileSummary {
    pub risk_classification: RiskClassification,
    pub risk_confidence: f32,
    pub decisions_analyzed: u32,
    pub profile_confidence: ProfileConfidence,
    pub top_values: Vec<String>,
    pub decision_style: StyleClassification,
    pub active_blind_spots: Vec<String>,
}
```

#### GetAgentInstructions

```rust
#[derive(Debug)]
pub struct GetAgentInstructionsQuery {
    pub user_id: UserId,
    pub decision_domain: Option<DecisionDomain>,
}

// Returns customized agent behavior instructions based on profile
```

---

## Agent Integration

### System Prompt Enhancement

When a user with a profile starts a session, the agent receives additional context:

```markdown
## User Profile Context

**Risk Profile:** Risk-Averse (High confidence)
- Financial: Low tolerance - tends to prefer stable options
- Career: Medium tolerance - open to changes at established companies

**Key Values:** Work-life balance (83%), Financial security (75%), Family impact (67%)

**Communication Style:**
- Prefers direct questions, minimal preambles
- Responds well to devil's advocate challenges
- Wants specific examples and data

**Blind Spots to Address:**
1. Tends to underweight long-term compounding - prompt for 10-year view
2. Often misses opportunity costs - explicitly ask "What are you giving up?"
3. May anchor on first numbers mentioned - introduce alternative anchors

**Domain-Specific Notes (Career):**
- Average DQ: 84% (strongest domain)
- Tends to analyze thoroughly - watch for analysis paralysis
- Always considers family impact - don't skip this

**Suggested Approach:**
1. Get to questions quickly
2. Challenge risk-averse defaults when upside is significant
3. Explicitly prompt for long-term thinking
4. Use devil's advocate sparingly but effectively
```

### Tool Integration

The profile can inform atomic decision tools:

```rust
// When rating consequences, agent can reference risk profile
fn suggest_consequence_challenge(
    profile: &DecisionProfile,
    rating: &ConsequenceRating,
) -> Option<String> {
    if profile.risk_profile.classification == RiskClassification::RiskAverse {
        if rating.is_optimistic() {
            return Some("Given your typical caution with uncertainty, \
                        are you sure this rating isn't optimistic?".to_string());
        }
    }
    None
}
```

---

## Privacy & Security

### Data Minimization

| Data | Collected | Purpose | Retention |
|------|-----------|---------|-----------|
| Decision titles | Yes | Context for patterns | Until deletion |
| Full document content | No | Not needed for profiling | Never stored |
| DQ scores | Yes | Pattern analysis | Until deletion |
| Conversation text | Summary only | Communication style | Summarized |
| Choice rationale | Yes | Risk/bias analysis | Until deletion |

### User Controls

| Control | Implementation |
|---------|----------------|
| **View Profile** | Full markdown document accessible via UI or filesystem |
| **Edit Profile** | Users can modify their profile directly |
| **Pause Collection** | Disable analysis without deleting |
| **Export Data** | Markdown, JSON, or PDF export |
| **Delete Everything** | Complete removal from filesystem and database |

### Consent Flow

```
1. User attempts first decision completion
   ↓
2. System prompts: "Would you like to enable Decision Intelligence?"
   - Explains what's collected
   - Explains benefits
   - Offers privacy policy link
   ↓
3. User chooses:
   - Enable all (collection + analysis + agent access)
   - Enable limited (collection only, no agent access)
   - Decline (no profile created)
   ↓
4. Consent recorded with timestamp
5. User can change settings anytime
```

---

## HTTP Endpoints

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| `POST` | `/api/profile` | CreateProfile | Create profile with consent |
| `GET` | `/api/profile` | GetProfile | Get user's profile |
| `GET` | `/api/profile/summary` | GetProfileSummary | Get profile summary for UI |
| `PUT` | `/api/profile/consent` | UpdateConsent | Update consent settings |
| `POST` | `/api/profile/outcome` | RecordOutcome | Record decision outcome |
| `GET` | `/api/profile/export` | ExportProfile | Export profile in format |
| `DELETE` | `/api/profile` | DeleteProfile | Delete all profile data |

---

## Frontend Components

### ProfileDashboard.svelte

Display profile insights with visualizations:
- Risk profile radar chart
- Values priority chart
- Decision history timeline
- Blind spots cards
- Growth achievements

### ProfileSettings.svelte

Manage privacy and consent:
- Toggle collection/analysis/agent access
- View what's being collected
- Export data
- Delete profile

### OnboardingConsent.svelte

First-time consent flow:
- Clear explanation of benefits
- Privacy commitments
- Granular consent options

---

## Implementation Phases

### Phase 1: Foundation
- DecisionProfile entity and value objects
- Database schema
- File storage adapter
- Basic CRUD operations

### Phase 2: Collection
- Post-decision analysis trigger
- Risk indicator extraction
- Values/objectives tracking
- Decision history recording

### Phase 3: Analysis
- Risk classification algorithm
- Cognitive bias detection
- Blind spot identification
- Communication preference learning

### Phase 4: Agent Integration
- Profile-to-instructions generator
- System prompt injection
- Tool-level guidance
- Feedback loop (agent can flag patterns)

### Phase 5: User Experience
- Profile dashboard UI
- Consent management
- Export functionality
- Outcome recording

---

## Related Documents

- [Agent-Native Enrichments Analysis](../../docs/architecture/AGENT-NATIVE-ENRICHMENTS.md)
- [Decision Document Specification](../cycle/decision-document.md)
- [Conversation Module](../../docs/modules/conversation.md)

---

*Specification Version: 1.0.0*
*Created: 2026-01-09*
*Author: Claude Opus 4.5*
