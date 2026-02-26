//! OAuth token exchange endpoint
//!
//! Provides secure token exchange using one-time codes instead of
//! returning tokens directly in URLs.

use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};

use super::AppState;
use crate::auth::create_token;
use crate::error::{ApiResult, AppError};
use crate::services::oauth_token_exchange::{exchange_code, ExchangeError};

/// Request to exchange a code for a token
#[derive(Debug, Deserialize)]
pub struct ExchangeRequest {
    code: String,
}

/// Response containing the JWT token
#[derive(Debug, Serialize)]
pub struct ExchangeResponse {
    token: String,
    token_type: String,
    expires_in: u64,
}

/// Build OAuth exchange routes
pub fn router() -> Router<AppState> {
    Router::new().route("/oauth2/exchange", post(exchange_token))
}

/// Exchange a one-time code for a JWT token
async fn exchange_token(
    State(state): State<AppState>,
    Json(input): Json<ExchangeRequest>,
) -> ApiResult<Json<ExchangeResponse>> {
    // Validate code length to prevent unnecessary Redis calls
    if input.code.len() < 10 {
        return Err(AppError::BadRequest("Invalid exchange code".to_string()));
    }

    // Exchange the code for user data
    let payload = match exchange_code(&state.redis, &input.code).await {
        Ok(payload) => payload,
        Err(ExchangeError::InvalidCode) => {
            return Err(AppError::BadRequest(
                "Invalid or already used exchange code".to_string()
            ));
        }
        Err(ExchangeError::CodeExpired) => {
            return Err(AppError::BadRequest(
                "Exchange code has expired".to_string()
            ));
        }
        Err(ExchangeError::Internal(msg)) => {
            tracing::error!("Exchange code error: {}", msg);
            return Err(AppError::Internal(
                "Failed to process exchange code".to_string()
            ));
        }
    };

    // Generate JWT token
    let token = create_token(
        payload.user_id,
        &payload.email,
        &payload.role,
        payload.org_id,
        &state.jwt_secret,
        state.jwt_expiry_hours,
    ).map_err(|e| AppError::Internal(format!("Failed to create token: {}", e)))?;

    tracing::info!(
        user_id = %payload.user_id,
        email = %payload.email,
        "OAuth token exchanged successfully"
    );

    Ok(Json(ExchangeResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_expiry_hours * 3600,
    }))
}
