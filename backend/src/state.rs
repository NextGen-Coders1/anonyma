use authkestra::axum::AuthkestraAxumError;
use authkestra::flow::{Authkestra, Configured, Missing, SessionStoreState};
use authkestra::session::{SessionConfig, SessionStore};
use axum::extract::FromRef;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;

/// Concrete Authkestra type: session store configured, no token manager.
pub type AuthkestraInstance = Authkestra<Configured<Arc<dyn SessionStore>>, Missing>;

/// SSE event payload sent to connected clients.
#[derive(Debug, Clone)]
pub struct SseEvent {
    /// Event type: "new_message" or "new_broadcast"
    pub event_type: String,
    /// JSON payload string
    pub data: String,
}

/// Per-user notification hub. Maps user UUID → broadcast sender.
/// Each connected user has a channel; when they connect a receiver is created.
pub type NotificationHub = Arc<Mutex<HashMap<Uuid, broadcast::Sender<SseEvent>>>>;

/// Application state with a concrete Authkestra type.
#[derive(Clone)]
pub struct AppState {
    pub authkestra: AuthkestraInstance,
    pub db_pool: Arc<PgPool>,
    /// SSE notification hub for real-time push
    pub notification_hub: NotificationHub,
}

// Implement FromRef for Authkestra (required for axum_router and AuthSession)
impl FromRef<AppState> for AuthkestraInstance {
    fn from_ref(state: &AppState) -> Self {
        state.authkestra.clone()
    }
}

// Implement FromRef for database pool extraction
impl FromRef<AppState> for Arc<PgPool> {
    fn from_ref(state: &AppState) -> Self {
        state.db_pool.clone()
    }
}

// Implement FromRef for SessionStore (required for AuthSession extractor)
impl FromRef<AppState> for Result<Arc<dyn SessionStore>, AuthkestraAxumError> {
    fn from_ref(state: &AppState) -> Self {
        Ok(state.authkestra.session_store.get_store())
    }
}

// Implement FromRef for SessionConfig (required for AuthSession)
impl FromRef<AppState> for SessionConfig {
    fn from_ref(state: &AppState) -> Self {
        state.authkestra.session_config.clone()
    }
}

// Implement FromRef for the notification hub
impl FromRef<AppState> for NotificationHub {
    fn from_ref(state: &AppState) -> Self {
        state.notification_hub.clone()
    }
}
