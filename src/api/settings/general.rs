// 通用设置相关接口
use super::common::{
    merge_settings, read_cache, require_version, respond_with_version, write_cache,
    CONFIG_CACHE_TTL,
};
use crate::common::state::AppState;
use crate::models::ApiResponse;
use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};
use serde_json::Value;
use std::sync::Arc;

// 获取通用设置
pub async fn get_general(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if let Some(cached) = read_cache(&state.general_cache, CONFIG_CACHE_TTL).await {
        return respond_with_version(&state, ApiResponse::success(cached), false);
    }
    let settings = state.general_settings.read().await.clone();
    write_cache(&state.general_cache, settings.clone()).await;
    respond_with_version(&state, ApiResponse::success(settings), false)
}

// 更新通用设置
pub async fn update_general(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(patch): Json<Value>,
) -> impl IntoResponse {
    if let Err(resp) = require_version(&headers, &state) {
        return respond_with_version(&state, resp, false);
    }
    let mut settings = state.general_settings.write().await;
    match merge_settings(&settings.clone(), patch) {
        Ok(merged) => {
            *settings = merged;
            write_cache(&state.general_cache, settings.clone()).await;
            respond_with_version(
                &state,
                ApiResponse::success("General settings updated".to_string()),
                true,
            )
        }
        Err(err) => respond_with_version(&state, ApiResponse::error(400, err), false),
    }
}
