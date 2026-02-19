-- Add notify_props column to users table for notification preferences
-- This stores user notification settings like push, email, call sounds, etc.
-- Required for Mattermost mobile app compatibility

ALTER TABLE users ADD COLUMN IF NOT EXISTS notify_props JSONB DEFAULT '{}';

-- Create index for efficient querying of notification preferences
CREATE INDEX IF NOT EXISTS idx_users_notify_props ON users USING GIN (notify_props);
