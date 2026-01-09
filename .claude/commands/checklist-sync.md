# Checklist Sync - Synchronize REQUIREMENTS Checklists

Synchronize REQUIREMENTS checklist status with actual filesystem state. Updates file/test inventory status based on what exists.

## Usage
```
/checklist-sync <module>
/checklist-sync session              # Sync CHECKLIST-session.md
/checklist-sync foundation           # Sync CHECKLIST-foundation.md
/checklist-sync --all                # Sync all checklists
/checklist-sync --summary            # Show progress summary only (no updates)
```

## Arguments
- `module`: Module name (maps to `REQUIREMENTS/CHECKLIST-<module>.md`)
- `--all`: Sync all checklists in REQUIREMENTS/ folder
- `--summary`: Display progress without modifying files
- `--verbose`: Show detailed file-by-file status

---

## Process

### Step 1: Load Checklist

```
📋 Loading: REQUIREMENTS/CHECKLIST-session.md
```

Parse the checklist file to extract:
1. **File Inventory** - All tables with `| File | Description | Status |` headers
2. **Test Inventory** - All tables with `| Test Name | Description | Status |` headers
3. **Exit Criteria** - The `### Module is COMPLETE when:` section

### Step 2: Check File Existence

For each file in File Inventory:
```
Checking: backend/src/domain/session/session.rs
  → File exists ✅

Checking: backend/src/domain/session/events.rs
  → File missing ⬜
```

**Status mapping:**
| Condition | Status |
|-----------|--------|
| File exists and non-empty | ✅ |
| File exists but empty | 🔄 |
| File does not exist | ⬜ |

### Step 3: Check Test Files

For test inventory entries:
1. If test file exists → Check if test function exists in file
2. Use pattern matching: `fn <test_name>` (Rust) or `test('<test_name>'` (TypeScript)

**Test status mapping:**
| Condition | Status |
|-----------|--------|
| Test function found in file | ✅ |
| Test file exists but test not found | 🔄 |
| Test file does not exist | ⬜ |

### Step 4: Update Checklist

Replace status symbols in the markdown tables:
- `⬜` → `✅` when file/test exists
- `✅` → `⬜` if file/test was removed (optional, with --strict flag)

### Step 5: Update Exit Criteria

Calculate and update counts in Exit Criteria:

**Before:**
```markdown
### Module is COMPLETE when:
- [ ] All 45 files in File Inventory exist
- [ ] All 85 tests in Test Inventory pass
```

**After:**
```markdown
### Module is COMPLETE when:
- [x] All 45 files in File Inventory exist (12/45 complete)
- [ ] All 85 tests in Test Inventory pass (28/85 complete)
```

### Step 6: Save and Report

```
📊 Checklist Sync Complete: session

File Inventory:
  ✅ 12/45 files exist (27%)

Test Inventory:
  ✅ 28/85 tests found (33%)

Business Rules:
  ✅ 2/6 rules implemented (33%)

Updated: REQUIREMENTS/CHECKLIST-session.md
```

---

## Output Formats

### Default Output
```
📋 Syncing: REQUIREMENTS/CHECKLIST-session.md

File Inventory Progress:
  Domain Layer:        4/4  ████████████████ 100%
  Domain Tests:        2/2  ████████████████ 100%
  Ports:               1/3  █████░░░░░░░░░░░  33%
  Application:         0/6  ░░░░░░░░░░░░░░░░   0%
  HTTP Adapter:        0/4  ░░░░░░░░░░░░░░░░   0%
  Postgres Adapter:    0/4  ░░░░░░░░░░░░░░░░   0%
  Migrations:          1/1  ████████████████ 100%
  Frontend:            0/8  ░░░░░░░░░░░░░░░░   0%

  Total Files:         8/45 (18%)

Test Inventory Progress:
  Domain Tests:        15/28 ████████░░░░░░░░  54%
  Command Tests:       0/18  ░░░░░░░░░░░░░░░░   0%
  Query Tests:         0/9   ░░░░░░░░░░░░░░░░   0%
  HTTP Tests:          0/10  ░░░░░░░░░░░░░░░░   0%
  Repo Tests:          0/10  ░░░░░░░░░░░░░░░░   0%
  Reader Tests:        0/10  ░░░░░░░░░░░░░░░░   0%

  Total Tests:         15/85 (18%)

✅ Checklist updated: REQUIREMENTS/CHECKLIST-session.md
```

### Verbose Output (--verbose)
```
📋 Syncing: REQUIREMENTS/CHECKLIST-session.md

Checking files...
  ✅ backend/src/domain/session/mod.rs
  ✅ backend/src/domain/session/session.rs
  ✅ backend/src/domain/session/events.rs
  ✅ backend/src/domain/session/errors.rs
  ⬜ backend/src/ports/session_repository.rs
  ⬜ backend/src/ports/session_reader.rs
  ...

Checking tests...
  ✅ test_session_new_creates_with_active_status
  ✅ test_session_new_generates_unique_id
  ⬜ test_session_reconstitute_preserves_all_fields
  ...
```

### Summary Output (--summary)
```
📊 REQUIREMENTS Progress Summary

| Module      | Files      | Tests      | Overall |
|-------------|------------|------------|---------|
| foundation  | 12/15 80%  | 45/52 87%  | 83%     |
| session     | 8/45 18%   | 15/85 18%  | 18%     |
| cycle       | 0/38 0%    | 0/72 0%    | 0%      |
| proact-types| 5/8 63%    | 20/25 80%  | 71%     |
| conversation| 0/25 0%    | 0/48 0%    | 0%      |
| analysis    | 0/12 0%    | 0/30 0%    | 0%      |
| dashboard   | 0/20 0%    | 0/35 0%    | 0%      |
| membership  | 0/32 0%    | 0/65 0%    | 0%      |

Total: 25/195 files (13%), 80/412 tests (19%)
```

---

## Integration Points

### Call from /dev (Recommended)

Add to `/dev` workflow after feature completion:

```
# After all tasks in feature complete:
/checklist-sync <module-from-feature>
```

### Call from /pr (Recommended)

Add checklist progress to PR description:

```markdown
## Requirements Progress
<!-- Auto-generated by /checklist-sync -->
Module: session
- Files: 12/45 (27%)
- Tests: 28/85 (33%)
```

### Manual Usage

Run periodically to see overall progress:
```bash
/checklist-sync --all --summary
```

---

## Technical Details

### File Path Resolution

The skill resolves file paths relative to project root:
- `backend/src/domain/session/session.rs` → `/home/david/PPC/choice-sherpa/backend/src/domain/session/session.rs`

### Table Parsing

Parses markdown tables with these headers:
- `| File | Description | Status |`
- `| Test Name | Description | Status |`
- `| Rule | Implementation | Test | Status |`

### Status Symbol Regex

```
⬜ → Not started (U+2B1C)
✅ → Complete (U+2705)
🔄 → In progress (U+1F504)
❌ → Blocked (U+274C)
⏭️ → Skipped (U+23ED)
```

Update regex: `\| ⬜ \|` → `| ✅ |`

---

## Error Handling

### Checklist Not Found
```
❌ Checklist not found: REQUIREMENTS/CHECKLIST-unknown.md

Available checklists:
  - foundation
  - session
  - cycle
  - proact-types
  - conversation
  - analysis
  - dashboard
  - membership
```

### Invalid Checklist Format
```
⚠️ Warning: Could not parse File Inventory in CHECKLIST-session.md
   Expected table header: | File | Description | Status |
```

### File Path Invalid
```
⚠️ Warning: Invalid path in checklist: backend/src/invalid//path.rs
   Skipping this entry.
```

---

## Best Practices

1. **Run after completing work** - Sync after finishing a set of files
2. **Include in PR workflow** - Run before `/pr` to ensure accurate progress
3. **Use --summary for standups** - Quick progress overview
4. **Run --all periodically** - Keep all checklists current

---

## Signals (for Ralph Loop)

| Signal | Meaning |
|--------|---------|
| `CHECKLIST_SYNCED: <module>` | Single module synced |
| `CHECKLIST_SYNCED: all` | All modules synced |
| `CHECKLIST_UNCHANGED: <module>` | No updates needed |

---

## See Also

- `/module-checklist` - Generate new checklists from specs
- `/dev` - Feature-driven development
- `/pr` - Create pull requests
- `/architecture-validate` - Validate against architecture

---

*Version: 1.0.0*
*Created: 2026-01-09*
