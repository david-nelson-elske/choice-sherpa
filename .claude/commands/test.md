# Test - Run Tests

Run tests with various configurations. Detects project type and uses appropriate commands.

## Usage
```
/test [scope] [options]
```

## Arguments
- `scope`: What to test (optional, defaults to "all")
  - `all` - Run all tests
  - `unit` - Unit tests only
  - `integration` - Integration tests only
  - `e2e` - End-to-end tests
  - `<path>` - Specific file or directory

- `options`: Additional flags
  - `--coverage` - Include coverage report
  - `--watch` - Watch mode
  - `--verbose` - Verbose output
  - `--filter <pattern>` - Filter tests by name

---

## Auto-Detection

The skill detects project type and uses appropriate commands:

| Detection | Test Command |
|-----------|-------------|
| `go.mod` exists | `go test ./...` |
| `package.json` with vitest | `npx vitest run` |
| `package.json` with jest | `npx jest` |
| `pyproject.toml` or `pytest.ini` | `pytest` |
| `Cargo.toml` | `cargo test` |

---

## Project Configuration

Override auto-detection in `CLAUDE.md`:

```markdown
## Test Commands
- test_all: `npm test`
- test_unit: `npm test -- --testPathPattern=unit`
- test_integration: `npm test -- --testPathPattern=integration`
- test_e2e: `npx playwright test`
- test_coverage: `npm test -- --coverage`
- test_watch: `npm test -- --watch`
```

---

## Commands by Language

### Go

```bash
# All tests
go test ./...

# Specific package
go test ./internal/user/...

# With coverage
go test -coverprofile=coverage.out ./...
go tool cover -html=coverage.out -o coverage.html

# Verbose
go test -v ./...

# Filter by name
go test -run "TestUser_Register" ./...

# Skip integration tests (short mode)
go test -short ./...

# With race detection
go test -race ./...
```

### TypeScript/JavaScript (Vitest)

```bash
# All tests
npx vitest run

# Watch mode
npx vitest

# With coverage
npx vitest run --coverage

# Specific file
npx vitest run src/services/user.test.ts

# Filter by name
npx vitest run -t "should validate email"

# UI mode
npx vitest --ui
```

### TypeScript/JavaScript (Jest)

```bash
# All tests
npx jest

# Watch mode
npx jest --watch

# With coverage
npx jest --coverage

# Specific file
npx jest src/services/user.test.ts

# Filter by name
npx jest -t "should validate email"
```

### Python (pytest)

```bash
# All tests
pytest

# Verbose
pytest -v

# With coverage
pytest --cov=src --cov-report=html

# Specific file
pytest tests/test_user.py

# Filter by name
pytest -k "test_register"

# Stop on first failure
pytest -x
```

### Rust

```bash
# All tests
cargo test

# Specific test
cargo test test_user_register

# With output
cargo test -- --nocapture

# Doc tests only
cargo test --doc
```

---

## Coverage Targets

After running `--coverage`, verify against targets:

| Layer | Recommended Target |
|-------|-------------------|
| Domain/Model | 90%+ |
| Service/Business | 85%+ |
| API/Controller | 80%+ |
| Utilities | 90%+ |

Configure in `CLAUDE.md`:
```markdown
## Coverage Targets
- domain: 90
- service: 85
- api: 80
- overall: 85
```

---

## Test Organization

### Recommended Structure

```
project/
├── src/
│   ├── models/
│   │   ├── user.ts
│   │   └── user.test.ts      # Co-located unit tests
│   └── services/
│       ├── auth.ts
│       └── auth.test.ts
├── tests/
│   ├── integration/          # Integration tests
│   │   └── api.test.ts
│   └── e2e/                  # End-to-end tests
│       └── login.spec.ts
```

### Go Structure

```
project/
├── internal/
│   ├── user/
│   │   ├── user.go
│   │   └── user_test.go      # Unit tests
│   └── handler/
│       ├── handler.go
│       └── handler_test.go
├── test/
│   └── integration/          # Integration tests
│       └── api_test.go
```

---

## Troubleshooting

### Tests Not Found

```bash
# Check test file naming
# Go: *_test.go
# JS/TS: *.test.ts, *.spec.ts, __tests__/*.ts
# Python: test_*.py, *_test.py

# Verify test discovery
npx vitest --reporter=verbose
pytest --collect-only
go test -v ./... 2>&1 | head -20
```

### Integration Tests Failing

```bash
# Ensure services are running
docker-compose up -d

# Check database connection
docker ps

# Run with verbose output
go test -v -tags=integration ./...
```

### Coverage Below Threshold

```bash
# Identify uncovered lines
go tool cover -func=coverage.out
npx vitest run --coverage --reporter=text

# Focus on critical paths first
```

---

## Output

```
🧪 Running tests...

Command: npx vitest run

Test Files  3 passed (3)
     Tests  12 passed (12)
  Duration  1.23s

✅ All tests passing

Coverage:
  Statements: 87% (target: 85%) ✅
  Branches:   82% (target: 80%) ✅
  Functions:  91% (target: 85%) ✅
  Lines:      87% (target: 85%) ✅
```

---

## See Also

- `/tdd` - TDD workflow
- `/lint` - Code quality checks
- `/dev` - Feature-driven development
