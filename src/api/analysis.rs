use crate::models::{ApiResponse, AnalysisReport, GenerateReportRequest, TrendData, ExportParams};
use crate::common::state::AppState;
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    Json,
    http::header,
};
use std::sync::Arc;

// GET /analysis/summary
pub async fn get_summary(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<serde_json::Value>> {
    let benchmarks = state.benchmarks.read().await;
    
    // Simple mock analysis
    let total_benchmarks = benchmarks.len();
    let best_performance = benchmarks.iter()
        .map(|b| b.metrics.throughput)
        .fold(0.0, f64::max);
        
    let summary = serde_json::json!({
        "total_benchmarks_run": total_benchmarks,
        "highest_throughput_achieved": best_performance,
        "most_used_algorithm": "tPBFT",
        "system_stability_score": 95.5
    });
    
    Json(ApiResponse::success(summary))
}

// GET /analysis/report (Get latest report or list)
pub async fn get_report(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<AnalysisReport>>> {
    let reports = state.analysis_reports.read().await;
    Json(ApiResponse::success(reports.clone()))
}

// GET /analysis/prediction
pub async fn get_prediction(
    State(_state): State<Arc<AppState>>,
) -> Json<ApiResponse<serde_json::Value>> {
    // Mock prediction
    let prediction = serde_json::json!({
        "predicted_throughput_next_hour": 5500.0,
        "predicted_latency_trend": "stable",
        "recommended_actions": ["maintain_current_config"]
    });
    Json(ApiResponse::success(prediction))
}

// GET /analysis/comparison
pub async fn get_comparison(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<serde_json::Value>> {
    let benchmarks = state.benchmarks.read().await;
    
    // Group by algorithm
    let mut comparison = serde_json::Map::new();
    
    // Mock comparison data if no benchmarks
    if benchmarks.is_empty() {
        comparison.insert("tPBFT".to_string(), serde_json::json!({"avg_tps": 5000.0, "avg_latency": 150.0}));
        comparison.insert("HotStuff".to_string(), serde_json::json!({"avg_tps": 4500.0, "avg_latency": 180.0}));
    } else {
        // Real aggregation logic would go here
        comparison.insert("tPBFT".to_string(), serde_json::json!({"avg_tps": 5200.0, "avg_latency": 140.0}));
    }
    
    Json(ApiResponse::success(serde_json::Value::Object(comparison)))
}

// GET /analysis/limits/:algo
pub async fn get_algo_limits(
    Path(algo): Path<String>,
    State(_state): State<Arc<AppState>>,
) -> Json<ApiResponse<serde_json::Value>> {
    // Return theoretical vs actual limits
    let limits = serde_json::json!({
        "algorithm": algo,
        "theoretical_tps": 10000.0,
        "achieved_tps": 5500.0,
        "theoretical_latency_min": 50.0,
        "achieved_latency_min": 120.0,
        "bottleneck": "Network Bandwidth"
    });
    
    Json(ApiResponse::success(limits))
}

// GET /analysis/trends
pub async fn get_trends(
    State(_state): State<Arc<AppState>>,
    Query(_params): Query<serde_json::Value>,
) -> Json<ApiResponse<Vec<TrendData>>> {
    // Mock trend data
    let now = chrono::Utc::now();
    let trends = (0..10).map(|i| {
        TrendData {
            timestamp: (now - chrono::Duration::minutes(i * 10)).to_rfc3339(),
            metric: "throughput".to_string(),
            value: 4000.0 + (i as f64) * 100.0,
        }
    }).collect();
    
    Json(ApiResponse::success(trends))
}

// POST /analysis/report/generate
pub async fn generate_report(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<GenerateReportRequest>,
) -> Response {
    let content = format!("REPORT: {}\n\n{}", req.title, req.content);
    let filename = format!("report_{}.txt", chrono::Utc::now().timestamp());
    
    (
        [
            (header::CONTENT_TYPE, "text/plain"),
            (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{}\"", filename)),
        ],
        content,
    ).into_response()
}

// POST /analysis/export
pub async fn export_analysis(
    State(_state): State<Arc<AppState>>,
    Json(params): Json<ExportParams>,
) -> Response {
    let content = "analysis_id,metric,value\n1,tps,5000\n2,latency,150";
    let filename = format!("analysis_export.{}", params.format);
    
    (
        [
            (header::CONTENT_TYPE, "text/csv"),
            (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{}\"", filename)),
        ],
        content,
    ).into_response()
}
