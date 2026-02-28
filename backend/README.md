# Anonyma Backend

High-performance Rust backend powering the Anonyma anonymous messaging platform.

## Architecture

The backend is built on modern Rust technologies optimized for performance, safety, and maintainability:

- **Web Framework**: Axum 0.8 for fast, ergonomic request handling
- **Database**: PostgreSQL with SQLx for compile-time verified queries
- **Authentication**: Authkestra for OAuth2 and session management
- **Real-time**: Server-Sent Events for live updates
- **Logging**: Tracing for structured, asynchronous diagnostics
- **Session Storage**: In-memory MemoryStore

## Prerequisites

- Rust (latest stable version)
- PostgreSQL 14 or higher
- sqlx-cli (optional, for manual migrations)

## Getting Started

### Environment Configuration

Create a `.env` file from the example:

```bash
cp .env.example .env
```

Configure the following variables:

```env
# Database connection
DATABASE_URL=postgresql://user:password@localhost/anonyma

# Server configuration
HOST=localhost
PORT=3000

# GitHub OAuth
GITHUB_CLIENT_ID=your_github_client_id
GITHUB_CLIENT_SECRET=your_github_client_secret
GITHUB_REDIRECT_URI=http://localhost:3000/auth/github/callback

# Session security (not used in current implementation)
SESSION_SECRET=your_random_secret_key_here
```

### Database Setup

The application automatically runs migrations on startup. For manual migration management:

```bash
# Create database
createdb anonyma

# Run migrations manually
sqlx migrate run
```

### Development

Start the development server:

```bash
cargo run
```

The server will be available at http://localhost:3000

### Production Build

Build an optimized release binary:

```bash
cargo build --release
./target/release/anonyma_backend
```

## API Reference

### Authentication Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/auth/register` | POST | Register new user with username and password |
| `/auth/login` | POST | Authenticate with username and password |
| `/auth/github` | GET | Initiate GitHub OAuth flow |
| `/logout` | GET | Terminate current session |

**Register/Login Request Body:**
```json
{
  "username": "string",
  "password": "string"
}
```

**Password Requirements:**
- Minimum 6 characters
- No maximum length

### User Management

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/me` | GET | Retrieve current user profile |
| `/api/me` | POST | Update profile information |
| `/api/users` | GET | List all users excluding current user |
| `/api/users/{id}/block` | POST | Block specified user |
| `/api/users/{id}/unblock` | POST | Unblock specified user |
| `/api/users/blocked` | GET | Retrieve list of blocked users |

### Messaging

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/messages` | POST | Send anonymous message |
| `/api/messages/inbox` | GET | Retrieve inbox messages |
| `/api/messages/search` | GET | Full-text message search |
| `/api/messages/{id}/react` | POST | Add emoji reaction to message |
| `/api/messages/{id}/edit` | POST | Edit message content |
| `/api/messages/{id}/delete` | DELETE | Soft delete message |
| `/api/messages/{id}/pin` | POST | Toggle message pin status |

### Conversations

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/conversations` | GET | List all conversation threads |
| `/api/conversations/{thread_id}` | GET | Retrieve messages in thread |
| `/api/conversations/{thread_id}/delete` | DELETE | Delete entire conversation |
| `/api/conversations/{thread_id}/pin` | POST | Toggle thread pin status |
| `/api/conversations/{thread_id}/typing` | POST | Send typing indicator |

### Broadcasts

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/broadcasts` | GET | List public broadcasts |
| `/api/broadcasts` | POST | Create new broadcast |
| `/api/broadcasts/{id}/view` | POST | Track broadcast view |
| `/api/broadcasts/{id}/comments` | GET | Retrieve broadcast comments |
| `/api/broadcasts/{id}/comments` | POST | Create comment on broadcast |
| `/api/broadcasts/comments/{id}/react` | POST | React to comment |
| `/api/broadcasts/comments/{id}/delete` | DELETE | Delete comment |

### User Preferences

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/preferences` | GET | Retrieve user preferences |
| `/api/preferences` | POST | Update user preferences |

### Real-time Communication

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/sse` | GET | Server-Sent Events stream |

**SSE Event Types:**
- `new_message` - New message received
- `message_reaction` - Reaction added to message
- `typing` - User typing in conversation
- `read_receipt` - Message read by recipient
- `new_broadcast` - New broadcast posted

## Database Schema

### Core Tables

**users** - User accounts and authentication
```sql
id UUID PRIMARY KEY
username TEXT UNIQUE NOT NULL
password_hash TEXT
provider TEXT NOT NULL DEFAULT 'local'
provider_id TEXT
bio TEXT
avatar_url TEXT
created_at TIMESTAMPTZ NOT NULL
updated_at TIMESTAMPTZ NOT NULL
```

**messages** - Peer-to-peer messages
```sql
id UUID PRIMARY KEY
thread_id UUID NOT NULL
recipient_id UUID NOT NULL
sender_id UUID
content TEXT NOT NULL
created_at TIMESTAMPTZ NOT NULL
is_read BOOLEAN NOT NULL DEFAULT FALSE
read_at TIMESTAMPTZ
edited_at TIMESTAMPTZ
deleted_at TIMESTAMPTZ
deleted_by UUID
```

**broadcasts** - Public broadcasts
```sql
id UUID PRIMARY KEY
sender_id UUID
content TEXT NOT NULL
is_anonymous BOOLEAN NOT NULL DEFAULT FALSE
created_at TIMESTAMPTZ NOT NULL
```

### Supporting Tables

- `user_preferences` - User settings and preferences
- `message_reactions` - Emoji reactions on messages
- `message_edits` - Message modification history
- `pinned_messages` - User-pinned messages
- `pinned_threads` - User-pinned conversations
- `typing_indicators` - Real-time typing state
- `user_blocks` - Blocked user relationships
- `broadcast_views` - Broadcast view tracking
- `broadcast_comments` - Comments on broadcasts
- `broadcast_comment_reactions` - Reactions on comments

Complete schema available in `migrations/20240101000000_complete_schema.sql`.

## Development

### Running Tests

Execute the test suite:

```bash
cargo test
```

### Code Quality

Run the linter:

```bash
cargo clippy
```

Format code:

```bash
cargo fmt
```

### Database Migrations

Create a new migration:

```bash
sqlx migrate add migration_name
```

Apply migrations:

```bash
sqlx migrate run
```

Revert the last migration:

```bash
sqlx migrate revert
```

## Background Tasks

The server runs periodic maintenance tasks:

- **Typing Indicator Cleanup**: Removes stale typing indicators every 10 seconds
  - Deletes indicators older than 10 seconds
  - Runs automatically on server startup
  - Prevents database bloat

## Security

### Authentication
- Argon2 password hashing for secure credential storage
- HTTP-only session cookies prevent XSS attacks
- GitHub OAuth 2.0 integration via Authkestra
- Session validation on all protected routes

### Privacy
- Sender identities stored but never exposed to recipients
- Soft deletion preserves audit trails
- User blocking prevents unwanted communication
- Thread-based routing maintains conversation flow

### Database
- Prepared statements prevent SQL injection
- Compile-time query verification with SQLx
- Connection pooling for efficient resource usage
- Foreign key constraints ensure data integrity

## Performance

The backend is optimized for high performance:

- Asynchronous runtime via Tokio for high concurrency
- Database connection pooling
- Compile-time query verification eliminates runtime overhead
- Zero-copy request parsing with Axum
- Efficient JSON serialization with Serde

## Debugging

Enable different logging levels:

```bash
# Debug level
RUST_LOG=debug cargo run

# Trace level
RUST_LOG=trace cargo run

# SQL query logging
RUST_LOG=sqlx=debug cargo run
```

## Dependencies

Key dependencies include:

- `axum` - Web framework
- `sqlx` - Database toolkit with PostgreSQL support
- `tokio` - Async runtime
- `serde` - Serialization framework
- `authkestra` - Authentication and session management
- `tower-http` - HTTP middleware (CORS, tracing)
- `tower-cookies` - Cookie management
- `tracing` - Structured logging
- `argon2` - Password hashing

See `Cargo.toml` for the complete dependency list.

## Deployment

### Production Checklist

1. Use production PostgreSQL instance
2. Enable HTTPS with valid certificates
3. Configure CORS for production domain
4. Set up reverse proxy (nginx or Caddy)
5. Configure database connection pooling
6. Implement monitoring and alerting
7. Establish backup and recovery procedures
8. Consider Redis for session storage

### Environment Variables

```env
DATABASE_URL=postgresql://user:pass@host/db
HOST=0.0.0.0
PORT=3000
GITHUB_CLIENT_ID=<production-client-id>
GITHUB_CLIENT_SECRET=<production-secret>
GITHUB_REDIRECT_URI=https://yourdomain.com/auth/github/callback
```

### Docker Deployment

```bash
docker build -t anonyma-backend .
docker run -p 3000:3000 --env-file .env anonyma-backend
```

## Project Structure

```
backend/
├── src/
│   ├── main.rs          # Server initialization and routing
│   ├── api.rs           # API endpoint handlers
│   ├── auth.rs          # Authentication logic
│   ├── db.rs            # Database operations
│   ├── config.rs        # Configuration management
│   └── state.rs         # Application state
├── migrations/          # Database migrations
│   └── 20240101000000_complete_schema.sql
├── Cargo.toml          # Rust dependencies
└── .env.example        # Environment template
```

## Contributing

Contributions should adhere to the following guidelines:

1. Follow Rust style conventions
2. Run `cargo fmt` before committing
3. Ensure `cargo clippy` passes without warnings
4. Include tests for new functionality
5. Update relevant documentation

## License

This project is licensed under the MIT License.
