use crate::models::{
    ApiResponse, ConsensusAlgorithm, ConsensusConfig, SelectAlgorithmRequest, 
    UpdateParametersRequest
};
use crate::state::AppState;
use axum::{
    extract::State,
    Json,
};
use std::sync::Arc;

// GET /consensus/algorithms
pub async fn get_algorithms(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<ConsensusAlgorithm>>> {
    let algorithms = state.algorithms.read().await;
    Json(ApiResponse::success(algorithms.clone()))
}

// GET /consensus/config
pub async fn get_config(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<ConsensusConfig>> {
    let config = state.consensus_config.read().await;
    Json(ApiResponse::success(config.clone()))
}

// POST /consensus/select
pub async fn select_algorithm(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SelectAlgorithmRequest>,
) -> Json<ApiResponse<String>> {
    let mut config = state.consensus_config.write().await;
    config.current_algorithm = req.algorithm_id;
    config.parameters = req.parameters;
    config.last_updated = chrono::Utc::now().to_rfc3339();
    
    Json(ApiResponse::success("Algorithm selected".to_string()))
}

// PUT /consensus/parameters
pub async fn update_parameters(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateParametersRequest>,
) -> Json<ApiResponse<String>> {
    let mut config = state.consensus_config.write().await;
    if config.current_algorithm == req.algorithm_id {
        config.parameters.insert(req.param_name, req.value);
        config.last_updated = chrono::Utc::now().to_rfc3339();
        Json(ApiResponse::success("Parameter updated".to_string()))
    } else {
        Json(ApiResponse::error(400, "Algorithm not active"))
    }
}

