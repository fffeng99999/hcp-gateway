use crate::common::state::AppState;
use crate::models::{AntiManipulationConfig, ApiResponse, ManipulationEvent};
use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;

// GET /anti-manipulation/config
pub async fn get_config(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<AntiManipulationConfig>> {
    let config = state.anti_manipulation_config.read().await;
    Json(ApiResponse::success(config.clone()))
}

// PUT /anti-manipulation/config
pub async fn update_config(
    State(state): State<Arc<AppState>>,
    Json(new_config): Json<AntiManipulationConfig>,
) -> Json<ApiResponse<String>> {
    let mut config = state.anti_manipulation_config.write().await;
    *config = new_config;
    Json(ApiResponse::success("Configuration updated".to_string()))
}

// GET /anti-manipulation/events
pub async fn get_events(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<ManipulationEvent>>> {
    let events = state.manipulation_events.read().await;
    Json(ApiResponse::success(events.clone()))
}

// GET /anti-manipulation/events/:id
pub async fn get_event_details(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<ManipulationEvent>> {
    let events = state.manipulation_events.read().await;
    if let Some(event) = events.iter().find(|e| e.id == id) {
        Json(ApiResponse::success(event.clone()))
    } else {
        Json(ApiResponse::error(404, "Event not found"))
    }
}
