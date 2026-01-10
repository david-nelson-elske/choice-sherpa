//! Decision document domain events.
//!
//! Events emitted during document lifecycle operations. These events track
//! document creation, updates, and synchronization for audit trails and
//! triggering side effects like filesystem operations.

use crate::domain::foundation::{ComponentType, CycleId, DecisionDocumentId, Timestamp, UserId};

use super::{DocumentVersion, SyncSource};

/// Events that can occur during document lifecycle.
#[derive(Debug, Clone, PartialEq)]
pub enum DocumentEvent {
    /// A new decision document was created for a cycle.
    Created {
        document_id: DecisionDocumentId,
        cycle_id: CycleId,
        user_id: UserId,
        file_path: String,
        created_at: Timestamp,
    },

    /// A decision document was updated from component changes.
    UpdatedFromComponents {
        document_id: DecisionDocumentId,
        cycle_id: CycleId,
        version: DocumentVersion,
        content_checksum: String,
        updated_at: Timestamp,
    },

    /// A decision document was edited by a user.
    EditedByUser {
        document_id: DecisionDocumentId,
        cycle_id: CycleId,
        user_id: UserId,
        version: DocumentVersion,
        content_checksum: String,
        edited_at: Timestamp,
    },

    /// A decision document was updated by an AI agent.
    UpdatedByAgent {
        document_id: DecisionDocumentId,
        cycle_id: CycleId,
        version: DocumentVersion,
        content_checksum: String,
        updated_at: Timestamp,
    },

    /// A decision document was synced from filesystem changes.
    SyncedFromFile {
        document_id: DecisionDocumentId,
        cycle_id: CycleId,
        version: DocumentVersion,
        content_checksum: String,
        synced_at: Timestamp,
    },

    /// A branched document was created from a parent.
    Branched {
        document_id: DecisionDocumentId,
        parent_document_id: DecisionDocumentId,
        cycle_id: CycleId,
        user_id: UserId,
        branch_point: ComponentType,
        branch_label: String,
        created_at: Timestamp,
    },

    /// Document content parsing detected errors.
    ParseErrorsDetected {
        document_id: DecisionDocumentId,
        cycle_id: CycleId,
        error_count: usize,
        first_error_message: String,
        detected_at: Timestamp,
    },

    /// Document was written to filesystem.
    WrittenToFile {
        document_id: DecisionDocumentId,
        file_path: String,
        size_bytes: usize,
        written_at: Timestamp,
    },
}

impl DocumentEvent {
    /// Returns the document ID associated with this event.
    pub fn document_id(&self) -> DecisionDocumentId {
        match self {
            DocumentEvent::Created { document_id, .. } => *document_id,
            DocumentEvent::UpdatedFromComponents { document_id, .. } => *document_id,
            DocumentEvent::EditedByUser { document_id, .. } => *document_id,
            DocumentEvent::UpdatedByAgent { document_id, .. } => *document_id,
            DocumentEvent::SyncedFromFile { document_id, .. } => *document_id,
            DocumentEvent::Branched { document_id, .. } => *document_id,
            DocumentEvent::ParseErrorsDetected { document_id, .. } => *document_id,
            DocumentEvent::WrittenToFile { document_id, .. } => *document_id,
        }
    }

    /// Returns the cycle ID if this event is associated with a cycle.
    pub fn cycle_id(&self) -> Option<CycleId> {
        match self {
            DocumentEvent::Created { cycle_id, .. } => Some(*cycle_id),
            DocumentEvent::UpdatedFromComponents { cycle_id, .. } => Some(*cycle_id),
            DocumentEvent::EditedByUser { cycle_id, .. } => Some(*cycle_id),
            DocumentEvent::UpdatedByAgent { cycle_id, .. } => Some(*cycle_id),
            DocumentEvent::SyncedFromFile { cycle_id, .. } => Some(*cycle_id),
            DocumentEvent::Branched { cycle_id, .. } => Some(*cycle_id),
            DocumentEvent::ParseErrorsDetected { cycle_id, .. } => Some(*cycle_id),
            DocumentEvent::WrittenToFile { .. } => None,
        }
    }

    /// Returns the sync source that triggered this event.
    pub fn sync_source(&self) -> Option<SyncSource> {
        match self {
            DocumentEvent::Created { .. } => Some(SyncSource::Initial),
            DocumentEvent::UpdatedFromComponents { .. } => Some(SyncSource::ComponentUpdate),
            DocumentEvent::EditedByUser { .. } => Some(SyncSource::UserEdit),
            DocumentEvent::UpdatedByAgent { .. } => Some(SyncSource::ComponentUpdate),
            DocumentEvent::SyncedFromFile { .. } => Some(SyncSource::FileSync),
            DocumentEvent::Branched { .. } => Some(SyncSource::Initial),
            DocumentEvent::ParseErrorsDetected { .. } => None,
            DocumentEvent::WrittenToFile { .. } => None,
        }
    }

    /// Returns the event type name for logging and debugging.
    pub fn event_type(&self) -> &'static str {
        match self {
            DocumentEvent::Created { .. } => "DocumentCreated",
            DocumentEvent::UpdatedFromComponents { .. } => "DocumentUpdatedFromComponents",
            DocumentEvent::EditedByUser { .. } => "DocumentEditedByUser",
            DocumentEvent::UpdatedByAgent { .. } => "DocumentUpdatedByAgent",
            DocumentEvent::SyncedFromFile { .. } => "DocumentSyncedFromFile",
            DocumentEvent::Branched { .. } => "DocumentBranched",
            DocumentEvent::ParseErrorsDetected { .. } => "DocumentParseErrorsDetected",
            DocumentEvent::WrittenToFile { .. } => "DocumentWrittenToFile",
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn test_document_id() -> DecisionDocumentId {
        DecisionDocumentId::new()
    }

    fn test_cycle_id() -> CycleId {
        CycleId::new()
    }

    fn test_user_id() -> UserId {
        UserId::new("test-user-123").unwrap()
    }

    // ───────────────────────────────────────────────────────────────
    // Document ID accessor tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn document_id_returns_id_for_created() {
        let doc_id = test_document_id();
        let event = DocumentEvent::Created {
            document_id: doc_id,
            cycle_id: test_cycle_id(),
            user_id: test_user_id(),
            file_path: "test/doc.md".to_string(),
            created_at: Timestamp::now(),
        };
        assert_eq!(event.document_id(), doc_id);
    }

    #[test]
    fn document_id_returns_id_for_updated_from_components() {
        let doc_id = test_document_id();
        let event = DocumentEvent::UpdatedFromComponents {
            document_id: doc_id,
            cycle_id: test_cycle_id(),
            version: DocumentVersion::initial(),
            content_checksum: "abc123".to_string(),
            updated_at: Timestamp::now(),
        };
        assert_eq!(event.document_id(), doc_id);
    }

    #[test]
    fn document_id_returns_id_for_edited_by_user() {
        let doc_id = test_document_id();
        let event = DocumentEvent::EditedByUser {
            document_id: doc_id,
            cycle_id: test_cycle_id(),
            user_id: test_user_id(),
            version: DocumentVersion::initial(),
            content_checksum: "abc123".to_string(),
            edited_at: Timestamp::now(),
        };
        assert_eq!(event.document_id(), doc_id);
    }

    #[test]
    fn document_id_returns_id_for_branched() {
        let doc_id = test_document_id();
        let parent_id = test_document_id();
        let event = DocumentEvent::Branched {
            document_id: doc_id,
            parent_document_id: parent_id,
            cycle_id: test_cycle_id(),
            user_id: test_user_id(),
            branch_point: ComponentType::Alternatives,
            branch_label: "Option B".to_string(),
            created_at: Timestamp::now(),
        };
        assert_eq!(event.document_id(), doc_id);
    }

    #[test]
    fn document_id_returns_id_for_written_to_file() {
        let doc_id = test_document_id();
        let event = DocumentEvent::WrittenToFile {
            document_id: doc_id,
            file_path: "test/doc.md".to_string(),
            size_bytes: 1024,
            written_at: Timestamp::now(),
        };
        assert_eq!(event.document_id(), doc_id);
    }

    // ───────────────────────────────────────────────────────────────
    // Cycle ID accessor tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn cycle_id_returns_some_for_created() {
        let cycle_id = test_cycle_id();
        let event = DocumentEvent::Created {
            document_id: test_document_id(),
            cycle_id,
            user_id: test_user_id(),
            file_path: "test/doc.md".to_string(),
            created_at: Timestamp::now(),
        };
        assert_eq!(event.cycle_id(), Some(cycle_id));
    }

    #[test]
    fn cycle_id_returns_none_for_written_to_file() {
        let event = DocumentEvent::WrittenToFile {
            document_id: test_document_id(),
            file_path: "test/doc.md".to_string(),
            size_bytes: 1024,
            written_at: Timestamp::now(),
        };
        assert_eq!(event.cycle_id(), None);
    }

    // ───────────────────────────────────────────────────────────────
    // Sync source accessor tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn sync_source_returns_initial_for_created() {
        let event = DocumentEvent::Created {
            document_id: test_document_id(),
            cycle_id: test_cycle_id(),
            user_id: test_user_id(),
            file_path: "test/doc.md".to_string(),
            created_at: Timestamp::now(),
        };
        assert_eq!(event.sync_source(), Some(SyncSource::Initial));
    }

    #[test]
    fn sync_source_returns_component_update_for_updated_from_components() {
        let event = DocumentEvent::UpdatedFromComponents {
            document_id: test_document_id(),
            cycle_id: test_cycle_id(),
            version: DocumentVersion::initial(),
            content_checksum: "abc123".to_string(),
            updated_at: Timestamp::now(),
        };
        assert_eq!(event.sync_source(), Some(SyncSource::ComponentUpdate));
    }

    #[test]
    fn sync_source_returns_user_edit_for_edited_by_user() {
        let event = DocumentEvent::EditedByUser {
            document_id: test_document_id(),
            cycle_id: test_cycle_id(),
            user_id: test_user_id(),
            version: DocumentVersion::initial(),
            content_checksum: "abc123".to_string(),
            edited_at: Timestamp::now(),
        };
        assert_eq!(event.sync_source(), Some(SyncSource::UserEdit));
    }

    #[test]
    fn sync_source_returns_file_sync_for_synced_from_file() {
        let event = DocumentEvent::SyncedFromFile {
            document_id: test_document_id(),
            cycle_id: test_cycle_id(),
            version: DocumentVersion::initial(),
            content_checksum: "abc123".to_string(),
            synced_at: Timestamp::now(),
        };
        assert_eq!(event.sync_source(), Some(SyncSource::FileSync));
    }

    #[test]
    fn sync_source_returns_none_for_parse_errors_detected() {
        let event = DocumentEvent::ParseErrorsDetected {
            document_id: test_document_id(),
            cycle_id: test_cycle_id(),
            error_count: 2,
            first_error_message: "Missing header".to_string(),
            detected_at: Timestamp::now(),
        };
        assert_eq!(event.sync_source(), None);
    }

    // ───────────────────────────────────────────────────────────────
    // Event type accessor tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn event_type_returns_correct_name_for_created() {
        let event = DocumentEvent::Created {
            document_id: test_document_id(),
            cycle_id: test_cycle_id(),
            user_id: test_user_id(),
            file_path: "test/doc.md".to_string(),
            created_at: Timestamp::now(),
        };
        assert_eq!(event.event_type(), "DocumentCreated");
    }

    #[test]
    fn event_type_returns_correct_name_for_all_variants() {
        let doc_id = test_document_id();
        let cycle_id = test_cycle_id();
        let user_id = test_user_id();
        let now = Timestamp::now();
        let version = DocumentVersion::initial();

        assert_eq!(
            DocumentEvent::Created {
                document_id: doc_id,
                cycle_id,
                user_id: user_id.clone(),
                file_path: "test.md".to_string(),
                created_at: now,
            }
            .event_type(),
            "DocumentCreated"
        );

        assert_eq!(
            DocumentEvent::UpdatedFromComponents {
                document_id: doc_id,
                cycle_id,
                version,
                content_checksum: "abc".to_string(),
                updated_at: now,
            }
            .event_type(),
            "DocumentUpdatedFromComponents"
        );

        assert_eq!(
            DocumentEvent::EditedByUser {
                document_id: doc_id,
                cycle_id,
                user_id: user_id.clone(),
                version,
                content_checksum: "abc".to_string(),
                edited_at: now,
            }
            .event_type(),
            "DocumentEditedByUser"
        );

        assert_eq!(
            DocumentEvent::UpdatedByAgent {
                document_id: doc_id,
                cycle_id,
                version,
                content_checksum: "abc".to_string(),
                updated_at: now,
            }
            .event_type(),
            "DocumentUpdatedByAgent"
        );

        assert_eq!(
            DocumentEvent::SyncedFromFile {
                document_id: doc_id,
                cycle_id,
                version,
                content_checksum: "abc".to_string(),
                synced_at: now,
            }
            .event_type(),
            "DocumentSyncedFromFile"
        );

        assert_eq!(
            DocumentEvent::Branched {
                document_id: doc_id,
                parent_document_id: test_document_id(),
                cycle_id,
                user_id,
                branch_point: ComponentType::Objectives,
                branch_label: "Alt".to_string(),
                created_at: now,
            }
            .event_type(),
            "DocumentBranched"
        );

        assert_eq!(
            DocumentEvent::ParseErrorsDetected {
                document_id: doc_id,
                cycle_id,
                error_count: 1,
                first_error_message: "Error".to_string(),
                detected_at: now,
            }
            .event_type(),
            "DocumentParseErrorsDetected"
        );

        assert_eq!(
            DocumentEvent::WrittenToFile {
                document_id: doc_id,
                file_path: "test.md".to_string(),
                size_bytes: 100,
                written_at: now,
            }
            .event_type(),
            "DocumentWrittenToFile"
        );
    }

    // ───────────────────────────────────────────────────────────────
    // Clone and equality tests
    // ───────────────────────────────────────────────────────────────

    #[test]
    fn events_are_clonable_and_comparable() {
        let event = DocumentEvent::Created {
            document_id: test_document_id(),
            cycle_id: test_cycle_id(),
            user_id: test_user_id(),
            file_path: "test/doc.md".to_string(),
            created_at: Timestamp::now(),
        };

        let cloned = event.clone();
        assert_eq!(event, cloned);
    }
}
