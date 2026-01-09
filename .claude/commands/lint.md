# Lint - Code Quality Checks

Run linters to catch code quality issues. Detects project type and uses appropriate tools.

## Usage
```
/lint [scope] [--fix]
```

## Arguments
- `scope`: What to lint (optional, defaults to "all")
  - `all` - Lint everything
  - `backend` - Backend code only
  - `frontend` - Frontend code only
  - `<path>` - Specific file or directory

- `--fix`: Auto-fix issues where possible

---

## Auto-Detection

The skill detects project type and uses appropriate linters:

| Detection | Linter |
|-----------|--------|
| `go.mod` | `golangci-lint run ./...` |
| `package.json` + eslint | `npx eslint .` |
| `pyproject.toml` + ruff | `ruff check .` |
| `pyproject.toml` + flake8 | `flake8 .` |
| `Cargo.toml` | `cargo clippy` |

---

## Project Configuration

Override auto-detection in `CLAUDE.md`:

```markdown
## Lint Commands
- lint_backend: `cd backend && golangci-lint run ./...`
- lint_frontend: `cd frontend && npm run lint`
- lint_fix_backend: `cd backend && golangci-lint run --fix ./...`
- lint_fix_frontend: `cd frontend && npm run lint -- --fix`
- typecheck: `cd frontend && npm run typecheck`
```

---

## Commands by Language

### Go (golangci-lint)

```bash
# Run all linters
golangci-lint run ./...

# Auto-fix
golangci-lint run --fix ./...

# Specific directory
golangci-lint run ./internal/user/...

# Verbose
golangci-lint run -v ./...

# Show all issues (not just new)
golangci-lint run --new=false ./...
```

**Common Issues:**

```go
// errcheck: Unhandled error
// ❌ Wrong
defer rows.Close()

// ✅ Correct
defer func() { _ = rows.Close() }()
```

```go
// unused: Unused variable
// ❌ Wrong
result, err := doSomething()
if err != nil { return err }
// result never used

// ✅ Correct
_, err := doSomething()
```

### TypeScript/JavaScript (ESLint)

```bash
# Run linter
npx eslint .

# Auto-fix
npx eslint . --fix

# Specific files
npx eslint src/services/*.ts

# With TypeScript type checking
npm run typecheck  # tsc --noEmit
```

**Common Issues:**

```typescript
// @typescript-eslint/no-unused-vars
// ❌ Wrong
import { unused } from './module';

// ✅ Correct - remove or prefix with underscore
import type { _Unused } from './module';
```

### Python (ruff)

```bash
# Run linter
ruff check .

# Auto-fix
ruff check --fix .

# Show all rules
ruff check --select ALL .

# Specific file
ruff check src/services/user.py
```

**Common Issues:**

```python
# F401: Unused import
# ❌ Wrong
from os import path  # never used

# ✅ Correct
# Remove unused import
```

### Python (flake8 + black)

```bash
# Lint
flake8 .

# Format
black .

# Check formatting
black --check .
```

### Rust (clippy)

```bash
# Run clippy
cargo clippy

# Treat warnings as errors
cargo clippy -- -D warnings

# Auto-fix
cargo clippy --fix
```

---

## Type Checking

For TypeScript projects, also run type checking:

```bash
# TypeScript
npx tsc --noEmit

# Or if configured
npm run typecheck
```

For Python with mypy:

```bash
mypy src/
```

---

## Pre-Commit Workflow

Run this sequence before every commit:

```bash
# 1. Lint
/lint

# 2. Type check (if applicable)
npm run typecheck  # TypeScript
mypy src/          # Python

# 3. Test
/test

# 4. Commit
/commit "feat: your message"
```

---

## CI Alignment

Ensure local linting matches CI configuration:

### Go
```yaml
# CI uses golangci-lint-action
- uses: golangci/golangci-lint-action@v4
  with:
    version: latest
```

```bash
# Match CI version locally
golangci-lint --version
```

### JavaScript/TypeScript
```yaml
# CI runs
- run: npm run lint
- run: npm run typecheck
```

### Python
```yaml
# CI runs
- run: ruff check .
- run: mypy src/
```

---

## Troubleshooting

### "Command not found"

```bash
# Install Go linter
go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest

# Install ruff
pip install ruff

# ESLint should be in devDependencies
npm install
```

### Different Results Locally vs CI

1. Check tool versions match
2. Pull latest main and rebase
3. Re-run after rebase (new code may have issues)

### Too Many Errors

```bash
# Fix incrementally - specific directory
golangci-lint run ./internal/user/...
npx eslint src/services/

# Or auto-fix what's possible
/lint --fix
```

---

## Output

```
🔍 Running lint checks...

Backend (golangci-lint):
  ✅ No issues found

Frontend (eslint):
  ⚠️  2 warnings

  src/components/Button.tsx
    12:5  warning  Unused variable 'x'  @typescript-eslint/no-unused-vars
    15:1  warning  Missing return type  @typescript-eslint/explicit-function-return-type

Frontend (typecheck):
  ✅ No type errors

Summary:
  Errors:   0
  Warnings: 2

Run `/lint --fix` to auto-fix where possible.
```

---

## See Also

- `/test` - Run tests
- `/tdd-refactor` - Code quality improvements
- `/dev` - Feature-driven workflow
- `/pr` - PR preparation (requires passing lint)
