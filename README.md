# Anonyma

Anonyma is a secure, privacy-first anonymous messaging and broadcast platform. It enables users to communicate without revealing their identities while providing modern social features including message reactions, comments, typing indicators, and real-time updates.

## Features

### Core Messaging
- Anonymous peer-to-peer messaging with thread-based conversations
- Real-time message delivery via Server-Sent Events
- Message reactions with enhanced emoji picker
- Read receipts for sent messages
- Full-text message search across conversations

### Enhanced Messaging
- Message editing with complete edit history
- Soft deletion of messages and threads
- Message and conversation pinning
- Real-time typing indicators
- Auto-saving message drafts with restoration

### Broadcasts
- Global broadcast board visible to all users
- Optional anonymous posting
- Threaded comment system with reactions
- Real-time view tracking and statistics

### Social Features
- Customizable user profiles
- User blocking and management
- Active user discovery

### User Experience
- Dark and light theme support with persistence
- Audio notifications for new messages
- Browser desktop notifications
- Comprehensive emoji library with search
- Smooth animations and transitions
- Fully responsive design

## Technology Stack

### Backend
- Rust with Axum 0.8 web framework
- PostgreSQL database with SQLx
- Authkestra for authentication (GitHub OAuth and local credentials)
- Server-Sent Events for real-time updates
- Structured logging with Tracing

### Frontend
- React 18 with Vite build tool
- TypeScript for type safety
- Tailwind CSS for styling
- shadcn/ui component library
- TanStack Query for state management
- Framer Motion for animations
- Lucide React for icons

## Documentation

- [Complete Implementation Status](./COMPLETE_IMPLEMENTATION.md) - Detailed feature documentation
- [Architecture and Design](./docs/ARCHITECTURE.md) - System architecture overview
- [Backend Development](./backend/README.md) - Backend API documentation
- [Frontend Development](./frontend/README.md) - Frontend development guide

## Quick Start

### Prerequisites
- Rust (latest stable version)
- Node.js 18 or higher
- PostgreSQL 14 or higher
- Docker (optional, for containerized database)

### Database Setup

Using Docker:
```bash
docker-compose up -d
```

Using local PostgreSQL:
```bash
createdb anonyma
```

### Backend Setup

```bash
cd backend
cp .env.example .env
# Configure DATABASE_URL and OAuth credentials in .env
cargo build
cargo run
```

The backend server will start on http://localhost:3000

### Frontend Setup

```bash
cd frontend
npm install
npm run dev
```

The frontend application will be accessible at http://localhost:8080

### Initial User Creation

1. Navigate to http://localhost:8080
2. Choose authentication method:
   - "Authenticate via GitHub" for OAuth login
   - "Authenticate with Password" for local credentials
3. For local authentication, enter a username and password (minimum 6 characters)
4. Begin using the application

## Testing

### Multi-User Testing

To test messaging between multiple users, use separate browser profiles:

**Chrome or Edge:**
1. Create multiple browser profiles via the profile menu
2. Open the application in each profile with different user accounts

**Firefox:**
1. Use Firefox profiles or container tabs
2. Alternatively, use different browsers simultaneously

### Running Tests

Backend tests:
```bash
cd backend
cargo test
cargo clippy
```

Frontend tests:
```bash
cd frontend
npm run test
npm run lint
```

## API Overview

### Authentication
- `POST /auth/register` - Register new user with username and password
- `POST /auth/login` - Authenticate with credentials
- `GET /auth/github` - Initiate GitHub OAuth flow
- `GET /logout` - End current session

### User Management
- `GET /api/me` - Retrieve current user profile
- `POST /api/me` - Update profile information
- `GET /api/users` - List all active users
- `POST /api/users/{id}/block` - Block specified user
- `POST /api/users/{id}/unblock` - Unblock specified user
- `GET /api/users/blocked` - Retrieve blocked users list

### Messaging
- `POST /api/messages` - Send new message
- `GET /api/messages/inbox` - Retrieve inbox messages
- `GET /api/messages/search` - Search messages by content
- `GET /api/conversations` - List all conversations
- `POST /api/messages/{id}/react` - Add emoji reaction
- `POST /api/messages/{id}/edit` - Edit message content
- `DELETE /api/messages/{id}/delete` - Delete message
- `DELETE /api/conversations/{thread_id}/delete` - Delete conversation
- `POST /api/messages/{id}/pin` - Pin or unpin message
- `POST /api/conversations/{thread_id}/pin` - Pin or unpin conversation
- `POST /api/conversations/{thread_id}/typing` - Send typing indicator

### Broadcasts
- `GET /api/broadcasts` - List all broadcasts
- `POST /api/broadcasts` - Create new broadcast
- `POST /api/broadcasts/{id}/view` - Track broadcast view
- `GET /api/broadcasts/{id}/comments` - Retrieve comments
- `POST /api/broadcasts/{id}/comments` - Create comment
- `POST /api/broadcasts/comments/{id}/react` - React to comment
- `DELETE /api/broadcasts/comments/{id}/delete` - Delete comment

### Preferences
- `GET /api/preferences` - Retrieve user preferences
- `POST /api/preferences` - Update user preferences

### Real-time
- `GET /api/sse` - Server-Sent Events stream for live updates

## Security and Privacy

### Anonymity Design
The system is architected with privacy as a core principle. Sender identities are stored server-side for reply routing but are never exposed to message recipients. Recipients see only "Anonymous Agent" for incoming messages, ensuring complete sender anonymity.

### Data Protection
- HTTP-only session cookies prevent cross-site scripting attacks
- Argon2 password hashing provides secure credential storage
- Soft deletion preserves audit trails while removing content from view
- User blocking prevents unwanted communication
- Secure session management via Authkestra

## Database Schema

The application uses a single comprehensive migration file containing 13 tables. See [ARCHITECTURE.md](./docs/ARCHITECTURE.md) for the complete Entity-Relationship diagram and detailed schema documentation.

### Core Tables
- `users` - User accounts and authentication
- `messages` - Peer-to-peer messages with threading
- `broadcasts` - Public broadcast messages

### Feature Tables
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

## Feature Status

All core features are fully implemented and functional:

- Anonymous peer-to-peer messaging with threading
- Full-text message search
- Message editing with history tracking
- Message and thread deletion
- Message and conversation pinning
- Real-time typing indicators
- Read receipts for message tracking
- Auto-saving message drafts
- Enhanced emoji reactions
- User blocking and management
- Public broadcasts with view tracking
- Threaded comment system
- Dark and light theme support
- Audio and browser notifications
- Real-time updates via Server-Sent Events

## Contributing

Contributions are welcome. Please ensure all tests pass and code follows the established style guidelines before submitting pull requests.

## License

This project is licensed under the MIT License. See the LICENSE file for details.

## Acknowledgments

This project is built with:
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [shadcn/ui](https://ui.shadcn.com/) - UI component library
- [Authkestra](https://github.com/authkestra/authkestra) - Authentication framework
- [Lucide](https://lucide.dev/) - Icon library
