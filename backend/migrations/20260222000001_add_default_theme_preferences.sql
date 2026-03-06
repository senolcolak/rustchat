-- Migration: Add default theme preferences for all users
-- This fixes the mobile app theme settings page

-- Insert default RustChat theme for users who don't have a theme preference
INSERT INTO mattermost_preferences (user_id, category, name, value)
SELECT 
    u.id as user_id,
    'theme' as category,
    '' as name,
    jsonb_build_object(
        'type', 'RustChat',
        'sidebarBg', '#1A1A18',
        'sidebarText', '#ffffff',
        'sidebarUnreadText', '#ffffff',
        'sidebarTextHoverBg', '#25262a',
        'sidebarTextActiveBorder', '#00FFC2',
        'sidebarTextActiveColor', '#ffffff',
        'sidebarHeaderBg', '#121213',
        'sidebarHeaderTextColor', '#ffffff',
        'sidebarTeamBarBg', '#121213',
        'onlineIndicator', '#00FFC2',
        'awayIndicator', '#ffbc1f',
        'dndIndicator', '#d24b4e',
        'mentionBg', '#ffffff',
        'mentionColor', '#1A1A18',
        'centerChannelBg', '#121213',
        'centerChannelColor', '#e3e4e8',
        'newMessageSeparator', '#00FFC2',
        'linkColor', '#00FFC2',
        'buttonBg', '#00FFC2',
        'buttonColor', '#121213',
        'errorTextColor', '#da6c6e',
        'mentionHighlightBg', '#0d6e6e',
        'mentionHighlightLink', '#a4f4f4',
        'codeTheme', 'monokai'
    )::text as value
FROM users u
WHERE NOT EXISTS (
    SELECT 1 FROM mattermost_preferences mp 
    WHERE mp.user_id = u.id AND mp.category = 'theme'
)
ON CONFLICT (user_id, category, name) DO NOTHING;

-- Add display preferences for timezone, clock format, and CRT
INSERT INTO mattermost_preferences (user_id, category, name, value)
SELECT 
    u.id as user_id,
    'display_settings' as category,
    'use_military_time' as name,
    'false' as value
FROM users u
WHERE NOT EXISTS (
    SELECT 1 FROM mattermost_preferences mp 
    WHERE mp.user_id = u.id AND mp.category = 'display_settings' AND mp.name = 'use_military_time'
)
ON CONFLICT (user_id, category, name) DO NOTHING;

INSERT INTO mattermost_preferences (user_id, category, name, value)
SELECT 
    u.id as user_id,
    'display_settings' as category,
    'timezone' as name,
    'Auto' as value
FROM users u
WHERE NOT EXISTS (
    SELECT 1 FROM mattermost_preferences mp 
    WHERE mp.user_id = u.id AND mp.category = 'display_settings' AND mp.name = 'timezone'
)
ON CONFLICT (user_id, category, name) DO NOTHING;

INSERT INTO mattermost_preferences (user_id, category, name, value)
SELECT 
    u.id as user_id,
    'display_settings' as category,
    'collapsed_reply_threads' as name,
    'on' as value
FROM users u
WHERE NOT EXISTS (
    SELECT 1 FROM mattermost_preferences mp 
    WHERE mp.user_id = u.id AND mp.category = 'display_settings' AND mp.name = 'collapsed_reply_threads'
)
ON CONFLICT (user_id, category, name) DO NOTHING;

-- Add default notification preferences
INSERT INTO mattermost_preferences (user_id, category, name, value)
SELECT 
    u.id as user_id,
    'notifications' as category,
    'desktop' as name,
    'mention' as value
FROM users u
WHERE NOT EXISTS (
    SELECT 1 FROM mattermost_preferences mp 
    WHERE mp.user_id = u.id AND mp.category = 'notifications' AND mp.name = 'desktop'
)
ON CONFLICT (user_id, category, name) DO NOTHING;

INSERT INTO mattermost_preferences (user_id, category, name, value)
SELECT 
    u.id as user_id,
    'notifications' as category,
    'push' as name,
    'mention' as value
FROM users u
WHERE NOT EXISTS (
    SELECT 1 FROM mattermost_preferences mp 
    WHERE mp.user_id = u.id AND mp.category = 'notifications' AND mp.name = 'push'
)
ON CONFLICT (user_id, category, name) DO NOTHING;

INSERT INTO mattermost_preferences (user_id, category, name, value)
SELECT 
    u.id as user_id,
    'notifications' as category,
    'email' as name,
    'true' as value
FROM users u
WHERE NOT EXISTS (
    SELECT 1 FROM mattermost_preferences mp 
    WHERE mp.user_id = u.id AND mp.category = 'notifications' AND mp.name = 'email'
)
ON CONFLICT (user_id, category, name) DO NOTHING;

-- Add sidebar settings preferences
INSERT INTO mattermost_preferences (user_id, category, name, value)
SELECT 
    u.id as user_id,
    'sidebar_settings' as category,
    'show_unread_section' as name,
    'true' as value
FROM users u
WHERE NOT EXISTS (
    SELECT 1 FROM mattermost_preferences mp 
    WHERE mp.user_id = u.id AND mp.category = 'sidebar_settings' AND mp.name = 'show_unread_section'
)
ON CONFLICT (user_id, category, name) DO NOTHING;

-- Create index on mattermost_preferences for faster lookups
CREATE INDEX IF NOT EXISTS idx_mattermost_preferences_category ON mattermost_preferences(category);
CREATE INDEX IF NOT EXISTS idx_mattermost_preferences_user_category ON mattermost_preferences(user_id, category);
