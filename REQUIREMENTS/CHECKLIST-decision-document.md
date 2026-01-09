# Decision Document Implementation Checklist

**Feature:** Decision Document as Live Artifact + Cycle Tree Visualization
**Module:** cycle
**Priority:** P1
**Specification:** [features/cycle/decision-document.md](../features/cycle/decision-document.md)
**Related:** [features/cycle/cycle-tree-visualization.md](../features/cycle/cycle-tree-visualization.md)
**Created:** 2026-01-09

---

## Overview

This checklist tracks implementation of the Decision Document feature - a continuously-updated markdown artifact that provides bidirectional sync between human-readable documents and structured PrOACT component data. It includes the Cycle Tree Visualization for exploring decision branches.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     STORAGE ARCHITECTURE                         │
├─────────────────────────────────────────────────────────────────┤
│   FILESYSTEM (Content)              DATABASE (Index/Metadata)   │
│   ════════════════════              ═════════════════════════   │
│   /decisions/{user_id}/             decision_documents table    │
│   ├── doc_abc123.md      ◄─────────►│ id, cycle_id, file_path │ │
│   └── doc_def456.md                 │ proact_status, progress  │ │
│                                     │ parent_id, branch_point  │ │
│   Flat per-user                     └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Foundation

### Domain Layer

- [ ] `domain/cycle/document.rs` - DecisionDocument entity
  - [ ] DecisionDocumentId value object
  - [ ] MarkdownContent value object with checksum
  - [ ] DocumentVersion value object
  - [ ] SyncSource enum (initial, component_update, user_edit, file_sync)
  - [ ] UpdatedBy enum (System, User(id), Agent)
  - [ ] DecisionDocument aggregate with invariants
  - [ ] Unit tests

- [ ] `domain/cycle/document/events.rs` - Domain events
  - [ ] DecisionDocumentCreated
  - [ ] DecisionDocumentUpdated
  - [ ] DecisionDocumentEdited
  - [ ] DecisionDocumentExported
  - [ ] DocumentSyncConflict

- [ ] `domain/cycle/visualization.rs` - Tree visualization types
  - [ ] PrOACTLetter enum (P, R, O, A, C, T)
  - [ ] LetterStatus enum (NotStarted, InProgress, Completed)
  - [ ] PrOACTStatus struct (6 letter statuses)
  - [ ] CycleTreeNode struct (view model)
  - [ ] BranchMetadata struct
  - [ ] PrOACTLetter::to_component_types() method
  - [ ] Unit tests

### Database

- [ ] `migrations/2026MMDD_create_decision_documents.sql`
  - [ ] decision_documents table with dual storage support:
    - [ ] id, cycle_id, user_id (denormalized)
    - [ ] file_path, content_checksum, file_size_bytes
    - [ ] version, last_sync_source, last_synced_at
    - [ ] proact_status JSONB (P/r/O/A/C/T statuses)
    - [ ] overall_progress (0-100%)
    - [ ] dq_score (when completed)
    - [ ] parent_document_id, branch_point, branch_label
    - [ ] extracted_json (cached), extraction_valid
    - [ ] Timestamps and updated_by fields
  - [ ] decision_document_versions table (metadata only)
  - [ ] Indexes (user, cycle, parent, progress, updated_at)
  - [ ] Full-text search index on extracted content
  - [ ] Progress update trigger

---

## Phase 2: Filesystem Storage

### Ports

- [ ] `ports/document_file_storage.rs` - DocumentFileStorage trait
  - [ ] write(user_id, document_id, content) → FilePath
  - [ ] read(user_id, document_id) → String
  - [ ] exists(user_id, document_id) → bool
  - [ ] delete(user_id, document_id)
  - [ ] metadata(user_id, document_id) → FileMetadata
  - [ ] list_user_files(user_id) → Vec<FileInfo>
  - [ ] checksum(user_id, document_id) → String
  - [ ] FilePath, FileMetadata, FileInfo structs
  - [ ] StorageError enum

### Adapters

- [ ] `adapters/filesystem/mod.rs` - Module setup
- [ ] `adapters/filesystem/document_storage.rs` - LocalDocumentFileStorage
  - [ ] Base directory configuration
  - [ ] User directory creation (lazy)
  - [ ] File naming convention: doc_{id}.md
  - [ ] Checksum computation (SHA-256)
  - [ ] Atomic write (temp file + rename)
  - [ ] Unit tests with temp directories
  - [ ] Integration tests

---

## Phase 3: Document Generation (JSON → MD)

### Ports

- [ ] `ports/document_generator.rs` - DocumentGenerator trait
  - [ ] generate(session_title, cycle, options) → String
  - [ ] generate_section(component_type, output) → String
  - [ ] generate_branched_document(parent_doc, branch_point, label) → String
  - [ ] GenerationOptions struct
  - [ ] DocumentFormat enum (Full, Summary, Export)

### Adapters

- [ ] `adapters/document/mod.rs` - Module setup
- [ ] `adapters/document/template_generator.rs` - TemplateDocumentGenerator
  - [ ] Template engine setup (Tera/Handlebars)
  - [ ] Component section templates
  - [ ] Table rendering helpers (Pugh matrix, objectives)
  - [ ] Branch header generation
  - [ ] Unit tests

### Templates

- [ ] `adapters/document/templates/document.md.tera` - Base template
- [ ] `adapters/document/templates/sections/`
  - [ ] issue_raising.md.tera
  - [ ] problem_frame.md.tera
  - [ ] objectives.md.tera (table format)
  - [ ] alternatives.md.tera (table + strategy table)
  - [ ] consequences.md.tera (Pugh matrix)
  - [ ] tradeoffs.md.tera
  - [ ] recommendation.md.tera
  - [ ] decision_quality.md.tera (scoring table)
  - [ ] notes_next_steps.md.tera
- [ ] `adapters/document/templates/branch_header.md.tera`

### Application Layer

- [ ] `application/commands/generate_document.rs`
  - [ ] GenerateDocumentCommand
  - [ ] GenerateDocumentHandler
  - [ ] Unit tests with mock generator

---

## Phase 4: Persistence (DB + File Coordination)

### Ports

- [ ] `ports/document_repository.rs` - DecisionDocumentRepository trait
  - [ ] save(doc, content) - Creates file + DB record
  - [ ] update(doc, content) - Updates file + DB record
  - [ ] find_by_id(id) → DecisionDocument
  - [ ] find_by_cycle(cycle_id) → DecisionDocument
  - [ ] sync_from_file(document_id) → SyncResult
  - [ ] verify_integrity(document_id) → IntegrityStatus
  - [ ] SyncResult struct
  - [ ] IntegrityStatus enum

- [ ] `ports/document_reader.rs` - DecisionDocumentReader trait
  - [ ] get_by_cycle(cycle_id) → DocumentView
  - [ ] get_content(cycle_id) → String (direct from file)
  - [ ] get_version_history(cycle_id, limit) → Vec<DocumentVersionInfo>
  - [ ] search(user_id, query) → Vec<DocumentSearchResult>
  - [ ] get_document_tree(session_id) → DocumentTree
  - [ ] DocumentView struct (with proact_status, branch info)
  - [ ] DocumentVersionInfo struct
  - [ ] DocumentSearchResult struct
  - [ ] DocumentTree, DocumentTreeNode structs

### Adapters

- [ ] `adapters/postgres/document_repository.rs`
  - [ ] PostgresDocumentRepository implementation
  - [ ] Coordinates DocumentFileStorage + database
  - [ ] SQL queries for save/update/find
  - [ ] Integrity verification logic
  - [ ] Integration tests

- [ ] `adapters/postgres/document_reader.rs`
  - [ ] PostgresDocumentReader implementation
  - [ ] Tree query (recursive CTE for branches)
  - [ ] Full-text search query
  - [ ] Progress aggregation
  - [ ] Integration tests

### Application Layer

- [ ] `application/queries/get_document.rs`
  - [ ] GetDocumentQuery
  - [ ] GetDocumentHandler
  - [ ] Unit tests

- [ ] `application/queries/get_document_tree.rs`
  - [ ] GetDocumentTreeQuery
  - [ ] GetDocumentTreeHandler
  - [ ] Unit tests

---

## Phase 5: HTTP Layer (Read Endpoints)

### Handlers

- [ ] `adapters/http/cycle/document_handlers.rs`
  - [ ] get_document handler
  - [ ] regenerate_document handler
  - [ ] DTOs (DocumentResponse with proact_status)
  - [ ] Error mapping

- [ ] `adapters/http/session/tree_handlers.rs`
  - [ ] get_cycle_tree handler
  - [ ] CycleTreeResponse DTO
  - [ ] Error mapping

### Routes

- [ ] `adapters/http/cycle/routes.rs` - Add document routes
  - [ ] GET /api/cycles/:id/document
  - [ ] POST /api/cycles/:id/document/regenerate
- [ ] `adapters/http/session/routes.rs` - Add tree routes
  - [ ] GET /api/sessions/:id/cycle-tree

### Tests

- [ ] HTTP handler tests
- [ ] Integration tests for document endpoints
- [ ] Integration tests for tree endpoint

---

## Phase 6: Document Parsing (MD → JSON)

### Ports

- [ ] `ports/document_parser.rs` - DocumentParser trait
  - [ ] parse(content) → ParseResult
  - [ ] parse_section(section_content, expected_type) → ParsedSection
  - [ ] validate_structure(content) → Vec<ParseError>
  - [ ] ParseResult struct
  - [ ] ParsedSection struct
  - [ ] ParsedMetadata struct
  - [ ] ParseError types (line, column, message, severity)

### Adapters

- [ ] `adapters/document/markdown_parser.rs` - MarkdownDocumentParser
  - [ ] Section extraction (## headers)
  - [ ] Markdown table parser
  - [ ] List/bullet parser
  - [ ] Metadata block parser (> blockquotes)
  - [ ] Unit tests for each component type

### Parser Tests

- [ ] Issue Raising parsing tests
- [ ] Problem Frame parsing tests
- [ ] Objectives parsing tests (table format)
- [ ] Alternatives parsing tests (table format)
- [ ] Consequences parsing tests (Pugh matrix)
- [ ] Tradeoffs parsing tests
- [ ] Recommendation parsing tests
- [ ] Decision Quality parsing tests (scoring table)
- [ ] Notes/Next Steps parsing tests
- [ ] Metadata block parsing tests
- [ ] Edge cases (malformed input, empty sections)
- [ ] Round-trip tests (generate → parse → generate)

---

## Phase 7: Bidirectional Sync

### Application Layer

- [ ] `application/commands/update_document_from_edit.rs`
  - [ ] UpdateDocumentFromEditCommand
  - [ ] UpdateDocumentFromEditHandler
  - [ ] Version conflict detection (optimistic locking)
  - [ ] Validation integration
  - [ ] EditResult enum (Success, NoChanges, ParseErrors)
  - [ ] Unit tests

### Event Handlers

- [ ] `application/event_handlers/document_sync_handler.rs`
  - [ ] Handle CycleComponentOutputUpdated
  - [ ] Sync source check to prevent loops
  - [ ] Section-level updates (preserve other user edits)
  - [ ] Update proact_status in database
  - [ ] Unit tests

### HTTP Layer

- [ ] `adapters/http/cycle/document_handlers.rs` - Add write endpoint
  - [ ] update_document handler
  - [ ] UpdateDocumentRequest DTO (content, expected_version)
  - [ ] UpdateDocumentResponse DTO (success, new_version, errors)
  - [ ] Error handling for parse/validation errors

### Routes

- [ ] PUT /api/cycles/:id/document

### Integration Tests

- [ ] Edit document → component updated
- [ ] Component updated → document regenerated
- [ ] Round-trip: generate → edit → save → regenerate
- [ ] Concurrent edit detection (version mismatch)
- [ ] Sync loop prevention

---

## Phase 8: Branching

### Application Layer

- [ ] `application/commands/branch_document.rs`
  - [ ] BranchDocumentCommand (parent_cycle_id, branch_at, label)
  - [ ] BranchDocumentHandler
  - [ ] Creates child cycle with inherited components
  - [ ] Creates child document with lineage header
  - [ ] Sets parent_document_id and branch_point
  - [ ] BranchResult struct
  - [ ] Unit tests

- [ ] `application/queries/get_branch_comparison.rs`
  - [ ] GetBranchComparisonQuery
  - [ ] GetBranchComparisonHandler
  - [ ] Compare sibling branches at specific sections
  - [ ] Unit tests

### HTTP Layer

- [ ] `adapters/http/cycle/document_handlers.rs` - Add branch endpoint
  - [ ] branch_document handler
  - [ ] BranchDocumentRequest DTO (branch_at, label)
  - [ ] BranchDocumentResponse DTO (new_cycle_id, new_document_id)

### Routes

- [ ] POST /api/cycles/:id/branch

### Tests

- [ ] Branch creation integration tests
- [ ] Branch inherits correct content
- [ ] Branch has correct metadata
- [ ] Tree correctly reflects branches

---

## Phase 9: Export

### Ports

- [ ] `ports/document_export.rs` - DocumentExportService trait
  - [ ] to_pdf(content) → Vec<u8>
  - [ ] to_html(content) → String

### Adapters

- [ ] `adapters/document/export_service.rs`
  - [ ] PDF export implementation (Pandoc/Typst)
  - [ ] HTML export implementation (pulldown-cmark)
  - [ ] Unit tests

### Application Layer

- [ ] `application/commands/export_document.rs`
  - [ ] ExportDocumentCommand
  - [ ] ExportDocumentHandler
  - [ ] ExportFormat enum (Markdown, Pdf, Html)
  - [ ] ExportedDocument struct
  - [ ] Unit tests

### HTTP Layer

- [ ] `adapters/http/cycle/document_handlers.rs` - Add export endpoint
  - [ ] export_document handler
  - [ ] Binary response handling
  - [ ] Content-Type headers (application/pdf, text/html, text/markdown)

### Routes

- [ ] GET /api/cycles/:id/document/export?format=

### Version History

- [ ] `application/queries/get_document_history.rs`
  - [ ] GetDocumentHistoryQuery
  - [ ] GetDocumentHistoryHandler

- [ ] GET /api/cycles/:id/document/history endpoint

---

## Phase 10: Frontend - Document Editor

### API Client

- [ ] `frontend/src/lib/document/types.ts` - TypeScript types
  - [ ] DocumentView (with proact_status, branch info)
  - [ ] UpdateDocumentRequest/Response
  - [ ] ExportFormat
  - [ ] ParseErrorDto, ValidationErrorDto
- [ ] `frontend/src/lib/document/api.ts` - API client functions
  - [ ] getDocument(cycleId)
  - [ ] saveDocument(cycleId, content, version)
  - [ ] regenerateDocument(cycleId)
  - [ ] exportDocument(cycleId, format)
  - [ ] getDocumentHistory(cycleId)
- [ ] `frontend/src/lib/document/stores.ts` - Svelte stores
  - [ ] document store
  - [ ] isEditing store
  - [ ] editedContent store
  - [ ] hasUnsavedChanges derived

### Components

- [ ] `frontend/src/lib/document/DocumentEditor.svelte`
  - [ ] Markdown editor integration (CodeMirror/Monaco)
  - [ ] Split view (editor + preview)
  - [ ] Section navigation outline
  - [ ] Save button with loading state
  - [ ] Version conflict dialog

- [ ] `frontend/src/lib/document/DocumentPreview.svelte`
  - [ ] Rendered markdown view
  - [ ] Component section highlighting
  - [ ] PrOACT status indicators

- [ ] `frontend/src/lib/document/ExportMenu.svelte`
  - [ ] Format selection (Markdown, PDF, HTML)
  - [ ] Download handling

- [ ] `frontend/src/lib/document/VersionHistory.svelte`
  - [ ] Version list with metadata
  - [ ] Diff view (optional)

### Routes

- [ ] `frontend/src/routes/cycles/[id]/document/+page.svelte`
- [ ] `frontend/src/routes/cycles/[id]/document/+page.ts` - Load function

### Tests

- [ ] Component unit tests
- [ ] Store tests
- [ ] API client tests

---

## Phase 11: Frontend - Cycle Tree Visualization

### API Client

- [ ] `frontend/src/lib/cycle/types.ts` - Tree types
  - [ ] CycleTreeNode
  - [ ] PrOACTStatus
  - [ ] LetterStatus enum
  - [ ] PrOACTLetter enum
- [ ] `frontend/src/lib/cycle/api.ts` - Tree API
  - [ ] getCycleTree(sessionId)
  - [ ] branchCycle(cycleId, branchAt, label)
- [ ] `frontend/src/lib/cycle/stores.ts` - Tree stores
  - [ ] cycleTree store
  - [ ] selectedCycleId store

### Components

- [ ] `frontend/src/lib/cycle/CycleTree.svelte`
  - [ ] Tree container with layout options
  - [ ] Node selection handling
  - [ ] Branch creation handling

- [ ] `frontend/src/lib/cycle/PrOACTNode.svelte`
  - [ ] 6-letter display (P r O A C T)
  - [ ] Status colors (green/orange/gray)
  - [ ] Letter click → navigate to section
  - [ ] Letter right-click → branch at
  - [ ] Label and timestamp display
  - [ ] Selected state styling

- [ ] `frontend/src/lib/cycle/LetterTooltip.svelte`
  - [ ] Component name and status
  - [ ] Summary info (e.g., "4 alternatives defined")
  - [ ] Last updated time
  - [ ] Action buttons (View, Branch Here)

- [ ] `frontend/src/lib/cycle/BranchLine.svelte`
  - [ ] Connection lines between nodes
  - [ ] Branch point indicator

- [ ] `frontend/src/lib/cycle/CreateBranchDialog.svelte`
  - [ ] Branch point selection
  - [ ] Label input
  - [ ] Confirmation

### Layout Options

- [ ] Vertical tree (default)
- [ ] Horizontal tree (wide screens)
- [ ] Timeline view (chronological)

### Routes

- [ ] `frontend/src/routes/sessions/[id]/tree/+page.svelte`
- [ ] `frontend/src/routes/sessions/[id]/tree/+page.ts` - Load function

### Tests

- [ ] CycleTree component tests
- [ ] PrOACTNode component tests
- [ ] Tree store tests
- [ ] API client tests

---

## Phase 12: Testing & Polish

### Unit Tests

- [ ] DocumentGenerator tests (all component types)
- [ ] DocumentParser tests (all component types)
- [ ] DocumentFileStorage tests
- [ ] Domain entity tests
- [ ] Command handler tests
- [ ] Query handler tests
- [ ] PrOACTStatus computation tests

### Integration Tests

- [ ] Full sync cycle tests
- [ ] Concurrent modification tests
- [ ] Large document performance tests
- [ ] Export tests (PDF, HTML)
- [ ] Branch creation and tree queries
- [ ] File + DB consistency tests

### E2E Tests

- [ ] Create session → view document
- [ ] Edit document → verify component updated
- [ ] Export document → verify download
- [ ] Create branch → verify tree updated
- [ ] Navigate tree → load correct document

### Documentation

- [ ] API documentation for document endpoints
- [ ] API documentation for tree endpoints
- [ ] User guide for document editing
- [ ] User guide for cycle tree navigation
- [ ] Architecture decision record (ADR)

---

## Acceptance Criteria

### Document Features
- [ ] User can view a human-readable markdown document for any cycle
- [ ] Document updates automatically when component data changes
- [ ] User can edit the markdown and changes sync back to components
- [ ] Version conflicts are detected and handled gracefully
- [ ] Document can be exported as Markdown, PDF, or HTML
- [ ] Version history is preserved and viewable
- [ ] Documents are stored on filesystem (flat per-user directory)
- [ ] Database indexes documents and tracks metadata

### Tree Visualization
- [ ] User can view cycle tree with PrOACT letter nodes
- [ ] Letters show correct status colors (green/orange/gray)
- [ ] Clicking a node loads the associated document
- [ ] Right-clicking a letter offers "branch here" option
- [ ] Branch creates new document with inherited content
- [ ] Tree correctly shows parent-child relationships

### Performance
- [ ] Document generation < 500ms
- [ ] Document save < 1s
- [ ] Tree query < 200ms
- [ ] No sync loops occur during edit/update cycles

---

## Dependencies

### External Crates

- [ ] `sha2` - For content checksums
- [ ] `tera` or `handlebars` - Template engine for generation
- [ ] `pulldown-cmark` - Markdown parsing
- [ ] `regex` - Pattern matching for section extraction
- [ ] `ammonia` - HTML sanitization (for security)
- [ ] `tokio::fs` - Async filesystem operations

### Frontend Libraries

- [ ] Markdown editor (CodeMirror 6 / Monaco)
- [ ] Markdown renderer (marked / markdown-it)
- [ ] Tree visualization (custom Svelte or d3.js)

### External Tools (Optional)

- [ ] Pandoc or Typst - PDF generation

---

## Notes

- Start with Phase 1-5 (read-only documents + tree view) for quick value delivery
- Phase 6-7 (bidirectional sync) are the most complex - allocate extra time
- Consider using existing markdown parsing crates before custom implementation
- PDF export may require external tooling (Pandoc, Typst, or cloud service)
- Filesystem storage enables future git integration and external tool access
- Database indexing enables fast tree queries and search

---

*Checklist Version: 2.0.0*
*Last Updated: 2026-01-09*
