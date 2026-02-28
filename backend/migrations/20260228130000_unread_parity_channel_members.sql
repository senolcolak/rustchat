-- Migration: unread parity columns for channel members
-- Adds Mattermost-compatible unread counters used by websocket/API contracts.

ALTER TABLE channel_members
    ADD COLUMN IF NOT EXISTS msg_count BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS mention_count BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS msg_count_root BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS mention_count_root BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS urgent_mention_count BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS manually_unread BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS last_update_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

CREATE INDEX IF NOT EXISTS idx_channel_members_user_channel_unread
    ON channel_members (user_id, channel_id);
