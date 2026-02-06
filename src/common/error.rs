use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use crate::models::ApiResponse;
use validator::ValidationErrors;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    InvalidInput(String),
    InternalError(String),
    ServiceUnavailable(String),
    AuthError(String),
    ValidationError(ValidationErrors),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, 404, msg),
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, 400, msg),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, 500, msg),
            AppError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, 503, msg),
            AppError::AuthError(msg) => (StatusCode::UNAUTHORIZED, 401, msg),
            AppError::ValidationError(errs) => (
                StatusCode::BAD_REQUEST, 
                400, 
                format!("Validation error: {}", errs)
            ),
        };

        let body = Json(ApiResponse::<()>::error(code, message));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<Json<ApiResponse<T>>, AppError>;
