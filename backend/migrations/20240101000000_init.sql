-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Users Table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT, -- Nullable if using only OAuth
    provider TEXT NOT NULL DEFAULT 'local', -- 'local', 'github', etc.
    provider_id TEXT, -- User ID from provider
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add unique constraint on provider + provider_id for upsert
CREATE UNIQUE INDEX users_provider_provider_id_idx ON users(provider, provider_id);

-- Anonymous Messages Table (P2P)
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    recipient_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    -- CRITICAL: Sender ID is strictly NOT stored here to ensure anonymity
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_read BOOLEAN NOT NULL DEFAULT FALSE
);

-- Broadcasts Table (Public)
CREATE TABLE broadcasts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    sender_id UUID REFERENCES users(id) ON DELETE SET NULL, 
    content TEXT NOT NULL,
    is_anonymous BOOLEAN NOT NULL DEFAULT FALSE, -- Toggle for anonymous broadcasts
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
