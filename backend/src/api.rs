use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{types::time::OffsetDateTime, PgPool};
use tracing::{info, warn};
use uuid::Uuid;

use authkestra::axum::{AuthSession, AuthkestraState};

pub fn api_router() -> Router {
    Router::new()
        .route("/me", get(me_handler))
        .route("/users", get(list_users_handler))
        .route("/messages", post(send_message_handler))
        .route("/messages/inbox", get(inbox_handler))
        .route("/broadcasts", post(create_broadcast_handler))
        .route("/broadcasts", get(list_broadcasts_handler))
}

// ===== Request/Response Types =====

#[derive(Serialize)]
struct UserResponse {
    id: Uuid,
    username: String,
    created_at: OffsetDateTime,
}

#[derive(Deserialize)]
struct SendMessageRequest {
    recipient_id: Uuid,
    content: String,
}

#[derive(Serialize)]
struct MessageResponse {
    id: Uuid,
    content: String,
    created_at: OffsetDateTime,
    is_read: bool,
}

#[derive(Deserialize)]
struct CreateBroadcastRequest {
    content: String,
    is_anonymous: bool,
}

#[derive(Serialize)]
struct BroadcastResponse {
    id: Uuid,
    sender_username: Option<String>,
    content: String,
    is_anonymous: bool,
    created_at: OffsetDateTime,
}

// ===== Handlers =====

async fn me_handler(
    AuthSession(session): AuthSession,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<UserResponse>, StatusCode> {
    let user_id = session
        .identity
        .id
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let user_uuid = Uuid::parse_str(&user_id).map_err(|e| {
        warn!("Invalid user ID format: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let user = crate::db::get_user_by_id(&pool, user_uuid)
        .await
        .map_err(|e| {
            warn!("Failed to fetch user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("User {} fetched their profile", user.username);

    Ok(Json(UserResponse {
        id: user.id,
        username: user.username,
        created_at: user.created_at,
    }))
}

async fn list_users_handler(
    AuthSession(_session): AuthSession,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<UserResponse>>, StatusCode> {
    let users = crate::db::get_all_users(&pool).await.map_err(|e| {
        warn!("Failed to fetch users: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Fetched {} users", users.len());

    Ok(Json(
        users
            .into_iter()
            .map(|u| UserResponse {
                id: u.id,
                username: u.username,
                created_at: u.created_at,
            })
            .collect(),
    ))
}

async fn send_message_handler(
    AuthSession(_session): AuthSession,
    Extension(pool): Extension<PgPool>,
    Json(req): Json<SendMessageRequest>,
) -> Result<StatusCode, StatusCode> {
    if req.content.trim().is_empty() {
        warn!("Attempted to send empty message");
        return Err(StatusCode::BAD_REQUEST);
    }

    crate::db::create_message(&pool, req.recipient_id, &req.content)
        .await
        .map_err(|e| {
            warn!("Failed to create message: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Anonymous message sent to user {}", req.recipient_id);

    Ok(StatusCode::CREATED)
}

async fn inbox_handler(
    AuthSession(session): AuthSession,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<MessageResponse>>, StatusCode> {
    let user_id = session
        .identity
        .id
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let user_uuid = Uuid::parse_str(&user_id).map_err(|e| {
        warn!("Invalid user ID format: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let messages = crate::db::get_user_inbox(&pool, user_uuid)
        .await
        .map_err(|e| {
            warn!("Failed to fetch inbox: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("User {} fetched {} messages", user_uuid, messages.len());

    Ok(Json(
        messages
            .into_iter()
            .map(|m| MessageResponse {
                id: m.id,
                content: m.content,
                created_at: m.created_at,
                is_read: m.is_read,
            })
            .collect(),
    ))
}

async fn create_broadcast_handler(
    AuthSession(session): AuthSession,
    Extension(pool): Extension<PgPool>,
    Json(req): Json<CreateBroadcastRequest>,
) -> Result<StatusCode, StatusCode> {
    if req.content.trim().is_empty() {
        warn!("Attempted to create empty broadcast");
        return Err(StatusCode::BAD_REQUEST);
    }

    let sender_id = if req.is_anonymous {
        None
    } else {
        let user_id = session
            .identity
            .id
            .ok_or(StatusCode::UNAUTHORIZED)?;
        Some(Uuid::parse_str(&user_id).map_err(|e| {
            warn!("Invalid user ID format: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?)
    };

    crate::db::create_broadcast(&pool, sender_id, &req.content, req.is_anonymous)
        .await
        .map_err(|e| {
            warn!("Failed to create broadcast: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!(
        "Broadcast created (anonymous: {})",
        req.is_anonymous
    );

    Ok(StatusCode::CREATED)
}

async fn list_broadcasts_handler(
    AuthSession(_session): AuthSession,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<BroadcastResponse>>, StatusCode> {
    let broadcasts = crate::db::get_broadcasts(&pool, 50).await.map_err(|e| {
        warn!("Failed to fetch broadcasts: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Fetched {} broadcasts", broadcasts.len());

    Ok(Json(
        broadcasts
            .into_iter()
            .map(|b| BroadcastResponse {
                id: b.id,
                sender_username: b.sender_username,
                content: b.content,
                is_anonymous: b.is_anonymous,
                created_at: b.created_at,
            })
            .collect(),
    ))
}
