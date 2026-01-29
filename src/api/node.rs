use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use axum::{extract::Path, extract::State, Json};
use serde_json::{json, Value};
use std::sync::Arc;

// GET /node/list
pub async fn list_nodes(State(state): State<Arc<AppState>>) -> ApiResult<Json<Vec<Value>>> {
    let nodes = state.nodes.read().await;
    Ok(Json(nodes.clone()))
}

// GET /node/:id
pub async fn get_node(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let nodes = state.nodes.read().await;
    nodes
        .iter()
        .find(|n| n.get("id").and_then(|v| v.as_str()) == Some(&id))
        .cloned()
        .map(Json)
        .ok_or_else(|| ApiError::NotFound(format!("Node {} not found", id)))
}

// GET /node/stats
pub async fn get_node_stats(State(state): State<Arc<AppState>>) -> ApiResult<Json<Value>> {
    let nodes = state.nodes.read().await;

    let total = nodes.len();
    let online = nodes
        .iter()
        .filter(|n| n.get("status").and_then(|v| v.as_str()) == Some("online"))
        .count();
    let offline = total - online;

    let leaders = nodes
        .iter()
        .filter(|n| n.get("role").and_then(|v| v.as_str()) == Some("leader"))
        .count();

    let validators = nodes
        .iter()
        .filter(|n| n.get("role").and_then(|v| v.as_str()) == Some("validator"))
        .count();

    Ok(Json(json!({
        "total_nodes": total,
        "online_nodes": online,
        "offline_nodes": offline,
        "leaders": leaders,
        "validators": validators,
        "availability": if total > 0 {
            (online as f64) / (total as f64)
        } else {
            0.0
        },
    })))
}
