use axum::{
    extract::{Path, State},
    Json,
};
use crate::{
    error::{ApiError, ApiResult},
    models::*,
    state::AppState,
};
use uuid::Uuid;

pub async fn submit_transaction(
    State(state): State<AppState>,
    Json(payload): Json<SubmitTransactionRequest>,
) -> ApiResult<Json<serde_json::json!>> {
    tracing::info!("Submitting transaction");
    
    let tx_id = Uuid::new_v4().to_string();
    let transaction = Transaction {
        id: tx_id.clone(),
        status: "pending".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        payload,
        result: None,
    };
    
    let mut transactions = state.transactions.write().await;
    transactions.insert(tx_id.clone(), transaction);
    
    Ok(Json(serde_json::json!({
        "tx_id": tx_id,
        "status": "submitted",
    })))
}

pub async fn get_transaction(
    State(state): State<AppState>,
    Path(tx_id): Path<String>,
) -> ApiResult<Json<Transaction>> {
    tracing::info!("Getting transaction: {}", tx_id);
    
    let transactions = state.transactions.read().await;
    transactions
        .get(&tx_id)
        .cloned()
        .map(Json)
        .ok_or_else(|| ApiError::NotFound(format!("Transaction {} not found", tx_id)))
}

pub async fn get_transaction_status(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<Transaction>>> {
    tracing::info!("Getting transaction status");
    
    let transactions = state.transactions.read().await;
    let txs: Vec<Transaction> = transactions.values().cloned().collect();
    
    Ok(Json(txs))
}

pub async fn get_transaction_history(
    State(state): State<AppState>,
) -> ApiResult<Json<Vec<Transaction>>> {
    tracing::info!("Getting transaction history");
    
    let transactions = state.transactions.read().await;
    let mut txs: Vec<Transaction> = transactions.values().cloned().collect();
    txs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    Ok(Json(txs))
}
