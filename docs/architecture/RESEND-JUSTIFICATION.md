# Email Provider Selection: Resend

> **Decision:** Resend
> **Date:** 2026-01-07

---

## Summary

Resend selected for transactional email delivery due to modern developer experience, SMTP support for Zitadel integration, available Rust SDK, and free tier sufficient for MVP volume.

---

## Email Requirements

| Source | Email Type | Expected Volume |
|--------|------------|-----------------|
| Zitadel | Password reset | Low |
| Zitadel | Email verification | Low |
| Zitadel | MFA codes (if email-based) | Low |
| Application | Session sharing (future) | Minimal |
| Application | Decision reminders (future) | Minimal |

**Total:** Low volume, primarily authentication-related transactional emails.

---

## Key Factors

### 1. Zitadel SMTP Compatibility

Zitadel requires SMTP for outbound email. Resend provides SMTP access:

```yaml
# Zitadel SMTP configuration
SMTP:
  Host: smtp.resend.com
  Port: 465
  User: resend
  Password: ${RESEND_API_KEY}
  TLS: true
  FromAddress: noreply@choicesherpa.com
```

### 2. Rust SDK Available

[`resend-rs`](https://crates.io/crates/resend-rs) crate available for application emails:

```toml
[dependencies]
resend-rs = "0.5"
```

### 3. Pricing Fit

| Tier | Emails/Month | Cost |
|------|--------------|------|
| Free | 3,000 | $0 |
| Pro | 50,000 | $20/mo |

Free tier sufficient for development and early production.

### 4. Developer Experience

- Simple REST API
- Clear documentation
- Quick setup (minutes, not hours)
- No complex configuration

---

## Integration Architecture

```
┌─────────────┐         SMTP          ┌─────────────┐
│   Zitadel   │ ───────────────────► │   Resend    │ ───► User inbox
│  (auth)     │   (password reset,   │             │
│             │    verification)     │             │
└─────────────┘                       └─────────────┘
                                            ▲
┌─────────────┐       REST API              │
│ Rust Backend│ ────────────────────────────┘
│  (future)   │   (sharing, reminders)
└─────────────┘
```

---

## Hexagonal Architecture Compliance

### Port Definition

```rust
// ports/email.rs

#[async_trait]
pub trait EmailProvider: Send + Sync {
    async fn send(&self, message: EmailMessage) -> Result<(), EmailError>;
}

pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    pub body: EmailBody,
}

pub enum EmailBody {
    Plain(String),
    Html(String),
}

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("Invalid recipient")]
    InvalidRecipient,
    #[error("Send failed")]
    SendFailed,
    #[error("Service unavailable")]
    ServiceUnavailable,
}
```

### Adapter Implementation

```rust
// adapters/email/resend.rs

use resend_rs::{Resend, CreateEmailOptions};
use crate::ports::{EmailProvider, EmailMessage, EmailError};

pub struct ResendAdapter {
    client: Resend,
    from_address: String,
}

impl ResendAdapter {
    pub fn new(api_key: &str, from_address: &str) -> Self {
        Self {
            client: Resend::new(api_key),
            from_address: from_address.to_string(),
        }
    }
}

#[async_trait]
impl EmailProvider for ResendAdapter {
    async fn send(&self, message: EmailMessage) -> Result<(), EmailError> {
        let options = CreateEmailOptions::new(&self.from_address, &message.to, &message.subject)
            .with_html(&message.body.to_html());

        self.client
            .emails
            .send(options)
            .await
            .map_err(|_| EmailError::SendFailed)?;

        Ok(())
    }
}
```

### Swappability

```rust
// Resend
let email: Arc<dyn EmailProvider> = Arc::new(ResendAdapter::new(key, from));

// Swap to SendGrid
let email: Arc<dyn EmailProvider> = Arc::new(SendGridAdapter::new(key, from));

// Swap to Amazon SES
let email: Arc<dyn EmailProvider> = Arc::new(SesAdapter::new(config));
```

---

## Configuration

```bash
# Environment variables (generic naming)
EMAIL_API_KEY=re_xxxxx
EMAIL_FROM_ADDRESS=noreply@choicesherpa.com
```

```rust
// config.rs
pub struct EmailConfig {
    pub api_key: String,
    pub from_address: String,
}
```

---

## Testing

```rust
pub struct MockEmailProvider {
    pub sent: Arc<Mutex<Vec<EmailMessage>>>,
}

#[async_trait]
impl EmailProvider for MockEmailProvider {
    async fn send(&self, message: EmailMessage) -> Result<(), EmailError> {
        self.sent.lock().unwrap().push(message);
        Ok(())
    }
}

#[tokio::test]
async fn test_sends_notification() {
    let mock = MockEmailProvider::default();
    // ... test logic
    assert_eq!(mock.sent.lock().unwrap().len(), 1);
}
```

---

## Alternatives Considered

| Provider | Rejection Reason |
|----------|------------------|
| **SendGrid** | More complex, overkill for volume |
| **Postmark** | Good alternative, slightly higher cost |
| **Amazon SES** | More AWS setup, better at high scale |
| **Mailgun** | Similar to SendGrid, more complexity |
| **Self-hosted** | Operational burden, deliverability risk |

---

## Trade-off Accepted

Younger company than SendGrid/Postmark, accepted in exchange for:

- Simpler developer experience
- Free tier covers MVP needs
- Clean API design
- Rust SDK available
- No vendor lock-in (standard SMTP/email)

---

## Sources

- [Resend Documentation](https://resend.com/docs)
- [resend-rs crate](https://crates.io/crates/resend-rs)
- [Resend SMTP Documentation](https://resend.com/docs/send-with-smtp)
