//! Channel Bookmark model
//!
//! Bookmarks are saved links or files in channels.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A bookmark in a channel
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ChannelBookmark {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub owner_id: Uuid,
    pub r#type: String,
    pub display_name: Option<String>,
    pub link_url: Option<String>,
    pub file_id: Option<Uuid>,
    pub emoji: Option<String>,
    pub sort_order: i32,
    pub image_url: Option<String>,
    pub create_at: i64,
    pub update_at: i64,
    pub delete_at: i64,
}

/// DTO for creating a bookmark
#[derive(Debug, Clone, Deserialize)]
pub struct CreateBookmarkRequest {
    pub channel_id: String,
    pub r#type: String,
    pub display_name: Option<String>,
    pub link_url: Option<String>,
    pub file_id: Option<String>,
    pub emoji: Option<String>,
    pub sort_order: Option<i32>,
    pub image_url: Option<String>,
}

/// DTO for updating a bookmark
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBookmarkRequest {
    pub display_name: Option<String>,
    pub link_url: Option<String>,
    pub emoji: Option<String>,
    pub sort_order: Option<i32>,
    pub image_url: Option<String>,
}

/// DTO for reordering bookmarks
#[derive(Debug, Clone, Deserialize)]
pub struct ReorderBookmarkRequest {
    pub sort_order: i32,
}

/// Bookmark response (Mattermost compatible)
#[derive(Debug, Clone, Serialize)]
pub struct BookmarkResponse {
    pub id: String,
    pub channel_id: String,
    pub owner_id: String,
    pub r#type: String,
    pub display_name: Option<String>,
    pub link_url: Option<String>,
    pub file_id: Option<String>,
    pub emoji: Option<String>,
    pub sort_order: i32,
    pub image_url: Option<String>,
    pub create_at: i64,
    pub update_at: i64,
}

impl From<ChannelBookmark> for BookmarkResponse {
    fn from(b: ChannelBookmark) -> Self {
        Self {
            id: crate::mattermost_compat::id::encode_mm_id(b.id),
            channel_id: crate::mattermost_compat::id::encode_mm_id(b.channel_id),
            owner_id: crate::mattermost_compat::id::encode_mm_id(b.owner_id),
            r#type: b.r#type,
            display_name: b.display_name,
            link_url: b.link_url,
            file_id: b.file_id.map(crate::mattermost_compat::id::encode_mm_id),
            emoji: b.emoji,
            sort_order: b.sort_order,
            image_url: b.image_url,
            create_at: b.create_at,
            update_at: b.update_at,
        }
    }
}
