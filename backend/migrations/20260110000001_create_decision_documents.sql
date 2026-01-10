-- 20260110000001_create_decision_documents.sql
-- Decision documents - live markdown artifacts for decision cycles

-- Decision documents metadata and index
CREATE TABLE decision_documents (
    id UUID PRIMARY KEY,
    cycle_id UUID NOT NULL UNIQUE,
    user_id VARCHAR(255) NOT NULL,

    -- File reference
    file_path VARCHAR(500) NOT NULL,
    content_checksum VARCHAR(64) NOT NULL,
    file_size_bytes INTEGER NOT NULL DEFAULT 0,

    -- Versioning
    version INTEGER NOT NULL DEFAULT 1,
    last_sync_source VARCHAR(50) NOT NULL DEFAULT 'initial'
        CHECK (last_sync_source IN ('initial', 'component_update', 'user_edit', 'file_sync')),
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Progress tracking (denormalized from document content)
    proact_status JSONB NOT NULL DEFAULT '{
        "p": "not_started",
        "r": "not_started",
        "o": "not_started",
        "a": "not_started",
        "c": "not_started",
        "t": "not_started"
    }',
    overall_progress INTEGER NOT NULL DEFAULT 0
        CHECK (overall_progress >= 0 AND overall_progress <= 100),
    dq_score INTEGER CHECK (dq_score IS NULL OR (dq_score >= 0 AND dq_score <= 100)),

    -- Branch metadata
    parent_document_id UUID REFERENCES decision_documents(id) ON DELETE SET NULL,
    branch_point VARCHAR(10) CHECK (branch_point IS NULL OR branch_point IN ('P', 'r', 'O', 'A', 'C', 'T')),
    branch_label VARCHAR(200),

    -- Cached JSON extraction (for dashboard widgets)
    extracted_json JSONB,
    extraction_valid BOOLEAN NOT NULL DEFAULT false,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by_type VARCHAR(20) NOT NULL DEFAULT 'system'
        CHECK (updated_by_type IN ('system', 'user', 'agent')),
    updated_by_id VARCHAR(255)
);

-- Version history (metadata only, content in filesystem)
CREATE TABLE decision_document_versions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_id UUID NOT NULL REFERENCES decision_documents(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,

    -- Snapshot metadata
    content_checksum VARCHAR(64) NOT NULL,
    file_size_bytes INTEGER NOT NULL,
    proact_status JSONB NOT NULL,

    -- Change tracking
    sync_source VARCHAR(50) NOT NULL
        CHECK (sync_source IN ('initial', 'component_update', 'user_edit', 'file_sync')),
    updated_by_type VARCHAR(20) NOT NULL
        CHECK (updated_by_type IN ('system', 'user', 'agent')),
    updated_by_id VARCHAR(255),
    change_summary TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(document_id, version)
);

-- Indexes for decision_documents
CREATE INDEX idx_decision_documents_user ON decision_documents(user_id);
CREATE INDEX idx_decision_documents_cycle ON decision_documents(cycle_id);
CREATE INDEX idx_decision_documents_parent ON decision_documents(parent_document_id)
    WHERE parent_document_id IS NOT NULL;
CREATE INDEX idx_decision_documents_progress ON decision_documents(overall_progress);
CREATE INDEX idx_decision_documents_updated ON decision_documents(updated_at DESC);

-- Indexes for decision_document_versions
CREATE INDEX idx_document_versions_document ON decision_document_versions(document_id);
CREATE INDEX idx_document_versions_created ON decision_document_versions(created_at DESC);

-- Full-text search on extracted content
CREATE INDEX idx_decision_documents_search ON decision_documents
    USING GIN (to_tsvector('english', COALESCE(extracted_json->>'title', '') || ' ' ||
                                       COALESCE(extracted_json->>'focal_decision', '')));

-- Trigger for progress sync and version history
CREATE OR REPLACE FUNCTION update_document_progress()
RETURNS TRIGGER AS $$
DECLARE
    completed_count INTEGER;
BEGIN
    -- Count completed letters
    SELECT COUNT(*) INTO completed_count
    FROM jsonb_each_text(NEW.proact_status)
    WHERE value = 'completed';

    -- Update overall progress
    NEW.overall_progress := (completed_count * 100) / 6;

    -- Update timestamp
    NEW.updated_at := NOW();

    -- Save version history when content changes
    IF OLD IS NULL OR OLD.content_checksum != NEW.content_checksum THEN
        INSERT INTO decision_document_versions (
            document_id, version, content_checksum, file_size_bytes,
            proact_status, sync_source, updated_by_type, updated_by_id
        ) VALUES (
            NEW.id, NEW.version, NEW.content_checksum, NEW.file_size_bytes,
            NEW.proact_status, NEW.last_sync_source, NEW.updated_by_type, NEW.updated_by_id
        );
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER decision_document_progress_trigger
    BEFORE INSERT OR UPDATE ON decision_documents
    FOR EACH ROW
    EXECUTE FUNCTION update_document_progress();

COMMENT ON TABLE decision_documents IS 'Decision document metadata and filesystem index';
COMMENT ON TABLE decision_document_versions IS 'Document version history (metadata only)';
COMMENT ON COLUMN decision_documents.file_path IS 'Relative path: {user_id}/doc_{id}.md';
COMMENT ON COLUMN decision_documents.content_checksum IS 'SHA-256 hash of file content';
COMMENT ON COLUMN decision_documents.proact_status IS 'Component completion status for P, r, O, A, C, T';
COMMENT ON COLUMN decision_documents.overall_progress IS 'Computed from proact_status (0-100%)';
COMMENT ON COLUMN decision_documents.extracted_json IS 'Cached component data for queries';
