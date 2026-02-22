-- Migration: Create scheduled_posts table
-- This enables scheduling posts for future delivery

CREATE TABLE IF NOT EXISTS scheduled_posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    -- For threaded replies
    root_id UUID REFERENCES posts(id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    file_ids TEXT[],
    -- When to send
    scheduled_at BIGINT NOT NULL,
    -- Processing status
    processed BOOLEAN NOT NULL DEFAULT FALSE,
    processed_at BIGINT,
    -- Error handling
    error_code VARCHAR(64),
    error_message TEXT,
    -- Timestamps
    create_at BIGINT NOT NULL DEFAULT extract(epoch from now()) * 1000,
    update_at BIGINT NOT NULL DEFAULT extract(epoch from now()) * 1000
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_scheduled_posts_user_id ON scheduled_posts(user_id);
CREATE INDEX IF NOT EXISTS idx_scheduled_posts_channel_id ON scheduled_posts(channel_id);
CREATE INDEX IF NOT EXISTS idx_scheduled_posts_scheduled_at ON scheduled_posts(scheduled_at);
CREATE INDEX IF NOT EXISTS idx_scheduled_posts_processed ON scheduled_posts(processed, scheduled_at);

-- Index for finding posts that need to be sent
CREATE INDEX IF NOT EXISTS idx_scheduled_posts_pending ON scheduled_posts(scheduled_at, processed) 
WHERE processed = FALSE;
