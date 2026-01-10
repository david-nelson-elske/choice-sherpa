# Lint - Code Quality Checks

Run linters to catch code quality issues.

## Usage

```
/lint [scope] [--fix]
```

| Argument | Description |
|----------|-------------|
| `scope` | `all` (default), `backend`, `frontend`, or `<path>` |
| `--fix` | Auto-fix issues where possible |

---

## Auto-Detection

| Detection | Linter |
|-----------|--------|
| `Cargo.toml` | `cargo clippy -- -D warnings` |
| `package.json` + ESLint | `npx eslint .` |

---

## Commands

### Rust (Clippy)

```bash
cargo clippy                    # Run clippy
cargo clippy -- -D warnings     # Treat warnings as errors
cargo clippy --fix              # Auto-fix
```

### TypeScript (ESLint)

```bash
npx eslint .                    # Run linter
npx eslint . --fix              # Auto-fix
npx tsc --noEmit                # Type check
```

---

## Configuration

Override in `CLAUDE.md`:

```markdown
## Lint Commands
- lint: cargo clippy -- -D warnings
- lint_frontend: cd frontend && npm run lint
- typecheck: cd frontend && npm run typecheck
```

---

## Pre-Commit Sequence

```
/lint → /test → /commit
```

---

## Output

```
🔍 Running lint checks...

Backend (clippy):
  ✅ No issues

Frontend (eslint):
  ⚠️  2 warnings

Summary:
  Errors:   0
  Warnings: 2

Run `/lint --fix` to auto-fix.
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Command not found | Install: `cargo install clippy` or `npm install` |
| Too many errors | Fix incrementally: `/lint backend` or `/lint --fix` |
