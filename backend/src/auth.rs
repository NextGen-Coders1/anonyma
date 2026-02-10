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

#[allow(dead_code)]
pub async fn me_handler(
    AuthSession(session): AuthSession,
    Extension(pool): Extension<Arc<PgPool>>,
) -> impl IntoResponse {
    let username = session
        .identity
        .username
        .as_deref()
        .unwrap_or("Anonymous")
        .to_string();

    // In this context, we know it's GitHub because that's our only provider
    let provider = "github";
    let pid = &session.identity.provider_id;

    // Lazy Registration: Upsert user to database
    let p_id: Option<String> = Some(pid.clone());
    match crate::db::upsert_user(&pool, &username, provider, p_id).await {
        Ok(_) => tracing::info!("User {} synced to database", username),
        Err(e) => tracing::warn!("Failed to sync user to database: {}", e),
    }

    // Return the username
    format!("Logged in as: {username}")
}
