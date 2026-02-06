use crate::models::{ApiResponse, Transaction, TransactionSubmitRequest, TransactionQueryParams};
use crate::common::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use std::sync::Arc;
use base64::prelude::*;

// POST /transactions/submit
pub async fn submit_transaction(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TransactionSubmitRequest>,
) -> Json<ApiResponse<String>> {
    if let Some(client_ref) = &state.consensus_client {
        let mut client = client_ref.clone();
        
        let tx_bytes = if let Some(b64) = req.payload.get("tx_bytes").and_then(|v| v.as_str()) {
            match BASE64_STANDARD.decode(b64) {
                Ok(b) => b,
                Err(_) => return Json(ApiResponse::error(400, "Invalid base64 tx_bytes")),
            }
        } else {
            // Fallback: If payload is simple JSON, maybe serialize it? 
            // For now, require tx_bytes to be correct.
            return Json(ApiResponse::error(400, "Payload must contain 'tx_bytes' (base64 encoded Cosmos Tx)"));
        };

        match client.broadcast_tx(tx_bytes).await {
            Ok(resp) => {
                if let Some(tx_resp) = resp.tx_response {
                    if tx_resp.code == 0 {
                         Json(ApiResponse::success(format!("Transaction submitted. Hash: {}", tx_resp.txhash)))
                    } else {
                         Json(ApiResponse::error(400, format!("CheckTx failed: Code {}, Log: {}", tx_resp.code, tx_resp.raw_log)))
                    }
                } else {
                     Json(ApiResponse::error(500, "Empty response from consensus"))
                }
            }
            Err(e) => Json(ApiResponse::error(500, format!("gRPC error: {}", e))),
        }
    } else {
        Json(ApiResponse::error(503, "Consensus client not available"))
    }
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
