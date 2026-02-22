// 存储配置相关接口
use super::common::{
    merge_settings, read_cache, require_version, respond_with_version, validate_path_internal,
    write_cache, CONFIG_CACHE_TTL,
};
use crate::common::state::AppState;
use crate::models::ApiResponse;
use axum::{extract::Query, extract::State, http::HeaderMap, response::IntoResponse, Json};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use sysinfo::Disks;
use std::path::Path;
use tokio::task;

// 存储路径校验请求参数
#[derive(Deserialize)]
pub struct ValidatePathQuery {
    pub path: String,
}

// Helper to calculate directory size
fn get_dir_size(path: impl AsRef<Path>) -> u64 {
    let mut size = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    size += get_dir_size(entry.path());
                } else {
                    size += metadata.len();
                }
            }
        }
    }
    size
}

// Helper to get disk total space
fn get_disk_total_space(path: &str) -> u64 {
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        if Path::new(path).starts_with(disk.mount_point()) {
            return disk.total_space();
        }
    }
    // Fallback to finding root
    for disk in &disks {
         if disk.mount_point() == Path::new("/") {
             return disk.total_space();
         }
    }
    0
}

// 获取存储配置
pub async fn get_storage(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // 每次都获取最新的存储使用情况，不使用缓存
    // if let Some(cached) = read_cache(&state.storage_cache, CONFIG_CACHE_TTL).await {
    //     return respond_with_version(&state, ApiResponse::success(cached), false);
    // }
    
    let mut settings = state.storage_settings.read().await.clone();
    
    // Calculate storage usage
    // Hardcoded path for hcp-project as requested
    let project_path = "/home/hcp-dev/hcp-project";
    
    // Run in blocking task to avoid blocking async runtime
    let used = tokio::task::spawn_blocking(move || get_dir_size(project_path)).await.unwrap_or(0);
    let total = tokio::task::spawn_blocking(move || get_disk_total_space(project_path)).await.unwrap_or(0);
    
    settings.storage_used = Some(used); // bytes
    settings.storage_total = Some(total); // bytes
    
    // Set DB types (Read-only display)
    settings.backend_db_type = "PostgreSQL".to_string();
    settings.blockchain_db_type = "RocksDB".to_string(); // Assuming RocksDB for blockchain

    // We can still cache it if we want, but usage changes frequently. 
    // Given the requirement for "Current storage usage", real-time is better.
    // write_cache(&state.storage_cache, settings.clone()).await;
    
    respond_with_version(&state, ApiResponse::success(settings), false)
}

// 更新存储配置
pub async fn update_storage(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(patch): Json<Value>,
) -> impl IntoResponse {
    if let Err(resp) = require_version(&headers, &state) {
        return respond_with_version(&state, resp, false);
    }
    let mut settings = state.storage_settings.write().await;
    match merge_settings(&settings.clone(), patch) {
        Ok(merged) => {
            *settings = merged;
            write_cache(&state.storage_cache, settings.clone()).await;
            respond_with_version(
                &state,
                ApiResponse::success("Storage settings updated".to_string()),
                true,
            )
        }
        Err(err) => respond_with_version(&state, ApiResponse::error(400, err), false),
    }
}

// 校验存储路径合法性
pub async fn validate_storage_path(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ValidatePathQuery>,
) -> impl IntoResponse {
    match validate_path_internal(&query.path) {
        Ok(result) => respond_with_version(&state, ApiResponse::success(result), false),
        Err(msg) => respond_with_version(&state, ApiResponse::error(400, msg), false),
    }
}
