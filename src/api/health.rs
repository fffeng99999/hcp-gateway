use crate::common::state::AppState;
use crate::models::ApiResponse;
use axum::{
    extract::State,
    http::StatusCode,
    response::sse::{Event, Sse},
    response::IntoResponse,
    Json,
};
use futures::stream::Stream;
use serde::Serialize;
use std::convert::Infallible;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::time::Duration;
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::StreamExt;

#[derive(Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
}

// 简单健康检查接口，返回网关当前状态与版本号
pub async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let is_healthy = state.consensus_healthy.load(Ordering::SeqCst);

    let status_str = if is_healthy { "healthy" } else { "unhealthy" };
    let code = if is_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let status = HealthStatus {
        status: status_str.to_string(),
        version: "1.0.0".to_string(),
    };
    (code, Json(ApiResponse::success(status)))
}

// 系统信息 SSE 流，周期性推送共识、服务健康状态与配置版本号
pub async fn system_stream(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let interval = IntervalStream::new(tokio::time::interval(Duration::from_secs(1)));

    let stream = interval.map(move |_| {
        let consensus_healthy = state.consensus_healthy.load(Ordering::SeqCst);
        let server_healthy = state.server_healthy.load(Ordering::SeqCst);
        let config_version = state.config_version.load(Ordering::SeqCst);

        let payload = serde_json::json!({
            "consensus_healthy": consensus_healthy,
            "server_healthy": server_healthy,
            "config_version": config_version
        });

        Ok(Event::default().data(payload.to_string()))
    });

    Sse::new(stream)
}
