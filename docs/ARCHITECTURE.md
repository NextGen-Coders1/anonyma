# System Architecture

## Overview

Anonyma employs a monorepo structure integrating a high-performance Rust backend with a React (Vite) frontend, designed for speed, type safety, and maximum user privacy.

## Technology Stack

| Component | Technology | Description |
|-----------|------------|-------------|
| **Backend** | Rust + Axum | High-performance, memory-safe API server. |
| **Auth** | Authkestra | OAuth2 (GitHub) and secure session management. |
| **Database** | PostgreSQL | Relational persistence managed via SQLx. |
| **Frontend** | React (Vite) | Modern web interface with Tailwind CSS and shadcn/ui. |

## Database Schema

```mermaid
erDiagram
    USERS {
        UUID id PK
        TEXT username
        TEXT provider
        TEXT provider_id
        TEXT bio
        TEXT avatar_url
        TIMESTAMP created_at
    }
    MESSAGES {
        UUID id PK
        UUID recipient_id FK
        TEXT content
        TIMESTAMP created_at
        BOOLEAN is_read
    }
    MESSAGE_REACTIONS {
        UUID id PK
        UUID message_id FK
        UUID user_id FK
        TEXT emoji
        TIMESTAMP created_at
    }
    BROADCASTS {
        UUID id PK
        UUID sender_id FK
        TEXT content
        BOOLEAN is_anonymous
        TIMESTAMP created_at
    }
    BROADCAST_VIEWS {
        UUID id PK
        UUID broadcast_id FK
        UUID user_id FK
        TIMESTAMP viewed_at
    }

    USERS ||--o{ MESSAGES : "receives"
    USERS ||--o{ BROADCASTS : "posts"
    USERS ||--o{ MESSAGE_REACTIONS : "reacts"
    USERS ||--o{ BROADCAST_VIEWS : "views"
    MESSAGES ||--o{ MESSAGE_REACTIONS : "has"
    BROADCASTS ||--o{ BROADCAST_VIEWS : "has"
```

## Security Design

### Anonymity First
The core design principle is that the Sender ID of a P2P message is never recorded in the messages table. This ensures cryptographic-level anonymity for the sender, as there is no database link between the message and its origin.

### Authentication
The system supports both local credentials and GitHub OAuth 2.0. Secure, HTTP-only session cookies are used to maintain agent state without exposing tokens to the client-side JavaScript.

### Real-time View Tracking
Broadcast views are tracked once per agent using an IntersectionObserver on the frontend, ensuring that Read Receipts are accurate and non-intrusive.

## CI/CD and Maintenance

- **Backend**: cargo clippy and cargo test.
- **Frontend**: eslint and vitest.
- **Database**: Versioned migrations via SQLx.
