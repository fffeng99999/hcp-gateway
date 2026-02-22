use crate::common::state::AppState;
use crate::models::{ApiResponse, FaultInjectionConfig, Node, NodeRegistrationRequest, NodeStats};
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;

// GET /nodes
pub async fn list_nodes(State(state): State<Arc<AppState>>) -> Json<ApiResponse<Vec<Node>>> {
    // Try to fetch from server first
    if let Some(client) = &state.server_client {
        let mut client = client.clone();
        let request = tonic::Request::new(crate::services::server_client::node::ListNodesRequest {
            role: "".to_string(),
            status: "".to_string(),
            region: "".to_string(),
            pagination: None,
        });

        match client.node_client.list_nodes(request).await {
            Ok(response) => {
                let resp = response.into_inner();
                let nodes: Vec<Node> = resp
                    .nodes
                    .into_iter()
                    .map(|n| Node {
                        id: n.id,
                        name: n.name,
                        address: n.address,
                        status: n.status,
                        role: n.role,
                        last_heartbeat: n.last_heartbeat,
                        health_score: n.trust_score,
                    })
                    .collect();
                return Json(ApiResponse::success(nodes));
            }
            Err(e) => {
                tracing::warn!("Failed to fetch nodes from server: {}", e);
            }
        }
    }

    let nodes = state.nodes.read().await;
    Json(ApiResponse::success(nodes.clone()))
}

// GET /nodes/stats
pub async fn get_node_stats(State(state): State<Arc<AppState>>) -> Json<ApiResponse<NodeStats>> {
    let nodes = state.nodes.read().await;
    let total_count = nodes.len() as u32;
    let online_count = nodes.iter().filter(|n| n.status == "online").count() as u32;
    let offline_count = total_count - online_count;

    // Mock latency calculation
    let average_latency = if total_count > 0 {
        nodes.iter().map(|_| 15.5).sum::<f64>() / total_count as f64
    } else {
        0.0
    };

    let stats = NodeStats {
        online_count,
        offline_count,
        total_count,
        average_latency,
    };

    Json(ApiResponse::success(stats))
}

// GET /nodes/health/all
pub async fn get_all_nodes_health(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<serde_json::Value>>> {
    let nodes = state.nodes.read().await;
    let health_data = nodes
        .iter()
        .map(|n| {
            serde_json::json!({
                "id": n.id,
                "status": n.status,
                "health_score": n.health_score,
                "last_heartbeat": n.last_heartbeat
            })
        })
        .collect();
    Json(ApiResponse::success(health_data))
}

// GET /nodes/:id
pub async fn get_node(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Node>> {
    let nodes = state.nodes.read().await;
    if let Some(node) = nodes.iter().find(|n| n.id == id) {
        Json(ApiResponse::success(node.clone()))
    } else {
        Json(ApiResponse::error(404, "Node not found"))
    }
}

// GET /nodes/:id/health
pub async fn get_node_health(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<serde_json::Value>> {
    let nodes = state.nodes.read().await;
    if let Some(node) = nodes.iter().find(|n| n.id == id) {
        let health = serde_json::json!({
            "status": node.status,
            "health_score": node.health_score,
            "details": {
                "cpu": 45.0,
                "memory": 60.0,
                "disk": 30.0
            }
        });
        Json(ApiResponse::success(health))
    } else {
        Json(ApiResponse::error(404, "Node not found"))
    }
}

// POST /nodes/register
pub async fn register_node(
    State(state): State<Arc<AppState>>,
    Json(req): Json<NodeRegistrationRequest>,
) -> Json<ApiResponse<Node>> {
    let mut nodes = state.nodes.write().await;

    let new_node = Node {
        id: uuid::Uuid::new_v4().to_string(),
        name: req.name,
        address: req.address,
        status: "online".to_string(),
        role: req.role,
        last_heartbeat: chrono::Utc::now().to_rfc3339(),
        health_score: 100.0,
    };

    nodes.push(new_node.clone());
    Json(ApiResponse::success(new_node))
}

// DELETE /nodes/:id
pub async fn remove_node(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<String>> {
    let mut nodes = state.nodes.write().await;
    if let Some(pos) = nodes.iter().position(|n| n.id == id) {
        nodes.remove(pos);
        Json(ApiResponse::success("Node removed".to_string()))
    } else {
        Json(ApiResponse::error(404, "Node not found"))
    }
}

// POST /nodes/:id/inject-fault
pub async fn inject_fault(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(config): Json<FaultInjectionConfig>,
) -> Json<ApiResponse<String>> {
    let mut nodes = state.nodes.write().await;
    if let Some(node) = nodes.iter_mut().find(|n| n.id == id) {
        // Simulate fault injection logic
        if config.fault_type == "crash" {
            node.status = "offline".to_string();
            node.health_score = 0.0;
        } else {
            node.health_score -= 20.0;
        }
        Json(ApiResponse::success(format!(
            "Fault injected: {}",
            config.fault_type
        )))
    } else {
        Json(ApiResponse::error(404, "Node not found"))
    }
}

// POST /nodes/:id/recover
pub async fn recover_node(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<String>> {
    let mut nodes = state.nodes.write().await;
    if let Some(node) = nodes.iter_mut().find(|n| n.id == id) {
        node.status = "online".to_string();
        node.health_score = 100.0;
        Json(ApiResponse::success("Node recovered".to_string()))
    } else {
        Json(ApiResponse::error(404, "Node not found"))
    }
}
