# TDD Cycle Principles

## The Cycle

```
RED ──────► GREEN ──────► REFACTOR
Write       Make it       Improve
failing     pass          quality
test        (minimal)     (tests stay green)
    │                          │
    └──────────────────────────┘
              Repeat
```

## RED Phase

**Goal:** Define expected behavior with a failing test.

### Rules
1. Write test BEFORE any implementation
2. Test must fail for the right reason (missing implementation, not syntax error)
3. One behavior per test
4. Name test clearly: what is being tested, under what condition, what is expected

### Checklist
- [ ] Test file in correct location
- [ ] Test name follows convention
- [ ] AAA pattern used (Arrange-Act-Assert)
- [ ] Test fails when run
- [ ] Failure is "missing implementation" not "syntax error"

## GREEN Phase

**Goal:** Make the test pass with minimal code.

### Rules
1. Write the simplest code that passes
2. No extra features
3. No optimization
4. Hardcoding is OK (next test forces generalization)
5. No refactoring yet

### Checklist
- [ ] Implementation is minimal
- [ ] The failing test now passes
- [ ] No regressions (other tests still pass)
- [ ] No new functionality beyond what test requires

## REFACTOR Phase

**Goal:** Improve code quality without changing behavior.

### Rules
1. Tests must stay green
2. Run tests after each small change
3. No new functionality
4. Undo immediately if tests fail

### Targets

| Smell | Refactoring |
|-------|-------------|
| Duplicate code (3+ lines) | Extract function |
| Magic numbers | Extract constant |
| Long function (>20 lines) | Extract method |
| Poor naming | Rename |
| Nested conditionals | Guard clauses |

### Checklist
- [ ] No duplicate code
- [ ] Using project abstractions where applicable
- [ ] Clear naming
- [ ] Methods under 20 lines
- [ ] All tests still passing

## When to Stop

- Code is clean enough to understand
- No obvious code smells
- You're tempted to add features (save for next RED)

## Common Mistakes

| Mistake | Correction |
|---------|------------|
| Writing implementation first | Always start with failing test |
| Testing implementation details | Test observable behavior only |
| Multiple behaviors per test | One test = one behavior |
| Skipping RED phase | Even if you "know" it works, verify test fails first |
| Refactoring with failing tests | GREEN first, then refactor |
| Adding features in GREEN | Minimal code only |

## Incremental Test-Driven Development

For complex behaviors, write tests incrementally:

```
RED   → Test: valid input passes
GREEN → Implement basic case

RED   → Test: empty input fails
GREEN → Add empty check

RED   → Test: boundary condition
GREEN → Add boundary handling
```

Each test drives one small change. Resist implementing ahead of tests.
