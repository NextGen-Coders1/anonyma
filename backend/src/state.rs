use authkestra::axum::AuthkestraAxumError;
use authkestra::flow::{Authkestra, Missing, SessionStoreState};
use authkestra::session::{SessionConfig, SessionStore};
use axum::extract::FromRef;
use sqlx::PgPool;
use std::sync::Arc;

// Custom app state with generic parameters matching Authkestra's types
#[derive(Clone)]
pub struct AppState<S = Missing, T = Missing> {
    pub authkestra: Authkestra<S, T>,
    pub db_pool: Arc<PgPool>,
}

// Implement FromRef for Authkestra (required)
impl<S: Clone, T: Clone> FromRef<AppState<S, T>> for Authkestra<S, T> {
    fn from_ref(state: &AppState<S, T>) -> Self {
        state.authkestra.clone()
    }
}

// Implement FromRef for database pool extraction
impl<S, T> FromRef<AppState<S, T>> for Arc<PgPool> {
    fn from_ref(state: &AppState<S, T>) -> Self {
        state.db_pool.clone()
    }
}

// Implement FromRef for SessionStore (required for AuthSession extractor)
impl<S, T> FromRef<AppState<S, T>> for Result<Arc<dyn SessionStore>, AuthkestraAxumError>
where
    S: SessionStoreState,
{
    fn from_ref(state: &AppState<S, T>) -> Self {
        Ok(state.authkestra.session_store.get_store())
    }
}

// Implement FromRef for SessionConfig (required for AuthSession)
impl<S, T> FromRef<AppState<S, T>> for SessionConfig {
    fn from_ref(state: &AppState<S, T>) -> Self {
        state.authkestra.session_config.clone()
    }
}
