#!/bin/bash

# Test script for push notifications
# Usage: ./test-push.sh <android|ios> <device_token>

set -e

PLATFORM=${1:-android}
TOKEN=${2:-test-token}
PROXY_URL=${PUSH_PROXY_URL:-http://localhost:3000}

echo "=== Testing Push Proxy ==="
echo "Platform: $PLATFORM"
echo "Token: ${TOKEN:0:20}..."
echo "Proxy URL: $PROXY_URL"
echo ""

# First, test health endpoint
echo "1. Testing health endpoint..."
HEALTH_RESPONSE=$(curl -s -w "\n%{http_code}" "$PROXY_URL/health" 2>/dev/null || echo "000")
HTTP_CODE=$(echo "$HEALTH_RESPONSE" | tail -n1)
BODY=$(echo "$HEALTH_RESPONSE" | sed '$d')

echo "   HTTP Status: $HTTP_CODE"
echo "   Response: $BODY"
echo ""

if [ "$HTTP_CODE" != "200" ]; then
    echo "ERROR: Health check failed! Is the push proxy running?"
    exit 1
fi

# Test Android push
if [ "$PLATFORM" == "android" ]; then
    echo "2. Sending Android/FCM test push..."
    
    REQUEST_BODY='{
        "token": "'"$TOKEN"'",
        "title": "Incoming call from Test User",
        "body": "Tap to answer",
        "platform": "android",
        "type": "call",
        "data": {
            "channel_id": "test-channel-id",
            "post_id": "test-post-id",
            "type": "message",
            "sub_type": "calls",
            "version": "2",
            "sender_id": "test-sender-id",
            "sender_name": "Test User",
            "server_url": "https://rustchat.com",
            "call_uuid": "550e8400-e29b-41d4-a716-446655440000"
        }
    }'
    
    echo "   Request:"
    echo "$REQUEST_BODY" | jq . 2>/dev/null || echo "$REQUEST_BODY"
    echo ""
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$REQUEST_BODY" \
        "$PROXY_URL/send" 2>/dev/null || echo "000")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')
    
    echo "   HTTP Status: $HTTP_CODE"
    echo "   Response: $BODY"
    
    if [ "$HTTP_CODE" == "200" ]; then
        echo ""
        echo "✓ SUCCESS: Push notification sent!"
        echo ""
        echo "Expected behavior on Android:"
        echo "  - App should receive message in FirebaseMessagingService.onMessageReceived()"
        echo "  - Full-screen incoming call UI should appear"
        echo "  - System notification should NOT appear"
    elif [ "$HTTP_CODE" == "503" ]; then
        echo ""
        echo "✗ ERROR: FCM not configured"
        echo "   Set FIREBASE_PROJECT_ID and GOOGLE_APPLICATION_CREDENTIALS"
    elif [ "$HTTP_CODE" == "410" ]; then
        echo ""
        echo "✗ ERROR: Token is unregistered (device uninstalled app)"
    else
        echo ""
        echo "✗ ERROR: Failed to send push notification"
    fi

# Test iOS push
elif [ "$PLATFORM" == "ios" ]; then
    echo "2. Sending iOS/APNS test push..."
    
    REQUEST_BODY='{
        "token": "'"$TOKEN"'",
        "title": "Incoming call from Test User",
        "body": "Tap to answer",
        "platform": "ios",
        "type": "call",
        "data": {
            "channel_id": "test-channel-id",
            "post_id": "test-post-id",
            "type": "message",
            "sub_type": "calls",
            "version": "2",
            "sender_id": "test-sender-id",
            "sender_name": "Test User",
            "server_url": "https://rustchat.com",
            "call_uuid": "550e8400-e29b-41d4-a716-446655440000"
        }
    }'
    
    echo "   Request:"
    echo "$REQUEST_BODY" | jq . 2>/dev/null || echo "$REQUEST_BODY"
    echo ""
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -X POST \
        -H "Content-Type: application/json" \
        -d "$REQUEST_BODY" \
        "$PROXY_URL/send" 2>/dev/null || echo "000")
    
    HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
    BODY=$(echo "$RESPONSE" | sed '$d')
    
    echo "   HTTP Status: $HTTP_CODE"
    echo "   Response: $BODY"
    
    if [ "$HTTP_CODE" == "200" ]; then
        echo ""
        echo "✓ SUCCESS: VoIP push notification sent!"
        echo ""
        echo "Expected behavior on iOS:"
        echo "  - Push should be received via PKPushRegistry"
        echo "  - CallKit should display native incoming call UI"
        echo "  - Phone should ring with the ringtone"
    elif [ "$HTTP_CODE" == "503" ]; then
        echo ""
        echo "✗ ERROR: APNS not configured"
        echo "   Set APNS_KEY_PATH, APNS_KEY_ID, APNS_TEAM_ID, APNS_BUNDLE_ID"
    elif [ "$HTTP_CODE" == "410" ]; then
        echo ""
        echo "✗ ERROR: Token is invalid (device unregistered)"
    else
        echo ""
        echo "✗ ERROR: Failed to send push notification"
    fi
else
    echo "Usage: $0 <android|ios> <device_token>"
    exit 1
fi

echo ""
echo "=== End of Test ==="
