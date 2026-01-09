# Integration: Notification Service

**Architecture:** docs/architecture/SYSTEM-ARCHITECTURE.md
**Type:** External Service Integration
**Priority:** P2 (Required for membership and user engagement)
**Depends On:** foundation module, membership module, event infrastructure

> Multi-channel notification system for email and in-app notifications, triggered by domain events.

---

## Overview

The Notification Service handles all outbound communications with users: transactional emails (receipts, password resets), engagement emails (decision reminders), and in-app notifications. It's event-driven—domain events trigger notification workflows.

### Notification Channels

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Domain Events                                  │
│   MembershipActivated │ CycleCompleted │ InactivityDetected                 │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                       Notification Router                                   │
│   Routes events to appropriate channels based on type and user preferences  │
└─────────────────────────────────────────────────────────────────────────────┘
          │                                           │
          ▼                                           ▼
┌─────────────────────────┐             ┌─────────────────────────┐
│       Email Channel     │             │    In-App Channel       │
│   (transactional/bulk)  │             │   (real-time/badge)     │
└─────────────────────────┘             └─────────────────────────┘
          │                                           │
          ▼                                           ▼
┌─────────────────────────┐             ┌─────────────────────────┐
│        Resend           │             │   WebSocket / Redis     │
│   (email provider)      │             │   (push to frontend)    │
└─────────────────────────┘             └─────────────────────────┘
```

---

## Notification Types

### Transactional (Immediate)

| Event | Notification | Channel | Template |
|-------|--------------|---------|----------|
| `MembershipActivated` | Welcome + receipt | Email | `welcome-paid` |
| `MembershipCanceled` | Cancellation confirmation | Email | `membership-canceled` |
| `MembershipExpiring` | Renewal reminder | Email | `renewal-reminder` |
| `PaymentFailed` | Payment issue alert | Email | `payment-failed` |
| `PasswordResetRequested` | Reset link | Email | `password-reset` |
| `ExportCompleted` | Download link | Email + In-App | `export-ready` |

### Engagement (Batched)

| Trigger | Notification | Channel | Template |
|---------|--------------|---------|----------|
| Session inactive 7 days | Continue reminder | Email | `session-reminder` |
| Cycle at Tradeoffs stage | Completion encouragement | Email | `almost-done` |
| First decision completed | Celebration | Email + In-App | `first-decision` |
| Weekly digest | Activity summary | Email | `weekly-digest` |

### In-App Only

| Event | Notification | Display |
|-------|--------------|---------|
| `CycleCompleted` | Decision complete | Toast + badge |
| `ComponentOutputUpdated` | Progress update | Badge |
| `NewFeatureAvailable` | Feature announcement | Banner |

---

## Architecture

### Port Definition

```rust
// backend/src/ports/notification.rs

use async_trait::async_trait;

/// Port for sending notifications
#[async_trait]
pub trait NotificationSender: Send + Sync {
    /// Send a single notification
    async fn send(&self, notification: Notification) -> Result<NotificationResult, NotificationError>;

    /// Send multiple notifications (batch)
    async fn send_batch(&self, notifications: Vec<Notification>) -> Result<BatchResult, NotificationError>;

    /// Schedule a notification for later
    async fn schedule(
        &self,
        notification: Notification,
        send_at: Timestamp,
    ) -> Result<ScheduledNotification, NotificationError>;

    /// Cancel a scheduled notification
    async fn cancel(&self, scheduled_id: &ScheduledNotificationId) -> Result<(), NotificationError>;
}

/// Port for notification preferences
#[async_trait]
pub trait NotificationPreferences: Send + Sync {
    /// Get user's notification preferences
    async fn get(&self, user_id: &UserId) -> Result<UserPreferences, NotificationError>;

    /// Update user's notification preferences
    async fn update(
        &self,
        user_id: &UserId,
        preferences: UserPreferences,
    ) -> Result<(), NotificationError>;

    /// Check if user opted out of a channel
    async fn is_opted_out(
        &self,
        user_id: &UserId,
        channel: NotificationChannel,
    ) -> Result<bool, NotificationError>;
}
```

### Domain Types

```rust
// backend/src/domain/notification/types.rs

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: NotificationId,
    pub user_id: UserId,
    pub notification_type: NotificationType,
    pub channel: NotificationChannel,
    pub template: TemplateId,
    pub data: TemplateData,
    pub priority: NotificationPriority,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationChannel {
    Email,
    InApp,
    Push,  // Future: mobile push
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    Transactional,  // Critical, always sent
    Engagement,     // Can be opted out
    Marketing,      // Requires explicit opt-in
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationPriority {
    High,    // Send immediately
    Normal,  // Send within minutes
    Low,     // Can be batched
}

#[derive(Debug, Clone)]
pub struct TemplateData(pub HashMap<String, serde_json::Value>);

#[derive(Debug, Clone)]
pub struct UserPreferences {
    pub email_enabled: bool,
    pub in_app_enabled: bool,
    pub engagement_emails: bool,
    pub weekly_digest: bool,
    pub timezone: String,
    pub quiet_hours: Option<QuietHours>,
}

#[derive(Debug, Clone)]
pub struct QuietHours {
    pub start: u8,  // Hour 0-23
    pub end: u8,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            email_enabled: true,
            in_app_enabled: true,
            engagement_emails: true,
            weekly_digest: true,
            timezone: "UTC".to_string(),
            quiet_hours: None,
        }
    }
}
```

---

## Email Provider Adapter

### Resend Implementation

Per the [RESEND-JUSTIFICATION.md](../../docs/architecture/RESEND-JUSTIFICATION.md), we use Resend for transactional email delivery.

```rust
// backend/src/adapters/email/resend.rs

use resend_rs::{Resend, CreateEmailOptions};
use crate::ports::email::{EmailSender, Email, EmailResult, EmailError};

pub struct ResendEmailSender {
    client: Resend,
    from_address: String,
}

impl ResendEmailSender {
    pub fn new(api_key: &str, from_address: &str) -> Self {
        Self {
            client: Resend::new(api_key),
            from_address: from_address.to_string(),
        }
    }
}

#[async_trait]
impl EmailSender for ResendEmailSender {
    async fn send(&self, email: Email) -> Result<EmailResult, EmailError> {
        let mut options = CreateEmailOptions::new(
            &self.from_address,
            vec![email.to.clone()],
            &email.subject.unwrap_or_else(|| "Choice Sherpa".to_string()),
        );

        // Set HTML body with template data interpolated
        if let Some(html) = &email.html_body {
            options = options.with_html(html);
        }

        if let Some(text) = &email.text_body {
            options = options.with_text(text);
        }

        let response = self.client
            .emails
            .send(options)
            .await
            .map_err(|e| EmailError::SendFailed(e.to_string()))?;

        Ok(EmailResult {
            message_id: Some(response.id),
            sent_at: Timestamp::now(),
        })
    }
}

/// Email structure
#[derive(Debug, Clone)]
pub struct Email {
    pub to: String,
    pub subject: Option<String>,
    pub html_body: Option<String>,
    pub text_body: Option<String>,
    pub template_data: TemplateData,
}
```

### Swappability

The port abstraction allows swapping email providers:

```rust
// Production: Resend
let email_sender: Arc<dyn EmailSender> = Arc::new(
    ResendEmailSender::new(&config.api_key, &config.from_address)
);

// Alternative: SendGrid (if needed in future)
let email_sender: Arc<dyn EmailSender> = Arc::new(
    SendGridEmailSender::new(config)
);

// Development: Console (logs emails)
let email_sender: Arc<dyn EmailSender> = Arc::new(
    ConsoleEmailSender::new()
);
```

---

## Email Templates

### Template Structure

```
templates/
├── layouts/
│   ├── base.html           # Common layout
│   └── transactional.html  # Receipt layout
├── transactional/
│   ├── welcome-paid.html
│   ├── password-reset.html
│   ├── payment-failed.html
│   └── export-ready.html
└── engagement/
    ├── session-reminder.html
    ├── almost-done.html
    └── weekly-digest.html
```

### Template Example

```html
<!-- templates/transactional/welcome-paid.html -->
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Welcome to Choice Sherpa</title>
</head>
<body style="font-family: sans-serif; max-width: 600px; margin: 0 auto;">
  <h1>Welcome, {{user.display_name}}!</h1>

  <p>Your {{tier}} membership is now active.</p>

  <table style="width: 100%; border-collapse: collapse;">
    <tr>
      <td>Plan</td>
      <td>{{tier}} - ${{amount_paid}}/{{billing_period}}</td>
    </tr>
    <tr>
      <td>Sessions</td>
      <td>{{session_limit}}</td>
    </tr>
    <tr>
      <td>Next billing</td>
      <td>{{next_billing_date}}</td>
    </tr>
  </table>

  <a href="{{dashboard_url}}" style="background: #2563eb; color: white; padding: 12px 24px;">
    Start Your First Decision
  </a>

  <p style="font-size: 12px; color: #666;">
    Questions? Reply to this email or visit our help center.
  </p>
</body>
</html>
```

### Template Aliases

Templates are rendered locally using a template engine (e.g., Tera) and sent via Resend:

| Template Alias | Purpose | Variables |
|---------------|---------|-----------|
| `welcome-paid` | New paid member welcome | `user`, `tier`, `amount_paid` |
| `welcome-free` | Free tier signup | `user` |
| `password-reset` | Password reset link | `reset_url`, `expires_in` |
| `payment-failed` | Payment retry request | `user`, `last_four`, `retry_url` |
| `membership-canceled` | Cancellation confirmation | `user`, `end_date` |
| `renewal-reminder` | Upcoming renewal | `user`, `tier`, `renewal_date` |
| `export-ready` | Export download link | `user`, `download_url`, `expires_in` |
| `session-reminder` | Inactive session nudge | `user`, `session_title`, `resume_url` |
| `almost-done` | Near-completion encouragement | `user`, `session_title`, `progress` |
| `weekly-digest` | Weekly activity summary | `user`, `sessions`, `decisions` |

---

## In-App Notifications

### Notification Storage

```rust
// backend/src/adapters/notification/in_app.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InAppNotification {
    pub id: NotificationId,
    pub user_id: UserId,
    pub title: String,
    pub body: String,
    pub action_url: Option<String>,
    pub icon: Option<String>,
    pub read_at: Option<Timestamp>,
    pub created_at: Timestamp,
}

#[async_trait]
pub trait InAppNotificationStore: Send + Sync {
    /// Create new notification
    async fn create(&self, notification: InAppNotification) -> Result<(), StoreError>;

    /// Get unread notifications for user
    async fn get_unread(&self, user_id: &UserId) -> Result<Vec<InAppNotification>, StoreError>;

    /// Get all notifications (paginated)
    async fn get_all(
        &self,
        user_id: &UserId,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<InAppNotification>, StoreError>;

    /// Mark as read
    async fn mark_read(&self, notification_id: &NotificationId) -> Result<(), StoreError>;

    /// Mark all as read for user
    async fn mark_all_read(&self, user_id: &UserId) -> Result<(), StoreError>;

    /// Get unread count
    async fn unread_count(&self, user_id: &UserId) -> Result<u32, StoreError>;
}
```

### Real-Time Push via WebSocket

```rust
// backend/src/adapters/notification/websocket_pusher.rs

pub struct WebSocketNotificationPusher {
    room_manager: Arc<RoomManager>,
}

impl WebSocketNotificationPusher {
    pub async fn push(&self, user_id: &UserId, notification: InAppNotification) {
        let message = NotificationMessage {
            r#type: "notification".to_string(),
            payload: serde_json::to_value(&notification).unwrap(),
        };

        // Push to user's personal room
        let room = format!("user:{}", user_id);
        self.room_manager.broadcast(&room, message).await;
    }
}

#[derive(Serialize)]
struct NotificationMessage {
    r#type: String,
    payload: serde_json::Value,
}
```

### Frontend Integration

```typescript
// frontend/src/lib/stores/notifications.ts

import { writable, derived } from 'svelte/store';

interface InAppNotification {
  id: string;
  title: string;
  body: string;
  actionUrl?: string;
  icon?: string;
  readAt?: string;
  createdAt: string;
}

export const notifications = writable<InAppNotification[]>([]);
export const unreadCount = derived(notifications, ($n) =>
  $n.filter((n) => !n.readAt).length
);

// WebSocket handler
export function handleNotificationMessage(data: any) {
  if (data.type === 'notification') {
    notifications.update((n) => [data.payload, ...n]);
    showToast(data.payload);
  }
}

// API calls
export async function fetchNotifications() {
  const response = await fetch('/api/notifications');
  const data = await response.json();
  notifications.set(data.notifications);
}

export async function markAsRead(id: string) {
  await fetch(`/api/notifications/${id}/read`, { method: 'POST' });
  notifications.update((n) =>
    n.map((notification) =>
      notification.id === id
        ? { ...notification, readAt: new Date().toISOString() }
        : notification
    )
  );
}
```

```svelte
<!-- frontend/src/lib/components/NotificationBell.svelte -->
<script lang="ts">
  import { unreadCount, notifications, fetchNotifications, markAsRead } from '$lib/stores/notifications';
  import { onMount } from 'svelte';

  let isOpen = false;

  onMount(() => {
    fetchNotifications();
  });
</script>

<div class="notification-bell">
  <button on:click={() => isOpen = !isOpen}>
    <BellIcon />
    {#if $unreadCount > 0}
      <span class="badge">{$unreadCount}</span>
    {/if}
  </button>

  {#if isOpen}
    <div class="dropdown">
      {#each $notifications as notification}
        <div
          class="notification"
          class:unread={!notification.readAt}
          on:click={() => markAsRead(notification.id)}
        >
          <h4>{notification.title}</h4>
          <p>{notification.body}</p>
          <time>{formatRelative(notification.createdAt)}</time>
        </div>
      {/each}
    </div>
  {/if}
</div>
```

---

## Event-Driven Notification Workflow

### Notification Handler

```rust
// backend/src/application/handlers/notification_handler.rs

pub struct NotificationEventHandler {
    notification_sender: Arc<dyn NotificationSender>,
    preferences: Arc<dyn NotificationPreferences>,
    user_repo: Arc<dyn UserRepository>,
    template_renderer: Arc<TemplateRenderer>,
}

impl EventHandler for NotificationEventHandler {
    fn handles(&self) -> &[&str] {
        &[
            "membership.activated.v1",
            "membership.canceled.v1",
            "membership.expiring.v1",
            "cycle.completed.v1",
            "export.completed.v1",
        ]
    }

    async fn handle(&self, event: EventEnvelope) -> Result<(), HandlerError> {
        let notification = self.build_notification(&event).await?;

        if let Some(notification) = notification {
            // Check preferences
            if !self.should_send(&notification).await? {
                info!(
                    user_id = %notification.user_id,
                    "Notification skipped due to user preferences"
                );
                return Ok(());
            }

            self.notification_sender.send(notification).await?;
        }

        Ok(())
    }
}

impl NotificationEventHandler {
    async fn build_notification(
        &self,
        event: &EventEnvelope,
    ) -> Result<Option<Notification>, HandlerError> {
        match event.event_type.as_str() {
            "membership.activated.v1" => {
                let payload: MembershipActivated = serde_json::from_value(event.payload.clone())?;
                let user = self.user_repo.find_by_id(&payload.user_id).await?;

                Some(Notification {
                    id: NotificationId::new(),
                    user_id: payload.user_id,
                    notification_type: NotificationType::Transactional,
                    channel: NotificationChannel::Email,
                    template: TemplateId::from("welcome-paid"),
                    data: TemplateData::from([
                        ("user", json!(user)),
                        ("tier", json!(payload.tier)),
                        ("amount_paid", json!(payload.amount_cents / 100)),
                    ]),
                    priority: NotificationPriority::High,
                    created_at: Timestamp::now(),
                })
            }

            "cycle.completed.v1" => {
                let payload: CycleCompleted = serde_json::from_value(event.payload.clone())?;

                // In-app notification only
                Some(Notification {
                    id: NotificationId::new(),
                    user_id: payload.user_id,
                    notification_type: NotificationType::Engagement,
                    channel: NotificationChannel::InApp,
                    template: TemplateId::from("cycle-completed"),
                    data: TemplateData::from([
                        ("session_title", json!(payload.session_title)),
                        ("dq_score", json!(payload.dq_score)),
                    ]),
                    priority: NotificationPriority::Normal,
                    created_at: Timestamp::now(),
                })
            }

            // ... other event types

            _ => None,
        }
    }

    async fn should_send(&self, notification: &Notification) -> Result<bool, HandlerError> {
        // Transactional always sent
        if notification.notification_type == NotificationType::Transactional {
            return Ok(true);
        }

        let prefs = self.preferences.get(&notification.user_id).await?;

        match notification.channel {
            NotificationChannel::Email => Ok(prefs.email_enabled && prefs.engagement_emails),
            NotificationChannel::InApp => Ok(prefs.in_app_enabled),
            NotificationChannel::Push => Ok(false), // Not implemented
        }
    }
}
```

---

## Scheduled Notifications

### Scheduler

```rust
// backend/src/infrastructure/notification/scheduler.rs

pub struct NotificationScheduler {
    redis: ConnectionManager,
    notification_sender: Arc<dyn NotificationSender>,
}

impl NotificationScheduler {
    /// Schedule a notification for future delivery
    pub async fn schedule(
        &self,
        notification: Notification,
        send_at: Timestamp,
    ) -> Result<ScheduledNotificationId, SchedulerError> {
        let id = ScheduledNotificationId::new();

        let payload = serde_json::to_string(&notification)?;

        // Use Redis sorted set with score = unix timestamp
        redis::cmd("ZADD")
            .arg("notifications:scheduled")
            .arg(send_at.as_unix_secs())
            .arg(&id.to_string())
            .query_async(&mut self.redis.clone())
            .await?;

        redis::cmd("SET")
            .arg(format!("notification:{}", id))
            .arg(&payload)
            .query_async(&mut self.redis.clone())
            .await?;

        Ok(id)
    }

    /// Process due notifications (run periodically)
    pub async fn process_due(&self) -> Result<u32, SchedulerError> {
        let now = Timestamp::now().as_unix_secs();

        // Get all notifications due now
        let due_ids: Vec<String> = redis::cmd("ZRANGEBYSCORE")
            .arg("notifications:scheduled")
            .arg(0)
            .arg(now)
            .query_async(&mut self.redis.clone())
            .await?;

        let mut processed = 0;

        for id in due_ids {
            // Get notification data
            let payload: Option<String> = redis::cmd("GET")
                .arg(format!("notification:{}", id))
                .query_async(&mut self.redis.clone())
                .await?;

            if let Some(payload) = payload {
                let notification: Notification = serde_json::from_str(&payload)?;

                // Send it
                if let Err(e) = self.notification_sender.send(notification).await {
                    error!("Failed to send scheduled notification {}: {}", id, e);
                    continue;
                }

                // Remove from scheduled set
                redis::cmd("ZREM")
                    .arg("notifications:scheduled")
                    .arg(&id)
                    .query_async(&mut self.redis.clone())
                    .await?;

                redis::cmd("DEL")
                    .arg(format!("notification:{}", id))
                    .query_async(&mut self.redis.clone())
                    .await?;

                processed += 1;
            }
        }

        Ok(processed)
    }
}
```

### Engagement Triggers

```rust
// backend/src/application/jobs/engagement_notifications.rs

pub struct EngagementNotificationJob {
    session_repo: Arc<dyn SessionRepository>,
    notification_sender: Arc<dyn NotificationSender>,
}

impl EngagementNotificationJob {
    /// Run daily to check for inactive sessions
    pub async fn check_inactive_sessions(&self) -> Result<u32, JobError> {
        let seven_days_ago = Timestamp::now() - Duration::days(7);

        let inactive_sessions = self.session_repo
            .find_inactive_since(seven_days_ago)
            .await?;

        let mut sent = 0;

        for session in inactive_sessions {
            // Skip if already reminded recently
            if session.last_reminder_at.map(|t| t > seven_days_ago).unwrap_or(false) {
                continue;
            }

            let notification = Notification {
                id: NotificationId::new(),
                user_id: session.user_id.clone(),
                notification_type: NotificationType::Engagement,
                channel: NotificationChannel::Email,
                template: TemplateId::from("session-reminder"),
                data: TemplateData::from([
                    ("session_title", json!(session.title)),
                    ("last_active", json!(session.updated_at)),
                    ("resume_url", json!(format!("/sessions/{}", session.id))),
                ]),
                priority: NotificationPriority::Low,
                created_at: Timestamp::now(),
            };

            self.notification_sender.send(notification).await?;
            sent += 1;
        }

        Ok(sent)
    }
}
```

---

## API Endpoints

```rust
// backend/src/adapters/http/routes/notifications.rs

/// GET /api/notifications
/// Get user's notifications (paginated)
pub async fn list_notifications(
    Extension(user): Extension<AuthenticatedUser>,
    Query(params): Query<PaginationParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let notifications = state.notification_store
        .get_all(&user.id, params.page, params.per_page)
        .await?;

    let unread = state.notification_store.unread_count(&user.id).await?;

    Json(NotificationsResponse {
        notifications,
        unread_count: unread,
        page: params.page,
        per_page: params.per_page,
    })
}

/// POST /api/notifications/{id}/read
/// Mark notification as read
pub async fn mark_read(
    Path(notification_id): Path<NotificationId>,
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    state.notification_store.mark_read(&notification_id).await?;
    StatusCode::NO_CONTENT
}

/// POST /api/notifications/read-all
/// Mark all notifications as read
pub async fn mark_all_read(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    state.notification_store.mark_all_read(&user.id).await?;
    StatusCode::NO_CONTENT
}

/// GET /api/notifications/preferences
/// Get user's notification preferences
pub async fn get_preferences(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let preferences = state.notification_preferences.get(&user.id).await?;
    Json(preferences)
}

/// PUT /api/notifications/preferences
/// Update user's notification preferences
pub async fn update_preferences(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(preferences): Json<UserPreferences>,
) -> impl IntoResponse {
    state.notification_preferences.update(&user.id, preferences).await?;
    StatusCode::NO_CONTENT
}
```

---

## Events

```rust
// Notification-related domain events

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSent {
    pub notification_id: NotificationId,
    pub user_id: UserId,
    pub channel: String,
    pub template: String,
    pub occurred_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationFailed {
    pub notification_id: NotificationId,
    pub user_id: UserId,
    pub channel: String,
    pub error: String,
    pub occurred_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailBounced {
    pub user_id: UserId,
    pub email: String,
    pub bounce_type: String,
    pub occurred_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailUnsubscribed {
    pub user_id: UserId,
    pub email: String,
    pub list: String,
    pub occurred_at: Timestamp,
}
```

---

## Webhook Handlers

Handle delivery status from Resend:

```rust
// backend/src/adapters/http/routes/webhooks/resend.rs

/// POST /webhooks/resend
/// Handle Resend event webhooks
/// See: https://resend.com/docs/dashboard/webhooks/introduction
pub async fn resend_webhook(
    State(state): State<AppState>,
    Json(event): Json<ResendWebhookEvent>,
) -> impl IntoResponse {
    match event.event_type.as_str() {
        "email.bounced" => {
            if let Some(data) = event.data {
                state.event_publisher.publish(EmailBounced {
                    user_id: find_user_by_email(&data.to).await?,
                    email: data.to,
                    bounce_type: "hard".to_string(), // Resend doesn't distinguish
                    occurred_at: Timestamp::now(),
                }).await?;

                // Optionally disable email for user
            }
        }

        "email.complained" => {
            // User marked as spam
            if let Some(data) = event.data {
                let user_id = find_user_by_email(&data.to).await?;
                let mut prefs = state.notification_preferences.get(&user_id).await?;
                prefs.engagement_emails = false;
                state.notification_preferences.update(&user_id, prefs).await?;

                warn!(
                    email = %data.to,
                    "User marked email as spam, disabling engagement emails"
                );
            }
        }

        "email.delivery_delayed" => {
            if let Some(data) = event.data {
                warn!(
                    email = %data.to,
                    "Email delivery delayed"
                );
            }
        }

        _ => {}
    }

    StatusCode::OK
}

#[derive(Debug, Deserialize)]
struct ResendWebhookEvent {
    #[serde(rename = "type")]
    event_type: String,
    data: Option<ResendEmailData>,
}

#[derive(Debug, Deserialize)]
struct ResendEmailData {
    to: String,
    from: String,
    subject: Option<String>,
}
```

---

## Configuration

```rust
// backend/src/config/notification.rs

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationConfig {
    pub email: EmailProviderConfig,
    pub templates: TemplateConfig,
    pub scheduling: SchedulingConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "provider")]
pub enum EmailProviderConfig {
    Resend {
        api_key: String,
        from_address: String,
    },
    Console,  // For development - logs emails
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateConfig {
    /// Path to local templates
    pub template_dir: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SchedulingConfig {
    /// How often to check for due notifications (seconds)
    pub poll_interval_secs: u32,
    /// Maximum batch size for scheduled notifications
    pub batch_size: u32,
}
```

### Environment Variables

```bash
# .env (Production)
EMAIL_PROVIDER=resend
EMAIL_API_KEY=re_xxxxx
EMAIL_FROM_ADDRESS=noreply@choicesherpa.com

# Development
EMAIL_PROVIDER=console  # Just logs emails to stdout
```

### Cargo Dependencies

```toml
[dependencies]
resend-rs = "0.5"
tera = "1.19"  # Template rendering
```

---

## Testing

```rust
#[tokio::test]
async fn test_membership_activation_sends_welcome_email() {
    let mock_sender = MockNotificationSender::new();
    let handler = NotificationEventHandler::new(mock_sender.clone(), ...);

    let event = EventEnvelope::new(MembershipActivated {
        user_id: UserId::new("user-1"),
        tier: MembershipTier::Monthly,
        amount_cents: 999,
    });

    handler.handle(event).await.unwrap();

    let sent = mock_sender.sent_notifications();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].template.as_str(), "welcome-paid");
    assert_eq!(sent[0].channel, NotificationChannel::Email);
}

#[tokio::test]
async fn test_engagement_emails_respect_preferences() {
    let mock_sender = MockNotificationSender::new();
    let mock_prefs = MockPreferences::new();

    // User has engagement emails disabled
    mock_prefs.set(UserId::new("user-1"), UserPreferences {
        engagement_emails: false,
        ..Default::default()
    });

    let handler = NotificationEventHandler::new(mock_sender.clone(), mock_prefs, ...);

    let event = EventEnvelope::new(CycleCompleted { ... });

    handler.handle(event).await.unwrap();

    // Should not have sent
    assert!(mock_sender.sent_notifications().is_empty());
}
```

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Required for API endpoints; webhook uses signature verification |
| Authorization Model | Users can only manage their own notification preferences |
| Sensitive Data | Email addresses, notification content |
| Rate Limiting | Required: 10 emails/minute per user |
| Audit Logging | Notification sent/failed events |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| `user.email` | PII | Do not log; encrypt at rest |
| `notification.content` | Confidential | Do not include in logs |
| `resend_api_key` | Secret | Load from secure vault/env |
| `template_data` | Confidential | May contain user data |
| `notification_id` | Internal | Safe to log |

### Email Security Controls

**CRITICAL: Emails MUST NOT contain sensitive decision content**

| Email Type | Content Restrictions |
|------------|---------------------|
| `welcome-paid` | Tier name, amount (no session data) |
| `password-reset` | Reset URL only (expires 1 hour) |
| `export-ready` | Download URL only (expires 24 hours) |
| `session-reminder` | Session title only (no alternatives/objectives) |
| `payment-failed` | Last 4 digits of card only |
| `weekly-digest` | Counts only (e.g., "3 sessions updated") |

### URL Expiration Requirements

| URL Type | Expiration | Implementation |
|----------|------------|----------------|
| Password reset | 1 hour | Signed JWT with exp claim |
| Export download | 24 hours | Signed URL with expiry |
| Unsubscribe | No expiry | HMAC-signed user ID |

### Additional Security Controls

- **No PII in Email Subjects**: Subjects must be generic (e.g., "Your export is ready" not "Career Decision export ready")
- **No Decision Content in Emails**: Never include alternatives, objectives, or analysis results in email body
- **Bounce Handling**: Disable email delivery after hard bounce
- **Spam Complaint Handling**: Immediately disable engagement emails on complaint
- **Rate Limiting**: Maximum 10 transactional emails per user per minute
- **Unsubscribe Links**: All engagement emails must include one-click unsubscribe

---

## Implementation Phases

### Phase 1: Core Infrastructure

- [ ] Define notification ports
- [ ] Create domain types
- [ ] Implement console email sender (for dev)
- [ ] Write unit tests

### Phase 2: Resend Integration

- [ ] Implement Resend adapter
- [ ] Set up local templates with Tera
- [ ] Webhook handler for bounces/complaints
- [ ] Integration tests

### Phase 3: Event-Driven Notifications

- [ ] NotificationEventHandler
- [ ] Wire up to event bus
- [ ] Handle MembershipActivated
- [ ] Handle ExportCompleted
- [ ] Test event → notification flow

### Phase 4: In-App Notifications

- [ ] Notification storage (Postgres)
- [ ] WebSocket push
- [ ] Frontend NotificationBell component
- [ ] API endpoints

### Phase 5: Preferences & Scheduling

- [ ] User preferences storage
- [ ] Preferences API
- [ ] Notification scheduler
- [ ] Engagement notification jobs

### Phase 6: Frontend Integration

- [ ] Notification preferences page
- [ ] Email preference center (unsubscribe links)
- [ ] Toast notifications
- [ ] Notification history

---

## Exit Criteria

1. **Transactional emails work**: Membership activation triggers welcome email
2. **In-app notifications display**: WebSocket pushes show in frontend
3. **Preferences respected**: Users can opt out of engagement emails
4. **Bounces handled**: Invalid emails automatically disabled
5. **Templates professional**: All templates match brand, mobile-responsive
6. **Scheduling works**: Delayed notifications send on time
