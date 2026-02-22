use crate::models::{ApiResponse, PerformanceMetrics, HistoryQueryParams};
use crate::common::state::AppState;
use axum::{
    extract::{Query, State, ws::{WebSocketUpgrade, WebSocket, Message}},
    Json,
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::time::Duration;

// GET /performance/metrics
pub async fn get_metrics(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<PerformanceMetrics>> {
    let history = state.performance_history.read().await;
    let latest = history.last().cloned().unwrap_or_default();
    Json(ApiResponse::success(latest))
}

// GET /performance/detailed
pub async fn get_detailed_metrics(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<PerformanceMetrics>> {
    let history = state.performance_history.read().await;
    let latest = history.last().cloned().unwrap_or_default();
    Json(ApiResponse::success(latest))
}

// GET /performance/history
pub async fn get_history(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HistoryQueryParams>,
) -> Json<ApiResponse<Vec<PerformanceMetrics>>> {
    let history = state.performance_history.read().await;
    // Filter by time would go here
    let limit = params.limit.unwrap_or(100);
    let result = history.iter().take(limit).cloned().collect();
    Json(ApiResponse::success(result))
}

// GET /performance/summary
pub async fn get_summary(
    State(state): State<Arc<AppState>>,
    Query(_params): Query<HistoryQueryParams>,
) -> Json<ApiResponse<serde_json::Value>> {
    let history = state.performance_history.read().await;
    let count = history.len() as f64;
    
    let avg_tps = if count > 0.0 {
        history.iter().map(|m| m.throughput).sum::<f64>() / count
    } else {
        0.0
    };
    
    let summary = serde_json::json!({
        "average_tps": avg_tps,
        "max_tps": history.iter().map(|m| m.throughput).fold(0.0, f64::max),
        "total_samples": count
    });
    
    Json(ApiResponse::success(summary))
}

// POST /performance/clear
pub async fn clear_data(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<String>> {
    let mut history = state.performance_history.write().await;
    history.clear();
    Json(ApiResponse::success("Performance data cleared".to_string()))
}

// POST /performance/export
pub async fn export_performance_data(
    State(_state): State<Arc<AppState>>,
) -> Json<ApiResponse<String>> {
    // Mock export
    Json(ApiResponse::success("Export started".to_string()))
}

// GET /performance/comparison
pub async fn get_performance_comparison(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<serde_json::Value>> {
    let history = state.performance_history.read().await;
    let latest = history.last().cloned().unwrap_or_default();

    let avg_throughput = if history.is_empty() {
        0.0
    } else {
        history.iter().map(|m| m.throughput).sum::<f64>() / history.len() as f64
    };

    let avg_latency = if history.is_empty() {
        0.0
    } else {
        history.iter().map(|m| m.latency).sum::<f64>() / history.len() as f64
    };

    let improvement = serde_json::json!({
        "current": {
            "throughput": latest.throughput,
            "latency": latest.latency,
        },
        "baseline": {
            "throughput": avg_throughput,
            "latency": avg_latency,
        },
        "improvement": {
            "throughput_pct": if avg_throughput > 0.0 {
                (latest.throughput - avg_throughput) * 100.0 / avg_throughput
            } else {
                0.0
            },
            "latency_pct": if avg_latency > 0.0 {
                (latest.latency - avg_latency) * 100.0 / avg_latency
            } else {
                0.0
            },
        }
    });

    Json(ApiResponse::success(improvement))
}

// WebSocket Handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    
    loop {
        interval.tick().await;
        
        let latest = {
            let history = state.performance_history.read().await;
            history.last().cloned()
        };

        if let Some(metrics) = latest {
            if let Ok(msg) = serde_json::to_string(&metrics) {
                if socket.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        }
    }
}
