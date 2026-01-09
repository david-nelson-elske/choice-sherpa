# TDD - Test-Driven Development Workflow

Execute the complete Red-Green-Refactor cycle for a single task.

## Workflow State Persistence

This skill persists state to survive context compaction. On invocation:

```bash
source .claude/lib/workflow-state.sh
RESUME=$(workflow_init "tdd" "$TASK_DESCRIPTION")

if [ "$RESUME" = "resume" ]; then
    # Display resume prompt and continue from saved state
    PHASE=$(workflow_current_phase)
    echo "Resuming TDD from: $PHASE phase"

    case "$PHASE" in
        "red")    echo "Continue writing failing test..." ;;
        "green")  echo "Continue implementing to make test pass..." ;;
        "refactor") echo "Continue refactoring..." ;;
        "commit") echo "Ready to commit..." ;;
    esac
else
    # Initialize new TDD workflow
    workflow_add_phase "red" "pending"
    workflow_add_phase "green" "pending"
    workflow_add_phase "refactor" "pending"
    workflow_add_phase "commit" "pending"
    workflow_add_task "$TASK_DESCRIPTION"
    workflow_task_start 1
    workflow_transition "red"
fi
```

**State transitions during execution:**
- Starting RED: `workflow_tdd_phase 1 "red"`
- After test written: `workflow_set '.test_state.test_file' "$TEST_FILE"`
- After failure confirmed: `workflow_tdd_phase_complete 1 "red"` → `workflow_transition "green"`
- Starting GREEN: `workflow_tdd_phase 1 "green"`
- After test passes: `workflow_tdd_phase_complete 1 "green"` → `workflow_transition "refactor"`
- Starting REFACTOR: `workflow_tdd_phase 1 "refactor"`
- After refactor: `workflow_tdd_phase_complete 1 "refactor"` → `workflow_transition "commit"`
- After commit: `workflow_task_complete 1 "$COMMIT_SHA"` → `workflow_complete`

**State file location:** `.claude/workflow-state/active/tdd-{hash}.json`

See `.claude/templates/WORKFLOW-STATE-SPEC.md` for full specification.

---

## Usage
```
/tdd <task-description>
/tdd "add email validation to User model"
```

## Arguments
- `task-description`: What behavior to implement

---

## The TDD Cycle

```
    ┌─────────┐
    │   RED   │ ← Write failing test
    └────┬────┘
         │
         ▼
    ┌─────────┐
    │  GREEN  │ ← Minimal implementation
    └────┬────┘
         │
         ▼
    ┌─────────┐
    │REFACTOR │ ← Improve code quality
    └────┬────┘
         │
         ▼
    ┌─────────┐
    │  VERIFY │ ← Run all tests
    └─────────┘
```

---

## Process

### Step 1: Analyze Task

Before writing code:

1. **Identify the behavior** being implemented
2. **Determine test location** based on project structure
3. **List test cases**:
   - Happy path (success case)
   - Edge cases
   - Error cases

### Step 2: RED Phase (`/tdd-red`)

Write a failing test FIRST:

```
🔴 RED: Writing test for "<task>"

Test: <test_file_path>
Case: <test_case_name>
```

The test must:
- Define expected behavior clearly
- Fail for the RIGHT reason (missing implementation, not syntax error)
- Follow project test conventions

### Step 3: GREEN Phase (`/tdd-green`)

Write MINIMAL code to pass:

```
🟢 GREEN: Implementing minimal solution

File: <implementation_file_path>
```

Rules:
- Only write code the test requires
- No extra features
- No premature optimization
- "Make it work" first

### Step 4: REFACTOR Phase (`/tdd-refactor`)

Improve code quality:

```
🔵 REFACTOR: Improving code quality

Changes:
- Extracted <method/function>
- Renamed <variable> for clarity
- Removed duplication in <location>
```

Rules:
- Tests must stay green
- Run tests after each small change
- Focus on readability and maintainability

### Step 5: Verify

Run full test suite to ensure no regressions:

```bash
/test
```

---

## Test Naming Conventions

### Go
```go
func Test<Type>_<Method>_<Scenario>_<Expected>(t *testing.T)

// Examples:
func TestUser_Validate_WithInvalidEmail_ReturnsError(t *testing.T)
func TestMoney_Add_WithPositiveValues_ReturnsSumInCents(t *testing.T)
```

### TypeScript/JavaScript
```typescript
describe('<Subject>', () => {
  it('should <expected> when <condition>', () => {
    // ...
  });
});

// Examples:
it('should return error when email is invalid')
it('should calculate sum in cents when adding positive values')
```

### Python
```python
def test_<subject>_<scenario>_<expected>():
    pass

# Examples:
def test_user_validate_with_invalid_email_returns_error():
def test_money_add_with_positive_values_returns_sum_in_cents():
```

---

## Test Structure (AAA Pattern)

All tests follow Arrange-Act-Assert:

```go
func TestExample(t *testing.T) {
    // Arrange: Set up test data and conditions
    user := NewUser("test@example.com")

    // Act: Execute the behavior being tested
    err := user.Validate()

    // Assert: Verify the expected outcome
    assert.NoError(t, err)
}
```

```typescript
it('should validate email format', () => {
  // Arrange
  const user = new User('test@example.com');

  // Act
  const result = user.validate();

  // Assert
  expect(result.isValid).toBe(true);
});
```

---

## When to Write Multiple Tests

For complex behaviors, write tests incrementally:

1. **Start simple**: Happy path first
2. **Add edge cases**: One at a time
3. **Add error cases**: Invalid inputs, boundaries

Each test case gets its own RED-GREEN cycle:
```
RED   → Test: valid email passes
GREEN → Implement basic validation

RED   → Test: empty email fails
GREEN → Add empty check

RED   → Test: malformed email fails
GREEN → Add format validation
```

---

## Coverage Guidance

After completing TDD cycle, check coverage:

```bash
/test --coverage
```

Target coverage by layer (adjust in CLAUDE.md):
- Domain/Model layer: 90%+
- Service/Business layer: 85%+
- API/Controller layer: 80%+
- Utilities: 90%+

---

## Common Mistakes to Avoid

### 1. Writing implementation first
```
❌ Write code → Write test → "See, it passes!"
✅ Write test → See it fail → Write code → See it pass
```

### 2. Testing implementation details
```
❌ expect(mock.internalMethod).toHaveBeenCalled()
✅ expect(result.value).toBe(expected)
```

### 3. Multiple behaviors per test
```
❌ it('should validate, save, and send email')
✅ it('should validate email format')
✅ it('should save to database')
✅ it('should send confirmation email')
```

### 4. Skipping RED phase
```
❌ "I know this will work, I'll just write it"
✅ Always see the test fail first - it validates your test works
```

---

## Output

After completing the TDD cycle:

```
✅ TDD cycle complete for: <task>

Tests written:
- test_user_validate_with_valid_email_succeeds
- test_user_validate_with_invalid_email_returns_error
- test_user_validate_with_empty_email_returns_error

Files modified:
- src/models/user.ts
- src/models/user.test.ts

Coverage: 94% (+3%)

Ready to commit? Use /commit or continue with next task.
```

---

## See Also

- `/tdd-red` - Detailed RED phase guidance
- `/tdd-green` - Detailed GREEN phase guidance
- `/tdd-refactor` - Detailed REFACTOR phase guidance
- `/dev` - Feature-driven workflow (multiple tasks)
- `/test` - Run tests
