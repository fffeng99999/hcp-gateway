use axum::{
    extract::{Path, State},
    Json,
};
use crate::{
    error::{ApiError, ApiResult},
    models::*,
    state::AppState,
};

pub async fn list_nodes(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<Node>>> {
    tracing::info!("Listing all nodes");
    
    let nodes = state.nodes.read().await;
    let node_list: Vec<Node> = nodes.values().cloned().collect();
    
    Ok(Json(node_list))
}

pub async fn get_node(
    State(state): State<AppState>,
    Path(node_id): Path<String>,
) -> ApiResult<Json<Node>> {
    tracing::info!("Getting node: {}", node_id);
    
    let nodes = state.nodes.read().await;
    nodes
        .get(&node_id)
        .cloned()
        .map(Json)
        .ok_or_else(|| ApiError::NotFound(format!("Node {} not found", node_id)))
}

pub async fn get_node_stats(
    State(state): State<AppState>,
) -> ApiResult<Json<NodeStats>> {
    tracing::info!("Getting node statistics");
    
    let nodes = state.nodes.read().await;
    let total_count = nodes.len() as u32;
    let online_count = nodes
        .values()
        .filter(|n| n.status == "online")
        .count() as u32;
    let offline_count = total_count - online_count;
    
    let stats = NodeStats {
        node_id: "cluster".to_string(),
        online_count,
        offline_count,
        total_count,
        average_latency: 50.0,
    };
    
    Ok(Json(stats))
}
