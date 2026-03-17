//! Rate limiting middleware

use crate::error::ApiError;
use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use deadpool_redis::Pool as RedisPool;

/// Rate limiting middleware layer
/// Extracts auth context and checks rate limit
pub async fn rate_limit_middleware(
    State(redis): State<RedisPool>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    // Extract entity_id and tier from request
    // This is a simplified version - in production, you'd extract from auth headers
    // For now, we'll skip rate limiting in middleware and do it in individual handlers

    // TODO: Extract auth from request headers manually
    // For Phase 1, rate limiting is checked in individual handlers

    Ok(next.run(request).await)
}
