use std::time::{Duration, Instant};

use futures_util::StreamExt;
use reqwest::StatusCode;
use rustchat::mattermost_compat::id::encode_mm_id;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

use crate::common::spawn_app;

mod common;

type WsClient =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

#[tokio::test]
async fn mobile_presence_lifecycle_resets_manual_status_on_disconnect() {
    let app = spawn_app().await;
    configure_presence_grace_seconds(&app, 1).await;

    let org_id = insert_org(&app, "Presence Lifecycle Org").await;
    let (token, user_id) =
        register_and_login(&app, org_id, "presence_user", "presence_user@example.com").await;

    let mut ws = connect_ws_v4(&app.address, &token).await;
    let _ = wait_for_event(&mut ws, "hello", Duration::from_secs(5)).await;

    let set_dnd = app
        .api_client
        .put(format!("{}/api/v4/users/me/status", app.address))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({
            "user_id": encode_mm_id(user_id),
            "status": "dnd"
        }))
        .send()
        .await
        .expect("status update should succeed");
    assert_eq!(set_dnd.status(), StatusCode::OK);

    let dnd_status = poll_my_status(&app, &token, Duration::from_secs(3)).await;
    assert_eq!(dnd_status["status"], "dnd");
    assert_eq!(dnd_status["manual"], true);

    ws.close(None)
        .await
        .expect("websocket close frame should be sent");
    drop(ws);

    let offline_status =
        wait_for_status(&app, &token, "offline", false, Duration::from_secs(8)).await;
    assert_eq!(offline_status["status"], "offline");
    assert_eq!(offline_status["manual"], false);

    let mut ws_reconnected = connect_ws_v4(&app.address, &token).await;
    let _ = wait_for_event(&mut ws_reconnected, "hello", Duration::from_secs(5)).await;

    let online_status =
        wait_for_status(&app, &token, "online", false, Duration::from_secs(8)).await;
    assert_eq!(online_status["status"], "online");
    assert_eq!(online_status["manual"], false);

    let _ = ws_reconnected.close(None).await;
}

#[tokio::test]
async fn mobile_background_disconnect_does_not_flip_offline_within_grace_window() {
    let app = spawn_app().await;
    configure_presence_grace_seconds(&app, 20).await;

    let org_id = insert_org(&app, "Presence Grace Org").await;
    let (token, user_id) = register_and_login(
        &app,
        org_id,
        "presence_grace_user",
        "presence_grace_user@example.com",
    )
    .await;

    let mut ws = connect_ws_v4(&app.address, &token).await;
    let _ = wait_for_event(&mut ws, "hello", Duration::from_secs(5)).await;

    let set_dnd = app
        .api_client
        .put(format!("{}/api/v4/users/me/status", app.address))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({
            "user_id": encode_mm_id(user_id),
            "status": "dnd"
        }))
        .send()
        .await
        .expect("status update should succeed");
    assert_eq!(set_dnd.status(), StatusCode::OK);

    ws.close(None)
        .await
        .expect("websocket close frame should be sent");
    drop(ws);

    tokio::time::sleep(Duration::from_secs(2)).await;

    let status_during_grace = poll_my_status(&app, &token, Duration::from_secs(3)).await;
    assert_eq!(status_during_grace["status"], "dnd");
    assert_eq!(status_during_grace["manual"], true);

    let mut ws_reconnected = connect_ws_v4(&app.address, &token).await;
    let _ = wait_for_event(&mut ws_reconnected, "hello", Duration::from_secs(5)).await;

    let online_status =
        wait_for_status(&app, &token, "online", false, Duration::from_secs(8)).await;
    assert_eq!(online_status["status"], "online");
    assert_eq!(online_status["manual"], false);

    let _ = ws_reconnected.close(None).await;
}

async fn configure_presence_grace_seconds(app: &common::TestApp, seconds: i32) {
    sqlx::query(
        r#"
        UPDATE server_config
        SET site = jsonb_set(
            COALESCE(site, '{}'::jsonb),
            '{mobile_presence_disconnect_grace_seconds}',
            to_jsonb($1::int),
            true
        )
        WHERE id = 'default'
        "#,
    )
    .bind(seconds)
    .execute(&app.db_pool)
    .await
    .expect("failed to update presence grace seconds");
}

async fn poll_my_status(app: &common::TestApp, token: &str, within: Duration) -> serde_json::Value {
    let deadline = Instant::now() + within;
    loop {
        let res = app
            .api_client
            .get(format!("{}/api/v4/users/me/status", app.address))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .expect("status request should succeed");
        if res.status() == StatusCode::OK {
            return res
                .json::<serde_json::Value>()
                .await
                .expect("status response should be valid json");
        }

        assert!(
            Instant::now() < deadline,
            "timed out polling my status endpoint"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_status(
    app: &common::TestApp,
    token: &str,
    expected_status: &str,
    expected_manual: bool,
    within: Duration,
) -> serde_json::Value {
    let deadline = Instant::now() + within;

    loop {
        let status = poll_my_status(app, token, Duration::from_secs(2)).await;
        let status_value = status["status"].as_str().unwrap_or_default();
        let manual_value = status["manual"].as_bool().unwrap_or(false);
        if status_value == expected_status && manual_value == expected_manual {
            return status;
        }

        assert!(
            Instant::now() < deadline,
            "timed out waiting for status={expected_status} manual={expected_manual}; got status={status_value} manual={manual_value}"
        );
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
}

async fn wait_for_event(
    ws: &mut WsClient,
    expected_event: &str,
    within: Duration,
) -> serde_json::Value {
    let deadline = Instant::now() + within;

    loop {
        let now = Instant::now();
        assert!(
            now < deadline,
            "timed out waiting for websocket event {expected_event}"
        );

        let timeout_left = deadline.saturating_duration_since(now);
        let message = tokio::time::timeout(timeout_left, ws.next())
            .await
            .expect("timeout while waiting for websocket frame")
            .expect("websocket closed unexpectedly")
            .expect("websocket frame should be valid");

        if let Message::Text(text) = message {
            let parsed: serde_json::Value =
                serde_json::from_str(&text).expect("frame should be valid JSON");
            if parsed["event"] == expected_event {
                return parsed["data"].clone();
            }
        }
    }
}

async fn connect_ws_v4(base_http_url: &str, token: &str) -> WsClient {
    let ws_base = base_http_url.replacen("http://", "ws://", 1);
    let ws_url = format!("{ws_base}/api/v4/websocket?token={token}");
    let (ws_stream, _) = connect_async(ws_url)
        .await
        .expect("websocket connection should succeed");
    ws_stream
}

async fn insert_org(app: &common::TestApp, name: &str) -> Uuid {
    let org_id = Uuid::new_v4();
    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
        .bind(org_id)
        .bind(name)
        .execute(&app.db_pool)
        .await
        .expect("failed to create organization");
    org_id
}

async fn register_and_login(
    app: &common::TestApp,
    org_id: Uuid,
    username: &str,
    email: &str,
) -> (String, Uuid) {
    app.api_client
        .post(format!("{}/api/v1/auth/register", app.address))
        .json(&serde_json::json!({
            "username": username,
            "email": email,
            "password": "Password123!",
            "display_name": username,
            "org_id": org_id,
        }))
        .send()
        .await
        .expect("register request failed")
        .error_for_status()
        .expect("register should succeed");

    let login = app
        .api_client
        .post(format!("{}/api/v4/users/login", app.address))
        .json(&serde_json::json!({
            "login_id": email,
            "password": "Password123!",
        }))
        .send()
        .await
        .expect("login request failed")
        .error_for_status()
        .expect("login should succeed");

    let token = login
        .headers()
        .get("Token")
        .and_then(|v| v.to_str().ok())
        .expect("token header missing")
        .to_string();

    let me = app
        .api_client
        .get(format!("{}/api/v4/users/me", app.address))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("me request failed")
        .error_for_status()
        .expect("me should succeed")
        .json::<serde_json::Value>()
        .await
        .expect("me response should be JSON");

    let user_id = me["id"]
        .as_str()
        .and_then(rustchat::mattermost_compat::id::parse_mm_or_uuid)
        .expect("user id should parse");

    (token, user_id)
}
