use crate::api::extractors::ValidatedJson;
use crate::common::error::{AppError, AppResult};
use crate::common::state::AppState;
use crate::models::{
    ApiResponse, ConsensusAlgorithm, ConsensusConfig, SelectAlgorithmRequest,
    UpdateParametersRequest,
};
use axum::{extract::State, Json};
use std::sync::Arc;

// GET /consensus/algorithms 获取当前可用的共识算法列表
pub async fn get_algorithms(
    State(state): State<Arc<AppState>>,
) -> AppResult<Vec<ConsensusAlgorithm>> {
    let algorithms = state.algorithms.read().await;
    Ok(Json(ApiResponse::success(algorithms.clone())))
}

// GET /consensus/config 获取当前共识配置
pub async fn get_config(State(state): State<Arc<AppState>>) -> AppResult<ConsensusConfig> {
    let config = state.consensus_config.read().await;
    Ok(Json(ApiResponse::success(config.clone())))
}

// POST /consensus/select 选择并切换当前使用的共识算法
pub async fn select_algorithm(
    State(state): State<Arc<AppState>>,
    ValidatedJson(req): ValidatedJson<SelectAlgorithmRequest>,
) -> AppResult<String> {
    let mut config = state.consensus_config.write().await;

    // 校验请求中的算法 ID 是否存在
    let algorithms = state.algorithms.read().await;
    if !algorithms.iter().any(|a| a.id == req.algorithm_id) {
        return Err(AppError::InvalidInput(format!(
            "Algorithm {} not found",
            req.algorithm_id
        )));
    }

    config.current_algorithm = req.algorithm_id;
    config.parameters = req.parameters;
    config.last_updated = chrono::Utc::now().to_rfc3339();

    Ok(Json(ApiResponse::success("Algorithm selected".to_string())))
}

// PUT /consensus/parameters 更新当前共识算法的特定参数
pub async fn update_parameters(
    State(state): State<Arc<AppState>>,
    ValidatedJson(req): ValidatedJson<UpdateParametersRequest>,
) -> AppResult<String> {
    let mut config = state.consensus_config.write().await;

    if config.current_algorithm == req.algorithm_id {
        config.parameters.insert(req.param_name, req.value);
        config.last_updated = chrono::Utc::now().to_rfc3339();
        Ok(Json(ApiResponse::success("Parameter updated".to_string())))
    } else {
        Err(AppError::InvalidInput("Algorithm not active".to_string()))
    }
}
