# Push Proxy Docker Deployment Guide

This guide explains how to deploy the RustChat Push Proxy using Docker Compose.

## Quick Start

```bash
# 1. Copy and configure environment variables
cp .env.example .env

# 2. Place your certificates in the secrets directory
mkdir -p secrets/
cp /path/to/firebase-key.json secrets/
cp /path/to/voip-cert.pem secrets/
cp /path/to/voip-key.pem secrets/

# 3. Build and start the service
docker-compose up -d push-proxy
```

## Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `FIREBASE_PROJECT_ID` | Yes* | - | Firebase project ID for Android pushes |
| `GOOGLE_APPLICATION_CREDENTIALS` | Yes* | - | Path to Firebase service account JSON |
| `APNS_CERT_PATH` | Yes† | - | Path to APNS VoIP certificate |
| `APNS_KEY_PATH` | No | - | Path to APNS private key (if separate) |
| `APNS_CERT_PASSWORD` | No | - | Password for encrypted certificate |
| `APNS_BUNDLE_ID` | Yes† | - | iOS app bundle identifier |
| `APNS_USE_PRODUCTION` | No | `false` | Use production APNS servers |
| `RUSTCHAT_PUSH_PORT` | No | `3000` | HTTP server port |
| `RUST_LOG` | No | `info` | Logging level |

*Required for Android support
†Required for iOS VoIP support

### Docker Compose Configuration

```yaml
version: '3.8'

services:
  push-proxy:
    build:
      context: ../push-proxy
      dockerfile: Dockerfile
    container_name: rustchat-push-proxy
    restart: unless-stopped
    environment:
      # Firebase (Android)
      - FIREBASE_PROJECT_ID=${FIREBASE_PROJECT_ID}
      - GOOGLE_APPLICATION_CREDENTIALS=/secrets/firebase-key.json
      
      # APNS (iOS VoIP)
      - APNS_CERT_PATH=/secrets/voip-cert.pem
      - APNS_KEY_PATH=/secrets/voip-key.pem
      - APNS_BUNDLE_ID=${APNS_BUNDLE_ID}
      - APNS_USE_PRODUCTION=${APNS_USE_PRODUCTION:-false}
      
      # General
      - RUSTCHAT_PUSH_PORT=3000
      - RUST_LOG=push_proxy=info,tower_http=warn
    volumes:
      - ./secrets/firebase-key.json:/secrets/firebase-key.json:ro
      - ./secrets/voip-cert.pem:/secrets/voip-cert.pem:ro
      - ./secrets/voip-key.pem:/secrets/voip-key.pem:ro
    networks:
      - rustchat-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s

networks:
  rustchat-network:
    external: true
```

## Certificate Setup

### Firebase Service Account

1. Go to [Firebase Console](https://console.firebase.google.com/)
2. Select your project
3. Click gear icon → Project settings
4. Service accounts tab
5. Click "Generate new private key"
6. Save the JSON file as `secrets/firebase-key.json`

### APNS VoIP Certificate

1. Go to [Apple Developer Portal](https://developer.apple.com/)
2. Certificates, Identifiers & Profiles
3. Create a new certificate → VoIP Services Certificate
4. Select your App ID and continue
5. Upload a Certificate Signing Request (CSR)
6. Download the certificate (`.cer` file)
7. Convert to PEM format:

```bash
# Convert .cer to .pem
openssl x509 -in voip-cert.cer -inform DER -out voip-cert.pem -outform PEM

# If you have a .p12 file
openssl pkcs12 -in voip-cert.p12 -out voip-cert.pem -nodes -clcerts
```

## Production Deployment

### 1. Environment Preparation

```bash
# Create production directory
mkdir -p /opt/rustchat/push-proxy
cd /opt/rustchat/push-proxy

# Create directory structure
mkdir -p secrets logs

# Set proper permissions
chmod 700 secrets
chmod 755 logs
```

### 2. SSL/TLS Configuration

For production, place the push proxy behind a reverse proxy (nginx/traefik) with SSL:

```nginx
# nginx.conf
server {
    listen 443 ssl http2;
    server_name push.rustchat.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://push-proxy:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_read_timeout 86400;
    }
}
```

### 3. Docker Compose Production

```yaml
version: '3.8'

services:
  push-proxy:
    image: rustchat/push-proxy:latest
    container_name: rustchat-push-proxy
    restart: unless-stopped
    deploy:
      resources:
        limits:
          cpus: '0.5'
          memory: 256M
        reservations:
          cpus: '0.25'
          memory: 128M
    environment:
      - FIREBASE_PROJECT_ID=${FIREBASE_PROJECT_ID}
      - GOOGLE_APPLICATION_CREDENTIALS=/secrets/firebase-key.json
      - APNS_CERT_PATH=/secrets/voip-cert.pem
      - APNS_KEY_PATH=/secrets/voip-key.pem
      - APNS_BUNDLE_ID=${APNS_BUNDLE_ID}
      - APNS_USE_PRODUCTION=true
      - RUSTCHAT_PUSH_PORT=3000
      - RUST_LOG=push_proxy=warn
    volumes:
      - ./secrets:/secrets:ro
      - ./logs:/logs
    networks:
      - rustchat
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"

networks:
  rustchat:
    external: true
```

### 4. Monitoring

Add health checks and monitoring:

```yaml
  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    ports:
      - "9090:9090"

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    volumes:
      - grafana-storage:/var/lib/grafana
```

## Troubleshooting

### Check Service Health

```bash
# Check if service is running
curl http://localhost:3000/health

# Expected response:
# {"status":"ok","service":"rustchat-push-proxy"}
```

### View Logs

```bash
# Follow logs
docker-compose logs -f push-proxy

# View recent logs
docker-compose logs --tail=100 push-proxy
```

### Test Push Notification

```bash
# Android (FCM)
curl -X POST http://localhost:3000/send \
  -H "Content-Type: application/json" \
  -d '{
    "token": "android-fcm-token",
    "title": "Test Call",
    "body": "Incoming call",
    "platform": "android",
    "type": "call",
    "data": {
      "channel_id": "test-channel",
      "post_id": "test-post",
      "type": "call",
      "sub_type": "calls"
    }
  }'

# iOS (VoIP)
curl -X POST http://localhost:3000/send \
  -H "Content-Type: application/json" \
  -d '{
    "token": "ios-apns-token",
    "title": "Test Call",
    "body": "Incoming call",
    "platform": "ios",
    "type": "call",
    "data": {
      "channel_id": "test-channel",
      "post_id": "test-post",
      "type": "call",
      "sub_type": "calls",
      "sender_name": "Test User",
      "call_uuid": "550e8400-e29b-41d4-a716-446655440000"
    }
  }'
```

### Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| `FIREBASE_PROJECT_ID must be set` | Missing env var | Set FIREBASE_PROJECT_ID or disable FCM |
| `Failed to get OAuth token` | Invalid service account key | Check firebase-key.json file |
| `APNS not configured` | Missing APNS_CERT_PATH | Set APNS_CERT_PATH or disable APNS |
| `Token unregistered` | Device uninstalled app | Token should be removed from server |
| `Invalid token` | Wrong token format | Check token format for platform |

## Security Best Practices

1. **Certificate Storage:**
   ```bash
   chmod 600 secrets/*.json
   chmod 600 secrets/*.pem
   chown -R root:root secrets/
   ```

2. **Network Isolation:**
   - Don't expose push-proxy port publicly
   - Use internal Docker network
   - Place behind reverse proxy with SSL

3. **Secret Rotation:**
   - Rotate Firebase keys every 90 days
   - Renew APNS certificates annually
   - Monitor certificate expiration

## Scaling

For high-volume deployments:

```yaml
deploy:
  replicas: 3
  update_config:
    parallelism: 1
    delay: 10s
  restart_policy:
    condition: on-failure
```

Consider using a message queue (Redis/RabbitMQ) for push notification queuing in high-volume scenarios.

## Backup and Recovery

Backup these files:
- `secrets/firebase-key.json`
- `secrets/voip-cert.pem`
- `secrets/voip-key.pem`
- `.env` configuration

Store backups in encrypted storage separate from the server.
