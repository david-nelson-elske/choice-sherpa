-- 20260111000002_create_conversations.sql
-- Conversations and messages for AI-guided component dialogues

-- Conversations table - tracks dialogue state for each component
CREATE TABLE conversations (
    id UUID PRIMARY KEY,
    component_id UUID NOT NULL UNIQUE REFERENCES components(id) ON DELETE CASCADE,
    component_type VARCHAR(20) NOT NULL CHECK (
        component_type IN (
            'issue_raising', 'problem_frame', 'objectives', 'alternatives',
            'consequences', 'tradeoffs', 'recommendation', 'decision_quality',
            'notes_next_steps'
        )
    ),
    state VARCHAR(20) NOT NULL CHECK (
        state IN ('initializing', 'ready', 'in_progress', 'confirmed', 'complete')
    ),
    current_phase VARCHAR(20) NOT NULL CHECK (
        current_phase IN ('intro', 'gather', 'clarify', 'extract', 'confirm')
    ),
    pending_extraction JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Primary indexes
CREATE INDEX idx_conversations_component_id ON conversations(component_id);
CREATE INDEX idx_conversations_state ON conversations(state);
CREATE INDEX idx_conversations_component_type ON conversations(component_type);

-- Composite index for common queries
CREATE INDEX idx_conversations_state_phase ON conversations(state, current_phase);

-- Messages table - conversation message history
CREATE TABLE messages (
    id UUID PRIMARY KEY,
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Message indexes
CREATE INDEX idx_messages_conversation_id ON messages(conversation_id);
CREATE INDEX idx_messages_created_at ON messages(created_at);
CREATE INDEX idx_messages_role ON messages(role);

-- Composite index for message retrieval
CREATE INDEX idx_messages_conversation_created
    ON messages(conversation_id, created_at DESC);

-- Trigger for conversations updated_at
CREATE TRIGGER update_conversations_updated_at
    BEFORE UPDATE ON conversations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Table comments
COMMENT ON TABLE conversations IS 'AI-guided conversations for each PrOACT component';
COMMENT ON COLUMN conversations.component_id IS 'One-to-one reference to component (each component has exactly one conversation)';
COMMENT ON COLUMN conversations.state IS 'Conversation lifecycle state: initializing, ready, in_progress, confirmed, complete';
COMMENT ON COLUMN conversations.current_phase IS 'Current agent phase: intro, gather, clarify, extract, confirm';
COMMENT ON COLUMN conversations.pending_extraction IS 'Extracted structured data awaiting user confirmation';

COMMENT ON TABLE messages IS 'Message history for conversations';
COMMENT ON COLUMN messages.role IS 'Message sender: user, assistant, or system';
COMMENT ON COLUMN messages.content IS 'Message text content';
