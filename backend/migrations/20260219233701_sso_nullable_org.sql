-- Make org_id nullable in sso_configs for single-tenant RustChat deployments
-- This allows SSO configs without organization context

ALTER TABLE sso_configs ALTER COLUMN org_id DROP NOT NULL;

-- Update the unique constraint comment
COMMENT ON TABLE sso_configs IS 'SSO/OAuth2/OIDC provider configurations. Supports github, google, and generic oidc providers. provider_key is used in URL paths and must be URL-safe. org_id is optional for single-tenant deployments.';
