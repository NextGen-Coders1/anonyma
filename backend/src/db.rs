use sqlx::{types::time::OffsetDateTime, PgPool, Result};
use uuid::Uuid;

pub async fn init_db(database_url: &str) -> Result<PgPool> {
    let pool = PgPool::connect(database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

// ===== Models =====

#[derive(Debug)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub provider: String,
    pub provider_id: Option<String>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug)]
pub struct Message {
    pub id: Uuid,
    pub recipient_id: Uuid,
    pub content: String,
    pub created_at: OffsetDateTime,
    pub is_read: bool,
}

#[derive(Debug)]
pub struct Broadcast {
    pub id: Uuid,
    pub sender_id: Option<Uuid>,
    pub sender_username: Option<String>,
    pub content: String,
    pub is_anonymous: bool,
    pub created_at: OffsetDateTime,
}

// ===== User Operations =====

pub async fn upsert_user(
    pool: &PgPool,
    username: &str,
    provider: &str,
    provider_id: &str,
) -> Result<User> {
    // Try to find existing user first
    let existing = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, provider, provider_id, created_at
        FROM users
        WHERE provider = $1 AND provider_id = $2
        "#,
        provider,
        provider_id
    )
    .fetch_optional(pool)
    .await?;

    if let Some(user) = existing {
        // Update existing user's username if changed
        let updated = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET username = $1
            WHERE id = $2
            RETURNING id, username, provider, provider_id, created_at
            "#,
            username,
            user.id
        )
        .fetch_one(pool)
        .await?;
        Ok(updated)
    } else {
        // Insert new user
        let new_user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, username, provider, provider_id, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            RETURNING id, username, provider, provider_id, created_at
            "#,
            Uuid::new_v4(),
            username,
            provider,
            provider_id
        )
        .fetch_one(pool)
        .await?;
        Ok(new_user)
    }
}

pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> Result<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, provider, provider_id, created_at
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_all_users(pool: &PgPool) -> Result<Vec<User>> {
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, provider, provider_id, created_at
        FROM users
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(users)
}

// ===== Message Operations =====

pub async fn create_message(pool: &PgPool, recipient_id: Uuid, content: &str) -> Result<Uuid> {
    let message_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO messages (id, recipient_id, content, created_at, is_read)
        VALUES ($1, $2, $3, NOW(), false)
        "#,
        message_id,
        recipient_id,
        content
    )
    .execute(pool)
    .await?;

    Ok(message_id)
}

pub async fn get_user_inbox(pool: &PgPool, recipient_id: Uuid) -> Result<Vec<Message>> {
    let messages = sqlx::query_as!(
        Message,
        r#"
        SELECT id, recipient_id, content, created_at, is_read
        FROM messages
        WHERE recipient_id = $1
        ORDER BY created_at DESC
        "#,
        recipient_id
    )
    .fetch_all(pool)
    .await?;

    Ok(messages)
}

// ===== Broadcast Operations =====

pub async fn create_broadcast(
    pool: &PgPool,
    sender_id: Option<Uuid>,
    content: &str,
    is_anonymous: bool,
) -> Result<Uuid> {
    let broadcast_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO broadcasts (id, sender_id, content, is_anonymous, created_at)
        VALUES ($1, $2, $3, $4, NOW())
        "#,
        broadcast_id,
        sender_id,
        content,
        is_anonymous
    )
    .execute(pool)
    .await?;

    Ok(broadcast_id)
}

pub async fn get_broadcasts(pool: &PgPool, limit: i64) -> Result<Vec<Broadcast>> {
    let broadcasts = sqlx::query_as!(
        Broadcast,
        r#"
        SELECT 
            b.id, 
            b.sender_id, 
            u.username as sender_username,
            b.content, 
            b.is_anonymous, 
            b.created_at
        FROM broadcasts b
        LEFT JOIN users u ON b.sender_id = u.id
        ORDER BY b.created_at DESC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(broadcasts)
}
