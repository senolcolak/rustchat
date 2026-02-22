-- Migration: Create channel_bookmarks table
-- This enables saving bookmarks in channels

CREATE TABLE IF NOT EXISTS channel_bookmarks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- Bookmark types: 'link', 'file'
    type VARCHAR(16) NOT NULL DEFAULT 'link',
    display_name VARCHAR(128),
    -- For link bookmarks
    link_url TEXT,
    -- For file bookmarks
    file_id UUID REFERENCES files(id) ON DELETE SET NULL,
    -- Optional emoji/icon
    emoji VARCHAR(64),
    -- Sorting order
    sort_order INTEGER NOT NULL DEFAULT 0,
    -- Image preview URL (for link bookmarks)
    image_url TEXT,
    -- Timestamps
    create_at BIGINT NOT NULL DEFAULT extract(epoch from now()) * 1000,
    update_at BIGINT NOT NULL DEFAULT extract(epoch from now()) * 1000,
    delete_at BIGINT DEFAULT 0
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_channel_bookmarks_channel_id ON channel_bookmarks(channel_id);
CREATE INDEX IF NOT EXISTS idx_channel_bookmarks_owner_id ON channel_bookmarks(owner_id);
CREATE INDEX IF NOT EXISTS idx_channel_bookmarks_sort_order ON channel_bookmarks(sort_order);
CREATE INDEX IF NOT EXISTS idx_channel_bookmarks_delete_at ON channel_bookmarks(delete_at);

-- Ensure unique sort order per channel (optional, for strict ordering)
-- CREATE UNIQUE INDEX idx_channel_bookmarks_channel_sort ON channel_bookmarks(channel_id, sort_order) WHERE delete_at = 0;
