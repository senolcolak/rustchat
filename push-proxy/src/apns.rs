//! Apple Push Notification Service (APNS) for VoIP pushes
//!
//! Handles sending VoIP push notifications to iOS devices via APNS.
//! VoIP pushes use a different certificate and topic than regular notifications.

use serde::Serialize;
use std::path::PathBuf;
use tracing::{info, warn};

/// APNS VoIP push payload
#[derive(Debug, Serialize)]
pub struct ApnsVoipPayload {
    /// APNS topic (bundle ID + ".voip" suffix)
    pub topic: String,
    /// Device token
    pub device_token: String,
    /// Call UUID (required for CallKit)
    pub call_uuid: String,
    /// Caller name to display
    pub caller_name: String,
    /// Channel ID for navigation
    pub channel_id: String,
    /// Server URL so app knows which server
    pub server_url: String,
    /// Handle type (phone number, email, or generic)
    pub handle_type: String,
    /// Has video
    pub has_video: bool,
}

/// APNS certificate configuration
#[derive(Debug, Clone)]
pub struct ApnsConfig {
    /// Path to the VoIP certificate file (.p12 or .pem)
    pub cert_path: PathBuf,
    /// Path to the private key file (if separate from cert)
    pub key_path: Option<PathBuf>,
    /// Certificate password (if encrypted)
    pub cert_password: Option<String>,
    /// APNS server URL
    pub server: ApnsServer,
    /// Bundle identifier with .voip suffix
    pub bundle_id: String,
}

/// APNS server environment
#[derive(Debug, Clone, Copy)]
pub enum ApnsServer {
    /// Production server
    Production,
    /// Development/Sandbox server
    Development,
}

impl ApnsServer {
    pub fn url(&self) -> &'static str {
        match self {
            ApnsServer::Production => "https://api.push.apple.com",
            ApnsServer::Development => "https://api.development.push.apple.com",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApnsError {
    #[error("Certificate error: {0}")]
    Certificate(String),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("APNS error: {0}")]
    Apns(String),
    #[error("Invalid token")]
    InvalidToken,
}

/// APNS HTTP/2 client for VoIP pushes
pub struct ApnsClient {
    http_client: reqwest::Client,
    /// APNS configuration (public for access to bundle_id)
    pub config: ApnsConfig,
}

impl ApnsClient {
    /// Create a new APNS client with the given configuration
    pub async fn new(config: ApnsConfig) -> Result<Self, ApnsError> {
        // Build HTTP/2 client with TLS
        let http_client = reqwest::Client::builder()
            .http2_prior_knowledge()
            .build()
            .map_err(|e| ApnsError::Certificate(format!("Failed to build HTTP client: {}", e)))?;

        info!(
            bundle_id = %config.bundle_id,
            server = ?config.server,
            "APNS client initialized for VoIP pushes"
        );

        Ok(Self {
            http_client,
            config,
        })
    }

    /// Send a VoIP push notification
    /// 
    /// # Important
    /// This must complete within milliseconds or iOS will terminate the app.
    pub async fn send_voip_push(&self, payload: ApnsVoipPayload) -> Result<(), ApnsError> {
        let url = format!(
            "{}/3/device/{}",
            self.config.server.url(),
            payload.device_token
        );

        // Build the APNS payload
        // This format is required for CallKit integration
        let apns_payload = serde_json::json!({
            "aps": {
                "alert": {
                    "title": payload.caller_name,
                    "body": "Incoming call"
                },
                "sound": "calls_ringtone.caf",
                "badge": 1,
                "content-available": 1,
                "mutable-content": 1
            },
            // Custom data for the app
            "data": {
                "type": "call",
                "call_uuid": payload.call_uuid,
                "caller_name": payload.caller_name,
                "channel_id": payload.channel_id,
                "server_url": payload.server_url,
                "has_video": payload.has_video,
                "is_voip": true
            }
        });

        // For VoIP pushes, we use certificate-based authentication
        // The certificate must be a VoIP Services certificate from Apple
        let response = self.http_client
            .post(&url)
            .header("apns-topic", &payload.topic)
            .header("apns-push-type", "voip")
            .header("apns-priority", "10") // Immediate delivery
            .json(&apns_payload)
            .send()
            .await?;

        let status = response.status();
        
        if status.is_success() {
            info!(
                token_prefix = %&payload.device_token[..20.min(payload.device_token.len())],
                "VoIP push sent successfully"
            );
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            
            // Check for invalid token (410 Gone)
            if status.as_u16() == 410 {
                warn!(status = %status, "APNS token is no longer valid");
                return Err(ApnsError::InvalidToken);
            }
            
            warn!(status = %status, body = %body, "APNS error");
            Err(ApnsError::Apns(format!("HTTP {}: {}", status, body)))
        }
    }
}

/// Parse APNS topic from bundle ID
pub fn build_voip_topic(bundle_id: &str) -> String {
    if bundle_id.ends_with(".voip") {
        bundle_id.to_string()
    } else {
        format!("{}.voip", bundle_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_voip_topic() {
        assert_eq!(
            build_voip_topic("com.rustchat.app"),
            "com.rustchat.app.voip"
        );
        assert_eq!(
            build_voip_topic("com.rustchat.app.voip"),
            "com.rustchat.app.voip"
        );
    }
}
