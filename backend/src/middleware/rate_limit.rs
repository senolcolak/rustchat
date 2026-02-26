//! Rate limiting middleware for API endpoints
//!
//! Provides IP-based and user-based rate limiting with Redis backend.

use std::net::SocketAddr;
use std::task::{Context, Poll};

use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
};
use deadpool_redis::redis::AsyncCommands;
use tower::{Layer, Service};

use crate::error::AppError;

/// Rate limit configuration for different endpoint categories
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Window size in seconds
    pub window_seconds: u64,
    /// Key prefix for Redis
    pub key_prefix: String,
}

impl RateLimitConfig {
    /// Default configuration for auth endpoints (login, register)
    pub fn auth_default() -> Self {
        Self {
            max_requests: 10,
            window_seconds: 60,
            key_prefix: "ratelimit:auth".to_string(),
        }
    }

    /// Default configuration for WebSocket connections
    pub fn websocket_default() -> Self {
        Self {
            max_requests: 30,
            window_seconds: 60,
            key_prefix: "ratelimit:ws".to_string(),
        }
    }

    /// Default configuration for password reset
    pub fn password_reset_default() -> Self {
        Self {
            max_requests: 5,
            window_seconds: 3600, // 1 hour
            key_prefix: "ratelimit:pwreset".to_string(),
        }
    }
}

/// Result of a rate limit check
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: u32,
    pub reset_at: i64,
    pub total_limit: u32,
}

/// Check if a request should be rate limited
/// 
/// Uses Redis for distributed rate limiting across multiple nodes.
pub async fn check_rate_limit(
    redis: &deadpool_redis::Pool,
    config: &RateLimitConfig,
    key: &str,
) -> Result<RateLimitResult, AppError> {
    let mut conn = redis
        .get()
        .await
        .map_err(|e| AppError::Internal(format!("Redis connection failed: {}", e)))?;

    let redis_key = format!("{}:{}", config.key_prefix, key);
    let window = config.window_seconds as i64;
    let now = chrono::Utc::now().timestamp();
    let window_start = now - window;

    // Use Redis sorted set for sliding window rate limiting
    // Remove entries outside the current window
    let _: () = conn
        .zrembyscore(&redis_key, 0, window_start)
        .await
        .map_err(|e| AppError::Internal(format!("Redis error: {}", e)))?;

    // Count current requests in window
    let current_count: u32 = conn
        .zcard(&redis_key)
        .await
        .map_err(|e| AppError::Internal(format!("Redis error: {}", e)))?;

    if current_count >= config.max_requests {
        // Get the oldest timestamp to calculate reset time
        let oldest: Vec<i64> = conn
            .zrange(&redis_key, 0, 0)
            .await
            .map_err(|e| AppError::Internal(format!("Redis error: {}", e)))?;

        let reset_at = oldest.first().copied().unwrap_or(now) + window;

        return Ok(RateLimitResult {
            allowed: false,
            remaining: 0,
            reset_at,
            total_limit: config.max_requests,
        });
    }

    // Add current request to the window
    let _: () = conn
        .zadd(&redis_key, now, now)
        .await
        .map_err(|e| AppError::Internal(format!("Redis error: {}", e)))?;

    // Set expiry on the key to auto-cleanup
    let _: () = conn
        .expire(&redis_key, window)
        .await
        .map_err(|e| AppError::Internal(format!("Redis error: {}", e)))?;

    Ok(RateLimitResult {
        allowed: true,
        remaining: config.max_requests - current_count - 1,
        reset_at: now + window,
        total_limit: config.max_requests,
    })
}

/// Extract client IP for rate limiting
pub fn extract_client_ip(addr: &SocketAddr, headers: &axum::http::HeaderMap) -> String {
    // Check for X-Forwarded-For header (when behind proxy)
    if let Some(forwarded) = headers
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
    {
        // Take the first IP in the chain (original client)
        let first_ip = forwarded.split(',').next().map(|s| s.trim());
        if let Some(ip) = first_ip {
            if !ip.is_empty() {
                return ip.to_string();
            }
        }
    }

    // Check for X-Real-IP header
    if let Some(real_ip) = headers
        .get("X-Real-IP")
        .and_then(|v| v.to_str().ok())
    {
        return real_ip.to_string();
    }

    // Fall back to direct connection IP
    addr.ip().to_string()
}

/// Rate limiting layer for Tower
#[derive(Debug, Clone)]
pub struct RateLimitLayer {
    config: RateLimitConfig,
}

impl RateLimitLayer {
    pub fn new(config: RateLimitConfig) -> Self {
        Self { config }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            config: self.config.clone(),
        }
    }
}

/// Rate limiting service
#[derive(Debug, Clone)]
pub struct RateLimitService<S> {
    inner: S,
    config: RateLimitConfig,
}

impl<S, B> Service<Request<B>> for RateLimitService<S>
where
    S: Service<Request<B>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let _config = self.config.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Note: Full implementation would check Redis here
            // For now, pass through and let handler-level limits work
            inner.call(req).await
        })
    }
}

/// Rate limit error response
pub fn rate_limit_response(result: &RateLimitResult) -> Response<Body> {
    let retry_after = (result.reset_at - chrono::Utc::now().timestamp()).max(0);
    
    let body = serde_json::json!({
        "error": {
            "code": "RATE_LIMIT_EXCEEDED",
            "message": "Too many requests. Please try again later.",
            "retry_after": retry_after,
        }
    });

    Response::builder()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .header("Content-Type", "application/json")
        .header("X-RateLimit-Limit", result.total_limit.to_string())
        .header("X-RateLimit-Remaining", result.remaining.to_string())
        .header("X-RateLimit-Reset", result.reset_at.to_string())
        .header("Retry-After", retry_after.to_string())
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn test_extract_client_ip_direct() {
        let addr = SocketAddr::from(([192, 168, 1, 1], 12345));
        let headers = HeaderMap::new();
        
        assert_eq!(extract_client_ip(&addr, &headers), "192.168.1.1");
    }

    #[test]
    fn test_extract_client_ip_forwarded() {
        let addr = SocketAddr::from(([192, 168, 1, 1], 12345));
        let mut headers = HeaderMap::new();
        headers.insert("X-Forwarded-For", "10.0.0.1, 10.0.0.2".parse().unwrap());
        
        assert_eq!(extract_client_ip(&addr, &headers), "10.0.0.1");
    }

    #[test]
    fn test_extract_client_ip_real_ip() {
        let addr = SocketAddr::from(([192, 168, 1, 1], 12345));
        let mut headers = HeaderMap::new();
        headers.insert("X-Real-IP", "10.0.0.5".parse().unwrap());
        
        assert_eq!(extract_client_ip(&addr, &headers), "10.0.0.5");
    }
}
