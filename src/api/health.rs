use crate::models::ApiResponse;
use crate::common::state::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::Serialize;
use std::sync::Arc;
use std::sync::atomic::Ordering;

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
}

pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let is_healthy = state.consensus_healthy.load(Ordering::SeqCst);
    
    let status_str = if is_healthy { "healthy" } else { "unhealthy" };
    let code = if is_healthy { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    
    let status = HealthStatus {
        status: status_str.to_string(),
        version: "1.0.0".to_string(),
    };
    (code, Json(ApiResponse::success(status)))
}
