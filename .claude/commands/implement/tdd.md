# TDD - Test-Driven Development

Execute the Red → Green → Refactor cycle.

## Usage

```
/tdd <task>                    # Full cycle
/tdd --phase red <task>        # RED only
/tdd --phase green             # GREEN only (requires prior RED)
/tdd --phase refactor          # REFACTOR only (requires prior GREEN)
```

## The Cycle

```
RED ──────► GREEN ──────► REFACTOR
Write       Make it       Improve
failing     pass          quality
test        (minimal)     (tests stay green)
```

---

## RED Phase

Write a failing test BEFORE implementation.

### Process

1. Identify test file location (co-located with source)
2. Write test using AAA pattern: Arrange → Act → Assert
3. Name test: `test_<subject>_<scenario>_<expected>` (Rust) or `should <expected> when <condition>` (TS)
4. Run test - **must fail**
5. Verify failure is "missing implementation" not syntax error

### Checklist

- [ ] Test in correct location
- [ ] One behavior per test
- [ ] Test fails for right reason

### Output

```
🔴 RED: Test failing
   File: <path>
   Test: <name>
   Reason: <expected failure>
```

---

## GREEN Phase

Write MINIMAL code to pass the test.

### Rules

- Only what the test requires
- No extra features
- No optimization
- Hardcoding is OK (next test forces generalization)

### Process

1. Implement smallest change to pass
2. Run test - **must pass**
3. Run all related tests - no regressions

### Checklist

- [ ] Implementation is minimal
- [ ] Target test passes
- [ ] No regressions

### Output

```
🟢 GREEN: Test passing
   File: <path>
   Implementation: <brief description>
```

---

## REFACTOR Phase

Improve code quality. Tests must stay green.

### Targets

| Smell | Fix |
|-------|-----|
| Duplicate code (3+ lines) | Extract function |
| Magic numbers | Extract constant |
| Long function (>20 lines) | Extract method |
| Poor naming | Rename |
| Nested conditionals | Guard clauses |

### Rules

- Run tests after EACH small change
- No new functionality
- Undo immediately if tests fail

### DRY Check

- [ ] No repeated code blocks
- [ ] Using project macros where applicable
- [ ] Single source of truth for business rules

### Output

```
🔵 REFACTOR: Code improved
   Changes: <list>
   Tests: ✅ All passing
```

---

## Verification

After cycle complete:

1. Run full test suite: `/test`
2. Run linter: `/lint`
3. Check coverage meets targets

---

## Complete Cycle Output

```
✅ TDD cycle complete for: <task>

Tests written:
- <test_names>

Files modified:
- <paths>

Ready to commit? Use /commit
```

---

## References

For language-specific syntax:
- Rust: `.claude/lib/examples/rust/testing.md`
- TypeScript: `.claude/lib/examples/typescript/testing.md`

For TDD principles:
- TDD Cycle: `.claude/lib/examples/shared/tdd-cycle.md`
- AAA Pattern: `.claude/lib/examples/shared/aaa-pattern.md`
