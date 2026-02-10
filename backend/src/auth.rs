use authkestra::axum::AuthSession;
use axum::response::{IntoResponse, Redirect};

pub async fn logout_handler(AuthSession(_session): AuthSession) -> impl IntoResponse {
    // Session doesn't have a logout method in this version
    // Logout is typically handled by clearing the cookie or redirecting to a logout endpoint
    // For now, just redirect
    Redirect::to("/")
}

pub async fn me_handler(AuthSession(session): AuthSession) -> impl IntoResponse {
    // session.identity is the Identity struct directly
    let username = session.identity.username.as_deref().unwrap_or("Anonymous");
    format!("Logged in as: {}", username)
}
