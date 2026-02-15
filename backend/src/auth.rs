use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use authkestra::axum::AuthSession;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
// use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[tracing::instrument(skip(session, pool))]
pub async fn login_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<LoginRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = crate::db::get_user_by_username(&pool, &req.username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let password_hash = user.password_hash.ok_or(StatusCode::UNAUTHORIZED)?;

    let parsed_hash =
        PasswordHash::new(&password_hash).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Login successful
    // Mutate session identity fields directly as we can't import Identity type
    session.0.identity.username = Some(user.username);
    session.0.identity.provider_id = user.id.to_string();
    // Assuming email is optional and can be left as is or updated if needed
    // session.0.identity.email = None;

    Ok(StatusCode::OK)
}

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    username: String,
    password: String,
}

#[tracing::instrument(skip(session, pool))]
pub async fn register_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<RegisterRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    if req.username.trim().is_empty() || req.password.len() < 6 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check if user exists
    if crate::db::get_user_by_username(&pool, &req.username)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .is_some()
    {
        return Err(StatusCode::CONFLICT);
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    let user = crate::db::create_local_user(&pool, &req.username, &password_hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Auto login
    session.0.identity.username = Some(user.username);
    session.0.identity.provider_id = user.id.to_string();

    Ok(StatusCode::CREATED)
}

#[tracing::instrument(skip(_session, cookies))]
pub async fn logout_handler(
    _session: AuthSession,
    cookies: tower_cookies::Cookies,
) -> impl IntoResponse {
    cookies.remove(tower_cookies::Cookie::new("authkestra-session", ""));
    // Redirect to frontend login page
    Redirect::to("http://localhost:8080/login")
}

#[allow(dead_code)]
#[tracing::instrument(skip(session, _pool))]
pub async fn me_handler(
    AuthSession(session): AuthSession,
    State(_pool): State<Arc<PgPool>>,
) -> impl IntoResponse {
    let username = session
        .identity
        .username
        .as_deref()
        .unwrap_or("Anonymous")
        .to_string();

    let _pid = &session.identity.provider_id;

    format!("Logged in as: {username}")
}
