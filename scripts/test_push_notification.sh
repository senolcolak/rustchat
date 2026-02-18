#!/bin/bash
# Test script for push notifications
# Usage: ./test_push_notification.sh <device_token> [platform]

set -e

DEVICE_TOKEN="${1:-}"
PLATFORM="${2:-android}"
PUSH_PROXY_URL="${RUSTCHAT_PUSH_PROXY_URL:-http://localhost:3001}"

if [ -z "$DEVICE_TOKEN" ]; then
    echo "Usage: $0 <device_token> [platform]"
    echo ""
    echo "Example:"
    echo "  $0 fcm_token_abc123 android"
    echo "  $0 apns_token_xyz789 ios"
    exit 1
fi

echo "Testing push notification to $PLATFORM device..."
echo "Push Proxy URL: $PUSH_PROXY_URL"
echo ""

# Test message notification
echo "1. Testing message notification..."
curl -s -X POST "$PUSH_PROXY_URL/send" \
  -H "Content-Type: application/json" \
  -d "{
    \"token\": \"$DEVICE_TOKEN\",
    \"title\": \"Test Message\",
    \"body\": \"This is a test message notification\",
    \"platform\": \"$PLATFORM\",
    \"type\": \"message\",
    \"data\": {
      \"channel_id\": \"test-channel-123\",
      \"post_id\": \"test-post-456\",
      \"type\": \"message\",
      \"sender_name\": \"Test User\"
    }
  }" | jq . 2>/dev/null || echo "Response received"

echo ""
echo ""

# Test call notification (VoIP)
echo "2. Testing call notification (VoIP)..."
curl -s -X POST "$PUSH_PROXY_URL/send" \
  -H "Content-Type: application/json" \
  -d "{
    \"token\": \"$DEVICE_TOKEN\",
    \"title\": \"Incoming Call\",
    \"body\": \"Test User is calling\",
    \"platform\": \"$PLATFORM\",
    \"type\": \"call\",
    \"data\": {
      \"channel_id\": \"test-channel-123\",
      \"post_id\": \"test-call-789\",
      \"type\": \"call\",
      \"sub_type\": \"calls\",
      \"sender_name\": \"Test User\",
      \"call_uuid\": \"$(uuidgen 2>/dev/null || echo "550e8400-e29b-41d4-a716-446655440000")\"
    }
  }" | jq . 2>/dev/null || echo "Response received"

echo ""
echo ""
echo "Tests completed!"
