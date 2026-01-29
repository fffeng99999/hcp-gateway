use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use axum::{extract::Path, extract::State, Json};
use serde_json::{json, Value};
use std::sync::Arc;

// GET /consensus/algorithms
pub async fn get_algorithms(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<Value>>> {
    let algorithms = state.algorithms.read().await;
    Ok(Json(algorithms.clone()))
}

// GET /consensus/config
pub async fn get_config(State(state): State<Arc<AppState>>) -> ApiResult<Json<Value>> {
    let config = state.consensus_config.read().await;
    Ok(Json(config.clone()))
}

// POST /consensus/select
pub async fn select_algorithm(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> ApiResult<Json<Value>> {
    let algorithm_id = payload
        .get("algorithm_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::InvalidInput("Missing algorithm_id".to_string()))?
        .to_string();

    let mut config = state.consensus_config.write().await;
    *config = json!({
        "current_algorithm": algorithm_id,
        "is_active": true,
        "last_updated": chrono::Utc::now().to_rfc3339(),
    });

    Ok(Json(config.clone()))
}

// PUT /consensus/parameters
pub async fn update_parameters(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> ApiResult<Json<Value>> {
    let mut config = state.consensus_config.write().await;
    if let Some(obj) = config.as_object_mut() {
        if let Some(params) = payload.get("parameters") {
            obj["parameters"] = params.clone();
        }
    }
    Ok(Json(config.clone()))
}

// POST /consensus/benchmark/start
pub async fn start_benchmark(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> ApiResult<Json<Value>> {
    let benchmark_id = uuid::Uuid::new_v4().to_string();
    let mut benchmarks = state.benchmarks.write().await;

    let benchmark = json!({
        "benchmark_id": benchmark_id,
        "algorithm_id": payload.get("algorithm_id").unwrap_or(&json!("tPBFT")),
        "parameters": payload.get("parameters").cloned().unwrap_or(json!({})),
        "start_time": chrono::Utc::now().to_rfc3339(),
        "status": "running",
    });

    benchmarks.push(benchmark.clone());

    Ok(Json(json!({
        "benchmark_id": benchmark_id,
        "status": "started",
    })))
}

// GET /consensus/benchmark/:id
pub async fn get_benchmark(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let benchmarks = state.benchmarks.read().await;
    benchmarks
        .iter()
        .find(|b| b.get("benchmark_id").and_then(|v| v.as_str()) == Some(&id))
        .cloned()
        .map(Json)
        .ok_or_else(|| ApiError::NotFound(format!("Benchmark {} not found", id)))
}

// POST /consensus/benchmark/:id/stop
pub async fn stop_benchmark(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let mut benchmarks = state.benchmarks.write().await;
    if let Some(benchmark) = benchmarks.iter_mut().find(|b| {
        b.get("benchmark_id").and_then(|v| v.as_str()) == Some(&id)
    }) {
        if let Some(obj) = benchmark.as_object_mut() {
            obj["status"] = json!("stopped");
            obj["end_time"] = json!(chrono::Utc::now().to_rfc3339());
        }
        Ok(Json(benchmark.clone()))
    } else {
        Err(ApiError::NotFound(format!("Benchmark {} not found", id)))
    }
}

// GET /consensus/benchmark/history
pub async fn get_benchmark_history(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<Value>>> {
    let benchmarks = state.benchmarks.read().await;
    Ok(Json(benchmarks.clone()))
}
