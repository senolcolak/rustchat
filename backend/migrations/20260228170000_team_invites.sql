-- Team invite IDs and one-time invite tokens

-- Stable invite ID on teams (used by /api/v4/teams/invite/{invite_id})
ALTER TABLE teams
    ADD COLUMN IF NOT EXISTS invite_id VARCHAR(64);

UPDATE teams
SET invite_id = replace(gen_random_uuid()::text, '-', '')
WHERE invite_id IS NULL OR invite_id = '';

ALTER TABLE teams
    ALTER COLUMN invite_id SET DEFAULT replace(gen_random_uuid()::text, '-', ''),
    ALTER COLUMN invite_id SET NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_teams_invite_id ON teams(invite_id);

-- One-time invite tokens (used by /api/v4/teams/members/invite?token=...)
CREATE TABLE IF NOT EXISTS team_invite_tokens (
    token VARCHAR(64) PRIMARY KEY,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_team_invite_tokens_team_id ON team_invite_tokens(team_id);
CREATE INDEX IF NOT EXISTS idx_team_invite_tokens_expires
    ON team_invite_tokens(expires_at)
    WHERE used_at IS NULL;
