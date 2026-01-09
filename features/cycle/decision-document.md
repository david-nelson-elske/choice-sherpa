# Decision Document as Live Artifact

**Module:** cycle
**Type:** Feature Enhancement
**Priority:** P1 (Phase 1 of Agent-Native Enrichments)
**Status:** Specification
**Version:** 1.0.0
**Created:** 2026-01-09
**Based on:** [Agent-Native Enrichments](../../docs/architecture/AGENT-NATIVE-ENRICHMENTS.md) - Suggestion 1

---

## Executive Summary

This feature introduces a continuously-updated **Decision Document** (`decision.md`) that both users and agents can operate on. The document serves as a human-readable, editable interface to the structured PrOACT component data, implementing the "Files as universal interface" agent-native principle.

### Key Benefits

| Benefit | Description |
|---------|-------------|
| **Transparency** | Users can see exactly what the system "understands" about their decision |
| **Editability** | Users can directly edit the document (parity with agent capabilities) |
| **Exportability** | Share with advisors, spouse, colleagues for review |
| **Auditability** | Version history shows how thinking evolved |
| **Trust** | Inspectable outputs build confidence in the system |

---

## Architectural Clarification: Document-First Design

> **IMPORTANT:** The Decision Document is the **PRIMARY working artifact**, not a secondary view.

### The Mental Model

```
┌─────────────────────────────────────────────────────────────────┐
│                    DECISION DOCUMENT                             │
│                  (The Working Artifact)                          │
│                                                                  │
│   ┌──────────────┐    ┌──────────────┐    ┌──────────────┐     │
│   │ User edits   │    │ AI completes │    │ System       │     │
│   │ directly     │    │ sections     │    │ extracts JSON│     │
│   └──────┬───────┘    └──────┬───────┘    └──────┬───────┘     │
│          │                   │                   │              │
│          └───────────────────┴───────────────────┘              │
│                              │                                   │
│                              ▼                                   │
│                    [decision.md content]                         │
└─────────────────────────────────────────────────────────────────┘
                               │
                               │ Extract for queries/display
                               ▼
                    ┌─────────────────────┐
                    │   JSON Components   │
                    │   (Derived Data)    │
                    └─────────────────────┘
```

### How AI Builds the Document

The AI **iteratively builds** the decision document through conversation:

1. **Input:** Current document state + component-specific prompts/skills
2. **Process:** AI analyzes conversation, updates relevant section
3. **Output:** Updated document with new content in appropriate section
4. **Repeat:** User responds, AI updates document further

```
Step 1: User describes situation
        ↓
        AI writes Issue Raising section
        Document: [Issue Raising ✓] [Problem Frame ○] [Objectives ○] ...

Step 2: User clarifies decision scope
        ↓
        AI writes Problem Frame section
        Document: [Issue Raising ✓] [Problem Frame ✓] [Objectives ○] ...

Step 3: User discusses what matters
        ↓
        AI writes Objectives section
        Document: [Issue Raising ✓] [Problem Frame ✓] [Objectives ✓] ...

... and so on through PrOACT
```

### JSON Extraction (Secondary)

JSON is extracted FROM the document for:
- Dashboard widgets (quick glance at alternatives, DQ score)
- API queries (list objectives, get consequence table)
- Validation (schema checking)
- Analytics (optional)

The document is the **source of truth**. JSON is derived.

---

## Core Concept

### Current State

Component outputs are JSON blobs stored in PostgreSQL, visible only through the application UI. Users cannot:
- Easily share their decision analysis
- Edit structured data directly
- See the "big picture" of their decision
- Version control their decision-making process

### Agent-Native Enhancement

The Decision Document becomes the **collaborative workspace** where:
1. **AI writes** sections based on conversation + prompts
2. **Users edit** directly when they want to refine
3. **System extracts** JSON for display/queries
4. **Version control** tracks how thinking evolved
5. **Export/share** for external review

---

## Document Structure

### Template

```markdown
# [Session Title]: [Focal Decision Statement]

> **Status:** [In Progress | Complete] | **Quality Score:** [DQ%]
> **Last Updated:** [timestamp] by [user | agent]
> **Cycle:** [cycle_id] | **Branch:** [parent_id if branched]

---

## 1. Issue Raising

### Potential Decisions
- [ ] [Decision 1]
- [ ] [Decision 2]

### Objectives Identified
- [Objective 1]
- [Objective 2]

### Uncertainties
- [Uncertainty 1]
- [Uncertainty 2]

### Considerations
- [Consideration 1]

---

## 2. Problem Frame

**Decision Maker:** [Name] ([Role])

**Focal Decision:**
> [Decision statement - clear, specific, actionable]

**Scope:** [What's in/out of this decision]

**Deadline:** [If applicable]

### Decision Hierarchy
| Level | Decision | Status |
|-------|----------|--------|
| Already Made | [Prior decision] | [Outcome] |
| **Focal** | [This decision] | In Progress |
| Deferred | [Future decision] | Pending |

### Parties Involved
| Name | Role | Key Concerns |
|------|------|--------------|
| [Name] | [stakeholder/advisor/decision_maker] | [Concerns] |

### Constraints
- **[Type]:** [Description]

---

## 3. Objectives

### Fundamental Objectives (What Really Matters)

| Objective | Measure | Direction |
|-----------|---------|-----------|
| [Maximize compensation] | Total comp ($/yr) | ↑ Higher is better |
| [Maintain work-life balance] | Hours/week | ↓ Lower is better |

### Means Objectives (Ways to Achieve)

| Means Objective | Supports |
|-----------------|----------|
| [Reduce commute time] | Work-life balance |

---

## 4. Alternatives

### Options Under Consideration

| # | Alternative | Description | Status Quo? |
|---|-------------|-------------|-------------|
| A | [Accept VP role] | [Full description...] | No |
| B | [Stay current] | [Full description...] | **Yes** |
| C | [Counter-offer] | [Full description...] | No |

### Strategy Table (if applicable)
| Sub-Decision | Option A | Option B | Option C |
|--------------|----------|----------|----------|
| [Location] | [Remote] | [Hybrid] | [On-site] |
| [Compensation] | [Equity-heavy] | [Salary-heavy] | [Mixed] |

---

## 5. Consequences

### Consequence Matrix (Pugh Analysis)

| Objective | A: Accept VP | B: Stay Current | C: Counter |
|-----------|:------------:|:---------------:|:----------:|
| Compensation | **+2** | 0 (baseline) | +1 |
| Work-Life | -1 | 0 (baseline) | +1 |
| Growth | **+2** | 0 (baseline) | +1 |
| **Total** | **+3** | 0 | +3 |

**Rating Scale:** -2 (Much Worse) → -1 (Worse) → 0 (Same) → +1 (Better) → +2 (Much Better)

### Key Uncertainties
| Uncertainty | Impact | Resolvable? |
|-------------|--------|-------------|
| [Startup funding runway] | High | Yes - ask directly |

---

## 6. Tradeoffs

### Dominated Alternatives
- **[None]** - All alternatives have distinct advantages

### Irrelevant Objectives
- **[None]** - All objectives differentiate options

### Key Tensions
| Alternative | Excels At | Sacrifices |
|-------------|-----------|------------|
| Accept VP | Compensation, Growth | Work-Life |
| Counter | Work-Life | May damage relationship |

---

## 7. Recommendation

### Synthesis
> [Agent's summary of the analysis - what the data shows, not what to decide]

### If One Stands Out
- **Standout Option:** [Alternative A: Accept VP]
- **Rationale:** [Scores highest on most important objectives...]

### Key Considerations Before Deciding
1. [Your work-life tolerance - is -1 acceptable?]
2. [Spouse's input on the move]
3. [Startup runway uncertainty]

### Remaining Uncertainties
- [Item 1] - [Resolution path]

---

## 8. Decision Quality Assessment

| Element | Score | Rationale |
|---------|:-----:|-----------|
| Helpful Problem Frame | 85% | [Clear decision statement, appropriate scope] |
| Clear Objectives | 90% | [Well-defined measures] |
| Creative Alternatives | 75% | [Could explore more hybrid options] |
| Reliable Consequences | 70% | [Some uncertainty in growth ratings] |
| Logically Correct Reasoning | 85% | [No obvious biases] |
| Clear Tradeoffs | 80% | [Tensions well-understood] |
| Commitment to Follow Through | ??% | [TBD by user] |
| **Overall Quality** | **70%** | *Weakest link: Reliable Consequences* |

### To Improve Quality
- [ ] Resolve startup funding uncertainty (+10% on Consequences)
- [ ] Consider hybrid negotiation option (+5% on Alternatives)

---

## Notes & Next Steps

### Open Questions
- [Question 1]

### Planned Actions
| Action | Owner | Due |
|--------|-------|-----|
| [Research startup funding] | Me | [Date] |

### When to Revisit This Decision
- If [startup raises Series B]
- If [current role changes significantly]

---

*Document Version: [auto-increment]*
*Analysis Quality: [DQ%] (Weakest: [Element Name])*
*Generated by Choice Sherpa | [Session URL]*
```

---

## Domain Model

### New Entity: DecisionDocument

```rust
use crate::foundation::{CycleId, Timestamp, UserId};
use serde::{Deserialize, Serialize};

/// DecisionDocument represents the live markdown artifact for a cycle
#[derive(Debug, Clone)]
pub struct DecisionDocument {
    // Identity
    id: DecisionDocumentId,
    cycle_id: CycleId,

    // Content
    content: MarkdownContent,

    // Versioning
    version: DocumentVersion,
    last_sync_source: SyncSource,
    last_synced_at: Timestamp,

    // Metadata
    created_at: Timestamp,
    updated_at: Timestamp,
    updated_by: UpdatedBy,

    // Domain events
    domain_events: Vec<DomainEvent>,
}

/// The actual markdown content with checksum for change detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownContent {
    raw: String,
    checksum: String,  // SHA-256 for change detection
}

impl MarkdownContent {
    pub fn new(raw: String) -> Self {
        let checksum = Self::compute_checksum(&raw);
        Self { raw, checksum }
    }

    fn compute_checksum(content: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn has_changed(&self, other: &str) -> bool {
        self.checksum != Self::compute_checksum(other)
    }
}

/// Version tracking for the document
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DocumentVersion(pub u32);

impl DocumentVersion {
    pub fn initial() -> Self {
        Self(1)
    }

    pub fn increment(&self) -> Self {
        Self(self.0 + 1)
    }
}

/// Tracks what caused the last sync
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncSource {
    /// Generated from component outputs (JSON → MD)
    ComponentUpdate,
    /// Parsed from user edits (MD → JSON)
    UserEdit,
    /// Initial generation
    Initial,
}

/// Who last updated the document
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdatedBy {
    System,
    User(UserId),
    Agent,
}
```

### Value Objects

```rust
use uuid::Uuid;

/// Unique identifier for a decision document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DecisionDocumentId(Uuid);

impl DecisionDocumentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Represents a parsed section from the markdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSection {
    pub component_type: ComponentType,
    pub raw_content: String,
    pub parsed_data: Option<serde_json::Value>,
    pub parse_errors: Vec<ParseError>,
}

/// Errors encountered during markdown parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseError {
    pub line: usize,
    pub column: Option<usize>,
    pub message: String,
    pub severity: ParseErrorSeverity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParseErrorSeverity {
    Warning,  // Data extracted but may be incomplete
    Error,    // Section could not be parsed
}
```

---

## Domain Invariants

1. **One Document Per Cycle**: Each cycle has exactly one decision document
2. **Sync Consistency**: Document must be synchronized with component state on read
3. **Version Monotonicity**: Document version only increases, never decreases
4. **Edit Ownership**: Only cycle owner can edit the document
5. **Content Integrity**: Checksum must match content for change detection

---

## Domain Events

| Event | Trigger | Data |
|-------|---------|------|
| `DecisionDocumentCreated` | Cycle created | cycle_id, document_id |
| `DecisionDocumentUpdated` | Component output changed | cycle_id, document_id, version, source |
| `DecisionDocumentEdited` | User edited markdown | cycle_id, document_id, version, user_id |
| `DecisionDocumentExported` | User exported document | cycle_id, format |
| `DocumentSyncConflict` | Edit/update race detected | cycle_id, resolution |

---

## Ports

### DocumentGenerator Port

```rust
use async_trait::async_trait;

/// Port for generating markdown from component outputs
#[async_trait]
pub trait DocumentGenerator: Send + Sync {
    /// Generate full markdown document from cycle state
    fn generate(
        &self,
        session_title: &str,
        cycle: &Cycle,
        options: GenerationOptions,
    ) -> Result<String, DocumentError>;

    /// Generate a single section for incremental updates
    fn generate_section(
        &self,
        component_type: ComponentType,
        output: &serde_json::Value,
    ) -> Result<String, DocumentError>;
}

#[derive(Debug, Clone, Default)]
pub struct GenerationOptions {
    pub include_metadata: bool,
    pub include_version_info: bool,
    pub include_empty_sections: bool,
    pub format: DocumentFormat,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum DocumentFormat {
    #[default]
    Full,           // Complete document with all sections
    Summary,        // Key sections only
    Export,         // For sharing (no internal IDs)
}
```

### DocumentParser Port

```rust
/// Port for parsing markdown edits back to structured data
#[async_trait]
pub trait DocumentParser: Send + Sync {
    /// Parse full document into component outputs
    fn parse(
        &self,
        content: &str,
    ) -> Result<ParseResult, DocumentError>;

    /// Parse a single section for validation
    fn parse_section(
        &self,
        section_content: &str,
        expected_type: ComponentType,
    ) -> Result<ParsedSection, DocumentError>;

    /// Validate document structure without extracting data
    fn validate_structure(
        &self,
        content: &str,
    ) -> Result<Vec<ParseError>, DocumentError>;
}

#[derive(Debug, Clone)]
pub struct ParseResult {
    pub sections: Vec<ParsedSection>,
    pub metadata: ParsedMetadata,
    pub errors: Vec<ParseError>,
    pub warnings: Vec<ParseError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedMetadata {
    pub title: Option<String>,
    pub focal_decision: Option<String>,
    pub status: Option<String>,
    pub dq_score: Option<u8>,
}
```

### DocumentFileStorage Port (Filesystem)

```rust
use std::path::PathBuf;

/// Port for filesystem operations on decision documents
#[async_trait]
pub trait DocumentFileStorage: Send + Sync {
    /// Write document content to filesystem
    async fn write(
        &self,
        user_id: &UserId,
        document_id: DecisionDocumentId,
        content: &str,
    ) -> Result<FilePath, StorageError>;

    /// Read document content from filesystem
    async fn read(
        &self,
        user_id: &UserId,
        document_id: DecisionDocumentId,
    ) -> Result<String, StorageError>;

    /// Check if document file exists
    async fn exists(
        &self,
        user_id: &UserId,
        document_id: DecisionDocumentId,
    ) -> Result<bool, StorageError>;

    /// Delete document file
    async fn delete(
        &self,
        user_id: &UserId,
        document_id: DecisionDocumentId,
    ) -> Result<(), StorageError>;

    /// Get file metadata (size, modified time)
    async fn metadata(
        &self,
        user_id: &UserId,
        document_id: DecisionDocumentId,
    ) -> Result<FileMetadata, StorageError>;

    /// List all document files for a user (for sync/recovery)
    async fn list_user_files(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<FileInfo>, StorageError>;

    /// Compute checksum of file content
    async fn checksum(
        &self,
        user_id: &UserId,
        document_id: DecisionDocumentId,
    ) -> Result<String, StorageError>;
}

#[derive(Debug, Clone)]
pub struct FilePath(pub PathBuf);

impl FilePath {
    pub fn relative(&self) -> String {
        self.0.to_string_lossy().to_string()
    }
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size_bytes: u64,
    pub modified_at: Timestamp,
    pub checksum: String,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub document_id: DecisionDocumentId,
    pub path: FilePath,
    pub size_bytes: u64,
    pub modified_at: Timestamp,
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("File not found: {path}")]
    NotFound { path: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("User directory not found: {user_id}")]
    UserDirectoryNotFound { user_id: String },
}
```

### DecisionDocumentRepository Port (Database + File Coordination)

```rust
/// Coordinates database metadata and filesystem content
/// This is the primary interface for document operations
#[async_trait]
pub trait DecisionDocumentRepository: Send + Sync {
    /// Save new document (creates file + DB record)
    async fn save(&self, doc: &DecisionDocument, content: &str) -> Result<(), DomainError>;

    /// Update existing document (updates file + DB record)
    async fn update(&self, doc: &DecisionDocument, content: &str) -> Result<(), DomainError>;

    /// Find by ID (loads metadata from DB, content from file)
    async fn find_by_id(&self, id: DecisionDocumentId) -> Result<Option<DecisionDocument>, DomainError>;

    /// Find by cycle (loads metadata from DB, content from file)
    async fn find_by_cycle(&self, cycle_id: CycleId) -> Result<Option<DecisionDocument>, DomainError>;

    /// Sync file changes to database (for external edits)
    async fn sync_from_file(&self, document_id: DecisionDocumentId) -> Result<SyncResult, DomainError>;

    /// Verify file and DB are in sync
    async fn verify_integrity(&self, document_id: DecisionDocumentId) -> Result<IntegrityStatus, DomainError>;
}

#[derive(Debug, Clone)]
pub struct SyncResult {
    pub changed: bool,
    pub new_checksum: String,
    pub new_version: u32,
}

#[derive(Debug, Clone)]
pub enum IntegrityStatus {
    InSync,
    FileModified { file_checksum: String, db_checksum: String },
    FileMissing,
    DbRecordMissing,
}

/// Read operations for decision documents
#[async_trait]
pub trait DecisionDocumentReader: Send + Sync {
    /// Get document view (metadata + content)
    async fn get_by_cycle(&self, cycle_id: CycleId) -> Result<Option<DocumentView>, DomainError>;

    /// Get content only (directly from file for large docs)
    async fn get_content(&self, cycle_id: CycleId) -> Result<Option<String>, DomainError>;

    /// Get version history (metadata only, from DB)
    async fn get_version_history(&self, cycle_id: CycleId, limit: i32) -> Result<Vec<DocumentVersionInfo>, DomainError>;

    /// Search documents by content (uses DB full-text index)
    async fn search(&self, user_id: &UserId, query: &str) -> Result<Vec<DocumentSearchResult>, DomainError>;

    /// Get tree of documents for a session (for cycle tree visualization)
    async fn get_document_tree(&self, session_id: SessionId) -> Result<DocumentTree, DomainError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentView {
    pub id: DecisionDocumentId,
    pub cycle_id: CycleId,
    pub file_path: String,
    pub content: String,
    pub version: u32,
    pub proact_status: PrOACTStatus,
    pub overall_progress: u8,
    pub dq_score: Option<u8>,
    pub last_sync_source: SyncSource,
    pub updated_at: Timestamp,
    pub updated_by: UpdatedBy,
    pub parent_document_id: Option<DecisionDocumentId>,
    pub branch_point: Option<PrOACTLetter>,
    pub branch_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentVersionInfo {
    pub version: u32,
    pub updated_at: Timestamp,
    pub updated_by: UpdatedBy,
    pub sync_source: SyncSource,
    pub checksum: String,
    pub proact_status: PrOACTStatus,
    pub change_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSearchResult {
    pub document_id: DecisionDocumentId,
    pub cycle_id: CycleId,
    pub title: String,
    pub snippet: String,  // Highlighted match
    pub relevance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTree {
    pub session_id: SessionId,
    pub documents: Vec<DocumentTreeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTreeNode {
    pub document_id: DecisionDocumentId,
    pub cycle_id: CycleId,
    pub label: String,
    pub proact_status: PrOACTStatus,
    pub branch_point: Option<PrOACTLetter>,
    pub children: Vec<DocumentTreeNode>,
}
```

---

## Application Layer

### Commands

#### GenerateDocument

Generates or regenerates the decision document from current cycle state.

```rust
#[derive(Debug, Clone)]
pub struct GenerateDocumentCommand {
    pub cycle_id: CycleId,
    pub options: GenerationOptions,
}

pub struct GenerateDocumentHandler {
    cycle_reader: Arc<dyn CycleReader>,
    session_reader: Arc<dyn SessionReader>,
    doc_repo: Arc<dyn DecisionDocumentRepository>,
    generator: Arc<dyn DocumentGenerator>,
    publisher: Arc<dyn DomainEventPublisher>,
}

impl GenerateDocumentHandler {
    pub async fn handle(&self, cmd: GenerateDocumentCommand) -> Result<DecisionDocumentId, DomainError> {
        // 1. Load cycle with all components
        let cycle = self.cycle_reader.get_by_id(cmd.cycle_id).await?
            .ok_or_else(|| DomainError::not_found("cycle"))?;

        // 2. Load session for title
        let session = self.session_reader.get_by_id(cycle.session_id).await?
            .ok_or_else(|| DomainError::not_found("session"))?;

        // 3. Check for existing document
        let existing = self.doc_repo.find_by_cycle(cmd.cycle_id).await?;

        // 4. Generate markdown
        let content = self.generator.generate(&session.title, &cycle, cmd.options)?;

        // 5. Create or update document
        let doc = match existing {
            Some(mut doc) => {
                doc.update_from_components(content)?;
                self.doc_repo.update(&doc).await?;
                doc
            }
            None => {
                let doc = DecisionDocument::new(cmd.cycle_id, content)?;
                self.doc_repo.save(&doc).await?;
                doc
            }
        };

        // 6. Publish events
        self.publisher.publish(doc.pull_domain_events()).await?;

        Ok(doc.id())
    }
}
```

#### UpdateDocumentFromEdit

Processes user edits to the markdown and syncs changes back to components.

```rust
#[derive(Debug, Clone)]
pub struct UpdateDocumentFromEditCommand {
    pub cycle_id: CycleId,
    pub user_id: UserId,
    pub new_content: String,
    pub expected_version: DocumentVersion,  // Optimistic locking
}

pub struct UpdateDocumentFromEditHandler {
    doc_repo: Arc<dyn DecisionDocumentRepository>,
    cycle_repo: Arc<dyn CycleRepository>,
    parser: Arc<dyn DocumentParser>,
    schema_validator: Arc<dyn ComponentSchemaValidator>,
    publisher: Arc<dyn DomainEventPublisher>,
}

impl UpdateDocumentFromEditHandler {
    pub async fn handle(&self, cmd: UpdateDocumentFromEditCommand) -> Result<EditResult, DomainError> {
        // 1. Load document with version check
        let mut doc = self.doc_repo.find_by_cycle(cmd.cycle_id).await?
            .ok_or_else(|| DomainError::not_found("document"))?;

        if doc.version() != cmd.expected_version {
            return Err(DomainError::conflict("Document has been modified. Please refresh."));
        }

        // 2. Check if content actually changed
        if !doc.content().has_changed(&cmd.new_content) {
            return Ok(EditResult::NoChanges);
        }

        // 3. Parse the edited markdown
        let parse_result = self.parser.parse(&cmd.new_content)?;

        // 4. Validate parsed data against schemas
        let mut validation_errors = Vec::new();
        for section in &parse_result.sections {
            if let Some(ref data) = section.parsed_data {
                if let Err(e) = self.schema_validator.validate_partial(section.component_type, data) {
                    validation_errors.push(ValidationError {
                        component: section.component_type,
                        error: e.to_string(),
                    });
                }
            }
        }

        // 5. If parsing had errors, return them (don't save invalid data)
        if !parse_result.errors.is_empty() || !validation_errors.is_empty() {
            return Ok(EditResult::ParseErrors {
                parse_errors: parse_result.errors,
                validation_errors,
            });
        }

        // 6. Update component outputs from parsed data
        let mut cycle = self.cycle_repo.find_by_id(cmd.cycle_id).await?
            .ok_or_else(|| DomainError::not_found("cycle"))?;

        for section in &parse_result.sections {
            if let Some(ref data) = section.parsed_data {
                cycle.update_component_output(section.component_type, data.clone())?;
            }
        }

        // 7. Update document with new content
        doc.apply_user_edit(cmd.new_content, cmd.user_id.clone())?;

        // 8. Save both
        self.cycle_repo.update(&cycle).await?;
        self.doc_repo.update(&doc).await?;

        // 9. Publish events
        let mut events = doc.pull_domain_events();
        events.extend(cycle.pull_domain_events());
        self.publisher.publish(events).await?;

        Ok(EditResult::Success {
            new_version: doc.version(),
            sections_updated: parse_result.sections.len(),
        })
    }
}

#[derive(Debug)]
pub enum EditResult {
    Success { new_version: DocumentVersion, sections_updated: usize },
    NoChanges,
    ParseErrors { parse_errors: Vec<ParseError>, validation_errors: Vec<ValidationError> },
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub component: ComponentType,
    pub error: String,
}
```

#### ExportDocument

Exports the document in various formats for sharing.

```rust
#[derive(Debug, Clone)]
pub struct ExportDocumentCommand {
    pub cycle_id: CycleId,
    pub user_id: UserId,
    pub format: ExportFormat,
}

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Markdown,
    Pdf,
    Html,
}

pub struct ExportDocumentHandler {
    doc_reader: Arc<dyn DecisionDocumentReader>,
    export_service: Arc<dyn DocumentExportService>,
    publisher: Arc<dyn DomainEventPublisher>,
}

impl ExportDocumentHandler {
    pub async fn handle(&self, cmd: ExportDocumentCommand) -> Result<ExportedDocument, DomainError> {
        // 1. Get current document
        let doc = self.doc_reader.get_by_cycle(cmd.cycle_id).await?
            .ok_or_else(|| DomainError::not_found("document"))?;

        // 2. Convert to requested format
        let exported = match cmd.format {
            ExportFormat::Markdown => ExportedDocument {
                content: doc.content.into_bytes(),
                content_type: "text/markdown".to_string(),
                filename: format!("decision-{}.md", cmd.cycle_id),
            },
            ExportFormat::Pdf => {
                let pdf_bytes = self.export_service.to_pdf(&doc.content).await?;
                ExportedDocument {
                    content: pdf_bytes,
                    content_type: "application/pdf".to_string(),
                    filename: format!("decision-{}.pdf", cmd.cycle_id),
                }
            },
            ExportFormat::Html => {
                let html = self.export_service.to_html(&doc.content).await?;
                ExportedDocument {
                    content: html.into_bytes(),
                    content_type: "text/html".to_string(),
                    filename: format!("decision-{}.html", cmd.cycle_id),
                }
            },
        };

        // 3. Record export event
        self.publisher.publish(vec![
            DomainEvent::DecisionDocumentExported {
                cycle_id: cmd.cycle_id,
                format: cmd.format,
                exported_at: Timestamp::now(),
            }
        ]).await?;

        Ok(exported)
    }
}

#[derive(Debug)]
pub struct ExportedDocument {
    pub content: Vec<u8>,
    pub content_type: String,
    pub filename: String,
}
```

### Queries

#### GetDocument

```rust
#[derive(Debug)]
pub struct GetDocumentQuery {
    pub cycle_id: CycleId,
    pub ensure_fresh: bool,  // If true, regenerate if stale
}

pub struct GetDocumentHandler {
    doc_reader: Arc<dyn DecisionDocumentReader>,
    generate_handler: Arc<GenerateDocumentHandler>,
}

impl GetDocumentHandler {
    pub async fn handle(&self, query: GetDocumentQuery) -> Result<DocumentView, DomainError> {
        // 1. Try to get existing document
        if let Some(doc) = self.doc_reader.get_by_cycle(query.cycle_id).await? {
            // 2. If freshness required, check if regeneration needed
            if query.ensure_fresh {
                // TODO: Compare timestamps with component updates
                // For now, always return existing
            }
            return Ok(doc);
        }

        // 3. Generate if not exists
        self.generate_handler.handle(GenerateDocumentCommand {
            cycle_id: query.cycle_id,
            options: GenerationOptions::default(),
        }).await?;

        // 4. Return newly generated document
        self.doc_reader.get_by_cycle(query.cycle_id).await?
            .ok_or_else(|| DomainError::internal("Document generation failed"))
    }
}
```

---

## HTTP Endpoints

| Method | Path | Handler | Description |
|--------|------|---------|-------------|
| `GET` | `/api/cycles/:id/document` | GetDocument | Get markdown document |
| `PUT` | `/api/cycles/:id/document` | UpdateDocumentFromEdit | Save user edits |
| `POST` | `/api/cycles/:id/document/regenerate` | GenerateDocument | Force regeneration |
| `GET` | `/api/cycles/:id/document/export?format=` | ExportDocument | Export to format |
| `GET` | `/api/cycles/:id/document/history` | GetVersionHistory | Version history |

### Request/Response DTOs

```rust
// GET /api/cycles/:id/document
#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub content: String,
    pub version: u32,
    pub last_updated: String,  // ISO 8601
    pub updated_by: String,    // "user", "system", "agent"
    pub sync_source: String,   // "component_update", "user_edit"
}

// PUT /api/cycles/:id/document
#[derive(Debug, Deserialize)]
pub struct UpdateDocumentRequest {
    pub content: String,
    pub expected_version: u32,
}

#[derive(Debug, Serialize)]
pub struct UpdateDocumentResponse {
    pub success: bool,
    pub new_version: Option<u32>,
    pub parse_errors: Vec<ParseErrorDto>,
    pub validation_errors: Vec<ValidationErrorDto>,
}

#[derive(Debug, Serialize)]
pub struct ParseErrorDto {
    pub line: usize,
    pub column: Option<usize>,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Serialize)]
pub struct ValidationErrorDto {
    pub component: String,
    pub error: String,
}

// GET /api/cycles/:id/document/export?format=
#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub format: String,  // "markdown", "pdf", "html"
}
// Response: Binary file with appropriate Content-Type
```

---

## Storage Architecture

### Dual Storage: Filesystem + Database

```
┌─────────────────────────────────────────────────────────────────┐
│                     STORAGE ARCHITECTURE                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   FILESYSTEM (Content)              DATABASE (Index/Metadata)   │
│   ════════════════════              ═════════════════════════   │
│                                                                  │
│   /decisions/{user_id}/             decision_documents table    │
│   ├── doc_abc123.md                 ┌─────────────────────────┐ │
│   ├── doc_def456.md      ◄─────────►│ id, cycle_id, file_path │ │
│   ├── doc_ghi789.md                 │ version, checksum       │ │
│   └── doc_jkl012.md                 │ status, progress        │ │
│                                     │ parent_id, branch_point │ │
│   Flat structure                    │ created_at, updated_at  │ │
│   per user                          │ extracted_json (cached) │ │
│                                     └─────────────────────────┘ │
│                                                                  │
│   Benefits:                         Benefits:                    │
│   • Git-compatible                  • Fast queries              │
│   • External editing                • Relationship tracking     │
│   • Easy backup/export              • Progress metrics          │
│   • Tool integration                • Branch navigation         │
│   • Large file handling             • Search/filter             │
│                                     • Access control            │
└─────────────────────────────────────────────────────────────────┘
```

### Why This Pattern?

| Concern | Filesystem | Database |
|---------|------------|----------|
| **Content storage** | ✓ Primary | Cached extract only |
| **Version history** | Git (optional) | Version table |
| **Relationships** | Filename only | Full graph |
| **Search** | grep/ripgrep | SQL + full-text |
| **Progress tracking** | In-document markers | Structured fields |
| **Branch navigation** | Not possible | Tree queries |
| **Access control** | OS permissions | Row-level security |
| **External tools** | ✓ Direct access | Via API |
| **Offline editing** | ✓ Sync later | Requires connection |

### Filesystem Structure

```
/data/decisions/
└── {user_id}/
    ├── doc_abc123.md           # Main career decision
    ├── doc_def456.md           # Branch: remote option
    ├── doc_ghi789.md           # Branch: counter-offer
    └── doc_jkl012.md           # Separate decision: house purchase
```

**Naming Convention:** `doc_{document_id}.md`
- Simple, flat structure
- No hierarchy in filesystem (that's what DB is for)
- Document ID links to database record
- User directory provides basic isolation

### Future Tool Integration

The filesystem-based storage enables future integrations:

| Tool | Integration |
|------|-------------|
| **Git** | Version control, diff, blame |
| **VS Code / Obsidian** | Direct editing with markdown preview |
| **Sync services** | Dropbox, iCloud, Syncthing |
| **CLI tools** | grep, awk, custom scripts |
| **AI assistants** | Claude, GPT with file access |
| **Export** | Already in portable format |

---

## Database Schema

### Primary Tables

```sql
-- Decision documents metadata and index
CREATE TABLE decision_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cycle_id UUID NOT NULL UNIQUE REFERENCES cycles(id) ON DELETE CASCADE,
    user_id VARCHAR(255) NOT NULL,  -- Denormalized for direct queries

    -- File reference
    file_path VARCHAR(500) NOT NULL,  -- Relative path: {user_id}/doc_{id}.md
    content_checksum VARCHAR(64) NOT NULL,  -- SHA-256 of file content
    file_size_bytes INTEGER NOT NULL DEFAULT 0,

    -- Versioning
    version INTEGER NOT NULL DEFAULT 1,
    last_sync_source VARCHAR(50) NOT NULL DEFAULT 'initial',
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
    overall_progress INTEGER NOT NULL DEFAULT 0,  -- 0-100%
    dq_score INTEGER,  -- Decision quality score when completed

    -- Branch metadata
    parent_document_id UUID REFERENCES decision_documents(id),
    branch_point VARCHAR(10),  -- P, r, O, A, C, T
    branch_label VARCHAR(200),

    -- Cached JSON extraction (for dashboard widgets)
    extracted_json JSONB,  -- Parsed component data
    extraction_valid BOOLEAN NOT NULL DEFAULT false,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by_type VARCHAR(20) NOT NULL DEFAULT 'system',
    updated_by_id VARCHAR(255),

    -- Constraints
    CONSTRAINT valid_sync_source CHECK (last_sync_source IN ('initial', 'component_update', 'user_edit', 'file_sync')),
    CONSTRAINT valid_updated_by_type CHECK (updated_by_type IN ('system', 'user', 'agent')),
    CONSTRAINT valid_branch_point CHECK (branch_point IS NULL OR branch_point IN ('P', 'r', 'O', 'A', 'C', 'T'))
);

-- Version history (metadata only, content in filesystem/git)
CREATE TABLE decision_document_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES decision_documents(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,

    -- Snapshot metadata (content stays in filesystem)
    content_checksum VARCHAR(64) NOT NULL,
    file_size_bytes INTEGER NOT NULL,
    proact_status JSONB NOT NULL,

    -- Change tracking
    sync_source VARCHAR(50) NOT NULL,
    updated_by_type VARCHAR(20) NOT NULL,
    updated_by_id VARCHAR(255),
    change_summary TEXT,  -- AI-generated summary of changes

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(document_id, version)
);

-- Indexes
CREATE INDEX idx_documents_user ON decision_documents(user_id);
CREATE INDEX idx_documents_cycle ON decision_documents(cycle_id);
CREATE INDEX idx_documents_parent ON decision_documents(parent_document_id);
CREATE INDEX idx_documents_progress ON decision_documents(overall_progress);
CREATE INDEX idx_documents_updated ON decision_documents(updated_at DESC);
CREATE INDEX idx_document_versions_document ON decision_document_versions(document_id);
CREATE INDEX idx_document_versions_created ON decision_document_versions(created_at DESC);

-- Full-text search on extracted content
CREATE INDEX idx_documents_search ON decision_documents
    USING GIN (to_tsvector('english', COALESCE(extracted_json->>'title', '') || ' ' ||
                                       COALESCE(extracted_json->>'focal_decision', '')));
```

### Progress Tracking Fields

```sql
-- proact_status JSONB structure
{
    "p": "completed",      -- Problem Frame
    "r": "completed",      -- Objectives (Really matters)
    "o": "in_progress",    -- Options/Alternatives
    "a": "not_started",    -- Analysis/Consequences
    "c": "not_started",    -- Clear Tradeoffs
    "t": "not_started"     -- Think Through (Recommendation + DQ)
}

-- Derived metrics
-- overall_progress = (completed_letters / 6) * 100
-- dq_score = extracted from document when T is completed
```

### Trigger for Progress Sync

```sql
-- Auto-update progress when document is synced
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

    -- Save version history
    IF OLD.content_checksum != NEW.content_checksum THEN
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
    BEFORE UPDATE ON decision_documents
    FOR EACH ROW
    EXECUTE FUNCTION update_document_progress();
```

---

## Branching and Document Lineage

### Each Branch = New Document

When a user branches at a specific PrOACT letter, a **new document** is created:

```
┌─────────────────────────────────────────────────────────────────┐
│ PARENT DOCUMENT (completed)                                      │
│ # Career Decision                                                │
│ [P ✓] [r ✓] [O ✓] [A ✓] [C ✓] [T ✓]                           │
└─────────────────────────────────────────────────────────────────┘
                    │
                    │ Branch at "O" (Alternatives)
                    ▼
┌─────────────────────────────────────────────────────────────────┐
│ CHILD DOCUMENT (new)                                             │
│ # Career Decision (Branch: Remote Option)                        │
│ > Parent: [Parent Doc ID] | Branched at: Alternatives            │
│                                                                  │
│ [P ✓] [r ✓] [O ◉] [A ○] [C ○] [T ○]                           │
│   │     │     │                                                  │
│   └─────┴─────┴── Inherited (read-only until modified)          │
│                   New work starts here                           │
└─────────────────────────────────────────────────────────────────┘
```

### Document Inheritance Rules

| Section | Inherited? | Editable? | Notes |
|---------|------------|-----------|-------|
| Sections BEFORE branch point | Yes | No* | Read-only to preserve lineage |
| Section AT branch point | Yes (copied) | Yes | This is where new exploration starts |
| Sections AFTER branch point | No | Yes | Start fresh |

*Users CAN edit inherited sections, but this triggers a "deep branch" that copies all prior content.

### Branch Metadata in Document

```markdown
# Career Decision (Branch: Remote Option)

> **Status:** In Progress | **Quality Score:** --
> **Last Updated:** 2026-01-09 by Agent
> **Cycle:** abc-123 | **Parent:** xyz-789 (branched at Alternatives)

---

## Lineage

| Ancestor | Branch Point | Document |
|----------|--------------|----------|
| Root | -- | [Career Decision](link) |
| **Current** | Alternatives | *this document* |

---
```

### Comparing Branches

Users can compare sibling branches to see how different explorations diverged:

```
┌─────────────────────┐     ┌─────────────────────┐
│ Branch A: Remote    │ vs  │ Branch B: Counter   │
├─────────────────────┤     ├─────────────────────┤
│ Alternatives:       │     │ Alternatives:       │
│ - Remote-first role │     │ - Counter-offer     │
│                     │     │ - Delayed start     │
├─────────────────────┤     ├─────────────────────┤
│ Consequences:       │     │ Consequences:       │
│ Work-life: +2       │     │ Work-life: +1       │
│ Compensation: +1    │     │ Compensation: +2    │
└─────────────────────┘     └─────────────────────┘
```

### BranchDocument Command

```rust
#[derive(Debug, Clone)]
pub struct BranchDocumentCommand {
    pub parent_cycle_id: CycleId,
    pub branch_at: PrOACTLetter,
    pub branch_label: String,
    pub user_id: UserId,
}

pub struct BranchDocumentHandler {
    cycle_repo: Arc<dyn CycleRepository>,
    doc_repo: Arc<dyn DecisionDocumentRepository>,
    generator: Arc<dyn DocumentGenerator>,
    publisher: Arc<dyn DomainEventPublisher>,
}

impl BranchDocumentHandler {
    pub async fn handle(&self, cmd: BranchDocumentCommand) -> Result<BranchResult, DomainError> {
        // 1. Load parent cycle and document
        let parent_cycle = self.cycle_repo.find_by_id(cmd.parent_cycle_id).await?
            .ok_or_else(|| DomainError::not_found("cycle"))?;
        let parent_doc = self.doc_repo.find_by_cycle(cmd.parent_cycle_id).await?
            .ok_or_else(|| DomainError::not_found("document"))?;

        // 2. Create branched cycle (inherits components up to branch point)
        let child_cycle = Cycle::branch_at(&parent_cycle, cmd.branch_at.to_component_types()[0])?;

        // 3. Create child document with inheritance
        let child_content = self.generator.generate_branched_document(
            &parent_doc,
            cmd.branch_at,
            &cmd.branch_label,
        )?;

        let child_doc = DecisionDocument::new_branch(
            child_cycle.id(),
            cmd.parent_cycle_id,
            cmd.branch_at,
            child_content,
        )?;

        // 4. Save both
        self.cycle_repo.save(&child_cycle).await?;
        self.doc_repo.save(&child_doc).await?;

        // 5. Publish events
        self.publisher.publish(vec![
            DomainEvent::CycleBranched { ... },
            DomainEvent::DecisionDocumentCreated { ... },
        ]).await?;

        Ok(BranchResult {
            new_cycle_id: child_cycle.id(),
            new_document_id: child_doc.id(),
        })
    }
}
```

---

## Sync Strategy

### Component Update → Document (JSON → MD)

```
┌─────────────────────────────────────────────────────────────┐
│ Trigger: CycleComponentOutputUpdated event                   │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│ 1. Load existing document for cycle                          │
│    - If none exists, generate full document                  │
│    - If exists, check last_sync_source                       │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. If last_sync_source == "user_edit"                        │
│    - User has made edits since last component sync           │
│    - Option A: Regenerate full document (lose edits)         │
│    - Option B: Regenerate only changed section               │
│    - Option C: Flag conflict for user resolution             │
│    (Default: Option B - surgical section update)             │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. Generate new section content for updated component        │
│ 4. Replace section in document (preserving other edits)      │
│ 5. Update version, sync_source = "component_update"          │
│ 6. Save to database                                          │
│ 7. Emit DecisionDocumentUpdated event                        │
└─────────────────────────────────────────────────────────────┘
```

### User Edit → Components (MD → JSON)

```
┌─────────────────────────────────────────────────────────────┐
│ Trigger: PUT /api/cycles/:id/document                        │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│ 1. Version check (optimistic locking)                        │
│    - If expected_version != current_version, reject          │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. Parse markdown into sections                              │
│    - Identify section headers (## 1. Issue Raising, etc.)    │
│    - Extract structured data from each section               │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. Validate extracted data against component schemas         │
│    - Allow partial data (not all fields required)            │
│    - Return parse/validation errors if any                   │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. For each valid section:                                   │
│    - Update component output in cycle aggregate              │
│    - Emit ComponentOutputUpdated event (but don't re-sync!)  │
└────────────────────────────┬────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│ 5. Update document:                                          │
│    - Save raw markdown (preserves user formatting)           │
│    - Update version, sync_source = "user_edit"               │
│ 6. Emit DecisionDocumentEdited event                         │
└─────────────────────────────────────────────────────────────┘
```

### Avoiding Sync Loops

```rust
/// Event handler must check sync source to avoid loops
pub struct ComponentUpdateHandler {
    // ...
}

impl EventHandler<CycleComponentOutputUpdated> for ComponentUpdateHandler {
    async fn handle(&self, event: CycleComponentOutputUpdated) -> Result<(), DomainError> {
        // Load document
        let doc = self.doc_repo.find_by_cycle(event.cycle_id).await?;

        // If this update came FROM the document sync, don't re-sync back
        if event.source == UpdateSource::DocumentSync {
            return Ok(());
        }

        // Otherwise, regenerate the section
        // ...
    }
}
```

---

## Frontend Integration

### Document Editor Component

```typescript
// src/lib/document/stores.ts
import { writable, derived } from 'svelte/store';
import type { DocumentView } from './types';

export const document = writable<DocumentView | null>(null);
export const isEditing = writable(false);
export const editedContent = writable<string | null>(null);
export const hasUnsavedChanges = derived(
    [document, editedContent],
    ([$document, $editedContent]) => {
        if (!$document || !$editedContent) return false;
        return $document.content !== $editedContent;
    }
);

// src/lib/document/api.ts
export async function getDocument(cycleId: string): Promise<DocumentView> {
    const response = await fetch(`/api/cycles/${cycleId}/document`);
    if (!response.ok) throw new Error('Failed to load document');
    return response.json();
}

export async function saveDocument(
    cycleId: string,
    content: string,
    expectedVersion: number
): Promise<UpdateDocumentResponse> {
    const response = await fetch(`/api/cycles/${cycleId}/document`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content, expected_version: expectedVersion }),
    });
    return response.json();
}

export async function exportDocument(
    cycleId: string,
    format: 'markdown' | 'pdf' | 'html'
): Promise<Blob> {
    const response = await fetch(
        `/api/cycles/${cycleId}/document/export?format=${format}`
    );
    if (!response.ok) throw new Error('Export failed');
    return response.blob();
}
```

### Markdown Editor Integration

Consider using a markdown editor library like:
- **CodeMirror 6** with markdown mode (lightweight, customizable)
- **Monaco Editor** (VS Code editor, full-featured)
- **Milkdown** (WYSIWYG markdown editor)

Key features needed:
1. Syntax highlighting for markdown
2. Section navigation (outline)
3. Real-time preview (split view)
4. Conflict resolution UI
5. Auto-save with debouncing

---

## Security Requirements

| Requirement | Implementation |
|-------------|----------------|
| Authentication | Required for all document operations |
| Authorization | Only cycle owner can view/edit document |
| Rate Limiting | Standard API limits apply |
| Input Validation | Markdown content size limits, sanitization |
| Export Security | No XSS in HTML export, PDF sanitized |

### Content Sanitization

```rust
/// Sanitize markdown before storage to prevent XSS if rendered as HTML
pub fn sanitize_markdown(content: &str) -> String {
    // 1. Remove raw HTML tags (keep markdown)
    let no_html = ammonia::clean(content);

    // 2. Limit content size (e.g., 100KB)
    if no_html.len() > 100_000 {
        return Err(DomainError::validation("Document too large"));
    }

    // 3. Remove script-like patterns in markdown
    // (fenced code blocks are OK, but not inline JS)

    Ok(no_html)
}
```

---

## Migration Path

### Phase 1: Generate-Only (Read-Only Documents)

1. Add `decision_documents` table
2. Implement `DocumentGenerator` adapter
3. Auto-generate document on cycle creation
4. Add GET endpoint for viewing
5. Add export functionality

### Phase 2: Bidirectional Sync

1. Implement `DocumentParser` adapter
2. Add PUT endpoint for editing
3. Implement sync strategy with conflict detection
4. Add version history tracking
5. Frontend editor integration

### Phase 3: Real-Time Collaboration (Future)

1. WebSocket-based live editing
2. Operational Transform or CRDT for concurrent edits
3. WebDAV integration for file-based sync

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_document_from_empty_cycle() {
        let generator = TemplateDocumentGenerator::new();
        let cycle = Cycle::new_test();
        let doc = generator.generate("My Decision", &cycle, GenerationOptions::default())?;

        assert!(doc.contains("# My Decision"));
        assert!(doc.contains("## 1. Issue Raising"));
        // All 8 sections present
    }

    #[test]
    fn test_parse_objectives_section() {
        let parser = MarkdownDocumentParser::new();
        let section = r#"
## 3. Objectives

### Fundamental Objectives (What Really Matters)

| Objective | Measure | Direction |
|-----------|---------|-----------|
| Maximize compensation | Total comp ($/yr) | ↑ Higher is better |
| Maintain balance | Hours/week | ↓ Lower is better |
"#;

        let result = parser.parse_section(section, ComponentType::Objectives)?;
        assert!(result.parsed_data.is_some());
        let data = result.parsed_data.unwrap();
        assert_eq!(data["fundamental_objectives"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_round_trip_preserves_data() {
        // Generate → Parse → Generate should produce equivalent content
        let generator = TemplateDocumentGenerator::new();
        let parser = MarkdownDocumentParser::new();

        let cycle = Cycle::with_full_data();
        let doc1 = generator.generate("Test", &cycle, GenerationOptions::default())?;
        let parsed = parser.parse(&doc1)?;

        // Update cycle from parsed
        let mut cycle2 = Cycle::new_test();
        for section in parsed.sections {
            if let Some(data) = section.parsed_data {
                cycle2.update_component_output(section.component_type, data)?;
            }
        }

        let doc2 = generator.generate("Test", &cycle2, GenerationOptions::default())?;

        // Should be semantically equivalent (formatting may differ)
        assert_eq!(parsed.sections.len(), 8);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_edit_document_updates_components() {
    let app = test_app().await;
    let cycle_id = create_test_cycle(&app).await;

    // Get initial document
    let doc = get_document(&app, cycle_id).await;

    // Edit the objectives section
    let edited = doc.content.replace(
        "| Maximize compensation | Total comp ($/yr) |",
        "| Maximize total compensation | Total comp + benefits ($/yr) |",
    );

    // Save edit
    let result = save_document(&app, cycle_id, edited, doc.version).await;
    assert!(result.success);

    // Verify component was updated
    let component = get_component(&app, cycle_id, ComponentType::Objectives).await;
    assert!(component.output.to_string().contains("total compensation"));
}
```

---

## Implementation Checklist

### Phase 1: Foundation

- [ ] Create database migration for `decision_documents` and `decision_document_versions`
- [ ] Define `DecisionDocument` domain entity in `backend/src/domain/cycle/document.rs`
- [ ] Define value objects: `DecisionDocumentId`, `MarkdownContent`, `DocumentVersion`
- [ ] Define domain events: `DecisionDocumentCreated`, `DecisionDocumentUpdated`

### Phase 2: Generation (JSON → MD)

- [ ] Define `DocumentGenerator` port in `backend/src/ports/document_generator.rs`
- [ ] Implement `TemplateDocumentGenerator` adapter using Handlebars/Tera
- [ ] Create markdown templates for all 9 component sections
- [ ] Implement `GenerateDocumentCommand` handler
- [ ] Add event handler to generate on `CycleCreated`

### Phase 3: Persistence

- [ ] Define `DecisionDocumentRepository` port
- [ ] Define `DecisionDocumentReader` port
- [ ] Implement Postgres adapters
- [ ] Implement `GetDocumentQuery` handler

### Phase 4: HTTP Layer

- [ ] Add `GET /api/cycles/:id/document` endpoint
- [ ] Add `POST /api/cycles/:id/document/regenerate` endpoint
- [ ] Add DTOs and route configuration
- [ ] Write HTTP handler tests

### Phase 5: Parsing (MD → JSON)

- [ ] Define `DocumentParser` port in `backend/src/ports/document_parser.rs`
- [ ] Implement section extraction from markdown
- [ ] Implement table parser for consequences matrix
- [ ] Implement list parser for objectives, alternatives
- [ ] Write parser unit tests with various input formats

### Phase 6: Bidirectional Sync

- [ ] Implement `UpdateDocumentFromEditCommand` handler
- [ ] Add sync source tracking to prevent loops
- [ ] Implement version check (optimistic locking)
- [ ] Add `PUT /api/cycles/:id/document` endpoint
- [ ] Write integration tests for round-trip editing

### Phase 7: Export

- [ ] Implement PDF export (via Pandoc or Typst)
- [ ] Implement HTML export (markdown-it or pulldown-cmark)
- [ ] Add `GET /api/cycles/:id/document/export` endpoint
- [ ] Add version history endpoint

### Phase 8: Frontend

- [ ] Create document store and API client
- [ ] Integrate markdown editor (CodeMirror/Monaco)
- [ ] Add preview pane
- [ ] Implement save with version conflict handling
- [ ] Add export buttons with format selection

### Phase 9: Testing & Polish

- [ ] Comprehensive unit tests for generator and parser
- [ ] Integration tests for sync scenarios
- [ ] E2E tests for edit workflow
- [ ] Performance testing (large documents)
- [ ] Documentation

---

## Related Documents

- [Agent-Native Enrichments Analysis](../../docs/architecture/AGENT-NATIVE-ENRICHMENTS.md)
- [Cycle Module Specification](../../docs/modules/cycle.md)
- [Component Output Schemas](../proact-types/component-schemas.md)
- [System Architecture](../../docs/architecture/SYSTEM-ARCHITECTURE.md)

---

*Specification Version: 1.0.0*
*Created: 2026-01-09*
*Author: Claude Opus 4.5*
