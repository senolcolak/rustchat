//! Reactions API
//!
//! Handles emoji reactions on posts.

use axum::{
    extract::{Path, State},
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use uuid::Uuid;

use crate::api::AppState;
use crate::api::v4::extractors::MmAuthUser;
use crate::error::{ApiResult, AppError};
use crate::mattermost_compat::id::{encode_mm_id, parse_mm_or_uuid};
use crate::realtime::{WsBroadcast, WsEnvelope};

/// Build reactions routes
pub fn router() -> Router<AppState> {
    Router::new()
        // GET /api/v4/posts/{post_id}/reactions
        .route("/posts/{post_id}/reactions", get(get_post_reactions))
        // POST /api/v4/reactions
        .route("/reactions", post(add_reaction))
        // DELETE /api/v4/users/{user_id}/posts/{post_id}/reactions/{emoji_name}
        .route("/users/{user_id}/posts/{post_id}/reactions/{emoji_name}", delete(delete_reaction))
}

/// Get all reactions for a post
/// GET /api/v4/posts/{post_id}/reactions
async fn get_post_reactions(
    State(state): State<AppState>,
    _auth: MmAuthUser,
    Path(post_id): Path<String>,
) -> ApiResult<Json<Vec<ReactionResponse>>> {
    let post_uuid = parse_mm_or_uuid(&post_id)
        .ok_or_else(|| AppError::BadRequest("Invalid post ID".to_string()))?;

    let reactions: Vec<Reaction> = sqlx::query_as(
        r#"
        SELECT id, post_id, user_id, emoji_name, create_at
        FROM reactions
        WHERE post_id = $1
        ORDER BY create_at ASC
        "#,
    )
    .bind(post_uuid)
    .fetch_all(&state.db)
    .await?;

    let responses: Vec<ReactionResponse> = reactions.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}

/// Add a reaction to a post
/// POST /api/v4/reactions
async fn add_reaction(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Json(body): Json<CreateReactionRequest>,
) -> ApiResult<Json<ReactionResponse>> {
    let post_uuid = parse_mm_or_uuid(&body.post_id)
        .ok_or_else(|| AppError::BadRequest("Invalid post ID".to_string()))?;

    // Check if post exists
    let post_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM posts WHERE id = $1)")
        .bind(post_uuid)
        .fetch_one(&state.db)
        .await?;

    if !post_exists {
        return Err(AppError::NotFound("Post not found".to_string()));
    }

    // Insert reaction (will fail with unique constraint if already exists)
    let reaction: Reaction = sqlx::query_as(
        r#"
        INSERT INTO reactions (post_id, user_id, emoji_name)
        VALUES ($1, $2, $3)
        ON CONFLICT (post_id, user_id, emoji_name) DO UPDATE
        SET emoji_name = $3
        RETURNING id, post_id, user_id, emoji_name, create_at
        "#,
    )
    .bind(post_uuid)
    .bind(auth.user_id)
    .bind(&body.emoji_name)
    .fetch_one(&state.db)
    .await?;

    // Broadcast reaction_added event
    broadcast_reaction_event(
        &state,
        "reaction_added",
        &reaction,
    )
    .await;

    Ok(Json(reaction.into()))
}

/// Delete a reaction
/// DELETE /api/v4/users/{user_id}/posts/{post_id}/reactions/{emoji_name}
pub async fn delete_reaction(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path((user_id, post_id, emoji_name)): Path<(String, String, String)>,
) -> ApiResult<Json<serde_json::Value>> {
    let post_uuid = parse_mm_or_uuid(&post_id)
        .ok_or_else(|| AppError::BadRequest("Invalid post ID".to_string()))?;
    
    // Parse user_id - can be "me" or a UUID
    let user_uuid = if user_id == "me" {
        auth.user_id
    } else {
        parse_mm_or_uuid(&user_id)
            .ok_or_else(|| AppError::BadRequest("Invalid user ID".to_string()))?
    };

    // Users can only delete their own reactions (unless admin)
    if user_uuid != auth.user_id && auth.role != "admin" {
        return Err(AppError::Forbidden(
            "Can only delete your own reactions".to_string(),
        ));
    }

    // Delete the reaction
    let result = sqlx::query(
        r#"
        DELETE FROM reactions
        WHERE post_id = $1 AND user_id = $2 AND emoji_name = $3
        RETURNING id, post_id, user_id, emoji_name, create_at
        "#,
    )
    .bind(post_uuid)
    .bind(user_uuid)
    .bind(&emoji_name)
    .fetch_optional(&state.db)
    .await?;

    if let Some(row) = result {
        let reaction = Reaction {
            id: row.try_get("id")?,
            post_id: row.try_get("post_id")?,
            user_id: row.try_get("user_id")?,
            emoji_name: row.try_get("emoji_name")?,
            create_at: row.try_get("create_at")?,
        };
        
        // Broadcast reaction_removed event
        broadcast_reaction_event(
            &state,
            "reaction_removed",
            &reaction,
        )
        .await;
    }

    Ok(Json(serde_json::json!({"status": "OK"})))
}

/// Broadcast reaction event via WebSocket
async fn broadcast_reaction_event(
    state: &AppState,
    event_type: &str,
    reaction: &Reaction,
) {
    let event = WsEnvelope {
        msg_type: "event".to_string(),
        event: event_type.to_string(),
        seq: None,
        channel_id: None,
        data: serde_json::json!({
            "reaction": {
                "user_id": encode_mm_id(reaction.user_id),
                "post_id": encode_mm_id(reaction.post_id),
                "emoji_name": reaction.emoji_name,
                "create_at": reaction.create_at,
            }
        }),
        broadcast: Some(WsBroadcast {
            user_id: Some(reaction.user_id),
            channel_id: None,
            team_id: None,
            exclude_user_id: None,
        }),
    };

    state.ws_hub.broadcast(event).await;
}

// Model types
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Reaction {
    pub id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub emoji_name: String,
    pub create_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateReactionRequest {
    pub post_id: String,
    pub emoji_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReactionResponse {
    pub user_id: String,
    pub post_id: String,
    pub emoji_name: String,
    pub create_at: i64,
}

impl From<Reaction> for ReactionResponse {
    fn from(r: Reaction) -> Self {
        Self {
            user_id: encode_mm_id(r.user_id),
            post_id: encode_mm_id(r.post_id),
            emoji_name: r.emoji_name,
            create_at: r.create_at,
        }
    }
}
