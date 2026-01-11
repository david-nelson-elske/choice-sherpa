//! WebSocket adapters for real-time dashboard updates.
//!
//! This module provides the infrastructure for pushing domain events
//! to connected frontend clients via WebSocket connections.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                         Event Bus                                    │
//! │   InMemoryEventBus (test) │ RedisEventBus (production)              │
//! └─────────────────────────────────────────────────────────────────────┘
//!                                     │
//!                                     │ subscribes
//!                                     ▼
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                    WebSocketEventBridge                              │
//! │   - Subscribes to dashboard-relevant events                         │
//! │   - Transforms EventEnvelope → DashboardUpdate                      │
//! │   - Routes to appropriate session rooms                             │
//! └─────────────────────────────────────────────────────────────────────┘
//!                                     │
//!                                     │ broadcasts
//!                                     ▼
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                      RoomManager                                     │
//! │   Room: session-123    Room: session-456    Room: session-789       │
//! │   ├── client-a         ├── client-d         ├── client-g            │
//! │   ├── client-b         └── client-e         └── client-h            │
//! │   └── client-c                                                       │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Components
//!
//! - [`messages`] - WebSocket message protocol types
//! - [`rooms`] - Room management for session-based routing
//! - [`handler`] - Axum WebSocket upgrade handler
//! - [`event_bridge`] - Bridge between event bus and WebSocket rooms

pub mod event_bridge;
pub mod handler;
pub mod messages;
pub mod rooms;

pub use event_bridge::{WebSocketEventBridge, DASHBOARD_EVENT_TYPES};
pub use handler::{websocket_router, ws_handler, WebSocketState};
pub use messages::{
    ClientMessage, ConnectedMessage, DashboardUpdate, DashboardUpdateMessage,
    DashboardUpdateType, ErrorMessage, PongMessage, ServerMessage,
};
pub use rooms::{ClientId, RoomManager};
