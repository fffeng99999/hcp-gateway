use axum::middleware::Next;
use axum::response::Response;
use tower::ServiceBuilder;
use tracing::{info, warn};

/// Request logging middleware
pub async fn logging_middleware(
    req: axum::extract::Request,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start = std::time::Instant::now();

    let response = next.run(req).await;

    let duration = start.elapsed();
    let status = response.status();

    if status.is_success() {
        info!(
            "Request completed: {} {} - {} in {:.2}ms",
            method,
            uri,
            status,
            duration.as_secs_f64() * 1000.0
        );
    } else {
        warn!(
            "Request error: {} {} - {} in {:.2}ms",
            method,
            uri,
            status,
            duration.as_secs_f64() * 1000.0
        );
    }

    response
}

/// Rate limiting middleware (placeholder for implementation)
pub struct RateLimiter {
    max_requests_per_second: u32,
}

impl RateLimiter {
    pub fn new(max_requests_per_second: u32) -> Self {
        RateLimiter {
            max_requests_per_second,
        }
    }
}

/// Error handling utilities
pub mod error_handler {
    use axum::response::{IntoResponse, Response};
    use axum::http::StatusCode;
    use axum::Json;
    use serde_json::json;

    pub fn handle_not_found() -> Response {
        (StatusCode::NOT_FOUND, Json(json!({
            "error": "Resource not found",
            "status": 404
        }))).into_response()
    }

    pub fn handle_internal_error(msg: &str) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "error": msg,
            "status": 500
        }))).into_response()
    }
}
