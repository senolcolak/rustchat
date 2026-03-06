-- Migration: Create reactions table for post reactions
-- This enables emoji reactions on posts

-- Create table if not exists (with all columns)
CREATE TABLE IF NOT EXISTS reactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    emoji_name VARCHAR(64) NOT NULL,
    create_at BIGINT NOT NULL DEFAULT extract(epoch from now()) * 1000,
    -- Ensure a user can only react once with the same emoji on a post
    UNIQUE(post_id, user_id, emoji_name)
);

-- Add create_at column if it doesn't exist (for tables created before this migration)
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'reactions' AND column_name = 'create_at'
    ) THEN
        ALTER TABLE reactions ADD COLUMN create_at BIGINT NOT NULL DEFAULT extract(epoch from now()) * 1000;
    END IF;
END $$;

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_reactions_post_id ON reactions(post_id);
CREATE INDEX IF NOT EXISTS idx_reactions_user_id ON reactions(user_id);
CREATE INDEX IF NOT EXISTS idx_reactions_emoji_name ON reactions(emoji_name);
CREATE INDEX IF NOT EXISTS idx_reactions_create_at ON reactions(create_at);

-- Add has_reactions column to posts table for quick lookup
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'posts' AND column_name = 'has_reactions'
    ) THEN
        ALTER TABLE posts ADD COLUMN has_reactions BOOLEAN DEFAULT FALSE;
    END IF;
END $$;

-- Function to update has_reactions on posts
CREATE OR REPLACE FUNCTION update_post_has_reactions()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE posts SET has_reactions = TRUE WHERE id = NEW.post_id;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        -- Check if there are any remaining reactions for this post
        IF NOT EXISTS (SELECT 1 FROM reactions WHERE post_id = OLD.post_id) THEN
            UPDATE posts SET has_reactions = FALSE WHERE id = OLD.post_id;
        END IF;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Trigger to automatically update has_reactions
DROP TRIGGER IF EXISTS trg_update_post_has_reactions ON reactions;
CREATE TRIGGER trg_update_post_has_reactions
    AFTER INSERT OR DELETE ON reactions
    FOR EACH ROW
    EXECUTE FUNCTION update_post_has_reactions();
