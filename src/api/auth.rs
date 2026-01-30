use axum::{Json, extract::State};
use crate::models::{LoginRequest, LoginResponse, ApiResponse, SystemUser};
use crate::auth::create_token;
use crate::error::{AppError, AppResult};
use crate::extractors::ValidatedJson;
use std::sync::Arc;
use crate::state::AppState;

pub async fn login(
    State(_state): State<Arc<AppState>>,
    ValidatedJson(req): ValidatedJson<LoginRequest>,
) -> AppResult<LoginResponse> {
    // Mock user verification
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
        
        Ok(Json(ApiResponse::success(LoginResponse {
            token,
            user,
        })))
    } else {
        Err(AppError::AuthError("Invalid credentials".to_string()))
    }
}
