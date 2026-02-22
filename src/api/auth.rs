use crate::api::extractors::ValidatedJson;
use crate::common::error::{AppError, AppResult};
use crate::common::state::AppState;
use crate::models::{ApiResponse, Claims, LoginRequest, LoginResponse, SystemUser};
use crate::services::server_client::auth::LoginRequest as AuthLoginRequest;
use crate::services::server_client::auth::User as AuthUser;
use crate::utils::auth::create_token;
use axum::{
    extract::{State},
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tonic::Code;
use tokio::time::timeout;
use uuid::Uuid;

pub async fn login(
    State(state): State<Arc<AppState>>,
    ValidatedJson(req): ValidatedJson<LoginRequest>,
) -> AppResult<LoginResponse> {
    let client = state
        .server_client
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("后端认证服务不可用".to_string()))?;
    let mut auth_client = client.auth_client.clone();

    let resp = timeout(
        Duration::from_secs(5),
        auth_client.login(AuthLoginRequest {
            username: req.username,
            password: req.password,
        }),
    )
    .await
    .map_err(|_| AppError::ServiceUnavailable("认证服务超时".to_string()))?
    .map_err(|e| match e.code() {
        Code::NotFound => AppError::NotFound("用户不存在".to_string()),
        Code::Unauthenticated => AppError::AuthError("用户名或密码错误".to_string()),
        Code::Unavailable => AppError::ServiceUnavailable("认证服务不可用".to_string()),
        _ => AppError::InternalError(format!("后端认证失败: {}", e.message())),
    })?;

    let pb_user = resp
        .into_inner()
        .user
        .ok_or_else(|| AppError::InternalError("Empty user response".to_string()))?;
    let user = map_auth_user(&pb_user);

    let mut users = state.users.write().await;
    if let Some(existing) = users.iter_mut().find(|u| u.username == user.username) {
        *existing = user.clone();
    } else {
        users.push(user.clone());
    }

    let token = create_token(&user)?;
    Ok(Json(ApiResponse::success(LoginResponse { token, user })))
}

fn map_auth_user(user: &AuthUser) -> SystemUser {
    let created_at = if user.created_at.is_empty() {
        chrono::Utc::now().to_rfc3339()
    } else {
        user.created_at.clone()
    };
    let status = if user.status.is_empty() {
        "active".to_string()
    } else {
        user.status.clone()
    };
    let role = if user.role.is_empty() {
        "user".to_string()
    } else {
        user.role.clone()
    };
    let last_login = if user.last_login.is_empty() {
        None
    } else {
        Some(user.last_login.clone())
    };

    SystemUser {
        id: if user.id.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            user.id.clone()
        },
        username: user.username.clone(),
        role,
        email: user.email.clone(),
        created_at,
        status,
        last_login,
    }
}

pub async fn me(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
) -> AppResult<SystemUser> {
    let users = state.users.read().await;
    if let Some(user) = users.iter().find(|u| u.username == claims.sub) {
        Ok(Json(ApiResponse::success(user.clone())))
    } else {
        Err(AppError::NotFound("User not found".to_string()))
    }
}

#[derive(Deserialize)]
pub struct ProfileUpdateRequest {
    pub username: Option<String>,
    pub email: Option<String>,
}

pub async fn update_profile(
    Extension(claims): Extension<Claims>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ProfileUpdateRequest>,
) -> AppResult<LoginResponse> {
    let mut users = state.users.write().await;
    let original_id = {
        let user = users
            .iter()
            .find(|u| u.username == claims.sub)
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        user.id.clone()
    };

    if let Some(username) = payload.username.clone() {
        let trimmed = username.trim();
        if trimmed.is_empty() {
            return Err(AppError::InvalidInput("用户名不能为空".to_string()));
        }
        if trimmed.len() < 3 {
            return Err(AppError::InvalidInput(
                "用户名长度至少为 3 个字符".to_string(),
            ));
        }
        if users
            .iter()
            .any(|u| u.username == trimmed && u.id != original_id)
        {
            return Err(AppError::InvalidInput(
                "用户名已被其他用户占用".to_string(),
            ));
        }
        if let Some(user) = users.iter_mut().find(|u| u.id == original_id) {
            user.username = trimmed.to_string();
        }
    }

    if let Some(email) = payload.email.clone() {
        let trimmed = email.trim();
        if trimmed.is_empty() || !trimmed.contains('@') {
            return Err(AppError::InvalidInput("邮箱格式不正确".to_string()));
        }
        if let Some(user) = users.iter_mut().find(|u| u.id == original_id) {
            user.email = trimmed.to_string();
        }
    }

    let updated_user = users
        .iter()
        .find(|u| u.id == original_id)
        .cloned()
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
    let token = create_token(&updated_user)?;

    Ok(Json(ApiResponse::success(LoginResponse {
        token,
        user: updated_user,
    })))
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub async fn change_password(
    Extension(_claims): Extension<Claims>,
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<ChangePasswordRequest>,
) -> AppResult<()> {
    if payload.new_password.len() < 6 {
        return Err(AppError::InvalidInput(
            "新密码长度至少为 6 位".to_string(),
        ));
    }

    Ok(Json(ApiResponse::success(())))
}
