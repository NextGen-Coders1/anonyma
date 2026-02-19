use axum::{
    extract::{FromRef, FromRequestParts, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{types::time::OffsetDateTime, PgPool};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use authkestra::axum::AuthSession;

pub fn api_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Arc<PgPool>: FromRef<S>,
    AuthSession: FromRequestParts<S>,
{
    Router::new()
        .route("/me", get(me_handler))
        .route("/me", post(update_profile_handler))
        .route("/me", axum::routing::delete(delete_account_handler))
        .route("/users", get(list_users_handler))
        .route("/debug/users", get(debug_list_users_handler))
        .route("/messages", post(send_message_handler))
        .route("/messages/inbox", get(inbox_handler))
        .route("/messages/:id/react", post(react_message_handler))
        .route("/broadcasts", post(create_broadcast_handler))
        .route("/broadcasts", get(list_broadcasts_handler))
        .route("/broadcasts/:id/view", post(view_broadcast_handler))
}

// ===== Request/Response Types =====

#[derive(Serialize)]
struct DebugUserResponse {
    id: Uuid,
    username: String,
    provider: String,
    provider_id: Option<String>,
    has_password: bool,
    #[serde(with = "time::serde::rfc3339")]
    created_at: OffsetDateTime,
}

#[derive(Serialize)]
struct UserResponse {
    id: Uuid,
    username: String,
    provider: String,
    bio: Option<String>,
    avatar_url: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    created_at: OffsetDateTime,
}

#[derive(Deserialize, Debug)]
struct UpdateProfileRequest {
    username: Option<String>,
    bio: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Deserialize, Debug)]
struct SendMessageRequest {
    recipient_id: Uuid,
    content: String,
}

#[derive(Serialize)]
struct MessageResponse {
    id: Uuid,
    content: String,
    #[serde(with = "time::serde::rfc3339")]
    created_at: OffsetDateTime,
    is_read: bool,
    reactions: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
struct ReactMessageRequest {
    emoji: String,
}

#[derive(Deserialize, Debug)]
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
    #[serde(with = "time::serde::rfc3339")]
    created_at: OffsetDateTime,
    view_count: i64,
}

#[tracing::instrument(skip(session, pool))]
async fn resolve_user(
    session: &mut AuthSession,
    pool: &PgPool,
) -> Result<crate::db::User, StatusCode> {
    let provider = session.0.identity.provider_id.clone();
    let external_id = session.0.identity.external_id.clone();
    let username = session.0.identity.username.clone();

    // If both are missing, we're definitely not logged in
    if external_id.is_empty() && username.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let username = username.unwrap_or_else(|| "Anonymous".to_string());

    // For local users, external_id is their UUID in our DB
    if provider == "local" {
        if let Ok(user_id) = Uuid::parse_str(&external_id) {
            info!("Resolving local user by UUID: {user_id}");
            return crate::db::get_user_by_id(pool, user_id)
                .await
                .map_err(|e| {
                    warn!("Failed to resolve user by ID {user_id}: {e}");
                    StatusCode::UNAUTHORIZED
                });
        }
    }

    // For GitHub/OAuth users, external_id is their provider-side ID
    info!("Resolving {provider} user with external_id: {external_id}");
    let user = crate::db::upsert_user(pool, &username, &provider, Some(external_id))
        .await
        .map_err(|e| {
            warn!("Failed to sync user: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(user)
}

// ===== Handlers =====

#[tracing::instrument(skip(session, pool))]
async fn me_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
) -> Result<Json<UserResponse>, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    info!("User {} fetched profile", user.username);

    Ok(Json(UserResponse {
        id: user.id,
        username: user.username,
        provider: user.provider,
        bio: user.bio,
        avatar_url: user.avatar_url,
        created_at: user.created_at,
    }))
}

#[tracing::instrument(skip(session, pool))]
async fn update_profile_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    let updated_user = crate::db::update_user_profile(
        &pool,
        user.id,
        req.username,
        req.bio,
        req.avatar_url,
    )
    .await
    .map_err(|e| {
        warn!("Failed to update profile for user {}: {}", user.id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("User {} updated profile", updated_user.username);

    Ok(Json(UserResponse {
        id: updated_user.id,
        username: updated_user.username,
        provider: updated_user.provider,
        bio: updated_user.bio,
        avatar_url: updated_user.avatar_url,
        created_at: updated_user.created_at,
    }))
}

#[tracing::instrument(skip(session, pool))]
async fn delete_account_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
) -> Result<StatusCode, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    crate::db::delete_user(&pool, user.id).await.map_err(|e| {
        warn!("Failed to delete user {}: {}", user.id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("User {} deleted their account", user.username);

    // Note: session logout should ideally be handled by the client redirecting to /logout
    Ok(StatusCode::NO_CONTENT)
}

#[tracing::instrument(skip(_session, pool))]
async fn list_users_handler(
    _session: AuthSession,
    State(pool): State<Arc<PgPool>>,
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
                provider: u.provider,
                bio: u.bio,
                avatar_url: u.avatar_url,
                created_at: u.created_at,
            })
            .collect(),
    ))
}


#[tracing::instrument(skip(_session, pool))]
async fn debug_list_users_handler(
    _session: AuthSession,
    State(pool): State<Arc<PgPool>>,
) -> Result<Json<Vec<DebugUserResponse>>, StatusCode> {
    let users = crate::db::get_all_users(&pool).await.map_err(|e| {
        warn!("Failed to fetch users: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(
        users
            .into_iter()
            .map(|u| DebugUserResponse {
                id: u.id,
                username: u.username,
                provider: u.provider,
                provider_id: u.provider_id,
                has_password: u.password_hash.is_some(),
                created_at: u.created_at,
            })
            .collect(),
    ))
}

#[tracing::instrument(skip(_session, pool))]
async fn send_message_handler(
    _session: AuthSession,
    State(pool): State<Arc<PgPool>>,
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

#[tracing::instrument(skip(session, pool))]
async fn inbox_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
) -> Result<Json<Vec<MessageResponse>>, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    let messages = crate::db::get_user_inbox(&pool, user.id)
        .await
        .map_err(|e| {
            warn!("Failed to fetch inbox: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("User {} fetched {} messages", user.username, messages.len());

    Ok(Json(
        messages
            .into_iter()
            .map(|m| MessageResponse {
                id: m.id,
                content: m.content,
                created_at: m.created_at,
                is_read: m.is_read,
                reactions: m.reactions,
            })
            .collect(),
    ))
}

#[tracing::instrument(skip(session, pool))]
async fn react_message_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    axum::extract::Path(message_id): axum::extract::Path<Uuid>,
    Json(req): Json<ReactMessageRequest>,
) -> Result<StatusCode, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    crate::db::add_message_reaction(&pool, message_id, user.id, &req.emoji)
        .await
        .map_err(|e| {
            warn!("Failed to add reaction: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}

#[tracing::instrument(skip(session, pool))]
async fn create_broadcast_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<CreateBroadcastRequest>,
) -> Result<StatusCode, StatusCode> {
    if req.content.trim().is_empty() {
        warn!("Attempted to create empty broadcast");
        return Err(StatusCode::BAD_REQUEST);
    }

    let user = resolve_user(&mut session, &pool).await?;
    let sender_id = if req.is_anonymous {
        None
    } else {
        Some(user.id)
    };

    crate::db::create_broadcast(&pool, sender_id, &req.content, req.is_anonymous)
        .await
        .map_err(|e| {
            warn!("Failed to create broadcast: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Broadcast created (anonymous: {})", req.is_anonymous);

    Ok(StatusCode::CREATED)
}

#[tracing::instrument(skip(_session, pool))]
async fn list_broadcasts_handler(
    _session: AuthSession,
    State(pool): State<Arc<PgPool>>,
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
                view_count: b.view_count.unwrap_or(0),
            })
            .collect(),
    ))
}

#[tracing::instrument(skip(session, pool))]
async fn view_broadcast_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    axum::extract::Path(broadcast_id): axum::extract::Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    crate::db::track_broadcast_view(&pool, broadcast_id, user.id)
        .await
        .map_err(|e| {
            warn!("Failed to track view: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}

