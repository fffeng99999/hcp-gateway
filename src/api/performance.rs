use crate::error::ApiResult;
use crate::state::AppState;
use axum::{extract::State, Json};
use serde_json::{json, Value};
use std::sync::Arc;

// GET /performance/metrics
pub async fn get_metrics(State(state): State<Arc<AppState>>) -> ApiResult<Json<Value>> {
    let benchmarks = state.benchmarks.read().await;

    if let Some(latest) = benchmarks.last() {
        if let Some(metrics) = latest.get("metrics") {
            return Ok(Json(json!({
                "current_metrics": metrics,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            })));
        }
    }

    Ok(Json(json!({
        "current_metrics": {
            "throughput": 0.0,
            "latency": 0.0,
        },
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })))
}

// GET /performance/history
pub async fn get_performance_history(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<Value>>> {
    let benchmarks = state.benchmarks.read().await;

    let history: Vec<Value> = benchmarks
        .iter()
        .map(|b| {
            json!({
                "benchmark_id": b.get("benchmark_id"),
                "algorithm_id": b.get("algorithm_id"),
                "metrics": b.get("metrics"),
                "timestamp": b.get("start_time"),
            })
        })
        .collect();

    Ok(Json(history))
}

// GET /performance/comparison
pub async fn get_performance_comparison(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Value>> {
    let benchmarks = state.benchmarks.read().await;

    let mut comparison = std::collections::HashMap::new();

    for benchmark in benchmarks.iter() {
        if let Some(algo_id) = benchmark.get("algorithm_id").and_then(|v| v.as_str()) {
            if let Some(metrics) = benchmark.get("metrics") {
                comparison
                    .entry(algo_id.to_string())
                    .or_insert_with(|| metrics.clone());
            }
        }
    }

    Ok(Json(json!(comparison)))
}
