#!/bin/bash
# Diagnose push proxy connectivity issues

echo "=== Push Proxy Diagnostic Script ==="
echo ""

# Check if container is running
echo "1. Checking if push-proxy container is running..."
if docker ps | grep -q push-proxy; then
    echo "   ✓ Container is running"
    docker ps | grep push-proxy
else
    echo "   ✗ Container is NOT running!"
    echo ""
    echo "   Attempting to start it..."
    docker-compose up -d push-proxy
    sleep 3
    if docker ps | grep -q push-proxy; then
        echo "   ✓ Container started successfully"
    else
        echo "   ✗ Failed to start container. Checking logs..."
        docker-compose logs push-proxy --tail=20
        exit 1
    fi
fi
echo ""

# Check port binding
echo "2. Checking port binding..."
if docker port rustchat-push-proxy 2>/dev/null | grep -q 3000; then
    echo "   ✓ Port 3000 is bound:"
    docker port rustchat-push-proxy
else
    echo "   ✗ Port 3000 not found in container mapping"
    echo "   Checking container network..."
    docker inspect rustchat-push-proxy --format='{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}'
fi
echo ""

# Check if port is listening
echo "3. Checking if port 3000 is listening..."
if netstat -tlnp 2>/dev/null | grep -q :3000; then
    echo "   ✓ Port 3000 is listening:"
    netstat -tlnp | grep :3000
elif ss -tlnp 2>/dev/null | grep -q :3000; then
    echo "   ✓ Port 3000 is listening:"
    ss -tlnp | grep :3000
else
    echo "   ✗ Port 3000 is not listening!"
fi
echo ""

# Test from host
echo "4. Testing health endpoint from host..."
HEALTH=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/health 2>/dev/null || echo "000")
if [ "$HEALTH" == "200" ]; then
    echo "   ✓ Health check responded with 200"
    curl -s http://localhost:3000/health | jq . 2>/dev/null || curl -s http://localhost:3000/health
elif [ "$HEALTH" == "000" ]; then
    echo "   ✗ Connection failed (curl exit code not 0)"
else
    echo "   ✗ Health check returned HTTP $HEALTH"
fi
echo ""

# Test from inside container
echo "5. Testing health from inside container..."
if docker exec rustchat-push-proxy wget -qO- http://localhost:3000/health 2>/dev/null; then
    echo "   ✓ Container can reach itself"
else
    echo "   ✗ Container cannot reach itself!"
fi
echo ""

# Check environment variables
echo "6. Checking environment variables..."
docker exec rustchat-push-proxy env 2>/dev/null | grep -E "(FIREBASE|APNS|RUSTCHAT)" | while read line; do
    echo "   $line"
done
echo ""

# Check recent logs
echo "7. Recent container logs (last 20 lines):"
docker logs --tail=20 rustchat-push-proxy 2>&1 | tail -20
echo ""

# Test push notification
echo "8. Testing push notification..."
RESPONSE=$(curl -s -w "\nHTTP_CODE:%{http_code}" http://localhost:3000/send \
    -X POST \
    -H "Content-Type: application/json" \
    -d '{
      "token": "test-token",
      "title": "Test",
      "body": "Test",
      "platform": "android",
      "type": "message",
      "data": {"channel_id": "test", "post_id": "test", "type": "message"}
    }' 2>/dev/null || echo "CONNECTION_FAILED")

if echo "$RESPONSE" | grep -q "CONNECTION_FAILED"; then
    echo "   ✗ Connection failed!"
elif echo "$RESPONSE" | grep -q "HTTP_CODE:000"; then
    echo "   ✗ No HTTP response"
else
    HTTP_CODE=$(echo "$RESPONSE" | grep "HTTP_CODE:" | cut -d: -f2)
    echo "   HTTP Response Code: $HTTP_CODE"
    echo "   Response Body:"
    echo "$RESPONSE" | grep -v "HTTP_CODE:"
fi
echo ""

echo "=== End of Diagnostics ==="
