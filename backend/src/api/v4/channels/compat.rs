use axum::{
    extract::{Path, State},
    Json,
};

use super::MmAuthUser;
use crate::api::AppState;
use crate::error::ApiResult;

/// POST /api/v4/channels/group/search
pub(super) async fn search_group_channels(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Json(_query): Json<serde_json::Value>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

/// PUT /api/v4/channels/{channel_id}/scheme
pub(super) async fn update_channel_scheme(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_channel_id): Path<String>,
    Json(_patch): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"status": "OK"})))
}

/// GET /api/v4/channels/{channel_id}/members_minus_group_members
pub(super) async fn get_channel_members_minus_group_members(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_channel_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

/// GET /api/v4/channels/{channel_id}/member_counts_by_group
pub(super) async fn get_channel_member_counts_by_group(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_channel_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

/// GET /api/v4/channels/{channel_id}/moderations
pub(super) async fn get_channel_moderations(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_channel_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

/// PUT /api/v4/channels/{channel_id}/moderations/patch
pub(super) async fn patch_channel_moderations(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_channel_id): Path<String>,
    Json(_patch): Json<serde_json::Value>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

/// GET /api/v4/channels/{channel_id}/common_teams
pub(super) async fn get_channel_common_teams(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_channel_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

/// GET /api/v4/channels/{channel_id}/groups
pub(super) async fn get_channel_groups(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_channel_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

/// GET /api/v4/channels/{channel_id}/bookmarks
pub(super) async fn get_channel_bookmarks(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_channel_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

/// POST /api/v4/channels/{channel_id}/bookmarks
pub(super) async fn create_channel_bookmark(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_channel_id): Path<String>,
    Json(_bookmark): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"status": "OK"})))
}

/// PATCH /api/v4/channels/{channel_id}/bookmarks/{bookmark_id}
pub(super) async fn patch_channel_bookmark(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path((_channel_id, _bookmark_id)): Path<(String, String)>,
    Json(_patch): Json<serde_json::Value>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"status": "OK"})))
}

/// POST /api/v4/channels/{channel_id}/bookmarks/{bookmark_id}/sort_order
pub(super) async fn update_channel_bookmark_sort_order(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path((_channel_id, _bookmark_id)): Path<(String, String)>,
    Json(_order): Json<serde_json::Value>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    Ok(Json(vec![]))
}

/// DELETE /api/v4/channels/{channel_id}/bookmarks/{bookmark_id}
pub(super) async fn delete_channel_bookmark(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path((_channel_id, _bookmark_id)): Path<(String, String)>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({"status": "OK"})))
}

/// GET /api/v4/channels/{channel_id}/access_control/attributes
pub(super) async fn get_channel_access_control_attributes(
    State(_state): State<AppState>,
    _auth: MmAuthUser,
    Path(_channel_id): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({})))
}
