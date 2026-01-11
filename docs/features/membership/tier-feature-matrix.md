# Feature: Tier Feature Matrix

**Module:** membership
**Type:** Business Rules
**Priority:** P0
**Status:** Specification Complete

> Complete feature matrix defining what each membership tier provides, including limits, capabilities, and pricing.

---

## Overview

Choice Sherpa offers three membership tiers:
1. **Free** - Workshop/beta users with promo code
2. **Premium** (Monthly) - Monthly subscription
3. **Pro** (Annual) - Annual subscription with discount

---

## Pricing

| Tier | Price (CAD) | Billing | Monthly Equivalent | Savings |
|------|-------------|---------|-------------------|---------|
| Free | $0.00 | N/A | $0.00 | N/A |
| Premium | $19.99/mo | Monthly | $19.99 | N/A |
| Pro | $149.99/yr | Annual | $12.50 | 38% |

### Stripe Price IDs

```rust
pub struct StripePriceConfig {
    /// Monthly subscription price ID
    pub premium_monthly: String,  // price_1XXX_monthly_cad
    /// Annual subscription price ID
    pub pro_annual: String,       // price_1XXX_annual_cad
}

impl Default for StripePriceConfig {
    fn default() -> Self {
        Self {
            premium_monthly: "price_premium_monthly_cad".to_string(),
            pro_annual: "price_pro_annual_cad".to_string(),
        }
    }
}
```

---

## Feature Matrix

### Session & Cycle Limits

| Feature | Free | Premium | Pro |
|---------|------|---------|-----|
| Active Sessions | 3 | 10 | Unlimited |
| Cycles per Session | 2 | 5 | Unlimited |
| Archived Sessions | 10 | 50 | Unlimited |
| Session History | 90 days | 1 year | Forever |

### PrOACT Components

| Feature | Free | Premium | Pro |
|---------|------|---------|-----|
| Issue Raising | ✓ | ✓ | ✓ |
| Problem Frame | ✓ | ✓ | ✓ |
| Objectives | ✓ | ✓ | ✓ |
| Alternatives | ✓ | ✓ | ✓ |
| Consequences | ✓ | ✓ | ✓ |
| Tradeoffs | ✓ | ✓ | ✓ |
| Recommendation | ✓ | ✓ | ✓ |
| Decision Quality | ✗ | ✓ | ✓ |

### AI Features

| Feature | Free | Premium | Pro |
|---------|------|---------|-----|
| AI Conversation | ✓ | ✓ | ✓ |
| AI Messages per Day | 50 | 200 | Unlimited |
| AI Model | Standard | Standard | Advanced |
| Data Extraction | ✓ | ✓ | ✓ |
| Smart Suggestions | Basic | Full | Full |
| Conversation Export | ✗ | ✓ | ✓ |

### Analysis & Visualization

| Feature | Free | Premium | Pro |
|---------|------|---------|-----|
| Pugh Matrix | ✓ | ✓ | ✓ |
| Pugh Scores | ✓ | ✓ | ✓ |
| Dominance Detection | ✓ | ✓ | ✓ |
| Tradeoff Analysis | Basic | Full | Full |
| DQ Scoring | ✗ | ✓ | ✓ |
| DQ Element Details | ✗ | ✓ | ✓ |
| Improvement Suggestions | ✗ | ✓ | ✓ |

### Export & Sharing

| Feature | Free | Premium | Pro |
|---------|------|---------|-----|
| PDF Export | ✗ | ✓ | ✓ |
| Share Read-Only Link | ✗ | ✓ | ✓ |
| Collaborative Sessions | ✗ | ✗ | Future |
| API Access | ✗ | ✗ | Future |

### Support

| Feature | Free | Premium | Pro |
|---------|------|---------|-----|
| Email Support | ✗ | ✓ | ✓ |
| Response Time | N/A | 48h | 24h |
| Priority Support | ✗ | ✗ | ✓ |

---

## Implementation

### TierLimits Struct

```rust
/// Complete feature limits for a membership tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierLimits {
    // Session & Cycle Limits
    pub max_active_sessions: Option<u32>,      // None = unlimited
    pub max_cycles_per_session: Option<u32>,   // None = unlimited
    pub max_archived_sessions: Option<u32>,    // None = unlimited
    pub session_history_days: Option<u32>,     // None = forever

    // AI Limits
    pub ai_enabled: bool,
    pub ai_messages_per_day: Option<u32>,      // None = unlimited
    pub ai_model_tier: AiModelTier,

    // Component Access
    pub dq_component_enabled: bool,

    // Analysis Features
    pub full_tradeoff_analysis: bool,
    pub dq_scoring_enabled: bool,
    pub improvement_suggestions_enabled: bool,

    // Export & Sharing
    pub pdf_export_enabled: bool,
    pub share_link_enabled: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AiModelTier {
    Standard,   // GPT-4o-mini or equivalent
    Advanced,   // GPT-4o or equivalent
}

impl TierLimits {
    pub fn for_tier(tier: MembershipTier) -> Self {
        match tier {
            MembershipTier::Free => Self::free(),
            MembershipTier::Monthly => Self::premium(),
            MembershipTier::Annual => Self::pro(),
        }
    }

    pub fn free() -> Self {
        Self {
            max_active_sessions: Some(3),
            max_cycles_per_session: Some(2),
            max_archived_sessions: Some(10),
            session_history_days: Some(90),

            ai_enabled: true,
            ai_messages_per_day: Some(50),
            ai_model_tier: AiModelTier::Standard,

            dq_component_enabled: false,

            full_tradeoff_analysis: false,
            dq_scoring_enabled: false,
            improvement_suggestions_enabled: false,

            pdf_export_enabled: false,
            share_link_enabled: false,
        }
    }

    pub fn premium() -> Self {
        Self {
            max_active_sessions: Some(10),
            max_cycles_per_session: Some(5),
            max_archived_sessions: Some(50),
            session_history_days: Some(365),

            ai_enabled: true,
            ai_messages_per_day: Some(200),
            ai_model_tier: AiModelTier::Standard,

            dq_component_enabled: true,

            full_tradeoff_analysis: true,
            dq_scoring_enabled: true,
            improvement_suggestions_enabled: true,

            pdf_export_enabled: true,
            share_link_enabled: true,
        }
    }

    pub fn pro() -> Self {
        Self {
            max_active_sessions: None, // Unlimited
            max_cycles_per_session: None,
            max_archived_sessions: None,
            session_history_days: None, // Forever

            ai_enabled: true,
            ai_messages_per_day: None, // Unlimited
            ai_model_tier: AiModelTier::Advanced,

            dq_component_enabled: true,

            full_tradeoff_analysis: true,
            dq_scoring_enabled: true,
            improvement_suggestions_enabled: true,

            pdf_export_enabled: true,
            share_link_enabled: true,
        }
    }

    /// Returns limits for users without membership
    pub fn no_membership() -> Self {
        Self {
            max_active_sessions: Some(0),
            max_cycles_per_session: Some(0),
            max_archived_sessions: Some(0),
            session_history_days: Some(0),

            ai_enabled: false,
            ai_messages_per_day: Some(0),
            ai_model_tier: AiModelTier::Standard,

            dq_component_enabled: false,

            full_tradeoff_analysis: false,
            dq_scoring_enabled: false,
            improvement_suggestions_enabled: false,

            pdf_export_enabled: false,
            share_link_enabled: false,
        }
    }
}
```

### Limit Checking Functions

```rust
impl TierLimits {
    /// Checks if user can create a new session
    pub fn can_create_session(&self, current_active: u32) -> bool {
        match self.max_active_sessions {
            None => true,
            Some(max) => current_active < max,
        }
    }

    /// Checks if user can create a new cycle in session
    pub fn can_create_cycle(&self, current_cycles: u32) -> bool {
        match self.max_cycles_per_session {
            None => true,
            Some(max) => current_cycles < max,
        }
    }

    /// Checks if user can send AI message
    pub fn can_send_ai_message(&self, messages_today: u32) -> bool {
        if !self.ai_enabled {
            return false;
        }
        match self.ai_messages_per_day {
            None => true,
            Some(max) => messages_today < max,
        }
    }

    /// Checks if user can access DQ component
    pub fn can_access_dq(&self) -> bool {
        self.dq_component_enabled
    }

    /// Checks if user can export to PDF
    pub fn can_export_pdf(&self) -> bool {
        self.pdf_export_enabled
    }

    /// Gets the AI model to use for this tier
    pub fn ai_model(&self) -> &str {
        match self.ai_model_tier {
            AiModelTier::Standard => "gpt-4o-mini",
            AiModelTier::Advanced => "gpt-4o",
        }
    }
}
```

---

## AccessChecker Integration

The `AccessChecker` port exposes tier limits to other modules:

```rust
#[async_trait]
pub trait AccessChecker: Send + Sync {
    /// Checks if user has any access (valid membership)
    async fn has_access(&self, user_id: &UserId) -> Result<bool, AccessError>;

    /// Checks if user can create a new session
    async fn can_create_session(&self, user_id: &UserId) -> Result<bool, AccessError>;

    /// Checks if user can create cycle in session
    async fn can_create_cycle(
        &self,
        user_id: &UserId,
        session_id: &SessionId,
    ) -> Result<bool, AccessError>;

    /// Checks if user can send AI message
    async fn can_send_ai_message(&self, user_id: &UserId) -> Result<bool, AccessError>;

    /// Checks if user can access specific component
    async fn can_access_component(
        &self,
        user_id: &UserId,
        component: ComponentType,
    ) -> Result<bool, AccessError>;

    /// Gets the user's tier (None if no membership)
    async fn get_tier(&self, user_id: &UserId) -> Result<Option<MembershipTier>, AccessError>;

    /// Gets full feature limits for user
    async fn get_limits(&self, user_id: &UserId) -> Result<TierLimits, AccessError>;

    /// Gets remaining AI messages for today
    async fn get_ai_messages_remaining(&self, user_id: &UserId) -> Result<Option<u32>, AccessError>;
}
```

### Implementation

```rust
pub struct AccessCheckerImpl {
    membership_reader: Arc<dyn MembershipReader>,
    session_reader: Arc<dyn SessionReader>,
    message_counter: Arc<dyn MessageCounter>,
}

#[async_trait]
impl AccessChecker for AccessCheckerImpl {
    async fn can_create_session(&self, user_id: &UserId) -> Result<bool, AccessError> {
        // Get user's membership
        let membership = self.membership_reader.get_by_user(user_id).await?;

        let limits = match membership {
            Some(m) if m.has_access => TierLimits::for_tier(m.tier),
            _ => return Ok(false), // No membership = no access
        };

        // Count current active sessions
        let current_count = self.session_reader
            .count_active_by_user(user_id)
            .await?;

        Ok(limits.can_create_session(current_count))
    }

    async fn can_send_ai_message(&self, user_id: &UserId) -> Result<bool, AccessError> {
        let membership = self.membership_reader.get_by_user(user_id).await?;

        let limits = match membership {
            Some(m) if m.has_access => TierLimits::for_tier(m.tier),
            _ => return Ok(false),
        };

        // Count messages sent today
        let messages_today = self.message_counter
            .count_today(user_id)
            .await?;

        Ok(limits.can_send_ai_message(messages_today))
    }

    async fn can_access_component(
        &self,
        user_id: &UserId,
        component: ComponentType,
    ) -> Result<bool, AccessError> {
        let membership = self.membership_reader.get_by_user(user_id).await?;

        let limits = match membership {
            Some(m) if m.has_access => TierLimits::for_tier(m.tier),
            _ => return Ok(false),
        };

        // Check if component is gated
        match component {
            ComponentType::DecisionQuality => Ok(limits.can_access_dq()),
            _ => Ok(true), // All other components available to all tiers
        }
    }

    async fn get_ai_messages_remaining(&self, user_id: &UserId) -> Result<Option<u32>, AccessError> {
        let membership = self.membership_reader.get_by_user(user_id).await?;

        let limits = match membership {
            Some(m) if m.has_access => TierLimits::for_tier(m.tier),
            _ => return Ok(Some(0)),
        };

        match limits.ai_messages_per_day {
            None => Ok(None), // Unlimited
            Some(max) => {
                let sent = self.message_counter.count_today(user_id).await?;
                Ok(Some(max.saturating_sub(sent)))
            }
        }
    }
}
```

---

## Frontend Usage

### TypeScript Types

```typescript
// frontend/src/modules/membership/domain/tier-limits.ts

export interface TierLimits {
  maxActiveSessions: number | null;     // null = unlimited
  maxCyclesPerSession: number | null;
  maxArchivedSessions: number | null;
  sessionHistoryDays: number | null;

  aiEnabled: boolean;
  aiMessagesPerDay: number | null;
  aiModelTier: 'standard' | 'advanced';

  dqComponentEnabled: boolean;

  fullTradeoffAnalysis: boolean;
  dqScoringEnabled: boolean;
  improvementSuggestionsEnabled: boolean;

  pdfExportEnabled: boolean;
  shareLinkEnabled: boolean;
}

export const TIER_LIMITS: Record<MembershipTier, TierLimits> = {
  free: {
    maxActiveSessions: 3,
    maxCyclesPerSession: 2,
    maxArchivedSessions: 10,
    sessionHistoryDays: 90,
    aiEnabled: true,
    aiMessagesPerDay: 50,
    aiModelTier: 'standard',
    dqComponentEnabled: false,
    fullTradeoffAnalysis: false,
    dqScoringEnabled: false,
    improvementSuggestionsEnabled: false,
    pdfExportEnabled: false,
    shareLinkEnabled: false,
  },
  monthly: {
    maxActiveSessions: 10,
    maxCyclesPerSession: 5,
    maxArchivedSessions: 50,
    sessionHistoryDays: 365,
    aiEnabled: true,
    aiMessagesPerDay: 200,
    aiModelTier: 'standard',
    dqComponentEnabled: true,
    fullTradeoffAnalysis: true,
    dqScoringEnabled: true,
    improvementSuggestionsEnabled: true,
    pdfExportEnabled: true,
    shareLinkEnabled: true,
  },
  annual: {
    maxActiveSessions: null,
    maxCyclesPerSession: null,
    maxArchivedSessions: null,
    sessionHistoryDays: null,
    aiEnabled: true,
    aiMessagesPerDay: null,
    aiModelTier: 'advanced',
    dqComponentEnabled: true,
    fullTradeoffAnalysis: true,
    dqScoringEnabled: true,
    improvementSuggestionsEnabled: true,
    pdfExportEnabled: true,
    shareLinkEnabled: true,
  },
};
```

### Upgrade Prompts

```typescript
// frontend/src/modules/membership/components/UpgradePrompt.svelte

export interface UpgradePromptConfig {
  feature: string;
  requiredTier: MembershipTier;
  message: string;
}

export const UPGRADE_PROMPTS: Record<string, UpgradePromptConfig> = {
  dq_component: {
    feature: 'Decision Quality',
    requiredTier: 'monthly',
    message: 'Upgrade to Premium to rate your decision quality and get improvement suggestions.',
  },
  pdf_export: {
    feature: 'PDF Export',
    requiredTier: 'monthly',
    message: 'Upgrade to Premium to export your decision analysis as a PDF.',
  },
  session_limit: {
    feature: 'More Sessions',
    requiredTier: 'monthly',
    message: 'You\'ve reached your session limit. Upgrade to Premium for 10 active sessions.',
  },
  unlimited_sessions: {
    feature: 'Unlimited Sessions',
    requiredTier: 'annual',
    message: 'Upgrade to Pro for unlimited sessions and advanced AI.',
  },
  ai_limit: {
    feature: 'More AI Messages',
    requiredTier: 'monthly',
    message: 'You\'ve reached your daily AI message limit. Upgrade for more.',
  },
};
```

---

## Security Requirements

| Requirement | Value |
|-------------|-------|
| Authentication | Required |
| Authorization Model | AccessChecker enforces tier-based permissions |
| Sensitive Data | Tier limits, usage counts |
| Rate Limiting | Not Required (enforced by API layer) |
| Audit Logging | Access denied events for limit enforcement |

### Data Classification

| Field/Entity | Classification | Handling Requirements |
|--------------|----------------|----------------------|
| `tier` | Public | Can be displayed in UI |
| `max_active_sessions` | Internal | Part of limits config |
| `ai_messages_per_day` | Internal | Usage counters not sensitive |
| `stripe_price_id` | Confidential | Do not expose to frontend |

### Security Controls

- **Fail-Secure Pattern**: AccessChecker MUST deny access on any error condition:
  ```rust
  // CORRECT: Fail-secure
  match self.membership_reader.get_by_user(user_id).await {
      Ok(Some(m)) if m.has_access => TierLimits::for_tier(m.tier),
      _ => return Ok(false), // Deny on error or missing
  }
  ```
- **No Free Tier Default**: Users without a valid membership get `TierLimits::no_membership()` which denies all access. There is no implicit free tier.
- **Server-Side Enforcement**: All tier checks must be enforced server-side; frontend checks are UX only
- **Cache Invalidation**: Access check cache must be invalidated immediately on membership state change

---

## Acceptance Criteria

### AC1: Free Tier Limits Enforced
**Given** user has Free tier membership
**When** user tries to create 4th active session
**Then** creation is blocked with upgrade prompt

### AC2: DQ Component Gated
**Given** user has Free tier membership
**When** user navigates to Decision Quality component
**Then** component shows upgrade prompt, not accessible

### AC3: AI Message Limits
**Given** user has Free tier with 50/day limit
**When** user sends 51st message in a day
**Then** message is blocked with limit reached message

### AC4: Unlimited for Pro
**Given** user has Pro (Annual) tier
**When** user creates any number of sessions
**Then** all creations succeed (no limit)

### AC5: Export Gated
**Given** user has Free tier
**When** user clicks "Export PDF"
**Then** export is blocked with upgrade prompt

### AC6: Limits API Response
**Given** user is authenticated
**When** frontend requests /api/membership/limits
**Then** response includes all limits for user's tier

---

*Version: 1.0.0*
*Created: 2026-01-08*
*Module: membership*
