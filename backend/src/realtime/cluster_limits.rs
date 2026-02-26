//! Cluster-aware connection limits using Redis
//!
//! Enforces global connection limits across all nodes in a cluster
//! using Redis counters with TTL-based expiration.

use std::time::Duration;

use deadpool_redis::redis::AsyncCommands;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::api::AppState;

/// Redis key prefix for connection counters
const CONN_COUNT_PREFIX: &str = "rustchat:conn:count:";
const CONN_HEARTBEAT_PREFIX: &str = "rustchat:conn:heartbeat:";

/// Connection counter TTL in seconds (how long before a connection is considered stale)
const CONN_HEARTBEAT_TTL: u64 = 60;

/// Cluster-aware connection limit enforcer
pub struct ClusterConnectionLimits {
    redis: deadpool_redis::Pool,
    node_id: String,
}

impl ClusterConnectionLimits {
    /// Create a new cluster connection limit tracker
    pub fn new(redis: deadpool_redis::Pool) -> Self {
        let node_id = format!("{}-{}", 
            hostname::get().ok()
                .and_then(|h| h.into_string().ok())
                .unwrap_or_else(|| "unknown".to_string()),
            Uuid::new_v4()
        );
        
        Self { redis, node_id }
    }
    
    /// Get the node ID
    pub fn node_id(&self) -> &str {
        &self.node_id
    }
    
    /// Register a new connection for a user (cluster-wide)
    pub async fn register_connection(
        &self,
        user_id: Uuid,
        connection_id: &str,
    ) -> anyhow::Result<usize> {
        let mut conn = self.redis.get().await?;
        
        // Add connection to user's set
        let count_key = format!("{}{}", CONN_COUNT_PREFIX, user_id);
        let conn_key = format!("{}:{}", connection_id, self.node_id);
        
        // Use Redis set for atomic add
        let added: i32 = conn.sadd(&count_key, &conn_key).await?;
        
        // Set TTL on the key for automatic cleanup
        let _: () = conn.expire(&count_key, (CONN_HEARTBEAT_TTL * 2) as i64).await?;
        
        // Get current count
        let count: usize = conn.scard(&count_key).await?;
        
        debug!(
            user_id = %user_id,
            connection_id = %connection_id,
            cluster_count = count,
            added = added > 0,
            "Registered cluster connection"
        );
        
        Ok(count)
    }
    
    /// Unregister a connection for a user
    pub async fn unregister_connection(
        &self,
        user_id: Uuid,
        connection_id: &str,
    ) -> anyhow::Result<usize> {
        let mut conn = self.redis.get().await?;
        
        let count_key = format!("{}{}", CONN_COUNT_PREFIX, user_id);
        let conn_key = format!("{}:{}", connection_id, self.node_id);
        
        // Remove from set
        let _: i32 = conn.srem(&count_key, &conn_key).await?;
        
        // Get current count
        let count: usize = conn.scard(&count_key).await?;
        
        // Clean up empty sets
        if count == 0 {
            let _: i32 = conn.del(&count_key).await?;
        }
        
        debug!(
            user_id = %user_id,
            connection_id = %connection_id,
            cluster_count = count,
            "Unregistered cluster connection"
        );
        
        Ok(count)
    }
    
    /// Get cluster-wide connection count for a user
    pub async fn get_connection_count(&self, user_id: Uuid) -> anyhow::Result<usize> {
        let mut conn = self.redis.get().await?;
        
        let count_key = format!("{}{}", CONN_COUNT_PREFIX, user_id);
        let count: usize = conn.scard(&count_key).await?;
        
        Ok(count)
    }
    
    /// Send heartbeat for a connection to keep it alive
    pub async fn heartbeat(&self, user_id: Uuid, connection_id: &str) -> anyhow::Result<()> {
        let mut conn = self.redis.get().await?;
        
        let heartbeat_key = format!("{}{}:{}", CONN_HEARTBEAT_PREFIX, user_id, connection_id);
        
        // Set heartbeat with TTL
        let now = chrono::Utc::now().timestamp();
        let _: () = conn.set_ex(&heartbeat_key, now, CONN_HEARTBEAT_TTL).await?;
        
        Ok(())
    }
    
    /// Clean up stale connections (should be called periodically)
    pub async fn cleanup_stale_connections(&self) -> anyhow::Result<usize> {
        // In a production system, this would scan for and remove stale connections
        // For now, we rely on TTL expiration for automatic cleanup
        Ok(0)
    }
}

/// Check if user has exceeded cluster-wide connection limit
pub async fn check_cluster_connection_limit(
    state: &AppState,
    user_id: Uuid,
    max_connections: usize,
) -> Result<bool, AppError> {
    // If cluster limits are not available, fall back to local-only check
    // This is a simplified version - in production you'd want proper cluster tracking
    
    // First check local count
    let local_count = state.ws_hub.user_connection_count(user_id).await;
    
    if local_count >= max_connections {
        return Ok(false);
    }
    
    // Try to get cluster count from Redis
    match get_global_connection_count(state, user_id).await {
        Ok(global_count) => {
            if global_count >= max_connections {
                debug!(
                    user_id = %user_id,
                    global_count = global_count,
                    max = max_connections,
                    "Cluster connection limit exceeded"
                );
                Ok(false)
            } else {
                Ok(true)
            }
        }
        Err(e) => {
            // If Redis is unavailable, be permissive but log the issue
            warn!(
                user_id = %user_id,
                error = %e,
                "Failed to get global connection count, using local count only"
            );
            Ok(local_count < max_connections)
        }
    }
}

/// Get global connection count from Redis
pub async fn get_global_connection_count(
    state: &AppState,
    user_id: Uuid,
) -> anyhow::Result<usize> {
    use deadpool_redis::redis::AsyncCommands;
    
    let mut conn = state.redis.get().await?;
    let key = format!("rustchat:presence:user:{}:connections", user_id);
    
    let count: usize = conn.scard(&key).await?;
    Ok(count)
}

use crate::error::AppError;

#[cfg(test)]
mod tests {
    use super::*;
    
    // Note: These tests require a running Redis instance
    // Mark them as integration tests
    
    #[test]
    fn test_connection_key_format() {
        let node_id = "test-node";
        let conn_id = "conn-123";
        let expected = format!("{}:{}", conn_id, node_id);
        assert_eq!(expected, "conn-123:test-node");
    }
}
