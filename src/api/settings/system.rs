// 系统信息相关接口
use crate::common::state::AppState;
use crate::models::ApiResponse;
use axum::{extract::State, response::IntoResponse};
use serde::Serialize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use super::common::respond_with_version;

#[derive(Serialize)]
pub struct SystemInfo {
    pub system_name: String,
    pub version: String,
    pub debug_mode: bool,
    pub config_version: u64,
    pub consensus_healthy: bool,
    pub server_healthy: bool,
}

pub async fn get_system_info(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let general = state.general_settings.read().await.clone();
    let info = SystemInfo {
        system_name: general.system_name,
        version: general.version,
        debug_mode: general.debug_mode,
        config_version: state.config_version.load(Ordering::SeqCst),
        consensus_healthy: state.consensus_healthy.load(Ordering::SeqCst),
        server_healthy: state.server_healthy.load(Ordering::SeqCst),
    };
    respond_with_version(&state, ApiResponse::success(info), false)
}
