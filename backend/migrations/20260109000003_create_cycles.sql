-- 20260109000003_create_cycles.sql
-- Cycle aggregate and component tables for PrOACT decision support

-- Cycles table - represents a complete or partial path through PrOACT
CREATE TABLE cycles (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL,
    parent_cycle_id UUID REFERENCES cycles(id) ON DELETE CASCADE,
    branch_point VARCHAR(20) CHECK (
        branch_point IS NULL OR
        branch_point IN (
            'issue_raising', 'problem_frame', 'objectives', 'alternatives',
            'consequences', 'tradeoffs', 'recommendation', 'decision_quality',
            'notes_next_steps'
        )
    ),
    status VARCHAR(20) NOT NULL CHECK (status IN ('active', 'completed', 'archived')),
    current_step VARCHAR(20) NOT NULL CHECK (
        current_step IN (
            'issue_raising', 'problem_frame', 'objectives', 'alternatives',
            'consequences', 'tradeoffs', 'recommendation', 'decision_quality',
            'notes_next_steps'
        )
    ),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Primary indexes
CREATE INDEX idx_cycles_session_id ON cycles(session_id);
CREATE INDEX idx_cycles_status ON cycles(status);

-- Partial index for finding branches efficiently
CREATE INDEX idx_cycles_parent_id
    ON cycles(parent_cycle_id)
    WHERE parent_cycle_id IS NOT NULL;

-- Components table - the 9 PrOACT components within a cycle
CREATE TABLE components (
    id UUID PRIMARY KEY,
    cycle_id UUID NOT NULL REFERENCES cycles(id) ON DELETE CASCADE,
    component_type VARCHAR(20) NOT NULL CHECK (
        component_type IN (
            'issue_raising', 'problem_frame', 'objectives', 'alternatives',
            'consequences', 'tradeoffs', 'recommendation', 'decision_quality',
            'notes_next_steps'
        )
    ),
    status VARCHAR(20) NOT NULL CHECK (
        status IN ('not_started', 'in_progress', 'complete', 'needs_revision')
    ),
    output JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Each cycle can only have one of each component type
    CONSTRAINT unique_component_per_cycle UNIQUE (cycle_id, component_type)
);

-- Component indexes
CREATE INDEX idx_components_cycle_id ON components(cycle_id);
CREATE INDEX idx_components_status ON components(status);

-- GIN index for JSONB queries on component output (optional, for advanced queries)
CREATE INDEX idx_components_output ON components USING GIN (output);

-- Triggers for updated_at (reuse function from memberships migration)
CREATE TRIGGER update_cycles_updated_at
    BEFORE UPDATE ON cycles
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_components_updated_at
    BEFORE UPDATE ON components
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Table comments
COMMENT ON TABLE cycles IS 'PrOACT decision cycles - complete or partial paths through the framework';
COMMENT ON COLUMN cycles.parent_cycle_id IS 'For branched cycles - references the parent cycle at branch_point';
COMMENT ON COLUMN cycles.branch_point IS 'Component where this cycle branched from parent';
COMMENT ON COLUMN cycles.current_step IS 'Currently active component in the workflow';

COMMENT ON TABLE components IS 'PrOACT component data within cycles';
COMMENT ON COLUMN components.output IS 'JSON output from the component (structure varies by type)';
COMMENT ON COLUMN components.status IS 'Workflow status: not_started, in_progress, complete, needs_revision';
