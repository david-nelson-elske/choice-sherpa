# Security Review - Application Security Analysis

Analyze code for security vulnerabilities. Called automatically by `/pr`.

## Usage

```
/security-review                    # Review staged changes
/security-review --pr               # Review PR diff
/security-review --full             # Full codebase audit
/security-review src/handlers/*.rs  # Specific files
```

| Option | Description |
|--------|-------------|
| `--pr` | Review current PR vs base |
| `--full` | Complete codebase scan |
| `--fix` | Auto-fix simple issues |
| `--strict` | Fail on warnings |

---

## Review Phases

### 1. Dependency Audit

```bash
cargo audit                    # Rust
npm audit --prefix frontend    # TypeScript
```

### 2. OWASP Top 10 Static Analysis

| Check | Severity | Pattern |
|-------|----------|---------|
| A01: Access Control | CRITICAL | Missing ownership checks |
| A02: Crypto Failures | CRITICAL | Weak algorithms, hardcoded secrets |
| A03: Injection | CRITICAL | String interpolation in queries |
| A04: Insecure Design | HIGH | Missing auth on endpoints |
| A07: Auth Failures | HIGH | Missing validation, weak sessions |
| A09: Logging Failures | MEDIUM | Sensitive data in logs |

### 3. Access Control Review

Verify for each resource-accessing function:
- Authentication required?
- Ownership verified?
- Rate limiting present?

### 4. Secrets Detection

Patterns: API keys, database URLs, private keys, AWS credentials

---

## Severity Levels

| Severity | Description | Merge? |
|----------|-------------|--------|
| CRITICAL | Exploitable (injection, auth bypass) | NO |
| HIGH | Significant risk (missing authz) | NO |
| MEDIUM | Security weakness | YES (justify) |
| LOW/INFO | Best practice | YES |

---

## Rust-Specific Checks

| Pattern | Severity | Issue |
|---------|----------|-------|
| `format!()` in SQL | CRITICAL | SQL injection |
| `Command::new().arg(user_input)` | CRITICAL | Command injection |
| `.unwrap()` in prod | HIGH | May panic |
| `unsafe` without comment | HIGH | Needs justification |
| `let _ = fallible()` | MEDIUM | Error ignored |

---

## TypeScript-Specific Checks

| Pattern | Severity | Issue |
|---------|----------|-------|
| `innerHTML = user` / `{@html user}` | CRITICAL | XSS |
| `localStorage.setItem('token',...)` | HIGH | Token exposure |
| Unvalidated `await response.json()` | MEDIUM | Type confusion |

---

## Output

```
Security Review Summary
=======================

Changes: 8 files (+324, -12 lines)

Findings:
  CRITICAL: 0
  HIGH:     1
  MEDIUM:   2

Status: NEEDS ATTENTION

Issues:
  1. [HIGH] Missing ownership check
     File: src/handlers/session.rs:78
     Fix: Add session.owner_id == user_id check
```

---

## Exceptions

```rust
// security: ignore - Test credentials only
const TEST_KEY: &str = "test-123";

// security: ignore A03 - Input pre-validated
let query = format!("...");
```

---

## Reference

- Security patterns: `.claude/lib/examples/rust/security.md`
- Security patterns: `.claude/lib/examples/typescript/security.md`
- Standard: `docs/architecture/APPLICATION-SECURITY-STANDARD.md`
