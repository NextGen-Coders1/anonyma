use authkestra::axum::AuthSession;
use axum::{
    extract::Extension,
    response::{IntoResponse, Redirect},
};
use sqlx::PgPool;
use std::sync::Arc;

pub async fn logout_handler(AuthSession(_session): AuthSession) -> impl IntoResponse {
    // Session doesn't have a logout method in this version
    // Logout is typically handled by clearing the cookie or redirecting to a logout endpoint
    // For now, just redirect
    Redirect::to("/")
}

pub async fn me_handler(
    AuthSession(session): AuthSession,
    Extension(pool): Extension<Arc<PgPool>>,
) -> impl IntoResponse {
    // session.identity is the Identity struct directly
    let username = session.identity.username.as_deref().unwrap_or("Anonymous");

    // TODO: Upsert user to database on first login
    // For now, just return the username
    format!("Logged in as: {}", username)
}
