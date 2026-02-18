use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use authkestra::axum::helpers::{create_axum_cookie, logout};
use authkestra::flow::SessionStoreState;
use authkestra::session::{Identity, SessionStore};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tower_cookies::Cookies;
use tracing::{info, warn};

use crate::state::AppState;

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[tracing::instrument(skip(cookies, state))]
pub async fn login_handler(
    cookies: Cookies,
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let pool = &state.db_pool;

    let user = crate::db::get_user_by_username(pool, &req.username)
        .await
        .map_err(|e| {
            warn!("DB error during login: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            warn!("Login failed: user '{}' not found", req.username);
            StatusCode::UNAUTHORIZED
        })?;

    let password_hash = user.password_hash.as_ref().ok_or_else(|| {
        warn!("Login failed: user '{}' has no password (OAuth only?)", user.username);
        StatusCode::UNAUTHORIZED
    })?;

    let parsed_hash = PasswordHash::new(password_hash).map_err(|e| {
        warn!("Failed to parse password hash for user {}: {e}", user.username);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|e| {
            warn!("Password verification failed for user {}: {e}", user.username);
            StatusCode::UNAUTHORIZED
        })?;

    // Password verified — create a server-side session
    info!("Password login successful for user: {}, user_id: {}", user.username, user.id);

    let identity = Identity {
        provider_id: "local".to_string(),
        external_id: user.id.to_string(),
        email: None,
        username: Some(user.username.clone()),
        attributes: HashMap::new(),
    };

    let session = state.authkestra.create_session(identity).await.map_err(|e| {
        warn!("Failed to create session: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let cookie = create_axum_cookie(&state.authkestra.session_config, session.id);
    cookies.add(cookie);

    info!("Session created and cookie set for user: {}", user.username);

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "user": { "id": user.id, "username": user.username }
        })),
    ))
}

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    username: String,
    password: String,
}

#[tracing::instrument(skip(cookies, state))]
pub async fn register_handler(
    cookies: Cookies,
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let pool = &state.db_pool;

    if req.username.trim().is_empty() {
        warn!("Registration failed: empty username");
        return Err(StatusCode::BAD_REQUEST);
    }

    if req.password.len() < 6 {
        warn!("Registration failed: password too short");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check if user exists (case-insensitive)
    let exists = crate::db::get_user_by_username(pool, &req.username)
        .await
        .map_err(|e| {
            warn!("DB error during registration check: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .is_some();

    if exists {
        warn!("Registration failed: user '{}' already exists", req.username);
        return Err(StatusCode::CONFLICT);
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| {
            warn!("Hashing failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .to_string();

    let user = crate::db::create_local_user(pool, &req.username, &password_hash)
        .await
        .map_err(|e| {
            warn!("Failed to create local user: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("New user registered: {}, id: {}", user.username, user.id);

    // Auto-login: create a session for the new user
    let identity = Identity {
        provider_id: "local".to_string(),
        external_id: user.id.to_string(),
        email: None,
        username: Some(user.username.clone()),
        attributes: HashMap::new(),
    };

    let session = state.authkestra.create_session(identity).await.map_err(|e| {
        warn!("Failed to create session after registration: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let cookie = create_axum_cookie(&state.authkestra.session_config, session.id);
    cookies.add(cookie);

    info!("Registration successful for user: {}, session created", user.username);

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "status": "ok",
            "user": { "id": user.id, "username": user.username }
        })),
    ))
}

#[tracing::instrument(skip(cookies, state))]
pub async fn logout_handler(
    cookies: Cookies,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let session_store: Arc<dyn SessionStore> = state.authkestra.session_store.get_store();
    let session_config = state.authkestra.session_config.clone();

    match logout(cookies, session_store, session_config, "http://localhost:8080/login").await {
        Ok(response) => response.into_response(),
        Err((status, msg)) => {
            warn!("Logout error: {msg}");
            (status, msg).into_response()
        }
    }
}

#[allow(dead_code)]
#[tracing::instrument(skip(session, _pool))]
pub async fn me_handler(
    session: authkestra::axum::AuthSession,
    State(_pool): State<Arc<PgPool>>,
) -> impl IntoResponse {
    let username = session
        .0
        .identity
        .username
        .as_deref()
        .unwrap_or("Anonymous")
        .to_string();

    let _pid = &session.0.identity.provider_id;

    format!("Logged in as: {username}")
}
