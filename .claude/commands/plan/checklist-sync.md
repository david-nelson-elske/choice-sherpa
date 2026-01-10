# Checklist Sync - Synchronize REQUIREMENTS Checklists

Synchronize checklist status with filesystem state.

## Usage

```
/checklist-sync <module>
/checklist-sync session              # Sync CHECKLIST-session.md
/checklist-sync --all                # Sync all checklists
/checklist-sync --summary            # Progress summary only
```

| Option | Description |
|--------|-------------|
| `module` | Module name (maps to `REQUIREMENTS/CHECKLIST-<module>.md`) |
| `--all` | Sync all checklists |
| `--summary` | Display progress without modifying |
| `--verbose` | Show file-by-file status |

---

## Process

1. **Load Checklist** - Parse file/test inventory tables
2. **Check Files** - Verify each file exists
3. **Check Tests** - Verify test functions exist in files
4. **Update Status** - Replace status symbols
5. **Update Counts** - Recalculate exit criteria progress
6. **Save & Report** - Write changes and show summary

---

## Status Mapping

### Files

| Condition | Status |
|-----------|--------|
| File exists, non-empty | ✅ |
| File exists, empty | 🔄 |
| File missing | ⬜ |

### Tests

| Condition | Status |
|-----------|--------|
| Test function found | ✅ |
| Test file exists, test missing | 🔄 |
| Test file missing | ⬜ |

---

## Output Formats

### Default

```
📋 Syncing: REQUIREMENTS/CHECKLIST-session.md

File Inventory Progress:
  Domain Layer:     4/4  ████████████████ 100%
  Ports:            1/3  █████░░░░░░░░░░░  33%
  Application:      0/6  ░░░░░░░░░░░░░░░░   0%

  Total Files:      8/45 (18%)

Test Inventory Progress:
  Domain Tests:    15/28 ████████░░░░░░░░  54%

  Total Tests:     15/85 (18%)

✅ Checklist updated
```

### Summary (--summary)

```
📊 REQUIREMENTS Progress Summary

| Module      | Files      | Tests      | Overall |
|-------------|------------|------------|---------|
| foundation  | 12/15 80%  | 45/52 87%  | 83%     |
| session     | 8/45 18%   | 15/85 18%  | 18%     |

Total: 25/195 files (13%), 80/412 tests (19%)
```

---

## Status Symbols

| Symbol | Meaning |
|--------|---------|
| ⬜ | Not started |
| 🔄 | In progress |
| ✅ | Complete |
| ❌ | Blocked |
| ⏭️ | Skipped |

---

## Table Parsing

Parses tables with these headers:
- `| File | Description | Status |`
- `| Test Name | Description | Status |`
- `| Rule | Implementation | Test | Status |`

---

## Error Handling

| Error | Resolution |
|-------|------------|
| Checklist not found | List available checklists |
| Invalid format | Show expected header |
| Invalid path | Skip entry, warn |

---

## Integration

| Skill | When to Sync |
|-------|--------------|
| `/dev` | After completing work |
| `/pr` | Before creating PR (add progress to description) |
| Manual | Periodic progress check |
