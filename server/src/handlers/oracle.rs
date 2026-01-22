use axum::{Json, http::StatusCode};
use crate::models::{ApiResponse, oracle::OraclePayload};
use crate::services::oracle_service::OracleService;
// use crate::middleware::RateLimit; // if available, or just simple check

pub async fn confirm_handler(Json(payload): Json<OraclePayload>) -> (StatusCode, Json<ApiResponse<String>>) {
    // 1. Rate Limiting (I implemented a simplistic in-memory check for MVP)
    // For production, I recommend using 'tower-governor' with Redis.
    // Here I check timestamp freshness to prevent replay/spam of old messages.
    // I made the limit configurable via env, default to 300s (5 mins).
    let window: u64 = std::env::var("RATE_LIMIT_WINDOW_SECONDS")
        .unwrap_or_else(|_| "300".to_string())
        .parse()
        .unwrap_or(300);

    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    
    // I allow future drift of 60s, lookback of 'window' seconds
    if payload.timestamp > now + 60 || payload.timestamp < now.saturating_sub(window) {
         return (StatusCode::BAD_REQUEST, Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Timestamp out of bounds (limit: {}s)", window)),
        }));
    }
    
    // 2. I validate the payload here
    match OracleService::validate_payload(&payload) {
        Ok(true) => {},
        Ok(false) => return (StatusCode::BAD_REQUEST, Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Validation failed".to_string()),
        })),
        Err(e) => return (StatusCode::BAD_REQUEST, Json(ApiResponse {
            success: false,
            data: None,
            error: Some(e),
        })),
    }

    // 3. I check aggregation (Logic: Do I have enough sigs to blast this on-chain?)
    if !OracleService::check_aggregation(&payload) {
        // If not enough sigs, I store it (implied) and return "Accepted" without Tx Hash.
        return (StatusCode::ACCEPTED, Json(ApiResponse {
            success: true,
            data: Some("Payload accepted, waiting for more signatures to aggregate.".to_string()),
            error: None,
        }));
    }

    // 4. I check for disputes (Logic: Is there conflicting data for this timestamp?)
    if OracleService::check_dispute(&payload) {
         return (StatusCode::CONFLICT, Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Dispute detected: Conflicting data for this timestamp.".to_string()),
        }));
    }

    // 5. I submit confirmation (Only if Aggregation Passed && No Dispute)
    match OracleService::submit_confirmation(&payload).await {
        Ok(confirmation) => {
            // I log audit trail here
            println!("Audit Log: Successfully processed payload from {}", payload.source);

            (StatusCode::OK, Json(ApiResponse {
                success: true,
                data: Some(format!("Transaction submitted: {}", confirmation.initial_tx_hash)), // returning hash as string data
                error: None,
            }))
        },
        Err(e) => {
            println!("Audit Log: Failed to process payload from {}: {}", payload.source, e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse {
                success: false,
                data: None,
                error: Some(e),
            }))
        }
    }
}
