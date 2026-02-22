use crate::common::state::AppState;
use crate::models::{AnalysisReport, ApiResponse, ExportParams, GenerateReportRequest, TrendData};
use axum::{
    extract::{Path, Query, State},
    http::header,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;

// GET /analysis/summary 获取基于基准测试结果的概要分析
pub async fn get_summary(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<serde_json::Value>> {
    let benchmarks = state.benchmarks.read().await;

    // 简单的示例性分析逻辑
    let total_benchmarks = benchmarks.len();
    let best_performance = benchmarks
        .iter()
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

// GET /analysis/report 获取分析报告列表或最新报告
pub async fn get_report(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<AnalysisReport>>> {
    let reports = state.analysis_reports.read().await;
    Json(ApiResponse::success(reports.clone()))
}

// GET /analysis/prediction 获取性能与容量的预测结果（示例数据）
pub async fn get_prediction(
    State(_state): State<Arc<AppState>>,
) -> Json<ApiResponse<serde_json::Value>> {
    // 目前使用固定示例数据，后续可接入真实模型
    let prediction = serde_json::json!({
        "predicted_throughput_next_hour": 5500.0,
        "predicted_latency_trend": "stable",
        "recommended_actions": ["maintain_current_config"]
    });
    Json(ApiResponse::success(prediction))
}

// GET /analysis/comparison 获取不同算法之间的性能对比
pub async fn get_comparison(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<serde_json::Value>> {
    let benchmarks = state.benchmarks.read().await;

    // 按算法维度聚合对比数据
    let mut comparison = serde_json::Map::new();

    // 若没有任何基准测试结果，则返回内置的示例对比数据
    if benchmarks.is_empty() {
        comparison.insert(
            "tPBFT".to_string(),
            serde_json::json!({"avg_tps": 5000.0, "avg_latency": 150.0}),
        );
        comparison.insert(
            "HotStuff".to_string(),
            serde_json::json!({"avg_tps": 4500.0, "avg_latency": 180.0}),
        );
    } else {
        // 真实场景下应根据 benchmarks 中的数据进行聚合计算
        comparison.insert(
            "tPBFT".to_string(),
            serde_json::json!({"avg_tps": 5200.0, "avg_latency": 140.0}),
        );
    }

    Json(ApiResponse::success(serde_json::Value::Object(comparison)))
}

// GET /analysis/limits/:algo 获取指定算法的理论与实测性能上限
pub async fn get_algo_limits(
    Path(algo): Path<String>,
    State(_state): State<Arc<AppState>>,
) -> Json<ApiResponse<serde_json::Value>> {
    // 返回理论极限值与当前实测值，帮助分析性能瓶颈
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

// GET /analysis/trends 获取性能指标随时间变化的趋势
pub async fn get_trends(
    State(_state): State<Arc<AppState>>,
    Query(_params): Query<serde_json::Value>,
) -> Json<ApiResponse<Vec<TrendData>>> {
    // 当前返回固定的趋势示例数据
    let now = chrono::Utc::now();
    let trends = (0..10)
        .map(|i| TrendData {
            timestamp: (now - chrono::Duration::minutes(i * 10)).to_rfc3339(),
            metric: "throughput".to_string(),
            value: 4000.0 + (i as f64) * 100.0,
        })
        .collect();

    Json(ApiResponse::success(trends))
}

// POST /analysis/report/generate 生成分析报告并以附件形式下载
pub async fn generate_report(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<GenerateReportRequest>,
) -> Response {
    let content = format!("REPORT: {}\n\n{}", req.title, req.content);
    let filename = format!("report_{}.txt", chrono::Utc::now().timestamp());

    (
        [
            (header::CONTENT_TYPE, "text/plain"),
            (
                header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{}\"", filename),
            ),
        ],
        content,
    )
        .into_response()
}

// POST /analysis/export 导出分析数据（CSV 等格式）
pub async fn export_analysis(
    State(_state): State<Arc<AppState>>,
    Json(params): Json<ExportParams>,
) -> Response {
    let content = "analysis_id,metric,value\n1,tps,5000\n2,latency,150";
    let filename = format!("analysis_export.{}", params.format);

    (
        [
            (header::CONTENT_TYPE, "text/csv"),
            (
                header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{}\"", filename),
            ),
        ],
        content,
    )
        .into_response()
}
