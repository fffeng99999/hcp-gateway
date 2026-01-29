use crate::error::{ApiError, ApiResult};
use crate::state::AppState;
use axum::{extract::Path, extract::State, Json};
use serde_json::{json, Value};
use std::sync::Arc;

// POST /transaction/submit
pub async fn submit_transaction(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> ApiResult<Json<Value>> {
    let tx_id = format!("tx_{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase());
    let mut transactions = state.transactions.write().await;

    let transaction = json!({
        "id": tx_id,
        "status": "pending",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "payload": payload,
    });

    transactions.push(transaction.clone());

    Ok(Json(json!({
        "tx_id": tx_id,
        "status": "submitted",
    })))
}

// GET /transaction/:id
pub async fn get_transaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let transactions = state.transactions.read().await;
    transactions
        .iter()
        .find(|t| t.get("id").and_then(|v| v.as_str()) == Some(&id))
        .cloned()
        .map(Json)
        .ok_or_else(|| ApiError::NotFound(format!("Transaction {} not found", id)))
}

// GET /transaction/status
pub async fn get_transaction_status(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Value>> {
    let transactions = state.transactions.read().await;

    let total = transactions.len();
    let success = transactions
        .iter()
        .filter(|t| t.get("status").and_then(|v| v.as_str()) == Some("success"))
        .count();
    let pending = transactions
        .iter()
        .filter(|t| t.get("status").and_then(|v| v.as_str()) == Some("pending"))
        .count();

    Ok(Json(json!({
        "total": total,
        "success": success,
        "pending": pending,
        "success_rate": if total > 0 {
            (success as f64) / (total as f64)
        } else {
            0.0
        },
    })))
}

// GET /transaction/history
pub async fn get_transaction_history(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<Vec<Value>>> {
    let transactions = state.transactions.read().await;
    Ok(Json(transactions.clone()))
}
