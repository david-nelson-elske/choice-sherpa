# Backend Language Selection: Rust

> **Decision:** Rust for backend implementation
> **Date:** 2026-01-07

---

## Summary

Rust selected over Go for type safety and runtime characteristics that align with the domain requirements.

---

## Key Factors

### 1. Type-Safe Domain Modeling

Nine PrOACT component types require polymorphic handling. Rust's sum types provide compile-time exhaustiveness:

```rust
enum Component {
    IssueRaising(IssueRaisingData),
    ProblemFrame(ProblemFrameData),
    // ...
}

// Compiler enforces all cases handled
match component {
    Component::IssueRaising(data) => ...,
    // Missing case = compile error
}
```

Go alternative (`interface{}`) defers type checking to runtime.

### 2. Compiler as TDD Partner

Rust's compiler catches bug categories before tests run:

- Missing enum cases
- Null/None access
- Unhandled errors
- Type mismatches

This reduces test surface area and catches errors earlier in the development cycle.

### 3. Runtime Predictability

Real-time conversation streaming benefits from:

- No garbage collection pauses
- Predictable latency under load
- Bounded memory footprint

### 4. AI Integration is Just HTTP

LLM integration (OpenAI, Anthropic) is REST + SSE streaming. No special "AI ecosystem" needed—standard HTTP client with streaming support suffices. Rust's `reqwest` + `tokio` handles this cleanly.

### 5. Hexagonal Architecture Fit

Rust traits map naturally to ports. The type system enforces boundary discipline that requires convention in other languages.

---

## Trade-off Accepted

Slower compile times (~3-10s incremental vs Go's ~1-2s) accepted in exchange for compiler-verified correctness.

---

## Initial Crate Stack

| Purpose | Crate |
|---------|-------|
| Async runtime | tokio |
| HTTP framework | axum |
| Database | sqlx |
| Serialization | serde |
| Error handling | thiserror |
| UUID | uuid |
| Testing mocks | mockall |
