use axum::Json;
use serde_json::json;

pub async fn health_check() -> Json<serde_json::Value> {
    tracing::info!("Health check requested");
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
