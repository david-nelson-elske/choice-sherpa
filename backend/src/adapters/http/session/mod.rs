//! HTTP adapter for session endpoints.

mod dto;
mod handlers;
mod routes;

pub use dto::{
    CreateSessionRequest, ErrorResponse, ListSessionsQuery, RenameSessionRequest,
    SessionCommandResponse, SessionListResponse, SessionResponse, SessionSummaryResponse,
};
pub use handlers::SessionHandlers;
pub use routes::session_routes;
