// 用户管理相关接口
use super::common::{read_cache, require_version, respond_with_version, write_cache, DB_CACHE_TTL};
use crate::common::state::AppState;
use crate::models::{ApiResponse, SystemUser};
use axum::{extract::Path, extract::State, http::HeaderMap, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// 获取用户列表
pub async fn get_users(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    if let Some(cached) = read_cache(&state.users_cache, DB_CACHE_TTL).await {
        return respond_with_version(&state, ApiResponse::success(cached), false);
    }
    let users = state.users.read().await.clone();
    write_cache(&state.users_cache, users.clone()).await;
    respond_with_version(&state, ApiResponse::success(users), false)
}

// 创建用户请求体
#[derive(Deserialize)]
pub struct UserCreateRequest {
    pub id: Option<String>,
    pub username: Option<String>,
    pub role: Option<String>,
    pub email: Option<String>,
    pub status: Option<String>,
}

// 创建用户
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<UserCreateRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_version(&headers, &state) {
        return respond_with_version(&state, resp, false);
    }
    let mut users = state.users.write().await;
    let username = payload.username.unwrap_or_default();
    let email = payload.email.unwrap_or_default();

    if username.trim().is_empty() {
        return respond_with_version(
            &state,
            ApiResponse::error(400, "用户名不能为空".to_string()),
            false,
        );
    }
    if !email.contains('@') {
        return respond_with_version(
            &state,
            ApiResponse::error(400, "邮箱格式不正确".to_string()),
            false,
        );
    }

    let user = SystemUser {
        id: payload.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        username,
        role: payload.role.unwrap_or_else(|| "观察者".to_string()),
        email,
        created_at: chrono::Utc::now().to_rfc3339(),
        status: payload.status.unwrap_or_else(|| "正常".to_string()),
        last_login: None,
    };

    users.push(user);
    write_cache(&state.users_cache, users.clone()).await;
    respond_with_version(
        &state,
        ApiResponse::success("User created".to_string()),
        true,
    )
}

// 更新用户请求体
#[derive(Deserialize)]
pub struct UserUpdateRequest {
    pub username: Option<String>,
    pub role: Option<String>,
    pub email: Option<String>,
    pub status: Option<String>,
}

// 更新用户信息
pub async fn update_user(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(user_update): Json<UserUpdateRequest>,
) -> impl IntoResponse {
    if let Err(resp) = require_version(&headers, &state) {
        return respond_with_version(&state, resp, false);
    }
    let mut users = state.users.write().await;

    if let Some(user) = users.iter_mut().find(|u| u.id == id) {
        if let Some(username) = user_update.username {
            if username.trim().is_empty() {
                return respond_with_version(
                    &state,
                    ApiResponse::error(400, "用户名不能为空".to_string()),
                    false,
                );
            }
            user.username = username;
        }
        if let Some(role) = user_update.role {
            user.role = role;
        }
        if let Some(email) = user_update.email {
            if !email.contains('@') {
                return respond_with_version(
                    &state,
                    ApiResponse::error(400, "邮箱格式不正确".to_string()),
                    false,
                );
            }
            user.email = email;
        }
        if let Some(status) = user_update.status {
            user.status = status;
        }

        write_cache(&state.users_cache, users.clone()).await;
        respond_with_version(
            &state,
            ApiResponse::success("User updated".to_string()),
            true,
        )
    } else {
        respond_with_version(&state, ApiResponse::error(404, "User not found"), false)
    }
}

// 删除用户
pub async fn delete_user(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(resp) = require_version(&headers, &state) {
        return respond_with_version(&state, resp, false);
    }
    let mut users = state.users.write().await;
    let initial_len = users.len();
    users.retain(|u| u.id != id);

    if users.len() < initial_len {
        write_cache(&state.users_cache, users.clone()).await;
        respond_with_version(
            &state,
            ApiResponse::success("User deleted".to_string()),
            true,
        )
    } else {
        respond_with_version(&state, ApiResponse::error(404, "User not found"), false)
    }
}

// 用户校验请求体
#[derive(Deserialize)]
pub struct UserValidateRequest {
    pub id: Option<String>,
    pub username: String,
    pub email: String,
    pub role: Option<String>,
}

// 单个字段错误
#[derive(Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

// 用户校验响应体
#[derive(Serialize)]
pub struct UserValidateResponse {
    pub valid: bool,
    pub errors: Vec<FieldError>,
}

// 校验用户字段
pub async fn validate_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UserValidateRequest>,
) -> impl IntoResponse {
    let mut errors: Vec<FieldError> = Vec::new();

    if payload.username.trim().is_empty() {
        errors.push(FieldError {
            field: "username".to_string(),
            message: "用户名不能为空".to_string(),
        });
    }

    if !payload.email.contains('@') {
        errors.push(FieldError {
            field: "email".to_string(),
            message: "邮箱格式不正确".to_string(),
        });
    }

    let users = state.users.read().await;
    for user in users.iter() {
        if let Some(ref id) = payload.id {
            if &user.id == id {
                continue;
            }
        }

        if user.username == payload.username {
            errors.push(FieldError {
                field: "username".to_string(),
                message: "用户名已存在".to_string(),
            });
            break;
        }
    }

    let is_valid = errors.is_empty();
    let response = UserValidateResponse {
        valid: is_valid,
        errors,
    };

    respond_with_version(&state, ApiResponse::success(response), false)
}

// 重置用户密码
pub async fn reset_user_password(
    Path(_id): Path<String>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if let Err(resp) = require_version(&headers, &state) {
        return respond_with_version(&state, resp, false);
    }
    respond_with_version(
        &state,
        ApiResponse::success("User password reset".to_string()),
        true,
    )
}
