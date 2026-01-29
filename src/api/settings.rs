use crate::models::{
    ApiResponse, BackupRecord, BackupSettings, GeneralSettings, NetworkSettings, 
    NotificationSettings, SecuritySettings, StorageSettings, SystemUser
};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

// ================= General Settings =================

// GET /settings/general
pub async fn get_general(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<GeneralSettings>> {
    let settings = state.general_settings.read().await;
    Json(ApiResponse::success(settings.clone()))
}

// PUT /settings/general
pub async fn update_general(
    State(state): State<Arc<AppState>>,
    Json(new_settings): Json<GeneralSettings>,
) -> Json<ApiResponse<String>> {
    let mut settings = state.general_settings.write().await;
    *settings = new_settings;
    Json(ApiResponse::success("General settings updated".to_string()))
}

// ================= Network Settings =================

// GET /settings/network
pub async fn get_network(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<NetworkSettings>> {
    let settings = state.network_settings.read().await;
    Json(ApiResponse::success(settings.clone()))
}

// PUT /settings/network
pub async fn update_network(
    State(state): State<Arc<AppState>>,
    Json(new_settings): Json<NetworkSettings>,
) -> Json<ApiResponse<String>> {
    let mut settings = state.network_settings.write().await;
    *settings = new_settings;
    Json(ApiResponse::success("Network settings updated".to_string()))
}

// ================= Storage Settings =================

// GET /settings/storage
pub async fn get_storage(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<StorageSettings>> {
    let settings = state.storage_settings.read().await;
    Json(ApiResponse::success(settings.clone()))
}

// PUT /settings/storage
pub async fn update_storage(
    State(state): State<Arc<AppState>>,
    Json(new_settings): Json<StorageSettings>,
) -> Json<ApiResponse<String>> {
    let mut settings = state.storage_settings.write().await;
    *settings = new_settings;
    Json(ApiResponse::success("Storage settings updated".to_string()))
}

// ================= Security Settings =================

// GET /settings/security
pub async fn get_security(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<SecuritySettings>> {
    let settings = state.security_settings.read().await;
    Json(ApiResponse::success(settings.clone()))
}

// PUT /settings/security
pub async fn update_security(
    State(state): State<Arc<AppState>>,
    Json(new_settings): Json<SecuritySettings>,
) -> Json<ApiResponse<String>> {
    let mut settings = state.security_settings.write().await;
    *settings = new_settings;
    Json(ApiResponse::success("Security settings updated".to_string()))
}

// ================= Notification Settings =================

// GET /settings/notifications
pub async fn get_notifications(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<NotificationSettings>> {
    let settings = state.notification_settings.read().await;
    Json(ApiResponse::success(settings.clone()))
}

// PUT /settings/notifications
pub async fn update_notifications(
    State(state): State<Arc<AppState>>,
    Json(new_settings): Json<NotificationSettings>,
) -> Json<ApiResponse<String>> {
    let mut settings = state.notification_settings.write().await;
    *settings = new_settings;
    Json(ApiResponse::success("Notification settings updated".to_string()))
}

// ================= Backup Settings =================

// GET /settings/backup
pub async fn get_backup(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<BackupSettings>> {
    let settings = state.backup_settings.read().await;
    Json(ApiResponse::success(settings.clone()))
}

// PUT /settings/backup
pub async fn update_backup(
    State(state): State<Arc<AppState>>,
    Json(new_settings): Json<BackupSettings>,
) -> Json<ApiResponse<String>> {
    let mut settings = state.backup_settings.write().await;
    *settings = new_settings;
    Json(ApiResponse::success("Backup settings updated".to_string()))
}

// POST /settings/backup/trigger
pub async fn trigger_backup(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<String>> {
    let mut backups = state.backups.write().await;
    let backup_id = Uuid::new_v4().to_string();
    
    backups.push(BackupRecord {
        id: backup_id.clone(),
        filename: format!("backup_{}.tar.gz", chrono::Utc::now().format("%Y%m%d%H%M%S")),
        size_bytes: 1024 * 1024 * 50, // Mock size 50MB
        created_at: chrono::Utc::now().to_rfc3339(),
        status: "success".to_string(),
    });
    
    Json(ApiResponse::success("Backup triggered successfully".to_string()))
}

// ================= User Settings =================

// GET /settings/users
pub async fn get_users(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<SystemUser>>> {
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
