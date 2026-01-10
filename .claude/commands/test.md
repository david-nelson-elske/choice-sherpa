# Test - Run Tests

Run tests with coverage and filtering options.

## Usage

```
/test [scope] [options]
```

| Argument | Description |
|----------|-------------|
| `scope` | `all` (default), `unit`, `integration`, `e2e`, or `<path>` |
| `--coverage` | Include coverage report |
| `--watch` | Watch mode |
| `--filter <pattern>` | Filter tests by name |

---

## Auto-Detection

| Detection | Command |
|-----------|---------|
| `Cargo.toml` | `cargo test` |
| `package.json` + Vitest | `npx vitest run` |

---

## Commands

### Rust

```bash
cargo test                              # All tests
cargo test test_user_register           # Specific test
cargo test -- --nocapture               # Show output
cargo tarpaulin --out Html              # Coverage
```

### TypeScript (Vitest)

```bash
npx vitest run                          # All tests
npx vitest run --coverage               # With coverage
npx vitest run -t "should validate"     # Filter by name
npx vitest                              # Watch mode
```

---

## Configuration

Override in `CLAUDE.md`:

```markdown
## Test Commands
- test_all: cargo test
- test_coverage: cargo tarpaulin --out Html
- test_frontend: cd frontend && npm test
```

---

## Coverage Targets

| Layer | Target |
|-------|--------|
| Domain/Model | 90%+ |
| Service/Business | 85%+ |
| API/Controller | 80%+ |

---

## Output

```
🧪 Running tests...

Test Files  3 passed (3)
     Tests  12 passed (12)
  Duration  1.23s

✅ All tests passing

Coverage:
  Statements: 87% (target: 85%) ✅
  Lines:      87% (target: 85%) ✅
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Tests not found | Check naming: `*_test.rs`, `*.test.ts` |
| Coverage below target | Run with `--coverage`, identify uncovered lines |
