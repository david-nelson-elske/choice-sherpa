-- 20260109000001_create_outbox.sql
-- Event sourcing outbox pattern for reliable event publishing

CREATE TABLE outbox (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    aggregate_type VARCHAR(100) NOT NULL,
    aggregate_id UUID NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ
);

-- Index for polling unprocessed events
CREATE INDEX idx_outbox_unprocessed
    ON outbox(created_at)
    WHERE processed_at IS NULL;

-- Index for aggregate lookup
CREATE INDEX idx_outbox_aggregate
    ON outbox(aggregate_type, aggregate_id);

-- Processed events for idempotency
CREATE TABLE processed_events (
    event_id UUID PRIMARY KEY,
    handler_name VARCHAR(100) NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for cleanup of old processed events
CREATE INDEX idx_processed_events_handler
    ON processed_events(handler_name, processed_at);

COMMENT ON TABLE outbox IS 'Transactional outbox for reliable event publishing';
COMMENT ON TABLE processed_events IS 'Idempotency tracking for event handlers';
