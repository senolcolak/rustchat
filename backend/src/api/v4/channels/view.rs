use axum::{
    body::Bytes,
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use super::{parse_body, parse_mm_or_uuid, ApiResult, AppState, MmAuthUser};

#[derive(Deserialize)]
struct ViewChannelRequest {
    channel_id: String,
    #[serde(rename = "prev_channel_id")]
    _prev_channel_id: Option<String>,
}

fn parse_view_channel_request(headers: &HeaderMap, body: &Bytes) -> ApiResult<ViewChannelRequest> {
    parse_body(headers, body, "Invalid view body")
}

pub(super) async fn view_channel(
    State(state): State<AppState>,
    auth: MmAuthUser,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<impl IntoResponse> {
    if body.is_empty() {
        return Ok(Json(serde_json::json!({"status": "OK"})));
    }

    let input = match parse_view_channel_request(&headers, &body) {
        Ok(value) => value,
        Err(_) => return Ok(Json(serde_json::json!({"status": "OK"}))),
    };

    if let Some(channel_id) = parse_mm_or_uuid(&input.channel_id) {
        sqlx::query(
            "UPDATE channel_members SET last_viewed_at = NOW() WHERE channel_id = $1 AND user_id = $2",
        )
        .bind(channel_id)
        .bind(auth.user_id)
        .execute(&state.db)
        .await?;

        let broadcast = crate::realtime::WsEnvelope::event(
            crate::realtime::EventType::ChannelViewed,
            serde_json::json!({
                "channel_id": channel_id,
            }),
            Some(channel_id),
        )
        .with_broadcast(crate::realtime::WsBroadcast {
            channel_id: None,
            team_id: None,
            user_id: Some(auth.user_id),
            exclude_user_id: None,
        });
        state.ws_hub.broadcast(broadcast).await;
    }

    Ok(Json(serde_json::json!({"status": "OK"})))
}

pub(super) async fn view_channel_for_user(
    State(state): State<AppState>,
    auth: MmAuthUser,
    Path(user_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<impl IntoResponse> {
    let resolved_user_id = if user_id == "me" {
        auth.user_id
    } else {
        parse_mm_or_uuid(&user_id)
            .ok_or_else(|| crate::error::AppError::BadRequest("Invalid user_id".to_string()))?
    };

    if resolved_user_id != auth.user_id {
        return Err(crate::error::AppError::Forbidden(
            "Cannot view channel for other users".to_string(),
        ));
    }

    if body.is_empty() {
        return Ok(Json(serde_json::json!({"status": "OK"})));
    }

    let input = match parse_view_channel_request(&headers, &body) {
        Ok(value) => value,
        Err(_) => return Ok(Json(serde_json::json!({"status": "OK"}))),
    };

    if let Some(channel_id) = parse_mm_or_uuid(&input.channel_id) {
        sqlx::query(
            "UPDATE channel_members SET last_viewed_at = NOW() WHERE channel_id = $1 AND user_id = $2",
        )
        .bind(channel_id)
        .bind(auth.user_id)
        .execute(&state.db)
        .await?;

        let broadcast = crate::realtime::WsEnvelope::event(
            crate::realtime::EventType::ChannelViewed,
            serde_json::json!({
                "channel_id": channel_id,
            }),
            Some(channel_id),
        )
        .with_broadcast(crate::realtime::WsBroadcast {
            channel_id: None,
            team_id: None,
            user_id: Some(auth.user_id),
            exclude_user_id: None,
        });
        state.ws_hub.broadcast(broadcast).await;
    }

    Ok(Json(serde_json::json!({"status": "OK"})))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn json_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("application/json"),
        );
        headers
    }

    #[test]
    fn parses_view_request_json() {
        let headers = json_headers();
        let body = Bytes::from_static(br#"{"channel_id":"test"}"#);

        let parsed = parse_view_channel_request(&headers, &body).expect("valid request");
        assert_eq!(parsed.channel_id, "test");
    }

    #[test]
    fn rejects_invalid_view_request() {
        let headers = json_headers();
        let body = Bytes::from_static(br#"{"channel":"missing_id"}"#);

        let parsed = parse_view_channel_request(&headers, &body);
        assert!(parsed.is_err());
    }
}
