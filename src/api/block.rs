use crate::models::ApiResponse;
use crate::common::state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;

// GET /blocks/:height
pub async fn get_block(
    State(state): State<Arc<AppState>>,
    Path(height): Path<i64>,
) -> Json<ApiResponse<serde_json::Value>> {
    if let Some(client_ref) = &state.consensus_client {
        let mut client = client_ref.clone();
        match client.get_block(height).await {
            Ok(resp) => {
                let json = serde_json::to_value(&resp).unwrap_or(serde_json::Value::Null);
                Json(ApiResponse::success(json))
            }
            Err(e) => Json(ApiResponse::error(500, format!("gRPC error: {}", e))),
        }
    } else {
        Json(ApiResponse::error(503, "Consensus client not available"))
    }
}
