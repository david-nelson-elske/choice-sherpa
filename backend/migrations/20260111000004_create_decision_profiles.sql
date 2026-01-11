-- Create decision profiles tables for cross-decision intelligence

-- Main decision profiles table
CREATE TABLE decision_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL UNIQUE,

    -- File reference (markdown file on filesystem)
    file_path VARCHAR(500) NOT NULL,
    content_checksum VARCHAR(64) NOT NULL,

    -- Profile version (monotonically increasing)
    version INTEGER NOT NULL DEFAULT 1,

    -- Core profile data stored as JSONB for flexibility
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
    CONSTRAINT valid_confidence CHECK (profile_confidence IN ('low', 'medium', 'high', 'very_high')),
    CONSTRAINT positive_version CHECK (version > 0),
    CONSTRAINT non_negative_decisions CHECK (decisions_analyzed >= 0)
);

-- Decision history table (separate for efficient queries and outcome tracking)
CREATE TABLE profile_decision_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id UUID NOT NULL REFERENCES decision_profiles(id) ON DELETE CASCADE,
    cycle_id UUID NOT NULL REFERENCES cycles(id) ON DELETE RESTRICT,

    -- Decision metadata
    decision_date TIMESTAMPTZ NOT NULL,
    title VARCHAR(500) NOT NULL,
    domain VARCHAR(50) NOT NULL,
    dq_score INTEGER,
    key_tradeoff TEXT,
    chosen_alternative VARCHAR(500),

    -- Outcome tracking (filled in later by user)
    outcome_recorded_at TIMESTAMPTZ,
    satisfaction VARCHAR(20),
    actual_consequences TEXT,
    would_decide_same BOOLEAN,

    -- Analysis data for pattern detection
    risk_indicators JSONB,
    objectives_used JSONB,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    UNIQUE(profile_id, cycle_id),
    CONSTRAINT valid_domain CHECK (domain IN (
        'career', 'financial', 'family', 'health', 'relationship',
        'education', 'housing', 'lifestyle', 'business', 'other'
    )),
    CONSTRAINT valid_dq_score CHECK (dq_score IS NULL OR (dq_score >= 0 AND dq_score <= 100)),
    CONSTRAINT valid_satisfaction CHECK (satisfaction IS NULL OR satisfaction IN (
        'very_dissatisfied', 'dissatisfied', 'neutral', 'satisfied', 'very_satisfied'
    ))
);

-- Indexes for efficient queries
CREATE INDEX idx_profiles_user ON decision_profiles(user_id);
CREATE INDEX idx_profiles_updated ON decision_profiles(updated_at DESC);
CREATE INDEX idx_profiles_confidence ON decision_profiles(profile_confidence);

CREATE INDEX idx_profile_history_profile ON profile_decision_history(profile_id);
CREATE INDEX idx_profile_history_cycle ON profile_decision_history(cycle_id);
CREATE INDEX idx_profile_history_domain ON profile_decision_history(domain);
CREATE INDEX idx_profile_history_date ON profile_decision_history(decision_date DESC);
CREATE INDEX idx_profile_history_outcome ON profile_decision_history(outcome_recorded_at) WHERE outcome_recorded_at IS NOT NULL;

-- GIN indexes for JSONB queries
CREATE INDEX idx_profiles_risk_classification ON decision_profiles USING GIN ((risk_profile->'classification'));
CREATE INDEX idx_profiles_risk_dimensions ON decision_profiles USING GIN (risk_profile);
CREATE INDEX idx_profiles_values ON decision_profiles USING GIN (values_priorities);

-- Comments for documentation
COMMENT ON TABLE decision_profiles IS 'User decision profiles capturing patterns, risk tolerance, and preferences across sessions';
COMMENT ON TABLE profile_decision_history IS 'Individual decision records for pattern analysis and outcome tracking';

COMMENT ON COLUMN decision_profiles.file_path IS 'Path to markdown file representation for user transparency';
COMMENT ON COLUMN decision_profiles.content_checksum IS 'SHA-256 checksum for detecting external edits';
COMMENT ON COLUMN decision_profiles.version IS 'Monotonically increasing version number';
COMMENT ON COLUMN decision_profiles.profile_confidence IS 'Confidence level based on decisions analyzed: low (<3), medium (3-7), high (8-15), very_high (16+)';
COMMENT ON COLUMN decision_profiles.consent IS 'User privacy consent settings (collection, analysis, agent_access)';

COMMENT ON COLUMN profile_decision_history.risk_indicators IS 'Risk behavior indicators extracted from this decision';
COMMENT ON COLUMN profile_decision_history.objectives_used IS 'Objectives identified in this decision for value pattern tracking';
COMMENT ON COLUMN profile_decision_history.outcome_recorded_at IS 'When user recorded actual outcome (NULL if pending)';
COMMENT ON COLUMN profile_decision_history.satisfaction IS 'User satisfaction with decision outcome';
COMMENT ON COLUMN profile_decision_history.would_decide_same IS 'Whether user would make same decision again';
