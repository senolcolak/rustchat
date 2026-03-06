-- Add auth_provider column to users table for SSO tracking
ALTER TABLE users 
    ADD COLUMN IF NOT EXISTS auth_provider VARCHAR(64),
    ADD COLUMN IF NOT EXISTS auth_provider_id VARCHAR(255);

-- Add index for looking up users by auth provider
CREATE INDEX IF NOT EXISTS idx_users_auth_provider ON users(auth_provider, auth_provider_id);

-- Comment explaining the columns
COMMENT ON COLUMN users.auth_provider IS 'SSO provider used for authentication (e.g., google, github, oidc)';
COMMENT ON COLUMN users.auth_provider_id IS 'External provider user ID (e.g., Google sub claim)';
