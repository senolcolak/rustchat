//! Channel Bookmark model for Mattermost mobile compatibility

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Bookmark types supported
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum BookmarkType {
    Link,
    File,
}

/// Channel bookmark stored in DB
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ChannelBookmark {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub owner_id: Uuid,
    pub file_id: Option<Uuid>,
    pub display_name: String,
    pub sort_order: i64,
    pub link_url: Option<String>,
    pub image_url: Option<String>,
    pub emoji: Option<String>,
    pub bookmark_type: String,
    pub original_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Request to create a channel bookmark
#[derive(Debug, Clone, Deserialize)]
pub struct CreateBookmarkRequest {
    pub display_name: String,
    #[serde(rename = "type")]
    pub bookmark_type: String,
    pub link_url: Option<String>,
    pub image_url: Option<String>,
    pub emoji: Option<String>,
    pub file_id: Option<String>,
}

/// Request to update a channel bookmark
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBookmarkRequest {
    pub display_name: Option<String>,
    pub link_url: Option<String>,
    pub image_url: Option<String>,
    pub emoji: Option<String>,
}
