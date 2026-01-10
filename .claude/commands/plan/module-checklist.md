# Module Checklist Generator

Generate an implementation tracking checklist from a module specification.

## Usage

```
/module-checklist <spec-path>
/module-checklist docs/modules/waitlist.md
/module-checklist events                    # Shorthand
```

---

## Output Location

`REQUIREMENTS/CHECKLIST-<name>.md`

---

## Output Structure

```markdown
# [Module] Module Checklist

**Module:** [Name]
**Dependencies:** [modules]
**Phase:** [1-4]

---

## File Inventory

### Domain Layer
| File | Description | Status |
|------|-------------|--------|
| `path/to/file.rs` | Description | ⬜ |

### Application Layer
| File | Description | Status |

### Adapters
| File | Description | Status |

---

## Test Inventory

### Domain Layer Tests
| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_name` | Description | ⬜ |

---

## Business Rules
| Rule | Implementation | Test | Status |
|------|----------------|------|--------|

---

## Exit Criteria
- [ ] All files exist
- [ ] All tests pass
- [ ] Coverage targets met
```

---

## Extraction Rules

### File Inventory

Extract from spec's "File Structure" sections:
```
backend/src/domain/events/
├── waitlist.rs              # WaitlistEntry entity
├── waitlist_test.rs         # Waitlist tests
```

Becomes:
| File | Description | Status |
|------|-------------|--------|
| `backend/src/domain/events/waitlist.rs` | WaitlistEntry entity | ⬜ |

### Test Names

Extract from spec's "Test Inventory":
```
test_event_join_waitlist_when_full_creates_entry
```

Becomes:
| Test Name | Description | Status |
|-----------|-------------|--------|
| `test_event_join_waitlist_when_full_creates_entry` | Join when full creates entry | ⬜ |

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

## Verification Commands

```bash
# Domain tests
cargo test --package backend domain::events -- waitlist

# Application tests
cargo test --package backend application::commands -- waitlist

# Coverage
cargo tarpaulin --out Html --packages backend
```

---

## Exit Signal

```
MODULE COMPLETE: [module]
Files: XX/XX
Tests: XX/XX passing
Coverage: Domain XX%, Application XX%
```
