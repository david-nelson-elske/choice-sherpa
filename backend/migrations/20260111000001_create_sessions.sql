-- 20260111000001_create_sessions.sql
-- Sessions table for decision context management

-- Sessions table - top-level container for decision contexts
CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    status VARCHAR(20) NOT NULL CHECK (status IN ('active', 'archived')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Primary indexes
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_sessions_updated_at ON sessions(updated_at DESC);

-- Composite index for user queries
CREATE INDEX idx_sessions_user_status ON sessions(user_id, status);

-- Full-text search index for title and description
CREATE INDEX idx_sessions_search ON sessions USING GIN (
    to_tsvector('english', COALESCE(title, '') || ' ' || COALESCE(description, ''))
);

-- Trigger for updated_at (reuse function from memberships migration)
CREATE TRIGGER update_sessions_updated_at
    BEFORE UPDATE ON sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Table comments
COMMENT ON TABLE sessions IS 'Decision sessions - top-level containers for decision contexts';
COMMENT ON COLUMN sessions.user_id IS 'Owner of the session (references external auth system)';
COMMENT ON COLUMN sessions.title IS 'Session title (1-500 characters)';
COMMENT ON COLUMN sessions.description IS 'Optional description of the decision context';
COMMENT ON COLUMN sessions.status IS 'Session status: active or archived (soft delete)';

-- Add session_id foreign key to cycles table
ALTER TABLE cycles
    ADD CONSTRAINT fk_cycles_session
    FOREIGN KEY (session_id)
    REFERENCES sessions(id)
    ON DELETE CASCADE;

-- Add index for cycle lookups by session
CREATE INDEX idx_cycles_session_status ON cycles(session_id, status);
