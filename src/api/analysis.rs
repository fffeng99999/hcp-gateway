use axum::{
    extract::State,
    Json,
};
use crate::{
    error::ApiResult,
    models::*,
    state::AppState,
};

pub async fn get_report(
    State(state): State<AppState>,
) -> ApiResult<Json<AnalysisReport>> {
    tracing::info!("Generating analysis report");
    
    let benchmarks = state.benchmarks.read().await;
    let total_benchmarks = benchmarks.len();
    
    // Calculate summary statistics
    let avg_throughput: f64 = if !benchmarks.is_empty() {
        benchmarks.values().map(|b| b.metrics.throughput).sum::<f64>() / total_benchmarks as f64
    } else {
        0.0
    };
    
    let avg_latency: f64 = if !benchmarks.is_empty() {
        benchmarks.values().map(|b| b.metrics.latency).sum::<f64>() / total_benchmarks as f64
    } else {
        0.0
    };
    
    let report = AnalysisReport {
        report_id: uuid::Uuid::new_v4().to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        title: "HCP Performance Analysis Report".to_string(),
        summary: format!(
            "Analyzed {} benchmarks with average throughput of {:.2} TPS and average latency of {:.2}ms",
            total_benchmarks, avg_throughput, avg_latency
        ),
        recommendations: vec![
            "Consider using tPBFT for scenarios requiring high throughput".to_string(),
            "Leios protocol shows promise for low-latency requirements".to_string(),
            "Network optimization can significantly improve consensus performance".to_string(),
        ],
    };
    
    Ok(Json(report))
}

pub async fn get_trends(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<TrendData>>> {
    tracing::info!("Getting performance trends");
    
    let benchmarks = state.benchmarks.read().await;
    let trends: Vec<TrendData> = benchmarks
        .values()
        .map(|b| TrendData {
            timestamp: b.start_time.clone(),
            algorithm: b.algorithm_id.clone(),
            throughput: b.metrics.throughput,
            latency: b.metrics.latency,
        })
        .collect();
    
    Ok(Json(trends))
}
