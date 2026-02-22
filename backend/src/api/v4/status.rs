//! User Status API
//!
//! Handles user presence status (online, away, dnd, offline)

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::AppState;
use crate::api::v4::extractors::MmAuthUser;
use crate::error::{ApiResult, AppError};
use crate::mattermost_compat::id::{encode_mm_id, parse_mm_or_uuid};
use crate::realtime::{WsBroadcast, WsEnvelope};

/// Build status routes
pub fn router() -> Router<AppState> {
    Router::new()
        // GET /api/v4/users/{user_id}/status
        .route("/users/{user_id}/status", get(get_user_status).put(update_user_status))
        // GET /api/v4/users/me/status
        .route("/users/me/status", get(get_my_status).put(update_my_status))
        // POST /api/v4/users/status/ids
        .route("/users/status/ids", post(get_statuses_by_ids))
}

/// User status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatusResponse {
    pub user_id: String,
    pub status: String, // online, away, dnd, offline
    pub manual: bool,
    pub last_activity_at: i64,
}

/// Update status request
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String, // online, away, dnd, offline
}

/// Bulk status request
#[derive(Debug, Clone, Deserialize)]
pub struct BulkStatusRequest {
    pub user_ids: Vec<String>,
}

/// Get current user's status
/// GET /api/v4/users/me/status
async fn get_my_status(
    State(state): State<AppState>,
    auth: MmAuthUser,
) -> ApiResult<Json<UserStatusResponse>> {
    get_user_status_by_id(&state, auth.user_id).await
}

/// Get user status by ID
/// GET /api/v4/users/{user_id}/status
async fn get_user_status(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> ApiResult<Json<UserStatusResponse>> {
    let user_uuid = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| AppError::BadRequest("Invalid user ID".to_string()))?;
    get_user_status_by_id(&state, user_uuid).await
}

/// Internal: Get user status by UUID
async fn get_user_status_by_id(
    state: &AppState,
    user_id: Uuid,
) -> ApiResult<Json<UserStatusResponse>> {
    let result: (String, bool, Option<chrono::DateTime<Utc>>) = sqlx::query_as(
        r#"
        SELECT presence, COALESCE(presence_manual, false), last_login_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;

    let (presence, manual, last_login) = result;

    Ok(Json(UserStatusResponse {
        user_id: encode_mm_id(user_id),
        status: if presence.is_empty() {
            "offline".to_string()
        } else {
            presence
        },
        manual,
        last_activity_at: last_login.map(|t| t.timestamp_millis()).unwrap_or(0),
    }))
}

/// Update my status
/// PUT /api/v4/users/me/status
async fn update_my_status(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Json(body): Json<UpdateStatusRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    update_user_status_internal(&state, auth.user_id, body.status).await
}

/// Update user status
/// PUT /api/v4/users/{user_id}/status
async fn update_user_status(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(user_id): Path<String>,
    Json(body): Json<UpdateStatusRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let target_user_id = parse_mm_or_uuid(&user_id)
        .ok_or_else(|| AppError::BadRequest("Invalid user ID".to_string()))?;
    
    // Users can only update their own status unless admin
    if target_user_id != auth.user_id && auth.role != "admin" {
        return Err(AppError::Forbidden(
            "Can only update your own status".to_string(),
        ));
    }

    update_user_status_internal(&state, target_user_id, body.status).await
}

/// Internal: Update user status
async fn update_user_status_internal(
    state: &AppState,
    user_id: Uuid,
    status: String,
) -> ApiResult<Json<serde_json::Value>> {
    // Validate status
    let valid_statuses = ["online", "away", "dnd", "offline"];
    if !valid_statuses.contains(&status.as_str()) {
        return Err(AppError::Validation(format!(
            "Invalid status: {}. Must be one of: online, away, dnd, offline",
            status
        )));
    }

    // Determine if this is a manual status
    let manual = status != "online";

    // Update user presence
    sqlx::query(
        r#"
        UPDATE users
        SET presence = $2, presence_manual = $3
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .bind(&status)
    .bind(manual)
    .execute(&state.db)
    .await?;

    // Broadcast status change
    broadcast_status_change(state, user_id, &status).await;

    Ok(Json(serde_json::json!({"status": status})))
}

/// Get statuses for multiple users
/// POST /api/v4/users/status/ids
async fn get_statuses_by_ids(
    State(state): State<AppState>,
    Json(body): Json<BulkStatusRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let user_ids: Vec<Uuid> = body
        .user_ids
        .iter()
        .filter_map(|id| parse_mm_or_uuid(id))
        .collect();

    if user_ids.is_empty() {
        return Ok(Json(serde_json::json!({})));
    }

    let statuses: Vec<(Uuid, String)> = sqlx::query_as(
        r#"
        SELECT id, presence
        FROM users
        WHERE id = ANY($1)
        "#,
    )
    .bind(&user_ids)
    .fetch_all(&state.db)
    .await?;

    // Build response map
    let mut result = serde_json::Map::new();
    for (user_id, status) in statuses {
        result.insert(encode_mm_id(user_id), serde_json::json!(status));
    }

    Ok(Json(serde_json::Value::Object(result)))
}

/// Broadcast status change via WebSocket
async fn broadcast_status_change(state: &AppState, user_id: Uuid, status: &str) {
    let event = WsEnvelope {
        msg_type: "event".to_string(),
        event: "status_change".to_string(),
        seq: None,
        channel_id: None,
        data: serde_json::json!({
            "user_id": encode_mm_id(user_id),
            "status": status,
        }),
        broadcast: Some(WsBroadcast {
            user_id: Some(user_id),
            channel_id: None,
            team_id: None,
            exclude_user_id: None,
        }),
    };

    state.ws_hub.broadcast(event).await;
}
