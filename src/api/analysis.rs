use crate::error::ApiResult;
use crate::state::AppState;
use axum::{extract::State, Json};
use serde_json::{json, Value};
use std::sync::Arc;

// GET /analysis/report
pub async fn get_report(State(state): State<Arc<AppState>>) -> ApiResult<Json<Value>> {
    let benchmarks = state.benchmarks.read().await;
    let transactions = state.transactions.read().await;
    let nodes = state.nodes.read().await;

    let total_benchmarks = benchmarks.len();
    let completed_benchmarks = benchmarks
        .iter()
        .filter(|b| b.get("status").and_then(|v| v.as_str()) == Some("completed"))
        .count();

    let total_transactions = transactions.len();
    let successful_transactions = transactions
        .iter()
        .filter(|t| t.get("status").and_then(|v| v.as_str()) == Some("success"))
        .count();

    let total_nodes = nodes.len();
    let online_nodes = nodes
        .iter()
        .filter(|n| n.get("status").and_then(|v| v.as_str()) == Some("online"))
        .count();

    Ok(Json(json!({
        "title": "System Performance Report",
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "summary": {
            "benchmarks": {
                "total": total_benchmarks,
                "completed": completed_benchmarks,
                "completion_rate": if total_benchmarks > 0 {
                    (completed_benchmarks as f64) / (total_benchmarks as f64)
                } else {
                    0.0
                }
            },
            "transactions": {
                "total": total_transactions,
                "successful": successful_transactions,
                "success_rate": if total_transactions > 0 {
                    (successful_transactions as f64) / (total_transactions as f64)
                } else {
                    0.0
                }
            },
            "network": {
                "total_nodes": total_nodes,
                "online_nodes": online_nodes,
                "availability": if total_nodes > 0 {
                    (online_nodes as f64) / (total_nodes as f64)
                } else {
                    0.0
                }
            }
        }
    })))
}

// GET /analysis/trends
pub async fn get_trends(State(state): State<Arc<AppState>>) -> ApiResult<Json<Value>> {
    let benchmarks = state.benchmarks.read().await;

    let mut trends = json!({
        "throughput_trend": [],
        "latency_trend": [],
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    if let Some(trends_obj) = trends.as_object_mut() {
        let mut throughput = vec![];
        let mut latency = vec![];

        for benchmark in benchmarks.iter() {
            if let Some(metrics) = benchmark.get("metrics") {
                if let Some(tp) = metrics.get("throughput").and_then(|v| v.as_f64()) {
                    throughput.push(json!(tp));
                }
                if let Some(lat) = metrics.get("latency").and_then(|v| v.as_f64()) {
                    latency.push(json!(lat));
                }
            }
        }

        trends_obj["throughput_trend"] = json!(throughput);
        trends_obj["latency_trend"] = json!(latency);
    }

    Ok(Json(trends))
}
