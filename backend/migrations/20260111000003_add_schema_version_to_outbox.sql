-- 20260111000003_add_schema_version_to_outbox.sql
-- Add schema versioning support to outbox table

-- Add schema_version column to outbox table
ALTER TABLE outbox
ADD COLUMN schema_version INTEGER NOT NULL DEFAULT 1;

-- Add index for querying by version (useful for migration and debugging)
CREATE INDEX idx_outbox_schema_version
    ON outbox(event_type, schema_version);

COMMENT ON COLUMN outbox.schema_version IS 'Schema version number for this event (e.g., 1, 2, 3)';
