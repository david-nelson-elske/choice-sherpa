//! AI Engine Command and Query Handlers
//!
//! CQRS handlers for AI-powered conversation management.
//!
//! ## Commands
//! - `StartConversation` - Initialize a new AI conversation for a cycle
//! - `SendMessage` - Send a user message and get AI response
//! - `RouteIntent` - Determine target component from user intent
//! - `EndConversation` - Terminate an active conversation
//!
//! ## Queries
//! - `GetConversationState` - Retrieve current conversation state

mod end_conversation;
mod get_conversation_state;
mod route_intent;
mod send_message;
mod start_conversation;

pub use end_conversation::{EndConversationCommand, EndConversationError, EndConversationHandler};
pub use get_conversation_state::{
    GetConversationStateHandler, GetConversationStateQuery, GetConversationStateResult,
};
pub use route_intent::{
    RouteIntentCommand, RouteIntentError, RouteIntentHandler, RouteIntentResult,
};
pub use send_message::{SendMessageCommand, SendMessageError, SendMessageHandler, SendMessageResult};
pub use start_conversation::{
    StartConversationCommand, StartConversationError, StartConversationHandler,
    StartConversationResult,
};
