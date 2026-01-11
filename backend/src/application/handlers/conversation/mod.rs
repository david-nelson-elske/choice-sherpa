//! Conversation command and query handlers.
//!
//! Handles sending messages and regenerating AI responses in conversations.

mod get_conversation;
mod regenerate_response;
mod send_message;

pub use send_message::{
    // Command
    SendMessageCommand,
    SendMessageError,
    SendMessageHandler,
    SendMessageResult,
    // Types
    MessageId,
    MessageRole,
    StoredMessage,
    StreamEvent,
    // Ports
    ComponentOwnershipChecker,
    ConversationRepository,
    ConversationRecord,
    OwnershipInfo,
};

pub use regenerate_response::{
    // Command
    RegenerateResponseCommand,
    RegenerateResponseError,
    RegenerateResponseHandler,
    RegenerateResponseResult,
    // Extended port
    ConversationRepositoryExt,
};

pub use get_conversation::{GetConversationHandler, GetConversationQuery};
