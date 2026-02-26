# Security Headers & Zero-Trust Guide

This guide covers the security headers implementation and zero-trust posture for RustChat.

## Table of Contents

1. [Security Headers Overview](#security-headers-overview)
2. [Zero-Trust Principles](#zero-trust-principles)
3. [Header Configuration](#header-configuration)
4. [Authorization Policy Engine](#authorization-policy-engine)
5. [Reliability Patterns](#reliability-patterns)

---

## Security Headers Overview

RustChat automatically applies security headers to all HTTP responses:

| Header | Purpose | Default Value |
|--------|---------|---------------|
| `Content-Security-Policy` | Prevents XSS and injection attacks | Strict policy with 'self' |
| `X-Frame-Options` | Prevents clickjacking | `SAMEORIGIN` |
| `X-Content-Type-Options` | Prevents MIME sniffing | `nosniff` |
| `Referrer-Policy` | Controls referrer information | `strict-origin-when-cross-origin` |
| `Permissions-Policy` | Restricts browser features | Limited camera/microphone |
| `Strict-Transport-Security` | Enforces HTTPS | 2 years with preload |
| `X-XSS-Protection` | Legacy XSS protection | `1; mode=block` |

---

## Zero-Trust Principles

### Never Trust, Always Verify

1. **Every request is authenticated** - No anonymous access to sensitive endpoints
2. **Every request is authorized** - Role-based permissions checked
3. **Minimal privileges** - Users have only necessary permissions
4. **Assume breach** - Defense in depth with multiple security layers

### Implementation

```rust
// Example: Using the policy engine
use rustchat::auth::policy::{PolicyEngine, permissions::*};
use rustchat::require_permission;

async fn delete_post(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
) -> ApiResult<Json<()>> {
    // Check permission using macro
    require_permission!(auth, POST_DELETE);
    
    // Or check with ownership
    let post = get_post(&state.db, post_id).await?;
    match PolicyEngine::check_ownership(
        &auth.role,
        &POST_DELETE,
        auth.user_id,
        post.user_id
    ) {
        AuthzResult::Allow => {},
        AuthzResult::Deny(reason) => return Err(AppError::Forbidden(reason.to_string())),
    }
    
    // ... delete post
}
```

---

## Header Configuration

### Production (Strict)

```bash
RUSTCHAT_ENVIRONMENT=production
```

Automatically enables:
- Strict CSP
- HSTS with preload
- All security headers

### Development (Permissive)

```bash
RUSTCHAT_ENVIRONMENT=development
```

Allows:
- Inline scripts/styles
- No HSTS (HTTP allowed)
- Broader permissions

### Custom Configuration

Modify `backend/src/middleware/security_headers.rs`:

```rust
let config = SecurityHeadersConfig {
    csp: "default-src 'self'".to_string(),
    hsts_enabled: true,
    hsts_max_age: 63072000,
    // ...
};
```

### Content Security Policy

Default CSP blocks:
- Inline scripts (use nonces or hashes)
- External scripts not explicitly allowed
- Data URIs in sensitive contexts
- Framing by external sites

**Frontend Requirements:**
```javascript
// Use external scripts
<script src="/app.js"></script>

// Avoid inline handlers
// ❌ Bad
<button onclick="doSomething()">

// ✅ Good
<button id="myBtn">
<script>
  document.getElementById('myBtn').addEventListener('click', doSomething);
</script>
```

---

## Authorization Policy Engine

### Roles

| Role | Description |
|------|-------------|
| `system_admin` | Full system access |
| `team_admin` | Team/channel management |
| `member` | Standard user permissions |
| `guest` | Read-only access |

### Permissions

```rust
use rustchat::auth::policy::permissions::*;

// User management
USER_READ, USER_UPDATE, USER_DELETE, USER_MANAGE

// Team operations
TEAM_CREATE, TEAM_READ, TEAM_UPDATE, TEAM_DELETE, TEAM_MANAGE

// Channel operations
CHANNEL_CREATE, CHANNEL_READ, CHANNEL_UPDATE, CHANNEL_DELETE, CHANNEL_MANAGE

// Post operations
POST_CREATE, POST_READ, POST_UPDATE, POST_DELETE

// System operations
SYSTEM_READ, SYSTEM_MANAGE, ADMIN_FULL
```

### Usage Examples

```rust
// Check admin access
require_admin!(auth_user);

// Check specific permission
require_permission!(auth_user, CHANNEL_DELETE);

// Programmatic check
match PolicyEngine::check_permission(&auth_user.role, &POST_DELETE) {
    AuthzResult::Allow => delete_post(),
    AuthzResult::Deny(reason) => Err(Forbidden(reason)),
}
```

### Extending Permissions

Add custom permissions:

```rust
// In auth/policy.rs
pub const CUSTOM_ACTION: Permission = Permission::new(Resource::System, Action::Manage);

// Add to role
impl Role {
    pub fn custom_role() -> Self {
        let mut permissions = HashSet::new();
        permissions.insert(CUSTOM_ACTION);
        // ...
    }
}
```

---

## Reliability Patterns

### Circuit Breakers

Protect external service calls from cascading failures:

```rust
use rustchat::middleware::reliability::{CircuitBreaker, CircuitBreakerConfig};

// State manages circuit breakers
let result = state.circuit_breakers.oidc.execute(|| async {
    // OIDC discovery call
    fetch_oidc_config(url).await
}).await;

match result {
    Ok(config) => // use config,
    Err(CircuitError::Open) => // circuit is open, fail fast,
    Err(CircuitError::Inner(e)) => // OIDC error,
}
```

### Available Circuit Breakers

| Service | Purpose |
|---------|---------|
| `oidc` | OIDC discovery and token validation |
| `s3` | File storage operations |
| `email` | SMTP/email provider calls |
| `turnstile` | Bot verification |

### Retry Logic

Automatic retry with exponential backoff:

```rust
use rustchat::middleware::reliability::{with_retry, RetryConfig};

let config = RetryConfig {
    max_attempts: 3,
    initial_delay: Duration::from_millis(100),
    backoff_multiplier: 2.0,
    ..Default::default()
};

let result = with_retry(&config, || async {
    flaky_external_call().await
}).await;
```

### Combined Resilience

Use both circuit breaker and retry:

```rust
use rustchat::middleware::reliability::with_resilience;

let result = with_resilience(
    &state.circuit_breakers.email,
    &retry_config,
    || async { send_email().await }
).await;
```

---

## Monitoring

### Circuit Breaker State

```bash
# Check circuit breaker states
curl http://localhost:3000/api/v1/admin/circuits
```

Response:
```json
{
  "oidc": "closed",
  "s3": "closed",
  "email": "half_open",
  "turnstile": "closed"
}
```

### Security Headers Verification

```bash
# Verify headers
curl -I https://chat.example.com/api/v1/health
```

Expected:
```
strict-transport-security: max-age=63072000; includeSubDomains; preload
content-security-policy: default-src 'self'; ...
x-frame-options: SAMEORIGIN
x-content-type-options: nosniff
```

---

## Deployment Checklist

- [ ] Set `RUSTCHAT_ENVIRONMENT=production`
- [ ] Verify HSTS headers in responses
- [ ] Test CSP doesn't break frontend
- [ ] Verify all admin endpoints require admin role
- [ ] Check circuit breaker metrics
- [ ] Test retry behavior with simulated failures
- [ ] Review permission assignments for all roles

---

## Troubleshooting

### CSP Violations

Check browser console for:
```
Refused to load script '...' because it violates Content Security Policy
```

**Solution:**
1. Move inline scripts to external files
2. Add necessary domains to CSP
3. Use nonces for dynamic scripts

### Circuit Breaker Open

```
Circuit breaker is open for: email
```

**Solution:**
1. Check external service health
2. Wait for recovery timeout (30s default)
3. Review failure threshold

### Permission Denied

```
Permission denied: User lacks POST_DELETE permission
```

**Solution:**
1. Check user role assignment
2. Verify ownership for resource-level permissions
3. Review role permission definitions
