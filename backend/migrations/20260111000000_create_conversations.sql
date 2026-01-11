-- 20260111000000_create_conversations.sql
-- Conversation aggregate and message tables for AI-guided dialogues

-- Conversations table - manages dialogue within a component
CREATE TABLE conversations (
    id UUID PRIMARY KEY,
    component_id UUID NOT NULL UNIQUE REFERENCES components(id) ON DELETE CASCADE,
    state VARCHAR(20) NOT NULL CHECK (
        state IN ('initializing', 'ready', 'in_progress', 'confirmed', 'complete')
    ),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Primary indexes
CREATE INDEX idx_conversations_component_id ON conversations(component_id);
CREATE INDEX idx_conversations_state ON conversations(state);

-- Messages table - individual messages within a conversation
CREATE TABLE messages (
    id UUID PRIMARY KEY,
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL CHECK (role IN ('system', 'user', 'assistant')),
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Message indexes
-- Composite index for efficient pagination queries (conversation + ordering)
CREATE INDEX idx_messages_conversation_created
    ON messages(conversation_id, created_at ASC);

-- Index for counting messages per conversation
CREATE INDEX idx_messages_conversation_id ON messages(conversation_id);

-- Partial index for user-visible messages only
CREATE INDEX idx_messages_visible
    ON messages(conversation_id, created_at ASC)
    WHERE role IN ('user', 'assistant');

-- Triggers for updated_at (reuse function from memberships migration)
CREATE TRIGGER update_conversations_updated_at
    BEFORE UPDATE ON conversations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Table comments
COMMENT ON TABLE conversations IS 'AI-guided conversations within PrOACT components';
COMMENT ON COLUMN conversations.component_id IS 'Each component has at most one conversation (unique constraint)';
COMMENT ON COLUMN conversations.state IS 'Lifecycle: initializing -> ready -> in_progress -> confirmed -> complete';

COMMENT ON TABLE messages IS 'Messages within conversations (immutable once created)';
COMMENT ON COLUMN messages.role IS 'Message sender: system (prompts), user (input), assistant (AI response)';
COMMENT ON COLUMN messages.content IS 'Message content (text only, no token limit at DB level)';
