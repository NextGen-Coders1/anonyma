# Anonyma

Anonyma is a secure, privacy-first anonymous messaging and broadcast platform. It allows agents to communicate without revealing their identities while providing modern social features like message reactions and view tracking.

## Features

- **Anonymous P2P Messaging**: Send encrypted-at-rest messages to other agents without revealing your identity.
- **Global Broadcast Board**: Post messages to the entire network anonymously or as your agent persona.
- **Agent Profiles**: Customize your shadow identity with a codename and bio.
- **Message Reactions**: React to incoming anonymous messages with emojis.
- **Read Receipts (Broadcasts)**: Real-time view tracking for broadcasts using Intersection Observer API.
- **Secure Authentication**: Multi-provider login support (GitHub OAuth and Local Credentials).

## Tech Stack

- **Backend**: Rust with Axum and SQLx.
- **Frontend**: React (Vite), TypeScript, Tailwind CSS, and shadcn/ui.
- **Database**: PostgreSQL managed with SQLx migrations.
- **State Management**: TanStack Query (React Query).

## Documentation

- [Architecture and Design](./docs/ARCHITECTURE.md)
- [Backend Development](./backend/README.md)
- [Frontend Development](./frontend/README.md)

## Quick Start

### 1. Database
Spin up the PostgreSQL instance:
```bash
docker-compose up -d
```

### 2. Backend
```bash
cd backend
cp .env.example .env
# Configure GITHUB_CLIENT_ID and GITHUB_CLIENT_SECRET in .env
cargo run
```

### 3. Frontend
```bash
cd frontend
npm install
npm run dev
```

The application will be accessible at http://localhost:8080.

## Security Policy

Anonyma is built with a focus on anonymity. Sender IDs for P2P messages are never stored in the database, ensuring that even with full database access, an agent's identity remains protected.

## License

MIT
