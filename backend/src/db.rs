use sqlx::{types::time::OffsetDateTime, PgPool, Result, FromRow};
use uuid::Uuid;

pub async fn init_db(database_url: &str) -> Result<PgPool> {
    let pool = PgPool::connect(database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

// ===== Models =====

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: Option<String>,
    pub provider: String,
    pub provider_id: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: OffsetDateTime,
}

#[allow(dead_code)]
#[derive(Debug, FromRow, Clone)]
pub struct Message {
    pub id: Uuid,
    pub recipient_id: Uuid,
    pub sender_id: Option<Uuid>,
    pub thread_id: Uuid,
    pub content: String,
    pub created_at: OffsetDateTime,
    pub is_read: bool,
    pub reactions: Option<serde_json::Value>,
}

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct Broadcast {
    pub id: Uuid,
    pub sender_id: Option<Uuid>,
    pub sender_username: Option<String>,
    pub content: String,
    pub is_anonymous: bool,
    pub created_at: OffsetDateTime,
    pub view_count: Option<i64>,
}

/// A conversation thread summary — the latest message in each thread,
/// enriched with unread count and recipient name (for the sender's benefit).
#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct ThreadSummary {
    pub id: Uuid,
    pub thread_id: Uuid,
    pub sender_id: Option<Uuid>,
    pub recipient_id: Uuid,
    pub content: String,
    pub created_at: OffsetDateTime,
    pub is_read: bool,
    /// How many unread messages are in this thread for the current viewer.
    pub unread_count: i64,
    /// Recipient's username — only populated when the viewer is the sender.
    /// Receivers always see NULL (anonymity preserved).
    pub recipient_username: Option<String>,
}

// ===== User Operations =====

#[tracing::instrument(skip(pool))]
pub async fn upsert_user(
    pool: &PgPool,
    username: &str,
    provider: &str,
    provider_id: Option<String>,
) -> Result<User> {
    // 1. Try to find user by provider and provider_id
    let existing_by_provider = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, password_hash, provider, provider_id, created_at, bio, avatar_url
        FROM users
        WHERE provider = $1 AND provider_id = $2
        "#,
    )
    .bind(provider)
    .bind(provider_id.as_ref())
    .fetch_optional(pool)
    .await?;

    if let Some(user) = existing_by_provider {
        // Update username if it changed on the provider's side
        if user.username != username {
            let updated = sqlx::query_as::<_, User>(
                r#"
                UPDATE users
                SET username = $1
                WHERE id = $2
                RETURNING id, username, password_hash, provider, provider_id, bio, avatar_url, created_at
                "#,
            )
            .bind(username)
            .bind(user.id)
            .fetch_one(pool)
            .await?;
            return Ok(updated);
        }
        return Ok(user);
    }

    // 2. Try to find user by username to handle linking or collisions
    let existing_by_username = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, password_hash, provider, provider_id, created_at, bio, avatar_url
        FROM users
        WHERE LOWER(username) = LOWER($1)
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    if let Some(user) = existing_by_username {
        // If the existing user is 'local' and we are now logging in with 'github',
        // we "link" them by updating the provider if it's currently local.
        if (user.provider == "local" || user.provider == "github") && user.provider_id.is_none() {
            let updated = sqlx::query_as::<_, User>(
                r#"
                UPDATE users
                SET provider = $1, provider_id = $2
                WHERE id = $3
                RETURNING id, username, password_hash, provider, provider_id, bio, avatar_url, created_at
                "#,
            )
            .bind(provider)
            .bind(provider_id)
            .bind(user.id)
            .fetch_one(pool)
            .await?;
            return Ok(updated);
        }
        // If it's already a different provider but matching provider_id, we update username if changed
        if user.provider == provider && user.provider_id == provider_id {
             if user.username != username {
                let updated = sqlx::query_as::<_, User>(
                    r#"
                    UPDATE users
                    SET username = $1
                    WHERE id = $2
                    RETURNING id, username, password_hash, provider, provider_id, bio, avatar_url, created_at
                    "#,
                )
                .bind(username)
                .bind(user.id)
                .fetch_one(pool)
                .await?;
                return Ok(updated);
            }
        }
        
        return Ok(user);
    }

    // 3. Insert new user
    let new_user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, username, provider, provider_id, created_at)
        VALUES ($1, $2, $3, $4, NOW())
        RETURNING id, username, password_hash, provider, provider_id, bio, avatar_url, created_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(username)
    .bind(provider)
    .bind(provider_id)
    .fetch_one(pool)
    .await?;
    
    Ok(new_user)
}

pub async fn create_local_user(
    pool: &PgPool,
    username: &str,
    password_hash: &str,
) -> Result<User> {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, username, password_hash, provider, created_at)
        VALUES ($1, $2, $3, 'local', NOW())
        RETURNING id, username, password_hash, provider, provider_id, bio, avatar_url, created_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(username)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;
    Ok(user)
}

pub async fn get_user_by_username(pool: &PgPool, username: &str) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, password_hash, provider, provider_id, bio, avatar_url, created_at
        FROM users
        WHERE LOWER(username) = LOWER($1)
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

#[allow(dead_code)]
pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<User> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, password_hash, provider, provider_id, bio, avatar_url, created_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_all_users(pool: &PgPool) -> Result<Vec<User>> {
    let users = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, password_hash, provider, provider_id, bio, avatar_url, created_at
        FROM users
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(users)
}

// ===== Message Operations =====

/// Create a new message in a new thread. Returns (message_id, thread_id).
#[tracing::instrument(skip(pool))]
pub async fn create_message(
    pool: &PgPool,
    sender_id: Option<Uuid>,
    recipient_id: Uuid,
    content: &str,
) -> Result<(Uuid, Uuid)> {
    let message_id = Uuid::new_v4();
    let thread_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO messages (id, thread_id, sender_id, recipient_id, content, created_at, is_read)
        VALUES ($1, $2, $3, $4, $5, NOW(), false)
        "#,
    )
    .bind(message_id)
    .bind(thread_id)
    .bind(sender_id)
    .bind(recipient_id)
    .bind(content)
    .execute(pool)
    .await?;

    Ok((message_id, thread_id))
}

/// Reply in an existing thread. Returns new message_id.
#[tracing::instrument(skip(pool))]
pub async fn create_reply(
    pool: &PgPool,
    thread_id: Uuid,
    sender_id: Uuid,
    recipient_id: Uuid,
    content: &str,
) -> Result<Uuid> {
    let message_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO messages (id, thread_id, sender_id, recipient_id, content, created_at, is_read)
        VALUES ($1, $2, $3, $4, $5, NOW(), false)
        "#,
    )
    .bind(message_id)
    .bind(thread_id)
    .bind(sender_id)
    .bind(recipient_id)
    .bind(content)
    .execute(pool)
    .await?;

    Ok(message_id)
}

/// Get all messages in a thread, ordered chronologically.
/// Never exposes sender_id to the caller — that stays server-side.
#[tracing::instrument(skip(pool))]
pub async fn get_thread_messages(pool: &PgPool, thread_id: Uuid) -> Result<Vec<Message>> {
    let messages = sqlx::query_as::<_, Message>(
        r#"
        SELECT 
            m.id,
            m.thread_id,
            m.sender_id,
            m.recipient_id,
            m.content,
            m.created_at,
            m.is_read,
            (
                SELECT json_object_agg(emoji, count)
                FROM (
                    SELECT emoji, count(*) as count
                    FROM message_reactions
                    WHERE message_id = m.id
                    GROUP BY emoji
                ) s
            ) as reactions
        FROM messages m
        WHERE m.thread_id = $1
        ORDER BY m.created_at ASC
        "#,
    )
    .bind(thread_id)
    .fetch_all(pool)
    .await?;

    Ok(messages)
}

/// Get all threads where the user is either sender or recipient.
/// Returns the latest message per thread, with unread count and
/// recipient username (only visible to the sender).
#[tracing::instrument(skip(pool))]
pub async fn get_user_conversations(pool: &PgPool, user_id: Uuid) -> Result<Vec<ThreadSummary>> {
    let threads = sqlx::query_as::<_, ThreadSummary>(
        r#"
        WITH latest_messages AS (
            SELECT DISTINCT ON (thread_id)
                id,
                thread_id,
                sender_id,
                recipient_id,
                content,
                created_at,
                is_read
            FROM messages
            WHERE sender_id = $1 OR recipient_id = $1
            ORDER BY thread_id, created_at DESC
        )
        SELECT 
            lm.id,
            lm.thread_id,
            lm.sender_id,
            lm.recipient_id,
            lm.content,
            lm.created_at,
            lm.is_read,
            -- Unread count for the current user as recipient
            (
                SELECT count(*)::bigint FROM messages
                WHERE thread_id = lm.thread_id
                  AND recipient_id = $1
                  AND is_read = false
            ) as unread_count,
            -- Recipient username: only shown to the sender
            -- For messages where current user is sender, show recipient's name
            CASE 
                WHEN lm.sender_id = $1 THEN (SELECT username FROM users WHERE id = lm.recipient_id)
                ELSE NULL 
            END as recipient_username
        FROM latest_messages lm
        ORDER BY lm.created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(threads)
}

/// Mark all messages in a thread as read for a given recipient.
pub async fn mark_thread_as_read(pool: &PgPool, thread_id: Uuid, reader_id: Uuid) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE messages
        SET is_read = true
        WHERE thread_id = $1 AND recipient_id = $2 AND is_read = false
        "#,
    )
    .bind(thread_id)
    .bind(reader_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Find the other participant in a thread (i.e. everyone who isn't `user_id`).
/// Returns the UUID of the other user, or None if not found.
#[allow(dead_code)]

#[tracing::instrument(skip(pool))]
pub async fn get_user_inbox(pool: &PgPool, recipient_id: Uuid) -> Result<Vec<Message>> {
    let messages = sqlx::query_as::<_, Message>(
        r#"
        SELECT 
            m.id, 
            m.thread_id,
            m.sender_id,
            m.recipient_id, 
            m.content, 
            m.created_at, 
            m.is_read,
            (
                SELECT json_object_agg(emoji, count)
                FROM (
                    SELECT emoji, count(*) as count
                    FROM message_reactions
                    WHERE message_id = m.id
                    GROUP BY emoji
                ) s
            ) as reactions
        FROM messages m
        WHERE m.recipient_id = $1
        ORDER BY m.created_at DESC
        "#,
    )
    .bind(recipient_id)
    .fetch_all(pool)
    .await?;

    Ok(messages)
}

pub async fn add_message_reaction(
    pool: &PgPool,
    message_id: Uuid,
    user_id: Uuid,
    emoji: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO message_reactions (message_id, user_id, emoji)
        VALUES ($1, $2, $3)
        ON CONFLICT (message_id, user_id) 
        DO UPDATE SET emoji = $3
        "#,
    )
    .bind(message_id)
    .bind(user_id)
    .bind(emoji)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get a message by its ID (used for reply — to find thread info).
pub async fn get_message_by_id(pool: &PgPool, message_id: Uuid) -> Result<Option<Message>> {
    let msg = sqlx::query_as::<_, Message>(
        r#"
        SELECT id, thread_id, sender_id, recipient_id, content, created_at, is_read, NULL::jsonb as reactions
        FROM messages
        WHERE id = $1
        "#,
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await?;
    Ok(msg)
}

// ===== Broadcast Operations =====

#[tracing::instrument(skip(pool))]
pub async fn create_broadcast(
    pool: &PgPool,
    sender_id: Option<Uuid>,
    content: &str,
    is_anonymous: bool,
) -> Result<Uuid> {
    let broadcast_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO broadcasts (id, sender_id, content, is_anonymous, created_at)
        VALUES ($1, $2, $3, $4, NOW())
        "#,
    )
    .bind(broadcast_id)
    .bind(sender_id)
    .bind(content)
    .bind(is_anonymous)
    .execute(pool)
    .await?;

    Ok(broadcast_id)
}

#[tracing::instrument(skip(pool))]
pub async fn get_broadcasts(pool: &PgPool, limit: i64) -> Result<Vec<Broadcast>> {
    let broadcasts = sqlx::query_as::<_, Broadcast>(
        r#"
        SELECT 
            b.id, 
            b.sender_id, 
            u.username as sender_username,
            b.content, 
            b.is_anonymous, 
            b.created_at,
            (SELECT count(*) FROM broadcast_views WHERE broadcast_id = b.id) as view_count
        FROM broadcasts b
        LEFT JOIN users u ON b.sender_id = u.id
        ORDER BY b.created_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(broadcasts)
}

pub async fn track_broadcast_view(pool: &PgPool, broadcast_id: Uuid, user_id: Uuid) -> Result<()> {
    sqlx::query(
        "INSERT INTO broadcast_views (broadcast_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
    )
    .bind(broadcast_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}


pub async fn update_user_profile(
    pool: &PgPool,
    user_id: Uuid,
    username: Option<String>,
    bio: Option<String>,
    avatar_url: Option<String>,
) -> Result<User> {
    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET 
            username = COALESCE($1, username),
            bio = COALESCE($2, bio),
            avatar_url = COALESCE($3, avatar_url),
            updated_at = NOW()
        WHERE id = $4
        RETURNING id, username, password_hash, provider, provider_id, bio, avatar_url, created_at
        "#,
    )
    .bind(username)
    .bind(bio)
    .bind(avatar_url)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn delete_user(pool: &PgPool, user_id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ===== Enhanced Features =====

// Message Search
pub async fn search_messages(
    pool: &PgPool,
    user_id: Uuid,
    query: &str,
    limit: i64,
) -> Result<Vec<Message>> {
    let messages = sqlx::query_as::<_, Message>(
        r#"
        SELECT 
            m.id, 
            m.thread_id,
            m.sender_id,
            m.recipient_id, 
            m.content, 
            m.created_at, 
            m.is_read,
            NULL::jsonb as reactions
        FROM messages m
        WHERE (m.recipient_id = $1 OR m.sender_id = $1)
          AND m.deleted_at IS NULL
          AND to_tsvector('english', m.content) @@ plainto_tsquery('english', $2)
        ORDER BY m.created_at DESC
        LIMIT $3
        "#,
    )
    .bind(user_id)
    .bind(query)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(messages)
}

// Message Deletion
pub async fn delete_message(
    pool: &PgPool,
    message_id: Uuid,
    user_id: Uuid,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE messages
        SET deleted_at = NOW(), deleted_by = $2
        WHERE id = $1 AND (sender_id = $2 OR recipient_id = $2)
        "#,
    )
    .bind(message_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

// Delete entire thread
pub async fn delete_thread(
    pool: &PgPool,
    thread_id: Uuid,
    user_id: Uuid,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE messages
        SET deleted_at = NOW(), deleted_by = $2
        WHERE thread_id = $1 AND (sender_id = $2 OR recipient_id = $2)
        "#,
    )
    .bind(thread_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

// Message Editing
pub async fn edit_message(
    pool: &PgPool,
    message_id: Uuid,
    user_id: Uuid,
    new_content: &str,
) -> Result<()> {
    // Get old content first
    let old_content: String = sqlx::query_scalar(
        "SELECT content FROM messages WHERE id = $1 AND sender_id = $2 AND deleted_at IS NULL"
    )
    .bind(message_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    // Store edit history
    sqlx::query(
        r#"
        INSERT INTO message_edits (message_id, old_content, edited_by)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(message_id)
    .bind(&old_content)
    .bind(user_id)
    .execute(pool)
    .await?;

    // Update message
    sqlx::query(
        r#"
        UPDATE messages
        SET content = $2, edited_at = NOW()
        WHERE id = $1 AND sender_id = $3 AND deleted_at IS NULL
        "#,
    )
    .bind(message_id)
    .bind(new_content)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

// Pin/Unpin Message
pub async fn toggle_pin_message(
    pool: &PgPool,
    message_id: Uuid,
    user_id: Uuid,
) -> Result<bool> {
    // Check if already pinned
    let is_pinned: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pinned_messages WHERE message_id = $1 AND user_id = $2)"
    )
    .bind(message_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    if is_pinned {
        // Unpin
        sqlx::query("DELETE FROM pinned_messages WHERE message_id = $1 AND user_id = $2")
            .bind(message_id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(false)
    } else {
        // Pin
        sqlx::query(
            "INSERT INTO pinned_messages (message_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(message_id)
        .bind(user_id)
        .execute(pool)
        .await?;
        Ok(true)
    }
}

// Pin/Unpin Thread
pub async fn toggle_pin_thread(
    pool: &PgPool,
    thread_id: Uuid,
    user_id: Uuid,
) -> Result<bool> {
    let is_pinned: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM pinned_threads WHERE thread_id = $1 AND user_id = $2)"
    )
    .bind(thread_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    if is_pinned {
        sqlx::query("DELETE FROM pinned_threads WHERE thread_id = $1 AND user_id = $2")
            .bind(thread_id)
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(false)
    } else {
        sqlx::query(
            "INSERT INTO pinned_threads (thread_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(thread_id)
        .bind(user_id)
        .execute(pool)
        .await?;
        Ok(true)
    }
}

// Get pinned threads for user
pub async fn get_pinned_threads(pool: &PgPool, user_id: Uuid) -> Result<Vec<Uuid>> {
    let thread_ids = sqlx::query_scalar(
        "SELECT thread_id FROM pinned_threads WHERE user_id = $1 ORDER BY pinned_at DESC"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(thread_ids)
}

// User Blocking
pub async fn block_user(
    pool: &PgPool,
    blocker_id: Uuid,
    blocked_id: Uuid,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO user_blocks (blocker_id, blocked_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
    )
    .bind(blocker_id)
    .bind(blocked_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn unblock_user(
    pool: &PgPool,
    blocker_id: Uuid,
    blocked_id: Uuid,
) -> Result<()> {
    sqlx::query("DELETE FROM user_blocks WHERE blocker_id = $1 AND blocked_id = $2")
        .bind(blocker_id)
        .bind(blocked_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_blocked_users(pool: &PgPool, user_id: Uuid) -> Result<Vec<Uuid>> {
    let blocked_ids = sqlx::query_scalar(
        "SELECT blocked_id FROM user_blocks WHERE blocker_id = $1"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(blocked_ids)
}

pub async fn is_blocked(
    pool: &PgPool,
    blocker_id: Uuid,
    blocked_id: Uuid,
) -> Result<bool> {
    let blocked: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM user_blocks WHERE blocker_id = $1 AND blocked_id = $2)"
    )
    .bind(blocker_id)
    .bind(blocked_id)
    .fetch_one(pool)
    .await?;
    Ok(blocked)
}

// Typing Indicators
pub async fn set_typing_indicator(
    pool: &PgPool,
    thread_id: Uuid,
    user_id: Uuid,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO typing_indicators (thread_id, user_id, started_at)
        VALUES ($1, $2, NOW())
        ON CONFLICT (thread_id, user_id) 
        DO UPDATE SET started_at = NOW()
        "#,
    )
    .bind(thread_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn clear_typing_indicator(
    pool: &PgPool,
    thread_id: Uuid,
    user_id: Uuid,
) -> Result<()> {
    sqlx::query("DELETE FROM typing_indicators WHERE thread_id = $1 AND user_id = $2")
        .bind(thread_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_typing_users(
    pool: &PgPool,
    thread_id: Uuid,
    exclude_user_id: Uuid,
) -> Result<Vec<Uuid>> {
    // Get users who started typing in last 5 seconds
    let user_ids = sqlx::query_scalar(
        r#"
        SELECT user_id FROM typing_indicators 
        WHERE thread_id = $1 
          AND user_id != $2
          AND started_at > NOW() - INTERVAL '5 seconds'
        "#,
    )
    .bind(thread_id)
    .bind(exclude_user_id)
    .fetch_all(pool)
    .await?;
    Ok(user_ids)
}

// Clean old typing indicators (call periodically)
pub async fn cleanup_typing_indicators(pool: &PgPool) -> Result<()> {
    sqlx::query("DELETE FROM typing_indicators WHERE started_at < NOW() - INTERVAL '10 seconds'")
        .execute(pool)
        .await?;
    Ok(())
}

// Read Receipts
pub async fn mark_message_read(
    pool: &PgPool,
    message_id: Uuid,
    reader_id: Uuid,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE messages
        SET is_read = true, read_at = NOW()
        WHERE id = $1 AND recipient_id = $2 AND is_read = false
        "#,
    )
    .bind(message_id)
    .bind(reader_id)
    .execute(pool)
    .await?;
    Ok(())
}

// Broadcast Comments
#[derive(Debug, FromRow)]
pub struct BroadcastComment {
    pub id: Uuid,
    pub broadcast_id: Uuid,
    pub user_id: Uuid,
    pub username: Option<String>,
    pub content: String,
    pub parent_comment_id: Option<Uuid>,
    pub created_at: OffsetDateTime,
    pub reactions: Option<serde_json::Value>,
}

pub async fn create_broadcast_comment(
    pool: &PgPool,
    broadcast_id: Uuid,
    user_id: Uuid,
    content: &str,
    parent_comment_id: Option<Uuid>,
) -> Result<Uuid> {
    let comment_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO broadcast_comments (id, broadcast_id, user_id, content, parent_comment_id)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(comment_id)
    .bind(broadcast_id)
    .bind(user_id)
    .bind(content)
    .bind(parent_comment_id)
    .execute(pool)
    .await?;
    Ok(comment_id)
}

pub async fn get_broadcast_comments(
    pool: &PgPool,
    broadcast_id: Uuid,
) -> Result<Vec<BroadcastComment>> {
    let comments = sqlx::query_as::<_, BroadcastComment>(
        r#"
        SELECT 
            bc.id,
            bc.broadcast_id,
            bc.user_id,
            u.username,
            bc.content,
            bc.parent_comment_id,
            bc.created_at,
            (
                SELECT json_object_agg(emoji, count)
                FROM (
                    SELECT emoji, count(*) as count
                    FROM broadcast_comment_reactions
                    WHERE comment_id = bc.id
                    GROUP BY emoji
                ) s
            ) as reactions
        FROM broadcast_comments bc
        LEFT JOIN users u ON bc.user_id = u.id
        WHERE bc.broadcast_id = $1 AND bc.deleted_at IS NULL
        ORDER BY bc.created_at ASC
        "#,
    )
    .bind(broadcast_id)
    .fetch_all(pool)
    .await?;
    Ok(comments)
}

pub async fn react_to_comment(
    pool: &PgPool,
    comment_id: Uuid,
    user_id: Uuid,
    emoji: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO broadcast_comment_reactions (comment_id, user_id, emoji)
        VALUES ($1, $2, $3)
        ON CONFLICT (comment_id, user_id) 
        DO UPDATE SET emoji = $3
        "#,
    )
    .bind(comment_id)
    .bind(user_id)
    .bind(emoji)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_broadcast_comment(
    pool: &PgPool,
    comment_id: Uuid,
    user_id: Uuid,
) -> Result<()> {
    sqlx::query(
        "UPDATE broadcast_comments SET deleted_at = NOW() WHERE id = $1 AND user_id = $2"
    )
    .bind(comment_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

// User Preferences
#[derive(Debug, FromRow)]
pub struct UserPreferences {
    pub user_id: Uuid,
    pub theme: String,
    pub notification_sound: bool,
    pub browser_notifications: bool,
    pub show_read_receipts: bool,
    pub show_typing_indicators: bool,
}

pub async fn get_user_preferences(pool: &PgPool, user_id: Uuid) -> Result<Option<UserPreferences>> {
    let prefs = sqlx::query_as::<_, UserPreferences>(
        "SELECT * FROM user_preferences WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(prefs)
}

pub async fn upsert_user_preferences(
    pool: &PgPool,
    user_id: Uuid,
    theme: Option<String>,
    notification_sound: Option<bool>,
    browser_notifications: Option<bool>,
    show_read_receipts: Option<bool>,
    show_typing_indicators: Option<bool>,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO user_preferences (
            user_id, theme, notification_sound, browser_notifications, 
            show_read_receipts, show_typing_indicators
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (user_id) DO UPDATE SET
            theme = COALESCE($2, user_preferences.theme),
            notification_sound = COALESCE($3, user_preferences.notification_sound),
            browser_notifications = COALESCE($4, user_preferences.browser_notifications),
            show_read_receipts = COALESCE($5, user_preferences.show_read_receipts),
            show_typing_indicators = COALESCE($6, user_preferences.show_typing_indicators),
            updated_at = NOW()
        "#,
    )
    .bind(user_id)
    .bind(theme)
    .bind(notification_sound)
    .bind(browser_notifications)
    .bind(show_read_receipts)
    .bind(show_typing_indicators)
    .execute(pool)
    .await?;
    Ok(())
}
