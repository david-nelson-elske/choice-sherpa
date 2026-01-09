# Security Review - Application Security Analysis

> **Purpose**: Analyze code for security vulnerabilities before merge.
> **Standard**: Enforces `docs/architecture/APPLICATION-SECURITY-STANDARD.md`
> **Integration**: Called automatically by `/pr`, can run standalone.

---

## Usage

```bash
# Review staged/uncommitted changes
/security-review

# Review specific files
/security-review src/handlers/auth.rs src/models/user.rs

# Review entire PR diff
/security-review --pr

# Full codebase audit (slow)
/security-review --full
```

## Options

| Option | Description |
|--------|-------------|
| `--pr` | Review changes in current PR branch vs base |
| `--full` | Full codebase security audit |
| `--fix` | Attempt to auto-fix simple issues |
| `--strict` | Fail on any warning (not just errors) |

---

## Security Review Process

### Phase 1: Dependency Audit

Run automated dependency scanners:

```bash
# Rust dependencies
cargo audit

# Node dependencies (if frontend changes)
npm audit --prefix frontend
```

**Output**:
```
Dependency Audit Results
========================
Rust:  0 vulnerabilities
Node:  0 vulnerabilities
```

**Failure Criteria**:
- Any HIGH or CRITICAL severity
- Any advisory without workaround

---

### Phase 2: Static Analysis

Analyze changed files for security patterns:

#### 2.1 OWASP Top 10 Checks

| Check | Pattern | Severity |
|-------|---------|----------|
| A01: Broken Access Control | Missing ownership checks | CRITICAL |
| A02: Cryptographic Failures | Weak algorithms, hardcoded secrets | CRITICAL |
| A03: Injection | String interpolation in queries | CRITICAL |
| A04: Insecure Design | Missing auth on endpoints | HIGH |
| A05: Security Misconfiguration | Debug enabled, verbose errors | MEDIUM |
| A06: Vulnerable Components | (Handled in Phase 1) | - |
| A07: Auth Failures | Missing validation, weak sessions | HIGH |
| A08: Integrity Failures | Missing signature verification | HIGH |
| A09: Logging Failures | Sensitive data in logs | MEDIUM |
| A10: SSRF | Unvalidated URLs | HIGH |

#### 2.2 Rust-Specific Checks

```rust
// CRITICAL: SQL Injection via string formatting
// Pattern: format!(...) or .to_string() in SQL context
format!("SELECT * FROM users WHERE id = '{}'", user_id)  // VULNERABLE

// CRITICAL: Command Injection
// Pattern: Command::new with shell or user input
Command::new("sh").arg("-c").arg(user_input)  // VULNERABLE

// HIGH: Unwrap in production code
// Pattern: .unwrap() outside of tests
some_option.unwrap()  // May panic

// HIGH: Unsafe without justification
// Pattern: unsafe block without safety comment
unsafe { /* no comment */ }  // SUSPICIOUS

// MEDIUM: Missing error handling
// Pattern: let _ = fallible_operation()
let _ = file.write_all(data);  // Error ignored
```

#### 2.3 Frontend-Specific Checks

```typescript
// CRITICAL: XSS via innerHTML or @html
element.innerHTML = userContent;  // VULNERABLE
{@html userContent}  // VULNERABLE unless sanitized

// HIGH: Sensitive data in client storage
localStorage.setItem('token', jwt);  // INSECURE
sessionStorage.setItem('password', pwd);  // INSECURE

// MEDIUM: Missing input validation
const data = await response.json();  // Unvalidated

// MEDIUM: Hardcoded URLs/secrets
const API_KEY = 'sk-1234...';  // HARDCODED SECRET
```

---

### Phase 3: Access Control Review

For any code that accesses resources:

```
Access Control Analysis
=======================

File: src/handlers/session.rs

get_session():
  Auth Required: YES (from middleware)
  Ownership Check: YES (line 45)
  Status: PASS

delete_session():
  Auth Required: YES (from middleware)
  Ownership Check: MISSING
  Status: FAIL - Add ownership verification

create_session():
  Auth Required: YES (from middleware)
  Ownership Check: N/A (creating new resource)
  Rate Limiting: YES (line 12)
  Status: PASS
```

---

### Phase 4: Secrets Detection

Scan for hardcoded secrets:

```
Secrets Scan
============

Patterns checked:
  - API keys (sk-, pk-, api_key, apikey)
  - Database URLs (postgres://, mysql://)
  - Private keys (BEGIN RSA PRIVATE KEY)
  - AWS credentials (AKIA, aws_secret)
  - Generic secrets (password=, secret=)

Results:
  src/config.rs:15  WARN  Pattern 'DATABASE_URL' - OK if env var
  src/tests/mod.rs:5  INFO  Test credential - OK (test only)

Status: PASS
```

---

### Phase 5: Security Headers Check

For HTTP-related code:

```
HTTP Security Headers
=====================

Required headers:
  Strict-Transport-Security: PRESENT
  X-Content-Type-Options: PRESENT
  X-Frame-Options: PRESENT
  Content-Security-Policy: PRESENT
  Referrer-Policy: PRESENT
  Permissions-Policy: PRESENT

Missing headers:
  None

Status: PASS
```

---

## Output Format

### Summary

```
Security Review Summary
=======================

Changes analyzed: 8 files (+324 lines, -12 lines)

Findings:
  CRITICAL: 0
  HIGH:     1
  MEDIUM:   2
  LOW:      0
  INFO:     3

Dependency vulnerabilities: 0

Overall Status: NEEDS ATTENTION

Issues to resolve before merge:
  1. [HIGH] Missing ownership check in delete_session()
     File: src/handlers/session.rs:78
     Fix: Add session.owner_id == user_id check

  2. [MEDIUM] Unwrap() in production code
     File: src/services/email.rs:45
     Fix: Use ? operator or handle error

  3. [MEDIUM] Error ignored
     File: src/adapters/postgres.rs:102
     Fix: Log or propagate the error
```

### Detailed Findings

Each finding includes:

```
[SEVERITY] Issue Title
======================
File: path/to/file.rs:line_number
Category: OWASP A0X / Code Quality / etc.

Description:
  Brief explanation of the vulnerability

Vulnerable Code:
  ```rust
  // The problematic code snippet
  ```

Recommended Fix:
  ```rust
  // Corrected code example
  ```

Reference:
  - docs/architecture/APPLICATION-SECURITY-STANDARD.md#section
  - https://owasp.org/relevant-page
```

---

## Severity Definitions

| Severity | Description | Merge Allowed? |
|----------|-------------|----------------|
| **CRITICAL** | Exploitable vulnerability (injection, auth bypass) | NO |
| **HIGH** | Significant risk (missing authz, data exposure) | NO |
| **MEDIUM** | Security weakness (error handling, logging) | YES (with justification) |
| **LOW** | Minor issue or best practice | YES |
| **INFO** | Informational / suggestion | YES |

---

## Integration with PR Workflow

When `/pr` is invoked, `/security-review` runs automatically:

```
/pr
  │
  ├─► /test              (must pass)
  ├─► /lint              (must pass)
  ├─► /security-review   (CRITICAL/HIGH must be 0)
  │
  └─► Create PR (if all pass)
```

### PR Checklist Addition

The PR body includes security status:

```markdown
## Security Review
- [x] Dependency audit passed
- [x] No CRITICAL/HIGH findings
- [x] Secrets scan passed
- [ ] Manual review required: [reason]
```

---

## Exceptions and Overrides

### Suppressing False Positives

Add inline comments to suppress specific warnings:

```rust
// security: ignore - Test credentials only used in tests
const TEST_API_KEY: &str = "test-key-12345";

// security: ignore A03 - Input validated by extract
let query = format!("SELECT * FROM cache WHERE key = '{}'", validated_key);
```

### Global Exceptions

For project-wide exceptions, add to `.security-review.yaml`:

```yaml
exceptions:
  - rule: hardcoded-secret
    pattern: "TEST_*"
    reason: "Test credentials are acceptable"

  - rule: unwrap-in-production
    files:
      - "src/bin/*.rs"  # CLI tools can panic
    reason: "CLI tools exit on error"
```

---

## Automated Fixes

With `--fix` flag, auto-correct simple issues:

| Issue | Auto-Fix |
|-------|----------|
| Missing `#[instrument]` on handlers | Add tracing attribute |
| Unwrap in simple cases | Convert to `?` |
| Missing security headers | Add middleware registration |
| Outdated dependencies | Run `cargo update` (minor only) |

**Note**: Auto-fix always shows diff for approval before applying.

---

## Full Audit Mode

`/security-review --full` performs comprehensive analysis:

1. All dependency audits
2. Full codebase scan (not just changes)
3. Architecture review
4. Configuration audit
5. Generate security report

Output: `docs/reviews/security-audit-YYYY-MM-DD.md`

---

## Security Review Checklist

This checklist is used for manual review alongside automated checks:

### Input Handling
- [ ] All user input validated at API boundaries
- [ ] No SQL string interpolation
- [ ] No shell command construction from input
- [ ] URL allowlist for external requests
- [ ] File paths validated (no traversal)

### Authentication & Authorization
- [ ] All non-public endpoints require auth
- [ ] Ownership verified for resource access
- [ ] No privilege escalation paths
- [ ] Session management secure

### Data Protection
- [ ] No secrets in code or logs
- [ ] PII handling follows policy
- [ ] Encryption used appropriately
- [ ] Secure deletion implemented

### Error Handling
- [ ] No sensitive data in responses
- [ ] Errors logged appropriately
- [ ] Fail-secure behavior

### Code Quality
- [ ] No unsafe without documentation
- [ ] No ignored errors
- [ ] No panics in library code

---

## Examples

### Example 1: Clean Review

```
> /security-review

Security Review Summary
=======================

Changes analyzed: 3 files (+45 lines, -10 lines)

Findings:
  CRITICAL: 0
  HIGH:     0
  MEDIUM:   0
  LOW:      0
  INFO:     1

Dependency vulnerabilities: 0

Overall Status: PASS

Info:
  1. [INFO] Consider adding rate limiting
     File: src/handlers/api.rs:25
     Suggestion: Add rate limit for sensitive endpoint
```

### Example 2: Issues Found

```
> /security-review

Security Review Summary
=======================

Changes analyzed: 5 files (+128 lines, -5 lines)

Findings:
  CRITICAL: 1
  HIGH:     0
  MEDIUM:   1
  LOW:      0

Dependency vulnerabilities: 0

Overall Status: BLOCKED

CRITICAL Issues (must fix):

[CRITICAL] SQL Injection Vulnerability
==========================================
File: src/repositories/search.rs:34
Category: OWASP A03 - Injection

Description:
  User input directly interpolated into SQL query string.
  Attacker can execute arbitrary SQL commands.

Vulnerable Code:
  let query = format!(
      "SELECT * FROM sessions WHERE title LIKE '%{}%'",
      search_term  // UNESCAPED USER INPUT
  );
  sqlx::query(&query).fetch_all(&pool).await

Recommended Fix:
  sqlx::query!(
      "SELECT * FROM sessions WHERE title LIKE $1",
      format!("%{}%", search_term)  // Parameterized
  )
  .fetch_all(&pool)
  .await

Reference:
  - docs/architecture/APPLICATION-SECURITY-STANDARD.md#a03-injection


MEDIUM Issues (should fix):

[MEDIUM] Unwrap in Production Path
==================================
File: src/services/notification.rs:67

Vulnerable Code:
  let config = load_config().unwrap();

Recommended Fix:
  let config = load_config()?;


Fix these issues before merging.
```

---

## See Also

- `docs/architecture/APPLICATION-SECURITY-STANDARD.md` - Security requirements
- `/pr` - Pull request workflow (includes security review)
- `/lint` - Code quality checks
- `/test` - Test execution

---

*Skill Version: 1.0.0*
*Created: 2026-01-08*
