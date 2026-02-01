//! TURN credential generation for Mattermost Calls
//!
//! Implements REST API style ephemeral credentials as used by Mattermost.
//! See: https://docs.mattermost.com/administration-guide/configure/plugins-configuration-settings.html

use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::time::{SystemTime, UNIX_EPOCH};

/// TURN credentials for a user
#[derive(Debug, Clone)]
pub struct TurnCredentials {
    pub username: String,
    pub credential: String,
}

/// Generates TURN REST API style ephemeral credentials
pub struct TurnCredentialGenerator {
    secret: String,
    ttl_minutes: u64,
}

impl TurnCredentialGenerator {
    /// Create a new credential generator
    pub fn new(secret: String, ttl_minutes: u64) -> Self {
        Self {
            secret,
            ttl_minutes,
        }
    }

    /// Generate ephemeral TURN credentials for a user
    ///
    /// Uses the TURN REST API authentication mechanism:
    /// - Username: "{timestamp}:{user_id}"
    /// - Credential: base64(HMAC-SHA1(secret, username))
    ///
    /// The timestamp is the expiration time (now + TTL).
    pub fn generate_credentials(&self, user_id: &str) -> TurnCredentials {
        // Calculate expiration timestamp
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expiration = now + (self.ttl_minutes * 60);

        // Username format: "{expiration}:{user_id}"
        let username = format!("{}:{}", expiration, user_id);

        // Generate HMAC-SHA1
        let credential = self.hmac_sha1(&self.secret, &username);

        TurnCredentials {
            username,
            credential: base64_encode(&credential),
        }
    }

    /// Validate credentials
    pub fn validate_credentials(&self, username: &str, credential: &str) -> bool {
        // Parse username to extract expiration and user_id
        let parts: Vec<&str> = username.split(':').collect();
        if parts.len() != 2 {
            return false;
        }

        let expiration = match parts[0].parse::<u64>() {
            Ok(ts) => ts,
            Err(_) => return false,
        };

        // Check if expired
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now > expiration {
            return false; // Credentials expired
        }

        // Validate HMAC
        let expected_credential = self.hmac_sha1(&self.secret, username);
        let expected_encoded = base64_encode(&expected_credential);

        credential == expected_encoded
    }

    /// Calculate HMAC-SHA1
    fn hmac_sha1(&self, key: &str, message: &str) -> Vec<u8> {
        type HmacSha1 = Hmac<Sha1>;

        let mut mac =
            HmacSha1::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");
        mac.update(message.as_bytes());

        mac.finalize().into_bytes().to_vec()
    }
}

/// Simple base64 encoding (using standard base64 alphabet)
fn base64_encode(data: &[u8]) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    STANDARD.encode(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_generation() {
        let generator = TurnCredentialGenerator::new(
            "test-secret".to_string(),
            1440, // 24 hours
        );

        let creds = generator.generate_credentials("user-123");

        // Username should contain expiration and user_id
        assert!(creds.username.contains("user-123"));
        assert!(creds.username.contains(':'));

        // Credential should be valid base64
        assert!(!creds.credential.is_empty());

        // Validation should succeed
        assert!(generator.validate_credentials(&creds.username, &creds.credential));
    }

    #[test]
    fn test_invalid_credential() {
        let generator = TurnCredentialGenerator::new("test-secret".to_string(), 1440);

        let creds = generator.generate_credentials("user-123");

        // Wrong credential should fail
        assert!(!generator.validate_credentials(&creds.username, "invalid-credential"));
    }
}
