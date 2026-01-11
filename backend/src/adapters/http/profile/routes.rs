//! HTTP routes for profile endpoints.

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use super::handlers::{
    create_profile, delete_profile, get_agent_instructions, get_profile_summary, record_outcome,
    update_consent, update_from_decision, ProfileHandlers,
};

/// Creates the profile router with all endpoints.
pub fn profile_routes(handlers: ProfileHandlers) -> Router {
    Router::new()
        .route("/", post(create_profile))
        .route("/", get(get_profile_summary))
        .route("/", delete(delete_profile))
        .route("/instructions", get(get_agent_instructions))
        .route("/consent", put(update_consent))
        .route("/outcome", post(record_outcome))
        .route("/update-from-decision", post(update_from_decision))
        .with_state(handlers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_routes_compiles() {
        // This test just ensures the route definitions compile correctly
        // Actual HTTP testing would require integration tests
    }
}
