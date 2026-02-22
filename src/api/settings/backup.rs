// 备份恢复相关接口
use super::common::{
    merge_settings, read_cache, require_version, respond_with_version, validate_path_internal,
    write_cache, CONFIG_CACHE_TTL, DB_CACHE_TTL,
};
use crate::common::state::AppState;
use crate::models::{ApiResponse, BackupRecord};
use axum::{extract::Path, extract::State, http::HeaderMap, response::IntoResponse, Json};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

// 备份路径校验请求体
#[derive(Deserialize)]
pub struct BackupPathRequest {
    pub path: String,
}

// 获取备份设置
pub async fn get_backup(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if let Some(cached) = read_cache(&state.backup_cache, CONFIG_CACHE_TTL).await {
        return respond_with_version(&state, ApiResponse::success(cached), false);
    }
    let settings = state.backup_settings.read().await.clone();
    write_cache(&state.backup_cache, settings.clone()).await;
    respond_with_version(&state, ApiResponse::success(settings), false)
}

// 更新备份设置
pub async fn update_backup(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(patch): Json<Value>,
) -> impl IntoResponse {
    if let Err(resp) = require_version(&headers, &state) {
        return respond_with_version(&state, resp, false);
    }
    let mut settings = state.backup_settings.write().await;
    match merge_settings(&settings.clone(), patch) {
        Ok(merged) => {
            *settings = merged;
            write_cache(&state.backup_cache, settings.clone()).await;
            respond_with_version(
                &state,
                ApiResponse::success("Backup settings updated".to_string()),
                true,
            )
        }
        Err(err) => respond_with_version(&state, ApiResponse::error(400, err), false),
    }
}

// 触发一次立即备份
pub async fn trigger_backup(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(resp) = require_version(&headers, &state) {
        return respond_with_version(&state, resp, false);
    }
    let mut backups = state.backups.write().await;
    let backup_id = Uuid::new_v4().to_string();

    backups.push(BackupRecord {
        id: backup_id.clone(),
        filename: format!(
            "backup_{}.tar.gz",
            chrono::Utc::now().format("%Y%m%d%H%M%S")
        ),
        size_bytes: 1024 * 1024 * 50,
        created_at: chrono::Utc::now().to_rfc3339(),
        status: "success".to_string(),
    });
    write_cache(&state.backups_cache, backups.clone()).await;

    respond_with_version(
        &state,
        ApiResponse::success("Backup triggered successfully".to_string()),
        true,
    )
}

// 获取备份记录列表
pub async fn get_backups(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if let Some(cached) = read_cache(&state.backups_cache, DB_CACHE_TTL).await {
        return respond_with_version(&state, ApiResponse::success(cached), false);
    }
    let backups = state.backups.read().await.clone();
    write_cache(&state.backups_cache, backups.clone()).await;
    respond_with_version(&state, ApiResponse::success(backups), false)
}

// 触发备份恢复
pub async fn restore_backup(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(resp) = require_version(&headers, &state) {
        return respond_with_version(&state, resp, false);
    }
    let backups = state.backups.read().await;
    if backups.iter().any(|b| b.id == id) {
        respond_with_version(
            &state,
            ApiResponse::success("Backup restore triggered".to_string()),
            true,
        )
    } else {
        respond_with_version(&state, ApiResponse::error(404, "Backup not found"), false)
    }
}

// 删除指定备份记录
pub async fn delete_backup(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(resp) = require_version(&headers, &state) {
        return respond_with_version(&state, resp, false);
    }
    let mut backups = state.backups.write().await;
    let before = backups.len();
    backups.retain(|b| b.id != id);
    if backups.len() == before {
        return respond_with_version(&state, ApiResponse::error(404, "Backup not found"), false);
    }
    write_cache(&state.backups_cache, backups.clone()).await;
    respond_with_version(
        &state,
        ApiResponse::success("Backup deleted".to_string()),
        true,
    )
}

// 校验备份路径合法性
pub async fn validate_backup_path(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<BackupPathRequest>,
) -> impl IntoResponse {
    match validate_path_internal(&payload.path) {
        Ok(result) => respond_with_version(&state, ApiResponse::success(result), false),
        Err(msg) => respond_with_version(&state, ApiResponse::error(400, msg), false),
    }
}
