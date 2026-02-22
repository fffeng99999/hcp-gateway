use crate::common::state::AppState;
use crate::models::{
    ApiResponse, BackupRecord, BackupSettings, GeneralSettings, NetworkSettings,
    NotificationSettings, SecuritySettings, StorageSettings, SystemUser,
};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use std::path::Path as FsPath;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use uuid::Uuid;

fn respond_with_version<T>(
    state: &Arc<AppState>,
    body: ApiResponse<T>,
    bump: bool,
) -> (HeaderMap, Json<ApiResponse<T>>)
where
    T: Serialize,
{
    let version = if bump {
        state.config_version.fetch_add(1, Ordering::SeqCst) + 1
    } else {
        state.config_version.load(Ordering::SeqCst)
    };

    let mut headers = HeaderMap::new();
    headers.insert("X-Config-Version", version.to_string().parse().unwrap());
    (headers, Json(body))
}

// ================= General Settings =================

pub async fn get_general(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let settings = state.general_settings.read().await;
    respond_with_version(&state, ApiResponse::success(settings.clone()), false)
}

pub async fn update_general(
    State(state): State<Arc<AppState>>,
    Json(new_settings): Json<GeneralSettings>,
) -> impl IntoResponse {
    let mut settings = state.general_settings.write().await;
    *settings = new_settings;
    respond_with_version(
        &state,
        ApiResponse::success("General settings updated".to_string()),
        true,
    )
}

// ================= Network Settings =================

pub async fn get_network(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let settings = state.network_settings.read().await;
    respond_with_version(&state, ApiResponse::success(settings.clone()), false)
}

pub async fn update_network(
    State(state): State<Arc<AppState>>,
    Json(new_settings): Json<NetworkSettings>,
) -> impl IntoResponse {
    let mut settings = state.network_settings.write().await;
    *settings = new_settings;
    respond_with_version(
        &state,
        ApiResponse::success("Network settings updated".to_string()),
        true,
    )
}

// ================= Storage Settings =================

pub async fn get_storage(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let settings = state.storage_settings.read().await;
    respond_with_version(&state, ApiResponse::success(settings.clone()), false)
}

pub async fn update_storage(
    State(state): State<Arc<AppState>>,
    Json(new_settings): Json<StorageSettings>,
) -> impl IntoResponse {
    let mut settings = state.storage_settings.write().await;
    *settings = new_settings;
    respond_with_version(
        &state,
        ApiResponse::success("Storage settings updated".to_string()),
        true,
    )
}

// ================= Security Settings =================

pub async fn get_security(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let settings = state.security_settings.read().await;
    respond_with_version(&state, ApiResponse::success(settings.clone()), false)
}

pub async fn update_security(
    State(state): State<Arc<AppState>>,
    Json(new_settings): Json<SecuritySettings>,
) -> impl IntoResponse {
    let mut settings = state.security_settings.write().await;
    *settings = new_settings;
    respond_with_version(
        &state,
        ApiResponse::success("Security settings updated".to_string()),
        true,
    )
}

// ================= Notification Settings =================

pub async fn get_notifications(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let settings = state.notification_settings.read().await;
    respond_with_version(&state, ApiResponse::success(settings.clone()), false)
}

pub async fn update_notifications(
    State(state): State<Arc<AppState>>,
    Json(new_settings): Json<NotificationSettings>,
) -> impl IntoResponse {
    let mut settings = state.notification_settings.write().await;
    *settings = new_settings;
    respond_with_version(
        &state,
        ApiResponse::success("Notification settings updated".to_string()),
        true,
    )
}

// ================= Backup Settings =================

pub async fn get_backup(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let settings = state.backup_settings.read().await;
    respond_with_version(&state, ApiResponse::success(settings.clone()), false)
}

pub async fn update_backup(
    State(state): State<Arc<AppState>>,
    Json(new_settings): Json<BackupSettings>,
) -> impl IntoResponse {
    let mut settings = state.backup_settings.write().await;
    *settings = new_settings;
    respond_with_version(
        &state,
        ApiResponse::success("Backup settings updated".to_string()),
        true,
    )
}

pub async fn trigger_backup(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut backups = state.backups.write().await;
    let backup_id = Uuid::new_v4().to_string();

    backups.push(BackupRecord {
        id: backup_id.clone(),
        filename: format!(
            "backup_{}.tar.gz",
            chrono::Utc::now().format("%Y%m%d%H%M%S")
        ),
        size_bytes: 1024 * 1024 * 50, // Mock size 50MB
        created_at: chrono::Utc::now().to_rfc3339(),
        status: "success".to_string(),
    });

    respond_with_version(
        &state,
        ApiResponse::success("Backup triggered successfully".to_string()),
        true,
    )
}

#[derive(Deserialize)]
pub struct BackupPathRequest {
    pub path: String,
}

pub async fn validate_backup_path(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<BackupPathRequest>,
) -> impl IntoResponse {
    let raw = payload.path.trim();
    if raw.is_empty() {
        return respond_with_version(
            &state,
            ApiResponse::error(400, "Backup path is empty".to_string()),
            false,
        );
    }

    let path = FsPath::new(raw);

    if !path.is_absolute() {
        return respond_with_version(
            &state,
            ApiResponse::error(400, "Backup path must be absolute".to_string()),
            false,
        );
    }

    let is_valid = if path.exists() {
        path.is_dir()
    } else if let Some(parent) = path.parent() {
        parent.exists() && parent.is_dir()
    } else {
        false
    };

    if !is_valid {
        return respond_with_version(
            &state,
            ApiResponse::error(400, "Backup path is not a valid directory".to_string()),
            false,
        );
    }

    let normalized = path.to_string_lossy().to_string();
    let result = json!({
        "is_valid": true,
        "normalized_path": normalized,
    });

    respond_with_version(&state, ApiResponse::success(result), false)
}

pub async fn get_users(State(state): State<Arc<AppState>>) -> Json<ApiResponse<Vec<SystemUser>>> {
    let users = state.users.read().await;
    Json(ApiResponse::success(users.clone()))
}

// POST /settings/users
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(mut user): Json<SystemUser>,
) -> Json<ApiResponse<String>> {
    let mut users = state.users.write().await;

    // Assign ID if not provided or empty
    if user.id.is_empty() {
        user.id = Uuid::new_v4().to_string();
    }
    user.created_at = chrono::Utc::now().to_rfc3339();

    users.push(user);
    Json(ApiResponse::success("User created".to_string()))
}

// PUT /settings/users/:id
pub async fn update_user(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(user_update): Json<SystemUser>,
) -> Json<ApiResponse<String>> {
    let mut users = state.users.write().await;

    if let Some(user) = users.iter_mut().find(|u| u.id == id) {
        user.username = user_update.username;
        user.role = user_update.role;
        user.email = user_update.email;
        // Don't update created_at or id
        Json(ApiResponse::success("User updated".to_string()))
    } else {
        Json(ApiResponse::error(404, "User not found"))
    }
}

// DELETE /settings/users/:id
pub async fn delete_user(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<String>> {
    let mut users = state.users.write().await;
    let initial_len = users.len();
    users.retain(|u| u.id != id);

    if users.len() < initial_len {
        Json(ApiResponse::success("User deleted".to_string()))
    } else {
        Json(ApiResponse::error(404, "User not found"))
    }
}
