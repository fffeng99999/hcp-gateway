use axum::{
    extract::State,
    Json,
};
use crate::{
    error::ApiResult,
    models::*,
    state::AppState,
};

pub async fn get_metrics(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<PerformanceMetrics>>> {
    tracing::info!("Getting current performance metrics");
    
    let benchmarks = state.benchmarks.read().await;
    let metrics: Vec<PerformanceMetrics> = benchmarks
        .values()
        .map(|b| b.metrics.clone())
        .collect();
    
    Ok(Json(metrics))
}

pub async fn get_history(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<BenchmarkResult>>> {
    tracing::info!("Getting performance history");
    
    let benchmarks = state.benchmarks.read().await;
    let mut history: Vec<BenchmarkResult> = benchmarks.values().cloned().collect();
    history.sort_by(|a, b| a.start_time.cmp(&b.start_time));
    
    Ok(Json(history))
}

pub async fn get_comparison(
    State(state): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    tracing::info!("Getting performance comparison");
    
    let benchmarks = state.benchmarks.read().await;
    
    let mut comparison = std::collections::HashMap::new();
    for benchmark in benchmarks.values() {
        comparison
            .entry(benchmark.algorithm_id.clone())
            .or_insert_with(Vec::new)
            .push(benchmark.metrics.clone());
    }
    
    let result = serde_json::json!({
        "algorithms": comparison
            .into_iter()
            .map(|(algo, metrics)| {
                let avg_throughput = metrics.iter().map(|m| m.throughput).sum::<f64>() / metrics.len() as f64;
                let avg_latency = metrics.iter().map(|m| m.latency).sum::<f64>() / metrics.len() as f64;
                
                (algo, serde_json::json!({
                    "count": metrics.len(),
                    "avg_throughput": avg_throughput,
                    "avg_latency": avg_latency,
                }))
            })
            .collect::<std::collections::HashMap<_, _>>()
    });
    
    Ok(Json(result))
}
