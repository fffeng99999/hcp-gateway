// 设置模块路由与导出
pub mod backup;
pub mod common;
pub mod general;
pub mod network;
pub mod notification;
pub mod security;
pub mod storage;
pub mod system;
pub mod user;

// 备份恢复接口导出
pub use backup::{
    delete_backup, get_backup, get_backups, restore_backup, trigger_backup, update_backup,
    validate_backup_path,
};
// 通用设置接口导出
pub use general::{get_general, update_general};
// 网络配置接口导出
pub use network::{get_network, update_network};
// 通知设置接口导出
pub use notification::{get_notifications, update_notifications};
// 安全设置接口导出
pub use security::{get_security, update_security};
// 存储配置接口导出
pub use storage::{get_storage, update_storage, validate_storage_path};
// 系统信息接口导出
pub use system::get_system_info;
// 用户管理接口导出
pub use user::{
    create_user, delete_user, get_users, reset_user_password, update_user, validate_user,
};

// 设置模块接口测试
#[cfg(test)]
mod tests {
    use crate::api::router::create_router;
    use crate::common::state::AppState;
    use crate::models::SystemUser;
    use crate::utils::auth::create_token;
    use crate::utils::mock_data as data;
    use axum::body::{to_bytes, Body};
    use axum::http::{header, Method, Request, StatusCode};
    use serde_json::{json, Value};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use tower::ServiceExt;

    fn build_state() -> Arc<AppState> {
        Arc::new(AppState::new(
            data::default_mock_data(),
            None,
            None,
            Arc::new(AtomicBool::new(true)),
            Arc::new(AtomicBool::new(true)),
        ))
    }

    fn auth_token() -> String {
        let user = SystemUser {
            id: "test_user".to_string(),
            username: "admin".to_string(),
            role: "admin".to_string(),
            email: "admin@hcp.com".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            status: "active".to_string(),
            last_login: None,
        };
        create_token(&user).unwrap()
    }

    async fn send_request(
        app: axum::Router,
        method: Method,
        uri: &str,
        body: Option<Value>,
        token: Option<&str>,
        version: Option<u64>,
    ) -> (StatusCode, Value, axum::http::HeaderMap) {
        let mut builder = Request::builder().method(method).uri(uri);
        if let Some(token) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {}", token));
        }
        if let Some(version) = version {
            builder = builder.header("X-Config-Version", version.to_string());
        }
        let request = if let Some(body) = body {
            builder
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap()
        } else {
            builder.body(Body::empty()).unwrap()
        };
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, payload, headers)
    }

    async fn update_empty(
        app: axum::Router,
        state: &Arc<AppState>,
        token: &str,
        uri: &str,
    ) -> Value {
        let version = state.config_version.load(Ordering::SeqCst);
        let (_, body, _) = send_request(
            app,
            Method::PUT,
            uri,
            Some(json!({})),
            Some(token),
            Some(version),
        )
        .await;
        body
    }

    #[tokio::test]
    async fn test_general_settings() {
        let state = build_state();
        let app = create_router(state.clone());
        let token = auth_token();

        let (status, body, _) = send_request(
            app.clone(),
            Method::GET,
            "/api/v1/settings/general",
            None,
            Some(&token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["code"], 0);

        let body = update_empty(app.clone(), &state, &token, "/api/v1/settings/general").await;
        assert_eq!(body["code"], 0);

        let (_, body, _) = send_request(
            app,
            Method::PUT,
            "/api/v1/settings/general",
            Some(json!({})),
            Some(&token),
            None,
        )
        .await;
        assert_eq!(body["code"], 428);
    }

    #[tokio::test]
    async fn test_network_settings() {
        let state = build_state();
        let app = create_router(state.clone());
        let token = auth_token();

        let (status, body, _) = send_request(
            app.clone(),
            Method::GET,
            "/api/v1/settings/network",
            None,
            Some(&token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["code"], 0);

        let body = update_empty(app.clone(), &state, &token, "/api/v1/settings/network").await;
        assert_eq!(body["code"], 0);

        let (_, body, _) = send_request(
            app,
            Method::PUT,
            "/api/v1/settings/network",
            Some(json!({})),
            Some(&token),
            None,
        )
        .await;
        assert_eq!(body["code"], 428);
    }

    #[tokio::test]
    async fn test_storage_settings_and_path() {
        let state = build_state();
        let app = create_router(state.clone());
        let token = auth_token();

        let (status, body, _) = send_request(
            app.clone(),
            Method::GET,
            "/api/v1/settings/storage",
            None,
            Some(&token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["code"], 0);

        let body = update_empty(app.clone(), &state, &token, "/api/v1/settings/storage").await;
        assert_eq!(body["code"], 0);

        let (_, body, _) = send_request(
            app.clone(),
            Method::PUT,
            "/api/v1/settings/storage",
            Some(json!({})),
            Some(&token),
            None,
        )
        .await;
        assert_eq!(body["code"], 428);

        let (_, body, _) = send_request(
            app.clone(),
            Method::GET,
            "/api/v1/settings/storage/validatePath?path=/tmp",
            None,
            Some(&token),
            None,
        )
        .await;
        assert_eq!(body["code"], 0);

        let (_, body, _) = send_request(
            app,
            Method::GET,
            "/api/v1/settings/storage/validatePath?path=relative/path",
            None,
            Some(&token),
            None,
        )
        .await;
        assert_eq!(body["code"], 400);
    }

    #[tokio::test]
    async fn test_security_settings() {
        let state = build_state();
        let app = create_router(state.clone());
        let token = auth_token();

        let (status, body, _) = send_request(
            app.clone(),
            Method::GET,
            "/api/v1/settings/security",
            None,
            Some(&token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["code"], 0);

        let body = update_empty(app.clone(), &state, &token, "/api/v1/settings/security").await;
        assert_eq!(body["code"], 0);

        let (_, body, _) = send_request(
            app,
            Method::PUT,
            "/api/v1/settings/security",
            Some(json!({})),
            Some(&token),
            None,
        )
        .await;
        assert_eq!(body["code"], 428);
    }

    #[tokio::test]
    async fn test_notification_settings() {
        let state = build_state();
        let app = create_router(state.clone());
        let token = auth_token();

        let (status, body, _) = send_request(
            app.clone(),
            Method::GET,
            "/api/v1/settings/notification",
            None,
            Some(&token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["code"], 0);

        let body = update_empty(app.clone(), &state, &token, "/api/v1/settings/notification").await;
        assert_eq!(body["code"], 0);

        let (_, body, _) = send_request(
            app,
            Method::PUT,
            "/api/v1/settings/notification",
            Some(json!({})),
            Some(&token),
            None,
        )
        .await;
        assert_eq!(body["code"], 428);
    }

    #[tokio::test]
    async fn test_backup_settings_and_records() {
        let state = build_state();
        let app = create_router(state.clone());
        let token = auth_token();

        let (status, body, _) = send_request(
            app.clone(),
            Method::GET,
            "/api/v1/settings/backup",
            None,
            Some(&token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["code"], 0);

        let body = update_empty(app.clone(), &state, &token, "/api/v1/settings/backup").await;
        assert_eq!(body["code"], 0);

        let version = state.config_version.load(Ordering::SeqCst);
        let (_, body, _) = send_request(
            app.clone(),
            Method::POST,
            "/api/v1/settings/backup/trigger",
            None,
            Some(&token),
            Some(version),
        )
        .await;
        assert_eq!(body["code"], 0);

        let (_, list_body, _) = send_request(
            app.clone(),
            Method::GET,
            "/api/v1/settings/backups",
            None,
            Some(&token),
            None,
        )
        .await;
        assert_eq!(list_body["code"], 0);
        let backup_id = list_body["data"][0]["id"].as_str().unwrap().to_string();

        let version = state.config_version.load(Ordering::SeqCst);
        let (_, body, _) = send_request(
            app.clone(),
            Method::POST,
            &format!("/api/v1/settings/backup/restore/{}", backup_id),
            None,
            Some(&token),
            Some(version),
        )
        .await;
        assert_eq!(body["code"], 0);

        let version = state.config_version.load(Ordering::SeqCst);
        let (_, body, _) = send_request(
            app.clone(),
            Method::DELETE,
            &format!("/api/v1/settings/backups/{}", backup_id),
            None,
            Some(&token),
            Some(version),
        )
        .await;
        assert_eq!(body["code"], 0);

        let (_, body, _) = send_request(
            app,
            Method::DELETE,
            "/api/v1/settings/backups/unknown",
            None,
            Some(&token),
            None,
        )
        .await;
        assert_eq!(body["code"], 428);
    }

    #[tokio::test]
    async fn test_users_endpoints() {
        let state = build_state();
        let app = create_router(state.clone());
        let token = auth_token();

        let (status, body, _) = send_request(
            app.clone(),
            Method::GET,
            "/api/v1/settings/users",
            None,
            Some(&token),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["code"], 0);

        {
            let mut users = state.users.write().await;
            users.clear();
            let mut cache = state.users_cache.write().await;
            cache.value = None;
            cache.updated_at = None;
        }
        let (_, body, _) = send_request(
            app.clone(),
            Method::GET,
            "/api/v1/settings/users",
            None,
            Some(&token),
            None,
        )
        .await;
        assert_eq!(body["code"], 0);

        let version = state.config_version.load(Ordering::SeqCst);
        let (_, body, _) = send_request(
            app.clone(),
            Method::POST,
            "/api/v1/settings/users",
            Some(json!({"username":"user1","email":"user1@hcp.com","role":"admin"})),
            Some(&token),
            Some(version),
        )
        .await;
        assert_eq!(body["code"], 0);

        let (_, body, _) = send_request(
            app.clone(),
            Method::POST,
            "/api/v1/settings/users",
            Some(json!({"username":"user2","email":"user2@hcp.com"})),
            Some(&token),
            None,
        )
        .await;
        assert_eq!(body["code"], 428);

        let (_, list_body, _) = send_request(
            app.clone(),
            Method::GET,
            "/api/v1/settings/users",
            None,
            Some(&token),
            None,
        )
        .await;
        let user_id = list_body["data"][0]["id"].as_str().unwrap().to_string();

        let version = state.config_version.load(Ordering::SeqCst);
        let (_, body, _) = send_request(
            app.clone(),
            Method::PUT,
            &format!("/api/v1/settings/users/{}", user_id),
            Some(json!({"status":"inactive"})),
            Some(&token),
            Some(version),
        )
        .await;
        assert_eq!(body["code"], 0);

        let version = state.config_version.load(Ordering::SeqCst);
        let (_, body, _) = send_request(
            app.clone(),
            Method::POST,
            &format!("/api/v1/settings/users/{}/reset-password", user_id),
            None,
            Some(&token),
            Some(version),
        )
        .await;
        assert_eq!(body["code"], 0);

        let version = state.config_version.load(Ordering::SeqCst);
        let (_, body, _) = send_request(
            app.clone(),
            Method::DELETE,
            &format!("/api/v1/settings/users/{}", user_id),
            None,
            Some(&token),
            Some(version),
        )
        .await;
        assert_eq!(body["code"], 0);

        let (_, body, _) = send_request(
            app.clone(),
            Method::POST,
            "/api/v1/settings/user/validate",
            Some(json!({"username":"","email":"invalid"})),
            Some(&token),
            None,
        )
        .await;
        assert_eq!(body["code"], 0);
        assert_eq!(body["data"]["valid"], false);

        let (_, body, _) = send_request(
            app,
            Method::DELETE,
            "/api/v1/settings/users/missing",
            None,
            Some(&token),
            None,
        )
        .await;
        assert_eq!(body["code"], 428);
    }
}
