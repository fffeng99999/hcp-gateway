use crate::models::{ApiResponse, PerformanceMetrics, HistoryQueryParams};
use crate::common::state::AppState;
use axum::{
    extract::{Query, State, ws::{WebSocketUpgrade, WebSocket, Message}},
    Json,
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::time::Duration;
use rand::Rng;

// GET /performance/metrics
pub async fn get_metrics(
    State(_state): State<Arc<AppState>>,
) -> Json<ApiResponse<PerformanceMetrics>> {
    // In a real app, this would fetch latest metrics from a service or channel
    let metrics = PerformanceMetrics {
        throughput: 5000.0,
        latency: 150.0,
        latency_p99: 200.0,
        latency_p999: 250.0,
        finality_time: 300.0,
        network_bandwidth: 100.0,
        cpu_usage: 45.0,
        memory_usage: 512.0,
    };
    Json(ApiResponse::success(metrics))
}

// GET /performance/detailed
pub async fn get_detailed_metrics() -> Json<ApiResponse<serde_json::Value>> {
    let detailed = serde_json::json!({
        "metrics": {
            "throughput": 5000.0,
            "latency": 150.0,
        },
        "resources": {
            "cpu_cores": [45.0, 40.0, 50.0, 42.0],
            "memory_heap": 512.0,
            "memory_stack": 12.0,
            "disk_io_read": 1024.0,
            "disk_io_write": 2048.0
        },
        "network": {
            "peers": 50,
            "bandwidth_in": 50.0,
            "bandwidth_out": 50.0
        }
    });
    Json(ApiResponse::success(detailed))
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
    State(_state): State<Arc<AppState>>,
) -> Json<ApiResponse<serde_json::Value>> {
    // This could call into analysis or return specific perf comparison
    // For now, we return a mock comparison
    let comparison = serde_json::json!({
        "current": {
            "throughput": 5000.0,
            "latency": 150.0
        },
        "baseline": {
            "throughput": 4500.0,
            "latency": 160.0
        },
        "improvement": {
            "throughput_pct": 11.1,
            "latency_pct": -6.25
        }
    });
    Json(ApiResponse::success(comparison))
}

// WebSocket Handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, _state: Arc<AppState>) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    
    loop {
        interval.tick().await;
        
        // Generate mock metrics
        let msg = {
            let mut rng = rand::thread_rng();
            let metrics = PerformanceMetrics {
                throughput: rng.gen_range(4000.0..6000.0),
                latency: rng.gen_range(100.0..200.0),
                latency_p99: rng.gen_range(150.0..250.0),
                latency_p999: rng.gen_range(200.0..300.0),
                finality_time: rng.gen_range(250.0..350.0),
                network_bandwidth: rng.gen_range(50.0..150.0),
                cpu_usage: rng.gen_range(30.0..60.0),
                memory_usage: rng.gen_range(400.0..800.0),
            };
            serde_json::to_string(&metrics)
        };
        
        if let Ok(msg) = msg {
            if socket.send(Message::Text(msg)).await.is_err() {
                // Client disconnected
                break;
            }
        }
    }
}
