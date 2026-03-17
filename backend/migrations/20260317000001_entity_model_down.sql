-- Rollback entity model migration

-- Drop indexes
DROP INDEX IF EXISTS idx_users_entity_type;
DROP INDEX IF EXISTS idx_users_api_key;

-- Drop constraints (constraints are dropped automatically with columns, but being explicit)
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_entity_type_check;
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_rate_limit_tier_check;

-- Drop columns
ALTER TABLE users DROP COLUMN IF EXISTS rate_limit_tier;
ALTER TABLE users DROP COLUMN IF EXISTS entity_metadata;
ALTER TABLE users DROP COLUMN IF EXISTS api_key_hash;
ALTER TABLE users DROP COLUMN IF EXISTS entity_type;
