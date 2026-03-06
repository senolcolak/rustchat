//! Security headers middleware
//!
//! Implements zero-trust security headers for all HTTP responses.
//! Protects against common web attacks including XSS, clickjacking,
//! MIME sniffing, and other browser-based vulnerabilities.

use axum::{
    body::Body,
    http::{header, Request, Response},
};
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// Security headers configuration
#[derive(Debug, Clone)]
pub struct SecurityHeadersConfig {
    /// Content Security Policy
    pub csp: String,
    /// Whether to include HSTS header
    pub hsts_enabled: bool,
    /// HSTS max age in seconds
    pub hsts_max_age: u64,
    /// Whether to include HSTS preload
    pub hsts_preload: bool,
    /// Whether to include HSTS includeSubDomains
    pub hsts_include_subdomains: bool,
    /// X-Frame-Options value
    pub frame_options: String,
    /// X-Content-Type-Options value
    pub content_type_options: String,
    /// Referrer-Policy value
    pub referrer_policy: String,
    /// Permissions-Policy value
    pub permissions_policy: String,
    /// X-XSS-Protection value
    pub xss_protection: String,
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        Self::strict()
    }
}

impl SecurityHeadersConfig {
    /// Strict security headers (recommended for production)
    pub fn strict() -> Self {
        Self {
            // Strict CSP - adjust based on your frontend needs
            csp: "default-src 'self'; \
                   script-src 'self' 'unsafe-inline'; \
                   style-src 'self' 'unsafe-inline'; \
                   img-src 'self' data: blob: https:; \
                   font-src 'self' data:; \
                   connect-src 'self' wss: https:; \
                   media-src 'self' blob:; \
                   frame-ancestors 'self'; \
                   base-uri 'self'; \
                   form-action 'self';"
                .replace("\n", " ")
                .replace("  ", " "),
            hsts_enabled: true,
            hsts_max_age: 63072000, // 2 years
            hsts_preload: true,
            hsts_include_subdomains: true,
            frame_options: "SAMEORIGIN".to_string(),
            content_type_options: "nosniff".to_string(),
            referrer_policy: "strict-origin-when-cross-origin".to_string(),
            permissions_policy: "camera=(), microphone=(), geolocation=(), \
                                payment=(), usb=(), magnetometer=(), \
                                gyroscope=(), speaker=()"
                .replace("\n", " ")
                .replace("  ", " "),
            xss_protection: "1; mode=block".to_string(),
        }
    }

    /// Permissive headers for development
    pub fn development() -> Self {
        Self {
            csp: "default-src 'self' 'unsafe-inline' 'unsafe-eval' \
                   http: https: ws: wss: data: blob:;"
                .replace("\n", " ")
                .replace("  ", " "),
            hsts_enabled: false, // Don't force HTTPS in dev
            hsts_max_age: 0,
            hsts_preload: false,
            hsts_include_subdomains: false,
            frame_options: "SAMEORIGIN".to_string(),
            content_type_options: "nosniff".to_string(),
            referrer_policy: "strict-origin-when-cross-origin".to_string(),
            permissions_policy: "camera=*, microphone=*, geolocation=*".to_string(),
            xss_protection: "1; mode=block".to_string(),
        }
    }

    /// API-only headers (no CSP needed for pure API)
    pub fn api_only() -> Self {
        Self {
            csp: "default-src 'none'; frame-ancestors 'none';".to_string(),
            hsts_enabled: true,
            hsts_max_age: 63072000,
            hsts_preload: true,
            hsts_include_subdomains: true,
            frame_options: "DENY".to_string(),
            content_type_options: "nosniff".to_string(),
            referrer_policy: "no-referrer".to_string(),
            permissions_policy: "()".to_string(),
            xss_protection: "1; mode=block".to_string(),
        }
    }
}

/// Security headers middleware layer
#[derive(Debug, Clone)]
pub struct SecurityHeadersLayer {
    config: SecurityHeadersConfig,
}

impl SecurityHeadersLayer {
    pub fn new(config: SecurityHeadersConfig) -> Self {
        Self { config }
    }

    pub fn strict() -> Self {
        Self::new(SecurityHeadersConfig::strict())
    }

    pub fn development() -> Self {
        Self::new(SecurityHeadersConfig::development())
    }

    pub fn api_only() -> Self {
        Self::new(SecurityHeadersConfig::api_only())
    }
}

impl<S> Layer<S> for SecurityHeadersLayer {
    type Service = SecurityHeadersService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SecurityHeadersService {
            inner,
            config: self.config.clone(),
        }
    }
}

/// Security headers middleware service
#[derive(Debug, Clone)]
pub struct SecurityHeadersService<S> {
    inner: S,
    config: SecurityHeadersConfig,
}

impl<S, B> Service<Request<B>> for SecurityHeadersService<S>
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
        let config = self.config.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let mut response = inner.call(req).await?;
            let headers = response.headers_mut();

            // Content Security Policy
            headers.insert(header::CONTENT_SECURITY_POLICY, config.csp.parse().unwrap());

            // X-Frame-Options
            headers.insert(
                header::X_FRAME_OPTIONS,
                config.frame_options.parse().unwrap(),
            );

            // X-Content-Type-Options
            headers.insert(
                header::X_CONTENT_TYPE_OPTIONS,
                config.content_type_options.parse().unwrap(),
            );

            // Referrer-Policy
            headers.insert(
                header::HeaderName::from_static("referrer-policy"),
                config.referrer_policy.parse().unwrap(),
            );

            // Permissions-Policy (formerly Feature-Policy)
            headers.insert(
                header::HeaderName::from_static("permissions-policy"),
                config.permissions_policy.parse().unwrap(),
            );

            // X-XSS-Protection (legacy but still useful)
            headers.insert(
                header::X_XSS_PROTECTION,
                config.xss_protection.parse().unwrap(),
            );

            // Strict-Transport-Security (HSTS)
            if config.hsts_enabled {
                let hsts_value = format!(
                    "max-age={}{}{}",
                    config.hsts_max_age,
                    if config.hsts_include_subdomains {
                        "; includeSubDomains"
                    } else {
                        ""
                    },
                    if config.hsts_preload { "; preload" } else { "" }
                );
                headers.insert(
                    header::STRICT_TRANSPORT_SECURITY,
                    hsts_value.parse().unwrap(),
                );
            }

            // Additional security headers

            // Prevent MIME sniffing
            headers.insert(header::X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());

            // Remove server information (if present)
            headers.remove(header::SERVER);

            Ok(response)
        })
    }
}

/// Helper to create a CORS-appropriate security headers configuration
/// that works with the CORS layer
pub fn cors_compatible_config() -> SecurityHeadersConfig {
    SecurityHeadersConfig {
        csp: "default-src 'self'; \
               script-src 'self' 'unsafe-inline'; \
               style-src 'self' 'unsafe-inline'; \
               img-src 'self' data: blob: https:; \
               font-src 'self' data:; \
               connect-src *; \
               media-src 'self' blob:; \
               frame-ancestors 'self'; \
               base-uri 'self'; \
               form-action 'self';"
            .replace("\n", " ")
            .replace("  ", " "),
        hsts_enabled: true,
        hsts_max_age: 63072000,
        hsts_preload: true,
        hsts_include_subdomains: true,
        frame_options: "SAMEORIGIN".to_string(),
        content_type_options: "nosniff".to_string(),
        referrer_policy: "strict-origin-when-cross-origin".to_string(),
        permissions_policy: "camera=(self), microphone=(self), geolocation=()".to_string(),
        xss_protection: "1; mode=block".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strict_config() {
        let config = SecurityHeadersConfig::strict();
        assert!(config.hsts_enabled);
        assert_eq!(config.hsts_max_age, 63072000);
        assert!(config.hsts_preload);
    }

    #[test]
    fn test_development_config() {
        let config = SecurityHeadersConfig::development();
        assert!(!config.hsts_enabled);
        assert!(config.csp.contains("unsafe-inline"));
    }

    #[test]
    fn test_api_only_config() {
        let config = SecurityHeadersConfig::api_only();
        assert!(config.csp.contains("default-src 'none'"));
        assert_eq!(config.frame_options, "DENY");
    }
}
