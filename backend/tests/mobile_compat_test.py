import requests
import sys
import os
import time

# Configuration
BASE_URL = os.environ.get("RUSTCHAT_URL", "http://localhost:8080")
USERNAME = os.environ.get("RUSTCHAT_USER", "sysadmin")
PASSWORD = os.environ.get("RUSTCHAT_PASS", "sysadmin")
TEAM_NAME = os.environ.get("RUSTCHAT_TEAM", "ad-hoc")
CHANNEL_NAME = os.environ.get("RUSTCHAT_CHANNEL", "town-square")

def get_session():
    s = requests.Session()
    print(f"Logging in as {USERNAME}...")
    r = s.post(f"{BASE_URL}/api/v4/users/login", json={"login_id": USERNAME, "password": PASSWORD})
    if r.status_code != 200:
        print(f"Login failed: {r.status_code} {r.text}")
        sys.exit(1)
    token = r.headers.get("Token")
    if token:
        s.headers.update({"Authorization": f"Bearer {token}"})
    print("Login successful.")
    return s

def get_channel_and_team(s):
    """Get first available team and channel"""
    r = s.get(f"{BASE_URL}/api/v4/users/me/teams")
    teams = r.json()
    if not teams:
        print("No teams found for user.")
        sys.exit(1)
    team = teams[0]
    
    r = s.get(f"{BASE_URL}/api/v4/teams/{team['id']}/channels")
    channels = r.json()
    channel = next((c for c in channels if c['name'] == 'town-square'), channels[0])
    
    return team, channel

def create_dummy_image():
    # 1x1 transparent GIF
    return b'GIF89a\x01\x00\x01\x00\x80\x00\x00\x00\x00\x00\xff\xff\xff!\xf9\x04\x01\x00\x00\x00\x00,\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x01D\x00;'

def test_mobile_upload(s, channel_id):
    print("\n=== Testing Mobile Upload ===")
    img_data = create_dummy_image()
    files = {'some_random_field_name': ('mobile_test.gif', img_data, 'image/gif')}
    
    r = s.post(f"{BASE_URL}/api/v4/files", files=files, data={'channel_id': channel_id})
    if r.status_code not in [200, 201]:
        print(f"Upload failed: {r.status_code} {r.text}")
        return None
    
    resp = r.json()
    if 'file_infos' not in resp or len(resp['file_infos']) == 0:
        print("No file_infos returned!")
        return None
        
    file_id = resp['file_infos'][0]['id']
    print(f"✓ Upload successful, File ID: {file_id}")
    return file_id

def test_channel_bookmarks(s, channel_id):
    print("\n=== Testing Channel Bookmarks API ===")
    
    # Create bookmark
    r = s.post(f"{BASE_URL}/api/v4/channels/{channel_id}/bookmarks", json={
        "display_name": "Test Bookmark",
        "type": "link",
        "link_url": "https://example.com",
        "emoji": "bookmark"
    })
    if r.status_code not in [200, 201]:
        print(f"✗ Create bookmark failed: {r.status_code} {r.text}")
        return None
    bookmark = r.json()
    bookmark_id = bookmark.get('id')
    print(f"✓ Created bookmark: {bookmark_id}")
    
    # List bookmarks
    r = s.get(f"{BASE_URL}/api/v4/channels/{channel_id}/bookmarks")
    if r.status_code != 200:
        print(f"✗ List bookmarks failed: {r.status_code}")
        return bookmark_id
    print(f"✓ Listed {len(r.json())} bookmarks")
    
    # Update bookmark
    r = s.patch(f"{BASE_URL}/api/v4/channels/{channel_id}/bookmarks/{bookmark_id}", json={
        "display_name": "Updated Bookmark"
    })
    if r.status_code == 200:
        print("✓ Updated bookmark")
    else:
        print(f"✗ Update bookmark failed: {r.status_code}")
    
    # Delete bookmark
    r = s.delete(f"{BASE_URL}/api/v4/channels/{channel_id}/bookmarks/{bookmark_id}")
    if r.status_code in [200, 204]:
        print("✓ Deleted bookmark")
    else:
        print(f"✗ Delete bookmark failed: {r.status_code}")
    
    return bookmark_id

def test_file_search(s, team_id):
    print("\n=== Testing File Search API ===")
    
    # Global search
    r = s.post(f"{BASE_URL}/api/v4/files/search", json={"terms": "test"})
    if r.status_code == 200:
        result = r.json()
        print(f"✓ Global file search returned {len(result.get('order', []))} results")
    else:
        print(f"✗ Global file search failed: {r.status_code}")
    
    # Team search
    r = s.post(f"{BASE_URL}/api/v4/teams/{team_id}/files/search", json={"terms": "test"})
    if r.status_code == 200:
        result = r.json()
        print(f"✓ Team file search returned {len(result.get('order', []))} results")
    else:
        print(f"✗ Team file search failed: {r.status_code}")

def test_custom_profile_attributes(s, user_id):
    print("\n=== Testing Custom Profile Attributes API ===")
    
    # Get fields
    r = s.get(f"{BASE_URL}/api/v4/custom_profile_attributes/fields")
    if r.status_code == 200:
        print(f"✓ Got {len(r.json())} custom profile fields")
    else:
        print(f"✗ Get fields failed: {r.status_code}")
    
    # Get user attributes
    r = s.get(f"{BASE_URL}/api/v4/users/{user_id}/custom_profile_attributes")
    if r.status_code == 200:
        print(f"✓ Got user custom attributes: {len(r.json())} values")
    else:
        print(f"✗ Get user attributes failed: {r.status_code}")

def test_scheduled_posts(s, channel_id, team_id):
    print("\n=== Testing Scheduled Posts API ===")
    
    # Create scheduled post
    scheduled_time = int((time.time() + 3600) * 1000)  # 1 hour from now
    r = s.post(f"{BASE_URL}/api/v4/posts/schedule", json={
        "channel_id": channel_id,
        "message": "This is a test scheduled post",
        "scheduled_at": scheduled_time
    })
    if r.status_code not in [200, 201]:
        print(f"✗ Create scheduled post failed: {r.status_code} {r.text}")
        return
    post = r.json()
    post_id = post.get('id')
    print(f"✓ Created scheduled post: {post_id}")
    
    # List scheduled posts
    r = s.get(f"{BASE_URL}/api/v4/posts/scheduled/team/{team_id}")
    if r.status_code == 200:
        print(f"✓ Listed scheduled posts")
    else:
        print(f"✗ List scheduled posts failed: {r.status_code}")
    
    # Delete scheduled post
    if post_id:
        r = s.delete(f"{BASE_URL}/api/v4/posts/schedule/{post_id}")
        if r.status_code in [200, 204]:
            print("✓ Deleted scheduled post")
        else:
            print(f"✗ Delete scheduled post failed: {r.status_code}")

def test_data_retention_policies(s, user_id):
    print("\n=== Testing User Data Retention Policies ===")
    
    # Get user team policies
    r = s.get(f"{BASE_URL}/api/v4/users/{user_id}/data_retention/team_policies")
    if r.status_code == 200:
        print("✓ Got user team retention policies")
    else:
        print(f"✗ Get team policies failed: {r.status_code}")
    
    # Get user channel policies
    r = s.get(f"{BASE_URL}/api/v4/users/{user_id}/data_retention/channel_policies")
    if r.status_code == 200:
        print("✓ Got user channel retention policies")
    else:
        print(f"✗ Get channel policies failed: {r.status_code}")

def test_nps(s):
    print("\n=== Testing NPS Endpoint ===")
    
    r = s.post(f"{BASE_URL}/api/v4/nps", json={"score": 10, "feedback": "Great!"})
    if r.status_code == 200:
        print("✓ NPS submission accepted")
    else:
        print(f"✗ NPS submission failed: {r.status_code}")

def main():
    s = get_session()
    team, channel = get_channel_and_team(s)
    channel_id = channel['id']
    team_id = team['id']
    
    # Get user ID
    r = s.get(f"{BASE_URL}/api/v4/users/me")
    user_id = r.json()['id']
    
    print(f"\nTeam: {team['name']} ({team_id})")
    print(f"Channel: {channel['name']} ({channel_id})")
    print(f"User: {user_id}")
    
    # Run all tests
    passed = 0
    failed = 0
    
    try:
        test_mobile_upload(s, channel_id)
        test_channel_bookmarks(s, channel_id)
        test_file_search(s, team_id)
        test_custom_profile_attributes(s, user_id)
        test_scheduled_posts(s, channel_id, team_id)
        test_data_retention_policies(s, user_id)
        test_nps(s)
        
        print("\n" + "="*50)
        print("ALL MOBILE COMPATIBILITY TESTS COMPLETED")
        print("="*50)
        
    except Exception as e:
        print(f"\nTest error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()

