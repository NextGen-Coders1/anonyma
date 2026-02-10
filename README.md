# Anonyma

Anonyma is a secure, peer-to-peer anonymous messaging platform designed for privacy and reliability, built with Rust and Next.js.

## Documentation

- [System Architecture](docs/ARCHITECTURE.md) - Detailed technical specifications, database schema, and technology stack.

## Development Setup

The project requires Docker, Rust (latest stable), and Node.js (v18+).

### Database

Initialize the PostgreSQL database container:

```bash
docker-compose up -d
```

### Backend

Configure and start the API server:

```bash
cd backend
cp .env.example .env
# Edit .env with valid GITHUB_CLIENT_ID and GITHUB_CLIENT_SECRET
cargo run
```

The server listens on `http://127.0.0.1:3000`.

### Frontend

Install dependencies and start the web interface:

```bash
cd frontend
npm install
npm run dev
```

The application is accessible at `http://localhost:8080`.

## Testing & Maintenance

Refers to standard tooling for validation:

- **Backend**: Run `cargo test` for unit and integration tests.
- **Frontend**: Run `npm run lint` for static analysis.
- **Database**: Manage schema changes with `sqlx migrate run`.

## License

MIT
