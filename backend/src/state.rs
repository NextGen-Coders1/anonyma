use authkestra::axum::AuthkestraAxumError;
use authkestra::flow::{Authkestra, Configured, Missing, SessionStoreState};
use authkestra::session::{SessionConfig, SessionStore};
use axum::extract::FromRef;
use sqlx::PgPool;
use std::sync::Arc;

/// Concrete Authkestra type: session store configured, no token manager.
pub type AuthkestraInstance = Authkestra<Configured<Arc<dyn SessionStore>>, Missing>;

/// Application state with a concrete Authkestra type.
#[derive(Clone)]
pub struct AppState {
    pub authkestra: AuthkestraInstance,
    pub db_pool: Arc<PgPool>,
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
