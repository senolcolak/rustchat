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

-- Normalize legacy installs where channel_bookmarks already existed with
-- timestamp columns (created_at/updated_at/deleted_at) and bookmark_type.
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = 'channel_bookmarks' AND column_name = 'bookmark_type'
    ) AND NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = 'channel_bookmarks' AND column_name = 'type'
    ) THEN
        ALTER TABLE channel_bookmarks RENAME COLUMN bookmark_type TO type;
    END IF;
END
$$;

ALTER TABLE IF EXISTS channel_bookmarks
    ADD COLUMN IF NOT EXISTS type VARCHAR(16) NOT NULL DEFAULT 'link',
    ADD COLUMN IF NOT EXISTS create_at BIGINT,
    ADD COLUMN IF NOT EXISTS update_at BIGINT,
    ADD COLUMN IF NOT EXISTS delete_at BIGINT DEFAULT 0;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = 'channel_bookmarks' AND column_name = 'created_at'
    ) THEN
        UPDATE channel_bookmarks
        SET create_at = (extract(epoch FROM created_at) * 1000)::BIGINT
        WHERE create_at IS NULL;
    END IF;
END
$$;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = 'channel_bookmarks' AND column_name = 'updated_at'
    ) THEN
        UPDATE channel_bookmarks
        SET update_at = (extract(epoch FROM updated_at) * 1000)::BIGINT
        WHERE update_at IS NULL;
    END IF;
END
$$;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = 'channel_bookmarks' AND column_name = 'deleted_at'
    ) THEN
        UPDATE channel_bookmarks
        SET delete_at = (extract(epoch FROM deleted_at) * 1000)::BIGINT
        WHERE deleted_at IS NOT NULL AND (delete_at IS NULL OR delete_at = 0);
    END IF;
END
$$;

UPDATE channel_bookmarks
SET
    type = COALESCE(type, 'link'),
    create_at = COALESCE(create_at, (extract(epoch FROM now()) * 1000)::BIGINT),
    update_at = COALESCE(update_at, create_at, (extract(epoch FROM now()) * 1000)::BIGINT),
    delete_at = COALESCE(delete_at, 0);

ALTER TABLE IF EXISTS channel_bookmarks
    ALTER COLUMN sort_order TYPE INTEGER USING sort_order::INTEGER,
    ALTER COLUMN create_at SET DEFAULT (extract(epoch FROM now()) * 1000)::BIGINT,
    ALTER COLUMN update_at SET DEFAULT (extract(epoch FROM now()) * 1000)::BIGINT,
    ALTER COLUMN delete_at SET DEFAULT 0,
    ALTER COLUMN create_at SET NOT NULL,
    ALTER COLUMN update_at SET NOT NULL,
    ALTER COLUMN delete_at SET NOT NULL;

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_channel_bookmarks_channel_id ON channel_bookmarks(channel_id);
CREATE INDEX IF NOT EXISTS idx_channel_bookmarks_owner_id ON channel_bookmarks(owner_id);
CREATE INDEX IF NOT EXISTS idx_channel_bookmarks_sort_order ON channel_bookmarks(sort_order);
CREATE INDEX IF NOT EXISTS idx_channel_bookmarks_delete_at ON channel_bookmarks(delete_at);

-- Ensure unique sort order per channel (optional, for strict ordering)
-- CREATE UNIQUE INDEX idx_channel_bookmarks_channel_sort ON channel_bookmarks(channel_id, sort_order) WHERE delete_at = 0;
