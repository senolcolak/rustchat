#!/bin/bash
# Clear device tokens from backend database
# This forces all users to re-register their push notification tokens

echo "=== Clearing Device Tokens ==="
echo ""
echo "This will delete ALL device registrations from the database."
echo "Users will need to log out and back in to receive push notifications."
echo ""
read -p "Are you sure? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "Cancelled."
    exit 0
fi

# Check if running inside Docker
if docker ps | grep -q rustchat-postgres; then
    echo "Clearing tokens via Docker..."
    docker exec -i rustchat-postgres psql -U rustchat -d rustchat <<EOF
DELETE FROM user_devices;
SELECT COUNT(*) as remaining_tokens FROM user_devices;
EOF
else
    echo "Postgres container not running. Make sure the backend is running."
    exit 1
fi

echo ""
echo "=== Done ==="
echo "All device tokens have been cleared."
echo "Users need to log out and back in to get new tokens."
