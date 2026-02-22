use crate::api::extractors::ValidatedJson;
use crate::common::error::{AppError, AppResult};
use crate::common::state::AppState;
use crate::models::{ApiResponse, LoginRequest, LoginResponse, SystemUser};
use crate::utils::auth::create_token;
use axum::{extract::State, Json};
use std::sync::Arc;

pub async fn login(
    State(_state): State<Arc<AppState>>,
    ValidatedJson(req): ValidatedJson<LoginRequest>,
) -> AppResult<LoginResponse> {
    // 使用内置账号进行模拟用户校验
    if req.username == "admin" && req.password == "admin123" {
        let user = SystemUser {
            id: "1".to_string(),
            username: "admin".to_string(),
            email: "admin@example.com".to_string(),
            role: "admin".to_string(),
            status: "active".to_string(),
            last_login: Some(chrono::Utc::now().to_rfc3339()),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let token = create_token(&user)?;

        Ok(Json(ApiResponse::success(LoginResponse { token, user })))
    } else {
        Err(AppError::AuthError("Invalid credentials".to_string()))
    }
}
