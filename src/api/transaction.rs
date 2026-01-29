use crate::models::{ApiResponse, Transaction, TransactionSubmitRequest, TransactionQueryParams};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use std::sync::Arc;

// POST /transactions/submit
pub async fn submit_transaction(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TransactionSubmitRequest>,
) -> Json<ApiResponse<String>> {
    let mut transactions = state.transactions.write().await;
    
    // Simulate batch submission
    let batch_size = req.batch_size.unwrap_or(1);
    let mut last_id = String::new();
    
    for _ in 0..batch_size {
        let id = uuid::Uuid::new_v4().to_string();
        last_id = id.clone();
        
        transactions.push(Transaction {
            id,
            from: "user_wallet".to_string(),
            to: "contract_address".to_string(),
            amount: 0.0,
            status: "pending".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            block_height: None,
        });
    }
    
    Json(ApiResponse::success(format!("Submitted {} transactions. Last ID: {}", batch_size, last_id)))
}

// GET /transactions/stats
pub async fn get_transaction_stats(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<serde_json::Value>> {
    let transactions = state.transactions.read().await;
    let total = transactions.len();
    let pending = transactions.iter().filter(|t| t.status == "pending").count();
    let confirmed = transactions.iter().filter(|t| t.status == "confirmed").count();
    let failed = transactions.iter().filter(|t| t.status == "failed").count();
    
    let stats = serde_json::json!({
        "total": total,
        "pending": pending,
        "confirmed": confirmed,
        "failed": failed,
        "pool_size": pending, // Assuming pool size is pending count
        "avg_confirmation_time_ms": 2500 // Mock
    });
    
    Json(ApiResponse::success(stats))
}

// GET /transactions/pending
pub async fn get_pending_transactions(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<Transaction>>> {
    let transactions = state.transactions.read().await;
    let pending: Vec<Transaction> = transactions.iter()
        .filter(|t| t.status == "pending")
        .cloned()
        .collect();
    Json(ApiResponse::success(pending))
}

// GET /transactions/confirmed
pub async fn get_confirmed_transactions(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<Transaction>>> {
    let transactions = state.transactions.read().await;
    let confirmed: Vec<Transaction> = transactions.iter()
        .filter(|t| t.status == "confirmed")
        .cloned()
        .collect();
    Json(ApiResponse::success(confirmed))
}

// GET /transactions/query
pub async fn query_transactions(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TransactionQueryParams>,
) -> Json<ApiResponse<Vec<Transaction>>> {
    let transactions = state.transactions.read().await;
    
    let filtered: Vec<Transaction> = transactions.iter()
        .filter(|t| {
            if let Some(ref status) = params.status {
                if &t.status != status { return false; }
            }
            if let Some(ref from) = params.from {
                if &t.from != from { return false; }
            }
            if let Some(ref to) = params.to {
                if &t.to != to { return false; }
            }
            true
        })
        .skip(params.offset.unwrap_or(0))
        .take(params.limit.unwrap_or(10))
        .cloned()
        .collect();
        
    Json(ApiResponse::success(filtered))
}

// GET /transactions/history (Alias for query)
pub async fn get_transaction_history(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TransactionQueryParams>,
) -> Json<ApiResponse<Vec<Transaction>>> {
    query_transactions(State(state), Query(params)).await
}

// GET /transactions/status
pub async fn get_transaction_status(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<serde_json::Value>> {
    get_transaction_stats(State(state)).await
}

// GET /transactions/:id
pub async fn get_transaction(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Transaction>> {
    let transactions = state.transactions.read().await;
    if let Some(tx) = transactions.iter().find(|t| t.id == id) {
        Json(ApiResponse::success(tx.clone()))
    } else {
        Json(ApiResponse::error(404, "Transaction not found"))
    }
}

// POST /transactions/:id/cancel
pub async fn cancel_transaction(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<String>> {
    let mut transactions = state.transactions.write().await;
    if let Some(tx) = transactions.iter_mut().find(|t| t.id == id) {
        if tx.status == "pending" {
            tx.status = "cancelled".to_string();
            Json(ApiResponse::success("Transaction cancelled".to_string()))
        } else {
            Json(ApiResponse::error(400, "Cannot cancel non-pending transaction"))
        }
    } else {
        Json(ApiResponse::error(404, "Transaction not found"))
    }
}
