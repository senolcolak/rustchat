use crate::common::spawn_app;
use reqwest::{Response, StatusCode};
use uuid::Uuid;

mod common;

#[tokio::test]
async fn plugins_read_endpoints_reflect_calls_plugin_state() {
    let app = spawn_app().await;
    let token = create_user_and_login(&app).await;

    // Calls plugin is enabled by default in server_config migration.
    let plugins_res = app
        .api_client
        .get(format!("{}/api/v4/plugins", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(plugins_res.status(), StatusCode::OK);
    let plugins_body: serde_json::Value = plugins_res.json().await.unwrap();
    assert!(plugins_body["active"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["id"] == "com.mattermost.calls"));

    // Disable calls plugin and verify read endpoints reflect actual state.
    sqlx::query(
        "UPDATE server_config SET plugins = jsonb_set(plugins, '{calls,enabled}', 'false'::jsonb, true) WHERE id = 'default'",
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    let plugins_res = app
        .api_client
        .get(format!("{}/api/v4/plugins", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(plugins_res.status(), StatusCode::OK);
    let plugins_body: serde_json::Value = plugins_res.json().await.unwrap();
    assert!(plugins_body["inactive"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["id"] == "com.mattermost.calls"));

    let plugin_status_res = app
        .api_client
        .get(format!(
            "{}/api/v4/plugins/com.mattermost.calls",
            &app.address
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(plugin_status_res.status(), StatusCode::OK);
    let plugin_status: serde_json::Value = plugin_status_res.json().await.unwrap();
    assert_eq!(plugin_status["id"], "com.mattermost.calls");
    assert_eq!(plugin_status["active"], false);

    let statuses_res = app
        .api_client
        .get(format!("{}/api/v4/plugins/statuses", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(statuses_res.status(), StatusCode::OK);
    let statuses_body: serde_json::Value = statuses_res.json().await.unwrap();
    assert_eq!(statuses_body[0]["plugin_id"], "com.mattermost.calls");
    assert_eq!(statuses_body[0]["is_active"], false);

    let webapp_res = app
        .api_client
        .get(format!("{}/api/v4/plugins/webapp", &app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(webapp_res.status(), StatusCode::OK);
    let webapp_body: serde_json::Value = webapp_res.json().await.unwrap();
    assert!(webapp_body.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn plugin_mutations_return_explicit_mm_501() {
    let app = spawn_app().await;
    let token = create_user_and_login(&app).await;

    let responses =
        vec![
        app.api_client
            .post(format!("{}/api/v4/plugins", &app.address))
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({"plugin": "ignored"}))
            .send()
            .await
            .unwrap(),
        app.api_client
            .post(format!("{}/api/v4/plugins/install_from_url", &app.address))
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({"plugin_download_url": "https://example.com/plugin.tar.gz"}))
            .send()
            .await
            .unwrap(),
        app.api_client
            .delete(format!("{}/api/v4/plugins/com.mattermost.calls", &app.address))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap(),
        app.api_client
            .post(format!(
                "{}/api/v4/plugins/com.mattermost.calls/enable",
                &app.address
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap(),
        app.api_client
            .post(format!(
                "{}/api/v4/plugins/com.mattermost.calls/disable",
                &app.address
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap(),
        app.api_client
            .post(format!(
                "{}/api/v4/plugins/marketplace/first_admin_visit",
                &app.address
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap(),
    ];

    for response in responses {
        assert_mm_not_implemented(response).await;
    }
}

#[tokio::test]
async fn dialogs_endpoints_return_explicit_mm_501() {
    let app = spawn_app().await;
    let token = create_user_and_login(&app).await;

    let responses = vec![
        app.api_client
            .post(format!("{}/api/v4/actions/dialogs/open", &app.address))
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({}))
            .send()
            .await
            .unwrap(),
        app.api_client
            .post(format!("{}/api/v4/actions/dialogs/submit", &app.address))
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({}))
            .send()
            .await
            .unwrap(),
        app.api_client
            .post(format!("{}/api/v4/actions/dialogs/lookup", &app.address))
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({}))
            .send()
            .await
            .unwrap(),
    ];

    for response in responses {
        assert_mm_not_implemented(response).await;
    }
}

async fn create_user_and_login(app: &common::TestApp) -> String {
    let org_id = Uuid::new_v4();
    sqlx::query("INSERT INTO organizations (id, name) VALUES ($1, $2)")
        .bind(org_id)
        .bind("Plugin Test Org")
        .execute(&app.db_pool)
        .await
        .unwrap();

    let user_data = serde_json::json!({
        "username": "plugin_user",
        "email": "plugin_user@example.com",
        "password": "Password123!",
        "display_name": "Plugin User",
        "org_id": org_id
    });

    app.api_client
        .post(format!("{}/api/v1/auth/register", &app.address))
        .json(&user_data)
        .send()
        .await
        .unwrap();

    let login_data = serde_json::json!({
        "login_id": "plugin_user@example.com",
        "password": "Password123!"
    });

    let login_res = app
        .api_client
        .post(format!("{}/api/v4/users/login", &app.address))
        .json(&login_data)
        .send()
        .await
        .unwrap();

    assert_eq!(login_res.status(), StatusCode::OK);

    login_res
        .headers()
        .get("Token")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

async fn assert_mm_not_implemented(response: Response) {
    assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["id"].is_string());
    assert!(body["message"].is_string());
    assert!(body["detailed_error"].is_string());
    assert!(body["request_id"].is_string());
    assert_eq!(body["status_code"], 501);
}
