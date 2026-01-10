-- Atomic Decision Tools - Tool invocation tracking and agent collaboration entities
--
-- This migration creates tables for:
-- 1. tool_invocations - Audit log of every tool call made by the AI agent
-- 2. revisit_suggestions - Queued suggestions for component revisits
-- 3. confirmation_requests - User confirmations requested by the agent

-- ============================================================================
-- Tool Invocations - Audit log for every tool call
-- ============================================================================

CREATE TABLE tool_invocations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cycle_id UUID NOT NULL,  -- References cycles table (added when cycles exist)
    component VARCHAR(50) NOT NULL,

    -- Tool details
    tool_name VARCHAR(100) NOT NULL,
    parameters JSONB NOT NULL DEFAULT '{}',

    -- Result
    result VARCHAR(20) NOT NULL,
    result_data JSONB,

    -- Context
    conversation_turn INTEGER NOT NULL,
    triggered_by TEXT,

    -- Timing
    invoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    duration_ms INTEGER NOT NULL DEFAULT 0,

    -- Constraints
    CONSTRAINT valid_component CHECK (component IN (
        'issue_raising', 'problem_frame', 'objectives', 'alternatives',
        'consequences', 'tradeoffs', 'recommendation', 'decision_quality'
    )),
    CONSTRAINT valid_result CHECK (result IN (
        'success', 'validation_error', 'not_found', 'conflict', 'internal_error'
    )),
    CONSTRAINT positive_turn CHECK (conversation_turn >= 0),
    CONSTRAINT positive_duration CHECK (duration_ms >= 0)
);

COMMENT ON TABLE tool_invocations IS 'Audit log of all tool invocations by the AI agent';
COMMENT ON COLUMN tool_invocations.cycle_id IS 'The decision cycle this invocation belongs to';
COMMENT ON COLUMN tool_invocations.component IS 'PrOACT component active when tool was invoked';
COMMENT ON COLUMN tool_invocations.tool_name IS 'Name of the tool that was invoked';
COMMENT ON COLUMN tool_invocations.parameters IS 'JSON parameters passed to the tool';
COMMENT ON COLUMN tool_invocations.result IS 'Outcome of the tool execution';
COMMENT ON COLUMN tool_invocations.result_data IS 'Data returned by the tool (if any)';
COMMENT ON COLUMN tool_invocations.conversation_turn IS 'Which conversation turn triggered this';
COMMENT ON COLUMN tool_invocations.triggered_by IS 'Context of what triggered the tool call';
COMMENT ON COLUMN tool_invocations.duration_ms IS 'How long the tool took to execute';

-- Indexes for common queries
CREATE INDEX idx_tool_invocations_cycle ON tool_invocations(cycle_id);
CREATE INDEX idx_tool_invocations_cycle_component ON tool_invocations(cycle_id, component);
CREATE INDEX idx_tool_invocations_tool_name ON tool_invocations(tool_name);
CREATE INDEX idx_tool_invocations_invoked_at ON tool_invocations(invoked_at DESC);
CREATE INDEX idx_tool_invocations_result ON tool_invocations(result);

-- ============================================================================
-- Revisit Suggestions - Queued suggestions for component revisits
-- ============================================================================

CREATE TABLE revisit_suggestions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cycle_id UUID NOT NULL,  -- References cycles table (added when cycles exist)

    -- What to revisit
    target_component VARCHAR(50) NOT NULL,
    reason TEXT NOT NULL,
    trigger TEXT NOT NULL,
    priority VARCHAR(20) NOT NULL,

    -- Status tracking
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    resolution TEXT,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,

    -- Constraints
    CONSTRAINT valid_target_component CHECK (target_component IN (
        'issue_raising', 'problem_frame', 'objectives', 'alternatives',
        'consequences', 'tradeoffs', 'recommendation', 'decision_quality'
    )),
    CONSTRAINT valid_priority CHECK (priority IN ('low', 'medium', 'high', 'critical')),
    CONSTRAINT valid_suggestion_status CHECK (status IN ('pending', 'accepted', 'dismissed', 'expired'))
);

COMMENT ON TABLE revisit_suggestions IS 'Queued suggestions from agent to revisit earlier components';
COMMENT ON COLUMN revisit_suggestions.target_component IS 'Component that should be revisited';
COMMENT ON COLUMN revisit_suggestions.reason IS 'Why this component should be revisited';
COMMENT ON COLUMN revisit_suggestions.trigger IS 'What in the conversation triggered this suggestion';
COMMENT ON COLUMN revisit_suggestions.priority IS 'Urgency level: low, medium, high, critical';
COMMENT ON COLUMN revisit_suggestions.status IS 'Current status of the suggestion';
COMMENT ON COLUMN revisit_suggestions.resolution IS 'User reason for accepting/dismissing';

-- Indexes for common queries
CREATE INDEX idx_revisit_suggestions_cycle ON revisit_suggestions(cycle_id);
CREATE INDEX idx_revisit_suggestions_pending ON revisit_suggestions(cycle_id, status)
    WHERE status = 'pending';
CREATE INDEX idx_revisit_suggestions_priority ON revisit_suggestions(cycle_id, priority DESC)
    WHERE status = 'pending';

-- ============================================================================
-- Confirmation Requests - User confirmations requested by agent
-- ============================================================================

CREATE TABLE confirmation_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cycle_id UUID NOT NULL,  -- References cycles table (added when cycles exist)
    conversation_turn INTEGER NOT NULL,

    -- Request details
    summary TEXT NOT NULL,
    options JSONB NOT NULL,
    default_option INTEGER,

    -- Response
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    chosen_option INTEGER,
    user_input TEXT,

    -- Timestamps
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    responded_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL,

    -- Constraints
    CONSTRAINT valid_confirmation_status CHECK (status IN ('pending', 'confirmed', 'rejected', 'expired')),
    CONSTRAINT positive_conversation_turn CHECK (conversation_turn >= 0),
    CONSTRAINT valid_default_option CHECK (default_option IS NULL OR default_option >= 0),
    CONSTRAINT valid_chosen_option CHECK (chosen_option IS NULL OR chosen_option >= 0)
);

COMMENT ON TABLE confirmation_requests IS 'User confirmation requests from the AI agent';
COMMENT ON COLUMN confirmation_requests.conversation_turn IS 'Which turn created this request';
COMMENT ON COLUMN confirmation_requests.summary IS 'What needs user confirmation';
COMMENT ON COLUMN confirmation_requests.options IS 'JSON array of option objects with label and description';
COMMENT ON COLUMN confirmation_requests.default_option IS 'Index of the default option (0-based)';
COMMENT ON COLUMN confirmation_requests.status IS 'Current status: pending, confirmed, rejected, expired';
COMMENT ON COLUMN confirmation_requests.chosen_option IS 'Index of option user chose (if confirmed)';
COMMENT ON COLUMN confirmation_requests.user_input IS 'Custom text input from user (if provided)';
COMMENT ON COLUMN confirmation_requests.expires_at IS 'When this request auto-expires';

-- Indexes for common queries
CREATE INDEX idx_confirmation_requests_cycle ON confirmation_requests(cycle_id);
CREATE INDEX idx_confirmation_requests_pending ON confirmation_requests(cycle_id, status)
    WHERE status = 'pending';
CREATE INDEX idx_confirmation_requests_expires ON confirmation_requests(expires_at)
    WHERE status = 'pending';
