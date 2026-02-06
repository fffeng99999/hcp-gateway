use crate::models::ApiResponse;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;

// GET /blocks/:height
pub async fn get_block(
    State(state): State<Arc<AppState>>,
    Path(height): Path<i64>,
) -> Json<ApiResponse<crate::grpc_client::block::Block>> {
    if let Some(client_ref) = &state.consensus_client {
        let mut client = client_ref.clone();
        match client.get_block(height).await {
            Ok(resp) => {
                if let Some(block) = resp.block {
                    Json(ApiResponse::success(block))
                } else {
                    Json(ApiResponse::error(404, "Block not found"))
                }
            }
            Err(e) => Json(ApiResponse::error(500, format!("gRPC error: {}", e))),
        }
    } else {
        Json(ApiResponse::error(503, "Consensus client not available"))
    }
}
