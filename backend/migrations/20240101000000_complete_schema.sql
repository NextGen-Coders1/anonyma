-- Complete Database Schema for Anonyma
-- This migration includes all tables and features

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================================
-- USERS & AUTHENTICATION
-- ============================================================================

-- Users Table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT, -- Nullable if using only OAuth
    provider TEXT NOT NULL DEFAULT 'local', -- 'local', 'github', etc.
    provider_id TEXT, -- User ID from provider
    bio TEXT,
    avatar_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add unique constraint on provider + provider_id for upsert
CREATE UNIQUE INDEX users_provider_provider_id_idx ON users(provider, provider_id);

-- User Preferences
CREATE TABLE user_preferences (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    theme TEXT DEFAULT 'dark',
    notification_sound BOOLEAN DEFAULT true,
    browser_notifications BOOLEAN DEFAULT true,
    show_read_receipts BOOLEAN DEFAULT true,
    show_typing_indicators BOOLEAN DEFAULT true,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- MESSAGING (P2P)
-- ============================================================================

-- Anonymous Messages Table (P2P with threading support)
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    thread_id UUID NOT NULL DEFAULT gen_random_uuid(),
    recipient_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    sender_id UUID REFERENCES users(id) ON DELETE SET NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    read_at TIMESTAMPTZ,
    edited_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,
    deleted_by UUID REFERENCES users(id)
);

-- Indexes for efficient queries
CREATE INDEX messages_thread_id_idx ON messages(thread_id);
CREATE INDEX messages_recipient_id_idx ON messages(recipient_id);
CREATE INDEX messages_sender_id_idx ON messages(sender_id);
CREATE INDEX messages_content_search_idx ON messages USING gin(to_tsvector('english', content));

-- Message Reactions
CREATE TABLE message_reactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    emoji TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(message_id, user_id)
);

CREATE INDEX message_reactions_message_id_idx ON message_reactions(message_id);

-- Message Edit History
CREATE TABLE message_edits (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    old_content TEXT NOT NULL,
    edited_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    edited_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX message_edits_message_id_idx ON message_edits(message_id);

-- Pinned Messages
CREATE TABLE pinned_messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    pinned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(message_id, user_id)
);

CREATE INDEX pinned_messages_user_id_idx ON pinned_messages(user_id);

-- Pinned Threads
CREATE TABLE pinned_threads (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    thread_id UUID NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    pinned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(thread_id, user_id)
);

CREATE INDEX pinned_threads_user_id_idx ON pinned_threads(user_id);
CREATE INDEX pinned_threads_thread_id_idx ON pinned_threads(thread_id);

-- Typing Indicators (ephemeral)
CREATE TABLE typing_indicators (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    thread_id UUID NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(thread_id, user_id)
);

CREATE INDEX typing_indicators_thread_id_idx ON typing_indicators(thread_id);
CREATE INDEX typing_indicators_started_at_idx ON typing_indicators(started_at);

-- ============================================================================
-- USER BLOCKING
-- ============================================================================

CREATE TABLE user_blocks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    blocker_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    blocked_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(blocker_id, blocked_id)
);

CREATE INDEX user_blocks_blocker_id_idx ON user_blocks(blocker_id);
CREATE INDEX user_blocks_blocked_id_idx ON user_blocks(blocked_id);

-- ============================================================================
-- BROADCASTS (PUBLIC)
-- ============================================================================

-- Broadcasts Table
CREATE TABLE broadcasts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    sender_id UUID REFERENCES users(id) ON DELETE SET NULL, 
    content TEXT NOT NULL,
    is_anonymous BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX broadcasts_created_at_idx ON broadcasts(created_at DESC);
CREATE INDEX broadcasts_sender_id_idx ON broadcasts(sender_id);

-- Broadcast Views (Read Receipts)
CREATE TABLE broadcast_views (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    broadcast_id UUID NOT NULL REFERENCES broadcasts(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    viewed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(broadcast_id, user_id)
);

CREATE INDEX broadcast_views_broadcast_id_idx ON broadcast_views(broadcast_id);
CREATE INDEX broadcast_views_user_id_idx ON broadcast_views(user_id);

-- Broadcast Comments
CREATE TABLE broadcast_comments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    broadcast_id UUID NOT NULL REFERENCES broadcasts(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    parent_comment_id UUID REFERENCES broadcast_comments(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX broadcast_comments_broadcast_id_idx ON broadcast_comments(broadcast_id);
CREATE INDEX broadcast_comments_parent_id_idx ON broadcast_comments(parent_comment_id);
CREATE INDEX broadcast_comments_user_id_idx ON broadcast_comments(user_id);

-- Broadcast Comment Reactions
CREATE TABLE broadcast_comment_reactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    comment_id UUID NOT NULL REFERENCES broadcast_comments(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    emoji TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(comment_id, user_id)
);

CREATE INDEX broadcast_comment_reactions_comment_id_idx ON broadcast_comment_reactions(comment_id);
CREATE INDEX broadcast_comment_reactions_user_id_idx ON broadcast_comment_reactions(user_id);
