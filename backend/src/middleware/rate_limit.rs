//! Rate limiting middleware
//!
//! ## Architecture
//!
//! Rate limiting in RustChat operates at two levels:
//!
//! ### 1. Entity-Level Rate Limiting (Implemented)
//! Authenticated requests (API keys) are rate limited per entity using Redis-backed
//! atomic counters. This is handled by:
//! - `services/rate_limit.rs` - RateLimitService with Lua scripts
//! - `auth/extractors.rs` - ApiKeyAuth and PolymorphicAuth call RateLimitService
//!
//! Tiers:
//! - HumanStandard: 1k req/hr
//! - AgentHigh: 10k req/hr
//! - ServiceUnlimited: no limit
//! - CIStandard: 5k req/hr
//!
//! ### 2. IP-Based Rate Limiting (Application Layer)
//! Unauthenticated endpoints (login, registration, password reset, WebSocket) are
//! rate limited by IP address using the RateLimitService (Redis-backed sliding window).
//!
//! When `TRUST_PROXY=true`, the client IP is read from the first value in the
//! `X-Forwarded-For` header (set by nginx/Cloudflare). Otherwise the socket address
//! is used directly.

use crate::error::AppError;
use axum::{
    extract::{ConnectInfo, Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;

/// Extract the originating client IP.
/// When the `TRUST_PROXY` environment variable is `"true"`, reads the first
/// value from `X-Forwarded-For` (the client IP). Otherwise uses the socket address.
fn extract_client_ip(addr: &SocketAddr, headers: &HeaderMap) -> String {
    let trust_proxy = std::env::var("TRUST_PROXY")
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if trust_proxy {
        if let Some(xff) = headers.get("x-forwarded-for") {
            if let Ok(s) = xff.to_str() {
                if let Some(ip) = s.split(',').next() {
                    return ip.trim().to_string();
                }
            }
        }
    }

    addr.ip().to_string()
}

/// Rate limit middleware for registration endpoints
pub async fn register_ip_rate_limit(
    State(state): State<crate::api::AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = extract_client_ip(&addr, &headers);
    state.rate_limit.check_register_ip(&ip).await?;
    Ok(next.run(request).await)
}

/// Rate limit middleware for auth endpoints
pub async fn auth_ip_rate_limit(
    State(state): State<crate::api::AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = extract_client_ip(&addr, &headers);
    state.rate_limit.check_auth_ip(&ip).await?;
    Ok(next.run(request).await)
}

/// Rate limit middleware for password reset endpoints
pub async fn password_reset_ip_rate_limit(
    State(state): State<crate::api::AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = extract_client_ip(&addr, &headers);
    state.rate_limit.check_password_reset_ip(&ip).await?;
    Ok(next.run(request).await)
}

/// Rate limit middleware for WebSocket endpoints
pub async fn websocket_ip_rate_limit(
    State(state): State<crate::api::AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = extract_client_ip(&addr, &headers);
    state.rate_limit.check_websocket_ip(&ip).await?;
    Ok(next.run(request).await)
}

// ============================================================================
// Legacy API - Kept for backward compatibility
// ============================================================================
// These types and functions are used by existing code (src/api/auth.rs, src/api/v4/users.rs)
// They are stubs that always allow requests. Real rate limiting is handled by:
// 1. Entity-level: RateLimitService (services/rate_limit.rs)
// 2. IP-level: Reverse proxy (nginx/Cloudflare)

/// Legacy rate limit configuration (stub)
#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    pub window_secs: u64,
    pub max_requests: u64,
}

impl RateLimitConfig {
    /// Create config for auth endpoints (stub)
    pub fn auth_per_minute(max_requests: u32) -> Self {
        Self {
            window_secs: 60,
            max_requests: max_requests as u64,
        }
    }
}

/// Legacy rate limit check result (stub)
#[derive(Debug, Clone, Copy)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: u64,
    pub reset_at: u64,
}

/// Legacy rate limit check function (stub)
///
/// **Note:** This is a stub that always returns allowed=true.
/// Real rate limiting is handled by:
/// - Entity-level: RateLimitService checks in auth extractors
/// - IP-level: Reverse proxy rate limiting
///
/// This function is kept for backward compatibility with existing code
/// in src/api/auth.rs and src/api/v4/users.rs
pub async fn check_rate_limit(
    _redis: &deadpool_redis::Pool,
    config: &RateLimitConfig,
    _key: &str,
) -> Result<RateLimitResult, AppError> {
    // Stub: always allow
    // Real rate limiting is handled by RateLimitService (entity-level)
    // and reverse proxy (IP-level)
    Ok(RateLimitResult {
        allowed: true,
        remaining: config.max_requests,
        reset_at: 0,
    })
}
