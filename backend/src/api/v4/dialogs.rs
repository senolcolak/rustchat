use crate::api::AppState;
use crate::error::ApiResult;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/actions/dialogs/open", post(open_dialog))
        .route("/actions/dialogs/submit", post(submit_dialog))
        .route("/actions/dialogs/lookup", post(lookup_dialog))
}

async fn open_dialog(
    State(_state): State<AppState>,
    _auth: crate::api::v4::extractors::MmAuthUser,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    Ok(crate::api::v4::mm_not_implemented(
        "api.actions.dialogs.open.not_implemented.app_error",
        "Interactive dialogs are not implemented.",
        "POST /api/v4/actions/dialogs/open is not supported in this server.",
    ))
}

async fn submit_dialog(
    State(_state): State<AppState>,
    _auth: crate::api::v4::extractors::MmAuthUser,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    Ok(crate::api::v4::mm_not_implemented(
        "api.actions.dialogs.submit.not_implemented.app_error",
        "Interactive dialogs are not implemented.",
        "POST /api/v4/actions/dialogs/submit is not supported in this server.",
    ))
}

async fn lookup_dialog(
    State(_state): State<AppState>,
    _auth: crate::api::v4::extractors::MmAuthUser,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    Ok(crate::api::v4::mm_not_implemented(
        "api.actions.dialogs.lookup.not_implemented.app_error",
        "Interactive dialogs are not implemented.",
        "POST /api/v4/actions/dialogs/lookup is not supported in this server.",
    ))
}
