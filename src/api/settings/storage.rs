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

// 存储路径校验请求参数
#[derive(Deserialize)]
pub struct ValidatePathQuery {
    pub path: String,
}

// 获取存储配置
pub async fn get_storage(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if let Some(cached) = read_cache(&state.storage_cache, CONFIG_CACHE_TTL).await {
        return respond_with_version(&state, ApiResponse::success(cached), false);
    }
    let settings = state.storage_settings.read().await.clone();
    write_cache(&state.storage_cache, settings.clone()).await;
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
