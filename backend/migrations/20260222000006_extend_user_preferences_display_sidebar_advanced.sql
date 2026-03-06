-- Extend user_preferences table with Display, Sidebar, and Advanced settings
-- S7: Display settings, S6: Sidebar settings, S5: Advanced settings

-- Display settings (S7)
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS collapsed_reply_threads BOOLEAN DEFAULT false;
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS use_military_time BOOLEAN DEFAULT false;
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS teammate_name_display VARCHAR(20) DEFAULT 'username'; -- 'username', 'nickname', 'full_name'
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS availability_status_visible BOOLEAN DEFAULT true;
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS show_last_active_time BOOLEAN DEFAULT true;
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS timezone VARCHAR(64) DEFAULT 'auto';
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS link_previews_enabled BOOLEAN DEFAULT true;
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS image_previews_enabled BOOLEAN DEFAULT true;
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS click_to_reply BOOLEAN DEFAULT true;
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS channel_display_mode VARCHAR(20) DEFAULT 'full'; -- 'full', 'centered'
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS quick_reactions_enabled BOOLEAN DEFAULT true;
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS emoji_picker_enabled BOOLEAN DEFAULT true;
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS language VARCHAR(10) DEFAULT 'en';

-- Sidebar settings (S6)
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS group_unread_channels VARCHAR(30) DEFAULT 'never'; -- 'never', 'only_for_favorites', 'always'
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS limit_visible_dms_gms VARCHAR(10) DEFAULT 'all'; -- 'all', '10', '20', '40'

-- Advanced settings (S5)
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS send_on_ctrl_enter BOOLEAN DEFAULT false;
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS enable_post_formatting BOOLEAN DEFAULT true;
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS enable_join_leave_messages BOOLEAN DEFAULT true;
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS enable_performance_debugging BOOLEAN DEFAULT false;
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS unread_scroll_position VARCHAR(20) DEFAULT 'last'; -- 'start', 'last', 'end'
ALTER TABLE user_preferences ADD COLUMN IF NOT EXISTS sync_drafts BOOLEAN DEFAULT true;
