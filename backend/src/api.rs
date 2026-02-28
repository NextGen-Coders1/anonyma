use axum::response::sse::{Event, KeepAlive};
use axum::{
    extract::{FromRef, FromRequestParts, State},
    http::StatusCode,
    response::{Json, Sse},
    routing::{get, post},
    Router,
};
use futures_util::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use sqlx::{types::time::OffsetDateTime, PgPool};
use std::convert::Infallible;
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::state::{NotificationHub, SseEvent};
use authkestra::axum::AuthSession;

pub fn api_router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Arc<PgPool>: FromRef<S>,
    NotificationHub: FromRef<S>,
    AuthSession: FromRequestParts<S>,
{
    Router::new()
        .route("/me", get(me_handler))
        .route("/me", post(update_profile_handler))
        .route("/me", axum::routing::delete(delete_account_handler))
        .route("/users", get(list_users_handler))
        .route("/debug/users", get(debug_list_users_handler))
        // Messaging
        .route("/messages", post(send_message_handler))
        .route("/messages/inbox", get(inbox_handler))
        .route("/messages/search", get(search_messages_handler))
        .route("/messages/{id}/react", post(react_message_handler))
        .route("/messages/{id}/reply", post(reply_message_handler))
        .route("/messages/{id}/edit", post(edit_message_handler))
        .route(
            "/messages/{id}/delete",
            axum::routing::delete(delete_message_handler),
        )
        .route("/messages/{id}/pin", post(toggle_pin_message_handler))
        // Conversations (threads)
        .route("/conversations", get(list_conversations_handler))
        .route("/conversations/{thread_id}", get(get_thread_handler))
        .route(
            "/conversations/{thread_id}/delete",
            axum::routing::delete(delete_thread_handler),
        )
        .route(
            "/conversations/{thread_id}/pin",
            post(toggle_pin_thread_handler),
        )
        .route(
            "/conversations/{thread_id}/typing",
            post(typing_indicator_handler),
        )
        // User Blocking
        .route("/users/{id}/block", post(block_user_handler))
        .route("/users/{id}/unblock", post(unblock_user_handler))
        .route("/users/blocked", get(get_blocked_users_handler))
        // Broadcasts
        .route("/broadcasts", post(create_broadcast_handler))
        .route("/broadcasts", get(list_broadcasts_handler))
        .route("/broadcasts/{id}/view", post(view_broadcast_handler))
        .route(
            "/broadcasts/{id}/comments",
            get(get_broadcast_comments_handler),
        )
        .route(
            "/broadcasts/{id}/comments",
            post(create_broadcast_comment_handler),
        )
        .route(
            "/broadcasts/comments/{id}/react",
            post(react_to_comment_handler),
        )
        .route(
            "/broadcasts/comments/{id}/delete",
            axum::routing::delete(delete_comment_handler),
        )
        // User Preferences
        .route("/preferences", get(get_preferences_handler))
        .route("/preferences", post(update_preferences_handler))
        // SSE real-time event stream
        .route("/events", get(sse_handler))
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

#[derive(Deserialize, Debug)]
struct ReplyRequest {
    content: String,
}

/// Message response sent to clients — sender_id is intentionally omitted to preserve anonymity.
#[derive(Serialize, Clone)]
struct MessageResponse {
    id: Uuid,
    thread_id: Uuid,
    content: String,
    /// true = the current viewer sent this message; false = received
    is_mine: bool,
    #[serde(with = "time::serde::rfc3339")]
    created_at: OffsetDateTime,
    is_read: bool,
    reactions: Option<serde_json::Value>,
    /// Number of unread messages in this thread for the current user (used in thread list)
    #[serde(skip_serializing_if = "Option::is_none")]
    unread_count: Option<i64>,
    /// Recipient's display name — only set when the current user is the SENDER.
    /// Receivers always get null to preserve anonymity.
    #[serde(skip_serializing_if = "Option::is_none")]
    to_username: Option<String>,
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
            return crate::db::get_user_by_id(pool, user_id).await.map_err(|e| {
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

// ===== SSE Helper =====

/// Notify a specific user over SSE (if they are connected).
async fn notify_user_sse(hub: &NotificationHub, user_id: Uuid, event: SseEvent) {
    let hub = hub.lock().await;
    if let Some(sender) = hub.get(&user_id) {
        // Ignore errors — user may have disconnected
        let _ = sender.send(event);
    }
}

/// Broadcast an SSE event to ALL connected users.
async fn notify_all_sse(hub: &NotificationHub, event: SseEvent) {
    let hub = hub.lock().await;
    for sender in hub.values() {
        let _ = sender.send(event.clone());
    }
}

// ===== Handlers =====

/// SSE endpoint — streams real-time events to the authenticated user.
#[tracing::instrument(skip(session, pool, hub))]
async fn sse_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    State(hub): State<NotificationHub>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;
    let user_id = user.id;

    info!("User {} connected to SSE stream", user.username);

    // Create or re-use a broadcast channel for this user
    let receiver = {
        let mut hub = hub.lock().await;
        let sender = hub.entry(user_id).or_insert_with(|| {
            let (tx, _) = tokio::sync::broadcast::channel(32);
            tx
        });
        sender.subscribe()
    };

    // Convert the broadcast receiver into a Stream of SSE Events
    let stream = stream::unfold(receiver, |mut rx| async move {
        match rx.recv().await {
            Ok(evt) => {
                let sse_event = Event::default().event(evt.event_type).data(evt.data);
                Some((Ok(sse_event), rx))
            }
            Err(_) => None, // Channel closed or lagged — end stream
        }
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

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

    let updated_user =
        crate::db::update_user_profile(&pool, user.id, req.username, req.bio, req.avatar_url)
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

#[tracing::instrument(skip(session, pool))]
async fn list_users_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
) -> Result<Json<Vec<UserResponse>>, StatusCode> {
    // Resolve the current user so we can exclude them from the list
    let current_user = resolve_user(&mut session, &pool).await?;

    let users = crate::db::get_all_users(&pool).await.map_err(|e| {
        warn!("Failed to fetch users: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let filtered: Vec<UserResponse> = users
        .into_iter()
        .filter(|u| u.id != current_user.id) // exclude self
        .map(|u| UserResponse {
            id: u.id,
            username: u.username,
            provider: u.provider,
            bio: u.bio,
            avatar_url: u.avatar_url,
            created_at: u.created_at,
        })
        .collect();

    info!("Fetched {} users (excluding self)", filtered.len());
    Ok(Json(filtered))
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

/// Send a new anonymous message (starts a new thread).
#[tracing::instrument(skip(session, pool, hub))]
async fn send_message_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    State(hub): State<NotificationHub>,
    Json(req): Json<SendMessageRequest>,
) -> Result<StatusCode, StatusCode> {
    if req.content.trim().is_empty() {
        warn!("Attempted to send empty message");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Resolve sender — may be None for fully anonymous (unauthenticated) sends
    let sender_id = resolve_user(&mut session, &pool).await.ok().map(|u| u.id);

    let (message_id, thread_id) =
        crate::db::create_message(&pool, sender_id, req.recipient_id, &req.content)
            .await
            .map_err(|e| {
                warn!("Failed to create message: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    info!(
        "Anonymous message {} sent to user {}",
        message_id, req.recipient_id
    );

    // Push SSE notification to recipient (if online)
    let payload = serde_json::json!({
        "message_id": message_id,
        "thread_id": thread_id,
        "content": req.content,
    })
    .to_string();
    notify_user_sse(
        &hub,
        req.recipient_id,
        SseEvent {
            event_type: "new_message".to_string(),
            data: payload,
        },
    )
    .await;

    Ok(StatusCode::CREATED)
}

/// Reply to an existing thread.
#[tracing::instrument(skip(session, pool, hub))]
async fn reply_message_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    State(hub): State<NotificationHub>,
    axum::extract::Path(message_id): axum::extract::Path<Uuid>,
    Json(req): Json<ReplyRequest>,
) -> Result<StatusCode, StatusCode> {
    if req.content.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let user = resolve_user(&mut session, &pool).await?;

    // Load the original message to find thread_id and who to reply to
    let original = crate::db::get_message_by_id(&pool, message_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // The reply goes to: if the current user is the recipient, reply to the sender;
    // if the current user is the sender, reply to the recipient.
    let reply_recipient_id = if original.recipient_id == user.id {
        // User is the recipient — reply to original sender (if known)
        original.sender_id.ok_or_else(|| {
            warn!("Cannot reply: original sender is anonymous (no sender_id stored)");
            StatusCode::BAD_REQUEST
        })?
    } else if original.sender_id == Some(user.id) {
        // User is the sender — reply to the recipient
        original.recipient_id
    } else {
        warn!(
            "User {} tried to reply to a message they're not part of",
            user.id
        );
        return Err(StatusCode::FORBIDDEN);
    };

    let new_message_id = crate::db::create_reply(
        &pool,
        original.thread_id,
        user.id,
        reply_recipient_id,
        &req.content,
    )
    .await
    .map_err(|e| {
        warn!("Failed to create reply: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!(
        "Reply {} in thread {} sent",
        new_message_id, original.thread_id
    );

    // Notify recipient over SSE
    let payload = serde_json::json!({
        "message_id": new_message_id,
        "thread_id": original.thread_id,
        "content": req.content,
    })
    .to_string();
    notify_user_sse(
        &hub,
        reply_recipient_id,
        SseEvent {
            event_type: "new_message".to_string(),
            data: payload,
        },
    )
    .await;

    Ok(StatusCode::CREATED)
}

/// List all conversations (threads) the current user participates in.
#[tracing::instrument(skip(session, pool))]
async fn list_conversations_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
) -> Result<Json<Vec<MessageResponse>>, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    let threads = crate::db::get_user_conversations(&pool, user.id)
        .await
        .map_err(|e| {
            warn!("Failed to fetch conversations: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(
        threads
            .into_iter()
            .map(|t| MessageResponse {
                id: t.id,
                thread_id: t.thread_id,
                content: t.content,
                is_mine: t.sender_id == Some(user.id),
                created_at: t.created_at,
                is_read: t.is_read,
                reactions: None,
                unread_count: Some(t.unread_count),
                to_username: t.recipient_username, // null for recipients, name for senders
            })
            .collect(),
    ))
}

/// Get all messages in a thread. Also marks received messages as read.
#[tracing::instrument(skip(session, pool))]
async fn get_thread_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    axum::extract::Path(thread_id): axum::extract::Path<Uuid>,
) -> Result<Json<Vec<MessageResponse>>, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    let msgs = crate::db::get_thread_messages(&pool, thread_id)
        .await
        .map_err(|e| {
            warn!("Failed to fetch thread {}: {}", thread_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Ensure the user is part of this thread
    let is_participant = msgs
        .iter()
        .any(|m| m.recipient_id == user.id || m.sender_id == Some(user.id));
    if !msgs.is_empty() && !is_participant {
        return Err(StatusCode::FORBIDDEN);
    }

    // Mark messages received by this user as read
    if let Err(e) = crate::db::mark_thread_as_read(&pool, thread_id, user.id).await {
        warn!("Failed to mark thread as read: {}", e);
    }

    Ok(Json(
        msgs.into_iter()
            .map(|m| MessageResponse {
                id: m.id,
                thread_id: m.thread_id,
                content: m.content,
                is_mine: m.sender_id == Some(user.id),
                created_at: m.created_at,
                is_read: m.is_read,
                reactions: m.reactions,
                unread_count: None,
                to_username: None, // individual messages don't need this
            })
            .collect(),
    ))
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
                thread_id: m.thread_id,
                content: m.content,
                is_mine: false, // inbox = always received
                created_at: m.created_at,
                is_read: m.is_read,
                reactions: m.reactions,
                unread_count: None,
                to_username: None,
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

#[tracing::instrument(skip(session, pool, hub))]
async fn create_broadcast_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    State(hub): State<NotificationHub>,
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

    let broadcast_id =
        crate::db::create_broadcast(&pool, sender_id, &req.content, req.is_anonymous)
            .await
            .map_err(|e| {
                warn!("Failed to create broadcast: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    info!(
        "Broadcast {} created (anonymous: {})",
        broadcast_id, req.is_anonymous
    );

    // Push SSE event to ALL connected users so their broadcasts page updates
    let payload = serde_json::json!({
        "broadcast_id": broadcast_id,
    })
    .to_string();
    notify_all_sse(
        &hub,
        SseEvent {
            event_type: "new_broadcast".to_string(),
            data: payload,
        },
    )
    .await;

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

// ===== Enhanced Features Handlers =====

// Message Search
#[derive(Deserialize, Debug)]
struct SearchQuery {
    q: String,
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    50
}

#[tracing::instrument(skip(session, pool))]
async fn search_messages_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    axum::extract::Query(query): axum::extract::Query<SearchQuery>,
) -> Result<Json<Vec<MessageResponse>>, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    if query.q.trim().is_empty() {
        return Ok(Json(vec![]));
    }

    let messages = crate::db::search_messages(&pool, user.id, &query.q, query.limit)
        .await
        .map_err(|e| {
            warn!("Failed to search messages: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(
        messages
            .into_iter()
            .map(|m| MessageResponse {
                id: m.id,
                thread_id: m.thread_id,
                content: m.content,
                is_mine: m.sender_id == Some(user.id),
                created_at: m.created_at,
                is_read: m.is_read,
                reactions: m.reactions,
                unread_count: None,
                to_username: None,
            })
            .collect(),
    ))
}

// Message Deletion
#[tracing::instrument(skip(session, pool))]
async fn delete_message_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    axum::extract::Path(message_id): axum::extract::Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    crate::db::delete_message(&pool, message_id, user.id)
        .await
        .map_err(|e| {
            warn!("Failed to delete message: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("User {} deleted message {}", user.username, message_id);
    Ok(StatusCode::NO_CONTENT)
}

// Thread Deletion
#[tracing::instrument(skip(session, pool))]
async fn delete_thread_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    axum::extract::Path(thread_id): axum::extract::Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    crate::db::delete_thread(&pool, thread_id, user.id)
        .await
        .map_err(|e| {
            warn!("Failed to delete thread: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("User {} deleted thread {}", user.username, thread_id);
    Ok(StatusCode::NO_CONTENT)
}

// Message Editing
#[derive(Deserialize, Debug)]
struct EditMessageRequest {
    content: String,
}

#[tracing::instrument(skip(session, pool))]
async fn edit_message_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    axum::extract::Path(message_id): axum::extract::Path<Uuid>,
    Json(req): Json<EditMessageRequest>,
) -> Result<StatusCode, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    if req.content.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    crate::db::edit_message(&pool, message_id, user.id, &req.content)
        .await
        .map_err(|e| {
            warn!("Failed to edit message: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("User {} edited message {}", user.username, message_id);
    Ok(StatusCode::OK)
}

// Pin/Unpin Message
#[tracing::instrument(skip(session, pool))]
async fn toggle_pin_message_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    axum::extract::Path(message_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    let is_pinned = crate::db::toggle_pin_message(&pool, message_id, user.id)
        .await
        .map_err(|e| {
            warn!("Failed to toggle pin message: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({ "pinned": is_pinned })))
}

// Pin/Unpin Thread
#[tracing::instrument(skip(session, pool))]
async fn toggle_pin_thread_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    axum::extract::Path(thread_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    let is_pinned = crate::db::toggle_pin_thread(&pool, thread_id, user.id)
        .await
        .map_err(|e| {
            warn!("Failed to toggle pin thread: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(serde_json::json!({ "pinned": is_pinned })))
}

// Typing Indicator
#[tracing::instrument(skip(session, pool, hub))]
async fn typing_indicator_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    State(hub): State<NotificationHub>,
    axum::extract::Path(thread_id): axum::extract::Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    crate::db::set_typing_indicator(&pool, thread_id, user.id)
        .await
        .map_err(|e| {
            warn!("Failed to set typing indicator: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Notify other participants via SSE
    let payload = serde_json::json!({
        "thread_id": thread_id,
        "user_id": user.id,
        "username": user.username,
    })
    .to_string();

    // Get other participant from the thread and notify them
    let other_user_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT DISTINCT CASE 
            WHEN sender_id IS NOT NULL AND sender_id != $2 THEN sender_id
            WHEN recipient_id != $2 THEN recipient_id
            ELSE NULL
        END AS other_user
        FROM messages 
        WHERE thread_id = $1 AND (sender_id != $2 OR recipient_id != $2)
        LIMIT 1
        "#,
    )
    .bind(thread_id)
    .bind(user.id)
    .fetch_optional(&*pool)
    .await;

    if let Ok(Some(other_user_id)) = other_user_id {
        notify_user_sse(
            &hub,
            other_user_id,
            SseEvent {
                event_type: "typing".to_string(),
                data: payload,
            },
        )
        .await;
    }

    Ok(StatusCode::OK)
}

// User Blocking
#[tracing::instrument(skip(session, pool))]
async fn block_user_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    axum::extract::Path(blocked_id): axum::extract::Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    if user.id == blocked_id {
        return Err(StatusCode::BAD_REQUEST);
    }

    crate::db::block_user(&pool, user.id, blocked_id)
        .await
        .map_err(|e| {
            warn!("Failed to block user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("User {} blocked user {}", user.username, blocked_id);
    Ok(StatusCode::OK)
}

#[tracing::instrument(skip(session, pool))]
async fn unblock_user_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    axum::extract::Path(blocked_id): axum::extract::Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    crate::db::unblock_user(&pool, user.id, blocked_id)
        .await
        .map_err(|e| {
            warn!("Failed to unblock user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("User {} unblocked user {}", user.username, blocked_id);
    Ok(StatusCode::OK)
}

#[tracing::instrument(skip(session, pool))]
async fn get_blocked_users_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
) -> Result<Json<Vec<Uuid>>, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    let blocked_ids = crate::db::get_blocked_users(&pool, user.id)
        .await
        .map_err(|e| {
            warn!("Failed to get blocked users: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(blocked_ids))
}

// Broadcast Comments
#[derive(Deserialize, Debug)]
struct CreateCommentRequest {
    content: String,
    parent_comment_id: Option<Uuid>,
}

#[derive(Serialize)]
struct CommentResponse {
    id: Uuid,
    broadcast_id: Uuid,
    user_id: Uuid,
    username: Option<String>,
    content: String,
    parent_comment_id: Option<Uuid>,
    #[serde(with = "time::serde::rfc3339")]
    created_at: OffsetDateTime,
    reactions: Option<serde_json::Value>,
}

#[tracing::instrument(skip(session, pool, hub))]
async fn create_broadcast_comment_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    State(hub): State<NotificationHub>,
    axum::extract::Path(broadcast_id): axum::extract::Path<Uuid>,
    Json(req): Json<CreateCommentRequest>,
) -> Result<StatusCode, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    if req.content.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let comment_id = crate::db::create_broadcast_comment(
        &pool,
        broadcast_id,
        user.id,
        &req.content,
        req.parent_comment_id,
    )
    .await
    .map_err(|e| {
        warn!("Failed to create comment: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!(
        "User {} commented on broadcast {}",
        user.username, broadcast_id
    );

    // Notify all users via SSE
    let payload = serde_json::json!({
        "broadcast_id": broadcast_id,
        "comment_id": comment_id,
    })
    .to_string();

    notify_all_sse(
        &hub,
        SseEvent {
            event_type: "new_comment".to_string(),
            data: payload,
        },
    )
    .await;

    Ok(StatusCode::CREATED)
}

#[tracing::instrument(skip(session, pool))]
async fn get_broadcast_comments_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    axum::extract::Path(broadcast_id): axum::extract::Path<Uuid>,
) -> Result<Json<Vec<CommentResponse>>, StatusCode> {
    let _user = resolve_user(&mut session, &pool).await?;

    let comments = crate::db::get_broadcast_comments(&pool, broadcast_id)
        .await
        .map_err(|e| {
            warn!("Failed to get comments: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(
        comments
            .into_iter()
            .map(|c| CommentResponse {
                id: c.id,
                broadcast_id: c.broadcast_id,
                user_id: c.user_id,
                username: c.username,
                content: c.content,
                parent_comment_id: c.parent_comment_id,
                created_at: c.created_at,
                reactions: c.reactions,
            })
            .collect(),
    ))
}

#[derive(Deserialize, Debug)]
struct ReactToCommentRequest {
    emoji: String,
}

#[tracing::instrument(skip(session, pool))]
async fn react_to_comment_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    axum::extract::Path(comment_id): axum::extract::Path<Uuid>,
    Json(req): Json<ReactToCommentRequest>,
) -> Result<StatusCode, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    crate::db::react_to_comment(&pool, comment_id, user.id, &req.emoji)
        .await
        .map_err(|e| {
            warn!("Failed to react to comment: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}

#[tracing::instrument(skip(session, pool))]
async fn delete_comment_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    axum::extract::Path(comment_id): axum::extract::Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    crate::db::delete_broadcast_comment(&pool, comment_id, user.id)
        .await
        .map_err(|e| {
            warn!("Failed to delete comment: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// User Preferences
#[derive(Serialize)]
struct PreferencesResponse {
    theme: String,
    notification_sound: bool,
    browser_notifications: bool,
    show_read_receipts: bool,
    show_typing_indicators: bool,
}

#[tracing::instrument(skip(session, pool))]
async fn get_preferences_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
) -> Result<Json<PreferencesResponse>, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    let prefs = crate::db::get_user_preferences(&pool, user.id)
        .await
        .map_err(|e| {
            warn!("Failed to get preferences: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let prefs = prefs.unwrap_or(crate::db::UserPreferences {
        user_id: user.id,
        theme: "dark".to_string(),
        notification_sound: true,
        browser_notifications: true,
        show_read_receipts: true,
        show_typing_indicators: true,
    });

    Ok(Json(PreferencesResponse {
        theme: prefs.theme,
        notification_sound: prefs.notification_sound,
        browser_notifications: prefs.browser_notifications,
        show_read_receipts: prefs.show_read_receipts,
        show_typing_indicators: prefs.show_typing_indicators,
    }))
}

#[derive(Deserialize, Debug)]
struct UpdatePreferencesRequest {
    theme: Option<String>,
    notification_sound: Option<bool>,
    browser_notifications: Option<bool>,
    show_read_receipts: Option<bool>,
    show_typing_indicators: Option<bool>,
}

#[tracing::instrument(skip(session, pool))]
async fn update_preferences_handler(
    mut session: AuthSession,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<UpdatePreferencesRequest>,
) -> Result<StatusCode, StatusCode> {
    let user = resolve_user(&mut session, &pool).await?;

    crate::db::upsert_user_preferences(
        &pool,
        user.id,
        req.theme,
        req.notification_sound,
        req.browser_notifications,
        req.show_read_receipts,
        req.show_typing_indicators,
    )
    .await
    .map_err(|e| {
        warn!("Failed to update preferences: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("User {} updated preferences", user.username);
    Ok(StatusCode::OK)
}
