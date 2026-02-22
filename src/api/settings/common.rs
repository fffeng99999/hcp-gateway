// 设置模块通用逻辑与工具函数
use crate::common::state::{AppState, SettingsCache};
use crate::models::ApiResponse;
use axum::http::HeaderMap;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path as FsPath;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// 统一封装配置相关接口响应，附带版本号头部
pub(super) fn respond_with_version<T>(
    state: &Arc<AppState>,
    body: ApiResponse<T>,
    bump: bool,
) -> (HeaderMap, Json<ApiResponse<T>>)
where
    T: Serialize,
{
    let version = if bump {
        state.config_version.fetch_add(1, Ordering::SeqCst) + 1
    } else {
        state.config_version.load(Ordering::SeqCst)
    };

    let mut headers = HeaderMap::new();
    headers.insert("X-Config-Version", version.to_string().parse().unwrap());
    (headers, Json(body))
}

// 将部分更新字段合并到当前设置
pub(super) fn merge_settings<T>(current: &T, patch: Value) -> Result<T, String>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
{
    let mut base = serde_json::to_value(current).map_err(|e| e.to_string())?;
    merge_json_value(&mut base, &patch);
    serde_json::from_value(base).map_err(|e| e.to_string())
}

// 深度合并 JSON 对象
fn merge_json_value(base: &mut Value, patch: &Value) {
    match (base, patch) {
        (Value::Object(base_map), Value::Object(patch_map)) => {
            for (key, patch_value) in patch_map {
                match base_map.get_mut(key) {
                    Some(base_value) => {
                        merge_json_value(base_value, patch_value);
                    }
                    None => {
                        base_map.insert(key.clone(), patch_value.clone());
                    }
                }
            }
        }
        (base_value, patch_value) => {
            *base_value = patch_value.clone();
        }
    }
}

// 配置类缓存有效期
pub(super) const CONFIG_CACHE_TTL: Duration = Duration::from_secs(10);
// 列表类缓存有效期
pub(super) const DB_CACHE_TTL: Duration = Duration::from_secs(30);

// 读取缓存并校验 TTL
pub(super) async fn read_cache<T: Clone>(
    cache: &Arc<RwLock<SettingsCache<T>>>,
    ttl: Duration,
) -> Option<T> {
    let cache_guard = cache.read().await;
    match (&cache_guard.value, cache_guard.updated_at) {
        (Some(value), Some(updated_at)) if Instant::now().duration_since(updated_at) <= ttl => {
            Some(value.clone())
        }
        _ => None,
    }
}

// 写入缓存并更新时间
pub(super) async fn write_cache<T: Clone>(cache: &Arc<RwLock<SettingsCache<T>>>, value: T) {
    let mut cache_guard = cache.write().await;
    cache_guard.value = Some(value);
    cache_guard.updated_at = Some(Instant::now());
}

// 校验请求头中的配置版本号
pub(super) fn require_version(
    headers: &HeaderMap,
    state: &Arc<AppState>,
) -> Result<(), ApiResponse<String>> {
    let current = state.config_version.load(Ordering::SeqCst);
    let header = headers
        .get("x-config-version")
        .ok_or_else(|| ApiResponse::error(428, "缺少配置版本号".to_string()))?;
    let header_str = header
        .to_str()
        .map_err(|_| ApiResponse::error(400, "配置版本号格式错误".to_string()))?;
    let client_version = header_str
        .parse::<u64>()
        .map_err(|_| ApiResponse::error(400, "配置版本号格式错误".to_string()))?;
    if client_version != current {
        return Err(ApiResponse::error(409, "配置版本已变更".to_string()));
    }
    Ok(())
}

// 校验路径是否为合法绝对目录路径
pub(super) fn validate_path_internal(raw: &str) -> Result<serde_json::Value, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("备份路径不能为空".to_string());
    }

    let path = FsPath::new(trimmed);

    if !path.is_absolute() {
        return Err("备份路径必须为绝对路径".to_string());
    }

    let is_valid = if path.exists() {
        path.is_dir()
    } else if let Some(parent) = path.parent() {
        parent.exists() && parent.is_dir()
    } else {
        false
    };

    if !is_valid {
        return Err("备份路径不是有效的目录或目录不可用".to_string());
    }

    let normalized = path.to_string_lossy().to_string();
    Ok(json!({
        "is_valid": true,
        "normalized_path": normalized,
    }))
}
