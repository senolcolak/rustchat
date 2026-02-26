//! OAuth secure token exchange service
//!
//! Provides one-time code exchange for OAuth callbacks to prevent
//! token leakage via browser history, logs, and referrers.

use std::time::Duration;

use deadpool_redis::redis::AsyncCommands;
use uuid::Uuid;

use crate::error::AppError;

const OAUTH_CODE_PREFIX: &str = "rustchat:oauth:code:";
const OAUTH_CODE_TTL_SECONDS: u64 = 60; // 1 minute - codes are short-lived

/// OAuth exchange code payload stored in Redis
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExchangeCodePayload {
    pub user_id: Uuid,
    pub email: String,
    pub role: String,
    pub org_id: Option<Uuid>,
    pub created_at: i64,
}

/// Generate a secure one-time exchange code
pub fn generate_exchange_code() -> String {
    // Use URL-safe base64 encoding of random bytes
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use rand::Rng;
    
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Store token data and return an exchange code
pub async fn create_exchange_code(
    redis: &deadpool_redis::Pool,
    user_id: Uuid,
    email: String,
    role: String,
    org_id: Option<Uuid>,
) -> Result<String, AppError> {
    let code = generate_exchange_code();
    let key = format!("{}{}", OAUTH_CODE_PREFIX, code);
    
    let payload = ExchangeCodePayload {
        user_id,
        email,
        role,
        org_id,
        created_at: chrono::Utc::now().timestamp(),
    };
    
    let serialized = serde_json::to_string(&payload)
        .map_err(|e| AppError::Internal(format!("Failed to serialize exchange code: {}", e)))?;
    
    let mut conn = redis
        .get()
        .await
        .map_err(|e| AppError::Internal(format!("Redis connection failed: {}", e)))?;
    
    // Store with TTL
    let _: () = conn
        .set_ex(&key, serialized, OAUTH_CODE_TTL_SECONDS)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to store exchange code: {}", e)))?;
    
    Ok(code)
}

/// Exchange a code for user data (one-time use)
pub async fn exchange_code(
    redis: &deadpool_redis::Pool,
    code: &str,
) -> Result<ExchangeCodePayload, ExchangeError> {
    let key = format!("{}{}", OAUTH_CODE_PREFIX, code);
    
    let mut conn = redis
        .get()
        .await
        .map_err(|e| ExchangeError::Internal(format!("Redis connection failed: {}", e)))?;
    
    // Get and delete atomically using a Lua script
    let lua_script = r#"
        local value = redis.call('get', KEYS[1])
        if value then
            redis.call('del', KEYS[1])
        end
        return value
    "#;
    
    let result: Option<String> = redis::cmd("EVAL")
        .arg(lua_script)
        .arg(1)
        .arg(&key)
        .query_async(&mut conn)
        .await
        .map_err(|e| ExchangeError::Internal(format!("Redis error: {}", e)))?;
    
    let stored = result.ok_or(ExchangeError::InvalidCode)?;
    
    let payload: ExchangeCodePayload = serde_json::from_str(&stored)
        .map_err(|e| ExchangeError::Internal(format!("Failed to deserialize payload: {}", e)))?;
    
    // Verify code hasn't expired (additional safety check)
    let age = chrono::Utc::now().timestamp() - payload.created_at;
    if age > OAUTH_CODE_TTL_SECONDS as i64 {
        return Err(ExchangeError::CodeExpired);
    }
    
    Ok(payload)
}

/// Errors that can occur during code exchange
#[derive(Debug, Clone)]
pub enum ExchangeError {
    InvalidCode,
    CodeExpired,
    Internal(String),
}

impl std::fmt::Display for ExchangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExchangeError::InvalidCode => write!(f, "Invalid or used exchange code"),
            ExchangeError::CodeExpired => write!(f, "Exchange code has expired"),
            ExchangeError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ExchangeError {}

impl From<ExchangeError> for AppError {
    fn from(err: ExchangeError) -> Self {
        match err {
            ExchangeError::InvalidCode | ExchangeError::CodeExpired => {
                AppError::BadRequest(err.to_string())
            }
            ExchangeError::Internal(msg) => AppError::Internal(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_exchange_code() {
        let code1 = generate_exchange_code();
        let code2 = generate_exchange_code();
        
        // Codes should be different
        assert_ne!(code1, code2);
        
        // Codes should be URL-safe (no special chars)
        assert!(code1.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
        
        // Codes should be reasonable length (base64 of 32 bytes)
        assert_eq!(code1.len(), 43);
    }
}
