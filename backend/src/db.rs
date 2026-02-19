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
#[derive(Debug, FromRow)]
pub struct Message {
    pub id: Uuid,
    pub recipient_id: Uuid,
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
        SELECT id, username, password_hash, provider, provider_id, created_at
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
        SELECT id, username, password_hash, provider, provider_id, created_at
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

#[tracing::instrument(skip(pool))]
pub async fn create_message(pool: &PgPool, recipient_id: Uuid, content: &str) -> Result<Uuid> {
    let message_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO messages (id, recipient_id, content, created_at, is_read)
        VALUES ($1, $2, $3, NOW(), false)
        "#,
    )
    .bind(message_id)
    .bind(recipient_id)
    .bind(content)
    .execute(pool)
    .await?;

    Ok(message_id)
}

#[tracing::instrument(skip(pool))]
pub async fn get_user_inbox(pool: &PgPool, recipient_id: Uuid) -> Result<Vec<Message>> {
    let messages = sqlx::query_as::<_, Message>(
        r#"
        SELECT 
            m.id, 
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

