-- Migration: Add notification settings to server configuration
-- This adds push and email notification configuration options
-- All settings are enabled by default for backward compatibility

-- Update existing server_config with new notification settings
-- The JSONB fields will be automatically merged with defaults by the application

-- Add comment documenting the new fields
COMMENT ON TABLE server_config IS 'Server configuration including site, auth, integrations, compliance, and email settings. Added notification settings: send_push_notifications (default true), send_email_notifications (default true), enable_email_batching (default false).';

-- Ensure default server config exists with notification settings
INSERT INTO server_config (
    id,
    site,
    authentication,
    integrations,
    compliance,
    email,
    experimental,
    updated_at
) VALUES (
    'default',
    '{
        "site_name": "RustChat",
        "send_push_notifications": true
    }'::jsonb,
    '{}'::jsonb,
    '{}'::jsonb,
    '{}'::jsonb,
    '{
        "send_email_notifications": true,
        "enable_email_batching": false,
        "email_batching_interval": 900,
        "smtp_port": 587,
        "smtp_security": "starttls",
        "from_name": "RustChat",
        "email_notification_content": "full"
    }'::jsonb,
    '{}'::jsonb,
    NOW()
)
ON CONFLICT (id) DO UPDATE SET
    site = jsonb_set(
        COALESCE(server_config.site, '{}'::jsonb),
        '{send_push_notifications}',
        COALESCE(server_config.site->'send_push_notifications', 'true'::jsonb),
        true
    ),
    email = jsonb_set(
        jsonb_set(
            jsonb_set(
                COALESCE(server_config.email, '{}'::jsonb),
                '{send_email_notifications}',
                COALESCE(server_config.email->'send_email_notifications', 'true'::jsonb),
                true
            ),
            '{enable_email_batching}',
            COALESCE(server_config.email->'enable_email_batching', 'false'::jsonb),
            true
        ),
        '{email_batching_interval}',
        COALESCE(server_config.email->'email_batching_interval', '900'::jsonb),
        true
    ),
    updated_at = NOW();
