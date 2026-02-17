//! Firebase Cloud Messaging (FCM) for Android pushes
//!
//! Handles sending push notifications to Android devices via FCM HTTP v1 API.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

/// Push notification payload from RustChat backend
#[derive(Debug, Serialize, Deserialize)]
pub struct PushPayload {
    pub token: String,
    pub title: String,
    pub body: String,
    pub data: PushData,
}

/// Data payload sent to mobile clients
#[derive(Debug, Serialize, Deserialize)]
pub struct PushData {
    pub channel_id: String,
    pub post_id: String,
    #[serde(rename = "type")]
    pub r#type: String, // "message", "clear", "session"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_type: Option<String>, // "calls" for call notifications
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_crt_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_url: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum FcmError {
    #[error("OAuth2 error: {0}")]
    Auth(#[from] std::io::Error),
    #[error("Firebase API error: {0}")]
    Api(String),
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Internal error: {0}")]
    Internal(String),
}

/// FCM HTTP v1 API client
pub struct FcmClient {
    client: reqwest::Client,
    project_id: String,
    authenticator: yup_oauth2::authenticator::Authenticator<
        yup_oauth2::hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>,
    >,
}

impl FcmClient {
    /// Create a new FCM client
    pub async fn new(project_id: String, key_path: PathBuf) -> Result<Self, anyhow::Error> {
        let secret = yup_oauth2::read_service_account_key(key_path).await?;
        let auth = yup_oauth2::ServiceAccountAuthenticator::builder(secret)
            .build()
            .await?;

        Ok(Self {
            client: reqwest::Client::new(),
            project_id,
            authenticator: auth,
        })
    }

    /// Send a push notification via FCM
    pub async fn send(&self, payload: PushPayload) -> Result<(), FcmError> {
        let token = self
            .authenticator
            .token(&["https://www.googleapis.com/auth/cloud-platform"])
            .await
            .map_err(|e| FcmError::Internal(format!("Failed to get OAuth token: {}", e)))?;

        let url = format!(
            "https://fcm.googleapis.com/v1/projects/{}/messages:send",
            self.project_id
        );

        let fcm_message = self.build_fcm_message(payload);

        let response = self.client.post(&url)
            .bearer_auth(token.token().unwrap_or_default())
            .json(&fcm_message)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("FCM API error: {} - {}", status, body);
            return Err(FcmError::Api(format!("Status {}: {}", status, body)));
        }

        info!("Successfully sent notification to FCM");
        Ok(())
    }

    /// Build FCM message with proper Android and APNS configuration
    fn build_fcm_message(&self, payload: PushPayload) -> serde_json::Value {
        let is_call = payload.data.sub_type.as_deref() == Some("calls");

        // Android config — use the mobile app's existing notification channel IDs:
        // "channel_01" = High Importance (IMPORTANCE_HIGH, plays sound)
        // "channel_02" = Min Importance (IMPORTANCE_MIN, silent)
        // For VoIP/call notifications, we use data-only messages on Android to allow
        // the app to show a full-screen incoming call UI
        let android_config = if is_call {
            serde_json::json!({
                "priority": "high",
                "ttl": "0s",
                "notification": {
                    "channel_id": "channel_01",
                    "sound": "default",
                    "click_action": "TOP_STORY_ACTIVITY"
                },
                // Data-only notification for background call handling
                "direct_boot_ok": true
            })
        } else {
            serde_json::json!({
                "priority": "normal",
                "notification": {
                    "channel_id": "channel_01",
                    "click_action": "TOP_STORY_ACTIVITY"
                }
            })
        };

        // Build APNS config for iOS (used as fallback when APNS client is not configured)
        let mut apns_headers = serde_json::Map::new();
        apns_headers.insert("apns-priority".to_string(), if is_call { 
            serde_json::json!("10") 
        } else { 
            serde_json::json!("5") 
        });

        let apns_sound = if is_call { "calls_ringtone.caf" } else { "default" };
        let apns_config = serde_json::json!({
            "headers": apns_headers,
            "payload": {
                "aps": {
                    "alert": {
                        "title": payload.title,
                        "body": payload.body
                    },
                    "sound": apns_sound,
                    "badge": 1,
                    "content-available": 1,
                    "mutable-content": 1
                }
            }
        });

        // Build the message
        // For call notifications on Android, we prioritize data payload
        let message = if is_call {
            serde_json::json!({
                "token": payload.token,
                "notification": {
                    "title": payload.title,
                    "body": payload.body
                },
                "data": {
                    "type": "call",
                    "channel_id": payload.data.channel_id,
                    "post_id": payload.data.post_id,
                    "sender_name": payload.data.sender_name.unwrap_or_default(),
                    "server_url": payload.data.server_url.unwrap_or_default(),
                },
                "android": android_config,
                "apns": apns_config
            })
        } else {
            serde_json::json!({
                "token": payload.token,
                "notification": {
                    "title": payload.title,
                    "body": payload.body
                },
                "data": payload.data,
                "android": android_config,
                "apns": apns_config
            })
        };

        serde_json::json!({ "message": message })
    }
}
