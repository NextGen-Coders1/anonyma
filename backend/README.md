# Anonyma Backend

The high-performance core of Anonyma, built with Rust.

## Architecture

- **Web Framework**: Axum - Fast, ergonomic, and modular.
- **Database Wrapper**: SQLx - Compile-time verified SQL queries.
- **Authentication**: Authkestra - Modern auth flow for Axum.
- **Logging**: Tracing - Structured, asynchronous diagnostics.

## Prerequisites

- Rust (Latest Stable)
- PostgreSQL 14+
- sqlx-cli (optional, for manual migrations)

## Getting Started

1. **Environment Setup**:
   ```bash
   cp .env.example .env
   ```
   Fill in your PostgreSQL URL and GitHub OAuth credentials.

2. **Run Migrations**:
   The application runs migrations automatically on startup.

3. **Development Mode**:
   ```bash
   cargo run
   ```

## API Overview

| Endpoint | Method | Description |
| :--- | :--- | :--- |
| /api/me | GET | Fetch current agent profile |
| /api/me | POST | Update username/bio |
| /api/messages | POST | Send an anonymous message |
| /api/messages/inbox | GET | Fetch incoming messages |
| /api/messages/:id/react | POST | React to a message |
| /api/broadcasts | GET | List public broadcasts |
| /api/broadcasts/:id/view | POST | Track broadcast view |

## Testing

```bash
cargo test
```
