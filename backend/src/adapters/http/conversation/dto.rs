//! HTTP DTOs for conversation endpoints.
//!
//! These types decouple the HTTP API from domain types, allowing independent evolution.

use serde::{Deserialize, Serialize};

use crate::domain::conversation::{AgentPhase, ConversationState};
use crate::domain::foundation::ComponentType;

// ════════════════════════════════════════════════════════════════════════════════
// Response DTOs
// ════════════════════════════════════════════════════════════════════════════════

/// View of a conversation for API responses.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationView {
    /// Conversation ID.
    pub id: String,
    /// Component ID this conversation belongs to.
    pub component_id: String,
    /// Component type for this conversation.
    pub component_type: ComponentType,
    /// Current state of the conversation.
    pub state: ConversationState,
    /// Current agent phase.
    pub phase: AgentPhase,
    /// Total message count.
    pub message_count: u32,
    /// When the conversation was created.
    pub created_at: String,
    /// When the conversation was last updated.
    pub updated_at: String,
}

/// View of a message for API responses.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageView {
    /// Message ID.
    pub id: String,
    /// Role of the message sender.
    pub role: MessageRoleDto,
    /// Content of the message.
    pub content: String,
    /// When the message was sent.
    pub timestamp: String,
    /// Token usage for this message (if assistant message).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<TokenUsageDto>,
}

/// Role of a message sender.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRoleDto {
    User,
    Assistant,
    System,
}

/// Token usage statistics.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageDto {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub estimated_cost_cents: u32,
}

/// Paginated response wrapper.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    /// The items in this page.
    pub items: Vec<T>,
    /// Total count of all items.
    pub total: u32,
    /// Offset used for this page.
    pub offset: u32,
    /// Limit used for this page.
    pub limit: u32,
    /// Whether there are more items after this page.
    pub has_more: bool,
}

impl<T> Page<T> {
    /// Create a new page from items.
    pub fn new(items: Vec<T>, total: u32, offset: u32, limit: u32) -> Self {
        let has_more = (offset + items.len() as u32) < total;
        Self {
            items,
            total,
            offset,
            limit,
            has_more,
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Request DTOs
// ════════════════════════════════════════════════════════════════════════════════

/// Query parameters for paginated message retrieval.
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    /// Number of items to skip.
    #[serde(default)]
    pub offset: Option<u32>,
    /// Maximum number of items to return.
    #[serde(default)]
    pub limit: Option<u32>,
}

impl PaginationParams {
    /// Default limit for messages.
    pub const DEFAULT_LIMIT: u32 = 50;
    /// Maximum allowed limit.
    pub const MAX_LIMIT: u32 = 100;

    /// Get the effective offset.
    pub fn effective_offset(&self) -> u32 {
        self.offset.unwrap_or(0)
    }

    /// Get the effective limit, capped at MAX_LIMIT.
    pub fn effective_limit(&self) -> u32 {
        self.limit
            .unwrap_or(Self::DEFAULT_LIMIT)
            .min(Self::MAX_LIMIT)
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// Error Response
// ════════════════════════════════════════════════════════════════════════════════

/// Standard error response.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            code: "BAD_REQUEST".to_string(),
            message: message.into(),
            details: None,
        }
    }

    pub fn not_found(resource_type: &str, id: &str) -> Self {
        Self {
            code: "NOT_FOUND".to_string(),
            message: format!("{} not found: {}", resource_type, id),
            details: None,
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            code: "FORBIDDEN".to_string(),
            message: message.into(),
            details: None,
        }
    }

    pub fn rate_limited(message: impl Into<String>) -> Self {
        Self {
            code: "RATE_LIMITED".to_string(),
            message: message.into(),
            details: None,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            code: "INTERNAL_ERROR".to_string(),
            message: message.into(),
            details: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod pagination_params {
        use super::*;

        #[test]
        fn default_limit_is_50() {
            let params = PaginationParams {
                offset: None,
                limit: None,
            };
            assert_eq!(params.effective_limit(), 50);
        }

        #[test]
        fn default_offset_is_0() {
            let params = PaginationParams {
                offset: None,
                limit: None,
            };
            assert_eq!(params.effective_offset(), 0);
        }

        #[test]
        fn limit_capped_at_max() {
            let params = PaginationParams {
                offset: None,
                limit: Some(500),
            };
            assert_eq!(params.effective_limit(), 100);
        }

        #[test]
        fn respects_provided_limit_under_max() {
            let params = PaginationParams {
                offset: None,
                limit: Some(25),
            };
            assert_eq!(params.effective_limit(), 25);
        }

        #[test]
        fn respects_provided_offset() {
            let params = PaginationParams {
                offset: Some(10),
                limit: None,
            };
            assert_eq!(params.effective_offset(), 10);
        }
    }

    mod page {
        use super::*;

        #[test]
        fn has_more_true_when_more_items_exist() {
            let page: Page<u32> = Page::new(vec![1, 2, 3], 10, 0, 3);
            assert!(page.has_more);
        }

        #[test]
        fn has_more_false_when_at_end() {
            let page: Page<u32> = Page::new(vec![8, 9, 10], 10, 7, 3);
            assert!(!page.has_more);
        }

        #[test]
        fn has_more_false_when_exact_end() {
            let page: Page<u32> = Page::new(vec![1, 2, 3], 3, 0, 3);
            assert!(!page.has_more);
        }
    }

    mod conversation_view {
        use super::*;

        #[test]
        fn serializes_to_camel_case() {
            let view = ConversationView {
                id: "conv-123".to_string(),
                component_id: "comp-456".to_string(),
                component_type: ComponentType::IssueRaising,
                state: ConversationState::InProgress,
                phase: AgentPhase::Gather,
                message_count: 5,
                created_at: "2026-01-10T00:00:00Z".to_string(),
                updated_at: "2026-01-10T01:00:00Z".to_string(),
            };

            let json = serde_json::to_string(&view).unwrap();
            assert!(json.contains("componentId"));
            assert!(json.contains("componentType"));
            assert!(json.contains("messageCount"));
            assert!(json.contains("createdAt"));
            assert!(json.contains("updatedAt"));
        }
    }

    mod message_view {
        use super::*;

        #[test]
        fn serializes_without_token_usage_when_none() {
            let view = MessageView {
                id: "msg-123".to_string(),
                role: MessageRoleDto::User,
                content: "Hello".to_string(),
                timestamp: "2026-01-10T00:00:00Z".to_string(),
                token_usage: None,
            };

            let json = serde_json::to_string(&view).unwrap();
            assert!(!json.contains("tokenUsage"));
        }

        #[test]
        fn serializes_with_token_usage_when_present() {
            let view = MessageView {
                id: "msg-123".to_string(),
                role: MessageRoleDto::Assistant,
                content: "Hello there".to_string(),
                timestamp: "2026-01-10T00:00:00Z".to_string(),
                token_usage: Some(TokenUsageDto {
                    prompt_tokens: 10,
                    completion_tokens: 5,
                    total_tokens: 15,
                    estimated_cost_cents: 1,
                }),
            };

            let json = serde_json::to_string(&view).unwrap();
            assert!(json.contains("tokenUsage"));
            assert!(json.contains("promptTokens"));
        }
    }

    mod error_response {
        use super::*;

        #[test]
        fn rate_limited_creates_correct_code() {
            let error = ErrorResponse::rate_limited("Too many requests");
            assert_eq!(error.code, "RATE_LIMITED");
            assert_eq!(error.message, "Too many requests");
        }
    }
}
